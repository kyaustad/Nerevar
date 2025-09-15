"use client";

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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
import { useTES3MP } from "@/hooks/use-tes3mp";

export function Tes3MPCard({ className }: { className?: string }) {
  const [isStarting, setIsStarting] = useState(false);
  const { isRunning, tes3mpPid, lastResult } = useTES3MP();

  const handleRunLauncher = async () => {
    setIsStarting(true);
    try {
      const result = await invoke("run_tes3mp");
      console.log("TES3MP start result:", result);
    } catch (error) {
      console.error("Failed to start TES3MP:", error);
    } finally {
      setIsStarting(false);
    }
  };

  const getStatusBadge = () => {
    if (isStarting) {
      return <Badge variant="secondary">Starting...</Badge>;
    }
    if (isRunning) {
      return <Badge variant="default">Running (PID: {tes3mpPid})</Badge>;
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
          <Image src="/tes3mp_logo.png" alt="Logo" width={32} height={32} />
          Tes3MP
        </CardTitle>
        <CardDescription className="text-xs">
          Launch Tes3MP Morrowind Multiplayer to your last connected server
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
              Starting Tes3MP...
            </>
          ) : isRunning ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Tes3MP Running...
            </>
          ) : (
            <>
              <Play className="w-4 h-4 mr-2" />
              Run Tes3MP
            </>
          )}
        </Button>

        {isRunning && (
          <p className="text-xs text-muted-foreground text-center">
            {`Tes3MP is running.`}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
