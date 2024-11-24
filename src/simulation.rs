use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, VecDeque},
    fmt,
};

use crate::{
    attack::{cast_time, simulate_spell},
    data_input::common::{compute_source_champion_stats, GameParams, OffensiveStats},
    Damage, SpellResult,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EventCategory {
    AttackCastStart,
    AttackCastEnd,
    // An aura is either a buff or a debuff
    AuraUpdateAttacker,
    AuraUpdateTarget,
    // cooldown
    CooldownEnded,
    // WAIT,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
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

struct State<'a> {
    damage: &'a mut Damage,
    time_ms: u64,
    cooldowns: &'a mut HashMap<AttackType, u64>,
    last_attack_time_ms: u64,
}

pub fn run(mut selected_commands: VecDeque<AttackType>, game_params: &GameParams) -> (Damage, u64) {
    // use a priority queue to manage the events
    let mut events: BinaryHeap<Event> = BinaryHeap::new();

    let mut state: State = State {
        damage: &mut Damage {
            min: 0.0,
            max: 0.0,
            avg: 0.0,
        },
        time_ms: 0,
        cooldowns: &mut HashMap::new(),
        last_attack_time_ms: 0,
    };

    // add first attack event
    insert_next_attack_event(&mut events, &mut selected_commands, &mut state, 0);

    // and launch
    return execute_commands(&mut events, &mut selected_commands, &mut state, game_params);
}

fn execute_commands(
    events: &mut BinaryHeap<Event>,
    remaining_commands: &mut VecDeque<AttackType>,
    state: &mut State,
    game_params: &GameParams,
) -> (Damage, u64) {
    loop {
        match events.pop() {
            None => return (state.damage.clone(), state.last_attack_time_ms),
            Some(next_event) => {
                on_event(&next_event, events, remaining_commands, game_params, state)
            }
        }
    }
}

fn on_event(
    event: &Event,
    events: &mut BinaryHeap<Event>,
    remaining_commands: &mut VecDeque<AttackType>,
    game_params: &GameParams,
    state: &mut State,
) {
    println!("on_event: {:#?}", event);
    // advance time
    state.time_ms = event.time_ms;

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
                game_params.champion_stats,
                &off_stats,
                game_params.level,
                game_params.def_stats,
                event.attack_type,
                game_params.config,
                game_params.abilities,
            );

            // println!("spell_result: {:#?}", spell_result);
            on_damage_event(&spell_result.damage, state, event.time_ms);

            if spell_result.cooldown.is_some() {
                let cooldown_end_ms = spell_result.cooldown.unwrap() + event.time_ms;
                insert_cooldown_ended_event(events, event, cooldown_end_ms);
                add_cooldown_to_state(state, event.attack_type, cooldown_end_ms);
            }
            insert_next_attack_event(events, remaining_commands, state, event.time_ms);
        }
        EventCategory::AuraUpdateAttacker => todo!(),
        EventCategory::AuraUpdateTarget => todo!(),
        EventCategory::CooldownEnded => on_cooldown_ended(event),
    }
}

fn add_cooldown_to_state(state: &mut State<'_>, attack_type: AttackType, cooldown_end_ms: u64) {
    state.cooldowns.insert(attack_type, cooldown_end_ms);
}

fn on_damage_event(damage: &Damage, state: &mut State, time_ms: u64) {
    state.damage.add(damage);
    state.last_attack_time_ms = time_ms;
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
    state: &mut State,
    current_time_ms: u64,
) {
    let command = commands.pop_front();
    if command.is_some() {
        let attack_type = command.unwrap();
        let time_ms: u64 = if state.cooldowns.contains_key(&attack_type) {
            // the ability is in cooldown. We can't queue it right away.
            state.cooldowns.remove(&attack_type).unwrap()
        } else {
            // ability is off CD. Let's start it right away
            current_time_ms
        };

        let event = Event {
            attack_type,
            category: EventCategory::AttackCastStart,
            time_ms,
        };

        events.push(event);
    }
}

fn insert_cooldown_ended_event(events: &mut BinaryHeap<Event>, event: &Event, time_ms: u64) {
    let event = Event {
        attack_type: event.attack_type,
        category: EventCategory::CooldownEnded,
        time_ms,
    };

    events.push(event);
}

fn on_cooldown_ended(event: &Event) {
    println!(
        "cooldown ended for {:#?} at {:#?}",
        event.attack_type, event.time_ms
    );
}
