use std::{
    collections::{HashMap, VecDeque},
    ops::{Add, Mul},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

mod attack;
mod data_input;
mod simulation;

use data_input::{
    common::{DefensiveStats, GameParams},
    items::Item,
};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

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

#[derive(Debug)]
struct SpellResult {
    damage: Damage,
    duration: u64,
}

fn main() -> std::io::Result<()> {
    let global_start = Instant::now();
    // let base_champion_stats: ChampionStats = get_base_champion_stats();
    // let item_ids: Vec<u64> = vec![
    //     //3158,
    //     3142, 6701, 3814, 6694, 6698, 6692, 3156, 3179, 6697, 6333, 3036, 3033, 6609, 3071, 6676,
    //     3072,
    // ];

    // let mut items_map: HashMap<u64, Item> = pull_item_stats(&item_ids);

    // enrich_items_data(&mut items_map);

    // let abilities: Vec<SpellData> = pull_abilities_data();

    let mut selected_commands = VecDeque::new();
    selected_commands.push_back(simulation::AttackType::Q);
    selected_commands.push_back(simulation::AttackType::W);
    selected_commands.push_back(simulation::AttackType::E);
    selected_commands.push_back(simulation::AttackType::AA);

    let level: u64 = 6;
    let gold_cap: u64 = 20000;
    let target_stats: DefensiveStats = DefensiveStats {
        armor: 100.0,
        hp: 2600.0,
    };

    // list of configs provided by the user. Static once the simulation is running
    let mut config: HashMap<String, String> = HashMap::new();
    config.insert(
        "CHAMPION_KHAZIX_ISOLATED_TARGET".to_string(),
        "TRUE".to_string(),
    );

    let item_ids: Vec<u64> = vec![
        3158, 3142, 6701, 3814, 6694, 6698, 6692, 3156, 3179, 6697, 6333, 3036, 3033, 6609, 3071,
        6676, 3072,
    ];

    let static_data = data_input::parse_files(&item_ids);

    let mut selected_items: Vec<&Item> = Vec::new();

    let selected_item_names: Vec<&str> = vec![
        "Ionian Boots of Lucidity",
        // "Youmuu's Ghostblade",
        // "Profane Hydra",
        "Bloodthirster",
        // "Opportunity",
        // "Black Cleaver",
        // "Serylda's Grudge",
    ];

    for ele in selected_item_names.iter() {
        let found_item = static_data
            .items_map
            .values()
            .find(|&item| item.name == *ele.to_string())
            .unwrap();
        selected_items.push(found_item);
    }

    let game_params: GameParams<'_> = GameParams {
        champion_stats: &static_data.base_champion_stats,
        level: level,
        items: &selected_items,
        config: &config,
        abilities: &static_data.abilities,
        // off_stats: off_stats,
        def_stats: &target_stats,
    };

    simulation::run(selected_commands, &game_params);

    return Ok(());

    // let perms = item_ids.into_iter().combinations(5);
    // let progress = Arc::new(AtomicUsize::new(0));
    // let size: usize = perms.size_hint().1.unwrap();
    // let best_builds: ArrayQueue<Build> = ArrayQueue::new(size);

    // // perms.par_bridge().for_each(|selected_item_ids| {
    // perms.for_each(|selected_item_ids| {
    //     let now = Instant::now();
    //     let champ_stats: ChampionStats = base_champion_stats.clone();
    //     let mut selected_items: Vec<&Item> = Vec::new();
    //     // println!("items:");
    //     for selected_item_id in selected_item_ids.iter() {
    //         // let new_item = items_map.get(selected_item_id).unwrap();
    //         // // println!("{:#?}", new_item.name);
    //         // selected_items.push(new_item);
    //     }

    //     if has_item_group_duplicates(&selected_items) || above_gold_cap(&selected_items, &gold_cap)
    //     {
    //         return;
    //     }

    //     println!("selected_items: {:#?}", selected_items);
    //     // let burst_total_damage: Damage = simulate_burst(
    //     //     &selected_items,
    //     //     &champ_stats,
    //     //     &level,
    //     //     &def_stats,
    //     //     &config,
    //     //     &abilities,
    //     // );
    //     let burst_total_damage = Damage {
    //         avg: 0.0f64,
    //         min: 0.0f64,
    //         max: 0.0f64,
    //     };
    //     println!("total_damage: {:#?}", burst_total_damage);

    //     let build = Build {
    //         damage: burst_total_damage,
    //         item_ids: selected_item_ids.clone(),
    //     };

    //     let push_result = best_builds.push(build);
    //     if push_result.is_err() {
    //         panic!();
    //     }

    //     let current_progress = progress.fetch_add(1, Ordering::Relaxed);
    //     let elapsed = now.elapsed();
    //     println!("Elapsed: {:.2?}", elapsed);
    //     println!(
    //         "Progress: {:#?}%",
    //         current_progress as f64 / size as f64 * 100.0
    //     );
    // });

    // let results = best_builds
    //     .into_iter()
    //     .sorted_by(|a, b| b.damage.min.partial_cmp(&a.damage.min).unwrap())
    //     .take(3)
    //     .map(|build| {
    //         let item_names = build
    //             .item_ids
    //             .iter()
    //             .map(|item_id| items_map.get(item_id).unwrap().name.clone())
    //             .collect_vec();

    //         let cost = build
    //             .item_ids
    //             .iter()
    //             .map(|item_id| items_map.get(item_id).unwrap())
    //             .fold(0, |acc, item| acc + item.total_cost);

    //         TopResult {
    //             min_damage: build.damage.min,
    //             max_damage: build.damage.max,
    //             avg_damage: build.damage.avg,
    //             item_names,
    //             cost,
    //         }
    //     })
    //     .collect_vec();

    // println!("Top results: {:#?}", results);
    // // println!("Top damage: {:#?}", top_total_damage);
    // // println!("Top build:");
    // // for item_id in top_build.iter() {
    // //     println!("{:#?}", items_map.get(item_id).unwrap().name);
    // // }

    // let global_elapsed = global_start.elapsed();
    // println!("Elapsed: {:.2?}", global_elapsed);

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

    Ok(())
}

// fn simulate_burst(
//     selected_items: &Vec<&Item>,
//     champ_stats: &ChampionStats,
//     level: &u64,
//     def_stats: &DefensiveStats,
//     config: &HashMap<String, String>,
//     abilities: &Vec<SpellData>,
// ) -> Damage {
//     let off_stats: OffensiveStats =
//         compute_source_champion_stats(champ_stats, *level as f64, &selected_items);
//     // println!("level: {:#?}, source_stats: {:#?}", level, off_stats);

//     let q_damage = simulate_spell(&off_stats, level, def_stats, "Q", config, abilities);
//     let aa_damage = simulate_spell(&off_stats, level, def_stats, "AA", config, abilities);
//     let w_damage = simulate_spell(&off_stats, level, def_stats, "W", config, abilities);
//     let e_damage = simulate_spell(&off_stats, level, def_stats, "E", config, abilities);
//     // println!("q_damage: {:#?}", q_damage);
//     // println!("aa_damage: {:#?}", aa_damage);
//     // println!("w_damage: {:#?}", w_damage);
//     // println!("e_damage: {:#?}", e_damage);

//     let burst_total_damage: Damage = q_damage * 1.0 + aa_damage * 1.0 + w_damage + e_damage;

//     return burst_total_damage;
// }
