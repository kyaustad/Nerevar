"use client";

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export interface OpenMWWizardEvent {
  pid: number;
  success: boolean;
  exit_code: number | null;
  message: string;
}

export const useOpenMWWizard = () => {
  const [isRunning, setIsRunning] = useState(false);
  const [wizardPid, setWizardPid] = useState<number | null>(null);
  const [lastResult, setLastResult] = useState<OpenMWWizardEvent | null>(null);

  useEffect(() => {
    // Listen for wizard started event
    const unlistenStarted = listen<number>("openmw-wizard-started", (event) => {
      console.log("OpenMW wizard started with PID:", event.payload);
      setIsRunning(true);
      setWizardPid(event.payload);
      setLastResult(null); // Clear previous result
    });

    // Listen for wizard exited event
    const unlistenExited = listen<OpenMWWizardEvent>(
      "openmw-wizard-exited",
      (event) => {
        console.log("OpenMW wizard exited:", event.payload);
        setIsRunning(false);
        setWizardPid(null);
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
    wizardPid,
    lastResult,
  };
};
