"use client";

import { useState } from "react";
import HeroDustStorm from "@/components/custom/hero-dust-storm";
import { env } from "@/env";
import { invoke } from "@tauri-apps/api/core";
import { useConfig } from "@/features/app-config/context/config-context";
import { motion } from "motion/react";
import { toast } from "sonner";

export default function Home() {
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadStatus, setDownloadStatus] = useState<string>("");
  const { config, isValid, refreshConfig } = useConfig();

  const handleCTAClick = async () => {
    setIsDownloading(true);
    setDownloadStatus("Installing Latest Windows Release of TES3MP...");

    try {
      const tes3mpReleaseUrl = env.NEXT_PUBLIC_LATEST_TES3MP_WINDOWS_RELEASE;

      toast.promise(
        invoke("download_latest_windows_release", {
          url: tes3mpReleaseUrl,
        }),
        {
          loading: "Installing TES3MP...",
          success: (data) => {
            if (!data) {
              throw new Error("Failed to install TES3MP");
            }
            setDownloadStatus("");
            refreshConfig();
            return "TES3MP installed successfully";
          },
          error: "TES3MP installation error",
        }
      );

      // const result: string = await invoke("download_latest_windows_release", {
      //   url: tes3mpReleaseUrl,
      // });

      // setDownloadStatus("");

      // Refresh config after successful installation
      await refreshConfig();
    } catch (error) {
      console.error("Tes3MP installation error:", error);
      toast.error("Tes3MP installation error");
      setDownloadStatus(
        `Error: ${error instanceof Error ? error.message : "Unknown error"}`
      );
    } finally {
      setIsDownloading(false);
    }
  };

  return (
    <div className="min-h-screen bg-background flex flex-col items-center justify-center">
      {config && (
        <div className="fixed top-0 inset-x-0 h-16 flex items-center justify-center z-50 bg-background/50 text-white p-4 rounded-lg text-center">
          {`Tes3MP Path: ${config.tes3mp_path} | Version: ${config.version}`}
        </div>
      )}

      <HeroDustStorm
        className="grow w-full min-h-screen flex items-center justify-center"
        title="NEREVAR"
        subtitle="Morrowind Multiplayer Manager"
      >
        {/* CTA Button with fire effect - positioned like original */}
        {(!config || !config?.tes3mp_path) && !isDownloading && (
          <motion.button
            className="relative px-8 py-4 bg-gradient-to-r from-orange-600 to-red-600 text-white font-bold rounded-lg shadow-lg hover:shadow-xl transition-all duration-300 overflow-hidden cursor-pointer hover:scale-105 disabled:opacity-20 disabled:cursor-not-allowed disabled:bg-gradient-to-r disabled:from-orange-600/50 disabled:to-red-600/50"
            style={{
              boxShadow:
                "0 0 20px rgba(255, 100, 0, 0.5), 0 4px 8px rgba(0,0,0,0.3)",
            }}
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.8, delay: 1.5 }}
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            onClick={handleCTAClick}
            disabled={isDownloading || isValid}
          >
            {/* Button fire glow */}
            <motion.div
              className="absolute inset-0 bg-gradient-to-r from-orange-400/30 to-red-400/30 rounded-lg"
              animate={{
                opacity: [0, 0.6, 0],
                scale: [1, 1.1, 1],
              }}
              transition={{
                duration: 1.5,
                repeat: Infinity,
                ease: "easeInOut",
              }}
            />
            {!isValid && !isDownloading && (
              <span className="relative font-sans text-lg z-10">
                {`Install TES3MP`}
              </span>
            )}
            {isValid && config && config.tes3mp_path && !isDownloading && (
              <span className="relative font-sans text-lg z-10">
                {`TES3MP Installed!`}
              </span>
            )}
            {isDownloading && !isValid && !config && (
              <span className="relative font-sans text-lg z-10">
                {`Installing...`}
              </span>
            )}
          </motion.button>
        )}
      </HeroDustStorm>

      {downloadStatus && (
        <div className="fixed bottom-4 left-4 right-4 bg-black/80 text-white p-4 rounded-lg text-center">
          {downloadStatus}
        </div>
      )}
    </div>
  );
}
