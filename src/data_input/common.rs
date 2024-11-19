use std::collections::HashMap;

use super::{
    abilities::SpellData,
    champions::{stat_increase, ChampionStats},
    items::Item,
};

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Offensive
#[derive(Debug)]
pub struct OffensiveStats {
    pub ability_haste: f64,
    pub ad_base: f64,
    pub ad_bonus: f64,
    pub lethality: f64,
    pub armor_penetration_perc: f64,
    pub crit_chance: f64,
    pub attack_speed_base: f64,
    pub attack_speed_bonus: f64,
    // pub attack_speed_ratio: f64,
    // pub attack_windup: f64,
}

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Defensive
#[derive(Debug)]
pub struct DefensiveStats {
    pub armor: f64,
    pub hp: f64,
}

pub struct GameParams<'a> {
    pub champion_stats: &'a ChampionStats,
    pub level: u64,
    pub items: &'a Vec<&'a Item>,
    pub config: &'a HashMap<String, String>,
    pub abilities: &'a Vec<SpellData>,
    pub def_stats: &'a DefensiveStats,
}

pub fn compute_source_champion_stats(
    champ_stats: &ChampionStats,
    level: u64,
    // runes: HashMap<String, String>,
    items: &Vec<&Item>,
    // auras: &Vec<&Aura>,
) -> OffensiveStats {
    // see https://leagueoflegends.fandom.com/wiki/Champion_statistic
    let mut offensive_stats: OffensiveStats = OffensiveStats {
        ability_haste: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.ability_haste),
        ad_base: champ_stats.attack_damage_flat
            + stat_increase(champ_stats.attack_damage_per_level, level as f64),
        ad_bonus: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.ad_bonus),
        armor_penetration_perc: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.armor_penetration_perc),
        crit_chance: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.crit_chance),
        lethality: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.lethality),
        // see https://leagueoflegends.fandom.com/wiki/Attack_speed#Calculations
        attack_speed_base: champ_stats.attack_speed_flat,
        attack_speed_bonus: stat_increase(champ_stats.attack_speed_per_level, level as f64)
            + items
                .iter()
                .fold(0.0, |acc, x| acc + x.offensive_stats.attack_speed_bonus),
        // attack_speed_ratio: champ_stats.attack_speed_ratio,
        // attack_windup: todo!(),
    };

    apply_passives(&mut offensive_stats, items);

    offensive_stats
}

fn apply_passives(offensive_stats: &mut OffensiveStats, items: &Vec<&Item>) {
    // todo: change this in a callback fashion
}
