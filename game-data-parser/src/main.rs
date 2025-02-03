use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use core::panic;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use shared_structs::champions::Champion;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

fn main() -> std::io::Result<()> {
    champions_gen();
    abilities_gen();
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

pub fn format_rust(contents: impl ToTokens) -> String {
    let contents =
        syn::parse2(contents.to_token_stream()).expect("Unable to parse the tokens as a syn::File");
    prettyplease::unparse(&contents)
}
