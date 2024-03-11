import { createStore } from 'zustand';
import {
  Connection,
  Edge,
  EdgeChange,
  Node,
  NodeChange,
  addEdge,
  OnNodesChange,
  OnEdgesChange,
  OnConnect,
  applyNodeChanges,
  applyEdgeChanges,
} from 'reactflow';
import { CustomNodeData } from '~/types/RFNodes';

export interface RFProps {
  id: number,
  nodes: Node<CustomNodeData>[];
  edges: Edge[];
}

export interface RFState extends RFProps {
  getId: () => string;
  onNodesChange: OnNodesChange;
  onEdgesChange: OnEdgesChange;
  onConnect: OnConnect;
  setNodes: (nodes) => void;
  removeNode: (nodeId) => void;
  setEdges: (edges) => void;
  updateNodeData: (nodeId: string, field: string, value: string) => void;
};

export type RFStore = ReturnType<typeof createRFStore>

export const createRFStore = (initProps?: Partial<RFProps>) => {
  const DEFAULT_PROPS: RFProps = {
    id: 0,
    nodes: [],
    edges: []
  }
  return createStore<RFState>((set, get) => ({
    ...DEFAULT_PROPS,
    ...initProps,
    getId: () => {
      set((state) => ({ id: state.id + 1 })); // Increment the ID using set
      return `${get().id}`;
    },
    onNodesChange: (changes: NodeChange[]) => {
      set({
        nodes: applyNodeChanges(changes, get().nodes),
      });
    },
    onEdgesChange: (changes: EdgeChange[]) => {
      set({
        edges: applyEdgeChanges(changes, get().edges),
      });
    },
    onConnect: (connection: Connection) => {
      set({
        edges: addEdge(connection, get().edges),
      });
    },
    setNodes: (nodes) => {
      set((state) => ({
        nodes: typeof nodes === 'function' ? nodes(state.nodes) : nodes,
      }));
    },
    setEdges: (edges) => {
      set((state) => ({
        edges: typeof edges === 'function' ? edges(state.edges) : edges,
      }));
    },
    removeNode: (nodeId: string) => {
      console.log(nodeId)
      console.log(get().nodes.filter((node) => node.id != nodeId))
      set({
        nodes: get().nodes.filter((node) => node.id != nodeId)
      })
    },
    updateNodeData: (nodeId: string, field: string, value: string) => {
      set({
        nodes: get().nodes.map((node) => {
          if (node.id === nodeId) {
            node.data[field] = value;
            node.data = { ...node.data }
          }
          return node;
        }),
      });
    },
  }));
}