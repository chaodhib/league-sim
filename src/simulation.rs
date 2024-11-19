use std::{
    cmp::Ordering,
    collections::{BinaryHeap, VecDeque},
    fmt,
};

use crate::{
    attack::{cast_time, cooldown, simulate_spell},
    data_input::common::{compute_source_champion_stats, GameParams, OffensiveStats},
    SpellResult,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EventCategory {
    AttackCastStart,
    AttackCastEnd,
    // An aura is either a buff or a debuff
    AuraUpdateAttacker,
    AuraUpdateTarget,
    // WAIT,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AttackType {
    AA,
    Q,
    W,
    E,
    R,
    // add item active?
}

impl fmt::Display for AttackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Event {
    time_ms: u64,
    category: EventCategory,
    attack_type: AttackType,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time_ms.cmp(&self.time_ms)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn run(mut selected_commands: VecDeque<AttackType>, game_params: &GameParams) {
    // use a priority queue to manage the events
    let mut events: BinaryHeap<Event> = BinaryHeap::new();

    // add first attack event
    insert_next_attack_event(&mut events, &mut selected_commands, 0);

    // and launch
    execute_commands(&mut events, &mut selected_commands, game_params);
}

fn execute_commands(
    events: &mut BinaryHeap<Event>,
    remaining_commands: &mut VecDeque<AttackType>,
    game_params: &GameParams,
) {
    loop {
        match events.pop() {
            None => return,
            Some(next_event) => on_event(&next_event, events, remaining_commands, game_params),
        }
    }
}

fn on_event(
    event: &Event,
    events: &mut BinaryHeap<Event>,
    remaining_commands: &mut VecDeque<AttackType>,
    game_params: &GameParams,
) {
    println!("on_event: {:#?}", event);
    match event.category {
        EventCategory::AttackCastStart => {
            let off_stats: OffensiveStats = compute_source_champion_stats(
                game_params.champion_stats,
                game_params.level,
                game_params.items,
            );

            // let cooldown = cooldown(
            //     game_params.champion_stats,
            //     &off_stats,
            //     event.attack_type,
            //     game_params.config,
            //     game_params.abilities,
            // );

            let cast_time = cast_time(
                game_params.champion_stats,
                &off_stats,
                event.attack_type,
                game_params.config,
                game_params.abilities,
            );
            // println!("cooldown: {:#?}", cooldown);
            // println!("cast_time: {:#?}", cast_time);

            insert_attack_cast_end_event(event, events, event.time_ms + cast_time);
        }
        EventCategory::AttackCastEnd => {
            let off_stats: OffensiveStats = compute_source_champion_stats(
                game_params.champion_stats,
                game_params.level,
                game_params.items,
            );

            let spell_result: SpellResult = simulate_spell(
                &off_stats,
                game_params.level,
                game_params.def_stats,
                event.attack_type,
                game_params.config,
                game_params.abilities,
            );
            println!("spell_result: {:#?}", spell_result);

            insert_next_attack_event(events, remaining_commands, event.time_ms);
        }
        EventCategory::AuraUpdateAttacker => todo!(),
        EventCategory::AuraUpdateTarget => todo!(),
    }
}

fn insert_attack_cast_end_event(
    attack_cast_start_event: &Event,
    events: &mut BinaryHeap<Event>,
    time_ms: u64,
) {
    let event = Event {
        attack_type: attack_cast_start_event.attack_type,
        category: EventCategory::AttackCastEnd,
        time_ms,
    };

    events.push(event);
}

fn insert_next_attack_event(
    events: &mut BinaryHeap<Event>,
    commands: &mut VecDeque<AttackType>,
    time_ms: u64,
) {
    let command = commands.pop_front();
    if command.is_some() {
        let event = Event {
            attack_type: command.unwrap(),
            category: EventCategory::AttackCastStart,
            time_ms,
        };

        events.push(event);
    }
}
