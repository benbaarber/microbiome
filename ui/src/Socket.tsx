import React, {
  createContext,
  DependencyList,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";

type Handler = (data: string) => void;

type MessageRouter = {
  [event: string]: Handler;
};

class WebSocketClient {
  private ws: WebSocket;
  private router: MessageRouter = {};
  private readonly url: string;
  private readonly maxReconnectAttempts = 5;
  private reconnectAttempts = 0;
  private reconnectDelay = 2000;

  constructor(url: string) {
    this.url = url;

    try {
      this.connect();
    } catch (err) {
      console.error(`Websocket context failed with error ${err}`);
    }
  }

  private connect() {
    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log("WebSocket Connected");
      this.reconnectAttempts = 0;
      this.reconnectDelay = 2000;
    };

    this.ws.onmessage = (e) => {
      const { event, data } = JSON.parse(e.data);

      if (event in this.router) {
        this.router[event]?.(data);
      } else {
        console.error(`No handler for ${event}`);
      }
    };

    this.ws.onerror = (e) => {
      console.error("WebSocket Error: ", e);
    };

    this.ws.onclose = () => {
      console.log("WebSocket Disconnected");
      this.reconnect();
    };
  }

  private reconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      setTimeout(() => {
        this.reconnectAttempts++;
        console.log(
          `Attempting to reconnect... (Attempt ${this.reconnectAttempts})`,
        );
        this.connect();
      }, this.reconnectDelay);

      this.reconnectDelay *= 2;
    } else {
      console.error("Max WebSocket reconnection attempts reached");
    }
  }

  route(event: string, handler: Handler) {
    this.router[event] = handler;
  }

  unroute(event: string) {
    delete this.router[event];
  }

  close() {
    this.ws?.close();
  }
}

const WebSocketContext = createContext<WebSocketClient>(null);

export const useWebSocket = (
  event: string,
  handler: (data: any) => void,
  deps?: DependencyList,
) => {
  const wsc = useContext(WebSocketContext);
  return useEffect(() => {
    wsc.route(event, handler);

    return () => {
      wsc.unroute(event);
    };
  }, deps);
};

export const WebSocketProvider: React.FC<{ children: ReactNode }> = ({
  children,
}) => {
  const [wsc, setWsc] = useState<WebSocketClient>(null);

  useEffect(() => {
    (async () => {
      const wsproto = {
        "https:": "wss:",
        "http:": "ws:",
      }[window.location.protocol];
      const url = `${wsproto}//${location.host}/ws`;
      const wsc = new WebSocketClient(url);
      setWsc(wsc);
    })();

    return () => {
      wsc?.close();
    };
  }, []);

  if (wsc === null) return;

  return (
    <WebSocketContext.Provider value={wsc}>
      {children}
    </WebSocketContext.Provider>
  );
};
