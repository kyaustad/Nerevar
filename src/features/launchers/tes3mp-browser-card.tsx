"use client";

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTES3MPBrowser } from "@/hooks/use-tes3mp-browser";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Loader2, Play, CheckCircle, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import Image from "next/image";

export function Tes3MPBrowserCard({ className }: { className?: string }) {
  const [isStarting, setIsStarting] = useState(false);
  const { isRunning, browserPid, lastResult } = useTES3MPBrowser();

  const handleRunLauncher = async () => {
    setIsStarting(true);
    try {
      const result = await invoke("run_tes3mp_browser");
      console.log("TES3MP browser start result:", result);
    } catch (error) {
      console.error("Failed to start TES3MP browser:", error);
    } finally {
      setIsStarting(false);
    }
  };

  const getStatusBadge = () => {
    if (isStarting) {
      return <Badge variant="secondary">Starting...</Badge>;
    }
    if (isRunning) {
      return <Badge variant="default">Running (PID: {browserPid})</Badge>;
    }
    if (lastResult) {
      return lastResult.success ? (
        <Badge variant="default" className="bg-green-500">
          <CheckCircle className="w-3 h-3 mr-1" />
          Closed
        </Badge>
      ) : (
        <Badge variant="destructive">
          <XCircle className="w-3 h-3 mr-1" />
          Failed
        </Badge>
      );
    }
    return <Badge variant="outline">Ready</Badge>;
  };

  return (
    <Card className={cn("w-full ", className)}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 w-full">
          <Image
            src="/multiplayer-icon.webp"
            alt="Logo"
            width={32}
            height={32}
          />
          Server Browser
        </CardTitle>
        <CardDescription className="text-xs">
          Launch the Tes3MP server browser
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4 flex flex-col p-2 h-full justify-between">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">Status:</span>
          {getStatusBadge()}
        </div>

        {/* {lastResult && (
          <div className="p-3 bg-muted rounded-md">
            <div className="text-sm">
              <div className="font-medium mb-1">Last Result:</div>
              <div className="text-muted-foreground">{lastResult.message}</div>
              {lastResult.exit_code !== null && (
                <div className="text-xs mt-1">
                  Exit Code: {lastResult.exit_code}
                </div>
              )}
            </div>
          </div>
        )} */}

        <Button
          onClick={handleRunLauncher}
          disabled={isStarting || isRunning}
          className="w-full"
        >
          {isStarting ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Starting Browser...
            </>
          ) : isRunning ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Browser Running...
            </>
          ) : (
            <>
              <Play className="w-4 h-4 mr-2" />
              Run Server Browser
            </>
          )}
        </Button>

        {isRunning && (
          <p className="text-xs text-muted-foreground text-center">
            {`The Server Browser is running.`}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
