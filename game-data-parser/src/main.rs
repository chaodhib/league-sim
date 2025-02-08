use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use core::panic;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use shared_structs::champions::Champion;
use shared_structs::items::ItemSourceData;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

fn main() -> std::io::Result<()> {
    champions_gen();
    abilities_gen();
    items_gen();
    Ok(())
}

fn champions_gen() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer: File = File::create("../league-sim-backend/src/data_input/champions_gen.rs")?;

    let file = File::open("source_3/champions_formatted.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let characters: HashMap<String, HashMap<String, Value>> =
        serde_json::from_reader(reader).unwrap();
    let character = characters.get("Khazix").unwrap();

    let id = character["id"].as_u64().unwrap();
    let key = character["key"].as_str().unwrap().to_string();
    let attack_type = character["attackType"].as_str().unwrap();
    let adaptive_type = character["adaptiveType"].as_str().unwrap();

    let armor_flat = character["stats"]["armor"]["flat"].as_f64().unwrap();
    let armor_per_level = character["stats"]["armor"]["perLevel"].as_f64().unwrap();
    let attack_damage_flat = character["stats"]["attackDamage"]["flat"].as_f64().unwrap();
    let attack_damage_per_level = character["stats"]["attackDamage"]["perLevel"]
        .as_f64()
        .unwrap();
    let attack_speed_flat = character["stats"]["attackSpeed"]["flat"].as_f64().unwrap();
    let attack_speed_per_level = character["stats"]["attackSpeed"]["perLevel"]
        .as_f64()
        .unwrap()
        / 100.0;
    let attack_speed_ratio = character["stats"]["attackSpeedRatio"]["flat"]
        .as_f64()
        .unwrap();
    let attack_delay_offset = character["stats"]["attackDelayOffset"]["flat"]
        .as_f64()
        .unwrap();
    let attack_cast_time = character["stats"]["attackCastTime"]["flat"]
        .as_f64()
        .unwrap();
    let attack_total_time = character["stats"]["attackTotalTime"]["flat"]
        .as_f64()
        .unwrap();
    let base_movement_speed = character["stats"]["movespeed"]["flat"].as_f64().unwrap();

    let tokens = quote! {
        use super::{
            champions::{AdaptiveType, AttackType, ChampionStats},
            common::Champion,
            ChampionData,
        };

        pub fn get_base_champion_stats(champion: Champion) -> (ChampionData, ChampionStats) {
            let champion_data = match champion {
                Champion::Khazix => ChampionData {
                    name: Champion::Khazix,
                    id: #id,
                    key: #key.to_string(),
                    attack_type: AttackType::from_str(#attack_type),
                    adaptive_type: AdaptiveType::from_str(#adaptive_type),
                },
            };

            let champion_stats = match champion {
                Champion::Khazix => ChampionStats {
                    armor_flat: #armor_flat,
                    armor_per_level: #armor_per_level,
                    attack_damage_flat: #attack_damage_flat,
                    attack_damage_per_level: #attack_damage_per_level,
                    attack_speed_flat: #attack_speed_flat,
                    attack_speed_per_level: #attack_speed_per_level,
                    attack_speed_ratio: #attack_speed_ratio,
                    attack_delay_offset: #attack_delay_offset,
                    attack_cast_time: #attack_cast_time,
                    attack_total_time: #attack_total_time,
                    base_movement_speed: #base_movement_speed,
                },
            };

            (champion_data, champion_stats)
        }

    };

    buffer.write_all(format_rust(tokens).as_bytes());

    Ok(())
}

fn abilities_gen() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("source_3/champions/Khazix.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let jd = &mut serde_json::Deserializer::from_reader(reader);

    let result: Result<Champion, _> = serde_path_to_error::deserialize(jd);
    match result {
        Ok(champion) => {
            let path = Path::new("../league-sim-backend/src/data_input/champions_gen/khazix.rs");
            uneval::to_file(champion, path).expect("Write failed");
        }
        Err(err) => {
            let path = err.path().to_string();
            panic!("Parsing error at path: {}", path);
        }
    };

    Ok(())
}

fn items_gen() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("source_3/items.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let jd = &mut serde_json::Deserializer::from_reader(reader);

    let result: Result<HashMap<String, ItemSourceData>, _> = serde_path_to_error::deserialize(jd);
    match result {
        Ok(item_map) => {
            let path = Path::new("../league-sim-backend/src/data_input/items_gen.rs");

            let mut mut_map = item_map.clone();

            mut_map.retain(|_, item| item.shop.purchasable);

            for (_, item) in mut_map.iter_mut() {
                let stats = item.stats.as_ref().unwrap();
                let mut shop = item.shop.clone();
                if shop.prices.clone().unwrap().total == 0 {
                    shop.prices = None;
                }
                *item = ItemSourceData {
                    name: item.name.clone(),
                    stats: convert_to_none_if_empty(shared_structs::items::Stats {
                        ability_power: none_if_empty(stats.ability_power.clone()),
                        armor: none_if_empty(stats.armor.clone()),
                        armor_penetration: none_if_empty(stats.armor_penetration.clone()),
                        attack_damage: none_if_empty(stats.attack_damage.clone()),
                        attack_speed: none_if_empty(stats.attack_speed.clone()),
                        cooldown_reduction: none_if_empty(stats.cooldown_reduction.clone()),
                        critical_strike_chance: none_if_empty(stats.critical_strike_chance.clone()),
                        gold_per10: none_if_empty(stats.gold_per10.clone()),
                        heal_and_shield_power: none_if_empty(stats.heal_and_shield_power.clone()),
                        health: none_if_empty(stats.health.clone()),
                        health_regen: none_if_empty(stats.health_regen.clone()),
                        lethality: none_if_empty(stats.lethality.clone()),
                        lifesteal: none_if_empty(stats.lifesteal.clone()),
                        magic_penetration: none_if_empty(stats.magic_penetration.clone()),
                        magic_resistance: none_if_empty(stats.magic_resistance.clone()),
                        mana: none_if_empty(stats.mana.clone()),
                        mana_regen: none_if_empty(stats.mana_regen.clone()),
                        movespeed: none_if_empty(stats.movespeed.clone()),
                        ability_haste: none_if_empty(stats.ability_haste.clone()),
                        omnivamp: none_if_empty(stats.omnivamp.clone()),
                        tenacity: none_if_empty(stats.tenacity.clone()),
                    }),
                    id: item.id.clone(),
                    tier: item.tier.clone(),
                    rank: item.rank.clone(),
                    removed: item.removed.clone(),
                    icon: item.icon.clone(),
                    passives: item
                        .passives
                        .clone()
                        .iter()
                        .map(|passive| {
                            let mut passive = passive.clone();
                            let stats = convert_to_none_if_empty(passive.stats.unwrap());
                            passive.stats = stats;

                            passive
                        })
                        .collect(),
                    active: item.active.clone(),
                    shop: shop.clone(),
                };
            }

            uneval::to_file(mut_map, path).expect("Write failed");
        }
        Err(err) => {
            let path = err.path().to_string();
            panic!("Parsing error at path: {}", path);
        }
    };

    Ok(())
}

fn convert_to_none_if_empty(
    stats: shared_structs::items::Stats,
) -> Option<shared_structs::items::Stats> {
    let stats = shared_structs::items::Stats {
        ability_power: none_if_empty(stats.ability_power.clone()),
        armor: none_if_empty(stats.armor.clone()),
        armor_penetration: none_if_empty(stats.armor_penetration.clone()),
        attack_damage: none_if_empty(stats.attack_damage.clone()),
        attack_speed: none_if_empty(stats.attack_speed.clone()),
        cooldown_reduction: none_if_empty(stats.cooldown_reduction.clone()),
        critical_strike_chance: none_if_empty(stats.critical_strike_chance.clone()),
        gold_per10: none_if_empty(stats.gold_per10.clone()),
        heal_and_shield_power: none_if_empty(stats.heal_and_shield_power.clone()),
        health: none_if_empty(stats.health.clone()),
        health_regen: none_if_empty(stats.health_regen.clone()),
        lethality: none_if_empty(stats.lethality.clone()),
        lifesteal: none_if_empty(stats.lifesteal.clone()),
        magic_penetration: none_if_empty(stats.magic_penetration.clone()),
        magic_resistance: none_if_empty(stats.magic_resistance.clone()),
        mana: none_if_empty(stats.mana.clone()),
        mana_regen: none_if_empty(stats.mana_regen.clone()),
        movespeed: none_if_empty(stats.movespeed.clone()),
        ability_haste: none_if_empty(stats.ability_haste.clone()),
        omnivamp: none_if_empty(stats.omnivamp.clone()),
        tenacity: none_if_empty(stats.tenacity.clone()),
    };

    if stats.ability_power.is_none()
        && stats.armor.is_none()
        && stats.armor_penetration.is_none()
        && stats.attack_damage.is_none()
        && stats.attack_speed.is_none()
        && stats.cooldown_reduction.is_none()
        && stats.critical_strike_chance.is_none()
        && stats.gold_per10.is_none()
        && stats.heal_and_shield_power.is_none()
        && stats.health.is_none()
        && stats.health_regen.is_none()
        && stats.lethality.is_none()
        && stats.lifesteal.is_none()
        && stats.magic_penetration.is_none()
        && stats.magic_resistance.is_none()
        && stats.mana.is_none()
        && stats.mana_regen.is_none()
        && stats.movespeed.is_none()
        && stats.ability_haste.is_none()
        && stats.omnivamp.is_none()
        && stats.tenacity.is_none()
    {
        None
    } else {
        Some(stats)
    }
}

fn none_if_empty(
    details: Option<shared_structs::items::StatDetails>,
) -> Option<shared_structs::items::StatDetails> {
    if details.is_some() {
        if details.as_ref().unwrap().flat != 0f64
            || details.as_ref().unwrap().percent != 0f64
            || details.as_ref().unwrap().per_level != 0f64
            || details.as_ref().unwrap().percent_per_level != 0f64
            || details.as_ref().unwrap().percent_base != 0f64
            || details.as_ref().unwrap().percent_bonus != 0f64
        {
            return details.clone();
        }
    }

    None
}

pub fn format_rust(contents: impl ToTokens) -> String {
    let contents =
        syn::parse2(contents.to_token_stream()).expect("Unable to parse the tokens as a syn::File");
    prettyplease::unparse(&contents)
}
