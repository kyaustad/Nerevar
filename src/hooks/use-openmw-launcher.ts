"use client";

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export interface OpenMWLauncherEvent {
  pid: number;
  success: boolean;
  exit_code: number | null;
  message: string;
}

export const useOpenMWLauncher = () => {
  const [isRunning, setIsRunning] = useState(false);
  const [launcherPid, setLauncherPid] = useState<number | null>(null);
  const [lastResult, setLastResult] = useState<OpenMWLauncherEvent | null>(
    null
  );

  useEffect(() => {
    // Listen for wizard started event
    const unlistenStarted = listen<number>(
      "openmw-launcher-started",
      (event) => {
        console.log("OpenMW launcher started with PID:", event.payload);
        setIsRunning(true);
        setLauncherPid(event.payload);
        setLastResult(null); // Clear previous result
      }
    );

    // Listen for wizard exited event
    const unlistenExited = listen<OpenMWLauncherEvent>(
      "openmw-launcher-exited",
      (event) => {
        console.log("OpenMW launcher exited:", event.payload);
        setIsRunning(false);
        setLauncherPid(null);
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
    launcherPid,
    lastResult,
  };
};
