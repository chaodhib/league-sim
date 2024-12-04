use std::{
    collections::{HashMap, HashSet, VecDeque},
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

use crossbeam::queue::ArrayQueue;
use data_input::{
    common::{compile_passive_effects, Champion, CritHandlingChoice, GameParams, TargetStats},
    items::{above_gold_cap, has_item_group_duplicates, Item},
    runes::Rune,
};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

struct Build {
    damage: f64,
    dps: f64,
    item_ids: Vec<u64>,
    time_ms: u64,
}

#[derive(Debug)]
struct TopResult {
    damage: f64,
    dps: f64,
    item_names: Vec<String>,
    cost: u64,
    time_ms: u64,
}

fn main() -> std::io::Result<()> {
    // run_multiple();
    run_single();

    Ok(())
}

fn run_multiple() {
    let global_start = Instant::now();

    let mut selected_commands = VecDeque::new();
    selected_commands.push_back(attack::AttackType::Q);
    selected_commands.push_back(attack::AttackType::W);
    selected_commands.push_back(attack::AttackType::E);
    selected_commands.push_back(attack::AttackType::AA);
    selected_commands.push_back(attack::AttackType::Q);
    let hp_perc = 100.0;
    let level: u64 = 6;
    let gold_cap: u64 = 20000;
    let target_stats: TargetStats = TargetStats {
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
        3158, // Ionian Boots of Lucidity
        3006, // Berserker's Greaves
        3142, // Youmuu's Ghostblade
        6701, // Opportunity
        3814, // Edge of Night
        6694, // Serylda's Grudge
        6698, // Profane Hydra
        6692, // Eclipse
        3156, // Maw of Malmortius
        3179, // Umbral Glaive
        6697, // Hubris
        6333, // Death's Dance
        3036, // Lord Dominik's Regards
        3033, // Mortal Reminder
        6609, // Chempunk Chainsword
        3071, // Black Cleaver
        6676, // The Collector
        3072, // Bloodthirster
    ];

    let mut runes: HashSet<Rune> = HashSet::new();
    runes.insert(Rune::DarkHarvest);
    runes.insert(Rune::SuddenImpact);
    runes.insert(Rune::EyeballCollection);
    runes.insert(Rune::AbsoluteFocus);
    runes.insert(Rune::GatheringStorm);
    runes.insert(Rune::AdaptiveForce1);
    runes.insert(Rune::AdaptiveForce2);

    let static_data = data_input::parse_files(&item_ids);

    // return;

    let perms = item_ids.into_iter().combinations(5);
    let progress = Arc::new(AtomicUsize::new(0));
    let size: usize = perms.size_hint().1.unwrap();
    let best_builds: ArrayQueue<Build> = ArrayQueue::new(size);

    // perms.par_bridge().for_each(|selected_item_ids| {
    perms.for_each(|selected_item_ids| {
        let now = Instant::now();
        // let champ_stats: ChampionStats = base_champion_stats.clone();
        let mut selected_items: Vec<&Item> = Vec::new();
        // println!("items:");
        for selected_item_id in selected_item_ids.iter() {
            let new_item = static_data.items_map.get(selected_item_id).unwrap();
            // // println!("{:#?}", new_item.name);
            selected_items.push(new_item);
        }

        if has_item_group_duplicates(&selected_items) || above_gold_cap(&selected_items, &gold_cap)
        {
            return;
        }

        let mut game_params: GameParams<'_> = GameParams {
            champion_stats: &static_data.base_champion_stats,
            champion: Champion::Khazix,
            level: level,
            items: &selected_items,
            initial_config: &config,
            abilities: &static_data.abilities,
            target_stats: &target_stats,
            runes: &runes,
            attacker_hp_perc: hp_perc,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
        };

        compile_passive_effects(&mut game_params);

        let (damage, time_ms) = simulation::run(selected_commands.clone(), &game_params);

        // println!("DPS:: {:#?}", damage * (1000_f64 / time_ms as f64));
        let build = Build {
            damage: damage.clone(),
            item_ids: selected_item_ids.clone(),
            dps: damage.clone() * (1000_f64 / time_ms as f64),
            time_ms,
        };

        let push_result = best_builds.push(build);
        if push_result.is_err() {
            panic!();
        }

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
        .sorted_by(|a, b| b.dps.partial_cmp(&a.dps).unwrap())
        .take(3)
        .map(|build| {
            let item_names = build
                .item_ids
                .iter()
                .map(|item_id| static_data.items_map.get(item_id).unwrap().name.clone())
                .collect_vec();

            let cost = build
                .item_ids
                .iter()
                .map(|item_id| static_data.items_map.get(item_id).unwrap())
                .fold(0, |acc, item| acc + item.total_cost);

            TopResult {
                damage: build.damage,
                dps: build.dps,
                item_names,
                cost,
                time_ms: build.time_ms,
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
}

fn run_single() {
    let global_start = Instant::now();

    let mut selected_commands = VecDeque::new();
    selected_commands.push_back(attack::AttackType::Q);
    selected_commands.push_back(attack::AttackType::W);
    selected_commands.push_back(attack::AttackType::E);
    selected_commands.push_back(attack::AttackType::AA);

    let hp_perc: f64 = 100.0;
    let level: u64 = 6;
    let _gold_cap: u64 = 20000;
    let target_stats: TargetStats = TargetStats {
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
        3158, // Ionian Boots of Lucidity
        3006, // Berserker's Greaves
        3142, // Youmuu's Ghostblade
        6701, // Opportunity
        3814, // Edge of Night
        6694, // Serylda's Grudge
        6698, // Profane Hydra
        6692, // Eclipse
        3156, // Maw of Malmortius
        3179, // Umbral Glaive
        6697, // Hubris
        6333, // Death's Dance
        3036, // Lord Dominik's Regards
        3033, // Mortal Reminder
        6609, // Chempunk Chainsword
        3071, // Black Cleaver
        6676, // The Collector
        3072, // Bloodthirster
    ];

    let static_data = data_input::parse_files(&item_ids);

    let mut selected_items: Vec<&Item> = Vec::new();

    let selected_item_names: Vec<&str> = vec![
        "Ionian Boots of Lucidity",
        // "Berserker's Greaves",
        // "Youmuu's Ghostblade",
        // "Profane Hydra",
        // "Bloodthirster",
        // "Opportunity",
        // "Black Cleaver",
        // "Serylda's Grudge",
    ];

    let mut runes: HashSet<Rune> = HashSet::new();
    runes.insert(Rune::DarkHarvest);
    runes.insert(Rune::SuddenImpact);
    runes.insert(Rune::EyeballCollection);
    runes.insert(Rune::AbsoluteFocus);
    runes.insert(Rune::GatheringStorm);
    runes.insert(Rune::AdaptiveForce1);
    runes.insert(Rune::AdaptiveForce2);

    for ele in selected_item_names.iter() {
        let found_item = static_data
            .items_map
            .values()
            .find(|&item| item.name == *ele.to_string())
            .unwrap();
        selected_items.push(found_item);
    }

    let mut game_params: GameParams<'_> = GameParams {
        champion: Champion::Khazix,
        champion_stats: &static_data.base_champion_stats,
        level: level,
        items: &selected_items,
        initial_config: &config,
        abilities: &static_data.abilities,
        target_stats: &target_stats,
        runes_data: &static_data.runes_data,
        runes: &runes,
        attacker_hp_perc: hp_perc,
        passive_effects: &mut Vec::new(),
        crit_handling: CritHandlingChoice::Min,
    };

    compile_passive_effects(&mut game_params);

    let (damage, time_ms) = simulation::run(selected_commands.clone(), &game_params);
    println!("damage: {:#?}", damage);
    println!("time_ms: {:#?}", time_ms);
    println!("DPS:: {:#?}", damage * (1000_f64 / time_ms as f64));

    let global_elapsed = global_start.elapsed();
    println!("Elapsed: {:.2?}", global_elapsed);
}
