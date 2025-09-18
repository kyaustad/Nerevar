"use client";

import { useForm } from "react-hook-form";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useEffect, useState } from "react";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { Tes3MPServerConfig } from "@/types/server-cfg";
import { invoke } from "@tauri-apps/api/core";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { SaveIcon, TrashIcon } from "lucide-react";
import { toast } from "sonner";

const formSchema = z.object({
  general: z.object({
    local_address: z.string().min(1),
    port: z.number().min(1),
    maximum_players: z.number().min(1),
    hostname: z.string().min(1),
    log_level: z.string().min(1),
    password: z.optional(z.string()),
  }),
  plugins: z.object({
    home: z.string().min(1),
    plugins: z.string().min(1),
  }),
  master_server: z.object({
    enabled: z.boolean(),
    address: z.string().min(1),
    port: z.number().min(1),
    rate: z.number().min(1),
  }),
});

export function ServerConfigurationForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      general: {
        local_address: "",
        port: 0,
        maximum_players: 0,
        hostname: "",
        log_level: "0",
        password: "",
      },
      plugins: {
        home: "",
        plugins: "",
      },
      master_server: {
        enabled: false,
        address: "",
        port: 0,
        rate: 0,
      },
    },
  });

  const getServerConfig = async () => {
    const configResponse = await invoke("get_tes3mp_server_config");
    const config = configResponse as Tes3MPServerConfig;
    console.log("Read Server Config:", config);
    // setServerConfig(config as Tes3MPServerConfig);
    form.reset({
      general: {
        local_address: config.general.local_address,
        port: config.general.port,
        maximum_players: config.general.maximum_players,
        hostname: config.general.hostname,
        log_level: config.general.log_level.toString(),
        password: config.general.password,
      },
      plugins: {
        home: config.plugins.home,
        plugins: config.plugins.plugins,
      },
      master_server: {
        enabled: config.master_server.enabled,
        address: config.master_server.address,
        port: config.master_server.port,
        rate: config.master_server.rate,
      },
    });
  };
  useEffect(() => {
    getServerConfig();
  }, []);

  function onSubmit(values: z.infer<typeof formSchema>) {
    console.log(values);
    try {
      toast.promise(
        invoke("set_tes3mp_server_config", {
          config: {
            general: {
              ...values.general,
              localAddress: values.general.local_address,
              maximumPlayers: values.general.maximum_players,
              logLevel: Number(values.general.log_level),
            },
            masterServer: {
              ...values.master_server,
            },
          },
        }),
        {
          loading: "Saving config...",
          success: "Server config saved successfully",
          error: "Failed to save server config",
        }
      );
    } catch (error) {
      console.error("Failed to save server config:", error);
      toast.error("Failed to save server config");
    }
  }

  function resetToDefaults() {
    try {
      toast.promise(
        invoke("set_tes3mp_server_config", {
          config: {
            general: {
              localAddress: "0.0.0.0",
              port: 25565,
              maximumPlayers: 64,
              logLevel: 1,
              password: "",
              hostname: "TES3MP Server",
            },
            masterServer: {
              enabled: true,
              address: "master.tes3mp.com",
              port: 25561,
              rate: 10000,
            },
          },
        }),
        {
          loading: "Resetting to defaults...",
          success: async (data) => {
            await getServerConfig();
            return "Server config reset to defaults";
          },
          error: "Failed to reset to defaults",
        }
      );
    } catch (error) {
      console.error("Failed to reset to defaults:", error);
      toast.error("Failed to reset to defaults");
    }
  }

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className="flex flex-col gap-8 w-full"
      >
        <div className="flex flex-row gap-4 w-full h-full place-items-start">
          <div className="flex flex-col gap-2 w-1/3">
            <FormField
              control={form.control}
              name="general.local_address"
              render={({ field }) => (
                <FormItem className="h-full gap-y-1">
                  <FormLabel className="text-lg font-medium">
                    Local Address
                  </FormLabel>
                  <FormDescription className="max-w-sm text-wrap">
                    The local address of your server.
                    <br />
                  </FormDescription>
                  <FormControl>
                    <Input {...field} value={field.value || ""} />
                  </FormControl>
                  <span className="text-xs text-muted-foreground italic max-w-sm text-wrap">
                    {`The default localAddress of "0.0.0.0" makes the server reachable at all of its local addresses. You almost never have to change this`}
                  </span>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="general.port"
              render={({ field }) => (
                <FormItem className="h-full gap-y-1">
                  <FormLabel className="text-lg font-medium">Port</FormLabel>
                  <FormDescription className="max-w-sm text-wrap">
                    The port of your server.
                  </FormDescription>
                  <FormControl>
                    <Input
                      type="number"
                      min={1}
                      step={1}
                      {...field}
                      value={field.value || ""}
                      onChange={(e) => field.onChange(Number(e.target.value))}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="general.maximum_players"
              render={({ field }) => (
                <FormItem className="h-full gap-y-1">
                  <FormLabel className="text-lg font-medium">
                    Maximum Players
                  </FormLabel>
                  <FormDescription className="max-w-sm text-wrap">
                    The maximum number of players that can connect to your
                    server at once.
                  </FormDescription>
                  <FormControl>
                    <Input
                      type="number"
                      min={1}
                      max={64}
                      step={1}
                      {...field}
                      value={field.value || ""}
                      onChange={(e) => field.onChange(Number(e.target.value))}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="general.hostname"
              render={({ field }) => (
                <FormItem className="h-full gap-y-1">
                  <FormLabel className="text-lg font-medium">
                    Hostname
                  </FormLabel>
                  <FormDescription className="max-w-sm text-wrap">
                    The name of your server that will be displayed to players in
                    the server list.
                  </FormDescription>
                  <FormControl>
                    <Input type="text" {...field} value={field.value || ""} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>
          <div className="flex flex-col gap-2 w-2/3">
            <FormField
              control={form.control}
              name="general.password"
              render={({ field }) => (
                <FormItem className="h-full gap-y-1">
                  <FormLabel className="text-lg font-medium">
                    Password
                  </FormLabel>
                  <FormDescription className="max-w-sm text-wrap">
                    The password to access your server.
                  </FormDescription>
                  <FormControl>
                    <Input
                      type="password"
                      {...field}
                      value={field.value || ""}
                    />
                  </FormControl>
                  <span className="text-xs text-muted-foreground italic max-w-sm text-wrap">
                    {`If you don't want to use a password, leave this field empty`}
                  </span>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="general.log_level"
              render={({ field }) => (
                <FormItem className="h-full gap-y-1">
                  <FormLabel className="text-lg font-medium">
                    Log Level
                  </FormLabel>
                  <FormDescription className="max-w-sm text-wrap">
                    Determines how verbose the server logs will be.
                  </FormDescription>
                  <FormControl>
                    <Select
                      onValueChange={(value) => field.onChange(value)}
                      value={field.value}
                    >
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="Select a log level" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="0">{`Verbose (spam)`}</SelectItem>
                        <SelectItem value="1">Info</SelectItem>
                        <SelectItem value="2">Warnings</SelectItem>
                        <SelectItem value="3">Errors</SelectItem>
                        <SelectItem value="4">Only fatal errors</SelectItem>
                      </SelectContent>
                    </Select>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <div className="grid grid-cols-2 gap-x-8 gap-y-1 p-1 border border-border rounded-md">
              <h3 className="text-xl font-medium col-span-2">Master Server</h3>
              <FormField
                control={form.control}
                name="master_server.enabled"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Enabled
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to show your server on the master server list.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value}
                        onCheckedChange={(checked) => {
                          if (checked === true) {
                            field.onChange(true);
                          } else {
                            field.onChange(false);
                          }
                        }}
                      />
                    </FormControl>

                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="master_server.address"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Address
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The address of the master server to connect to.
                    </FormDescription>
                    <FormControl>
                      <Input type="text" {...field} value={field.value || ""} />
                    </FormControl>
                    <span className="text-xs text-muted-foreground italic max-w-sm text-wrap">
                      {`You shouldn't really need to change this`}
                    </span>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="master_server.port"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">Port</FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The port of the master server to connect to.
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>

                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="master_server.rate"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">Rate</FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The rate to report your server to the master server.
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>

                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
          </div>
        </div>
        <div className="flex flex-col gap-2 w-full">
          <Button type="submit" className="w-full ">
            <SaveIcon className="w-4 h-4" />
            Save
          </Button>
          <Button
            type="button"
            className="w-full "
            variant="destructive"
            onClick={resetToDefaults}
          >
            <TrashIcon className="w-4 h-4" />
            Reset to Tes3MP defaults
          </Button>
        </div>
      </form>
    </Form>
  );
}
