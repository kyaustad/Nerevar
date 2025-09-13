"use client";

import { ThemeProvider } from "next-themes";
import { Toaster } from "@/components/ui/sonner";
import { ConfigProvider } from "@/features/app-config/context/config-context";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <ConfigProvider>
        {children}
        <Toaster />
      </ConfigProvider>
    </ThemeProvider>
  );
}
