use crate::types::{
    ConfigSettings, GeneralConfig, MasterServerConfig, PluginsConfig, ServerSettings,
    Tes3MPServerConfig,
};
use regex::Regex;

pub fn parse_server_config(content: &str) -> Result<Tes3MPServerConfig, String> {
    let mut general = GeneralConfig {
        local_address: "0.0.0.0".to_string(),
        port: 25565,
        maximum_players: 64,
        hostname: "TES3MP server".to_string(),
        log_level: 1,
        password: String::new(),
    };

    let mut plugins = PluginsConfig {
        home: "./server".to_string(),
        plugins: "serverCore.lua".to_string(),
    };

    let mut master_server = MasterServerConfig {
        enabled: true,
        address: "master.tes3mp.com".to_string(),
        port: 25561,
        rate: 10000,
    };

    let mut current_section = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check if this is a section header
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = Some(trimmed[1..trimmed.len() - 1].to_string());
            continue;
        }

        // Check if this line contains a key-value pair
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            let value = trimmed[eq_pos + 1..].trim();

            match current_section.as_deref() {
                Some("General") => match key {
                    "localAddress" => general.local_address = value.to_string(),
                    "port" => general.port = value.parse().unwrap_or(25565),
                    "maximumPlayers" => general.maximum_players = value.parse().unwrap_or(64),
                    "hostname" => general.hostname = value.to_string(),
                    "logLevel" => general.log_level = value.parse().unwrap_or(1),
                    "password" => general.password = value.to_string(),
                    _ => {}
                },
                Some("Plugins") => match key {
                    "home" => plugins.home = value.to_string(),
                    "plugins" => plugins.plugins = value.to_string(),
                    _ => {}
                },
                Some("MasterServer") => match key {
                    "enabled" => master_server.enabled = value.parse().unwrap_or(true),
                    "address" => master_server.address = value.to_string(),
                    "port" => master_server.port = value.parse().unwrap_or(25561),
                    "rate" => master_server.rate = value.parse().unwrap_or(10000),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok(Tes3MPServerConfig {
        general,
        plugins,
        master_server,
    })
}

pub fn parse_server_settings(content: &str) -> Result<ServerSettings, String> {
    // Helper function to extract string values
    let extract_string = |content: &str, pattern: &str| -> Option<String> {
        let pattern_str = format!("config\\.{}\\s*=\\s*\"([^\"]*)\"", pattern);
        let re = Regex::new(&pattern_str).ok()?;
        re.captures(content)?.get(1)?.as_str().to_string().into()
    };

    // Helper function to extract integer values
    let extract_int = |content: &str, pattern: &str| -> Option<i32> {
        let pattern_str = format!("config\\.{}\\s*=\\s*(-?\\d+)", pattern);
        let re = Regex::new(&pattern_str).ok()?;
        re.captures(content)?.get(1)?.as_str().parse().ok()
    };

    // Helper function to extract boolean values
    let extract_bool = |content: &str, pattern: &str| -> Option<bool> {
        let pattern_str = format!("config\\.{}\\s*=\\s*(true|false)", pattern);
        let re = Regex::new(&pattern_str).ok()?;
        match re.captures(content)?.get(1)?.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    };

    // Helper function to extract float values
    let extract_float = |content: &str, pattern: &str| -> Option<f64> {
        let pattern_str = format!("config\\.{}\\s*=\\s*([\\d.]+)", pattern);
        let re = Regex::new(&pattern_str).ok()?;
        re.captures(content)?.get(1)?.as_str().parse().ok()
    };

    // Extract all the configuration values
    let game_mode = extract_string(content, "gameMode").unwrap_or_else(|| "Default".to_string());
    let login_time = extract_int(content, "loginTime").unwrap_or(60);
    let max_clients_per_ip = extract_int(content, "maxClientsPerIP").unwrap_or(3);
    let difficulty = extract_int(content, "difficulty").unwrap_or(0);
    let pass_time_when_empty = extract_bool(content, "passTimeWhenEmpty").unwrap_or(false);
    let night_start_hour = extract_int(content, "nightStartHour").unwrap_or(20);
    let night_end_hour = extract_int(content, "nightEndHour").unwrap_or(6);
    let allow_console = extract_bool(content, "allowConsole").unwrap_or(false);
    let allow_bed_rest = extract_bool(content, "allowBedRest").unwrap_or(true);
    let allow_wilderness_rest = extract_bool(content, "allowWildernessRest").unwrap_or(true);
    let allow_wait = extract_bool(content, "allowWait").unwrap_or(true);
    let share_journal = extract_bool(content, "shareJournal").unwrap_or(true);
    let share_faction_ranks = extract_bool(content, "shareFactionRanks").unwrap_or(true);
    let share_faction_expulsion = extract_bool(content, "shareFactionExpulsion").unwrap_or(false);
    let share_faction_reputation = extract_bool(content, "shareFactionReputation").unwrap_or(true);
    let share_topics = extract_bool(content, "shareTopics").unwrap_or(true);
    let share_bounty = extract_bool(content, "shareBounty").unwrap_or(false);
    let share_reputation = extract_bool(content, "shareReputation").unwrap_or(true);
    let share_map_exploration = extract_bool(content, "shareMapExploration").unwrap_or(false);
    let share_videos = extract_bool(content, "shareVideos").unwrap_or(true);
    let use_instanced_spawn = extract_bool(content, "useInstancedSpawn").unwrap_or(true);
    let respawn_at_imperial_shrine =
        extract_bool(content, "respawnAtImperialShrine").unwrap_or(true);
    let respawn_at_tribunal_temple =
        extract_bool(content, "respawnAtTribunalTemple").unwrap_or(true);
    let max_attribute_value = extract_int(content, "maxAttributeValue").unwrap_or(200);
    let max_speed_value = extract_int(content, "maxSpeedValue").unwrap_or(365);
    let max_skill_value = extract_int(content, "maxSkillValue").unwrap_or(200);
    let max_acrobatics_value = extract_int(content, "maxAcrobaticsValue").unwrap_or(1200);
    let ignore_modifier_with_max_skill =
        extract_bool(content, "ignoreModifierWithMaxSkill").unwrap_or(false);
    let players_respawn = extract_bool(content, "playersRespawn").unwrap_or(true);
    let death_time = extract_int(content, "deathTime").unwrap_or(5);
    let death_penalty_jail_days = extract_int(content, "deathPenaltyJailDays").unwrap_or(5);
    let bounty_reset_on_death = extract_bool(content, "bountyResetOnDeath").unwrap_or(false);
    let bounty_death_penalty = extract_bool(content, "bountyDeathPenalty").unwrap_or(false);
    let allow_suicide_command = extract_bool(content, "allowSuicideCommand").unwrap_or(true);
    let allow_fixme_command = extract_bool(content, "allowFixmeCommand").unwrap_or(true);
    let fixme_interval = extract_int(content, "fixmeInterval").unwrap_or(30);
    let ping_difference_required_for_authority =
        extract_int(content, "pingDifferenceRequiredForAuthority").unwrap_or(40);
    let enforced_log_level = extract_int(content, "enforcedLogLevel").unwrap_or(-1);
    let physics_framerate = extract_int(content, "physicsFramerate").unwrap_or(60);
    let allow_on_container_for_unloaded_cells =
        extract_bool(content, "allowOnContainerForUnloadedCells").unwrap_or(false);
    let enable_player_collision = extract_bool(content, "enablePlayerCollision").unwrap_or(true);
    let enable_actor_collision = extract_bool(content, "enableActorCollision").unwrap_or(true);
    let enable_placed_object_collision =
        extract_bool(content, "enablePlacedObjectCollision").unwrap_or(false);
    let use_actor_collision_for_placed_objects =
        extract_bool(content, "useActorCollisionForPlacedObjects").unwrap_or(false);
    let maximum_object_scale = extract_float(content, "maximumObjectScale").unwrap_or(20.0);
    let enforce_data_files = extract_bool(content, "enforceDataFiles").unwrap_or(true);

    let config = ConfigSettings {
        game_mode,
        login_time,
        max_clients_per_ip,
        difficulty,
        pass_time_when_empty,
        night_start_hour,
        night_end_hour,
        allow_console,
        allow_bed_rest,
        allow_wilderness_rest,
        allow_wait,
        share_journal,
        share_faction_ranks,
        share_faction_expulsion,
        share_faction_reputation,
        share_topics,
        share_bounty,
        share_reputation,
        share_map_exploration,
        share_videos,
        use_instanced_spawn,
        respawn_at_imperial_shrine,
        respawn_at_tribunal_temple,
        max_attribute_value,
        max_speed_value,
        max_skill_value,
        max_acrobatics_value,
        ignore_modifier_with_max_skill,
        players_respawn,
        death_time,
        death_penalty_jail_days,
        bounty_reset_on_death,
        bounty_death_penalty,
        allow_suicide_command,
        allow_fixme_command,
        fixme_interval,
        ping_difference_required_for_authority,
        enforced_log_level,
        physics_framerate,
        allow_on_container_for_unloaded_cells,
        enable_player_collision,
        enable_actor_collision,
        enable_placed_object_collision,
        use_actor_collision_for_placed_objects,
        maximum_object_scale,
        enforce_data_files,
    };

    Ok(ServerSettings { config })
}

pub fn update_server_settings(content: &str, settings: &ServerSettings) -> Result<String, String> {
    let mut updated_content = content.to_string();

    // Helper function to replace config values
    let replace_config_value = |content: &str, key: &str, value: &str| -> Result<String, String> {
        let pattern = format!(r"config\.{}\s*=\s*[^\n]*", key);
        let replacement = format!("config.{} = {}", key, value);

        let re = Regex::new(&pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
        if re.is_match(content) {
            Ok(re.replace_all(content, &replacement).to_string())
        } else {
            // Field doesn't exist, append it at the end before "return config"
            let insert_point = content.rfind("return config").unwrap_or(content.len());
            let before_return = &content[..insert_point];
            let after_return = &content[insert_point..];
            Ok(format!(
                "{}\nconfig.{} = {}\n{}",
                before_return, key, value, after_return
            ))
        }
    };

    // Update string values
    updated_content = replace_config_value(
        &updated_content,
        "gameMode",
        &format!("\"{}\"", settings.config.game_mode),
    )?;

    // Update integer values
    updated_content = replace_config_value(
        &updated_content,
        "loginTime",
        &settings.config.login_time.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "maxClientsPerIP",
        &settings.config.max_clients_per_ip.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "difficulty",
        &settings.config.difficulty.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "nightStartHour",
        &settings.config.night_start_hour.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "nightEndHour",
        &settings.config.night_end_hour.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "maxAttributeValue",
        &settings.config.max_attribute_value.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "maxSpeedValue",
        &settings.config.max_speed_value.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "maxSkillValue",
        &settings.config.max_skill_value.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "maxAcrobaticsValue",
        &settings.config.max_acrobatics_value.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "deathTime",
        &settings.config.death_time.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "deathPenaltyJailDays",
        &settings.config.death_penalty_jail_days.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "fixmeInterval",
        &settings.config.fixme_interval.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "pingDifferenceRequiredForAuthority",
        &settings
            .config
            .ping_difference_required_for_authority
            .to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "enforcedLogLevel",
        &settings.config.enforced_log_level.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "physicsFramerate",
        &settings.config.physics_framerate.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "maximumObjectScale",
        &settings.config.maximum_object_scale.to_string(),
    )?;

    // Update boolean values
    updated_content = replace_config_value(
        &updated_content,
        "passTimeWhenEmpty",
        &settings.config.pass_time_when_empty.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "allowConsole",
        &settings.config.allow_console.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "allowBedRest",
        &settings.config.allow_bed_rest.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "allowWildernessRest",
        &settings.config.allow_wilderness_rest.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "allowWait",
        &settings.config.allow_wait.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareJournal",
        &settings.config.share_journal.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareFactionRanks",
        &settings.config.share_faction_ranks.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareFactionExpulsion",
        &settings.config.share_faction_expulsion.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareFactionReputation",
        &settings.config.share_faction_reputation.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareTopics",
        &settings.config.share_topics.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareBounty",
        &settings.config.share_bounty.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareReputation",
        &settings.config.share_reputation.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareMapExploration",
        &settings.config.share_map_exploration.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "shareVideos",
        &settings.config.share_videos.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "useInstancedSpawn",
        &settings.config.use_instanced_spawn.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "respawnAtImperialShrine",
        &settings.config.respawn_at_imperial_shrine.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "respawnAtTribunalTemple",
        &settings.config.respawn_at_tribunal_temple.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "ignoreModifierWithMaxSkill",
        &settings.config.ignore_modifier_with_max_skill.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "playersRespawn",
        &settings.config.players_respawn.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "bountyResetOnDeath",
        &settings.config.bounty_reset_on_death.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "bountyDeathPenalty",
        &settings.config.bounty_death_penalty.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "allowSuicideCommand",
        &settings.config.allow_suicide_command.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "allowFixmeCommand",
        &settings.config.allow_fixme_command.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "allowOnContainerForUnloadedCells",
        &settings
            .config
            .allow_on_container_for_unloaded_cells
            .to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "enablePlayerCollision",
        &settings.config.enable_player_collision.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "enableActorCollision",
        &settings.config.enable_actor_collision.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "enablePlacedObjectCollision",
        &settings.config.enable_placed_object_collision.to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "useActorCollisionForPlacedObjects",
        &settings
            .config
            .use_actor_collision_for_placed_objects
            .to_string(),
    )?;
    updated_content = replace_config_value(
        &updated_content,
        "enforceDataFiles",
        &settings.config.enforce_data_files.to_string(),
    )?;

    Ok(updated_content)
}
