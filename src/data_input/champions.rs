use std::{collections::HashMap, fs::File, io::BufReader};

use serde_json::Value;

#[derive(Clone, Debug)]
pub struct ChampionStats {
    pub armor_flat: f64,
    pub armor_per_level: f64,
    pub attack_damage_flat: f64,
    pub attack_damage_per_level: f64,

    // attack speed calculation
    pub attack_speed_flat: f64,
    pub attack_speed_per_level: f64,
    pub attack_speed_ratio: f64,

    // windup calculation (older formula)
    pub attack_delay_offset: f64,
    // windup calculation (newer formula)
    pub attack_cast_time: f64,
    pub attack_total_time: f64,
}

pub fn get_base_champion_stats() -> ChampionStats {
    let file = File::open("source_3/champions_formatted.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let characters: HashMap<String, HashMap<String, Value>> =
        serde_json::from_reader(reader).unwrap();
    let character = characters.get("Khazix").unwrap();

    ChampionStats {
        armor_flat: character["stats"]["armor"]["flat"].as_f64().unwrap(),
        armor_per_level: character["stats"]["armor"]["perLevel"].as_f64().unwrap(),
        attack_damage_flat: character["stats"]["attackDamage"]["flat"].as_f64().unwrap(),
        attack_damage_per_level: character["stats"]["attackDamage"]["perLevel"]
            .as_f64()
            .unwrap(),
        attack_speed_flat: character["stats"]["attackSpeed"]["flat"].as_f64().unwrap(),
        attack_speed_per_level: character["stats"]["attackSpeed"]["perLevel"]
            .as_f64()
            .unwrap()
            / 100.0,
        attack_speed_ratio: character["stats"]["attackSpeedRatio"]["flat"]
            .as_f64()
            .unwrap(),
        attack_delay_offset: character["stats"]["attackDelayOffset"]["flat"]
            .as_f64()
            .unwrap(),
        attack_cast_time: character["stats"]["attackCastTime"]["flat"]
            .as_f64()
            .unwrap(),
        attack_total_time: character["stats"]["attackTotalTime"]["flat"]
            .as_f64()
            .unwrap(),
    }
}

// source: https://leagueoflegends.fandom.com/wiki/Champion_statistic#Increasing_Statistics
pub fn stat_increase(per_level: f64, level: f64) -> f64 {
    per_level * (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0))
}
