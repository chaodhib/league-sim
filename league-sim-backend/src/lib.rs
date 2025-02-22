mod utils;

use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    ops::{Add, Mul},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    u64, // time::Instant,
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

#[derive(Debug, Clone, serde::Deserialize)]
struct SimulationInputData {
    mode: String,
    #[serde(rename(deserialize = "abilitySequence"))]
    ability_sequence: Vec<String>,
    champion: ChampionInputData,
    config: HashMap<String, String>,
    game: GameInputData,
    items: ItemInputData,
    // runes: Vec<RuneInputData>,
    #[serde(rename(deserialize = "selectedItemIds"))]
    selected_item_ids: Vec<u64>,
    target: TargetInputData,
    general: GeneralInputData,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ChampionInputData {
    level: u64,
    #[serde(rename(deserialize = "healthPercentage"))]
    health_percentage: f64,
    #[serde(rename(deserialize = "unseenThreatBuff"))]
    unseen_threat_buff: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GameInputData {
    #[serde(rename(deserialize = "critHandling"))]
    crit_handling: String,
    #[serde(rename(deserialize = "gameTime"))]
    game_time: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ItemInputData {
    #[serde(rename(deserialize = "maxGold"))]
    max_gold: Option<u64>,
    #[serde(rename(deserialize = "numItems"))]
    num_items: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TargetInputData {
    armor: u64,
    #[serde(rename(deserialize = "maxHealth"))]
    max_health: u64,
    #[serde(rename(deserialize = "currentHealth"))]
    current_health: u64,
    #[serde(rename(deserialize = "magicResistance"))]
    magic_resistance: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]

struct GeneralInputData {
    #[serde(rename(deserialize = "topResultNumber"))]
    pub top_result_number: u64,
    #[serde(rename(deserialize = "sortCriteria"))]
    pub sort_criteria: String,
}

#[derive(Debug, Clone)]
struct Build {
    damage: f64,
    dps: f64,
    item_ids: Vec<u64>,
    time_ms: u64,
    selected_commands: Vec<attack::AttackType>,
    kill: bool,
    damage_history: Vec<simulation::DamageInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TopResult {
    damage: f64,
    dps: f64,
    item_names: Vec<String>,
    cost: u64,
    time_ms: u64,
    selected_commands: Vec<attack::AttackType>,
    kill: bool,
    damage_history: Vec<simulation::DamageInfo>,
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, league-sim!");
}

#[wasm_bindgen]
pub fn init() {
    set_panic_hook();
}

#[wasm_bindgen]
pub fn execute_simulation(js_val: JsValue) -> Result<JsValue, JsValue> {
    // log(format!("execute_simulation: {:#?}", js_val).as_str());

    let deserializer = serde_wasm_bindgen::Deserializer::from(js_val);
    let result: Result<SimulationInputData, _> = serde_path_to_error::deserialize(deserializer);
    match result {
        Ok(simulation_input_data) => {
            log(format!("execute_simulation: {:#?}", simulation_input_data).as_str());

            let mut runes: HashSet<Rune> = HashSet::new();
            runes.insert(Rune::DarkHarvest);
            runes.insert(Rune::SuddenImpact);
            runes.insert(Rune::AbsoluteFocus);
            runes.insert(Rune::GatheringStorm);
            runes.insert(Rune::AdaptiveForce1);
            runes.insert(Rune::AdaptiveForce2);
            // runes.insert(Rune::AbilityHaste);

            let results: Vec<TopResult>;

            match simulation_input_data.mode.as_str() {
                "items" => {
                    results = optimize_items(simulation_input_data, runes);
                }
                // "single" => {
                //     run_single(
                //         simulation_input_data.config,
                //         simulation_input_data.selected_item_ids,
                //         runes,
                //     );
                // }
                // "combo" => {
                //     run_ttk(config, item_ids, runes);
                // }
                _ => {
                    panic!("Unknown mode: {:#?}", simulation_input_data.mode);
                }
            }

            // let map: HashMap<String, String> = HashMap::new();
            Ok(serde_wasm_bindgen::to_value(&results)?)
        }
        Err(err) => {
            let path = err.path().to_string();
            panic!("Parsing error at path: {}", path);
        }
    }

    // log(format!("execute_simulation: {:#?}", simulation_input_data).as_str());
}

fn optimize_items(input: SimulationInputData, runes: HashSet<Rune>) -> Vec<TopResult> {
    let mut selected_commands = VecDeque::new();
    input.ability_sequence.iter().for_each(|ability| {
        selected_commands.push_back(attack::AttackType::from_str(ability));
    });

    let target_stats: TargetStats = TargetStats {
        armor: input.target.armor as f64,
        max_health: input.target.max_health as f64,
        current_health: input.target.current_health as f64,
        magic_resistance: input.target.magic_resistance as f64,
    };

    let static_data =
        data_input::parse_files(Champion::Khazix, &input.selected_item_ids, &input.config);

    let perms = input
        .selected_item_ids
        .into_iter()
        .combinations(input.items.num_items as usize);
    let progress = Arc::new(AtomicUsize::new(0));
    let size: usize = perms.size_hint().1.unwrap();
    let best_builds: ArrayQueue<Build> = ArrayQueue::new(size);

    // perms.par_bridge().for_each(|selected_item_ids| {
    perms.for_each(|selected_item_ids| {
        let mut selected_items: Vec<&ItemData> = Vec::new();
        for selected_item_id in selected_item_ids.iter() {
            let new_item = static_data.items_map.get(selected_item_id).unwrap();
            selected_items.push(new_item);
        }

        if has_item_group_duplicates(&selected_items)
            || input
                .items
                .max_gold
                .is_some_and(|gold_cap| above_gold_cap(&selected_items, &gold_cap))
        {
            return;
        }

        let mut initial_attacker_auras: Vec<AuraApplication> = Vec::new();

        if input.champion.unseen_threat_buff {
            initial_attacker_auras.push(AuraApplication {
                aura: Aura::UnseenThreat,
                stacks: None,
                start_ms: 0,
                end_ms: None,
            });
        }

        let crit_handling = match input.game.crit_handling.as_str() {
            "average" => CritHandlingChoice::Avg,
            "never" => CritHandlingChoice::Min,
            "always" => CritHandlingChoice::Max,
            &_ => panic!(),
        };

        let mut game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: input.champion.level,
            items: &selected_items,
            initial_config: &input.config,
            abilities: &static_data.abilities,
            initial_target_stats: &target_stats,
            runes: &runes,
            attacker_hp_perc: input.champion.health_percentage,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling,
            initial_attacker_auras: &initial_attacker_auras,
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: input.game.game_time * 60 * 1000,
        };

        compile_passive_effects(&mut game_params);

        let (damage, damage_history, time_ms, kill) =
            simulation::run(selected_commands.clone(), &game_params);

        let build = Build {
            damage,
            item_ids: selected_item_ids.clone(),
            dps: damage * (1000_f64 / time_ms as f64),
            selected_commands: selected_commands.clone().into(),
            time_ms,
            kill,
            damage_history,
        };

        let push_result = best_builds.push(build);
        if push_result.is_err() {
            panic!();
        }

        let current_progress = progress.fetch_add(1, Ordering::Relaxed);
        log(format!(
            "Progress: {:#?}%",
            (current_progress as f64 / size as f64 * 100.0) as u32
        )
        .as_str());
    });

    let results: Vec<TopResult> = sort_best_builds(
        static_data,
        best_builds.into_iter().collect_vec(),
        input.general.top_result_number as usize,
        input.general.sort_criteria,
    );

    results
}

fn sort_best_builds(
    static_data: data_input::StaticData,
    best_builds: Vec<Build>,
    top_result_number: usize,
    sort_criteria: String,
) -> Vec<TopResult> {
    let compare_dps = |a: &Build, b: &Build| {
        let dps_ord = b.dps.partial_cmp(&a.dps).unwrap();
        if dps_ord == std::cmp::Ordering::Equal {
            a.time_ms.partial_cmp(&b.time_ms).unwrap()
        } else {
            dps_ord
        }
    };
    let compare_damage = |a: &Build, b: &Build| {
        let damage_ord = b.damage.partial_cmp(&a.damage).unwrap();
        if damage_ord == std::cmp::Ordering::Equal {
            a.time_ms.partial_cmp(&b.time_ms).unwrap()
        } else {
            damage_ord
        }
    };
    let compare_time = |a: &Build, b: &Build| {
        let time_ord = a.time_ms.partial_cmp(&b.time_ms).unwrap();
        if time_ord == std::cmp::Ordering::Equal {
            b.damage.partial_cmp(&a.damage).unwrap()
        } else {
            time_ord
        }
    };

    let cmp_fct = match sort_criteria.as_str() {
        "dps_desc" => compare_dps,
        "damage_desc" => compare_damage,
        "time_asc" => compare_time,
        &_ => panic!(),
    };

    let results = best_builds
        .into_iter()
        .sorted_by(cmp_fct)
        .take(top_result_number)
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
                kill: build.kill,
                damage_history: build.damage_history,
            }
        })
        .collect_vec();
    results
}
