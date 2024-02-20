const WindowMinimize = ({ width, height, fill }) => {
  return (
    <svg
      className="mb-2"
      fill={fill}
      width={width}
      height={height}
      viewBox="0 0 512 512"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path d="M480 480H32c-17.7 0-32-14.3-32-32s14.3-32 32-32h448c17.7 0 32 14.3 32 32s-14.3 32-32 32z" />
    </svg>
  );
};

export default WindowMinimize;
