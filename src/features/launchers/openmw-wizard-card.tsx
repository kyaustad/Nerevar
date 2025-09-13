"use client";

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useOpenMWWizard } from "@/hooks/use-openmw-wizard";
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

export function OpenMWWizardCard({ className }: { className?: string }) {
  const [isStarting, setIsStarting] = useState(false);
  const { isRunning, wizardPid, lastResult } = useOpenMWWizard();

  const handleRunWizard = async () => {
    setIsStarting(true);
    try {
      const result = await invoke("run_openmw_wizard");
      console.log("Wizard start result:", result);
    } catch (error) {
      console.error("Failed to start wizard:", error);
    } finally {
      setIsStarting(false);
    }
  };

  const getStatusBadge = () => {
    if (isStarting) {
      return <Badge variant="secondary">Starting...</Badge>;
    }
    if (isRunning) {
      return <Badge variant="default">Running (PID: {wizardPid})</Badge>;
    }
    if (lastResult) {
      return lastResult.success ? (
        <Badge variant="default" className="bg-green-500">
          <CheckCircle className="w-3 h-3 mr-1" />
          Completed
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
        <CardTitle className="flex items-center gap-2">
          <Image src="/openmw-wizard.png" alt="Logo" width={32} height={32} />
          OpenMW Wizard
        </CardTitle>
        <CardDescription>
          Launch the OpenMW configuration wizard for Tes3MP
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
          onClick={handleRunWizard}
          disabled={isStarting || isRunning}
          className="w-full"
        >
          {isStarting ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Starting Wizard...
            </>
          ) : isRunning ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Wizard Running...
            </>
          ) : (
            <>
              <Play className="w-4 h-4 mr-2" />
              Run OpenMW Wizard
            </>
          )}
        </Button>

        {isRunning && (
          <p className="text-xs text-muted-foreground text-center">
            The wizard is running. You'll be notified when it completes.
          </p>
        )}
      </CardContent>
    </Card>
  );
}
