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
            compute_attacker_stats, compute_target_stats, AttackerStats, Aura, AuraApplication,
            Champion, DamageType, GameParams, PassiveEffect, Unit,
        },
        items::Item,
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
        aura.on_end(self, game_params, event, events, Unit::Attacker, true);
        self.attacker_auras.remove(aura);
    }

    pub fn on_expire_attacker_aura(
        &mut self,
        aura: &Aura,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        aura.on_end(self, game_params, event, events, Unit::Attacker, false);
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
        aura.on_end(self, game_params, event, events, Unit::Target, true);
        self.target_auras.remove(aura);
    }

    pub fn on_expire_target_aura(
        &mut self,
        aura: &Aura,
        game_params: &GameParams<'_>,
        event: &Event,
        events: &mut BinaryHeap<Event>,
    ) {
        aura.on_end(self, game_params, event, events, Unit::Target, false);
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
    insert_next_attack_event(&mut events, &mut selected_commands, &mut state, game_params);

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

            ensure_spell_off_cooldown(event.attack_type.unwrap(), game_params, state);
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
            insert_next_attack_event(events, remaining_commands, state, game_params);
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
    attack_type: AttackType,
    game_params: &GameParams<'_>,
    state: &mut State<'_>,
) {
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
    game_params: &GameParams,
) {
    // If there are no more commands, do nothing
    let Some(next_command_attack_type) = commands.front() else {
        return;
    };

    // Calculate when the next attack should occur
    let next_possible_attack_ms =
        next_earliest_time_possible_for(*next_command_attack_type, state, game_params);

    if *next_command_attack_type != AttackType::AA
        && game_params.weave_auto_attacks
        && can_weave_auto_attack(state, game_params, next_possible_attack_ms)
    {
        let next_possible_auto_attack_ms =
            next_earliest_time_possible_for(AttackType::AA, state, game_params);
        let event: Event = Event {
            attack_type: Some(AttackType::AA),
            category: EventCategory::AttackCastStart,
            time_ms: next_possible_auto_attack_ms,
            passive_effect: None,
            aura: None,
        };

        events.push(event);
    } else {
        let event = Event {
            attack_type: Some(*next_command_attack_type),
            category: EventCategory::AttackCastStart,
            time_ms: next_possible_attack_ms,
            passive_effect: None,
            aura: None,
        };

        events.push(event);
        commands.pop_front();
    }
}

/// Calculates the time when the next attack can occur based on cooldowns and auras
fn next_earliest_time_possible_for(
    attack_type: AttackType,
    state: &State,
    game_params: &GameParams,
) -> u64 {
    let current_time_ms = state.time_ms;
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
        if state.recast_ready.contains(&attack_type) {
            return current_time_ms;
        }

        panic!("this should not happen")
    }

    // Default case: ability is on cooldown, return the cooldown time
    *state.cooldowns.get(&attack_type).unwrap()
}

fn can_weave_auto_attack(
    state: &mut State<'_>,
    game_params: &GameParams<'_>,
    next_possible_attack_ms: u64,
) -> bool {
    let next_possible_aa_ms = next_earliest_time_possible_for(AttackType::AA, state, game_params);

    let attacker_stats: AttackerStats = compute_attacker_stats(game_params, state);

    let aa_cast_time = cast_time(
        &attacker_stats,
        AttackType::AA,
        game_params.initial_config,
        game_params.abilities,
    );

    let aa_cast_end_ms = next_possible_aa_ms + aa_cast_time;

    aa_cast_end_ms <= next_possible_attack_ms
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
mod integration_tests {
    use super::*;
    use crate::data_input::{
        self,
        common::{Aura, AuraApplication, CritHandlingChoice, GameParams, TargetStats},
    };
    use std::collections::{HashMap, HashSet, VecDeque};

    #[test]
    fn test_next_earliest_time_possible_for_no_cooldown() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let mut game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: false,
        };

        let r_ability = static_data
            .abilities
            .iter()
            .find(|ability| ability.attack_type == AttackType::R)
            .unwrap();

        let next_attack_time =
            next_earliest_time_possible_for(AttackType::Q, &mut state, &game_params);
        assert_eq!(next_attack_time, current_time_ms);

        fast_forward_to(next_attack_time, &mut state, &game_params);

        ensure_spell_off_cooldown(AttackType::R, &game_params, &mut state);

        let cast_end_ms = r_ability.cast_time_ms.unwrap_or_default() + next_attack_time;

        fast_forward_to(cast_end_ms, &mut state, &game_params);

        simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: cast_end_ms,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );
    }

    #[test]
    fn test_next_earliest_time_possible_for_with_invisibility() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: false,
        };

        let spell_result = simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: 2_500,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );

        let r_ability = static_data
            .abilities
            .iter()
            .find(|ability| ability.attack_type == AttackType::R)
            .unwrap();

        let cd_ms = *r_ability.cooldown_ms.as_ref().unwrap().get(&3u64).unwrap();

        assert_eq!(
            SpellResult {
                damage: None,
                damage_type: None,
                cooldown: Some(cd_ms),
            },
            spell_result
        );

        add_cooldown_to_state(&mut state, AttackType::R, cd_ms);

        let invis_end_ms = state
            .attacker_auras
            .get(&Aura::Invisibility)
            .unwrap()
            .end_ms
            .unwrap();

        let recast_gap = r_ability.recast_gap_duration.unwrap();

        let next_attack_time =
            next_earliest_time_possible_for(AttackType::R, &mut state, &game_params);
        assert_eq!(next_attack_time, invis_end_ms + recast_gap);

        // cannot fast forward to next attack time directly because of how
        // refresh_cds_and_auras works.
        fast_forward_to(invis_end_ms, &mut state, &game_params);
        fast_forward_to(next_attack_time, &mut state, &game_params);

        ensure_spell_off_cooldown(AttackType::R, &game_params, &mut state);

        let cast_end_ms = r_ability.cast_time_ms.unwrap_or_default() + next_attack_time;

        fast_forward_to(cast_end_ms, &mut state, &game_params);

        simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: cast_end_ms,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );
    }

    #[test]
    fn test_next_earliest_time_possible_for_with_void_assault_delay() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: false,
        };

        simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: 2_500,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );

        let r_ability = static_data
            .abilities
            .iter()
            .find(|ability| ability.attack_type == AttackType::R)
            .unwrap();

        let cd_ms = *r_ability.cooldown_ms.as_ref().unwrap().get(&3u64).unwrap();

        add_cooldown_to_state(&mut state, AttackType::R, cd_ms);

        let invis_end_ms = state
            .attacker_auras
            .get(&Aura::Invisibility)
            .unwrap()
            .end_ms
            .unwrap();

        fast_forward_to(invis_end_ms, &mut state, &game_params);

        let gap_end_ms = state
            .attacker_auras
            .get(&Aura::VoidAssaultDelay)
            .unwrap()
            .end_ms
            .unwrap();

        let next_attack_time =
            next_earliest_time_possible_for(AttackType::R, &mut state, &game_params);
        assert_eq!(next_attack_time, gap_end_ms);

        fast_forward_to(next_attack_time, &mut state, &game_params);

        ensure_spell_off_cooldown(AttackType::R, &game_params, &mut state);

        let cast_end_ms = r_ability.cast_time_ms.unwrap_or_default() + gap_end_ms;

        fast_forward_to(cast_end_ms, &mut state, &game_params);

        simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: gap_end_ms,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );
    }

    #[test]
    fn test_next_earliest_time_possible_for_with_void_assault_recast_ready() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: false,
        };

        simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: 2_500,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );

        let r_ability = static_data
            .abilities
            .iter()
            .find(|ability| ability.attack_type == AttackType::R)
            .unwrap();

        let cd_ms = *r_ability.cooldown_ms.as_ref().unwrap().get(&3u64).unwrap();

        add_cooldown_to_state(&mut state, AttackType::R, cd_ms);

        let invis_end_ms = state
            .attacker_auras
            .get(&Aura::Invisibility)
            .unwrap()
            .end_ms
            .unwrap();

        fast_forward_to(invis_end_ms, &mut state, &game_params);

        let gap_end_ms = state
            .attacker_auras
            .get(&Aura::VoidAssaultDelay)
            .unwrap()
            .end_ms
            .unwrap();

        fast_forward_to(gap_end_ms, &mut state, &game_params);

        let possible_recast_end_ms = state
            .attacker_auras
            .get(&Aura::VoidAssaultRecastReady)
            .unwrap()
            .end_ms
            .unwrap();

        // check that the ability can be recasted straight away at any time between the beginning
        // of VoidAssaultRecastReady to its end.
        for i in gap_end_ms..possible_recast_end_ms {
            fast_forward_to(i, &mut state, &game_params);

            let next_attack_time =
                next_earliest_time_possible_for(AttackType::R, &mut state, &game_params);
            assert_eq!(next_attack_time, i);
        }

        ensure_spell_off_cooldown(AttackType::R, &game_params, &mut state);

        let cast_end_ms = r_ability.cast_time_ms.unwrap_or_default() + state.time_ms;

        fast_forward_to(cast_end_ms, &mut state, &game_params);

        simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: gap_end_ms,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );
    }

    #[test]
    fn test_next_earliest_time_possible_for_on_cooldown() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: false,
        };

        simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: current_time_ms,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );

        let r_ability = static_data
            .abilities
            .iter()
            .find(|ability| ability.attack_type == AttackType::R)
            .unwrap();

        let cd_ms = *r_ability.cooldown_ms.as_ref().unwrap().get(&3u64).unwrap();

        add_cooldown_to_state(&mut state, AttackType::R, cd_ms + current_time_ms);

        let invis_end_ms = state
            .attacker_auras
            .get(&Aura::Invisibility)
            .unwrap()
            .end_ms
            .unwrap();

        fast_forward_to(invis_end_ms, &mut state, &game_params);

        let gap_end_ms = state
            .attacker_auras
            .get(&Aura::VoidAssaultDelay)
            .unwrap()
            .end_ms
            .unwrap();

        fast_forward_to(gap_end_ms, &mut state, &game_params);

        let possible_recast_end_ms = state
            .attacker_auras
            .get(&Aura::VoidAssaultRecastReady)
            .unwrap()
            .end_ms
            .unwrap();

        fast_forward_to(possible_recast_end_ms, &mut state, &game_params);

        let next_attack_time =
            next_earliest_time_possible_for(AttackType::R, &mut state, &game_params);
        assert_eq!(next_attack_time, cd_ms + current_time_ms);

        fast_forward_to(next_attack_time, &mut state, &game_params);

        ensure_spell_off_cooldown(AttackType::R, &game_params, &mut state);

        let cast_end_ms = r_ability.cast_time_ms.unwrap_or_default() + gap_end_ms;

        fast_forward_to(cast_end_ms, &mut state, &game_params);

        simulate_spell(
            &compute_attacker_stats(&game_params, &state),
            &compute_target_stats(&game_params, &state),
            &game_params,
            &mut state,
            AttackType::R,
            &Event {
                attack_type: Some(AttackType::R),
                category: EventCategory::AttackCastEnd,
                time_ms: gap_end_ms,
                passive_effect: None,
                aura: None,
            },
            &mut BinaryHeap::new(),
        );
    }

    fn fast_forward_to(new_time_s: u64, state: &mut State<'_>, game_params: &GameParams<'_>) {
        state.time_ms = new_time_s;

        // the content of the event does not matter
        let event = Event {
            attack_type: None,
            category: EventCategory::AuraAttackerEnd,
            time_ms: 0,
            passive_effect: None,
            aura: Some(Aura::Invisibility),
        };

        let events = &mut BinaryHeap::new();

        state.refresh_cds_and_auras(game_params, &event, events);
    }
}

#[cfg(test)]
mod insert_next_attack_event_tests {
    use super::*;
    use crate::data_input::{
        self,
        common::{Aura, AuraApplication, CritHandlingChoice, GameParams, TargetStats},
    };
    use std::collections::{HashMap, HashSet, VecDeque};

    #[test]
    fn works() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: false,
        };

        let events: &mut BinaryHeap<Event> = &mut BinaryHeap::new();
        let mut commands = VecDeque::new();
        commands.push_back(AttackType::Q);
        commands.push_back(AttackType::AA);

        insert_next_attack_event(events, &mut commands, &mut state, &game_params);

        assert_eq!(
            events.as_slice(),
            vec![Event {
                time_ms: current_time_ms,
                category: EventCategory::AttackCastStart,
                attack_type: Some(AttackType::Q),
                passive_effect: None,
                aura: None
            }]
        );

        assert_eq!(commands, vec![AttackType::AA]);
    }

    #[test]
    fn works_with_weaving_auto_attack() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: true,
        };

        let attacker_stats = compute_attacker_stats(&game_params, &state);
        let aa_cast_time = cast_time(
            &attacker_stats,
            AttackType::AA,
            &config,
            &static_data.abilities,
        );

        add_cooldown_to_state(&mut state, AttackType::Q, current_time_ms + aa_cast_time);

        let events: &mut BinaryHeap<Event> = &mut BinaryHeap::new();
        let mut commands = VecDeque::new();
        commands.push_back(AttackType::Q);

        insert_next_attack_event(events, &mut commands, &mut state, &game_params);

        assert_eq!(
            events.as_slice(),
            vec![Event {
                time_ms: current_time_ms,
                category: EventCategory::AttackCastStart,
                attack_type: Some(AttackType::AA),
                passive_effect: None,
                aura: None
            }]
        );

        assert_eq!(commands, vec![AttackType::Q]);
    }

    #[test]
    fn does_not_weave_auto_attack_if_not_enough_time() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: true,
        };

        let attacker_stats = compute_attacker_stats(&game_params, &state);
        let aa_cast_time = cast_time(
            &attacker_stats,
            AttackType::AA,
            &config,
            &static_data.abilities,
        );

        let cooldown_end_ms = current_time_ms + aa_cast_time - 1;
        add_cooldown_to_state(&mut state, AttackType::Q, cooldown_end_ms);

        let events: &mut BinaryHeap<Event> = &mut BinaryHeap::new();
        let mut commands = VecDeque::new();
        commands.push_back(AttackType::Q);

        insert_next_attack_event(events, &mut commands, &mut state, &game_params);

        assert_eq!(
            events.as_slice(),
            vec![Event {
                time_ms: cooldown_end_ms,
                category: EventCategory::AttackCastStart,
                attack_type: Some(AttackType::Q),
                passive_effect: None,
                aura: None
            }]
        );

        assert_eq!(commands, vec![]);
    }

    #[test]
    fn does_not_weave_auto_attack_with_another_auto() {
        let current_time_ms = 5_000;

        let mut state = State {
            total_damage: 0.0,
            time_ms: current_time_ms,
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

        let config = HashMap::new();
        let static_data = data_input::parse_files(Champion::Khazix, &Vec::new(), &config);

        let mut runes: HashSet<Rune> = HashSet::new();
        runes.insert(Rune::DarkHarvest);
        runes.insert(Rune::SuddenImpact);
        runes.insert(Rune::AbsoluteFocus);
        runes.insert(Rune::GatheringStorm);
        runes.insert(Rune::AdaptiveForce1);
        runes.insert(Rune::AdaptiveForce2);

        let game_params: GameParams<'_> = GameParams {
            champion: Champion::Khazix,
            champion_data: &static_data.champion_data,
            champion_stats: &static_data.base_champion_stats,
            level: 18,
            items: &Vec::new(),
            initial_config: &config,
            abilities: &static_data.abilities,
            initial_target_stats: &TargetStats {
                armor: 0.0,
                magic_resistance: 0.0,
                max_health: 1000.0,
                current_health: 1000.0,
            },
            runes: &runes,
            attacker_hp_perc: 100.0,
            runes_data: &static_data.runes_data,
            passive_effects: &mut Vec::new(),
            crit_handling: CritHandlingChoice::Min,
            initial_attacker_auras: &Vec::new(),
            initial_target_auras: &Vec::new(),
            abilities_extra_data: &static_data.abilities_extra_data,
            start_time_ms: 0,
            capture_event_history: false,
            weave_auto_attacks: true,
        };

        let attacker_stats = compute_attacker_stats(&game_params, &state);
        let aa_cast_time = cast_time(
            &attacker_stats,
            AttackType::AA,
            &config,
            &static_data.abilities,
        );

        let cooldown_end_ms = current_time_ms + aa_cast_time;
        add_cooldown_to_state(&mut state, AttackType::AA, cooldown_end_ms);

        let events: &mut BinaryHeap<Event> = &mut BinaryHeap::new();
        let mut commands = VecDeque::new();
        commands.push_back(AttackType::AA);

        insert_next_attack_event(events, &mut commands, &mut state, &game_params);

        assert_eq!(
            events.as_slice(),
            vec![Event {
                time_ms: cooldown_end_ms,
                category: EventCategory::AttackCastStart,
                attack_type: Some(AttackType::AA),
                passive_effect: None,
                aura: None
            }]
        );

        assert_eq!(commands, vec![]);
    }
}
