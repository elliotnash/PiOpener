import { create } from "zustand";
import EventSource, { type EventSourceListener } from "react-native-sse";
import "react-native-url-polyfill/auto";
import { useSettingsStore } from "~/store/settings";
import { useEffect } from "react";

interface StatusState {
  status: StatusEvent | null;
  isConnected: boolean;
  error: string | null;
  connect: () => void;
  disconnect: () => void;
}

interface StatusEvent {
  status: string;
  setpoint: string;
  position: number;
}

export const useStatusStore = create<StatusState>((set, get) => {
  let eventSource: EventSource | null = null;

  const connect = () => {
    // Get API URL and key from settings store
    const { apiUrl, apiKey } = useSettingsStore.getState();

    console.log(apiKey, apiUrl);

    if (!apiUrl || !apiKey) {
      set({ error: "API URL or API Key not configured" });
      return;
    }

    // Close existing connection if any
    if (eventSource) {
      disconnect();
    }

    try {
      const url = new URL(`${apiUrl}/watch-status`);

      eventSource = new EventSource(url, {
        headers: {
          Authorization: {
            toString: () => `Bearer ${apiKey}`,
          },
        },
      });

      const listener: EventSourceListener = (event) => {
        if (event.type === "open") {
          set({ isConnected: true, error: null });
          console.log("SSE connection opened");
        } else if (event.type === "message" && event.data) {
          try {
            const data = JSON.parse(event.data) as StatusEvent;
            set({
              status: data,
              error: null,
            });
          } catch (err) {
            console.error("Error parsing SSE message:", err);
            set({ error: "Failed to parse status update" });
          }
        } else if (event.type === "error" || event.type === "exception") {
          const errorMessage = event.message || "Unknown error";
          console.error("SSE connection error:", errorMessage);
          set({ isConnected: false, error: errorMessage });
        }
      };

      eventSource.addEventListener("open", listener);
      eventSource.addEventListener("message", listener);
      eventSource.addEventListener("error", listener);
    } catch (err) {
      console.error("Failed to establish SSE connection:", err);
      set({
        isConnected: false,
        error: err instanceof Error ? err.message : "Unknown error",
      });
    }
  };

  const disconnect = () => {
    if (eventSource) {
      eventSource.removeAllEventListeners();
      eventSource.close();
      eventSource = null;
      set({ isConnected: false });
      console.log("SSE connection closed");
    }
  };

  return {
    status: null,
    isConnected: false,
    error: null,
    connect,
    disconnect,
  };
});

export function useStatus() {
  const { status, isConnected, error, connect, disconnect } = useStatusStore();

  const { apiKey, apiUrl } = useSettingsStore();

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, [connect, disconnect, apiKey, apiUrl]);

  return {
    // Status data
    status,

    // Connection state
    isConnected,
    error,
  };
}
