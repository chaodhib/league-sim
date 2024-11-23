use std::collections::HashMap;

use crate::{
    data_input::{
        abilities::{find_ability, SpellData},
        champions::ChampionStats,
        common::{DefensiveStats, OffensiveStats},
    },
    simulation::AttackType,
    Damage, SpellResult,
};

pub fn simulate_spell(
    champ_stats: &ChampionStats,
    off_stats: &OffensiveStats,
    level: u64,
    def_stats: &DefensiveStats,
    spell_name: AttackType,
    config: &HashMap<String, String>,
    abilities: &Vec<SpellData>,
) -> SpellResult {
    let mut ability: Option<&SpellData> = None;
    if spell_name != AttackType::AA {
        ability = Some(find_ability(abilities, spell_name, config));
    }

    let spell_result: SpellResult = match spell_name {
        AttackType::AA => simulate_aa(champ_stats, off_stats, def_stats, level),
        AttackType::Q => simulate_q(off_stats, def_stats, level, ability.unwrap()),
        AttackType::W => simulate_w(off_stats, def_stats, level, ability.unwrap()),
        AttackType::E => simulate_e(off_stats, def_stats, level, ability.unwrap()),
        AttackType::R => todo!(),
        // &_ => todo!(),
    };

    // println!("damage: {:#?}", damage);

    return spell_result;
}

pub fn cast_time(
    champ_stats: &ChampionStats,
    off_stats: &OffensiveStats,
    spell_name: AttackType,
    config: &HashMap<String, String>,
    abilities: &Vec<SpellData>,
) -> u64 {
    if spell_name != AttackType::AA {
        let ability: &SpellData = find_ability(abilities, spell_name, config);

        ability.cast_time_ms.unwrap_or_default()
    } else {
        // the cast time of an auto attack is the windup time
        let windup_percent = if champ_stats.attack_delay_offset != 0_f64 {
            0.3_f64 + champ_stats.attack_delay_offset
        } else {
            champ_stats.attack_cast_time / champ_stats.attack_total_time
        };

        (1000.0_f64 * windup_percent / total_attack_speed(off_stats, champ_stats)).round() as u64
    }
}

pub fn total_attack_speed(off_stats: &OffensiveStats, champ_stats: &ChampionStats) -> f64 {
    // total attack speed = base_attack_speed + attack_speed_ratio * attack_speed_bonus
    // see https://wiki.leagueoflegends.com/en-us/Attack_speed#Generalization
    let total_attack_speed: f64 =
        off_stats.attack_speed_base + off_stats.attack_speed_bonus * champ_stats.attack_speed_ratio;

    // println!("total_attack_speed: {:#?}", total_attack_speed);

    total_attack_speed
}

// pub fn cooldown(
//     champ_stats: &ChampionStats,
//     off_stats: &OffensiveStats,
//     spell_name: AttackType,
//     config: &HashMap<String, String>,
//     abilities: &Vec<SpellData>,
// ) -> u64 {
//     if spell_name != AttackType::AA {
//         let ability: &SpellData = find_ability(abilities, spell_name, config);

//         ability.cooldown_ms
//     } else {
//         // the cooldown of an auto attack is the attack timer
//         // attack_timer = 1 / total attack speed
//         // see https://wiki.leagueoflegends.com/en-us/Basic_attack#Attack_speed

//         (1000.0_f64 / total_attack_speed(off_stats, champ_stats)).round() as u64
//     }
// }

fn compute_ability_damage(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    ability: &SpellData,
    // config: &HashMap<String, String>,
    spell_rank: u64,
) -> Damage {
    let base_damage: &f64 = ability.ad_damage.get(&spell_rank).unwrap();
    // println!("1 base_damage: {:#?}", base_damage);

    // include AD ratio
    let bonus_damage: f64 = ability.coefficient_ad * off_stats.ad_bonus;
    // println!("2 bonus_damage: {:#?}", bonus_damage);

    let total_damage: f64 = base_damage + bonus_damage;
    // println!("3 total_damage: {:#?}", total_damage);

    let dmg = compute_mitigated_damage(def_stats, off_stats, total_damage);

    // println!("5 total_damage post mitigation: {:#?}", dmg);

    return Damage {
        min: dmg,
        max: dmg,
        avg: dmg,
    };
}

fn simulate_aa(
    champ_stats: &ChampionStats,
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    _level: u64,
) -> SpellResult {
    let base_damage: f64 = off_stats.ad_base + off_stats.ad_bonus;
    let crit_damage: f64 = if off_stats.crit_chance > 0.0 {
        base_damage * 1.75
    } else {
        base_damage
    };
    let avg_damage: f64 = base_damage * (1.0 + off_stats.crit_chance * 0.75);

    // println!("1 base_damage: {:#?}", base_damage);

    // the cooldown of an auto attack is the attack timer
    // attack_timer = 1 / total attack speed
    // see https://wiki.leagueoflegends.com/en-us/Basic_attack#Attack_speed

    let cooldown = (1000.0_f64 / total_attack_speed(off_stats, champ_stats)).round() as u64;

    SpellResult {
        damage: Damage {
            min: compute_mitigated_damage(def_stats, off_stats, base_damage),
            max: compute_mitigated_damage(def_stats, off_stats, crit_damage),
            avg: compute_mitigated_damage(def_stats, off_stats, avg_damage),
        },
        cooldown: Some(cooldown),
    }
}

fn simulate_q(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    level: u64,
    ability: &SpellData,
) -> SpellResult {
    let spell_rank: u64 = match level {
        1..=3 => 1,
        4 => 2,
        5..=6 => 3,
        7..=8 => 4,
        9..=18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    return SpellResult {
        damage: compute_ability_damage(off_stats, def_stats, ability, spell_rank),
        cooldown: cooldown(ability, spell_rank),
    };
}

fn simulate_w(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    level: u64,
    ability: &SpellData,
) -> SpellResult {
    let spell_rank = match level {
        1 => 0,
        2..=7 => 1,
        8..=9 => 2,
        10..=11 => 3,
        12 => 4,
        13..=18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    SpellResult {
        damage: compute_ability_damage(off_stats, def_stats, ability, spell_rank),
        cooldown: cooldown(ability, spell_rank),
    }
}

fn simulate_e(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    level: u64,
    ability: &SpellData,
) -> SpellResult {
    let spell_rank = match level {
        1..=2 => 0,
        3..=13 => 1,
        14 => 2,
        15..=16 => 3,
        17 => 4,
        18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    SpellResult {
        damage: compute_ability_damage(off_stats, def_stats, ability, spell_rank),
        cooldown: cooldown(ability, spell_rank),
    }
}

fn compute_mitigated_damage(
    def_stats: &DefensiveStats,
    off_stats: &OffensiveStats,
    base_damage: f64,
) -> f64 {
    let mut armor = def_stats.armor;

    // println!("3 armor: {:#?}", armor);

    // todo: add armor reduction

    // include armor penetration %
    armor *= 1.0 - off_stats.armor_penetration_perc;

    // println!("4 armor: {:#?}", armor);

    // include lethality
    armor = (armor - off_stats.lethality).max(0.0);

    // println!("5 armor: {:#?}", armor);

    base_damage * 100.0 / (100.0 + armor)
}

fn cooldown(ability: &SpellData, spell_rank: u64) -> Option<u64> {
    if ability.cooldown_ms.is_some() {
        Some(
            *ability
                .cooldown_ms
                .as_ref()
                .unwrap()
                .get(&spell_rank)
                .unwrap(),
        )
    } else {
        None
    }
}
