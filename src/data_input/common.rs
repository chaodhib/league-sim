use std::{
    collections::{HashMap, HashSet},
    ops::{Add, AddAssign},
};

use crate::simulation::State;

use super::{
    abilities::SpellData,
    champions::{stat_increase, ChampionStats},
    items::Item,
    runes::{collect_runes_stats, Rune, RunesData},
};

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Offensive
#[derive(Debug, Default)]
pub struct OffensiveStats {
    pub ability_haste: f64,
    pub ability_power: f64,
    pub ad_base: f64,
    pub ad_bonus: f64,
    pub lethality: f64,
    pub armor_penetration_perc: f64,
    pub crit_chance: f64,
    pub attack_speed_base: f64,
    pub attack_speed_bonus: f64,
    pub damage_physical_multiplier: f64,
    pub damage_magical_multiplier: f64,
    pub damage_true_multiplier: f64,
    pub adaptive_force: f64,
}

impl AddAssign for OffensiveStats {
    fn add_assign(&mut self, other: OffensiveStats) {
        self.ability_haste += other.ability_haste;
        self.ad_base += other.ad_base;
        self.ad_bonus += other.ad_bonus;
        self.lethality += other.lethality;
        self.armor_penetration_perc += other.armor_penetration_perc;
        self.crit_chance += other.crit_chance;
        self.attack_speed_base += other.attack_speed_base;
        self.attack_speed_bonus += other.attack_speed_bonus;
        // multipliers are combined multiplicatively
        self.damage_physical_multiplier *= other.damage_physical_multiplier;
        self.damage_magical_multiplier *= other.damage_magical_multiplier;
        self.damage_true_multiplier *= other.damage_true_multiplier;
        self.adaptive_force += other.adaptive_force;
        self.ability_power += other.ability_power;
    }
}

impl Add for OffensiveStats {
    type Output = OffensiveStats;

    fn add(mut self, other: OffensiveStats) -> OffensiveStats {
        self += other;
        self
    }
}

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Defensive
#[derive(Debug)]
pub struct DefensiveStats {
    pub armor: f64,
    pub hp: f64,
}

// this is a container for data that is constant throughout the duration of each simulation
pub struct GameParams<'a> {
    pub champion_stats: &'a ChampionStats,
    pub level: u64,
    pub items: &'a Vec<&'a Item>,
    pub configs: &'a HashMap<String, String>,
    pub abilities: &'a Vec<SpellData>,
    pub def_stats: &'a DefensiveStats,
    pub runes: &'a HashSet<Rune>,
    pub hp_perc: f64,
    pub runes_data: &'a RunesData,
}

pub fn compute_source_champion_stats(game_params: &GameParams, state: &State) -> OffensiveStats {
    let champion_stats = game_params.champion_stats;
    let level = game_params.level;
    let items = game_params.items;
    // see https://leagueoflegends.fandom.com/wiki/Champion_statistic
    let mut offensive_stats: OffensiveStats = OffensiveStats {
        ability_haste: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.ability_haste),
        ad_base: champion_stats.attack_damage_flat
            + stat_increase(champion_stats.attack_damage_per_level, level as f64),
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
        attack_speed_base: champion_stats.attack_speed_flat,
        attack_speed_bonus: stat_increase(champion_stats.attack_speed_per_level, level as f64)
            + items
                .iter()
                .fold(0.0, |acc, x| acc + x.offensive_stats.attack_speed_bonus),
        damage_physical_multiplier: 0.0,
        damage_magical_multiplier: 0.0,
        damage_true_multiplier: 0.0,
        adaptive_force: 0.0,
        // todo
        ability_power: 0.0,
    };
    offensive_stats += collect_runes_stats(state, game_params);

    apply_passives(&mut offensive_stats, items);
    apply_adaptive_force(&mut offensive_stats);

    offensive_stats
}

fn apply_passives(offensive_stats: &mut OffensiveStats, items: &Vec<&Item>) {
    // todo: change this in a callback fashion
}

fn apply_adaptive_force(offensive_stats: &mut OffensiveStats) {
    if offensive_stats.ad_bonus > offensive_stats.ability_power {
        offensive_stats.ad_bonus += offensive_stats.adaptive_force * 0.6;
    } else {
        offensive_stats.ability_power += offensive_stats.adaptive_force;
    }
    offensive_stats.adaptive_force = 0.0;
}
