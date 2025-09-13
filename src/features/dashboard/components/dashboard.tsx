"use client";

import BackgroundEmbers from "@/components/custom/background-embers";
import { OpenMWWizardCard } from "@/features/launchers/openmw-wizard-card";
import { OpenMWLauncherCard } from "@/features/launchers/openmw-launcher-card";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { Separator } from "@/components/ui/separator";

export function Dashboard() {
  return (
    <AnimatePresence mode="wait">
      <BackgroundEmbers
        className="relative min-h-screen w-full flex flex-col items-center p-8 "
        contentClassName="p-8"
      >
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.9 }}
          transition={{
            duration: 1,
            ease: "easeInOut",
          }}
          className="flex flex-col items-center gap-4 w-full h-full"
        >
          <h1 className="text-5xl font-sovngarde-bold ">Nerevar</h1>
          <Separator />
          <Image
            src="/logo.webp"
            alt="Logo"
            width={64}
            height={64}
            className="absolute top-4 right-4"
            style={{
              objectFit: "contain",
            }}
          />
          <div className="grid grid-cols-1 md:grid-cols-2 h-full gap-4 justify-items-center w-full ">
            <div className="col-span-2 grid grid-cols-1 md:grid-cols-2 h-fit gap-4 justify-items-center w-full ">
              <div className="flex col-span-2 justify-center">
                <h2 className="text-3xl font-sovngarde text-white">
                  OpenMW Launchers
                </h2>
              </div>
              <div className="flex flex-col gap-4 w-full h-full items-start">
                <OpenMWWizardCard className="w-full max-w-md max-h-64" />
              </div>
              <div className="flex flex-col gap-4 w-full h-full items-end">
                <OpenMWLauncherCard className="w-full max-w-md max-h-64" />
              </div>
            </div>
          </div>
        </motion.div>
      </BackgroundEmbers>
    </AnimatePresence>
  );
}
