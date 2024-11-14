use std::{
    collections::HashMap,
    ops::{Add, Mul},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use abilities::SpellData;
use champions::ChampionStats;
use crossbeam::queue::ArrayQueue;
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

mod abilities;
mod champions;
mod game_data;
mod items;

use abilities::*;
use champions::*;
use game_data::*;
use items::*;

struct Build {
    damage: Damage,
    item_ids: Vec<u64>,
}

#[derive(Debug)]
struct TopResult {
    min_damage: f64,
    max_damage: f64,
    avg_damage: f64,
    item_names: Vec<String>,
    cost: u64,
}

#[derive(Debug)]
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

    let base_champion_stats: ChampionStats = get_base_champion_stats();
    // println!("base_champion_stats {:#?}", base_champion_stats);

    let item_ids: Vec<u64> = vec![
        //3158,
        3142, 6701, 3814, 6694, 6698, 6692, 3156, 3179, 6697, 6333, 3036, 3033, 6609, 3071, 6676,
        3072,
    ];

    let mut items_map: HashMap<u64, Item> = items::pull_item_stats(&item_ids);

    enrich_items_data(&mut items_map);
    // for ele in items_map.iter() {
    //     println!("{:#?}", ele);
    // }

    let abilities: Vec<SpellData> = pull_abilities_data();

    // println!("abilities_data: {:#?}", abilities);

    let level: u64 = 6;
    let gold_cap: u64 = 20000;
    let def_stats: DefensiveStats = DefensiveStats {
        armor: 100.0,
        hp: 2600.0,
    };

    let mut config: HashMap<String, String> = HashMap::new();
    config.insert(
        "CHAMPION_KHAZIX_ISOLATED_TARGET".to_string(),
        "TRUE".to_string(),
    );

    let perms = item_ids.into_iter().combinations(5);
    let progress = Arc::new(AtomicUsize::new(0));
    let size: usize = perms.size_hint().1.unwrap();
    let best_builds: ArrayQueue<Build> = ArrayQueue::new(size);

    // perms.par_bridge().for_each(|selected_item_ids| {
    perms.for_each(|selected_item_ids| {
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

        println!("selected_items: {:#?}", selected_items);
        let burst_total_damage: Damage = simulate_burst(
            &selected_items,
            &champ_stats,
            &level,
            &def_stats,
            &config,
            &abilities,
        );
        println!("total_damage: {:#?}", burst_total_damage);

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
        .take(3)
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

fn compute_ability_damage(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    ability: &SpellData,
    // config: &HashMap<String, String>,
    spell_rank: &u64,
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

fn compute_q_damage(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    level: &u64,
    ability: &SpellData,
) -> Damage {
    let spell_rank: u64 = match level {
        1..=3 => 1,
        4 => 2,
        5..=6 => 3,
        7..=8 => 4,
        9..=18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    return compute_ability_damage(off_stats, def_stats, ability, &spell_rank);
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
    let avg_damage: f64 = base_damage * (1.0 + off_stats.crit_chance * 0.75);

    // println!("1 base_damage: {:#?}", base_damage);

    return Damage {
        min: compute_mitigated_damage(def_stats, off_stats, base_damage),
        max: compute_mitigated_damage(def_stats, off_stats, crit_damage),
        avg: compute_mitigated_damage(def_stats, off_stats, avg_damage),
    };
}

fn compute_w_damage(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    level: &u64,
    ability: &SpellData,
) -> Damage {
    let spell_rank = match level {
        1 => 0,
        2..=7 => 1,
        8..=9 => 2,
        10..=11 => 3,
        12 => 4,
        13..=18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    return compute_ability_damage(off_stats, def_stats, ability, &spell_rank);
}

fn compute_e_damage(
    off_stats: &OffensiveStats,
    def_stats: &DefensiveStats,
    level: &u64,
    ability: &SpellData,
) -> Damage {
    let spell_rank = match level {
        1..=2 => 0,
        3..=13 => 1,
        14 => 2,
        15..=16 => 3,
        17 => 4,
        18 => 5,
        0_u64 | 19_u64..=u64::MAX => panic!(),
    };

    return compute_ability_damage(off_stats, def_stats, ability, &spell_rank);
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

    return base_damage * 100.0 / (100.0 + armor);
}

fn simulate_spell(
    off_stats: &OffensiveStats,
    level: &u64,
    def_stats: &DefensiveStats,
    spell_name: &str,
    config: &HashMap<String, String>,
    abilities: &Vec<SpellData>,
) -> Damage {
    //     let source_stats = compute_source_champion_stats(level as f64, &selected_items);
    //     println!("level: {:#?}, source_stats: {:#?}", level, source_stats);
    // }

    let mut ability: Option<&SpellData> = None;
    if spell_name != "AA" {
        ability = Some(find_ability(abilities, spell_name, config));
    }

    let damage: Damage = match spell_name {
        "AA" => compute_aa_damage(off_stats, def_stats, level),
        "Q" => compute_q_damage(off_stats, def_stats, level, ability.unwrap()),
        "W" => compute_w_damage(off_stats, def_stats, level, ability.unwrap()),
        "E" => compute_e_damage(off_stats, def_stats, level, ability.unwrap()),
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
    config: &HashMap<String, String>,
    abilities: &Vec<SpellData>,
) -> Damage {
    let off_stats: OffensiveStats =
        compute_source_champion_stats(champ_stats, *level as f64, &selected_items);
    // println!("level: {:#?}, source_stats: {:#?}", level, off_stats);

    let q_damage = simulate_spell(&off_stats, level, def_stats, "Q", config, abilities);
    let aa_damage = simulate_spell(&off_stats, level, def_stats, "AA", config, abilities);
    let w_damage = simulate_spell(&off_stats, level, def_stats, "W", config, abilities);
    let e_damage = simulate_spell(&off_stats, level, def_stats, "E", config, abilities);
    // println!("q_damage: {:#?}", q_damage);
    // println!("aa_damage: {:#?}", aa_damage);
    // println!("w_damage: {:#?}", w_damage);
    // println!("e_damage: {:#?}", e_damage);

    let burst_total_damage: Damage = q_damage * 1.0 + aa_damage * 1.0 + w_damage + e_damage;

    return burst_total_damage;
}
