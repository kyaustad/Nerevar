"use client";

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export interface TES3MPBrowserEvent {
  pid: number;
  success: boolean;
  exit_code: number | null;
  message: string;
}

export const useTES3MPBrowser = () => {
  const [isRunning, setIsRunning] = useState(false);
  const [browserPid, setBrowserPid] = useState<number | null>(null);
  const [lastResult, setLastResult] = useState<TES3MPBrowserEvent | null>(null);

  useEffect(() => {
    // Listen for TES3MP browser started event
    const unlistenStarted = listen<number>(
      "tes3mp-browser-started",
      (event) => {
        console.log("TES3MP browser started with PID:", event.payload);
        setIsRunning(true);
        setBrowserPid(event.payload);
        setLastResult(null); // Clear previous result
      }
    );

    // Listen for TES3MP browser exited event
    const unlistenExited = listen<TES3MPBrowserEvent>(
      "tes3mp-browser-exited",
      (event) => {
        console.log("TES3MP browser exited:", event.payload);
        setIsRunning(false);
        setBrowserPid(null);
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
    browserPid,
    lastResult,
  };
};
