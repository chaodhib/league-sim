use crate::champions::*;
use crate::items::*;

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Offensive
#[derive(Debug)]
pub struct OffensiveStats {
    pub ability_haste: f64,
    pub ad_base: f64,
    pub ad_bonus: f64,
    pub lethality: f64,
    pub armor_penetration_perc: f64,
    pub crit_chance: f64,
}

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Defensive
#[derive(Debug)]
pub struct DefensiveStats {
    pub armor: f64,
    pub hp: f64,
}

pub fn compute_source_champion_stats(
    champ_stats: &ChampionStats,
    level: f64,
    // runes: HashMap<String, String>,
    items: &Vec<&Item>,
) -> OffensiveStats {
    // see https://leagueoflegends.fandom.com/wiki/Champion_statistic
    let mut offensive_stats: OffensiveStats = OffensiveStats {
        ability_haste: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.ability_haste),
        ad_base: champ_stats.attack_damage_flat
            + stat_increase(champ_stats.attack_damage_per_level, level),
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
    };

    apply_passives(&mut offensive_stats, items);

    return offensive_stats;
}

fn apply_passives(offensive_stats: &mut OffensiveStats, items: &Vec<&Item>) {
    // todo: change this in a callback fashion
}
