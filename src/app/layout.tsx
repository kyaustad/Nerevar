"use client";

import {
  ChevronDownIcon,
  ExpandIcon,
  Maximize2Icon,
  SquareXIcon,
  XIcon,
} from "lucide-react";
import "./globals.css";
import { Providers } from "./providers";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  useEffect(() => {
    // Only run on client side
    if (typeof window !== "undefined") {
      const appWindow = getCurrentWindow();
      if (appWindow) {
        document
          .getElementById("titlebar-minimize")
          ?.addEventListener("click", () => appWindow.minimize());
        document
          .getElementById("titlebar-maximize")
          ?.addEventListener("click", () => appWindow.toggleMaximize());
        document
          .getElementById("titlebar-close")
          ?.addEventListener("click", () => appWindow.close());
      }
    }
  }, []);
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="antialiased">
        <div className="titlebar">
          <div data-tauri-drag-region className="flex-1"></div>
          <div className="controls flex items-center gap-2">
            <button id="titlebar-minimize" title="minimize">
              {/* https://api.iconify.design/mdi:window-minimize.svg */}
              {/* <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
              >
                <path fill="currentColor" d="M19 13H5v-2h14z" />
              </svg> */}
              <ChevronDownIcon />
            </button>
            <button id="titlebar-maximize" title="maximize">
              {/* https://api.iconify.design/mdi:window-maximize.svg */}
              {/* <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
              >
                <path fill="currentColor" d="M4 4h16v16H4zm2 4v10h12V8z" />
              </svg> */}
              <Maximize2Icon />
            </button>
            <button id="titlebar-close" title="close">
              {/* https://api.iconify.design/mdi:close.svg */}
              {/* <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
              >
                <path
                  fill="currentColor"
                  d="M13.46 12L19 17.54V19h-1.46L12 13.46L6.46 19H5v-1.46L10.54 12L5 6.46V5h1.46L12 10.54L17.54 5H19v1.46z"
                />
              </svg> */}
              <XIcon />
            </button>
          </div>
        </div>
        <div className="content-container">
          <Providers>{children}</Providers>
        </div>
      </body>
    </html>
  );
}
