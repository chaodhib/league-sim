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

// source: https://leagueoflegends.fandom.com/wiki/Champion_statistic#Increasing_Statistics
pub fn stat_increase(per_level: f64, level: f64) -> f64 {
    per_level * (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0))
}
