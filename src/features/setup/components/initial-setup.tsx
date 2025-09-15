"use client";

import { useState } from "react";
import HeroDustStorm from "@/components/custom/hero-dust-storm";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "motion/react";
import { toast } from "sonner";
import Image from "next/image";
import { useOpenMWConfig } from "@/features/app-config/context/openmw-config-context";
import { useOpenMWWizard } from "@/hooks/use-openmw-wizard";
import { CheckCircle } from "lucide-react";

export function InitialSetup({ onFinish }: { onFinish: () => void }) {
  const [isDownloading, setIsDownloading] = useState(false);
  const [tes3mpInstalled, setTes3mpInstalled] = useState(false);
  const { config, isLoading, refreshConfig } = useOpenMWConfig();
  const { isRunning, wizardPid, lastResult } = useOpenMWWizard();

  const handleCTAClick = async () => {
    setIsDownloading(true);

    toast.promise(invoke("download_latest_windows_release"), {
      loading: "Installing TES3MP...",
      success: async (data) => {
        if (!data) {
          throw new Error("Failed to install TES3MP");
        }
        await refreshConfig();

        setIsDownloading(false);
        setTes3mpInstalled(true);
        return "TES3MP installed successfully";
      },
      error: (error) => {
        console.error("Tes3MP installation error:", error);
        setIsDownloading(false);
        setTes3mpInstalled(false);
        return "TES3MP installation error";
      },
    });
  };

  const handleOpenMWWizardClick = async () => {
    try {
      const result = await invoke("run_openmw_wizard");
      console.log("OpenMW wizard result:", result);
    } catch (error) {
      console.error("OpenMW wizard error:", error);
    }
  };

  return (
    <div className="min-h-[calc(100vh-30px)] bg-background flex flex-col items-center justify-center">
      {tes3mpInstalled && (
        <div className="fixed top-8 inset-x-0 h-16 flex items-center justify-center z-50 bg-background/50 text-white p-4 rounded-lg text-center">
          {`Tes3MP Installed Successfully`}
        </div>
      )}
      {config && !isLoading && !isRunning && !lastResult?.success && (
        <div className="fixed bottom-0 inset-x-0 h-16 flex items-center justify-center z-50 bg-background/50 text-white p-4 rounded-lg text-center">
          {`OpenMW Installed Globally. It is still recommended to re-run the wizard to ensure everything is set up correctly.`}
        </div>
      )}

      <HeroDustStorm
        className="grow w-full min-h-full flex items-center justify-center"
        title="NEREVAR"
        subtitle="Morrowind Multiplayer Manager"
      >
        {/* centered image of logo */}
        <div className="absolute -top-52  left-0 w-full h-full flex items-center justify-center">
          <Image src="/logo.webp" alt="Logo" width={150} height={150} />
        </div>
        {/* CTA Button with fire effect - positioned like original */}
        <AnimatePresence mode="sync">
          {!tes3mpInstalled && !isDownloading && (
            <motion.button
              key="install-tes3mp"
              className="relative px-8 py-4 bg-gradient-to-r from-orange-600 to-red-600 text-white font-bold rounded-lg shadow-lg hover:shadow-xl transition-all duration-300 overflow-hidden cursor-pointer hover:scale-105 disabled:opacity-20 disabled:cursor-not-allowed disabled:bg-gradient-to-r disabled:from-orange-600/50 disabled:to-red-600/50"
              style={{
                boxShadow:
                  "0 0 20px rgba(255, 100, 0, 0.5), 0 4px 8px rgba(0,0,0,0.3)",
              }}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.9 }}
              transition={{ duration: 0.3 }}
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              onClick={handleCTAClick}
              disabled={isDownloading}
            >
              {/* Button fire glow */}
              <motion.div
                key="install-tes3mp-glow"
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

              <span className="relative font-sans text-lg z-10">
                {`Install TES3MP`}
              </span>
            </motion.button>
          )}
          <div className="flex flex-col gap-4">
            {tes3mpInstalled && !isLoading && !isDownloading && (
              <>
                <motion.button
                  key="install-openmw"
                  className="relative px-8 py-4 bg-gradient-to-b from-red-600 to-orange-600 text-white font-bold rounded-lg shadow-lg hover:shadow-xl transition-all duration-300 overflow-hidden cursor-pointer hover:scale-105 disabled:opacity-20 disabled:cursor-not-allowed disabled:bg-gradient-to-r disabled:from-blue-600/50 disabled:to-purple-600/50"
                  style={{
                    boxShadow:
                      "0 0 20px rgba(255, 100, 0, 0.5), 0 4px 8px rgba(0,0,0,0.3)",
                  }}
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.9 }}
                  transition={{ duration: 0.3 }}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  onClick={handleOpenMWWizardClick}
                  disabled={isDownloading || isRunning}
                >
                  {/* Button fire glow */}
                  <motion.div
                    key="install-openmw-glow"
                    className="absolute inset-0 bg-gradient-to-r from-amber-400/30 to-orange-400/30 rounded-lg"
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

                  <span className="relative font-sans text-lg z-10">
                    {!isRunning
                      ? `Run OpenMW Wizard`
                      : `Running OpenMW Wizard... | `}
                    {isRunning && (
                      <span className="text-xs text-white/50">
                        PID: {wizardPid}
                      </span>
                    )}
                  </span>
                </motion.button>
              </>
            )}
            {!isRunning && lastResult?.success && (
              <>
                <motion.button
                  key="wizard-successful"
                  className="relative px-8 py-4 bg-gradient-to-b from-green-600 to-emerald-600 text-white font-bold rounded-lg shadow-lg hover:shadow-xl transition-all duration-300 overflow-hidden cursor-pointer hover:scale-105 disabled:opacity-20 disabled:cursor-not-allowed disabled:bg-gradient-to-r disabled:from-blue-600/50 disabled:to-purple-600/50"
                  style={{
                    boxShadow:
                      "0 0 20px rgba(255, 100, 0, 0.5), 0 4px 8px rgba(0,0,0,0.3)",
                  }}
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.9 }}
                  transition={{ duration: 0.3 }}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  onClick={onFinish}
                  disabled={isDownloading || isRunning}
                >
                  {/* Button fire glow */}
                  <motion.div
                    key="wizard-successful-glow"
                    className="absolute inset-0 bg-gradient-to-r from-emerald-400/30 to-green-400/30 rounded-lg"
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
                  <div className="flex items-center justify-center">
                    <CheckCircle className="min-w-4 min-h-4 text-white mr-2" />

                    <span className="relative font-sans text-lg z-10">
                      {`Finish`}
                    </span>
                  </div>
                </motion.button>
              </>
            )}
          </div>
        </AnimatePresence>
      </HeroDustStorm>

      {/* {downloadStatus && (
        <div className="fixed bottom-4 left-4 right-4 bg-black/80 text-white p-4 rounded-lg text-center">
          {downloadStatus}
        </div>
      )} */}
    </div>
  );
}
