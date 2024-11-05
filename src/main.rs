// use ddragon::{
// cache_middleware::CacheMiddleware, models::Items, Client, ClientBuilder, ClientError,
// };
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    ops::{Add, Mul},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use crossbeam::queue::ArrayQueue;
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    id: u64,
    total_cost: u64,
    offensive_stats: OffensiveStats,
    item_groups: Vec<String>,
}

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Offensive
#[derive(Serialize, Deserialize, Debug)]
struct OffensiveStats {
    pub ability_haste: f64,
    pub ad_base: f64,
    pub ad_bonus: f64,
    pub lethality: f64,
    pub armor_penetration_perc: f64,
    pub crit_chance: f64,
}

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Defensive
#[derive(Serialize, Deserialize, Debug)]
struct DefensiveStats {
    pub armor: f64,
    pub hp: f64,
}

#[derive(Clone)]
struct ChampionStats {
    pub armor_flat: f64,
    pub armor_per_level: f64,
    pub attack_damage_flat: f64,
    pub attack_damage_per_level: f64,
    pub attack_speed_flat: f64,
    pub attack_speed_per_level: f64,
}

struct Build {
    damage: Damage,
    item_ids: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopResult {
    min_damage: f64,
    max_damage: f64,
    avg_damage: f64,
    item_names: Vec<String>,
    cost: u64,
}

struct Damage {
    min: f64,
    max: f64,
    avg: f64,
}

impl Add for Damage {
    type Output = Self;

    fn add(self, other: Damage) -> Damage {
        Damage {
            min: self.min + other.min,
            max: self.max + other.max,
            avg: self.avg + other.avg,
        }
    }
}

impl Mul<f64> for Damage {
    type Output = Self;

    fn mul(self, other: f64) -> Damage {
        Damage {
            min: self.min * other,
            max: self.max * other,
            avg: self.avg * other,
        }
    }
}

fn main() -> std::io::Result<()> {
    let global_start = Instant::now();
    // let client = ClientBuilder::new().build()?;
    // let champions = client.champions()?;
    // let runes = client.runes()?;
    // let items: Items = client.items()?;
    // println!("version {:#?}", items.version);

    // for ele in p.iter() {
    //     if has_desireable_stats(ele.1) {
    //         println!("key: {:#?}. itemID: {:#?}", ele.0, ele.1["name"]);
    //     }
    // }

    let item_ids: Vec<u64> = vec![
        //3158,
        3142, 6701, 3814, 6694, 6698, 6692, 3156, 3179, 6697, 6333, 3036, 3033, 6609, 3071, 6676,
        3072,
    ];

    let mut items_map: HashMap<u64, Item> = pull_item_stats(&item_ids);
    // for ele in items.iter() {
    //     println!("{:#?}", ele);
    // }

    enrich_items_data(&mut items_map);

    let level: u64 = 18;
    let gold_cap: u64 = 99_000;
    let def_stats: DefensiveStats = DefensiveStats {
        armor: 100.0,
        hp: 2600.0,
    };

    let perms = item_ids.into_iter().combinations(5);
    let progress = Arc::new(AtomicUsize::new(0));
    let size: usize = perms.size_hint().1.unwrap();
    let best_builds: ArrayQueue<Build> = ArrayQueue::new(size);
    let base_champion_stats = get_base_champion_stats();

    perms.par_bridge().for_each(|selected_item_ids| {
        let now = Instant::now();
        let champ_stats: ChampionStats = base_champion_stats.clone();
        let mut selected_items: Vec<&Item> = Vec::new();
        // println!("items:");
        for selected_item_id in selected_item_ids.iter() {
            let new_item = items_map.get(selected_item_id).unwrap();
            // println!("{:#?}", new_item.name);
            selected_items.push(new_item);
        }

        if has_item_group_duplicates(&selected_items) || above_gold_cap(&selected_items, &gold_cap)
        {
            return;
        }

        let burst_total_damage: Damage =
            simulate_burst(&selected_items, &champ_stats, &level, &def_stats);
        // println!("total_damage: {:#?}", burst_total_damage);

        let build = Build {
            damage: burst_total_damage,
            item_ids: selected_item_ids.clone(),
        };

        let push_result = best_builds.push(build);
        if push_result.is_err() {
            panic!();
        }

        // if burst_total_damage > top_total_damage {
        //     top_total_damage = burst_total_damage;
        //     top_build = selected_item_ids;
        // }

        let current_progress = progress.fetch_add(1, Ordering::Relaxed);
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        println!(
            "Progress: {:#?}%",
            current_progress as f64 / size as f64 * 100.0
        );
    });

    let results = best_builds
        .into_iter()
        .sorted_by(|a, b| b.damage.min.partial_cmp(&a.damage.min).unwrap())
        .take(50)
        .map(|build| {
            let item_names = build
                .item_ids
                .iter()
                .map(|item_id| items_map.get(item_id).unwrap().name.clone())
                .collect_vec();

            let cost = build
                .item_ids
                .iter()
                .map(|item_id| items_map.get(item_id).unwrap())
                .fold(0, |acc, item| acc + item.total_cost);

            TopResult {
                min_damage: build.damage.min,
                max_damage: build.damage.max,
                avg_damage: build.damage.avg,
                item_names,
                cost,
            }
        })
        .collect_vec();

    println!("Top results: {:#?}", results);
    // println!("Top damage: {:#?}", top_total_damage);
    // println!("Top build:");
    // for item_id in top_build.iter() {
    //     println!("{:#?}", items_map.get(item_id).unwrap().name);
    // }

    let global_elapsed = global_start.elapsed();
    println!("Elapsed: {:.2?}", global_elapsed);

    // println!("{:#?}", perms);
    // itertools::assert_equal(
    //     perms,
    //     vec![
    //         vec![5, 6],
    //         vec![5, 7],
    //         vec![6, 5],
    //         vec![6, 7],
    //         vec![7, 5],
    //         vec![7, 6],
    //     ],
    // );

    // let selected_item_names: Vec<&str> = vec![
    //     "Ionian Boots of Lucidity",
    //     // "Youmuu's Ghostblade",
    //     // "Profane Hydra",
    //     "Bloodthirster",
    //     // "Opportunity",
    //     // "Black Cleaver",
    //     // "Serylda's Grudge",
    // ];

    // for ele in selected_item_names.iter() {
    //     let found_item = items
    //         .iter()
    //         .find(|&item| item.name == ele.to_string())
    //         .unwrap();
    //     selected_items.insert(found_item.id, found_item);
    // }

    Ok(())
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

fn pull_item_stats(item_ids: &Vec<u64>) -> HashMap<u64, Item> {
    let file = File::open("source_3/items_formatted.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let json_input: HashMap<String, Value> = serde_json::from_reader(reader).unwrap();

    let mut map = HashMap::new();
    for ele in json_input.iter() {
        let item_id = ele.1["id"].as_u64().unwrap();
        if !item_ids.contains(&item_id) {
            continue;
        }

        let stats = OffensiveStats {
            ability_haste: ele.1["stats"]["abilityHaste"]["flat"].as_f64().unwrap_or(
                ele.1["stats"]["ability_haste"]["flat"]
                    .as_f64()
                    .unwrap_or_default(),
            ),
            ad_base: 0.0,
            ad_bonus: ele.1["stats"]["attackDamage"]["flat"].as_f64().unwrap_or(
                ele.1["stats"]["attack_damage"]["flat"]
                    .as_f64()
                    .unwrap_or_default(),
            ),
            armor_penetration_perc: ele.1["stats"]["armorPenetration"]["percent"]
                .as_f64()
                .unwrap_or(
                    ele.1["stats"]["armor_penetration"]["percent"]
                        .as_f64()
                        .unwrap_or_default(),
                ),
            crit_chance: ele.1["stats"]["criticalStrikeChance"]["percent"]
                .as_f64()
                .unwrap_or(
                    ele.1["stats"]["critical_strike_chance"]["percent"]
                        .as_f64()
                        .unwrap_or_default(),
                ),
            lethality: ele.1["stats"]["lethality"]["flat"]
                .as_f64()
                .unwrap_or_default(),
        };

        let item = Item {
            id: ele.1["id"].as_u64().unwrap(),
            name: ele.1["name"].as_str().unwrap().to_string(),
            total_cost: ele.1["shop"]["prices"]["total"].as_u64().unwrap(),
            offensive_stats: stats,
            item_groups: Vec::new(),
        };

        map.insert(item.id, item);
    }

    return map;
}

fn compute_source_champion_stats(
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

fn get_base_champion_stats() -> ChampionStats {
    let file = File::open("source_3/champions_formatted.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let json_input: HashMap<String, Value> = serde_json::from_reader(reader).unwrap();
    let character = json_input.get("Khazix").unwrap();

    return ChampionStats {
        armor_flat: character["stats"]["armor"]["flat"].as_f64().unwrap(),
        armor_per_level: character["stats"]["armor"]["perLevel"].as_f64().unwrap(),
        attack_damage_flat: character["stats"]["attackDamage"]["flat"].as_f64().unwrap(),
        attack_damage_per_level: character["stats"]["attackDamage"]["perLevel"]
            .as_f64()
            .unwrap(),
        attack_speed_flat: character["stats"]["attackSpeed"]["flat"].as_f64().unwrap(),
        attack_speed_per_level: character["stats"]["attackSpeed"]["perLevel"]
            .as_f64()
            .unwrap(),
    };
}

fn stat_increase(per_level: f64, level: f64) -> f64 {
    return per_level * (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0));
}

fn apply_passives(offensive_stats: &mut OffensiveStats, items: &Vec<&Item>) {
    // todo: change this in a callback fashion
    if items
        .iter()
        .find(|&elem| elem.name == "Serylda's Grudge".to_string())
        .is_some()
    {
        offensive_stats.armor_penetration_perc += 20.0 + offensive_stats.lethality * 0.11;
    }
}

fn compute_q_damage(off_stats: &OffensiveStats, def_stats: &DefensiveStats, level: &u64) -> Damage {
    let spell_rank = match level {
        1..=3 => 1,
        4 => 2,
        5..=6 => 3,
        7..=8 => 4,
        9..=18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };
    // let base_damage = match spell_rank {
    //     1 => 70.0,
    //     2 => 95.0,
    //     3 => 120.0,
    //     4 => 145.0,
    //     5 => 170.0,
    //     0_u64 | 6_u64..=u64::MAX => panic!(),
    // };

    let mut base_damage: f64 = match spell_rank {
        1 => 147.0,
        2 => 199.5,
        3 => 252.0,
        4 => 304.5,
        5 => 357.0,
        0_u64 | 6_u64..=u64::MAX => panic!(),
    };

    // println!("1 base_damage: {:#?}", base_damage);

    // include AD ratio
    base_damage += 2.31 * off_stats.ad_bonus;

    // println!("2 base_damage: {:#?}", base_damage);
    let dmg = compute_mitigated_damage(def_stats, off_stats, base_damage);
    return Damage {
        min: dmg,
        max: dmg,
        avg: dmg,
    };
}

fn compute_aa_damage(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    _level: &u64,
) -> Damage {
    let base_damage: f64 = off_stats.ad_base + off_stats.ad_bonus;
    let crit_damage: f64 = if off_stats.crit_chance > 0.0 {
        base_damage * 1.75
    } else {
        base_damage
    };
    let avg_damage: f64 = base_damage * (1.0 + off_stats.crit_chance * 0.75 / 100.0);

    // println!("1 base_damage: {:#?}", base_damage);

    return Damage {
        min: compute_mitigated_damage(def_stats, off_stats, base_damage),
        max: compute_mitigated_damage(def_stats, off_stats, crit_damage),
        avg: compute_mitigated_damage(def_stats, off_stats, avg_damage),
    };
}

fn compute_w_damage(off_stats: &OffensiveStats, def_stats: &DefensiveStats, level: &u64) -> Damage {
    let spell_rank = match level {
        1 => 0,
        2..=7 => 1,
        8..=9 => 2,
        10..=11 => 3,
        12 => 4,
        13..=18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    let mut base_damage: f64 = match spell_rank {
        1 => 85.0,
        2 => 115.0,
        3 => 145.0,
        4 => 175.0,
        5 => 205.0,
        0_u64 | 6_u64..=u64::MAX => panic!(),
    };

    // println!("1 base_damage: {:#?}", base_damage);

    // include AD ratio
    base_damage += 1.0 * off_stats.ad_bonus;

    // println!("2 base_damage: {:#?}", base_damage);

    let dmg = compute_mitigated_damage(def_stats, off_stats, base_damage);

    return Damage {
        min: dmg,
        max: dmg,
        avg: dmg,
    };
}

fn compute_e_damage(off_stats: &OffensiveStats, def_stats: &DefensiveStats, level: &u64) -> Damage {
    let spell_rank = match level {
        1..=2 => 0,
        3..=13 => 1,
        14 => 2,
        15..=16 => 3,
        17 => 4,
        18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    let mut base_damage: f64 = match spell_rank {
        1 => 65.0,
        2 => 100.0,
        3 => 135.0,
        4 => 170.0,
        5 => 205.0,
        0_u64 | 6_u64..=u64::MAX => panic!(),
    };

    // println!("1 base_damage: {:#?}", base_damage);

    // include AD ratio
    base_damage += 0.2 * off_stats.ad_bonus;

    // println!("2 base_damage: {:#?}", base_damage);

    let dmg = compute_mitigated_damage(def_stats, off_stats, base_damage);

    return Damage {
        min: dmg,
        max: dmg,
        avg: dmg,
    };
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
    armor *= 1.0 - off_stats.armor_penetration_perc / 100.0;

    // println!("4 armor: {:#?}", armor);

    // include lethality
    armor = (armor - off_stats.lethality).max(0.0);

    // println!("5 armor: {:#?}", armor);

    return base_damage * 100.0 / (100.0 + armor);
}

fn simulate_spell(
    off_stats: &OffensiveStats,
    level: &u64,
    def_stats: &DefensiveStats,
    spell_name: &str,
) -> Damage {
    //     let source_stats = compute_source_champion_stats(level as f64, &selected_items);
    //     println!("level: {:#?}, source_stats: {:#?}", level, source_stats);
    // }

    let damage: Damage = match spell_name {
        "Q" => compute_q_damage(off_stats, def_stats, level),
        "AA" => compute_aa_damage(off_stats, def_stats, level),
        "W" => compute_w_damage(off_stats, def_stats, level),
        "E" => compute_e_damage(off_stats, def_stats, level),
        &_ => todo!(),
    };

    // println!("damage: {:#?}", damage);

    return damage;
}

fn simulate_burst(
    selected_items: &Vec<&Item>,
    champ_stats: &ChampionStats,
    level: &u64,
    def_stats: &DefensiveStats,
) -> Damage {
    let off_stats: OffensiveStats =
        compute_source_champion_stats(champ_stats, *level as f64, &selected_items);
    // println!("level: {:#?}, source_stats: {:#?}", level, off_stats);

    let q_damage = simulate_spell(&off_stats, level, def_stats, "Q");
    let aa_damage = simulate_spell(&off_stats, level, def_stats, "AA");
    let w_damage = simulate_spell(&off_stats, level, def_stats, "W");
    let e_damage = simulate_spell(&off_stats, level, def_stats, "E");

    let burst_total_damage: Damage = q_damage * 1.0 + aa_damage * 1.0 + w_damage + e_damage;

    return burst_total_damage;
}

fn enrich_items_data(items_map: &mut HashMap<u64, Item>) {
    let file = File::open("source_2/items_formatted.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let json_input: HashMap<String, Value> = serde_json::from_reader(reader).unwrap();

    let mut sanity_checker: Vec<String> = Vec::new();

    for ele in items_map.iter_mut() {
        let item_key = format!("Items/{}", ele.0);
        let item_data = json_input.get(&item_key).unwrap();

        let item_groups = item_data["mItemGroups"].as_array().unwrap();
        for item_group_source in item_groups.iter() {
            let new_value = item_group_source.as_str().unwrap();
            if new_value != "Items/ItemGroups/Default" {
                ele.1.item_groups.push(new_value.to_string());
            }

            if new_value.starts_with("{") {
                sanity_checker.push(new_value.to_string());
            }
        }
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
}

fn has_item_group_duplicates(selected_items: &Vec<&Item>) -> bool {
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

fn above_gold_cap(selected_items: &Vec<&Item>, gold_cap: &u64) -> bool {
    let build_cost: u64 = selected_items
        .iter()
        .fold(0, |acc, item| acc + item.total_cost);

    build_cost > *gold_cap
}
