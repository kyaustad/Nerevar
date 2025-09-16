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
import { transformServerSettingsConfig } from "@/lib/utils";
import { ServerSettings } from "@/types/server-settings";
import { Separator } from "@/components/ui/separator";

const formSchema = z.object({
  gameMode: z.string().min(1),
  loginTime: z.number().min(1),
  maxClientsPerIP: z.number().min(1),
  difficulty: z.number().min(-100).max(100),
  gameSettings: z.array(
    z.object({
      name: z.string().min(1),
      value: z.boolean(),
    })
  ),

  defaultTimeTable: z.object({
    year: z.number().min(0),
    month: z.number().min(1),
    day: z.number().min(1),
    hour: z.number().min(1),
    daysPassed: z.number().min(1),
  }),

  passTimeWhenEmpty: z.boolean(),
  nightStartHour: z.number().min(0),
  nightEndHour: z.number().min(0),
  allowConsole: z.boolean(),
  allowBedRest: z.boolean(),
  allowWildernessRest: z.boolean(),
  allowWait: z.boolean(),
  shareJournal: z.boolean(),
  shareFactionRanks: z.boolean(),
  shareFactionExpulsion: z.boolean(),
  shareFactionReputation: z.boolean(),
  shareTopics: z.boolean(),
  shareBounty: z.boolean(),
  shareReputation: z.boolean(),
  shareMapExploration: z.boolean(),
  shareVideos: z.boolean(),
  useInstancedSpawn: z.boolean(),

  respawnAtImperialShrine: z.boolean(),
  respawnAtTribunalTemple: z.boolean(),

  maxAttributeValue: z.number().min(1),
  maxSpeedValue: z.number().min(1),
  maxSkillValue: z.number().min(1),
  maxAcrobaticsValue: z.number().min(1),
  ignoreModifierWithMaxSkill: z.boolean(),
  playersRespawn: z.boolean(),
  deathTime: z.number().min(1),
  deathPenaltyJailDays: z.number().min(1),
  bountyResetOnDeath: z.boolean(),
  bountyDeathPenalty: z.boolean(),
  allowSuicideCommand: z.boolean(),
  allowFixmeCommand: z.boolean(),
  fixmeInterval: z.number().min(1),

  pingDifferenceRequiredForAuthority: z.number().min(1),
  enforcedLogLevel: z.number().min(-1).max(4),
  physicsFramerate: z.number().min(1),
  allowOnContainerForUnloadedCells: z.boolean(),
  enablePlayerCollision: z.boolean(),
  enableActorCollision: z.boolean(),
  enablePlacedObjectCollision: z.boolean(),

  useActorCollisionForPlacedObjects: z.boolean(),
  maximumObjectScale: z.number().min(1),
  enforceDataFiles: z.boolean(),
});

export function ServerSettingsForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      gameMode: "",
      loginTime: 1,
      maxClientsPerIP: 1,
      difficulty: 0,
      gameSettings: [],

      defaultTimeTable: {
        year: 0,
        month: 1,
        day: 1,
        hour: 1,
        daysPassed: 1,
      },

      passTimeWhenEmpty: false,
      nightStartHour: 0,
      nightEndHour: 0,
      allowConsole: false,
      allowBedRest: false,
      allowWildernessRest: false,
      allowWait: false,
      shareJournal: false,
      shareFactionRanks: false,
      shareFactionExpulsion: false,
      shareFactionReputation: false,
      shareTopics: false,
      shareBounty: false,
      shareReputation: false,
      shareMapExploration: false,
      shareVideos: false,
      useInstancedSpawn: false,

      respawnAtImperialShrine: false,
      respawnAtTribunalTemple: false,

      maxAttributeValue: 1,
      maxSpeedValue: 1,
      maxSkillValue: 1,
      maxAcrobaticsValue: 1,
      ignoreModifierWithMaxSkill: false,

      playersRespawn: false,
      deathTime: 1,
      deathPenaltyJailDays: 1,
      bountyResetOnDeath: false,
      bountyDeathPenalty: false,
      allowSuicideCommand: false,
      allowFixmeCommand: false,
      fixmeInterval: 1,

      pingDifferenceRequiredForAuthority: 1,
      enforcedLogLevel: 0,
      physicsFramerate: 1,
      allowOnContainerForUnloadedCells: false,
      enablePlayerCollision: false,
      enableActorCollision: false,
      enablePlacedObjectCollision: false,
      useActorCollisionForPlacedObjects: false,
      maximumObjectScale: 1,
      enforceDataFiles: false,
    },
  });

  const getServerSettings = async () => {
    const configResponse = await invoke("get_tes3mp_server_settings");
    console.log("Config Response:", configResponse);
    const config = configResponse as ServerSettings;
    const formattedConfig = transformServerSettingsConfig(config.config);
    console.log("Read Server Settings:", config);
    console.log("Formatted Config:", { config: formattedConfig });
    //write back to file to test formatting
    // await invoke("set_tes3mp_server_settings", {
    //   config: {
    //     ...formattedConfig,
    //     gameMode: "Testis",
    //   },
    // });

    form.reset({
      ...formattedConfig,
    });
  };
  useEffect(() => {
    getServerSettings();
  }, []);

  // Debug form validation
  //   useEffect(() => {
  //     console.log("Form errors:", form.formState.errors);
  //     console.log("Form is valid:", form.formState.isValid);
  //     console.log("Form is submitting:", form.formState.isSubmitting);
  //   }, [
  //     form.formState.errors,
  //     form.formState.isValid,
  //     form.formState.isSubmitting,
  //   ]);

  async function onSubmit(values: z.infer<typeof formSchema>) {
    console.log("SUBMIT PRESSED");
    console.log(values);
    try {
      toast.promise(
        invoke("set_tes3mp_server_settings", {
          config: {
            ...values,
          },
        }),
        {
          loading: "Saving config...",
          success: async (data) => {
            await getServerSettings();
            return "Server config saved successfully";
          },
          error: "Failed to save server config",
        }
      );
      //   window.location.reload();
    } catch (error) {
      console.error("Failed to save server config:", error);
      toast.error("Failed to save server config");
    }
  }

  function resetToDefaults() {
    try {
      //   toast.promise(
      //     invoke("set_tes3mp_server_config", {
      //       config: {
      //         general: {
      //           localAddress: "0.0.0.0",
      //           port: 25565,
      //           maximumPlayers: 64,
      //           logLevel: 1,
      //           password: "",
      //           hostname: "TES3MP Server",
      //         },
      //         masterServer: {
      //           enabled: true,
      //           address: "master.tes3mp.com",
      //           port: 25561,
      //           rate: 10000,
      //         },
      //       },
      //     }),
      //     {
      //       loading: "Resetting to defaults...",
      //       success: async (data) => {
      //         await getServerSettings();
      //         return "Server config reset to defaults";
      //       },
      //       error: "Failed to reset to defaults",
      //     }
      //   );
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
        <div className="flex flex-col gap-2 w-full">
          <Button
            type="submit"
            className="w-full "
            disabled={form.formState.isSubmitting}
          >
            <SaveIcon className="w-4 h-4" />
            Save
          </Button>
        </div>
        <div className="flex flex-row gap-4 w-full h-full place-items-start">
          <div className="grid grid-cols-3 gap-4 w-full">
            {/* COLUMN 1 */}
            <div className="flex flex-col gap-2 w-full h-fit p-1 border border-border rounded-md">
              <FormField
                control={form.control}
                name="gameMode"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Game Mode
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The game mode displayed for your server in the server
                      browser
                    </FormDescription>
                    <FormControl>
                      <Input {...field} value={field.value || ""} />
                    </FormControl>
                    {/* <span className="text-xs text-muted-foreground italic max-w-sm text-wrap">
                    {`The default localAddress of "0.0.0.0" makes the server reachable at all of its local addresses. You almost never have to change this`}
                  </span> */}
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="loginTime"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Login Time
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The time in seconds that players will have to login to the
                      server after connecting.
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="maxClientsPerIP"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Maximum Clients Per IP
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The maximum number of players that can connect to your
                      server from a single IP address.
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        max={64}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="difficulty"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Difficulty
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`In OpenMW, the difficulty slider goes between -100 and 100, with 0 as the default, though you can use any integer value here`}
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={-100}
                        max={100}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="passTimeWhenEmpty"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Pass Time When Empty
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to pass time when the server is empty.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>

                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="nightStartHour"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Night Start Hour
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The hour at which the night starts.
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={0}
                        max={23}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="nightEndHour"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Night End Hour
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The hour at which the night ends.
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={0}
                        max={23}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="allowConsole"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Allow Console
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to allow the command console to be used.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="allowBedRest"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Allow Bed Rest
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to allow players to rest in beds.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="allowWildernessRest"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Allow Wilderness Rest
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to allow players to rest in the wilderness.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="allowWait"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Allow Wait
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to allow players to wait.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="pingDifferenceRequiredForAuthority"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Ping Difference Required For Authority
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`The difference in ping needs to be in favor of a new arrival to a cell or region
                    compared to that cell or region's current player authority for the new arrival to become
                    the authority there --Note: Setting this too low will lead to constant authority changes which cause more lag`}
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="enforcedLogLevel"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Enforced Log Level
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`The log level enforced on clients by default, determining how much debug information is displayed in their debug window and logs --Note 1: Set this to -1 to allow clients to use whatever log level they have set in their client settings --Note 2: If you set this to 0 or 1, clients will be able to read about the movements and actions of other players that they would otherwise not know about, while also incurring a framerate loss on highly populated servers`}
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={-1}
                        max={4}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="allowSuicideCommand"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Allow Suicide Command
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`Whether players should be allowed to use the /suicide command`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="allowFixmeCommand"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Allow Fixme Command
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`Whether players should be allowed to use the /fixme command`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
            {/* COLUMN 2 */}
            <div className="flex flex-col gap-2 w-full h-fit p-1 border border-border rounded-md">
              <FormField
                control={form.control}
                name="enforceDataFiles"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Enforce Data Files
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to enforce that all clients connect with a
                      specific list of data files defined in
                      data/requiredDataFiles.json
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="shareJournal"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Journal
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to share journal entries across the players on the
                      server.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="shareFactionRanks"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Faction Ranks
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to share faction ranks across the players on the
                      server.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="shareFactionExpulsion"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Faction Expulsion
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to share faction expulsion across the players on
                      the server.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="shareFactionReputation"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Faction Reputation
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to share faction reputation across the players on
                      the server.
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="shareTopics"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Topics
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`Whether to share dialogue topics (and statuses) across the players on the server.`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>

                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="shareBounty"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Bounty
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether crime bounties should be shared across players on
                      the server or not
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="shareReputation"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Reputation
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether reputation should be shared across players on the
                      server or not
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="shareMapExploration"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Map Exploration
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether map exploration should be shared across players on
                      the server or not
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="shareVideos"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Share Videos
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether ingame videos should be played for other players
                      when triggered by one player
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="useInstancedSpawn"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Use Instanced Spawn
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether the instanced spawn should be used instead of the
                      noninstanced one
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="respawnAtImperialShrine"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Respawn At Imperial Shrine
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to respawn players at the nearest Imperial shrine
                      instead of the default respawn location
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="respawnAtTribunalTemple"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Respawn At Tribunal Temple
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether to respawn players at the nearest Tribunal temple
                      instead of the default respawn location
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="playersRespawn"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Players Respawn
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether players should respawn when dying
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="bountyResetOnDeath"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Bounty Reset On Death
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {` Whether players' bounties are reset to 0 after dying`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="bountyDeathPenalty"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Bounty Death Penalty
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Whether players spend time in jail proportional to their
                      bounty after dying
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
            {/* COLUMN 3 */}
            <div className="flex flex-col gap-2 w-full h-fit p-1 border border-border rounded-md">
              <FormField
                control={form.control}
                name="maxAttributeValue"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Max Attribute Value
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The maximum value that any attribute except Speed is
                      allowed to have
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="maxSpeedValue"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Max Speed Value
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The maximum value that Speed is allowed to have
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="maxSkillValue"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Max Skill Value
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The maximum value that any skill except Acrobatics is
                      allowed to have
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />

              <FormField
                control={form.control}
                name="maxAcrobaticsValue"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Max Acrobatics Value
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The maximum value that Acrobatics is allowed to have
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>

                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="ignoreModifierWithMaxSkill"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Ignore Modifier With Max Skill
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      Allow modifier values to bypass allowed skill values
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <Separator />

              <FormField
                control={form.control}
                name="deathTime"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Death Time
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The time to stay dead before being respawned, in seconds
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="deathPenaltyJailDays"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Death Penalty Jail Days
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      The number of days spent in jail as a penalty for dying,
                      when respawning
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <Separator />
              <FormField
                control={form.control}
                name="fixmeInterval"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Fixme Interval
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`The number of seconds need to pass between uses of the
                    /fixme command by a player`}
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <Separator />
              <FormField
                control={form.control}
                name="physicsFramerate"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Physics Framerate
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`The physics framerate used by default --Note: In OpenMW, the physics framerate is 60 by default`}
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
                        onChange={(e) => field.onChange(Number(e.target.value))}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="allowOnContainerForUnloadedCells"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Allow On Container For Unloaded Cells
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`Whether players are allowed to interact with containers located in unloaded cells`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="enablePlayerCollision"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Enable Player Collision
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`Whether players should collide with other actors`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="enableActorCollision"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Enable Actor Collision
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`Whether actors should collide with other actors`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="enablePlacedObjectCollision"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Enable Placed Object Collision
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`Whether placed objects should collide with actors`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="useActorCollisionForPlacedObjects"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Use Actor Collision For Placed Objects
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`Whether placed object collision (when turned on) resembles actor collision, in that it prevents players from standing on top of the placed objects without slipping`}
                    </FormDescription>
                    <FormControl>
                      <Switch
                        checked={field.value || false}
                        onCheckedChange={(checked) => field.onChange(checked)}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Separator />
              <FormField
                control={form.control}
                name="maximumObjectScale"
                render={({ field }) => (
                  <FormItem className="h-full gap-y-1">
                    <FormLabel className="text-lg font-medium">
                      Maximum Object Scale
                    </FormLabel>
                    <FormDescription className="max-w-sm text-wrap">
                      {`The maximum scale that an object is allowed to have`}
                    </FormDescription>
                    <FormControl>
                      <Input
                        type="number"
                        min={1}
                        step={1}
                        {...field}
                        value={field.value.toString() || ""}
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
          <Button
            type="submit"
            className="w-full "
            disabled={form.formState.isSubmitting}
          >
            <SaveIcon className="w-4 h-4" />
            Save
          </Button>
        </div>
      </form>
    </Form>
  );
}
