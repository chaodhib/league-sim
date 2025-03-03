use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    fmt, u64,
};

use itertools::Itertools;

use crate::{
    attack::{cast_time, simulate_spell, AttackType, SpellCategory, SpellResult},
    data_input::{
        abilities::{find_ability, SpellData},
        common::{
            compile_passive_effects, compute_attacker_stats, compute_target_stats, AttackerStats,
            Aura, AuraApplication, Champion, DamageType, GameParams, PassiveEffect, TargetStats,
            Unit,
        },
        items::{Item, ItemData},
        runes::Rune,
    },
};

#[derive(Copy, Clone, Eq, PartialEq, Debug, serde::Serialize)]
pub enum EventCategory {
    AttackCastStart,
    AttackCastEnd,
    // An aura is either a buff or a debuff
    AuraAttackerStart,
    AuraAttackerEnd,
    AuraTargetStart,
    AuraTargetEnd,
    CooldownEnded,
    PassiveTriggered,
    TargetDied,
}

#[derive(Eq, PartialEq, Debug, Clone, serde::Serialize)]
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
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum DamageSource {
    Ability,
    Rune,
    ItemPassive,
    ItemActive,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct DamageInfo {
    pub amount: f64,
    pub damage_type: DamageType,
    pub time_ms: u64,
    pub source: DamageSource,
    pub source_ability: Option<AttackType>,
    pub source_rune: Option<Rune>,
    pub source_item: Option<Item>,
}

pub struct State<'a> {
    pub total_damage: f64,
    pub damage_history: &'a mut Vec<DamageInfo>,
    pub event_history: &'a mut Vec<Event>,
    pub time_ms: u64,
    pub cooldowns: &'a mut HashMap<AttackType, u64>,
    pub effects_cooldowns: &'a mut HashMap<PassiveEffect, u64>,
    pub last_attack_time_ms: u64,
    pub config: &'a mut HashMap<String, String>,
    pub attacker_auras: &'a mut HashMap<Aura, AuraApplication>,
    pub target_auras: &'a mut HashMap<Aura, AuraApplication>,
    pub recast_charges: &'a mut Vec<AttackType>,
    pub recast_ready: &'a mut HashSet<AttackType>,
    pub is_casting: bool,
}

impl State<'_> {
    fn refresh_cds_and_auras(
        &mut self,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        let current_time = self.time_ms;
        // println!("refresh_cds_and_auras. Current time: {:#?}", current_time);

        self.effects_cooldowns
            .retain(|_, end_at| *end_at > current_time);

        self.cooldowns.retain(|_, end_at| *end_at > current_time);

        // self.attacker_auras needs to be emptied in a way
        // that callbacks are called repeadly as long as at least one aura
        // has been removed
        while self
            .attacker_auras
            .values()
            .any(|aura_app| aura_app.end_ms.is_some_and(|x| x <= current_time))
        {
            self.attacker_auras
                .clone()
                .iter()
                .filter(|(_, aura_app)| aura_app.end_ms.is_some_and(|x| x <= current_time))
                .for_each(|(aura, _aura_app)| {
                    // println!("expired aura: {:#?} {:#?}", aura, end_at);
                    self.on_expire_attacker_aura(aura, game_params, event, events)
                });
        }

        while self
            .target_auras
            .values()
            .any(|aura_app| aura_app.end_ms.is_some_and(|x| x <= current_time))
        {
            self.target_auras
                .clone()
                .iter()
                .filter(|(_, aura_app)| aura_app.end_ms.is_some_and(|x| x <= current_time))
                .for_each(|(aura, _aura_app)| {
                    // println!("expired aura: {:#?} {:#?}", aura, end_at);
                    self.on_expire_target_aura(aura, game_params, event, events)
                });
        }
    }

    pub fn add_attacker_aura(
        &mut self,
        aura: Aura,
        duration: Option<u64>,
        stacks: Option<u64>,
        events: &mut BinaryHeap<Event>,
    ) {
        let end_time = if duration.is_none() {
            None
        } else {
            Some(self.time_ms + duration.unwrap())
        };

        insert_aura_attacker_start_event(events, self.time_ms, aura.clone());
        if end_time.is_some() {
            insert_aura_attacker_end_event(events, end_time.unwrap(), aura.clone());
        }
        self.attacker_auras.insert(
            aura,
            AuraApplication {
                aura: aura,
                start_ms: self.time_ms,
                end_ms: end_time,
                stacks: stacks,
            },
        );
        aura.on_start(self, Unit::Attacker);
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
        aura.on_end(self, game_params, event, events, Unit::Attacker);
        self.attacker_auras.remove(aura);
    }

    pub fn on_expire_attacker_aura(
        &mut self,
        aura: &Aura,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        aura.on_end(self, game_params, event, events, Unit::Attacker);
        self.attacker_auras.remove(aura);
    }

    pub fn add_target_aura(
        &mut self,
        aura: Aura,
        duration: Option<u64>,
        stacks: Option<u64>,
        events: &mut BinaryHeap<Event>,
    ) {
        let end_time = if duration.is_none() {
            None
        } else {
            Some(self.time_ms + duration.unwrap())
        };

        insert_aura_target_start_event(events, self.time_ms, aura.clone());
        if end_time.is_some() {
            insert_aura_target_end_event(events, end_time.unwrap(), aura.clone());
        }
        self.target_auras.insert(
            aura,
            AuraApplication {
                aura: aura,
                start_ms: self.time_ms,
                end_ms: end_time,
                stacks: stacks,
            },
        );
        aura.on_start(self, Unit::Target);
    }

    pub fn end_early_target_aura(
        &mut self,
        aura: &Aura,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        // remove the default 'aura_target_end' event (added in add_target_aura)
        events.retain(|event| {
            event.category != EventCategory::AuraTargetEnd || event.aura != Some(aura.clone())
        });

        // then proceed
        insert_aura_target_end_event(events, self.time_ms, aura.clone());
        aura.on_end(self, game_params, event, events, Unit::Target);
        self.target_auras.remove(aura);
    }

    pub fn on_expire_target_aura(
        &mut self,
        aura: &Aura,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        aura.on_end(self, game_params, event, events, Unit::Target);
        self.target_auras.remove(aura);
    }
}

pub fn run(
    mut selected_commands: VecDeque<AttackType>,
    game_params: &GameParams,
) -> (f64, Vec<DamageInfo>, Vec<Event>, u64, bool) {
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
        event_history: &mut Vec::new(),
        recast_charges: &mut Vec::new(),
        recast_ready: &mut HashSet::new(),
        is_casting: false,
    };

    // add first attack event
    insert_next_attack_event(
        &mut events,
        &mut selected_commands,
        &mut state,
        0,
        game_params,
    );

    add_initial_auras(game_params, &mut state, &mut events);

    // and launch
    return execute_commands(&mut events, &mut selected_commands, &mut state, game_params);
}

fn add_initial_auras(
    game_params: &GameParams<'_>,
    state: &mut State<'_>,
    events: &mut BinaryHeap<Event>,
) {
    if game_params
        .initial_config
        .get("ITEM_OPPORTUNITY_PREPARATION_READY")
        .unwrap_or(&"TRUE".to_string())
        == "TRUE"
        && game_params
            .items
            .iter()
            .any(|item_data| item_data.item == Item::Opportunity)
    {
        // println!("Adding Opportunity aura");
        state.add_attacker_aura(Aura::Preparation, Some(3_000), None, events);
    }

    if game_params
        .initial_config
        .get("ITEM_HUBRIS_EMINENCE_ACTIVE")
        .unwrap_or(&"TRUE".to_string())
        == "TRUE"
        && game_params
            .items
            .iter()
            .any(|item_data| item_data.item == Item::Hubris)
    {
        let stacks = game_params
            .initial_config
            .get("ITEM_HUBRIS_EMINENCE_STACKS")
            .unwrap_or(&"17".to_string())
            .parse::<u64>()
            .unwrap();
        // println!("Adding Hubris aura with {} stacks", stacks);
        state.add_attacker_aura(Aura::HubrisEminence, Some(90_000), Some(stacks), events);
    }

    for aura_app in game_params.initial_attacker_auras.iter() {
        state.add_attacker_aura(aura_app.aura, aura_app.end_ms, aura_app.stacks, events);
    }

    for aura_app in game_params.initial_target_auras.iter() {
        state.add_target_aura(aura_app.aura, aura_app.end_ms, aura_app.stacks, events);
    }
}

fn execute_commands(
    events: &mut BinaryHeap<Event>,
    remaining_commands: &mut VecDeque<AttackType>,
    state: &mut State,
    game_params: &GameParams,
) -> (f64, Vec<DamageInfo>, Vec<Event>, u64, bool) {
    loop {
        match events.pop() {
            None => {
                return (
                    state.total_damage.clone(),
                    state.damage_history.clone(),
                    state.event_history.clone(),
                    state.last_attack_time_ms,
                    false,
                )
            }
            Some(next_event) => {
                if game_params.capture_event_history {
                    state.event_history.push(next_event.clone());
                }

                if next_event.category == EventCategory::TargetDied {
                    return (
                        state.total_damage.clone(),
                        state.damage_history.clone(),
                        state.event_history.clone(),
                        state.last_attack_time_ms,
                        true,
                    );
                }

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
    // if state.total_damage >= game_params.initial_target_stats.current_health {
    //     return;
    // }

    // println!("on_event. event: {:#?}. events:  {:#?}", event, events);
    // advance time
    if state.time_ms != event.time_ms {
        if state.time_ms > event.time_ms {
            panic!("Time went backwards");
        }

        let time_passed = event.time_ms - state.time_ms;

        state.time_ms = event.time_ms;
        state.refresh_cds_and_auras(game_params, event, events);
        on_time_passed(event, events, game_params, state, time_passed);
    }

    match event.category {
        EventCategory::AttackCastStart => {
            state.is_casting = true;

            ensure_spell_off_cooldown(event, events, game_params, state);
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

            insert_attack_cast_end_event(event, events, state.time_ms + cast_time);
        }
        EventCategory::AttackCastEnd => {
            state.is_casting = false;

            handle_dash_if_applicable(event, events, game_params, state);
            trigger_stealth_exit_if_applicable(event, events, game_params, state);

            let attacker_stats: AttackerStats = compute_attacker_stats(game_params, state);
            let target_stats = compute_target_stats(game_params, state);

            on_pre_damage_events(
                &(DamageInfo {
                    amount: 0.0,
                    damage_type: DamageType::Unknown,
                    time_ms: state.time_ms,
                    source: DamageSource::Ability,
                    source_ability: Some(event.attack_type.unwrap()),
                    source_rune: None,
                    source_item: None,
                }),
                &attacker_stats,
                state,
                game_params,
                event,
                events,
            );

            let spell_result: SpellResult = simulate_spell(
                &attacker_stats,
                &target_stats,
                game_params,
                state,
                event.attack_type.unwrap(),
                event,
                events,
            );

            // println!("spell_result: {:#?}", spell_result);
            if spell_result.damage.is_some() {
                let damage_info = on_damage_from_ability(
                    &spell_result.damage.unwrap(),
                    spell_result.damage_type.unwrap(),
                    state,
                    event.attack_type.unwrap(),
                );
                on_post_damage_events(
                    &damage_info,
                    &attacker_stats,
                    state,
                    game_params,
                    event,
                    events,
                );
            }

            let target_stats = compute_target_stats(game_params, state);
            if target_stats.current_health <= 0.0 {
                let event = Event {
                    attack_type: None,
                    category: EventCategory::TargetDied,
                    time_ms: state.time_ms,
                    passive_effect: None,
                    aura: None,
                };

                events.push(event);
                return;
            }

            if spell_result.cooldown.is_some() {
                let cooldown_end_ms = spell_result.cooldown.unwrap() + state.time_ms;
                insert_cooldown_ended_event(events, event, cooldown_end_ms);
                add_cooldown_to_state(state, event.attack_type.unwrap(), cooldown_end_ms);
            }
            insert_next_attack_event(
                events,
                remaining_commands,
                state,
                state.time_ms,
                game_params,
            );
        }
        // EventCategory::CooldownEnded => on_cooldown_ended(event),
        // EventCategory::PassiveTriggered => on_passive_triggered(event),
        EventCategory::CooldownEnded => (),
        EventCategory::PassiveTriggered => (),
        EventCategory::AuraAttackerStart => (),
        EventCategory::AuraAttackerEnd => (),
        EventCategory::AuraTargetStart => (),
        EventCategory::AuraTargetEnd => (),
        EventCategory::TargetDied => (),
    }
}

fn on_time_passed(
    event: &crate::simulation::Event,
    events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    game_params: &GameParams<'_>,
    state: &mut State<'_>,
    duration: u64,
) {
    if !state.is_casting {
        for effect in game_params.passive_effects.iter() {
            effect.handle_on_movement(event, events, game_params, state, duration);
        }
    }
}

fn ensure_spell_off_cooldown(
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
    if state.attacker_auras.get(&Aura::Invisibility).is_none() {
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
    damage_type: DamageType,
    state: &mut State,
    attack_type: AttackType,
) -> DamageInfo {
    state.total_damage += damage;
    state.last_attack_time_ms = state.time_ms;
    let damage = DamageInfo {
        amount: *damage,
        damage_type: damage_type,
        time_ms: state.time_ms,
        source: DamageSource::Ability,
        source_ability: Some(attack_type),
        source_rune: None,
        source_item: None,
    };
    state.damage_history.push(damage.clone());

    return damage;
}

pub fn on_damage_from_rune(damage: &f64, damage_type: DamageType, state: &mut State, rune: Rune) {
    state.total_damage += damage;
    state.damage_history.push(DamageInfo {
        amount: *damage,
        damage_type,
        time_ms: state.time_ms,
        source: DamageSource::Rune,
        source_ability: None,
        source_rune: Some(rune),
        source_item: None,
    });
    state.last_attack_time_ms = state.time_ms;
}

pub fn on_damage_from_item(
    damage: &f64,
    damage_type: DamageType,
    state: &mut State,
    item_name: Item,
) -> DamageInfo {
    let damage_info = DamageInfo {
        amount: *damage,
        damage_type,
        time_ms: state.time_ms,
        source: DamageSource::ItemPassive,
        source_ability: None,
        source_rune: None,
        source_item: Some(item_name),
    };

    state.total_damage += damage;
    state.damage_history.push(damage_info.clone());
    state.last_attack_time_ms = state.time_ms;

    return damage_info;
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
    // If there are no more commands, do nothing
    let Some(attack_type) = commands.pop_front() else {
        return;
    };

    // Calculate when the next attack should occur
    let time_ms = calculate_next_attack_time(attack_type, state, current_time_ms, game_params);

    // Create and push the attack event
    let event = Event {
        attack_type: Some(attack_type),
        category: EventCategory::AttackCastStart,
        time_ms,
        passive_effect: None,
        aura: None,
    };

    events.push(event);
}

/// Calculates the time when the next attack should occur based on cooldowns and auras
fn calculate_next_attack_time(
    attack_type: AttackType,
    state: &mut State,
    current_time_ms: u64,
    game_params: &GameParams,
) -> u64 {
    // If ability is not on cooldown, it can be used immediately
    if !state.cooldowns.contains_key(&attack_type) {
        return current_time_ms;
    }

    // Special case: ability has recast charges
    if state.recast_charges.contains(&attack_type) {
        let ability = find_ability(
            game_params.abilities,
            attack_type,
            game_params.initial_config,
        );

        // Check for invisibility aura
        if let Some(invis_aura_app) = state.attacker_auras.get(&Aura::Invisibility) {
            return invis_aura_app.end_ms.unwrap() + ability.recast_gap_duration.unwrap();
        }

        // Check for void assault delay aura
        if let Some(delay_aura_app) = state.attacker_auras.get(&Aura::VoidAssaultDelay) {
            return delay_aura_app.end_ms.unwrap();
        }

        // Check for void assault recast ready aura
        if state
            .attacker_auras
            .contains_key(&Aura::VoidAssaultRecastReady)
        {
            return current_time_ms;
        } else {
            panic!("this should not happen")
        }
    }

    // Default case: ability is on cooldown, schedule it after cooldown ends
    // Remove and return the cooldown time
    state.cooldowns.remove(&attack_type).unwrap()
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

pub fn insert_aura_target_start_event(events: &mut BinaryHeap<Event>, time_ms: u64, aura: Aura) {
    let event = Event {
        attack_type: None,
        category: EventCategory::AuraTargetStart,
        time_ms,
        passive_effect: None,
        aura: Some(aura),
    };

    events.push(event);
}

pub fn insert_aura_target_end_event(events: &mut BinaryHeap<Event>, time_ms: u64, aura: Aura) {
    let event = Event {
        attack_type: None,
        category: EventCategory::AuraTargetEnd,
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

pub fn on_pre_damage_events(
    damage_info: &DamageInfo,
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
        effect.handle_on_pre_damage(
            damage_info,
            attacker_stats,
            state,
            game_params,
            event,
            events,
        );
    }

    // println!("aura effects:");
    // for (aura, _) in state.attacker_auras.clone().iter() {
    //     // println!("{:#?}", aura);
    //     aura.on_post_damage(
    //         damage_info,
    //         attacker_stats,
    //         state,
    //         game_params,
    //         event,
    //         events,
    //     );
    // }
    // println!("on_post_damage_events-----------------------");
}

pub fn on_post_damage_events(
    damage_info: &DamageInfo,
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
        effect.handle_on_post_damage(
            damage_info,
            attacker_stats,
            state,
            game_params,
            event,
            events,
        );
    }

    // println!("aura effects: {:#?}", state.attacker_auras);
    for (aura, _) in state.attacker_auras.clone().iter() {
        // println!("----");
        // println!("{:#?}", aura);
        aura.on_post_damage(
            damage_info,
            attacker_stats,
            state,
            game_params,
            event,
            events,
        );
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
