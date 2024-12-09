use std::{collections::HashMap, fs::File, io::BufReader};

use serde_json::Value;

use crate::{
    attack::{compute_mitigated_damage, AttackType, SpellCategory},
    simulation,
};

use super::common::{
    compute_attacker_stats, compute_target_stats, DamageType, EffectWithCallbacks, PassiveEffect,
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
}

pub struct UnseenThreat {
    base_damage: f64,
    per_level_bonus: f64,
    bonus_ad_ratio: f64,
}

impl EffectWithCallbacks for UnseenThreat {
    fn on_post_damage(
        &self,
        damage: f64,
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

        if event.attack_type.is_some_and(|x| x != AttackType::AA) {
            return;
        }

        let attacker_stats = compute_attacker_stats(game_params, state);
        let target_stats = compute_target_stats(game_params, state);

        let magic_damage: f64 = self.base_damage
            + self.per_level_bonus * game_params.level as f64
            + self.bonus_ad_ratio * attacker_stats.ad_bonus;

        let mitigated_dmg = compute_mitigated_damage(
            &attacker_stats,
            &target_stats,
            magic_damage,
            DamageType::Magical,
        );

        simulation::on_damage_from_ability(&mitigated_dmg, state, event.time_ms, AttackType::P);

        // remove buff
        state.remove_attacker_aura(
            &super::common::Aura::UnseenThreat,
            game_params,
            event,
            events,
        );
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
        state.add_attacker_aura(
            super::common::Aura::UnseenThreat,
            u64::MAX,
            game_params,
            event,
            events,
        );

        let stealth_duration = if game_params
            .initial_config
            .get("CHAMPION_KHAZIX_R_EVOLVED")
            .unwrap()
            == "TRUE"
        {
            self.evolved_duration
        } else {
            self.base_duration
        };

        state.add_attacker_aura(
            super::common::Aura::Invisibility,
            stealth_duration,
            game_params,
            event,
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
    let file = File::open("/home/chaodhib/git/lolstaticdata/champions/Khazix.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let json_input: HashMap<String, Value> = serde_json::from_reader(reader).unwrap();
    let abilities = json_input.get("abilities").unwrap();

    let mut abilities_data = Vec::new();

    // Q (variation 1)
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            abilities["Q"][0]["effects"][1]["leveling"][0]["modifiers"][0]["values"][rank - 1]
                .as_f64()
                .unwrap(),
        );
    }

    let cast_time_s = abilities["Q"][0]["castTime"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let mut cooldown_ms: HashMap<u64, u64> = HashMap::new();
    for rank in 1..=5usize {
        cooldown_ms.insert(
            rank.try_into().unwrap(),
            abilities["Q"][0]["cooldown"]["modifiers"][0]["values"][rank - 1]
                .as_u64()
                .unwrap()
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
        coefficient_ad: abilities["Q"][0]["effects"][1]["leveling"][0]["modifiers"][1]["values"][0]
            .as_f64()
            .unwrap()
            * 0.01,
        coefficient_ap: 0.0f64,
        key: "Q".to_string(),
        variation_name: Some("Physical Damage".to_string()),
        cast_time_ms: Some((cast_time_s * 1000f64) as u64),
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: None,
        damage_type: Some(DamageType::Physical),
        active_effect: None,
    });

    // Q (variation 2)
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            abilities["Q"][0]["effects"][1]["leveling"][1]["modifiers"][0]["values"][rank - 1]
                .as_f64()
                .unwrap(),
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage,
        ap_damage,
        coefficient_ad: abilities["Q"][0]["effects"][1]["leveling"][1]["modifiers"][1]["values"][0]
            .as_f64()
            .unwrap()
            * 0.01,
        coefficient_ap: 0.0f64,
        key: "Q".to_string(),
        variation_name: Some("Increased Damage".to_string()),
        cast_time_ms: Some((cast_time_s * 1000f64) as u64),
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: None,
        damage_type: Some(DamageType::Physical),
        active_effect: None,
    });

    // W
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            abilities["W"][0]["effects"][0]["leveling"][0]["modifiers"][0]["values"][rank - 1]
                .as_f64()
                .unwrap(),
        );
    }

    let cast_time_s = abilities["W"][0]["castTime"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let mut cooldown_ms: HashMap<u64, u64> = HashMap::new();
    for rank in 1..=5usize {
        cooldown_ms.insert(
            rank.try_into().unwrap(),
            abilities["W"][0]["cooldown"]["modifiers"][0]["values"][rank - 1]
                .as_u64()
                .unwrap()
                * 1000u64,
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage,
        ap_damage,
        coefficient_ad: abilities["W"][0]["effects"][0]["leveling"][0]["modifiers"][1]["values"][0]
            .as_f64()
            .unwrap()
            * 0.01,
        coefficient_ap: 0.0f64,
        key: "W".to_string(),
        variation_name: None,
        cast_time_ms: Some((cast_time_s * 1000f64) as u64),
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: None,
        damage_type: Some(DamageType::Physical),
        active_effect: None,
    });

    // E
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            abilities["E"][0]["effects"][0]["leveling"][0]["modifiers"][0]["values"][rank - 1]
                .as_f64()
                .unwrap(),
        );
    }

    let mut cooldown_ms: HashMap<u64, u64> = HashMap::new();
    for rank in 1..=5usize {
        cooldown_ms.insert(
            rank.try_into().unwrap(),
            abilities["E"][0]["cooldown"]["modifiers"][0]["values"][rank - 1]
                .as_u64()
                .unwrap()
                * 1000u64,
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage,
        ap_damage,
        coefficient_ad: abilities["E"][0]["effects"][0]["leveling"][0]["modifiers"][1]["values"][0]
            .as_f64()
            .unwrap()
            * 0.01,
        coefficient_ap: 0.0f64,
        key: "E".to_string(),
        variation_name: None,
        cast_time_ms: None,
        cooldown_ms: Some(cooldown_ms.clone()),
        // passive_effects: Vec::new(),
        category: Some(SpellCategory::Dash),
        damage_type: Some(DamageType::Physical),
        active_effect: None,
    });

    // R
    let mut cooldown_ms: HashMap<u64, u64> = HashMap::new();
    for rank in 1..=3usize {
        cooldown_ms.insert(
            rank.try_into().unwrap(),
            abilities["R"][0]["cooldown"]["modifiers"][0]["values"][rank - 1]
                .as_u64()
                .unwrap()
                * 1000u64,
        );
    }

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
