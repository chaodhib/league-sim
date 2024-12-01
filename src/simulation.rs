use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, VecDeque},
    fmt,
};

use crate::{
    attack::{cast_time, simulate_spell, AttackType, Damage, SpellCategory, SpellResult},
    data_input::{
        abilities::{find_ability, SpellData},
        common::{compute_attacker_stats, AttackerStats, GameParams, PassiveEffect},
    },
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EventCategory {
    AttackCastStart,
    AttackCastEnd,
    // An aura is either a buff or a debuff
    AuraUpdateAttacker,
    AuraUpdateTarget,
    CooldownEnded,
    PassiveTriggered,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Event {
    pub time_ms: u64,
    pub category: EventCategory,
    pub attack_type: Option<AttackType>,
    pub passive_effect: Option<PassiveEffect>,
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

pub struct State<'a> {
    pub total_damage: &'a mut Damage,
    pub time_ms: u64,
    pub cooldowns: &'a mut HashMap<AttackType, u64>,
    pub effects_cooldowns: &'a mut HashMap<PassiveEffect, u64>,
    pub last_attack_time_ms: u64,
    pub config: &'a mut HashMap<String, String>,
}

pub fn run(mut selected_commands: VecDeque<AttackType>, game_params: &GameParams) -> (Damage, u64) {
    // use a priority queue to manage the events
    let mut events: BinaryHeap<Event> = BinaryHeap::new();

    let mut state: State = State {
        total_damage: &mut Damage {
            min: 0.0,
            max: 0.0,
            avg: 0.0,
        },
        time_ms: 0,
        cooldowns: &mut HashMap::new(),
        last_attack_time_ms: 0,
        effects_cooldowns: &mut HashMap::new(),
        config: &mut HashMap::new(),
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
            None => return (state.total_damage.clone(), state.last_attack_time_ms),
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
            handle_stealth_exit_if_applicable(
                event.attack_type.unwrap(),
                game_params.initial_config,
                game_params.abilities,
            );
            let attacker_stats: AttackerStats = compute_attacker_stats(game_params, state);

            let cast_time = cast_time(
                &attacker_stats,
                event.attack_type.unwrap(),
                game_params.initial_config,
                game_params.abilities,
            );
            // println!("cooldown: {:#?}", cooldown);
            // println!("cast_time: {:#?}", cast_time);

            insert_attack_cast_end_event(event, events, event.time_ms + cast_time);
        }
        EventCategory::AttackCastEnd => {
            handle_dash_if_applicable(
                event.attack_type.unwrap(),
                game_params.initial_config,
                game_params.abilities,
            );
            handle_stealth_exit_if_applicable(
                event.attack_type.unwrap(),
                game_params.initial_config,
                game_params.abilities,
            );

            let attacker_stats: AttackerStats = compute_attacker_stats(game_params, state);

            let spell_result: SpellResult = simulate_spell(
                &attacker_stats,
                game_params.level,
                game_params.target_stats,
                event.attack_type.unwrap(),
                game_params.initial_config,
                game_params.abilities,
            );

            // println!("spell_result: {:#?}", spell_result);
            on_damage_event(&spell_result.damage, state, event.time_ms);
            on_post_damage_events(
                &spell_result.damage,
                &attacker_stats,
                state,
                game_params,
                event,
                events,
            );

            if spell_result.cooldown.is_some() {
                let cooldown_end_ms = spell_result.cooldown.unwrap() + event.time_ms;
                insert_cooldown_ended_event(events, event, cooldown_end_ms);
                add_cooldown_to_state(state, event.attack_type.unwrap(), cooldown_end_ms);
            }
            insert_next_attack_event(events, remaining_commands, state, event.time_ms);
        }
        EventCategory::AuraUpdateAttacker => todo!(),
        EventCategory::AuraUpdateTarget => todo!(),
        EventCategory::CooldownEnded => on_cooldown_ended(event),
        EventCategory::PassiveTriggered => on_passive_triggered(event),
    }
}

fn handle_dash_if_applicable(
    spell_name: AttackType,
    initial_config: &HashMap<String, String>,
    abilities: &Vec<SpellData>,
) {
    let mut ability: Option<&SpellData> = None;
    if spell_name != AttackType::AA {
        ability = Some(find_ability(abilities, spell_name, initial_config));
    }

    if ability.is_some()
        && ability
            .unwrap()
            .category
            .clone()
            .is_some_and(|cat| cat == SpellCategory::Dash)
    {
        // println!("handling dash: {:#?}", ability);
        // game_params.passive_effects.iter().for_each(|effect| {
        //     effect.handle_on_post_damage(damage, attacker_stats, state, game_params, event, events)
        // });
    }
}

fn handle_stealth_exit_if_applicable(
    spell_name: AttackType,
    initial_config: &HashMap<String, String>,
    abilities: &Vec<SpellData>,
) {
    let mut ability: Option<&SpellData> = None;
    if spell_name != AttackType::AA {
        ability = Some(find_ability(abilities, spell_name, initial_config));
    }
}

fn add_cooldown_to_state(state: &mut State<'_>, attack_type: AttackType, cooldown_end_ms: u64) {
    state.cooldowns.insert(attack_type, cooldown_end_ms);
}

fn on_damage_event(damage: &Damage, state: &mut State, time_ms: u64) {
    state.total_damage.add(damage);
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
        passive_effect: None,
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
            attack_type: Some(attack_type),
            category: EventCategory::AttackCastStart,
            time_ms,
            passive_effect: None,
        };

        events.push(event);
    }
}

fn insert_cooldown_ended_event(events: &mut BinaryHeap<Event>, event: &Event, time_ms: u64) {
    let event = Event {
        attack_type: event.attack_type,
        category: EventCategory::CooldownEnded,
        time_ms,
        passive_effect: None,
    };

    events.push(event);
}

pub fn insert_passive_triggered_event(
    events: &mut BinaryHeap<Event>,
    time_ms: u64,
    passive_effect: PassiveEffect,
) {
    let event = Event {
        attack_type: None,
        category: EventCategory::PassiveTriggered,
        time_ms,
        passive_effect: Some(passive_effect),
    };

    events.push(event);
}

fn on_cooldown_ended(event: &Event) {
    println!(
        "cooldown ended for {:#?} at {:#?}",
        event.attack_type, event.time_ms
    );
}

fn on_passive_triggered(event: &Event) {
    println!(
        "passive triggered for {:#?} at {:#?}",
        event.passive_effect.unwrap(),
        event.time_ms
    );
}

fn on_post_damage_events(
    damage: &Damage,
    attacker_stats: &AttackerStats,
    state: &mut State,
    game_params: &GameParams,
    event: &Event,
    events: &mut BinaryHeap<Event>,
) {
    println!("game_params: {:#?}", game_params.passive_effects);
    game_params.passive_effects.iter().for_each(|effect| {
        effect.handle_on_post_damage(damage, attacker_stats, state, game_params, event, events)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BinaryHeap;

    #[test]
    fn it_works() {
        let mut events: BinaryHeap<&Event> = BinaryHeap::new();

        let event_1 = Event {
            attack_type: Some(super::AttackType::Q),
            category: super::EventCategory::AttackCastStart,
            time_ms: 1000,
            passive_effect: None,
        };

        let event_2 = Event {
            attack_type: Some(super::AttackType::Q),
            category: super::EventCategory::AttackCastStart,
            time_ms: 1000,
            passive_effect: None,
        };

        let event_3 = Event {
            attack_type: Some(super::AttackType::Q),
            category: super::EventCategory::AttackCastStart,
            time_ms: 5000,
            passive_effect: None,
        };

        let event_0 = Event {
            attack_type: Some(super::AttackType::Q),
            category: super::EventCategory::AttackCastStart,
            time_ms: 0,
            passive_effect: None,
        };

        events.push(&event_1);
        events.push(&event_2);
        events.push(&event_3);
        events.push(&event_0);

        assert_eq!(events.pop(), Some(&event_0));
        assert_eq!(events.pop(), Some(&event_1));
        assert_eq!(events.pop(), Some(&event_2));
        assert_eq!(events.pop(), Some(&event_3));
    }
}
