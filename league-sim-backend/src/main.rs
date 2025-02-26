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
    common::{
        compile_passive_effects, Aura, AuraApplication, Champion, CritHandlingChoice, GameParams,
        TargetStats,
    },
    items::{above_gold_cap, has_item_group_duplicates, Item, ItemData},
    runes::Rune,
};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use simulation::State;

#[derive(Debug, Clone)]
struct Build {
    damage: f64,
    dps: f64,
    item_ids: Vec<u64>,
    time_ms: u64,
    selected_commands: Vec<attack::AttackType>,
    kill: bool,
}

#[derive(Debug, Clone)]
struct TopResult {
    damage: f64,
    dps: f64,
    item_names: Vec<String>,
    cost: u64,
    time_ms: u64,
    selected_commands: Vec<attack::AttackType>,
}

fn main() -> std::io::Result<()> {
    // list of configs provided by the user. Static once the simulation is running
    let mut config: HashMap<String, String> = HashMap::new();
    config.insert(
        "CHAMPION_KHAZIX_ISOLATED_TARGET".to_string(),
        "TRUE".to_string(),
    );

    config.insert("CHAMPION_KHAZIX_Q_EVOLVED".to_string(), "TRUE".to_string());
    config.insert("CHAMPION_KHAZIX_R_EVOLVED".to_string(), "FALSE".to_string());
    config.insert("RUNE_DARK_HARVEST_STACKS".to_string(), "0".to_string());
    config.insert(
        "ITEM_HUBRIS_EMINENCE_ACTIVE".to_string(),
        "FALSE".to_string(),
    );
    config.insert(
        "ITEM_OPPORTUNITY_PREPARATION_READY".to_string(),
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
        6699, // Voltaic Cyclosword
        6695, // Serpent's Fang
        3026, // Guardian Angel
        3161, // Spear of Shojin
        6696, // Axiom Arc
        6610, // Sundered Sky
        3074, // Ravenous Hydra
        // 3004, // Manamune
        3143, // Randuin's Omen
        3110, // Frozen Heart
        6631, // Stridebreaker
        3153, // Blade of the Ruined King
    ];

    let mut runes: HashSet<Rune> = HashSet::new();
    runes.insert(Rune::DarkHarvest);
    runes.insert(Rune::SuddenImpact);
    runes.insert(Rune::AbsoluteFocus);
    runes.insert(Rune::GatheringStorm);
    runes.insert(Rune::AdaptiveForce1);
    runes.insert(Rune::AdaptiveForce2);
    // runes.insert(Rune::AbilityHaste);

    run_multiple(config, item_ids, runes);
    // run_single(config, item_ids, runes);
    // run_ttk(config, item_ids, runes);

    Ok(())
}

fn run_multiple(config: HashMap<String, String>, item_ids: Vec<u64>, runes: HashSet<Rune>) {
    let global_start = Instant::now();

    let mut selected_commands = VecDeque::new();
    selected_commands.push_back(attack::AttackType::R);
    selected_commands.push_back(attack::AttackType::AA);
    selected_commands.push_back(attack::AttackType::Q);
    selected_commands.push_back(attack::AttackType::W);
    // selected_commands.push_back(attack::AttackType::E);
    // selected_commands.push_back(attack::AttackType::R);
    // selected_commands.push_back(attack::AttackType::AA);
    // selected_commands.push_back(attack::AttackType::Q);
    let hp_perc = 100.0;
    let level: u64 = 18;
    let gold_cap: u64 = 20000;
    let target_stats: TargetStats = TargetStats {
        armor: 100.0,
        max_health: 2400.0,
        current_health: 2400.0,
        magic_resistance: 100.0,
    };

    let static_data = data_input::parse_files(Champion::Khazix, &item_ids, &config);

    // return;

    let perms = item_ids.into_iter().combinations(5);
    let progress = Arc::new(AtomicUsize::new(0));
    let size: usize = perms.size_hint().1.unwrap();
    let best_builds: ArrayQueue<Build> = ArrayQueue::new(size);

    // perms.par_bridge().for_each(|selected_item_ids| {
    perms.for_each(|selected_item_ids| {
        let now = Instant::now();
        // let champ_stats: ChampionStats = base_champion_stats.clone();
        let mut selected_items: Vec<&ItemData> = Vec::new();
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
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: level,
            items: &selected_items,
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &target_stats,
            runes: &runes,
            attacker_hp_perc: hp_perc,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &vec![AuraApplication {
                aura: Aura::UnseenThreat,
                stacks: None,
                start_ms: 0,
                end_ms: None,
            }],
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
        };

        compile_passive_effects(&mut game_params);

        let (damage, _damage_history, _, time_ms, kill) =
            simulation::run(selected_commands.clone(), &game_params);

        // println!("DPS:: {:#?}", damage * (1000_f64 / time_ms as f64));
        let build = Build {
            damage: damage.clone(),
            item_ids: selected_item_ids.clone(),
            dps: damage.clone() * (1000_f64 / time_ms as f64),
            selected_commands: selected_commands.clone().into(),
            time_ms,
            kill,
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

    let results = sort_best_builds(static_data, best_builds.into_iter().collect_vec());
    println!("Top results: {:#?}", results);

    let global_elapsed = global_start.elapsed();
    println!("Elapsed: {:.2?}", global_elapsed);
}

fn sort_best_builds(
    static_data: data_input::StaticData,
    best_builds: Vec<Build>,
) -> Vec<TopResult> {
    let results = best_builds
        .into_iter()
        // .sorted_by(|a, b| b.time_ms.partial_cmp(&a.time_ms).unwrap())
        .sorted_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap())
        .take(50)
        .map(|build| {
            let item_names = build
                .item_ids
                .iter()
                .map(|item_id| static_data.items_map.get(item_id).unwrap().item.clone())
                .map(|item_name| Item::to_string(item_name))
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
                selected_commands: build.selected_commands,
            }
        })
        .collect_vec();
    results
}

fn run_single(config: HashMap<String, String>, item_ids: Vec<u64>, runes: HashSet<Rune>) {
    let global_start = Instant::now();

    let mut selected_commands = VecDeque::new();
    selected_commands.push_back(attack::AttackType::Q);
    selected_commands.push_back(attack::AttackType::W);
    // selected_commands.push_back(attack::AttackType::E);
    // selected_commands.push_back(attack::AttackType::R);
    selected_commands.push_back(attack::AttackType::AA);
    // selected_commands.push_back(attack::AttackType::R);
    selected_commands.push_back(attack::AttackType::AA);
    selected_commands.push_back(attack::AttackType::Q);
    selected_commands.push_back(attack::AttackType::AA);
    selected_commands.push_back(attack::AttackType::Q);
    selected_commands.push_back(attack::AttackType::AA);
    selected_commands.push_back(attack::AttackType::Q);
    selected_commands.push_back(attack::AttackType::AA);
    let hp_perc = 100.0;
    let level: u64 = 18;
    let gold_cap: u64 = 20000;
    let target_stats: TargetStats = TargetStats {
        armor: 100.0,
        max_health: 2400.0,
        current_health: 2400.0,
        magic_resistance: 100.0,
    };

    let static_data = data_input::parse_files(Champion::Khazix, &item_ids, &config);

    let mut selected_items: Vec<&ItemData> = Vec::new();

    let selected_item_names: Vec<&str> = vec![
        // "Ionian Boots of Lucidity",
        // // "Eclipse",
        // // "Berserker's Greaves",
        // "Youmuu's Ghostblade",
        // // "Profane Hydra",
        // // "Bloodthirster",
        // // "Opportunity",
        // // "Black Cleaver",
        // // "Serylda's Grudge",
        // "Edge of Night",
        "Youmuu's Ghostblade",
        "Serylda's Grudge",
        "Profane Hydra",
        "Hubris",
        "Voltaic Cyclosword",
    ];

    for ele in selected_item_names.iter() {
        let found_item = static_data
            .items_map
            .values()
            .find(|&item| Item::to_string(item.item) == *ele.to_string())
            .unwrap();
        selected_items.push(found_item);
    }

    let mut game_params: GameParams<'_> = GameParams {
        champion: Champion::Khazix,
        champion_data: &static_data.champion_data,
        champion_stats: &static_data.base_champion_stats,
        level: level,
        items: &selected_items,
        initial_config: &config,
        abilities: &static_data.abilities,
        initial_target_stats: &target_stats,
        runes_data: &static_data.runes_data,
        runes: &runes,
        attacker_hp_perc: hp_perc,
        passive_effects: &mut Vec::new(),
        crit_handling: CritHandlingChoice::Min,
        // initial_attacker_auras: &vec![AuraApplication {
        //     aura: Aura::UnseenThreat,
        //     stacks: None,
        //     start_ms: 0,
        //     end_ms: None,
        // }],
        // initial_attacker_auras: &vec![],
        // initial_attacker_auras: &vec![AuraApplication {
        //     aura: Aura::HubrisEminence,
        //     stacks: Some(17),
        //     start_ms: 0,
        //     end_ms: Some(90_000),
        // }],
        initial_attacker_auras: &vec![
            AuraApplication {
                aura: Aura::UnseenThreat,
                stacks: None,
                start_ms: 0,
                end_ms: None,
            },
            AuraApplication {
                aura: Aura::Energized,
                stacks: Some(100),
                start_ms: 0,
                end_ms: None,
            },
        ],
        initial_target_auras: &Vec::new(),
        abilities_extra_data: &static_data.abilities_extra_data,
        start_time_ms: 0,
        capture_event_history: false,
    };

    compile_passive_effects(&mut game_params);

    let (damage, damage_history, _, time_ms, kill) =
        simulation::run(selected_commands.clone(), &game_params);
    println!("kill: {:#?}", kill);
    println!("damage: {:#?}", damage);
    println!("time_ms: {:#?}", time_ms);
    println!("DPS: {:#?}", damage * (1000_f64 / time_ms as f64));
    println!("damage_history: {:#?}", damage_history);

    let global_elapsed = global_start.elapsed();
    println!("Elapsed: {:.2?}", global_elapsed);
}

fn run_ttk(config: HashMap<String, String>, item_ids: Vec<u64>, runes: HashSet<Rune>) {
    let global_start = Instant::now();

    let hp_perc = 100.0;
    let level: u64 = 18;
    let gold_cap: u64 = 20000;
    let target_stats: TargetStats = TargetStats {
        armor: 100.0,
        max_health: 2400.0,
        current_health: 2400.0,
        magic_resistance: 100.0,
    };

    let static_data = data_input::parse_files(Champion::Khazix, &item_ids, &config);

    let mut selected_items: Vec<&ItemData> = Vec::new();

    let selected_item_names: Vec<&str> = vec![
        // "Ionian Boots of Lucidity",
        // // "Eclipse",
        // // "Berserker's Greaves",
        // "Youmuu's Ghostblade",
        // // "Profane Hydra",
        // // "Bloodthirster",
        // // "Opportunity",
        // // "Black Cleaver",
        // // "Serylda's Grudge",
        // "Edge of Night",
        "Youmuu's Ghostblade",
        "Serylda's Grudge",
        "Profane Hydra",
        "Hubris",
        "Voltaic Cyclosword",
    ];

    for ele in selected_item_names.iter() {
        let found_item = static_data
            .items_map
            .values()
            .find(|&item| Item::to_string(item.item) == *ele.to_string())
            .unwrap();
        selected_items.push(found_item);
    }

    let mut game_params: GameParams<'_> = GameParams {
        champion: Champion::Khazix,
        champion_data: &static_data.champion_data,
        champion_stats: &static_data.base_champion_stats,
        level: level,
        items: &selected_items,
        initial_config: &config,
        abilities: &static_data.abilities,
        initial_target_stats: &target_stats,
        runes_data: &static_data.runes_data,
        runes: &runes,
        attacker_hp_perc: hp_perc,
        passive_effects: &mut Vec::new(),
        crit_handling: CritHandlingChoice::Min,
        initial_attacker_auras: &vec![
            // AuraApplication {
            //     aura: Aura::UnseenThreat,
            //     stacks: None,
            //     start_ms: 0,
            //     end_ms: None,
            // },
            AuraApplication {
                aura: Aura::Energized,
                stacks: Some(100),
                start_ms: 0,
                end_ms: None,
            },
        ],
        initial_target_auras: &Vec::new(),
        abilities_extra_data: &static_data.abilities_extra_data,
        start_time_ms: 0,
        capture_event_history: false,
    };

    compile_passive_effects(&mut game_params);

    let mut possible_commands = Vec::new();
    possible_commands.push(attack::AttackType::AA);
    possible_commands.push(attack::AttackType::Q);
    possible_commands.push(attack::AttackType::W);
    possible_commands.push(attack::AttackType::E);
    possible_commands.push(attack::AttackType::R);

    // let perms = possible_commands
    //     .into_iter()
    //     .combinations_with_replacement(1);

    // let size: usize = perms.size_hint().1.unwrap();
    let mut best_builds: Vec<Build> = Vec::new();
    // let mut best_build: Option<Build> = Option::None;

    test_next_possibilities(
        &possible_commands,
        &VecDeque::new(),
        &game_params,
        &mut best_builds,
    );

    // perms.par_bridge().for_each(|selected_item_ids| {
    // perms.for_each(|selected_commands| {
    //     println!("selected_commands: {:#?}", selected_commands);

    //     let (damage, damage_history, time_ms, kill) =
    //         simulation::run(selected_commands.clone().into(), &game_params);
    //     println!("damage: {:#?}", damage);
    //     println!("time_ms: {:#?}", time_ms);
    //     println!("DPS: {:#?}", damage * (1000_f64 / time_ms as f64));
    //     // println!("damage_history: {:#?}", damage_history);

    //     let build = Build {
    //         damage: damage.clone(),
    //         item_ids: selected_items
    //             .iter()
    //             .map(|item_data| item_data.id)
    //             .collect_vec()
    //             .clone(),
    //         dps: damage.clone() * (1000_f64 / time_ms as f64),
    //         selected_commands,
    //         time_ms,
    //         kill,
    //     };

    //     best_builds.push(build);
    // });

    // let mut builds = best_builds.into_iter().collect_vec();
    // builds.retain(|build| build.kill);

    let results = sort_best_builds(static_data, best_builds);
    let mut filtered_results = results.clone();
    let best_build = results.get(0);
    if best_build.is_some() {
        filtered_results.retain(|result| result.time_ms == best_build.unwrap().time_ms);
        println!("Top results: {:#?}", filtered_results);
    }

    // println!("Best build: {:#?}", best_build);

    let global_elapsed = global_start.elapsed();
    println!("Elapsed: {:.2?}", global_elapsed);
}

fn test_next_possibilities(
    possible_commands: &Vec<attack::AttackType>,
    commands_so_far: &VecDeque<attack::AttackType>,
    game_params: &GameParams<'_>,
    best_builds: &mut Vec<Build>,
) {
    for next_command in possible_commands.iter() {
        let mut selected_commands: VecDeque<attack::AttackType> = commands_so_far.clone();
        selected_commands.push_back(next_command.clone());

        println!("running: {:#?}", selected_commands);
        let (damage, damage_history, _, time_ms, kill) =
            simulation::run(selected_commands.clone(), &game_params);
        println!(
            "damage: {:#?}. time_ms: {:#?}. kill: {:#?}",
            damage, time_ms, kill
        );

        if kill || time_ms > 100_000 {
            // if is_better_build(best_build, damage, time_ms) {
            let new_build = Build {
                damage: damage.clone(),
                item_ids: Vec::new(),
                dps: damage.clone() * (1000_f64 / time_ms as f64),
                selected_commands: selected_commands.into(),
                time_ms,
                kill,
            };

            best_builds.push(new_build);
            // }

            return;
        } else {
            test_next_possibilities(
                possible_commands,
                &selected_commands,
                game_params,
                best_builds,
            );
        }
    }
}

fn is_better_build(best_build: &Option<Build>, damage: f64, time_ms: u64) -> bool {
    if best_build.is_none() {
        return true;
    }

    let option = best_build.as_ref();

    let build = option.unwrap();

    time_ms < build.time_ms || (time_ms == build.time_ms && damage > build.damage)
}
