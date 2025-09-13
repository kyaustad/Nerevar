"use client";

import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { type NerevarConfig } from "@/types/nerevar-config";

export type ConfigContextType = {
  config: NerevarConfig | null;
  isLoading: boolean;
  error: string | null;
  isValid: boolean;
  refreshConfig: () => Promise<void>;
  clearConfig: () => void;
};

export const ConfigContext = createContext<ConfigContextType | undefined>(
  undefined
);

export const useConfig = () => {
  const context = useContext(ConfigContext);
  if (context === undefined) {
    throw new Error("useConfig must be used within a ConfigProvider");
  }
  return context;
};

type ConfigProviderProps = {
  children: ReactNode;
};

export const ConfigProvider = ({ children }: ConfigProviderProps) => {
  const [config, setConfig] = useState<NerevarConfig | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isValid = config !== null && config.tes3mp_path.length > 0;

  const loadConfig = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const result: NerevarConfig | null = await invoke("get_nerevar_config");
      console.log("Loaded config:", result);
      setConfig(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Unknown error";
      console.error("Failed to load config:", errorMessage);
      setError(errorMessage);
      setConfig(null);
    } finally {
      setIsLoading(false);
    }
  };

  const refreshConfig = async () => {
    await loadConfig();
  };

  const clearConfig = () => {
    setConfig(null);
    setError(null);
  };

  // Load config on mount
  useEffect(() => {
    loadConfig();
  }, []);

  const value: ConfigContextType = {
    config,
    isLoading,
    error,
    isValid,
    refreshConfig,
    clearConfig,
  };

  return (
    <ConfigContext.Provider value={value}>{children}</ConfigContext.Provider>
  );
};
