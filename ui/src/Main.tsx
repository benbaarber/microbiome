import React, { useCallback, useState } from "react";
import { useWebSocket } from "./Socket";
import Display from "./canvas/display";

const Main: React.FC = () => {
  const [d, setD] = useState<Display>();

  useWebSocket("state", (data) => {
    console.log("Data received", data);
    d?.draw(data);
  });

  const canvasMounted = useCallback((node: HTMLCanvasElement) => {
    if (!node) return;

    const d = new Display();
    const cx = node.getContext("2d");
    d.bind(node, cx);
    setD(d);
  }, []);

  return (
    <div className={"relative h-full w-full overflow-hidden"}>
      {/* {false ? (
        <div className="flex h-full w-full animate-pulse items-center justify-center text-sm text-gray-600">
          <span>Waiting on data...</span>
        </div>
      ) : (
        <canvas ref={canvasMounted} />
      )} */}
      <canvas ref={canvasMounted} />
    </div>
  );
};

export default Main;
