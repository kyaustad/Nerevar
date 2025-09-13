"use client";

import { createContext, useContext, useState, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";

export type OpenMWConfigContextType = {
  config: Record<string, any> | null;
  isLoading: boolean;
  error: string | null;
  refreshConfig: () => Promise<void>;
  clearConfig: () => void;
};

export const OpenMWConfigContext = createContext<
  OpenMWConfigContextType | undefined
>(undefined);

export const useOpenMWConfig = () => {
  const context = useContext(OpenMWConfigContext);
  if (context === undefined) {
    throw new Error(
      "useOpenMWConfig must be used within an OpenMWConfigProvider"
    );
  }
  return context;
};

type OpenMWConfigProviderProps = {
  children: ReactNode;
};

export const OpenMWConfigProvider = ({
  children,
}: OpenMWConfigProviderProps) => {
  const [config, setConfig] = useState<Record<string, any> | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const result: Record<string, any> | null = await invoke(
        "get_openmw_config"
      );
      console.log("Loaded OpenMW config:", result);
      setConfig(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Unknown error";
      console.error("Failed to load OpenMW config:", errorMessage);
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

  const value: OpenMWConfigContextType = {
    config,
    isLoading,
    error,
    refreshConfig,
    clearConfig,
  };

  return (
    <OpenMWConfigContext.Provider value={value}>
      {children}
    </OpenMWConfigContext.Provider>
  );
};
