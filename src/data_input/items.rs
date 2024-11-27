use std::{collections::HashMap, fs::File, io::BufReader};

use serde_json::Value;

use super::common::OffensiveStats;

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub id: u64,
    pub total_cost: u64,
    pub offensive_stats: OffensiveStats,
    pub item_groups: Vec<String>,
}

pub fn pull_items_data(item_ids: &[u64]) -> HashMap<u64, Item> {
    let file = File::open("source_2/items_formatted.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let json_input: HashMap<String, Value> = serde_json::from_reader(reader).unwrap();

    let mut map = HashMap::new();
    let mut sanity_checker: Vec<String> = Vec::new();
    for ele in json_input.iter() {
        let item_id = ele.1["itemID"].as_u64().unwrap_or_default();
        if !item_ids.contains(&item_id) {
            continue;
        }

        let stats = OffensiveStats {
            ability_haste: ele.1["mAbilityHasteMod"].as_f64().unwrap_or_default(),
            // ad_base: 0.0,
            ad_bonus: ele.1["mFlatPhysicalDamageMod"].as_f64().unwrap_or_default(),
            armor_penetration_perc: ele.1["mPercentArmorPenetrationMod"]
                .as_f64()
                .unwrap_or_default(),
            crit_chance: ele.1["mFlatCritChanceMod"].as_f64().unwrap_or_default(),
            lethality: ele.1["PhysicalLethality"].as_f64().unwrap_or_default(),
            // attack_speed_base: 0.0,
            attack_speed_bonus: ele.1["mPercentAttackSpeedMod"].as_f64().unwrap_or_default(),
            ..Default::default()
        };

        let mut item_groups = Vec::new();

        let item_groups_source = ele.1["mItemGroups"].as_array().unwrap();
        for item_group_source in item_groups_source.iter() {
            let new_value = item_group_source.as_str().unwrap();
            if new_value != "Items/ItemGroups/Default" {
                item_groups.push(new_value.to_string());
            }

            if new_value.starts_with("{") {
                sanity_checker.push(new_value.to_string());
            }
        }

        let item = Item {
            id: ele.1["itemID"].as_u64().unwrap(),
            name: "".to_string(),
            total_cost: 0,
            offensive_stats: stats,
            item_groups,
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

    enrich_items_data(&mut map);

    map
}

pub fn has_item_group_duplicates(selected_items: &[&Item]) -> bool {
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

pub fn above_gold_cap(selected_items: &[&Item], gold_cap: &u64) -> bool {
    let build_cost: u64 = selected_items
        .iter()
        .fold(0, |acc, item| acc + item.total_cost);

    build_cost > *gold_cap
}

fn enrich_items_data(items_map: &mut HashMap<u64, Item>) {
    let file = File::open("source_1/items_formatted.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let json_input: Vec<Value> = serde_json::from_reader(reader).unwrap();

    for ele in items_map.iter_mut() {
        // let item_key = format!("Items/{}", ele.0);
        let item_data = json_input
            .iter()
            .find(|&x| &x["id"].as_u64().unwrap_or_default() == ele.0)
            .unwrap();

        ele.1.name = item_data["name"].as_str().unwrap().to_string();
        ele.1.total_cost = item_data["priceTotal"].as_u64().unwrap();
    }
}
