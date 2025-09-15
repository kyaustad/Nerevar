"use client";

import { useConfig } from "@/features/app-config/context/config-context";
import { InitialSetup } from "@/features/setup/components/initial-setup";
import BackgroundEmbers from "@/components/custom/background-embers";
import { Loader2 } from "lucide-react";
import { Dashboard } from "@/features/dashboard/components/dashboard";

export default function Home() {
  const { config, isLoading, refreshConfig } = useConfig();

  if (isLoading) {
    return (
      <BackgroundEmbers className="min-h-[calc(100vh-30px)] w-full  flex items-center justify-center">
        <div className="flex items-center  justify-center h-full w-full">
          <Loader2 className="animate-spin min-w-16 min-h-16 text-white/50 font-bold text-2xl" />
        </div>
      </BackgroundEmbers>
    );
  }

  if (!config && !isLoading) {
    return <InitialSetup onFinish={refreshConfig} />;
  }

  return <Dashboard />;
}
