"use client";

import BackgroundEmbers from "@/components/custom/background-embers";
import { OpenMWWizardCard } from "@/features/launchers/openmw-wizard-card";
import { OpenMWLauncherCard } from "@/features/launchers/openmw-launcher-card";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { Separator } from "@/components/ui/separator";
import { CheckForTes3MpUpdateButton } from "@/features/check-for-updates/tes3mp/components/check-for-update-button";
import { ModeSwitcher } from "@/features/mode-toggle/components/mode-switcher";

import { CheckForAppUpdate } from "@/features/check-for-updates/nerevar/components/check-for-app-update";
import { ServerList } from "@/features/server-list/components/server-list";
import { Tes3MPBrowserCard } from "@/features/launchers/tes3mp-browser-card";
import { Tes3MPCard } from "@/features/launchers/tes3mp-card";
import { useConfig } from "@/features/app-config/context/config-context";
import { useState } from "react";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import { FolderIcon, RouterIcon, SquarePenIcon } from "lucide-react";
import { useRouter } from "next/navigation";
import { Tes3MPServerCard } from "@/features/launchers/tes3mp-server-card";
import { invoke } from "@tauri-apps/api/core";
import { ManualConnectCard } from "@/features/launchers/manual-connect-card";

export function Dashboard() {
  const { config, isLoading } = useConfig();
  const [particlesEnabled, setParticlesEnabled] = useState(true);
  const [currentMode, setCurrentMode] = useState<"player" | "server">(
    config?.mode ?? "player"
  );
  const router = useRouter();
  return (
    <AnimatePresence mode="popLayout">
      <BackgroundEmbers
        className="relative min-h-[calc(100vh-30px)] min-w-full flex flex-col items-center p-8 "
        contentClassName="p-8 w-full"
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
          <div className="flex flex-col gap-2 items-center justify-center text-center">
            <h1 className="text-5xl font-sovngarde-bold ">Nerevar</h1>
            <CheckForAppUpdate className="" />
            <Button
              variant="default"
              size="sm"
              onClick={() => {
                invoke("open_nerevar_appdata_dir_in_explorer");
              }}
            >
              <FolderIcon className="w-4 h-4" />
              <span>{`Open App Data`}</span>
            </Button>
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

            <Image
              src="/logo.webp"
              alt="Logo"
              width={64}
              height={64}
              className=""
              style={{
                objectFit: "contain",
              }}
            />
          </div>
          {/* <Image
            src="/multiplayer-icon.webp"
            alt="Logo"
            width={64}
            height={64}
            className="absolute top-4 left-4"
            style={{
              objectFit: "contain",
            }}
          /> */}
          {/* mode selection */}
          <div className="absolute top-4 left-4 flex flex-col gap-2 items-center justify-center">
            <ModeSwitcher onModeChanged={(mode) => setCurrentMode(mode)} />
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 h-full gap-4 justify-items-center w-full ">
            {/* Tes3MP Client */}
            <div className="col-span-1 h-fit grid grid-cols-1 md:grid-cols-2 gap-4 justify-items-center w-full p-4 rounded-lg border-2 border-border/75">
              <div className="flex flex-col col-span-2 justify-center items-center gap-2">
                <div className="flex flex-row gap-2 items-center justify-center">
                  <h2 className="text-3xl font-sovngarde text-white">
                    {`Tes3MP`}
                  </h2>
                  <p className="text-xs text-muted-foreground items-center text-center">
                    {`v${config?.version || "Unknown Version"}`}
                  </p>
                </div>
                <p className="text-sm text-muted-foreground">
                  {`Manage your Tes3MP installation and updates. Updating Tes3MP will erase all your current settings and server configurations. Working on fixing this this.`}
                </p>
              </div>
              <div className="flex flex-col col-span-2 gap-4 w-full h-full items-start">
                <CheckForTes3MpUpdateButton className="w-full" />
              </div>
              <div className="flex flex-col col-span-2 gap-4 w-full h-full items-start">
                <ManualConnectCard className="w-full max-h-64 min-h-full" />
              </div>
              <div className="flex flex-col col-span-1 gap-4 w-full h-full items-start">
                <Tes3MPBrowserCard className="w-full max-w-md max-h-64 min-h-full" />
              </div>
              <div className="flex flex-col col-span-1 gap-4 w-full h-full items-start">
                <Tes3MPCard className="w-full max-w-md max-h-64 min-h-full" />
              </div>

              {/* Tes3MP Server */}

              {currentMode === "server" && (
                <>
                  <div className="flex flex-col col-span-2 gap-4 w-full h-full items-start">
                    <Tes3MPServerCard className="w-full  max-h-64 min-h-full" />
                  </div>

                  <Separator className="w-full col-span-2" />
                  <div className="flex flex-col col-span-2 justify-center items-center gap-2">
                    <div className="flex flex-row gap-2 items-center justify-center text-center">
                      <h2 className="text-3xl font-sovngarde text-white">
                        {`Server Configuration`}
                      </h2>
                      {/* <p className="text-xs text-muted-foreground items-center text-center align-middle ">
                      {`v${config?.version || "Unknown Version"}`}
                    </p> */}
                    </div>
                    <p className="text-sm text-muted-foreground text-center">
                      {`Manage your Tes3MP server configuration and settings`}
                    </p>
                  </div>
                  <Button
                    className="w-full"
                    onClick={() => router.push("/edit-server-config")}
                  >
                    <SquarePenIcon />
                    Server Configuration
                  </Button>
                  <Button
                    className="w-full"
                    onClick={() => router.push("/edit-server-settings")}
                  >
                    <RouterIcon />
                    Server Settings
                  </Button>
                </>
              )}
            </div>
            {/* Tes3MP Server List */}
            <div className="col-span-1 grid grid-cols-1 md:grid-cols-2 h-fit gap-4 justify-items-center w-full p-4 rounded-lg border-2 border-border/75">
              <div className="flex flex-col col-span-2 justify-center items-center gap-2">
                <h2 className="text-3xl font-sovngarde text-white">
                  {`Morrowind Multiplayer`}
                </h2>
                <p className="text-sm text-muted-foreground">
                  {`Browse and connect to Tes3MP servers. Currently ping is broken.`}
                </p>
              </div>
              <div className="flex flex-col col-span-2 gap-4 w-full h-full items-start">
                <ServerList />
              </div>
            </div>

            {/* OpenMW */}
            <div className="col-span-2 grid grid-cols-1 md:grid-cols-2 h-fit gap-4 justify-items-center w-full p-4 rounded-lg border-2 border-border/75">
              <div className="flex flex-col col-span-2 justify-center items-center gap-2">
                <h2 className="text-3xl font-sovngarde text-white">OpenMW</h2>
                <p className="text-sm text-muted-foreground">
                  {`Manage OpenMW specific settings`}
                </p>
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
