"use client";

import { useState, useEffect, useMemo } from "react";
import { fetch } from "@tauri-apps/plugin-http";
import { env } from "@/env";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import {
  WorkflowIcon,
  Lock,
  ComputerIcon,
  StarIcon,
  Loader2,
  Search,
  Filter,
  SortAsc,
  SortDesc,
  Users,
  Code,
  RefreshCcw,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useQuery } from "@tanstack/react-query";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

type ServerResponseType = {
  [key: string]: {
    modname: string;
    passw: boolean;
    hostname: string;
    query_port: number;
    last_update: number;
    players: number;
    version: string;
    max_players: number;
  };
};

type SortOption =
  | "name-asc"
  | "name-desc"
  | "players-asc"
  | "players-desc"
  | "last-update-asc"
  | "last-update-desc"
  | "version-asc"
  | "version-desc";

type FilterOptions = {
  search: string;
  playerCount: "all" | "empty" | "low" | "medium" | "high" | "full";
  passwordProtected: "all" | "yes" | "no";
  version: string;
  sortBy: SortOption;
};

// Ping utility function
async function pingServer(ip: string, port: number): Promise<number> {
  const startTime = Date.now();

  try {
    const result = await invoke("ping_server_tcp", { ip, port });
    if (result) {
      return Number(result);
    } else {
      return 9999;
    }
  } catch (error) {
    // If the server doesn't respond to HTTP, try a TCP connection simulation
    // This is a simplified approach - in a real implementation you might want
    // to use a proper TCP ping or ICMP ping if available
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);

      // Try to fetch a simple endpoint or just test connectivity
      await fetch(`http://${ip}`, {
        method: "HEAD",
        signal: controller.signal,
        headers: {
          "User-Agent": "Nerevar-Launcher/1.0",
        },
      });

      clearTimeout(timeoutId);
      const endTime = Date.now();
      return endTime - startTime;
    } catch {
      // If all else fails, return a high ping value to indicate poor connectivity
      return 9999;
    }
  }
}

export function ServerList({ className }: { className?: string }) {
  const [filters, setFilters] = useState<FilterOptions>({
    search: "",
    playerCount: "all",
    passwordProtected: "all",
    version: "all",
    sortBy: "players-desc",
  });

  const fetchServers = async () => {
    const response = await fetch(
      `${env.NEXT_PUBLIC_NEREVAR_API_URL}/servers/list`
    );
    const data = await response.json();
    return data.data as ServerResponseType;
  };

  const {
    data: servers,
    isLoading,
    error,
    refetch,
    isRefetching,
  } = useQuery({
    queryKey: ["servers"],
    queryFn: () => fetchServers(),
  });

  // Filter and sort servers
  const filteredAndSortedServers = useMemo(() => {
    if (!servers) return [];

    let serverEntries = Object.entries(servers);

    // Apply search filter
    if (filters.search) {
      const searchLower = filters.search.toLowerCase();
      serverEntries = serverEntries.filter(
        ([_, server]) =>
          server.hostname.toLowerCase().includes(searchLower) ||
          server.modname.toLowerCase().includes(searchLower)
      );
    }

    // Apply player count filter
    if (filters.playerCount !== "all") {
      serverEntries = serverEntries.filter(([_, server]) => {
        const playerRatio = server.players / server.max_players;
        switch (filters.playerCount) {
          case "empty":
            return server.players === 0;
          case "low":
            return playerRatio > 0 && playerRatio <= 0.25;
          case "medium":
            return playerRatio > 0.25 && playerRatio <= 0.75;
          case "high":
            return playerRatio > 0.75 && playerRatio < 1;
          case "full":
            return server.players === server.max_players;
          default:
            return true;
        }
      });
    }

    // Apply password protection filter
    if (filters.passwordProtected !== "all") {
      serverEntries = serverEntries.filter(([_, server]) => {
        if (filters.passwordProtected === "yes") return server.passw;
        if (filters.passwordProtected === "no") return !server.passw;
        return true;
      });
    }

    // Apply version filter
    if (filters.version !== "all") {
      serverEntries = serverEntries.filter(
        ([_, server]) => server.version === filters.version
      );
    }

    // Apply sorting
    serverEntries.sort(([keyA, serverA], [keyB, serverB]) => {
      switch (filters.sortBy) {
        case "name-asc":
          return serverA.hostname.localeCompare(serverB.hostname);
        case "name-desc":
          return serverB.hostname.localeCompare(serverA.hostname);
        case "players-asc":
          return serverA.players - serverB.players;
        case "players-desc":
          return serverB.players - serverA.players;
        case "last-update-asc":
          return serverA.last_update - serverB.last_update;
        case "last-update-desc":
          return serverB.last_update - serverA.last_update;
        case "version-asc":
          return serverA.version.localeCompare(serverB.version);
        case "version-desc":
          return serverB.version.localeCompare(serverA.version);
        default:
          return 0;
      }
    });

    return serverEntries;
  }, [servers, filters]);

  // Get unique versions for filter dropdown
  const availableVersions = useMemo(() => {
    if (!servers) return [];
    const versions = new Set(
      Object.values(servers).map((server) => server.version)
    );
    return Array.from(versions).sort();
  }, [servers]);

  console.log("servers:", servers);

  const [accordionValue, setAccordionValue] = useState("filters");
  const [connectDialogOpen, setConnectDialogOpen] = useState(false);
  const [selectedServer, setSelectedServer] = useState<
    ServerResponseType[keyof ServerResponseType] | null
  >(null);
  const [selectedServerKey, setSelectedServerKey] = useState<string>("");

  return (
    <Card className={cn("w-full", className)}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Filter className="w-5 h-5" />
          All Servers
          <Badge variant="secondary" className="ml-auto">
            {filteredAndSortedServers.length} servers
          </Badge>
        </CardTitle>
        <CardDescription>
          {`List of servers publicly available via the Master Server`}
        </CardDescription>
        <div className="flex flex-row gap-2 items-center justify-between">
          <Button
            variant="outline"
            size="sm"
            onClick={() => refetch()}
            disabled={isRefetching}
          >
            <RefreshCcw className="w-4 h-4" />
            {isRefetching ? "Refreshing..." : "Refresh"}
          </Button>
        </div>

        {/* Filter and Search Controls */}
        <Accordion
          type="single"
          collapsible
          value={accordionValue}
          onValueChange={setAccordionValue}
        >
          <AccordionItem value="filters">
            <AccordionTrigger>Filter and Sort</AccordionTrigger>
            <AccordionContent>
              <div className="flex flex-col gap-3 mt-4">
                {/* Search Input */}
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
                  <Input
                    placeholder="Search servers by name or mode..."
                    value={filters.search}
                    onChange={(e) =>
                      setFilters((prev) => ({
                        ...prev,
                        search: e.target.value,
                      }))
                    }
                    className="pl-10"
                  />
                </div>

                {/* Filter Controls Row */}
                <div className="flex flex-wrap gap-2">
                  {/* Player Count Filter */}
                  <Select
                    value={filters.playerCount}
                    onValueChange={(value: FilterOptions["playerCount"]) =>
                      setFilters((prev) => ({ ...prev, playerCount: value }))
                    }
                  >
                    <SelectTrigger className="w-[140px]">
                      <SelectValue placeholder="Players" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Players</SelectItem>
                      <SelectItem value="empty">Empty (0)</SelectItem>
                      <SelectItem value="low">Low (1-25%)</SelectItem>
                      <SelectItem value="medium">Medium (26-75%)</SelectItem>
                      <SelectItem value="high">High (76-99%)</SelectItem>
                      <SelectItem value="full">Full (100%)</SelectItem>
                    </SelectContent>
                  </Select>

                  {/* Password Protection Filter */}
                  <Select
                    value={filters.passwordProtected}
                    onValueChange={(
                      value: FilterOptions["passwordProtected"]
                    ) =>
                      setFilters((prev) => ({
                        ...prev,
                        passwordProtected: value,
                      }))
                    }
                  >
                    <SelectTrigger className="w-[140px]">
                      <SelectValue placeholder="Password" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Servers</SelectItem>
                      <SelectItem value="no">No Password</SelectItem>
                      <SelectItem value="yes">Password Required</SelectItem>
                    </SelectContent>
                  </Select>

                  {/* Version Filter */}
                  <Select
                    value={filters.version}
                    onValueChange={(value) =>
                      setFilters((prev) => ({ ...prev, version: value }))
                    }
                  >
                    <SelectTrigger className="w-[120px]">
                      <SelectValue placeholder="Version" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Versions</SelectItem>
                      {availableVersions.map((version) => (
                        <SelectItem key={version} value={version}>
                          v{version}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>

                  {/* Sort Options */}
                  <Select
                    value={filters.sortBy}
                    onValueChange={(value: SortOption) =>
                      setFilters((prev) => ({ ...prev, sortBy: value }))
                    }
                  >
                    <SelectTrigger className="w-full">
                      <SelectValue placeholder="Sort by" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="players-desc">
                        <div className="flex items-center gap-2">
                          <Users className="w-4 h-4" />
                          Players (High to Low)
                        </div>
                      </SelectItem>
                      <SelectItem value="players-asc">
                        <div className="flex items-center gap-2">
                          <Users className="w-4 h-4" />
                          Players (Low to High)
                        </div>
                      </SelectItem>
                      <SelectItem value="name-asc">
                        <div className="flex items-center gap-2">
                          <SortAsc className="w-4 h-4" />
                          Name (A-Z)
                        </div>
                      </SelectItem>
                      <SelectItem value="name-desc">
                        <div className="flex items-center gap-2">
                          <SortDesc className="w-4 h-4" />
                          Name (Z-A)
                        </div>
                      </SelectItem>
                      <SelectItem value="last-update-desc">
                        <div className="flex items-center gap-2">
                          <Code className="w-4 h-4" />
                          Recently Updated
                        </div>
                      </SelectItem>
                      <SelectItem value="version-desc">
                        <div className="flex items-center gap-2">
                          <Code className="w-4 h-4" />
                          Version (Newest)
                        </div>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </CardHeader>

      <CardContent className="flex flex-col gap-2 max-h-72 overflow-y-auto bg-accent/20 p-2 rounded-lg">
        {(isLoading || isRefetching) && (
          <div className="flex flex-row gap-2 items-center justify-center">
            <Loader2 className="min-w-4 min-h-4 max-w-4 max-h-4 text-white animate-spin" />
            <p className="text-sm text-white/80">Loading...</p>
          </div>
        )}
        {error && <div>Error: {error.message || "Unknown error"}</div>}
        {filteredAndSortedServers.length === 0 && !isLoading && !error && (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Filter className="w-8 h-8 text-muted-foreground mb-2" />
            <p className="text-sm text-muted-foreground">
              No servers match your filters
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={() =>
                setFilters({
                  search: "",
                  playerCount: "all",
                  passwordProtected: "all",
                  version: "all",
                  sortBy: "players-desc",
                })
              }
              className="mt-2"
            >
              Clear Filters
            </Button>
          </div>
        )}
        {filteredAndSortedServers.length > 0 && (
          <div className="flex flex-col gap-2">
            {filteredAndSortedServers.map(([serverKey, server]) => (
              <ServerEntry
                key={serverKey}
                server={server}
                serverKey={serverKey}
                onConnectClicked={() => {
                  setSelectedServer(server);
                  setSelectedServerKey(serverKey);
                  setConnectDialogOpen(true);
                }}
              />
            ))}
          </div>
        )}
      </CardContent>
      <ConnectDialog
        server={selectedServer}
        serverKey={selectedServerKey}
        open={connectDialogOpen}
        onOpenChange={(open) => {
          setConnectDialogOpen(open);
          if (!open) {
            setSelectedServer(null);
            setSelectedServerKey("");
          }
        }}
      />
    </Card>
  );
}

function ServerEntry({
  server,
  serverKey,
  onConnectClicked,
}: {
  server: ServerResponseType[keyof ServerResponseType] | null;
  serverKey: string;
  onConnectClicked: () => void;
}) {
  if (!server) return null;
  return (
    <div className="flex flex-col gap-2 border-2 border-border/75 rounded-lg p-2 bg-primary/50">
      <div className="flex flex-row gap-2 items-center justify-between w-full ">
        <div className="flex flex-col gap-1 h-full justify-between flex-1">
          <div className="flex flex-row gap-2 items-center">
            <p className="text-lg text-white/80 truncate text-ellipsis text-wrap">
              {server.hostname}
            </p>
          </div>
          <div className="flex flex-row gap-2 items-center">
            <Badge variant="secondary">
              <p className="text-xs text-white/80">
                {`${server.players} / ${server.max_players} players`}
              </p>
            </Badge>
            <Badge>
              <p className="text-xs text-black/80">{`Tes3MP: v${server.version}`}</p>
            </Badge>
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge variant="outline">
                  <p className="text-xs text-white/80">
                    {`Mode: `}{" "}
                    <span className="text-accent-foreground/80 font-bold">
                      {server.modname.slice(0, 10)}
                      {server.modname.length > 10 && "..."}
                    </span>
                  </p>
                </Badge>
              </TooltipTrigger>
              <TooltipContent>{server.modname}</TooltipContent>
            </Tooltip>
          </div>
          <div className="flex flex-row gap-2 items-center">
            <div className="flex items-center justify-center p-1 bg-background/20 rounded-lg gap-1 w-full">
              <ComputerIcon className="min-w-4 min-h-4 max-w-4 max-h-4 text-blue-500" />
              <div className="flex flex-row gap-1 w-full justify-around">
                <p className="text-xs text-white/80">
                  IP: {serverKey.split(":")[0]}
                </p>
                <p className="text-xs text-white/80">
                  Port: {serverKey.split(":")[1]}
                </p>
              </div>
            </div>
          </div>
        </div>
        <div className="flex flex-col gap-2 items-center justify-center">
          <Button variant="outline" size="sm" className="w-full">
            <StarIcon className="min-w-4 min-h-4 max-w-4 max-h-5 text-white" />
            Favorite
          </Button>
          <Button variant="secondary" onClick={onConnectClicked}>
            <WorkflowIcon className="min-w-5 min-h-5 max-w-5 max-h-5 text-white" />
            Connect
          </Button>
          {server.passw && (
            <div className="flex items-center justify-center p-1 bg-background/20 rounded-lg gap-1 w-full">
              <Lock className="min-w-4 min-h-4 max-w-4 max-h-4 text-red-500" />
              <p className="text-xs text-white/80">Password</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ConnectDialog({
  server,
  serverKey,
  open,
  onOpenChange,
}: {
  server: ServerResponseType[keyof ServerResponseType] | null;
  serverKey: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [password, setPassword] = useState("");
  const [ping, setPing] = useState<number | null>(null);
  const [pingStatus, setPingStatus] = useState<
    "idle" | "pinging" | "success" | "error"
  >("idle");
  const [isConnecting, setIsConnecting] = useState(false);

  useEffect(() => {
    if (server && open) {
      // Reset state when dialog opens
      setPing(null);
      setPingStatus("idle");
      setPassword("");

      // Start pinging immediately when dialog opens
      performPing();
    }
  }, [server, open]);

  const performPing = async () => {
    if (!server || !serverKey) return;

    setPingStatus("pinging");
    try {
      const [ip, port] = serverKey.split(":");
      const pingResult = await pingServer(ip, parseInt(port));

      if (pingResult === 9999) {
        setPingStatus("error");
        setPing(null);
      } else {
        setPing(pingResult);
        setPingStatus("success");
      }
    } catch (error) {
      console.error("Ping failed:", error);
      setPingStatus("error");
      setPing(null);
    }
  };

  const handleConnect = async () => {
    if (!server || !serverKey) return;

    setIsConnecting(true);
    try {
      // Update TES3MP client config with server details
      const [ip, port] = serverKey.split(":");
      const success = await invoke("set_tes3mp_client_config", {
        ip,
        port: parseInt(port),
        password: password,
      });

      if (success) {
        console.log("TES3MP client config updated successfully");
        // Here you could also launch TES3MP automatically
        await invoke("run_tes3mp");
      } else {
        throw new Error("Failed to update TES3MP client config");
      }

      // Connection successful - close dialog
      onOpenChange(false);
    } catch (error) {
      console.error("Connection failed:", error);
      // You might want to show an error message to the user here
      toast.error(
        "Failed to update TES3MP client config when trying to connect"
      );
    } finally {
      setIsConnecting(false);
    }
  };

  const getPingColor = (ping: number | null) => {
    if (ping === null) return "text-muted-foreground";
    if (ping < 50) return "text-green-500";
    if (ping < 100) return "text-yellow-500";
    if (ping < 200) return "text-orange-500";
    return "text-red-500";
  };

  const getPingStatusText = () => {
    switch (pingStatus) {
      case "pinging":
        return "Pinging...";
      case "success":
        return `${ping}ms`;
      case "error":
        return "Unreachable";
      default:
        return "Not tested";
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <WorkflowIcon className="w-5 h-5" />
            Connect to {server?.hostname || "Server"}
          </DialogTitle>
          <DialogDescription>Connect to this TES3MP server</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Server Info */}
          <div className="space-y-2">
            <div className="flex justify-between items-center">
              <span className="text-sm font-medium">Server:</span>
              <span className="text-sm text-muted-foreground">
                {server?.hostname}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm font-medium">Players:</span>
              <span className="text-sm text-muted-foreground">
                {server?.players} / {server?.max_players}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm font-medium">Version:</span>
              <span className="text-sm text-muted-foreground">
                TES3MP v{server?.version}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm font-medium">Mode:</span>
              <span className="text-sm text-muted-foreground">
                {server?.modname}
              </span>
            </div>
          </div>

          {/* Ping Status */}
          <div className="flex justify-between items-center">
            <span className="text-sm font-medium">Ping:</span>
            <div className="flex items-center gap-2">
              {pingStatus === "pinging" && (
                <Loader2 className="w-4 h-4 animate-spin text-blue-500" />
              )}
              <span className={`text-sm ${getPingColor(ping)}`}>
                {getPingStatusText()}
              </span>
              {pingStatus !== "pinging" && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={performPing}
                  className="h-6 px-2 text-xs"
                >
                  Retry
                </Button>
              )}
            </div>
          </div>

          {/* Password Input */}
          {server?.passw && (
            <div className="space-y-2">
              <Label htmlFor="password">Server Password</Label>
              <Input
                id="password"
                type="password"
                placeholder="Enter server password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full"
              />
            </div>
          )}

          {/* Connection Buttons */}
          <div className="flex gap-2 pt-2">
            <Button
              variant="outline"
              onClick={() => onOpenChange(false)}
              className="flex-1"
              disabled={isConnecting}
            >
              Cancel
            </Button>
            <Button
              onClick={handleConnect}
              disabled={isConnecting || (server?.passw && !password.trim())}
              className="flex-1"
            >
              {isConnecting ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Connecting...
                </>
              ) : (
                <>
                  <WorkflowIcon className="w-4 h-4 mr-2" />
                  Connect
                </>
              )}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
