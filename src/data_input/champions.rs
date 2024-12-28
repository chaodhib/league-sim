use std::{collections::HashMap, fs::File, io::BufReader};

use serde_json::Value;

use super::common::Champion;

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

    pub base_movement_speed: f64,
}

pub enum AttackType {
    Melee,
    Ranged,
}

impl AttackType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "MELEE" => AttackType::Melee,
            "RANGED" => AttackType::Ranged,
            _ => panic!("Invalid attack type"),
        }
    }
}

pub enum AdaptiveType {
    Physical,
    Magic,
}

impl AdaptiveType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "PHYSICAL_DAMAGE" => AdaptiveType::Physical,
            "MAGIC_DAMAGE" => AdaptiveType::Magic,
            _ => panic!("Invalid adaptive type"),
        }
    }
}

pub struct ChampionData {
    pub name: Champion,
    pub id: u64,
    pub key: String,
    pub attack_type: AttackType,
    pub adaptive_type: AdaptiveType,
}

pub fn get_base_champion_stats(champion: Champion) -> (ChampionData, ChampionStats) {
    let file = File::open("source_3/champions_formatted.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let characters: HashMap<String, HashMap<String, Value>> =
        serde_json::from_reader(reader).unwrap();
    let character = characters.get("Khazix").unwrap();

    let champion_data = ChampionData {
        name: Champion::Khazix,
        id: character["id"].as_u64().unwrap(),
        key: character["key"].as_str().unwrap().to_string(),
        attack_type: AttackType::from_str(character["attackType"].as_str().unwrap()),
        adaptive_type: AdaptiveType::from_str(character["adaptiveType"].as_str().unwrap()),
    };

    let champion_stats = ChampionStats {
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
        base_movement_speed: character["stats"]["movespeed"]["flat"].as_f64().unwrap(),
    };

    (champion_data, champion_stats)
}

// source: https://leagueoflegends.fandom.com/wiki/Champion_statistic#Increasing_Statistics
pub fn stat_increase(per_level: f64, level: f64) -> f64 {
    per_level * (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0))
}
