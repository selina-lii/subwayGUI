// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::anyhow;
use anyhow::Error as Err;
use glob::glob;
use serde::{ser::Error, Deserialize, Serialize};
use serde_json::error::Error as SerdeError;
use serde_json::Map;
use std::collections::HashMap;
use std::string::String;
use std::thread::panicking;
use std::{fs, path::Path};
use tauri::InvokeError;
use walkdir::DirEntry;
use walkdir::WalkDir;

fn filter(entry: &DirEntry, pattern: &str) -> bool {
    let is_match = entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(pattern))
        .unwrap_or(false);
    // Debug print the file name and whether it matches the pattern
    println!("Checking: {:?}, Match: {}", entry.path(), is_match);

    is_match
}

#[derive(Serialize, Deserialize, Debug)]
struct Input {
    patterns: Vec<Pattern>,
    folders: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Pattern {
    filename: String,
    status: String,
}

fn parse_input_data(json_str: &str) -> Result<Input, SerdeError> {
    serde_json::from_str(json_str)
}
#[derive(Debug)]
struct SerializableError {
    message: String,
}

impl From<&str> for SerializableError {
    fn from(error: &str) -> Self {
        SerializableError {
            message: error.to_string(),
        }
    }
}

impl From<SerdeError> for SerializableError {
    fn from(error: SerdeError) -> Self {
        SerializableError {
            message: error.to_string(),
        }
    }
}

// Implementing `Into<InvokeError>` for `SerializableError`
impl Into<InvokeError> for SerializableError {
    fn into(self) -> InvokeError {
        InvokeError::from(self.message)
    }
}

/// Constructs a full filename (with extension) from a `Path`, if possible.
/// Returns `None` if any part of the filename is not valid unicode.
fn construct_full_filename(path: &Path) -> Option<String> {
    let file_stem = path.file_stem()?.to_str()?;
    let extension = path.extension()?.to_str()?;

    Some(format!("{}.{}", file_stem, extension))
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn fileMatch(directory: &str) -> Result<String, SerializableError> {
    // Specify the folder path and the pattern
    //TODO Mach a list of file patterns

    let inputData: Input = parse_input_data(directory)?;
    let mut res: HashMap<String, Vec<Pattern>> = HashMap::new();

    for folder in inputData.folders {
        // match file for each folder
        let mut temp: Vec<Pattern> = Vec::new();

        for pattern in inputData.patterns.clone() {
            println!("{:?}", pattern);

            // let walker = WalkDir::new(folder.clone())
            //     .into_iter()
            //     .filter_entry(|e| filter(e, &pattern.ext));

            let mut found = 0;
            // for entry in walker.filter_map(Result::ok) {
            //     if let Some(filename) = entry.path().to_str() {
            //         temp.push(Pattern { ext: filename.to_string(), status: "found".to_string() });
            //         found = true;
            //         println!("Found: {}", entry.path().display());
            //     }
            // }
            for entry in WalkDir::new(folder.clone())
                .into_iter()
                .filter_map(Result::ok) // Handle potential errors
                .filter(|e| e.file_type().is_file())
            // Make sure it's a file
            {
                let path = entry.path();
                // println!("{:?}", path.file_name());

                if let Some(full_filename) = construct_full_filename(path) {
                    if full_filename.ends_with(&pattern.filename) {
                        println!(
                            "Found valid file: {} for pattern: {}",
                            entry.path().display(),
                            &pattern.filename
                        );
                        temp.push(Pattern {
                            filename: entry.path().display().to_string(),
                            status: "valid".to_string(),
                        });
                        found += 1;
                    }
                }
            }

            // for entry in WalkDir::new(&folder).into_iter().filter_map(Result::ok) {
            //     println!("Entry: {:?}", entry.path());
            // }

            if found == 0 {
                println!("File missing for pattern: {}", &pattern.filename);
                temp.push(Pattern {
                    filename: pattern.filename,
                    status: "missing".to_string(),
                });
            }
        }
        res.insert(folder.clone(), temp);
    }
    serde_json::to_string(&res).map_err(|e| {
        println!("Serialization error: {}", e);
        SerializableError::from("Failed to serialize response")
    })
}

#[tauri::command]
fn save_file(path: String, contents: String) {
  fs::write(path, contents).unwrap();
}

#[derive(Debug, Deserialize)]
struct Node {
    id: String,
    node_type: String,
    node_label: String,
    node_data: NodeData,
}

#[derive(Debug, Deserialize)]
struct JsonData {
    nodes: Vec<Node>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FilePattern {
    id: String,
    node_data: NodeData,
    node_label: String,
    node_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct NodeData {
    extension: String,
    #[serde(default)]
    prefix: String,
}

fn check_file_patterns(directory: String, extensions: Vec<&str>) -> Vec<bool> {
    let mut results = vec![false; extensions.len()];

    for entry in WalkDir::new(directory).into_iter().filter_map(Result::ok) {
        if let Some(file_name) = entry.file_name().to_str() {
            for (i, &ext) in extensions.iter().enumerate() {
                if file_name.ends_with(ext) {
                    results[i] = true;
                    break;
                }
            }
        }
    }

    results
}

#[tauri::command]
fn find_matches(targetPath: String, schemaPath: String) -> Vec<bool> {
    let data = fs::read_to_string(schemaPath).expect("Unable to read file");
    let json_object: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    let mut extensions = vec![]; 
    println!("test {} ", json_object["nodes"]);
    if let Some(nodes) = json_object["nodes"].as_array() {
      for node in nodes {
        println!("node {}", node["node_data"]["extension"]);
        extensions.push(node["node_data"]["extension"].as_str().expect("test"))
    }
    }
    let results = check_file_patterns(targetPath, extensions);
    results
}


#[cfg(test)]
mod tests {
    use super::*; // Import the functions from the outer module

    #[test] // Use this for async tests, requires the `tokio` crate. Otherwise, just use #[test] for synchronous tests.
    fn test_file_match() {
        println!(
            "Current working directory: {:?}",
            std::env::current_dir().unwrap()
        );

        // Setup
        let input = serde_json::json!({
            "patterns": [{"filename": ".txt", "status": "unknown"}, {"filename": ".docx", "status": "unknown"}, {"filename": ".csv", "status": "unknown"}, {"filename": ".py", "status": "unknown"}],
            "folders": ["test_pipeline"]
        }).to_string();

        // Action
        let result = fileMatch(&input).unwrap(); // Adjust for async/sync as necessary

        // Verify
        println!("{:?}", result);
        assert!(result.contains("valid"));
        // assert!(result.contains("invalid"));
        // assert!(result.contains("missing")); // Use this line if you expect some patterns to not be found
        // More detailed assertions can be made based on the expected structure of your result
    }
}

#[tauri::command]
async fn read_file(path: std::path::PathBuf) -> Vec<u8> {
    // std::fs::ReadDir(path);
    std::fs::read(path).unwrap()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fileMatch, read_file, save_file, find_matches])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
