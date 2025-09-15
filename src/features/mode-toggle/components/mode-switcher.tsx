"use client";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useState, useEffect } from "react";
import { useConfig } from "@/features/app-config/context/config-context";

export function ModeSwitcher({
  onModeChanged,
}: {
  onModeChanged: (mode: "player" | "server") => void;
}) {
  const { config, setMode: setConfigMode } = useConfig();
  const [mode, setMode] = useState(config?.mode || "player");

  useEffect(() => {
    setMode(config?.mode || "player");
  }, [config]);

  const handleModeChange = async (value: string) => {
    try {
      setMode(value as "player" | "server");
      onModeChanged(value as "player" | "server");
      await setConfigMode(value as "player" | "server");
    } catch (error) {
      console.error("Failed to set mode:", error);
    }
  };

  return (
    <div className="flex flex-col items-center gap-2">
      <p className="text-sm text-muted-foreground">{`App Mode:`}</p>
      <Select value={mode} onValueChange={handleModeChange}>
        <SelectTrigger>
          <SelectValue placeholder="Select a mode" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="player">Player</SelectItem>
          <SelectItem value="server">Server Admin</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}
