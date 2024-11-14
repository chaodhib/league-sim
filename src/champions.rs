use std::{collections::HashMap, fs::File, io::BufReader};

use serde_json::Value;

#[derive(Clone, Debug)]
pub struct ChampionStats {
    pub armor_flat: f64,
    pub armor_per_level: f64,
    pub attack_damage_flat: f64,
    pub attack_damage_per_level: f64,
    pub attack_speed_flat: f64,
    pub attack_speed_per_level: f64,
}

pub fn get_base_champion_stats() -> ChampionStats {
    let file = File::open("/home/chaodhib/git/lolstaticdata/champions/Khazix.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let character: HashMap<String, Value> = serde_json::from_reader(reader).unwrap();

    return ChampionStats {
        armor_flat: character["stats"]["armor"]["flat"].as_f64().unwrap(),
        armor_per_level: character["stats"]["armor"]["perLevel"].as_f64().unwrap(),
        attack_damage_flat: character["stats"]["attackDamage"]["flat"].as_f64().unwrap(),
        attack_damage_per_level: character["stats"]["attackDamage"]["perLevel"]
            .as_f64()
            .unwrap(),
        attack_speed_flat: character["stats"]["attackSpeed"]["flat"].as_f64().unwrap(),
        attack_speed_per_level: character["stats"]["attackSpeed"]["perLevel"]
            .as_f64()
            .unwrap(),
    };
}

// source: https://leagueoflegends.fandom.com/wiki/Champion_statistic#Increasing_Statistics
pub fn stat_increase(per_level: f64, level: f64) -> f64 {
    return per_level * (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0));
}
