" use client";

//import { env } from "@/env";
import { useConfig } from "@/features/app-config/context/config-context";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription } from "@/components/ui/alert";

export function CheckForAppUpdate({ className }: { className?: string }) {
  const { config, isLoading } = useConfig();
  const [appUpdateAvailable, setAppUpdateAvailable] = useState(false);
  const [appUpdateVersion, setAppUpdateVersion] = useState("");
  const [currentVersion, setCurrentVersion] = useState("");
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);
  const [updateStage, setUpdateStage] = useState("");
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const getCurrentAppVersion = async (): Promise<string> => {
    try {
      const version = await invoke("get_app_version");
      return version as string;
    } catch (error) {
      console.log("Failed to get app version:", error);
      setError(error as string);
      return "";
    }
  };

  const checkForAppUpdate = async (): Promise<{
    update_available: boolean;
    version: string;
    url?: string;
  }> => {
    try {
      const result = await invoke("check_for_app_update");
      return result as {
        update_available: boolean;
        version: string;
        url?: string;
      };
    } catch (error) {
      console.log("Failed to check for app update:", error);
      setError(error as string);
      return {
        update_available: false,
        version: "",
      };
    }
  };

  useEffect(() => {
    const checkForUpdate = async () => {
      if (!isLoading && config) {
        try {
          const currentVer = await getCurrentAppVersion();
          setCurrentVersion(currentVer);

          const { update_available, version, url } = await checkForAppUpdate();
          setAppUpdateAvailable(update_available);
          setAppUpdateVersion(version);
          setDownloadUrl(url || null);
        } catch (error) {
          console.log("Failed to check for app update:", error);
          setError(error as string);
        }
      }
    };
    checkForUpdate();
  }, [isLoading, config]);

  const handleUpdate = async () => {
    if (!downloadUrl) {
      setUpdateError("No download URL available");
      return;
    }

    setIsUpdating(true);
    setUpdateError(null);
    setUpdateProgress(0);
    setUpdateStage("Downloading update...");

    try {
      // Step 1: Download the update
      setUpdateProgress(25);
      const tempFilePath = await invoke("download_app_update", { downloadUrl });

      // Step 2: Apply the update (this will restart the app automatically)
      setUpdateProgress(50);
      setUpdateStage("Applying update...");
      await invoke("apply_app_update", { tempFilePath });

      setUpdateProgress(100);
      setUpdateStage("Update complete! Restarting...");
    } catch (error) {
      console.log("Update failed:", error);
      setError(error as string);
      setUpdateError(error as string);
      setIsUpdating(false);
    }
  };

  return (
    <div className={className}>
      {appUpdateAvailable && (
        <div className="flex flex-col gap-3 text-xs">
          <div className="text-white text-center">
            <p>{`A new version of Nerevar is available`}</p>
            <p>
              {`Current Version: `}
              {currentVersion}
            </p>
            <p>
              {`New Version: `}
              {appUpdateVersion}
            </p>
          </div>

          {updateError && (
            <Alert variant="destructive">
              <AlertDescription>{updateError}</AlertDescription>
            </Alert>
          )}

          {isUpdating && (
            <div className="space-y-2">
              <div className="text-white">
                <p>{updateStage}</p>
              </div>
              <Progress value={updateProgress} className="w-full" />
            </div>
          )}

          {!isUpdating && downloadUrl && (
            <Button onClick={handleUpdate} size="sm" className="w-full">
              Update Now
            </Button>
          )}
        </div>
      )}
      {!appUpdateAvailable && currentVersion && (
        <div className="flex flex-col gap-2 text-xs text-muted-foreground">
          <p>You are using the latest version of Nerevar</p>
        </div>
      )}
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
    </div>
  );
}
