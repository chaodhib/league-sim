use std::{
    collections::{BinaryHeap, HashMap},
    fmt,
    ops::{Add, Mul},
};

use crate::{
    data_input::{
        abilities::{find_ability, SpellData},
        common::{AttackerStats, CritHandlingChoice, DamageType, GameParams, TargetStats},
    },
    simulation::{Event, State},
};

#[derive(Debug)]
pub struct SpellResult {
    pub damage: Option<f64>,
    pub damage_type: Option<DamageType>,
    pub cooldown: Option<u64>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AttackType {
    AA,
    Q,
    W,
    E,
    R,
    P,
    // add item active?
}

#[derive(Debug, PartialEq, Clone)]
pub enum SpellCategory {
    Dash,
    Stealth,
}

impl fmt::Display for AttackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub fn simulate_spell(
    attacker_stats: &AttackerStats,
    target_stats: &TargetStats,
    game_params: &GameParams,
    state: &mut State,
    spell_name: AttackType,
    event: &Event,
    events: &mut BinaryHeap<Event>,
) -> SpellResult {
    let mut ability: Option<&SpellData> = None;
    if spell_name != AttackType::AA {
        ability = Some(find_ability(
            game_params.abilities,
            spell_name,
            game_params.initial_config,
        ));
    }

    let mut spell_result: SpellResult = match spell_name {
        AttackType::AA => simulate_aa(attacker_stats, target_stats, game_params.crit_handling),
        AttackType::Q => simulate_q(attacker_stats, target_stats, ability.unwrap()),
        AttackType::W => simulate_w(attacker_stats, target_stats, ability.unwrap()),
        AttackType::E => simulate_e(attacker_stats, target_stats, ability.unwrap()),
        AttackType::R => simulate_r(attacker_stats, ability.unwrap()),
        AttackType::P => panic!(),
        // &_ => todo!(),
    };

    if state.recast_ready.contains(&AttackType::R) {
        spell_result.cooldown = None;
    }

    if ability.is_some() && ability.unwrap().active_effect.is_some() {
        ability.unwrap().active_effect.unwrap().on_effect(
            attacker_stats,
            state,
            game_params,
            event,
            events,
        );
    }

    // println!("damage: {:#?}", damage);

    return spell_result;
}

pub fn cast_time(
    attacker_stats: &AttackerStats,
    spell_name: AttackType,
    config: &HashMap<String, String>,
    abilities: &Vec<SpellData>,
) -> u64 {
    if spell_name != AttackType::AA {
        let ability: &SpellData = find_ability(abilities, spell_name, config);

        ability.cast_time_ms.unwrap_or_default()
    } else {
        // the cast time of an auto attack is the windup time
        let windup_percent = if attacker_stats.attack_delay_offset != 0_f64 {
            0.3_f64 + attacker_stats.attack_delay_offset
        } else {
            attacker_stats.attack_cast_time / attacker_stats.attack_total_time
        };

        (1000.0_f64 * windup_percent / total_attack_speed(attacker_stats)).round() as u64
    }
}

pub fn total_attack_speed(attacker_stats: &AttackerStats) -> f64 {
    // total attack speed = base_attack_speed + attack_speed_ratio * attack_speed_bonus
    // see https://wiki.leagueoflegends.com/en-us/Attack_speed#Generalization
    let total_attack_speed: f64 = attacker_stats.attack_speed_base
        + attacker_stats.attack_speed_bonus * attacker_stats.attack_speed_ratio;

    // println!("total_attack_speed: {:#?}", total_attack_speed);

    total_attack_speed
}

fn compute_ability_damage(
    attacker_stats: &AttackerStats,
    target_stats: &TargetStats,
    ability: &SpellData,
    // config: &HashMap<String, String>,
    spell_rank: u64,
) -> f64 {
    let base_damage: &f64 = ability.ad_damage.get(&spell_rank).unwrap();
    // println!("1 base_damage: {:#?}", base_damage);

    // include AD ratio
    let bonus_damage: f64 = ability.coefficient_ad * attacker_stats.ad_bonus;
    // println!("2 bonus_damage: {:#?}", bonus_damage);

    let mut total_damage: f64 = base_damage + bonus_damage;
    // println!("3 total_damage: {:#?}", total_damage);

    // include damage modifiers
    total_damage *= attacker_stats.damage_ability_multiplier + 1.0;

    let dmg = compute_mitigated_damage(
        attacker_stats,
        target_stats,
        total_damage,
        ability.damage_type.unwrap(),
    );

    // println!("5 total_damage post mitigation: {:#?}", dmg);

    dmg
}

fn simulate_aa(
    attacker_stats: &AttackerStats,
    target_stats: &TargetStats,
    crit_handling: CritHandlingChoice,
) -> SpellResult {
    let base_damage: f64 = attacker_stats.ad_base + attacker_stats.ad_bonus;
    let crit_damage: f64 = if attacker_stats.crit_chance > 0.0 {
        base_damage * 1.75
    } else {
        base_damage
    };
    let avg_damage: f64 = base_damage * (1.0 + attacker_stats.crit_chance * 0.75);

    // println!("1 base_damage: {:#?}", base_damage);

    // the cooldown of an auto attack is the attack timer
    // attack_timer = 1 / total attack speed
    // see https://wiki.leagueoflegends.com/en-us/Basic_attack#Attack_speed

    let cooldown = (1000.0_f64 / total_attack_speed(attacker_stats)).round() as u64;

    let damage = match crit_handling {
        CritHandlingChoice::Min => compute_mitigated_damage(
            attacker_stats,
            target_stats,
            base_damage,
            DamageType::Physical,
        ),
        CritHandlingChoice::Max => compute_mitigated_damage(
            attacker_stats,
            target_stats,
            crit_damage,
            DamageType::Physical,
        ),
        CritHandlingChoice::Avg => compute_mitigated_damage(
            attacker_stats,
            target_stats,
            avg_damage,
            DamageType::Physical,
        ),
    };

    SpellResult {
        damage: Some(damage),
        cooldown: Some(cooldown),
        damage_type: Some(DamageType::Physical),
    }
}

fn simulate_q(
    attacker_stats: &AttackerStats,
    target_stats: &TargetStats,
    ability: &SpellData,
) -> SpellResult {
    let spell_rank: u64 = match attacker_stats.level {
        1..=3 => 1,
        4 => 2,
        5..=6 => 3,
        7..=8 => 4,
        9..=18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    return SpellResult {
        damage: Some(compute_ability_damage(
            attacker_stats,
            target_stats,
            ability,
            spell_rank,
        )),
        cooldown: cooldown(ability, spell_rank, attacker_stats),
        damage_type: Some(DamageType::Physical),
    };
}

fn simulate_w(
    attacker_stats: &AttackerStats,
    target_stats: &TargetStats,
    ability: &SpellData,
) -> SpellResult {
    let spell_rank = match attacker_stats.level {
        1 => 0,
        2..=7 => 1,
        8..=9 => 2,
        10..=11 => 3,
        12 => 4,
        13..=18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    SpellResult {
        damage: Some(compute_ability_damage(
            attacker_stats,
            target_stats,
            ability,
            spell_rank,
        )),
        cooldown: cooldown(ability, spell_rank, attacker_stats),
        damage_type: Some(DamageType::Physical),
    }
}

fn simulate_e(
    attacker_stats: &AttackerStats,
    target_stats: &TargetStats,
    ability: &SpellData,
) -> SpellResult {
    let spell_rank = match attacker_stats.level {
        1..=2 => 0,
        3..=13 => 1,
        14 => 2,
        15..=16 => 3,
        17 => 4,
        18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    SpellResult {
        damage: Some(compute_ability_damage(
            attacker_stats,
            target_stats,
            ability,
            spell_rank,
        )),
        cooldown: cooldown(ability, spell_rank, attacker_stats),
        damage_type: Some(DamageType::Physical),
    }
}
fn simulate_r(attacker_stats: &AttackerStats, ability: &SpellData) -> SpellResult {
    let spell_rank = match attacker_stats.level {
        1..=5 => 0,
        6..=10 => 1,
        11..=15 => 2,
        16..=18 => 3,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    SpellResult {
        damage: None,
        cooldown: cooldown(ability, spell_rank, attacker_stats),
        damage_type: None,
    }
}

pub fn compute_mitigated_damage(
    attacker_stats: &AttackerStats,
    target_stats: &TargetStats,
    base_damage: f64,
    damage_type: DamageType,
) -> f64 {
    if damage_type == DamageType::True {
        return base_damage;
    }

    let mut damage_resistance = match damage_type {
        DamageType::Physical => target_stats.armor,
        DamageType::Magical => target_stats.magic_resistance,
        DamageType::True => panic!(),
        DamageType::Unknown => panic!(),
    };

    // println!("3 armor: {:#?}", armor);

    // todo: 1) reduction, flat
    // todo: 2) reduction, %

    // 3) penetration, %
    let penetration_perc = match damage_type {
        DamageType::Physical => attacker_stats.armor_penetration_perc,
        DamageType::Magical => 0.0,
        DamageType::True => panic!(),
        DamageType::Unknown => panic!(),
    };

    damage_resistance *= 1.0 - penetration_perc;

    // 4) penetration, flat
    let penetration_flat = match damage_type {
        DamageType::Physical => attacker_stats.lethality,
        DamageType::Magical => 0.0,
        DamageType::True => panic!(),
        DamageType::Unknown => panic!(),
    };
    damage_resistance = (damage_resistance - penetration_flat).max(0.0);

    // println!("5 armor: {:#?}", armor);

    base_damage * 100.0 / (100.0 + damage_resistance)
}

fn cooldown(ability: &SpellData, spell_rank: u64, attacker_stats: &AttackerStats) -> Option<u64> {
    if ability.cooldown_ms.is_some() {
        let base_cd = *ability
            .cooldown_ms
            .as_ref()
            .unwrap()
            .get(&spell_rank)
            .unwrap();

        let mut haste = attacker_stats.ability_haste;
        if ability.key == "R" {
            haste += attacker_stats.ultimate_haste;
        } else if vec!["Q", "W", "E"].contains(&ability.key.as_str()) {
            haste += attacker_stats.basic_ability_haste;
        }

        let reduced_cd: u64 = (base_cd as f64 * 100.0 / (100.0 + haste)) as u64;

        // println!("base_cd: {:#?}", base_cd);
        // println!("reduced_cd: {:#?}", reduced_cd);

        Some(reduced_cd)
    } else {
        None
    }
}
