import React from "react";
import { createRoot } from "react-dom/client";
import { WebSocketProvider } from "./Socket";
import Main from "./Main";
import "./style.css";

const App: React.FC = () => {
  return (
    <div className="mono no-scrollbar h-screen w-screen">
      <WebSocketProvider>
        <Main />
      </WebSocketProvider>
    </div>
  );
};

const root = createRoot(document.getElementById("root") as HTMLDivElement);
root.render(<App />);
