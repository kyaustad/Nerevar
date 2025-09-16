export type ServerSettings = {
  config: {
    gameMode: string;
    loginTime: number;
    maxClientsPerIP: number;
    difficulty: number;
    gameSettings: {
      name: string;
      value: boolean;
    }[];
    vrSettings: {
      name: string;
      value: number;
    }[];
    defaultTimeTable: {
      year: number;
      month: number;
      day: number;
      hour: number;
      daysPassed: number;
      dayTimeScale: number;
      nightTimeScale: number;
    };
    worldStartupScripts: string[];
    playerStartupScripts: string[];
    passTimeWhenEmpty: boolean;
    nightStartHour: number;
    nightEndHour: number;
    allowConsole: boolean;
    allowBedRest: boolean;
    allowWildernessRest: boolean;
    allowWait: boolean;
    shareJournal: boolean;
    shareFactionRanks: boolean;
    shareFactionExpulsion: boolean;
    shareFactionReputation: boolean;
    shareTopics: boolean;
    shareBounty: boolean;
    shareReputation: boolean;
    shareMapExploration: boolean;
    shareVideos: boolean;
    useInstancedSpawn: boolean;
    instancedSpawn: {
      cellDescription: string;
      position: number[];
      rotation: number[];
      text: string;
    };
    noninstancedSpawn: {
      cellDescription: string;
      position: number[];
      rotation: number[];
      text: string;
    };
    defaultRespawn: {
      cellDescription: string;
      position: number[];
      rotation: number[];
    };
    respawnAtImperialShrine: boolean;
    respawnAtTribunalTemple: boolean;
    forbiddenCells: string[];
    maxAttributeValue: number;
    maxSpeedValue: number;
    maxSkillValue: number;
    maxAcrobaticsValue: number;
    ignoreModifierWithMaxSkill: boolean;
    bannedEquipmentItems: string[];
    playersRespawn: boolean;
    deathTime: number;
    deathPenaltyJailDays: number;
    bountyResetOnDeath: boolean;
    bountyDeathPenalty: boolean;
    allowSuicideCommand: boolean;
    allowFixmeCommand: boolean;
    fixmeInterval: number;
    rankColors: {
      serverOwner: string;
      admin: string;
      moderator: string;
    };
    pingDifferenceRequiredForAuthority: number;
    enforcedLogLevel: number;
    physicsFramerate: number;
    allowOnContainerForUnloadedCells: boolean;
    enablePlayerCollision: boolean;
    enableActorCollision: boolean;
    enablePlacedObjectCollision: boolean;
    enforcedCollisionRefIds: string[];
    useActorCollisionForPlacedObjects: boolean;
    maximumObjectScale: number;
    enforceDataFiles: boolean;
  };
};
