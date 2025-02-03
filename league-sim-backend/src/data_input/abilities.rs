use std::collections::HashMap;

use crate::{
    attack::{compute_mitigated_damage, AttackType, SpellCategory},
    simulation::{self, DamageInfo},
};

use shared_structs::champions::*;

use super::common::{
    compute_attacker_stats, compute_target_stats, DamageType, EffectWithCallbacks,
};

// #[derive(Debug)]
pub struct SpellData {
    pub key: String,
    pub coefficient_ad: f64,
    pub coefficient_ap: f64,
    pub ad_damage: HashMap<u64, f64>,
    pub ap_damage: HashMap<u64, f64>,
    pub variation_name: Option<String>,
    pub cast_time_ms: Option<u64>,
    pub cooldown_ms: Option<HashMap<u64, u64>>,
    // pub passive_effects: Vec<&'static dyn Effect>,
    pub category: Option<SpellCategory>,
    pub damage_type: Option<DamageType>,
    pub active_effect: Option<&'static dyn ScriptedEffect>,
    pub recast_gap_duration: Option<u64>,
    pub recast_charges: Option<u64>,
    pub recast_window: Option<u64>,
}

pub struct UnseenThreat {
    base_damage: f64,
    per_level_bonus: f64,
    bonus_ad_ratio: f64,
}

impl EffectWithCallbacks for UnseenThreat {
    fn on_post_damage(
        &self,
        damage_info: &DamageInfo,
        attacker_stats: &super::common::AttackerStats,
        state: &mut crate::simulation::State<'_>,
        game_params: &super::common::GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        if !state
            .attacker_auras
            .contains_key(&super::common::Aura::UnseenThreat)
        {
            panic!();
        }

        if damage_info
            .source_ability
            .is_none_or(|x| x != AttackType::AA)
        {
            return;
        }

        let attacker_stats = compute_attacker_stats(game_params, state);
        let target_stats = compute_target_stats(game_params, state);

        let mut magic_damage: f64 = self.base_damage
            + self.per_level_bonus * game_params.level as f64
            + self.bonus_ad_ratio * attacker_stats.ad_bonus;

        magic_damage *= attacker_stats.damage_ability_multiplier + 1.0;

        let mitigated_dmg = compute_mitigated_damage(
            &attacker_stats,
            &target_stats,
            magic_damage,
            DamageType::Magical,
        );

        simulation::on_damage_from_ability(
            &mitigated_dmg,
            DamageType::Magical,
            state,
            AttackType::P,
        );

        // remove buff
        state.end_early_attacker_aura(
            &super::common::Aura::UnseenThreat,
            game_params,
            event,
            events,
        );

        let new_damage_info = DamageInfo {
            amount: mitigated_dmg,
            damage_type: DamageType::Magical,
            time_ms: state.time_ms,
            source: simulation::DamageSource::Ability,
            source_ability: Some(AttackType::P),
            source_rune: None,
            source_item: None,
        };

        // simulation::on_post_damage_events(
        //     &new_damage_info,
        //     &attacker_stats,
        //     state,
        //     game_params,
        //     event,
        //     events,
        // );
    }
}

pub struct AbilitiesExtraData {
    pub unseen_threat: UnseenThreat,
}

pub trait ScriptedEffect {
    fn on_effect(
        &self,
        attacker_stats: &super::common::AttackerStats,
        state: &mut crate::simulation::State<'_>,
        game_params: &super::common::GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    );
}

struct KhazixR {
    base_duration: u64,
    evolved_duration: u64,
}

impl ScriptedEffect for KhazixR {
    fn on_effect(
        &self,
        attacker_stats: &super::common::AttackerStats,
        state: &mut crate::simulation::State<'_>,
        game_params: &super::common::GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        let r_evolved: bool = game_params
            .initial_config
            .get("CHAMPION_KHAZIX_R_EVOLVED")
            .unwrap()
            == "TRUE";

        // first cast scenario
        if !state.cooldowns.contains_key(&AttackType::R)
        //&& !state.recast_ready.contains(&AttackType::R)
        {
            state.recast_charges.retain(|&x| x != AttackType::R);

            if r_evolved {
                state.recast_charges.push(AttackType::R);
                state.recast_charges.push(AttackType::R);
            } else {
                state.recast_charges.push(AttackType::R);
            }
        } else if state.recast_ready.contains(&AttackType::R) {
            // recast scenario
            state.end_early_attacker_aura(
                &super::common::Aura::VoidAssaultRecastReady,
                game_params,
                event,
                events,
            );

            // remove one recast charge
            let index = state
                .recast_charges
                .iter()
                .position(|value| *value == crate::attack::AttackType::R)
                .unwrap();
            state.recast_charges.remove(index);
        } else {
            panic!()
        }

        state.add_attacker_aura(super::common::Aura::UnseenThreat, None, None, events);

        let stealth_duration = if r_evolved {
            self.evolved_duration
        } else {
            self.base_duration
        };

        state.add_attacker_aura(
            super::common::Aura::Invisibility,
            Some(stealth_duration),
            None,
            events,
        );
    }
}

// fn has_desireable_stats(item: &Value) -> bool {
//     return (not_nul(&item, "abilityHaste")
//         || not_nul(&item, "armorPenetration")
//         || not_nul(&item, "attackDamage")
//         // || not_nul(&item, "cooldownReduction")
//         || not_nul(&item, "lethality"))
//         && !not_nul(&item, "abilityPower")
//         && !not_nul(&item, "criticalStrikeChance")
//         && !not_nul(&item, "attackSpeed");
// }

// fn not_nul(item: &Value, stat_category: &str) -> bool {
//     let values = item["stats"].clone()[stat_category].clone();
//     match values.as_object() {
//         Some(category) => {
//             for ele in category.iter() {
//                 if ele.1.as_number().unwrap().as_f64() != Some(0.0) {
//                     // println!("item: {:#?}", item);
//                     return true;
//                 }
//             }
//         }
//         None => (),
//     };
//     return false;
// }

pub fn pull_abilities_data(
    config: &HashMap<String, String>,
) -> (Vec<SpellData>, AbilitiesExtraData) {
    let khazix: Champion = include!("champions_gen/khazix.rs");
    let mut abilities_data = Vec::new();

    // Q (variation 1)
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            khazix.abilities.q[0].effects[1].leveling[0].modifiers[0].values[rank - 1],
        );
    }

    let cast_time_s = khazix.abilities.q[0]
        .cast_time
        .as_ref()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let mut cooldown_ms: HashMap<u64, u64> = HashMap::new();
    for rank in 1..=5usize {
        cooldown_ms.insert(
            rank.try_into().unwrap(),
            khazix.abilities.q[0].cooldown.as_ref().unwrap().modifiers[0].values[rank - 1] as u64
                * 1000u64,
        );
    }

    if config.get("CHAMPION_KHAZIX_ISOLATED_TARGET").unwrap() == "TRUE"
        && config
            .get("CHAMPION_KHAZIX_Q_EVOLVED")
            .unwrap_or(&"TRUE".to_string())
            == "TRUE"
    {
        for cooldown in cooldown_ms.values_mut() {
            *cooldown = (*cooldown as f64 * (1.0 - 0.45)) as u64;
        }
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage,
        ap_damage,
        coefficient_ad: khazix.abilities.q[0].effects[1].leveling[0].modifiers[1].values[0] * 0.01,
        coefficient_ap: 0.0f64,
        key: "Q".to_string(),
        variation_name: Some("Physical Damage".to_string()),
        cast_time_ms: Some((cast_time_s * 1000f64) as u64),
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: None,
        damage_type: Some(DamageType::Physical),
        active_effect: None,
        recast_gap_duration: None,
        recast_charges: None,
        recast_window: None,
    });

    // Q (variation 2)
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            khazix.abilities.q[0].effects[1].leveling[1].modifiers[0].values[rank - 1],
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage,
        ap_damage,
        coefficient_ad: khazix.abilities.q[0].effects[1].leveling[1].modifiers[1].values[0] * 0.01,
        coefficient_ap: 0.0f64,
        key: "Q".to_string(),
        variation_name: Some("Increased Damage".to_string()),
        cast_time_ms: Some((cast_time_s * 1000f64) as u64),
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: None,
        damage_type: Some(DamageType::Physical),
        active_effect: None,
        recast_gap_duration: None,
        recast_charges: None,
        recast_window: None,
    });

    // W
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            khazix.abilities.w[0].effects[0].leveling[0].modifiers[0].values[rank - 1],
        );
    }

    let cast_time_s = khazix.abilities.w[0]
        .cast_time
        .as_ref()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let mut cooldown_ms: HashMap<u64, u64> = HashMap::new();
    for rank in 1..=5usize {
        cooldown_ms.insert(
            rank.try_into().unwrap(),
            khazix.abilities.w[0].cooldown.as_ref().unwrap().modifiers[0].values[rank - 1] as u64
                * 1000u64,
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage,
        ap_damage,
        coefficient_ad: khazix.abilities.w[0].effects[0].leveling[0].modifiers[1].values[0] * 0.01,
        coefficient_ap: 0.0f64,
        key: "W".to_string(),
        variation_name: None,
        cast_time_ms: Some((cast_time_s * 1000f64) as u64),
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: None,
        damage_type: Some(DamageType::Physical),
        active_effect: None,
        recast_gap_duration: None,
        recast_charges: None,
        recast_window: None,
    });

    // E
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            khazix.abilities.e[0].effects[0].leveling[0].modifiers[0].values[rank - 1],
        );
    }

    let mut cooldown_ms: HashMap<u64, u64> = HashMap::new();
    for rank in 1..=5usize {
        cooldown_ms.insert(
            rank.try_into().unwrap(),
            khazix.abilities.e[0].cooldown.as_ref().unwrap().modifiers[0].values[rank - 1] as u64
                * 1000u64,
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage,
        ap_damage,
        coefficient_ad: khazix.abilities.e[0].effects[0].leveling[0].modifiers[1].values[0] * 0.01,
        coefficient_ap: 0.0f64,
        key: "E".to_string(),
        variation_name: None,
        cast_time_ms: None,
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: Some(SpellCategory::Dash),
        damage_type: Some(DamageType::Physical),
        active_effect: None,
        recast_gap_duration: None,
        recast_charges: None,
        recast_window: None,
    });

    // R
    let mut cooldown_ms: HashMap<u64, u64> = HashMap::new();
    for rank in 1..=3usize {
        cooldown_ms.insert(
            rank.try_into().unwrap(),
            khazix.abilities.r[0].cooldown.as_ref().unwrap().modifiers[0].values[rank - 1] as u64
                * 1000u64,
        );
    }

    let recast_charges = if config
        .get("CHAMPION_KHAZIX_R_EVOLVED")
        .unwrap_or(&"FALSE".to_string())
        == "TRUE"
    {
        2
    } else {
        1
    };

    abilities_data.push(SpellData {
        ad_damage: HashMap::new(),
        ap_damage: HashMap::new(),
        coefficient_ad: 0.0f64,
        coefficient_ap: 0.0f64,
        key: "R".to_string(),
        variation_name: None,
        cast_time_ms: None,
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: Some(SpellCategory::Stealth),
        damage_type: None,
        active_effect: Some(&KhazixR {
            base_duration: 1250,
            evolved_duration: 2000,
        }),
        recast_gap_duration: Some(2000),
        recast_charges: Some(recast_charges),
        recast_window: Some(12_000),
    });

    // println!("abilities_data {:#?}", abilities_data);

    (
        abilities_data,
        AbilitiesExtraData {
            unseen_threat: UnseenThreat {
                base_damage: 8.0,
                per_level_bonus: 6.0,
                bonus_ad_ratio: 0.4,
            },
        },
    )
}

pub fn find_ability<'a>(
    abilities: &'a Vec<SpellData>,
    spell_name: AttackType,
    configs: &HashMap<String, String>,
) -> &'a SpellData {
    // println!("abilities {:#?}", abilities);
    // println!("spell_name {:#?}", spell_name);
    // println!("configs {:#?}", configs);
    let mut variation_name: Option<String> = None;

    if spell_name == AttackType::Q {
        // Khazix's Q
        if configs.get("CHAMPION_KHAZIX_ISOLATED_TARGET").unwrap() == "TRUE" {
            variation_name = Some("Increased Damage".to_string());
        } else {
            variation_name = Some("Physical Damage".to_string());
        }
    }

    let ability: &SpellData = if variation_name.is_some() {
        abilities
            .iter()
            .find(|&x| x.key == spell_name.to_string() && x.variation_name == variation_name)
            .unwrap()
    } else {
        abilities
            .iter()
            .find(|&x| x.key == spell_name.to_string())
            .unwrap()
    };

    ability
}
