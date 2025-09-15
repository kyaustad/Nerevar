"use client";

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export interface TES3MPEvent {
  pid: number;
  success: boolean;
  exit_code: number | null;
  message: string;
}

export const useTES3MP = () => {
  const [isRunning, setIsRunning] = useState(false);
  const [tes3mpPid, setTES3MPPid] = useState<number | null>(null);
  const [lastResult, setLastResult] = useState<TES3MPEvent | null>(null);

  useEffect(() => {
    // Listen for TES3MP started event
    const unlistenStarted = listen<number>("tes3mp-started", (event) => {
      console.log("TES3MP started with PID:", event.payload);
      setIsRunning(true);
      setTES3MPPid(event.payload);
      setLastResult(null); // Clear previous result
    });

    // Listen for TES3MP exited event
    const unlistenExited = listen<TES3MPEvent>("tes3mp-exited", (event) => {
      console.log("TES3MP exited:", event.payload);
      setIsRunning(false);
      setTES3MPPid(null);
      setLastResult(event.payload);
    });

    // Cleanup listeners on unmount
    return () => {
      unlistenStarted.then((unlisten) => unlisten());
      unlistenExited.then((unlisten) => unlisten());
    };
  }, []);

  return {
    isRunning,
    tes3mpPid,
    lastResult,
  };
};
