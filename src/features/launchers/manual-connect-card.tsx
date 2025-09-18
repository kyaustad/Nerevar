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

import { ChevronsLeftRightEllipsisIcon, LinkIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { Input } from "@/components/ui/input";

export function ManualConnectCard({ className }: { className?: string }) {
  const [ip, setIp] = useState("127.0.0.1");
  const [port, setPort] = useState("25565");
  const [password, setPassword] = useState("");

  const handleConnect = async () => {
    try {
      const result = await invoke("set_tes3mp_client_config", {
        ip,
        port: parseInt(port),
        password,
      });
      if (!result) {
        throw new Error("Failed to update client config to connect to server");
      }
      await invoke("run_tes3mp");
    } catch (error) {
      console.error("Failed to connect to server:", error);
    }
  };

  return (
    <Card className={cn("w-full ", className)}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 w-full">
          {/* <Image src="/tes3mp_logo.png" alt="Logo" width={32} height={32} /> */}
          <LinkIcon className="w-4 h-4" />
          Manually Connect
        </CardTitle>
        <CardDescription className="text-xs">
          {`Manually connect to a server using its IP, Port and Password`}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2 flex flex-col p-2 h-full justify-between">
        <div className="flex flex-row gap-2">
          <Input
            placeholder="IP"
            value={ip}
            className="w-2/3"
            onChange={(e) => setIp(e.target.value)}
          />
          <Input
            placeholder="Port"
            type="number"
            value={port}
            className="w-1/3"
            onChange={(e) => setPort(e.target.value)}
          />
        </div>
        <Input
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <Button onClick={handleConnect}>
          <ChevronsLeftRightEllipsisIcon className="w-4 h-4" />
          Connect
        </Button>
      </CardContent>
    </Card>
  );
}
