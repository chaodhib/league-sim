use std::{cmp, collections::HashMap, fs::File, io::BufReader};

use serde_json::Value;

use crate::{
    attack::{compute_mitigated_damage, AttackType},
    simulation::{self, on_post_damage_events, DamageInfo, DamageSource, State},
};

use super::common::{
    compute_attacker_stats, compute_target_stats, AttackerStats, DamageType, GameParams,
    PassiveEffect,
};

use shared_structs::items_cdragon::*;
use shared_structs::items_meraki::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item {
    Unknown,
    IonianBootsofLucidity,
    BerserkersGreaves,
    YoumuusGhostblade,
    Opportunity,
    EdgeofNight,
    SeryldasGrudge,
    ProfaneHydra,
    Eclipse,
    MawofMalmortius,
    UmbralGlaive,
    Hubris,
    DeathsDance,
    LordDominiksRegards,
    MortalReminder,
    ChempunkChainsword,
    BlackCleaver,
    TheCollector,
    Bloodthirster,
    VoltaicCyclosword,
    SerpentsFang,
    GuardianAngel,
    SpearofShojin,
    AxiomArc,
    SunderedSky,
    RavenousHydra,
    RanduinsOmen,
    FrozenHeart,
    Stridebreaker,
    BladeofTheRuinedKing,
}

impl Item {
    pub fn from_string(string: String) -> Option<Self> {
        match string.as_str() {
            "Ionian Boots of Lucidity" => Some(Item::IonianBootsofLucidity),
            "Berserker's Greaves" => Some(Item::BerserkersGreaves),
            "Youmuu's Ghostblade" => Some(Item::YoumuusGhostblade),
            "Opportunity" => Some(Item::Opportunity),
            "Edge of Night" => Some(Item::EdgeofNight),
            "Serylda's Grudge" => Some(Item::SeryldasGrudge),
            "Profane Hydra" => Some(Item::ProfaneHydra),
            "Eclipse" => Some(Item::Eclipse),
            "Maw of Malmortius" => Some(Item::MawofMalmortius),
            "Umbral Glaive" => Some(Item::UmbralGlaive),
            "Hubris" => Some(Item::Hubris),
            "Death's Dance" => Some(Item::DeathsDance),
            "Lord Dominik's Regards" => Some(Item::LordDominiksRegards),
            "Mortal Reminder" => Some(Item::MortalReminder),
            "Chempunk Chainsword" => Some(Item::ChempunkChainsword),
            "Black Cleaver" => Some(Item::BlackCleaver),
            "The Collector" => Some(Item::TheCollector),
            "Bloodthirster" => Some(Item::Bloodthirster),
            "Voltaic Cyclosword" => Some(Item::VoltaicCyclosword),
            "Serpent's Fang" => Some(Item::SerpentsFang),
            "Guardian Angel" => Some(Item::GuardianAngel),
            "Spear of Shojin" => Some(Item::SpearofShojin),
            "Axiom Arc" => Some(Item::AxiomArc),
            "Sundered Sky" => Some(Item::SunderedSky),
            "Ravenous Hydra" => Some(Item::RavenousHydra),
            "Randuin's Omen" => Some(Item::RanduinsOmen),
            "Frozen Heart" => Some(Item::FrozenHeart),
            "Stridebreaker" => Some(Item::Stridebreaker),
            "Blade of the Ruined King" => Some(Item::BladeofTheRuinedKing),
            &_ => None,
        }
    }

    pub fn to_string(self) -> String {
        match self {
            Item::Unknown => "".to_string(),
            Item::IonianBootsofLucidity => "Ionian Boots of Lucidity".to_string(),
            Item::BerserkersGreaves => "Berserker's Greaves".to_string(),
            Item::YoumuusGhostblade => "Youmuu's Ghostblade".to_string(),
            Item::Opportunity => "Opportunity".to_string(),
            Item::EdgeofNight => "Edge of Night".to_string(),
            Item::SeryldasGrudge => "Serylda's Grudge".to_string(),
            Item::ProfaneHydra => "Profane Hydra".to_string(),
            Item::Eclipse => "Eclipse".to_string(),
            Item::MawofMalmortius => "Maw of Malmortius".to_string(),
            Item::UmbralGlaive => "Umbral Glaive".to_string(),
            Item::Hubris => "Hubris".to_string(),
            Item::DeathsDance => "Death's Dance".to_string(),
            Item::LordDominiksRegards => "Lord Dominik's Regards".to_string(),
            Item::MortalReminder => "Mortal Reminder".to_string(),
            Item::ChempunkChainsword => "Chempunk Chainsword".to_string(),
            Item::BlackCleaver => "Black Cleaver".to_string(),
            Item::TheCollector => "The Collector".to_string(),
            Item::Bloodthirster => "Bloodthirster".to_string(),
            Item::VoltaicCyclosword => "Voltaic Cyclosword".to_string(),
            Item::SerpentsFang => "Serpent's Fang".to_string(),
            Item::GuardianAngel => "Guardian Angel".to_string(),
            Item::SpearofShojin => "Spear of Shojin".to_string(),
            Item::AxiomArc => "Axiom Arc".to_string(),
            Item::SunderedSky => "Sundered Sky".to_string(),
            Item::RavenousHydra => "Ravenous Hydra".to_string(),
            Item::RanduinsOmen => "Randuin's Omen".to_string(),
            Item::FrozenHeart => "Frozen Heart".to_string(),
            Item::Stridebreaker => "Stridebreaker".to_string(),
            Item::BladeofTheRuinedKing => "Blade of the Ruined King".to_string(),
        }
    }

    pub fn handle_on_pre_damage(
        &self,
        passive_effect: &PassiveEffect,
        damage_info: &DamageInfo,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        match &self {
            Item::SunderedSky => match passive_effect {
                PassiveEffect::LightshieldStrike => {
                    if damage_info
                        .source_ability
                        .is_none_or(|attack_type| attack_type != AttackType::AA)
                    {
                        return;
                    }

                    if state
                        .effects_cooldowns
                        .contains_key(&PassiveEffect::LightshieldStrike)
                    {
                        return;
                    }

                    state.add_target_aura(
                        super::common::Aura::LightshieldStrike,
                        None,
                        None,
                        events,
                    );

                    state
                        .effects_cooldowns
                        .insert(PassiveEffect::LightshieldStrike, event.time_ms + 8_000);
                }
                _ => panic!("Unhandled passive effect for SunderedSky"),
            },
            Item::VoltaicCyclosword => match passive_effect {
                PassiveEffect::Energized => {
                    if damage_info
                        .source_ability
                        .is_none_or(|attack_type| attack_type != AttackType::AA)
                    {
                        return;
                    }

                    if let Some(aura_app) =
                        state.attacker_auras.get(&super::common::Aura::Energized)
                    {
                        if aura_app.stacks.unwrap() == 100 {
                            state.end_early_attacker_aura(
                                &super::common::Aura::Energized,
                                game_params,
                                event,
                                events,
                            );

                            let target_stats = compute_target_stats(game_params, state);

                            const UNMITIGATED_DAMAGE: f64 = 100.0;

                            let mitigated_damage = compute_mitigated_damage(
                                attacker_stats,
                                &target_stats,
                                UNMITIGATED_DAMAGE,
                                DamageType::Physical,
                            );

                            simulation::on_damage_from_item(
                                &mitigated_damage,
                                DamageType::Physical,
                                state,
                                Item::VoltaicCyclosword,
                            );

                            return;
                        }
                    }

                    const STACKS_PER_AA: u64 = 6;
                    if let Some(aura_app) = state
                        .attacker_auras
                        .get_mut(&super::common::Aura::Energized)
                    {
                        aura_app.stacks = Some(aura_app.stacks.unwrap() + STACKS_PER_AA);
                    } else {
                        state.add_attacker_aura(
                            super::common::Aura::Energized,
                            None,
                            Some(cmp::min(STACKS_PER_AA, 100)),
                            events,
                        );
                    };
                }
                _ => panic!("Unhandled passive effect for SunderedSky"),
            },
            Item::BladeofTheRuinedKing => match passive_effect {
                PassiveEffect::MistsEdge => {
                    if damage_info
                        .source_ability
                        .is_none_or(|attack_type| attack_type != AttackType::AA)
                    {
                        return;
                    }

                    let perc_hp_dmg: f64 = match game_params.champion_data.attack_type {
                        super::champions::AttackType::Melee => 0.08,
                        super::champions::AttackType::Ranged => 0.05,
                    };

                    let target_stats = compute_target_stats(game_params, state);

                    let unmitigated_damage = target_stats.current_health * perc_hp_dmg;
                    let mitigated_damage = compute_mitigated_damage(
                        attacker_stats,
                        &target_stats,
                        unmitigated_damage,
                        DamageType::Physical,
                    );

                    simulation::on_damage_from_item(
                        &mitigated_damage,
                        DamageType::Physical,
                        state,
                        Item::BladeofTheRuinedKing,
                    );
                }
                _ => panic!("Unhandled passive effect for BladeofTheRuinedKing"),
            },
            _ => todo!(),
        }
    }

    pub fn handle_on_post_damage(
        &self,
        passive_effect: &PassiveEffect,
        damage_info: &DamageInfo,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        match &self {
            Item::BlackCleaver => match passive_effect {
                PassiveEffect::Carve => {
                    if damage_info.damage_type == DamageType::Physical {
                        let stack = if let Some(aura_app) =
                            state.target_auras.get(&super::common::Aura::Carve)
                        {
                            aura_app.stacks.unwrap() + 1
                        } else {
                            1
                        };

                        state.add_target_aura(
                            super::common::Aura::Carve,
                            Some(6_000),
                            Some(cmp::min(stack, 5)),
                            events,
                        );

                        // println!("Carve stack:  {:#?}", stack);
                    }
                }
                _ => panic!("Unhandled passive effect for BlackCleaver"),
            },
            Item::Eclipse => match passive_effect {
                PassiveEffect::EverRisingMoon => {
                    if damage_info.source != DamageSource::Ability
                        && damage_info.source != DamageSource::ItemActive
                    {
                        return;
                    }

                    if state
                        .effects_cooldowns
                        .contains_key(&super::common::PassiveEffect::EverRisingMoon)
                    {
                        return;
                    }

                    if let Some(aura_app) =
                        state.target_auras.get(&super::common::Aura::EverRisingMoon)
                    {
                        state.end_early_target_aura(
                            &super::common::Aura::EverRisingMoon,
                            game_params,
                            event,
                            events,
                        );

                        state.effects_cooldowns.insert(
                            super::common::PassiveEffect::EverRisingMoon,
                            event.time_ms + 6_000,
                        );

                        let perc_hp_dmg: f64 = match game_params.champion_data.attack_type {
                            super::champions::AttackType::Melee => 0.06,
                            super::champions::AttackType::Ranged => 0.04,
                        };

                        let target_stats = compute_target_stats(game_params, state);

                        let unmitigated_damage = target_stats.max_health * perc_hp_dmg;
                        let mitigated_damage = compute_mitigated_damage(
                            attacker_stats,
                            &target_stats,
                            unmitigated_damage,
                            DamageType::Physical,
                        );

                        let damage_info = simulation::on_damage_from_item(
                            &mitigated_damage,
                            DamageType::Physical,
                            state,
                            Item::Eclipse,
                        );

                        on_post_damage_events(
                            &damage_info,
                            &attacker_stats,
                            state,
                            game_params,
                            event,
                            events,
                        );
                    } else {
                        state.add_target_aura(
                            super::common::Aura::EverRisingMoon,
                            Some(2_000),
                            Some(1),
                            events,
                        );
                    };
                }
                _ => panic!("Unhandled passive effect for Eclipse"),
            },
            Item::SpearofShojin => match passive_effect {
                PassiveEffect::FocusedWill => {
                    if damage_info.source_ability.is_some_and(|attack_type| {
                        vec![AttackType::Q, AttackType::W, AttackType::E].contains(&attack_type)
                    }) {
                        let stack = if let Some(aura_app) =
                            state.attacker_auras.get(&super::common::Aura::FocusedWill)
                        {
                            aura_app.stacks.unwrap() + 1
                        } else {
                            1
                        };

                        state.add_attacker_aura(
                            super::common::Aura::FocusedWill,
                            Some(6_000),
                            Some(cmp::min(stack, 4)),
                            events,
                        );
                    }
                }
                _ => panic!("Unhandled passive effect for SpearofShojin"),
            },
            Item::SunderedSky => match passive_effect {
                PassiveEffect::LightshieldStrike => {
                    state.end_early_attacker_aura(
                        &super::common::Aura::LightshieldStrike,
                        game_params,
                        event,
                        events,
                    );
                }
                _ => panic!("Unhandled passive effect for SunderedSky"),
            },
            Item::TheCollector => match passive_effect {
                PassiveEffect::Death => {
                    let current_health =
                        game_params.initial_target_stats.current_health - state.total_damage;
                    let hp_prc = current_health / game_params.initial_target_stats.max_health;
                    if hp_prc < 5.0 {
                        simulation::on_damage_from_item(
                            &current_health,
                            DamageType::True,
                            state,
                            Item::TheCollector,
                        );
                    }
                }
                _ => panic!("Unhandled passive effect for SunderedSky"),
            },
            _ => todo!(),
        }
    }

    pub(crate) fn handle_on_movement(
        &self,
        passive_effect: &PassiveEffect,
        duration: u64,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        match &self {
            Item::VoltaicCyclosword => match passive_effect {
                PassiveEffect::Energized => {
                    let attacker_stats = compute_attacker_stats(game_params, state);
                    let move_speed = (attacker_stats.movement_speed_base
                        + attacker_stats.movement_speed_flat_bonus)
                        * (1.0 + attacker_stats.movement_speed_perc_bonus);

                    let distance_traveled = move_speed * duration as f64 / 1000.0;
                    let generated_stacks = (distance_traveled / 24.0) as u64;

                    // println!(
                    //     "time: {:#?}. generated_stacks: {:#?}. move_speed: {:#?}. distance_traveled: {:#?}. duration: {:#?}",
                    //     state.time_ms, generated_stacks, move_speed, distance_traveled, duration
                    // );

                    if let Some(aura_app) = state
                        .attacker_auras
                        .get_mut(&super::common::Aura::Energized)
                    {
                        let new_stacks = cmp::min(aura_app.stacks.unwrap() + generated_stacks, 100);
                        aura_app.stacks = Some(new_stacks);
                    } else {
                        state.add_attacker_aura(
                            super::common::Aura::Energized,
                            None,
                            Some(cmp::min(generated_stacks, 100)),
                            events,
                        );
                    };
                }
                _ => panic!("Unhandled passive effect for SunderedSky"),
            },
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct ItemData {
    pub item: Item,
    pub id: u64,
    pub total_cost: u64,
    pub offensive_stats: AttackerStats,
    pub item_groups: Vec<String>,
    pub passives: Vec<PassiveEffect>,
}

pub fn pull_items_data(item_ids: &[u64]) -> HashMap<u64, ItemData> {
    let item_map: HashMap<String, ItemDataCdragon> = include!("items_gen/items_cdragon_gen.rs");
    let item_map_meraki: HashMap<String, ItemDataMeraki> =
        include!("items_gen/items_meraki_gen.rs");

    let mut map = HashMap::new();
    let mut sanity_checker: Vec<String> = Vec::new();
    for (_key, item_data_meraki) in item_map_meraki.iter() {
        let item_id = item_data_meraki.id as u64;
        let item_data = item_map.get(&format!("Items/{item_id}")).unwrap();

        let stats = AttackerStats {
            ability_haste: item_data_meraki
                .clone()
                .stats
                .unwrap_or_default()
                .ability_haste
                .unwrap_or_default()
                .flat,
            // ad_base: 0.0,
            ad_bonus: item_data_meraki
                .clone()
                .stats
                .unwrap_or_default()
                .attack_damage
                .unwrap_or_default()
                .flat,
            armor_penetration_perc: item_data_meraki
                .clone()
                .stats
                .unwrap_or_default()
                .armor_penetration
                .unwrap_or_default()
                .percent
                / 100.0,
            crit_chance: item_data_meraki
                .clone()
                .stats
                .unwrap_or_default()
                .critical_strike_chance
                .unwrap_or_default()
                .percent
                / 100.0,
            lethality: item_data_meraki
                .clone()
                .stats
                .unwrap_or_default()
                .lethality
                .unwrap_or_default()
                .flat,
            // attack_speed_base: 0.0,
            attack_speed_bonus: item_data_meraki
                .clone()
                .stats
                .unwrap_or_default()
                .attack_speed
                .unwrap_or_default()
                .flat
                / 100.0,
            movement_speed_flat_bonus: item_data_meraki
                .clone()
                .stats
                .unwrap_or_default()
                .movespeed
                .unwrap_or_default()
                .flat,
            movement_speed_perc_bonus: item_data_meraki
                .clone()
                .stats
                .unwrap_or_default()
                .movespeed
                .unwrap_or_default()
                .percent
                / 100.0,
            ..Default::default()
        };

        let mut item_groups = Vec::new();

        let item_groups_source = item_data.m_item_groups.clone();
        for item_group_source in item_groups_source.iter() {
            let new_value = item_group_source.as_str();
            if new_value != "Items/ItemGroups/Default" {
                item_groups.push(new_value.to_string());
            }

            if new_value.starts_with("{") {
                sanity_checker.push(new_value.to_string());
            }
        }

        let mut passives = Vec::new();

        for passive in item_data_meraki.passives.iter() {
            let passive_name = passive.name.clone().unwrap_or_default();
            if let Some(passive_effect) = PassiveEffect::from_string(&passive_name) {
                // println!(
                //     "{:#?},{:#?}",
                //     item_data_meraki.name,
                //     passive_name.to_string()
                // );
                passives.push(passive_effect);
            }
        }

        let item = ItemData {
            id: item_id,
            item: Item::from_string(item_data_meraki.name.clone()).unwrap_or(Item::Unknown),
            total_cost: item_data_meraki
                .shop
                .prices
                .clone()
                .unwrap_or_default()
                .total as u64,
            offensive_stats: stats,
            item_groups,
            passives: passives,
        };

        map.insert(item.id, item);
    }

    // ensure that the unknown hashes amongst the item groups are not causing issues
    sanity_checker.sort();
    let length_before_dedup = sanity_checker.len();
    sanity_checker.dedup();
    let length_after_dedup = sanity_checker.len();
    if length_before_dedup != length_after_dedup {
        println!("{:#?}", sanity_checker);
        panic!();
    }

    map
}

pub fn has_item_group_duplicates(selected_items: &[&ItemData]) -> bool {
    let mut item_groups_present: Vec<String> = Vec::new();

    for selected_item in selected_items.iter() {
        item_groups_present.extend(selected_item.item_groups.clone())
    }

    item_groups_present.sort();
    let length_before_dedup = item_groups_present.len();
    item_groups_present.dedup();
    let length_after_dedup = item_groups_present.len();

    length_before_dedup != length_after_dedup
}

pub fn above_gold_cap(selected_items: &[&ItemData], gold_cap: &u64) -> bool {
    let build_cost: u64 = selected_items
        .iter()
        .fold(0, |acc, item| acc + item.total_cost);

    build_cost > *gold_cap
}
