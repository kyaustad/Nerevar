"use client";

import BackgroundEmbers from "@/components/custom/background-embers";

import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { Separator } from "@/components/ui/separator";
import { useState } from "react";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import { ArrowLeftIcon } from "lucide-react";
import { useRouter } from "next/navigation";
import { ServerConfigurationForm } from "@/features/server-configuration/server-config-form";

export default function EditServerSettingsPage() {
  const [particlesEnabled, setParticlesEnabled] = useState(true);
  const router = useRouter();
  return (
    <BackgroundEmbers
      className="relative min-h-[calc(100vh-30px)] min-w-full flex flex-col items-center p-8 "
      contentClassName="px-8 py-4 w-full"
      particlesEnabled={particlesEnabled}
    >
      <motion.div
        initial={{ opacity: 0, scale: 1 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.9 }}
        transition={{
          duration: 1,
          ease: "easeInOut",
        }}
        className="flex flex-col items-center gap-4 w-full mb-8"
      >
        <div className="flex flex-col gap-1 items-center justify-center text-center">
          <h1 className="text-5xl font-sovngarde-bold ">
            Server Configuration
          </h1>
        </div>
        <Separator />
        <div className="absolute top-4 right-4 z-50 flex items-center justify-center gap-2">
          <div className="flex items-center justify-center gap-2">
            <Checkbox
              checked={particlesEnabled}
              onCheckedChange={(checked) => {
                if (checked === "indeterminate") {
                  setParticlesEnabled(true);
                } else {
                  setParticlesEnabled(checked);
                }
              }}
            />
            <p className="text-xs text-muted-foreground">
              Background Particles
            </p>
          </div>
        </div>

        <div className="absolute top-4 left-4">
          {/* <ModeSwitcher onModeChanged={(mode) => setCurrentMode(mode)} /> */}
          <Button onClick={() => router.back()} variant="default">
            <ArrowLeftIcon />
            Back
          </Button>
        </div>
        <div className="flex flex-col gap-4 w-full h-full items-start">
          <ServerConfigurationForm />
        </div>
      </motion.div>
    </BackgroundEmbers>
  );
}
