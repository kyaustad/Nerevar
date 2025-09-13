"use client";

import { ThemeProvider } from "next-themes";
import { Toaster } from "@/components/ui/sonner";
import { ConfigProvider } from "@/features/app-config/context/config-context";
import { OpenMWConfigProvider } from "@/features/app-config/context/openmw-config-context";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark">
      <ConfigProvider>
        <OpenMWConfigProvider>{children}</OpenMWConfigProvider>
        <Toaster />
      </ConfigProvider>
    </ThemeProvider>
  );
}
