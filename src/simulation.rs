use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    fmt,
};

use itertools::Itertools;

use crate::{
    attack::{cast_time, simulate_spell, AttackType, SpellCategory, SpellResult},
    data_input::{
        abilities::{find_ability, SpellData},
        common::{
            compile_passive_effects, compute_attacker_stats, AttackerStats, Aura, Champion,
            GameParams, PassiveEffect, TargetStats,
        },
        items::Item,
        runes::Rune,
    },
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EventCategory {
    AttackCastStart,
    AttackCastEnd,
    // An aura is either a buff or a debuff
    AuraAttackerStart,
    AuraAttackerEnd,
    // AuraUpdateTarget,
    CooldownEnded,
    PassiveTriggered,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Event {
    pub time_ms: u64,
    pub category: EventCategory,
    pub attack_type: Option<AttackType>,
    pub passive_effect: Option<PassiveEffect>,
    pub aura: Option<Aura>,
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
#[derive(Clone, Debug)]
enum DamageSource {
    Ability,
    // Passive,
    Rune,
    Item,
}

#[derive(Clone, Debug)]
pub struct Damage {
    amount: f64,
    time_ms: u64,
    source: DamageSource,
    source_ability: Option<AttackType>,
    source_rune: Option<Rune>,
    source_item: Option<String>,
}

pub struct State<'a> {
    pub total_damage: f64,
    pub damage_history: &'a mut Vec<Damage>,
    pub time_ms: u64,
    pub cooldowns: &'a mut HashMap<AttackType, u64>,
    pub effects_cooldowns: &'a mut HashMap<PassiveEffect, u64>,
    pub last_attack_time_ms: u64,
    pub config: &'a mut HashMap<String, String>,
    pub attacker_auras: &'a mut HashMap<Aura, u64>,
    pub target_auras: &'a mut HashMap<Aura, u64>,
    pub recast_charges: &'a mut Vec<AttackType>,
    pub recast_ready: &'a mut HashSet<AttackType>,
}

impl State<'_> {
    // fn refresh_cds_and_auras(state: &mut State<'_>) {
    fn refresh_cds_and_auras(
        &mut self,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        let current_time = self.time_ms;
        println!("refresh_cds_and_auras. Current time: {:#?}", current_time);

        self.effects_cooldowns
            .retain(|_, end_at| *end_at > current_time);

        self.cooldowns.retain(|_, end_at| *end_at > current_time);

        // self.attacker_auras needs to be emptied in a way
        // that callbacks are called repeadly as long as at least one aura
        // has been removed
        while self
            .attacker_auras
            .values()
            .any(|end_at| *end_at <= current_time)
        {
            self.attacker_auras
                .clone()
                .iter()
                .filter(|(_, end_at)| **end_at <= current_time)
                .for_each(|(aura, end_at)| {
                    println!("expired aura: {:#?} {:#?}", aura, end_at);
                    self.on_expire_attacker_aura(aura, game_params, event, events)
                });

            // self.attacker_auras
            // .retain(|_, end_at| *end_at > current_time);
        }
    }

    pub fn add_attacker_aura(
        &mut self,
        aura: Aura,
        duration: u64,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        let end_time = if duration == u64::MAX {
            u64::MAX
        } else {
            self.time_ms + duration
        };

        insert_aura_attacker_start_event(events, self.time_ms, aura.clone());
        insert_aura_attacker_end_event(events, end_time, aura.clone());
        aura.on_start(self, game_params, event, events);
        self.attacker_auras.insert(aura, end_time);
    }

    pub fn end_early_attacker_aura(
        &mut self,
        aura: &Aura,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        // remove the default 'aura_attacker_end' event (added in add_attacker_aura)
        events.retain(|event| {
            event.category != EventCategory::AuraAttackerEnd || event.aura != Some(aura.clone())
        });

        // then proceed
        insert_aura_attacker_end_event(events, self.time_ms, aura.clone());
        aura.on_end(self, game_params, event, events);
        self.attacker_auras.remove(aura);
    }

    pub fn on_expire_attacker_aura(
        &mut self,
        aura: &Aura,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        aura.on_end(self, game_params, event, events);
        self.attacker_auras.remove(aura);
    }
}

pub fn run(
    mut selected_commands: VecDeque<AttackType>,
    game_params: &GameParams,
) -> (f64, Vec<Damage>, u64) {
    // use a priority queue to manage the events
    let mut events: BinaryHeap<Event> = BinaryHeap::new();

    let mut state: State = State {
        total_damage: 0.0,
        time_ms: 0,
        cooldowns: &mut HashMap::new(),
        last_attack_time_ms: 0,
        effects_cooldowns: &mut HashMap::new(),
        config: &mut HashMap::new(),
        attacker_auras: &mut HashMap::new(),
        target_auras: &mut HashMap::new(),
        damage_history: &mut Vec::new(),
        recast_charges: &mut Vec::new(),
        recast_ready: &mut HashSet::new(),
    };

    // add first attack event
    insert_next_attack_event(
        &mut events,
        &mut selected_commands,
        &mut state,
        0,
        game_params,
    );

    add_initial_auras(game_params, &mut state);

    // and launch
    return execute_commands(&mut events, &mut selected_commands, &mut state, game_params);
}

fn add_initial_auras(game_params: &GameParams<'_>, state: &mut State<'_>) {
    for aura in game_params.initial_attacker_auras.iter() {
        state.attacker_auras.insert(aura.clone(), 5_000);
    }

    for aura in game_params.initial_target_auras.iter() {
        state.target_auras.insert(aura.clone(), 5_000);
    }
}

fn execute_commands(
    events: &mut BinaryHeap<Event>,
    remaining_commands: &mut VecDeque<AttackType>,
    state: &mut State,
    game_params: &GameParams,
) -> (f64, Vec<Damage>, u64) {
    loop {
        match events.pop() {
            None => {
                return (
                    state.total_damage.clone(),
                    state.damage_history.clone(),
                    state.last_attack_time_ms,
                )
            }
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
    if event.time_ms == u64::MAX {
        return;
    }

    println!("on_event: {:#?}", event);
    // advance time
    state.time_ms = event.time_ms;
    state.refresh_cds_and_auras(game_params, event, events);

    match event.category {
        EventCategory::AttackCastStart => {
            check_spell_off_cooldown(event, events, game_params, state);
            trigger_stealth_exit_if_applicable(event, events, game_params, state);
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
            handle_dash_if_applicable(event, events, game_params, state);
            trigger_stealth_exit_if_applicable(event, events, game_params, state);

            let attacker_stats: AttackerStats = compute_attacker_stats(game_params, state);

            let spell_result: SpellResult = simulate_spell(
                &attacker_stats,
                game_params,
                state,
                event.attack_type.unwrap(),
                event,
                events,
            );

            // println!("spell_result: {:#?}", spell_result);
            if spell_result.damage.is_some() {
                on_damage_from_ability(
                    &spell_result.damage.unwrap(),
                    state,
                    event.time_ms,
                    event.attack_type.unwrap(),
                );
                on_post_damage_events(
                    spell_result.damage.unwrap(),
                    &attacker_stats,
                    state,
                    game_params,
                    event,
                    events,
                );
            }

            if spell_result.cooldown.is_some() {
                let cooldown_end_ms = spell_result.cooldown.unwrap() + event.time_ms;
                insert_cooldown_ended_event(events, event, cooldown_end_ms);
                add_cooldown_to_state(state, event.attack_type.unwrap(), cooldown_end_ms);
            }
            insert_next_attack_event(
                events,
                remaining_commands,
                state,
                event.time_ms,
                game_params,
            );
        }
        // EventCategory::CooldownEnded => on_cooldown_ended(event),
        // EventCategory::PassiveTriggered => on_passive_triggered(event),
        EventCategory::CooldownEnded => (),
        EventCategory::PassiveTriggered => (),
        EventCategory::AuraAttackerStart => (),
        EventCategory::AuraAttackerEnd => (),
    }
}

fn check_spell_off_cooldown(
    event: &Event,
    events: &mut BinaryHeap<Event>,
    game_params: &GameParams<'_>,
    state: &mut State<'_>,
) {
    let attack_type = event.attack_type.unwrap();
    if state.cooldowns.contains_key(&attack_type) && !state.recast_ready.contains(&attack_type) {
        println!("state.cooldowns: {:#?}", state.cooldowns);
        println!("state.recast_ready: {:#?}", state.recast_ready);
        panic!();
    }
}

fn handle_dash_if_applicable(
    event: &Event,
    events: &mut BinaryHeap<Event>,
    game_params: &GameParams,
    state: &mut State,
) {
    let attack_type = event.attack_type.unwrap();
    let mut ability: Option<&SpellData> = None;
    if attack_type != AttackType::AA {
        ability = Some(find_ability(
            game_params.abilities,
            attack_type,
            game_params.initial_config,
        ));
    }

    if ability.is_some()
        && ability
            .unwrap()
            .category
            .as_ref()
            .is_some_and(|cat| cat == &SpellCategory::Dash)
    {
        // println!("handle_dash_if_applicable dash: {:#?}", attack_type);

        for effect in game_params.passive_effects.iter() {
            effect.handle_dash_event(event, events, game_params, state)
        }

        // for effect in state
        //     .attacker_auras
        //     .clone()
        //     .iter()
        //     .flat_map(|aura| aura.0.passive_effects())
        // {
        //     effect.handle_dash_event(event, events, game_params, state)
        // }
    }
}

fn trigger_stealth_exit_if_applicable(
    event: &Event,
    events: &mut BinaryHeap<Event>,
    game_params: &GameParams,
    state: &mut State,
) {
    if !state
        .attacker_auras
        .get(&Aura::Invisibility)
        .is_some_and(|&time_end| state.time_ms < time_end)
    {
        return;
    }

    // kha'zix is an exception in the sense that its stealth is broken only at the end
    // of the AA windup. Regarding abilities, it's the same as other champs
    if game_params.champion == Champion::Khazix {
        if event
            .attack_type
            .is_some_and(|attack_type| attack_type == AttackType::AA)
            && event.category == EventCategory::AttackCastEnd
        {
            trigger_stealth_exit(event, events, game_params, state);
        } else if event
            .attack_type
            .is_some_and(|attack_type| attack_type != AttackType::AA)
            && event.category == EventCategory::AttackCastStart
        {
            trigger_stealth_exit(event, events, game_params, state);
        }
    } else {
        if event.category == EventCategory::AttackCastStart {
            trigger_stealth_exit(event, events, game_params, state);
        }
    }
}

fn trigger_stealth_exit(
    event: &Event,
    events: &mut BinaryHeap<Event>,
    game_params: &GameParams<'_>,
    state: &mut State<'_>,
) {
    state.end_early_attacker_aura(&Aura::Invisibility, game_params, event, events);

    for effect in game_params.passive_effects.iter() {
        effect.handle_stealth_exit_event(event, events, game_params, state)
    }

    // for effect in state
    //     .attacker_auras
    //     .clone()
    //     .iter()
    //     .flat_map(|aura| aura.0.passive_effects())
    // {
    //     effect.handle_stealth_exit_event(event, events, game_params, state)
    // }
}

fn add_cooldown_to_state(state: &mut State<'_>, attack_type: AttackType, cooldown_end_ms: u64) {
    state.cooldowns.insert(attack_type, cooldown_end_ms);
}

pub fn on_damage_from_ability(
    damage: &f64,
    state: &mut State,
    time_ms: u64,
    attack_type: AttackType,
) {
    state.total_damage += damage;
    state.damage_history.push(Damage {
        amount: *damage,
        time_ms: time_ms,
        source: DamageSource::Ability,
        source_ability: Some(attack_type),
        source_rune: None,
        source_item: None,
    });
    state.last_attack_time_ms = time_ms;
}

pub fn on_damage_from_rune(damage: &f64, state: &mut State, event: &Event, rune: Rune) {
    state.total_damage += damage;
    state.damage_history.push(Damage {
        amount: *damage,
        time_ms: event.time_ms,
        source: DamageSource::Rune,
        source_ability: None,
        source_rune: Some(rune),
        source_item: None,
    });
    state.last_attack_time_ms = event.time_ms;
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
        aura: None,
    };

    events.push(event);
}

fn insert_next_attack_event(
    events: &mut BinaryHeap<Event>,
    commands: &mut VecDeque<AttackType>,
    state: &mut State,
    current_time_ms: u64,
    game_params: &GameParams,
) {
    let command = commands.pop_front();
    if command.is_some() {
        let attack_type = command.unwrap();

        let time_ms: u64 = if state.cooldowns.contains_key(&attack_type) {
            if state.recast_charges.contains(&attack_type) {
                let ability = find_ability(
                    game_params.abilities,
                    attack_type,
                    game_params.initial_config,
                );
                if let Some(invis_end) = state.attacker_auras.get(&Aura::Invisibility) {
                    invis_end + ability.recast_gap_duration.unwrap()
                } else {
                    if let Some(delay_end) = state.attacker_auras.get(&Aura::VoidAssaultDelay) {
                        *delay_end
                    } else {
                        if let Some(_) = state.attacker_auras.get(&Aura::VoidAssaultRecastReady) {
                            0
                        } else {
                            // the ability is in cooldown. We can't queue it right away.
                            state.cooldowns.remove(&attack_type).unwrap()
                        }
                    }
                }
            } else {
                // the ability is in cooldown. We can't queue it right away.
                state.cooldowns.remove(&attack_type).unwrap()
            }
        } else {
            // ability is off CD. Let's start it right away
            current_time_ms
        };

        let event = Event {
            attack_type: Some(attack_type),
            category: EventCategory::AttackCastStart,
            time_ms,
            passive_effect: None,
            aura: None,
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
        aura: None,
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
        aura: None,
    };

    events.push(event);
}

pub fn insert_aura_attacker_start_event(events: &mut BinaryHeap<Event>, time_ms: u64, aura: Aura) {
    let event = Event {
        attack_type: None,
        category: EventCategory::AuraAttackerStart,
        time_ms,
        passive_effect: None,
        aura: Some(aura),
    };

    events.push(event);
}

pub fn insert_aura_attacker_end_event(events: &mut BinaryHeap<Event>, time_ms: u64, aura: Aura) {
    let event = Event {
        attack_type: None,
        category: EventCategory::AuraAttackerEnd,
        time_ms,
        passive_effect: None,
        aura: Some(aura),
    };

    events.push(event);
}

// fn on_cooldown_ended(event: &Event) {
//     println!(
//         "cooldown ended for {:#?} at {:#?}",
//         event.attack_type, event.time_ms
//     );
// }

// fn on_passive_triggered(event: &Event) {
//     println!(
//         "passive triggered for {:#?} at {:#?}",
//         event.passive_effect.unwrap(),
//         event.time_ms
//     );
// }

fn on_post_damage_events(
    damage: f64,
    attacker_stats: &AttackerStats,
    state: &mut State,
    game_params: &GameParams,
    event: &Event,
    events: &mut BinaryHeap<Event>,
) {
    // println!("on_post_damage_events");
    // println!("passive_effects:");
    for effect in game_params.passive_effects.iter() {
        // println!("{:#?}", effect);
        effect.handle_on_post_damage(damage, attacker_stats, state, game_params, event, events);
    }

    // println!("aura effects:");
    for (aura, _) in state.attacker_auras.clone().iter() {
        // println!("{:#?}", aura);
        aura.on_post_damage(damage, attacker_stats, state, game_params, event, events);
    }
    // println!("on_post_damage_events-----------------------");
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
            aura: None,
        };

        let event_2 = Event {
            attack_type: Some(super::AttackType::W),
            category: super::EventCategory::AttackCastStart,
            time_ms: 1000,
            passive_effect: None,
            aura: None,
        };

        let event_3 = Event {
            attack_type: Some(super::AttackType::E),
            category: super::EventCategory::AttackCastStart,
            time_ms: 1000,
            passive_effect: None,
            aura: None,
        };

        // let event_0 = Event {
        //     attack_type: Some(super::AttackType::Q),
        //     category: super::EventCategory::AttackCastStart,
        //     time_ms: 0,
        //     passive_effect: None,
        //     aura: None,
        // };

        // events.push(&event_1);
        // events.push(&event_2);
        // events.push(&event_3);
        // events.push(&event_0);

        events.push(&event_1);
        events.push(&event_2);
        events.push(&event_3);

        // assert_eq!(events.pop(), Some(&event_0));
        assert_eq!(events.pop(), Some(&event_1));
        assert_eq!(events.pop(), Some(&event_2));
        assert_eq!(events.pop(), Some(&event_3));
    }
}
