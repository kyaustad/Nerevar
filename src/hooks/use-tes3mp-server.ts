"use client";

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export interface TES3MPServerEvent {
  pid: number;
  success: boolean;
  exit_code: number | null;
  message: string;
}

export const useTES3MPServer = () => {
  const [isRunning, setIsRunning] = useState(false);
  const [serverPid, setServerPid] = useState<number | null>(null);
  const [lastResult, setLastResult] = useState<TES3MPServerEvent | null>(null);

  useEffect(() => {
    // Listen for TES3MP started event
    const unlistenStarted = listen<number>("tes3mp-server-started", (event) => {
      console.log("TES3MP started with PID:", event.payload);
      setIsRunning(true);
      setServerPid(event.payload);
      setLastResult(null); // Clear previous result
    });

    // Listen for TES3MP exited event
    const unlistenExited = listen<TES3MPServerEvent>(
      "tes3mp-server-exited",
      (event) => {
        console.log("TES3MP exited:", event.payload);
        setIsRunning(false);
        setServerPid(null);
        setLastResult(event.payload);
      }
    );

    // Cleanup listeners on unmount
    return () => {
      unlistenStarted.then((unlisten) => unlisten());
      unlistenExited.then((unlisten) => unlisten());
    };
  }, []);

  return {
    isRunning,
    serverPid,
    lastResult,
  };
};
