use serde::{Deserialize, Serialize};

// Server Settings structures for Lua config parsing
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerSettings {
    pub config: ConfigSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSettings {
    pub game_mode: String,
    pub login_time: i32,
    pub max_clients_per_ip: i32,
    pub difficulty: i32,
    pub game_settings: Vec<GameSetting>,
    pub vr_settings: Vec<VrSetting>,
    pub default_time_table: DefaultTimeTable,
    pub world_startup_scripts: Vec<String>,
    pub player_startup_scripts: Vec<String>,
    pub pass_time_when_empty: bool,
    pub night_start_hour: i32,
    pub night_end_hour: i32,
    pub allow_console: bool,
    pub allow_bed_rest: bool,
    pub allow_wilderness_rest: bool,
    pub allow_wait: bool,
    pub share_journal: bool,
    pub share_faction_ranks: bool,
    pub share_faction_expulsion: bool,
    pub share_faction_reputation: bool,
    pub share_topics: bool,
    pub share_bounty: bool,
    pub share_reputation: bool,
    pub share_map_exploration: bool,
    pub share_videos: bool,
    pub use_instanced_spawn: bool,
    pub instanced_spawn: SpawnLocation,
    pub noninstanced_spawn: SpawnLocation,
    pub default_respawn: RespawnLocation,
    pub respawn_at_imperial_shrine: bool,
    pub respawn_at_tribunal_temple: bool,
    pub forbidden_cells: Vec<String>,
    pub max_attribute_value: i32,
    pub max_speed_value: i32,
    pub max_skill_value: i32,
    pub max_acrobatics_value: i32,
    pub ignore_modifier_with_max_skill: bool,
    pub banned_equipment_items: Vec<String>,
    pub players_respawn: bool,
    pub death_time: i32,
    pub death_penalty_jail_days: i32,
    pub bounty_reset_on_death: bool,
    pub bounty_death_penalty: bool,
    pub allow_suicide_command: bool,
    pub allow_fixme_command: bool,
    pub fixme_interval: i32,
    pub rank_colors: RankColors,
    pub ping_difference_required_for_authority: i32,
    pub enforced_log_level: i32,
    pub physics_framerate: i32,
    pub allow_on_container_for_unloaded_cells: bool,
    pub enable_player_collision: bool,
    pub enable_actor_collision: bool,
    pub enable_placed_object_collision: bool,
    pub enforced_collision_ref_ids: Vec<String>,
    pub use_actor_collision_for_placed_objects: bool,
    pub maximum_object_scale: f64,
    pub enforce_data_files: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameSetting {
    pub name: String,
    pub value: serde_json::Value, // Can be bool or number
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VrSetting {
    pub name: String,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultTimeTable {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub days_passed: i32,
    pub day_time_scale: i32,
    pub night_time_scale: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnLocation {
    pub cell_description: String,
    pub position: Vec<f64>,
    pub rotation: Vec<f64>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RespawnLocation {
    pub cell_description: String,
    pub position: Vec<f64>,
    pub rotation: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RankColors {
    pub server_owner: String,
    pub admin: String,
    pub moderator: String,
}

impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            game_mode: "Default".to_string(),
            login_time: 60,
            max_clients_per_ip: 3,
            difficulty: 0,
            game_settings: Vec::new(),
            vr_settings: Vec::new(),
            default_time_table: DefaultTimeTable {
                year: 427,
                month: 7,
                day: 16,
                hour: 9,
                days_passed: 1,
                day_time_scale: 30,
                night_time_scale: 40,
            },
            world_startup_scripts: Vec::new(),
            player_startup_scripts: Vec::new(),
            pass_time_when_empty: false,
            night_start_hour: 20,
            night_end_hour: 6,
            allow_console: false,
            allow_bed_rest: true,
            allow_wilderness_rest: true,
            allow_wait: true,
            share_journal: true,
            share_faction_ranks: true,
            share_faction_expulsion: false,
            share_faction_reputation: true,
            share_topics: true,
            share_bounty: false,
            share_reputation: true,
            share_map_exploration: false,
            share_videos: true,
            use_instanced_spawn: true,
            instanced_spawn: SpawnLocation {
                cell_description: "Seyda Neen, Census and Excise Office".to_string(),
                position: vec![1130.3388671875, -387.14947509766, 193.0],
                rotation: vec![0.09375, 1.5078122615814],
                text: "Multiplayer skips several minutes of the game's introduction and places you at the first quest giver.".to_string(),
            },
            noninstanced_spawn: SpawnLocation {
                cell_description: "-3, -2".to_string(),
                position: vec![-23894.0, -15079.0, 505.0],
                rotation: vec![0.0, 1.2],
                text: "Multiplayer skips over the original character generation.".to_string(),
            },
            default_respawn: RespawnLocation {
                cell_description: "Balmora, Temple".to_string(),
                position: vec![4700.5673828125, 3874.7416992188, 14758.990234375],
                rotation: vec![0.25314688682556, 1.570611000061],
            },
            respawn_at_imperial_shrine: true,
            respawn_at_tribunal_temple: true,
            forbidden_cells: Vec::new(),
            max_attribute_value: 200,
            max_speed_value: 365,
            max_skill_value: 200,
            max_acrobatics_value: 1200,
            ignore_modifier_with_max_skill: false,
            banned_equipment_items: Vec::new(),
            players_respawn: true,
            death_time: 5,
            death_penalty_jail_days: 5,
            bounty_reset_on_death: false,
            bounty_death_penalty: false,
            allow_suicide_command: true,
            allow_fixme_command: true,
            fixme_interval: 30,
            rank_colors: RankColors {
                server_owner: "Orange".to_string(),
                admin: "Red".to_string(),
                moderator: "Green".to_string(),
            },
            ping_difference_required_for_authority: 40,
            enforced_log_level: -1,
            physics_framerate: 60,
            allow_on_container_for_unloaded_cells: false,
            enable_player_collision: true,
            enable_actor_collision: true,
            enable_placed_object_collision: false,
            enforced_collision_ref_ids: Vec::new(),
            use_actor_collision_for_placed_objects: false,
            maximum_object_scale: 2.0,
            enforce_data_files: false,
        }
    }
}
