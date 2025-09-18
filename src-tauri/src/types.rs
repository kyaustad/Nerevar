use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Player,
    Server,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCheckResponse {
    pub update_available: bool,
    pub version: String,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NerevarConfig {
    pub tes3mp_path: String,
    pub version: String,
    pub last_updated: String,
    pub mode: Option<Mode>,
}

// Use a flexible map for OpenMW config since it can contain any settings
pub type OpenMWConfig = std::collections::HashMap<String, serde_json::Value>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tes3MPServerConfig {
    pub general: GeneralConfig,
    pub plugins: PluginsConfig,
    pub master_server: MasterServerConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    pub local_address: String,
    pub port: u16,
    pub maximum_players: u16,
    pub hostname: String,
    pub log_level: u8,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginsConfig {
    pub home: String,
    pub plugins: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MasterServerConfig {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
    pub rate: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerSettings {
    pub config: ConfigSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSettings {
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    #[serde(rename = "loginTime")]
    pub login_time: i32,
    #[serde(rename = "maxClientsPerIP")]
    pub max_clients_per_ip: i32,
    pub difficulty: i32,
    #[serde(rename = "passTimeWhenEmpty")]
    pub pass_time_when_empty: bool,
    #[serde(rename = "nightStartHour")]
    pub night_start_hour: i32,
    #[serde(rename = "nightEndHour")]
    pub night_end_hour: i32,
    #[serde(rename = "allowConsole")]
    pub allow_console: bool,
    #[serde(rename = "allowBedRest")]
    pub allow_bed_rest: bool,
    #[serde(rename = "allowWildernessRest")]
    pub allow_wilderness_rest: bool,
    #[serde(rename = "allowWait")]
    pub allow_wait: bool,
    #[serde(rename = "shareJournal")]
    pub share_journal: bool,
    #[serde(rename = "shareFactionRanks")]
    pub share_faction_ranks: bool,
    #[serde(rename = "shareFactionExpulsion")]
    pub share_faction_expulsion: bool,
    #[serde(rename = "shareFactionReputation")]
    pub share_faction_reputation: bool,
    #[serde(rename = "shareTopics")]
    pub share_topics: bool,
    #[serde(rename = "shareBounty")]
    pub share_bounty: bool,
    #[serde(rename = "shareReputation")]
    pub share_reputation: bool,
    #[serde(rename = "shareMapExploration")]
    pub share_map_exploration: bool,
    #[serde(rename = "shareVideos")]
    pub share_videos: bool,
    #[serde(rename = "useInstancedSpawn")]
    pub use_instanced_spawn: bool,
    #[serde(rename = "respawnAtImperialShrine")]
    pub respawn_at_imperial_shrine: bool,
    #[serde(rename = "respawnAtTribunalTemple")]
    pub respawn_at_tribunal_temple: bool,
    #[serde(rename = "maxAttributeValue")]
    pub max_attribute_value: i32,
    #[serde(rename = "maxSpeedValue")]
    pub max_speed_value: i32,
    #[serde(rename = "maxSkillValue")]
    pub max_skill_value: i32,
    #[serde(rename = "maxAcrobaticsValue")]
    pub max_acrobatics_value: i32,
    #[serde(rename = "ignoreModifierWithMaxSkill")]
    pub ignore_modifier_with_max_skill: bool,
    #[serde(rename = "playersRespawn")]
    pub players_respawn: bool,
    #[serde(rename = "deathTime")]
    pub death_time: i32,
    #[serde(rename = "deathPenaltyJailDays")]
    pub death_penalty_jail_days: i32,
    #[serde(rename = "bountyResetOnDeath")]
    pub bounty_reset_on_death: bool,
    #[serde(rename = "bountyDeathPenalty")]
    pub bounty_death_penalty: bool,
    #[serde(rename = "allowSuicideCommand")]
    pub allow_suicide_command: bool,
    #[serde(rename = "allowFixmeCommand")]
    pub allow_fixme_command: bool,
    #[serde(rename = "fixmeInterval")]
    pub fixme_interval: i32,
    #[serde(rename = "pingDifferenceRequiredForAuthority")]
    pub ping_difference_required_for_authority: i32,
    #[serde(rename = "enforcedLogLevel")]
    pub enforced_log_level: i32,
    #[serde(rename = "physicsFramerate")]
    pub physics_framerate: i32,
    #[serde(rename = "allowOnContainerForUnloadedCells")]
    pub allow_on_container_for_unloaded_cells: bool,
    #[serde(rename = "enablePlayerCollision")]
    pub enable_player_collision: bool,
    #[serde(rename = "enableActorCollision")]
    pub enable_actor_collision: bool,
    #[serde(rename = "enablePlacedObjectCollision")]
    pub enable_placed_object_collision: bool,
    #[serde(rename = "useActorCollisionForPlacedObjects")]
    pub use_actor_collision_for_placed_objects: bool,
    #[serde(rename = "maximumObjectScale")]
    pub maximum_object_scale: f64,
    #[serde(rename = "enforceDataFiles")]
    pub enforce_data_files: bool,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct GameSetting {
//     pub name: String,
//     pub value: String,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct VrSetting {
//     pub name: String,
//     pub value: String,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct DefaultTimeTable {
//     pub days_in_month: i32,
//     pub months_in_year: i32,
//     pub hours_in_day: i32,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct SpawnLocation {
//     pub pos_x: f64,
//     pub pos_y: f64,
//     pub pos_z: f64,
//     pub rot_x: f64,
//     pub rot_y: f64,
//     pub rot_z: f64,
//     pub cell: String,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct RespawnLocation {
//     pub pos_x: f64,
//     pub pos_y: f64,
//     pub pos_z: f64,
//     pub rot_x: f64,
//     pub rot_y: f64,
//     pub rot_z: f64,
//     pub cell: String,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct RankColors {
//     pub admin: String,
//     pub moderator: String,
//     pub user: String,
// }
