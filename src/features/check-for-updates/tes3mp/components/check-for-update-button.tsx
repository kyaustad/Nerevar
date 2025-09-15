"use client";

import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { Download, Loader2, RotateCwIcon } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

type UpdateCheckResponse = {
  update_available: boolean;
  version: string;
};

export function CheckForTes3MpUpdateButton({
  className,
}: {
  className?: string;
}) {
  const [isCheckingForUpdate, setIsCheckingForUpdate] = useState(false);
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [updateVersion, setUpdateVersion] = useState("");

  const handleCheckForUpdate = async () => {
    setIsCheckingForUpdate(true);
    try {
      const result: UpdateCheckResponse = await invoke(
        "check_for_tes3mp_update"
      );
      console.log("TES3MP update result:", result);
      if (result.update_available) {
        toast.info(`Tes3MP ${result.version} is available`);
        setUpdateAvailable(true);
        setUpdateVersion(result.version);
      } else {
        toast.info(`Tes3MP is up to date!`);
        setUpdateAvailable(false);
        setUpdateVersion("");
      }
    } catch (error) {
      toast.error("Failed to check for update");
      console.error("Failed to check for update:", error);
    } finally {
      setIsCheckingForUpdate(false);
    }
  };
  const handleDownloadUpdate = async () => {
    try {
      toast.promise(invoke("download_latest_windows_release"), {
        loading: "Downloading update...",
        success: "Update downloaded successfully!",
        error: "Error downloading update",
      });
      setUpdateAvailable(false);
      setUpdateVersion("");
    } catch (error) {
      toast.error("Failed to download update");
      console.error("Failed to download update:", error);
    }
  };

  if (updateAvailable) {
    return (
      <Button className={className} onClick={handleDownloadUpdate}>
        <Download className="mr-2" />
        Download Update
      </Button>
    );
  }

  return (
    <Button
      onClick={handleCheckForUpdate}
      disabled={isCheckingForUpdate}
      className={className}
    >
      {isCheckingForUpdate ? (
        <>
          <Loader2 className="animate-spin mr-2" />
          {`Checking...`}
        </>
      ) : (
        <>
          <RotateCwIcon className="mr-2" />
          {`Check for Update`}
        </>
      )}
    </Button>
  );
}
