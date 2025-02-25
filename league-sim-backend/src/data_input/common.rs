use std::{
    cmp::{self, Ordering},
    collections::{HashMap, HashSet},
    f32::consts::E,
    ops::{Add, AddAssign},
};

use itertools::Itertools;

use crate::{
    attack::AttackType,
    simulation::{DamageInfo, State},
};

use super::{
    abilities::{self, find_ability, AbilitiesExtraData, SpellData},
    champions::{stat_increase, AdaptiveType, ChampionData, ChampionStats},
    items::{Item, ItemData},
    runes::{collect_runes_stats, Rune, RunesData},
};

#[derive(PartialEq)]
pub enum Champion {
    Khazix,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CritHandlingChoice {
    Min,
    Max,
    Avg,
}

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Offensive
#[derive(Debug, Default)]
pub struct AttackerStats {
    pub level: u64,
    pub ability_haste: f64,
    pub basic_ability_haste: f64,
    pub ultimate_haste: f64,
    pub ability_power: f64,
    pub ad_base: f64,
    pub ad_bonus: f64,
    pub lethality: f64,
    pub armor_penetration_perc: f64,
    pub crit_chance: f64,
    pub attack_speed_base: f64,
    pub attack_speed_bonus: f64,
    pub attack_speed_ratio: f64,
    pub attack_delay_offset: f64,
    pub attack_cast_time: f64,
    pub attack_total_time: f64,
    // pub damage_physical_multiplier: f64,
    // pub damage_magical_multiplier: f64,
    // pub damage_true_multiplier: f64,
    pub damage_ability_multiplier: f64,
    pub adaptive_force: f64,
    pub movement_speed_base: f64,
    pub movement_speed_flat_bonus: f64,
    pub movement_speed_perc_bonus: f64,
}

impl AddAssign for AttackerStats {
    fn add_assign(&mut self, other: AttackerStats) {
        self.ability_haste += other.ability_haste;
        self.basic_ability_haste += other.basic_ability_haste;
        self.ultimate_haste += other.ultimate_haste;
        self.ad_base += other.ad_base;
        self.ad_bonus += other.ad_bonus;
        self.lethality += other.lethality;
        self.armor_penetration_perc += other.armor_penetration_perc;
        self.crit_chance = f64::min(self.crit_chance + other.crit_chance, 1.0);
        self.attack_speed_base += other.attack_speed_base;
        self.attack_speed_bonus += other.attack_speed_bonus;
        // // multipliers are combined multiplicatively
        // self.damage_physical_multiplier *= other.damage_physical_multiplier;
        // self.damage_magical_multiplier *= other.damage_magical_multiplier;
        // self.damage_true_multiplier *= other.damage_true_multiplier;
        self.damage_ability_multiplier =
            (self.damage_ability_multiplier + 1.0) * (other.damage_ability_multiplier + 1.0) - 1.0;
        self.adaptive_force += other.adaptive_force;
        self.ability_power += other.ability_power;
    }
}

impl Add for AttackerStats {
    type Output = AttackerStats;

    fn add(mut self, other: AttackerStats) -> AttackerStats {
        self += other;
        self
    }
}

// see https://leagueoflegends.fandom.com/wiki/Champion_statistic?so=search#Defensive
#[derive(Debug, Clone)]
pub struct TargetStats {
    pub armor: f64,
    pub magic_resistance: f64,
    pub max_health: f64,
    pub current_health: f64,
}

// this is a container for data that is constant throughout the duration of each simulation
pub struct GameParams<'a> {
    pub champion: Champion,
    pub champion_data: &'a ChampionData,
    pub champion_stats: &'a ChampionStats,
    pub level: u64,
    pub items: &'a Vec<&'a ItemData>,
    pub initial_config: &'a HashMap<String, String>,
    pub abilities: &'a Vec<SpellData>,
    pub abilities_extra_data: &'a AbilitiesExtraData,
    pub initial_target_stats: &'a TargetStats,
    pub runes: &'a HashSet<Rune>,
    pub attacker_hp_perc: f64,
    pub runes_data: &'a RunesData,
    pub passive_effects: &'a mut Vec<PassiveEffect>,
    pub crit_handling: CritHandlingChoice,
    pub initial_attacker_auras: &'a Vec<AuraApplication>,
    pub initial_target_auras: &'a Vec<AuraApplication>,
    pub start_time_ms: u64,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, serde::Serialize)]
pub enum PassiveEffect {
    // Items
    Carve,
    EverRisingMoon,
    Eminence,
    IonianInsight,
    Preparation,
    DragonForce,
    FocusedWill,
    LightshieldStrike,
    Death,
    Energized,
    Galvanize,
    Firmament,
    MistsEdge,

    // Runes
    DarkHarvest,
    SuddenImpact,
}

impl PassiveEffect {
    pub fn from_string(name: &str) -> Option<PassiveEffect> {
        match name {
            "Flux" => None,
            "Carve" => Some(Self::Carve),
            "Fervor" => None,
            "Ichorshield" => None,
            "Hackshorn" => None,
            "Ignore Pain" => None,
            "Defy" => None,
            "Ever Rising Moon" => Some(Self::EverRisingMoon),
            "Annul" => None,
            "Winter's Caress" => None,
            "Rebirth" => None,
            "Eminence" => Some(Self::Eminence),
            "Ionian Insight" => Some(Self::IonianInsight),
            "Lifeline" => None,
            "Grievous Wounds" => None,
            "Preparation" => Some(Self::Preparation),
            "Extraction" => None,
            "Cleave" => None,
            "Resilience" => None,
            "Shield Reaver" => None,
            "Bitter Cold" => None,
            "Dragonforce" => Some(Self::DragonForce),
            "Focused Will" => Some(Self::FocusedWill),
            "Lightshield Strike" => Some(Self::LightshieldStrike),
            "Death" => Some(Self::Death),
            "Taxes" => None,
            "Blackout" => None,
            "Extinguish" => None,
            "Energized" => Some(Self::Energized),
            "Galvanize" => Some(Self::Galvanize),
            "Firmament" => Some(Self::Firmament),
            "Haunt" => None,
            "Mist's Edge" => Some(Self::MistsEdge),
            "Clawing Shadows" => None,

            &_ => None,
        }
    }

    pub fn handle_on_pre_damage(
        &self,
        damage_info: &DamageInfo,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        match self {
            PassiveEffect::LightshieldStrike => Item::SunderedSky.handle_on_pre_damage(
                self,
                damage_info,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            PassiveEffect::Energized => Item::VoltaicCyclosword.handle_on_pre_damage(
                self,
                damage_info,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            PassiveEffect::MistsEdge => Item::BladeofTheRuinedKing.handle_on_pre_damage(
                self,
                damage_info,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            PassiveEffect::DarkHarvest => Rune::DarkHarvest.handle_on_pre_damage(
                damage_info,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            &_ => (),
        }
    }

    pub fn handle_on_post_damage(
        &self,
        damage_info: &DamageInfo,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        match self {
            PassiveEffect::Carve => Item::BlackCleaver.handle_on_post_damage(
                self,
                damage_info,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            PassiveEffect::SuddenImpact => (),
            PassiveEffect::EverRisingMoon => Item::Eclipse.handle_on_post_damage(
                self,
                damage_info,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            PassiveEffect::FocusedWill => Item::SpearofShojin.handle_on_post_damage(
                self,
                damage_info,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            PassiveEffect::LightshieldStrike => Item::SunderedSky.handle_on_post_damage(
                self,
                damage_info,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            &_ => (),
        }
    }

    pub(crate) fn handle_dash_event(
        &self,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
        game_params: &GameParams<'_>,
        state: &mut State<'_>,
    ) {
        match self {
            PassiveEffect::SuddenImpact => {
                Rune::SuddenImpact.handle_dash_event(event, events, state, game_params)
            }
            PassiveEffect::Carve => (),
            PassiveEffect::EverRisingMoon => (),
            PassiveEffect::Eminence => (),
            PassiveEffect::IonianInsight => (),
            PassiveEffect::Preparation => (),
            PassiveEffect::DragonForce => (),
            PassiveEffect::FocusedWill => (),
            PassiveEffect::LightshieldStrike => (),
            PassiveEffect::Death => (),
            PassiveEffect::Energized => (),
            PassiveEffect::Firmament => (),
            PassiveEffect::DarkHarvest => (),
            PassiveEffect::Galvanize => (),
            PassiveEffect::MistsEdge => (),
        }
    }

    pub(crate) fn handle_stealth_exit_event(
        &self,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
        game_params: &GameParams<'_>,
        state: &mut State<'_>,
    ) {
        match self {
            PassiveEffect::SuddenImpact => {
                Rune::SuddenImpact.handle_stealth_exit_event(event, events, state, game_params)
            }
            PassiveEffect::Carve => (),
            PassiveEffect::EverRisingMoon => (),
            PassiveEffect::Eminence => (),
            PassiveEffect::IonianInsight => (),
            PassiveEffect::Preparation => (),
            PassiveEffect::DragonForce => (),
            PassiveEffect::FocusedWill => (),
            PassiveEffect::LightshieldStrike => (),
            PassiveEffect::Death => (),
            PassiveEffect::Energized => (),
            PassiveEffect::Firmament => (),
            PassiveEffect::DarkHarvest => (),
            PassiveEffect::Galvanize => (),
            PassiveEffect::MistsEdge => (),
        }
    }

    fn offensive_stats(
        &self,
        state: &State<'_>,
        game_params: &GameParams<'_>,
    ) -> Option<AttackerStats> {
        match self {
            PassiveEffect::IonianInsight => {
                let offensive_stats = AttackerStats {
                    // todo: add summoner spell haste
                    ..Default::default()
                };

                Some(offensive_stats)
            }
            Self::DragonForce => {
                let offensive_stats = AttackerStats {
                    basic_ability_haste: 25.0,
                    ..Default::default()
                };

                Some(offensive_stats)
            }
            _ => None,
        }
    }

    pub fn execution_order(a: &Self, b: &Self) -> Ordering {
        let order_fn = |x: &Self| -> u32 {
            match &x {
                PassiveEffect::Firmament => 1,
                PassiveEffect::MistsEdge => 2,
                PassiveEffect::DarkHarvest => 3,
                PassiveEffect::LightshieldStrike => 4,
                _ => 5,
            }
        };

        let order_a: u32 = order_fn(a);
        let order_b: u32 = order_fn(b);

        // println!("order_a: {:#?}. effect: {:#?}", order_a, a);
        // println!("order_b: {:#?}. effect: {:#?}", order_b, b);

        order_a.cmp(&order_b)
    }

    pub fn handle_on_movement(
        &self,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
        game_params: &GameParams<'_>,
        state: &mut State<'_>,
        duration: u64,
    ) {
        match self {
            PassiveEffect::Energized => Item::VoltaicCyclosword.handle_on_movement(
                self,
                duration,
                state,
                game_params,
                event,
                events,
            ),
            &_ => (),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy, serde::Serialize)]
pub enum DamageType {
    Physical,
    Magical,
    True,
    Unknown,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Unit {
    Attacker,
    Target,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, serde::Serialize)]
pub enum Aura {
    SuddenImpactReady,
    Invisibility, // Stealth includes Camouflage & Invisibility
    UnseenThreat,
    VoidAssaultDelay,
    VoidAssaultRecastReady,
    Carve,
    EverRisingMoon,
    HubrisEminence,
    Preparation,
    FocusedWill,
    LightshieldStrike,
    Energized,
}

impl Aura {
    pub fn on_start(
        &self,
        state: &mut State<'_>,
        affected_unit: Unit,
        // game_params: &GameParams<'_>,
        // event: &crate::simulation::Event,
        // events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        // println!("aura.on_start: {:#?} {:#?}", self, event.time_ms);
        match self {
            Aura::VoidAssaultRecastReady => {
                state.recast_ready.insert(AttackType::R);
            }
            _ => (),
        }
    }

    pub fn on_end(
        &self,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
        affected_unit: Unit,
    ) {
        // println!("aura.on_end: {:#?} {:#?}", self, event.time_ms);

        match self {
            Aura::Invisibility => {
                if state.recast_charges.contains(&crate::attack::AttackType::R) {
                    let r_ability = find_ability(
                        game_params.abilities,
                        crate::attack::AttackType::R,
                        game_params.initial_config,
                    );

                    state.add_attacker_aura(
                        Aura::VoidAssaultDelay,
                        r_ability.recast_gap_duration,
                        None,
                        events,
                    );
                }
            }
            Aura::VoidAssaultDelay => {
                let r_ability = find_ability(
                    game_params.abilities,
                    crate::attack::AttackType::R,
                    game_params.initial_config,
                );
                state.add_attacker_aura(
                    Aura::VoidAssaultRecastReady,
                    r_ability.recast_window,
                    None,
                    events,
                );
            }
            Aura::VoidAssaultRecastReady => {
                state.recast_ready.remove(&AttackType::R);
            }
            _ => (),
        }
    }

    pub fn on_post_damage(
        &self,
        damage_info: &DamageInfo,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        match self {
            Aura::SuddenImpactReady => {
                game_params.runes_data.sudden_impact.handle_on_post_damage(
                    damage_info,
                    attacker_stats,
                    state,
                    game_params,
                    event,
                    events,
                );
            }

            Aura::UnseenThreat => {
                game_params
                    .abilities_extra_data
                    .unseen_threat
                    .on_post_damage(
                        damage_info,
                        attacker_stats,
                        state,
                        game_params,
                        event,
                        events,
                    );
            }

            _ => (),
        }
    }

    fn offensive_stats(
        &self,
        state: &State<'_>,
        game_params: &GameParams<'_>,
    ) -> Option<AttackerStats> {
        match self {
            Aura::HubrisEminence => {
                if game_params
                    .items
                    .iter()
                    .all(|item_data| item_data.item != Item::Hubris)
                {
                    return None;
                }

                let mut offensive_stats = AttackerStats {
                    ..Default::default()
                };

                let stacks: u64 =
                    if let Some(aura_app) = state.attacker_auras.get(&Aura::HubrisEminence) {
                        aura_app.stacks.unwrap()
                    } else {
                        panic!("HubrisEminence aura not found");
                    };

                offensive_stats.ad_bonus = stacks as f64;

                Some(offensive_stats)
            }
            Aura::Preparation => {
                let lethality = match game_params.champion_data.attack_type {
                    super::champions::AttackType::Melee => 11.0,
                    super::champions::AttackType::Ranged => 7.0,
                };

                let offensive_stats = AttackerStats {
                    lethality,
                    ..Default::default()
                };

                Some(offensive_stats)
            }
            Aura::FocusedWill => {
                if game_params
                    .items
                    .iter()
                    .all(|item_data| item_data.item != Item::SpearofShojin)
                {
                    panic!("SpearofShojin item not found despite a FocusedWill aura being present");
                }

                let stacks: u64 =
                    if let Some(aura_app) = state.attacker_auras.get(&Aura::FocusedWill) {
                        aura_app.stacks.unwrap()
                    } else {
                        panic!("FocusedWill aura not found");
                    };

                let offensive_stats = AttackerStats {
                    damage_ability_multiplier: (stacks as f64 * 0.03),
                    ..Default::default()
                };

                Some(offensive_stats)
            }
            Aura::LightshieldStrike => {
                if game_params
                    .items
                    .iter()
                    .all(|item_data| item_data.item != Item::SunderedSky)
                {
                    panic!(
                        "SunderedSky item not found despite a LightshieldStrike aura being present"
                    );
                }

                let offensive_stats = AttackerStats {
                    crit_chance: 1.0,
                    ..Default::default()
                };

                Some(offensive_stats)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuraApplication {
    pub aura: Aura,
    pub start_ms: u64,
    pub end_ms: Option<u64>,
    pub stacks: Option<u64>,
}

pub trait EffectWithCallbacks {
    fn on_post_damage(
        &self,
        damage_info: &DamageInfo,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
    }
}

pub fn compute_attacker_stats(game_params: &GameParams, state: &State) -> AttackerStats {
    let champion_stats = game_params.champion_stats;
    let level = game_params.level;
    let items = game_params.items;
    // see https://leagueoflegends.fandom.com/wiki/Champion_statistic
    let mut offensive_stats: AttackerStats = AttackerStats {
        level,
        ability_haste: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.ability_haste),
        ad_base: champion_stats.attack_damage_flat
            + stat_increase(champion_stats.attack_damage_per_level, level as f64),
        ad_bonus: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.ad_bonus),
        armor_penetration_perc: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.armor_penetration_perc),
        crit_chance: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.crit_chance),
        lethality: items
            .iter()
            .fold(0.0, |acc, x| acc + x.offensive_stats.lethality),
        // see https://leagueoflegends.fandom.com/wiki/Attack_speed#Calculations
        attack_speed_base: champion_stats.attack_speed_flat,
        attack_speed_bonus: stat_increase(champion_stats.attack_speed_per_level, level as f64)
            + items
                .iter()
                .fold(0.0, |acc, x| acc + x.offensive_stats.attack_speed_bonus),
        // damage_physical_multiplier: 0.0,
        // damage_magical_multiplier: 0.0,
        // damage_true_multiplier: 0.0,
        damage_ability_multiplier: 0.0,
        adaptive_force: 0.0,
        // todo
        ability_power: 0.0,
        attack_speed_ratio: champion_stats.attack_speed_ratio,
        attack_delay_offset: champion_stats.attack_delay_offset,
        attack_cast_time: champion_stats.attack_cast_time,
        attack_total_time: champion_stats.attack_total_time,
        // todo
        basic_ability_haste: 0.0,
        ultimate_haste: 0.0,
        movement_speed_base: champion_stats.base_movement_speed,
        movement_speed_flat_bonus: items.iter().fold(0.0, |acc, x| {
            acc + x.offensive_stats.movement_speed_flat_bonus
        }),
        movement_speed_perc_bonus: items.iter().fold(0.0, |acc, x| {
            acc + x.offensive_stats.movement_speed_perc_bonus
        }),
    };
    offensive_stats += collect_runes_stats(state, game_params);
    offensive_stats += collect_aura_stats(state, game_params);
    offensive_stats += collect_passive_effects_stats(state, game_params);

    apply_adaptive_force(&mut offensive_stats, game_params);

    // println!(
    //     "compute_attacker_stats ad: {:#?}, time: {:#?}",
    //     offensive_stats.ad_base + offensive_stats.ad_bonus,
    //     state.time_ms
    // );

    offensive_stats
}

fn collect_aura_stats(state: &State<'_>, game_params: &GameParams<'_>) -> AttackerStats {
    let mut offensive_stats = AttackerStats {
        ..Default::default()
    };

    for (aura, aura_app) in state.attacker_auras.iter() {
        let aura_stats: Option<AttackerStats> = aura.offensive_stats(state, game_params);
        if aura_stats.is_some() {
            offensive_stats += aura_stats.unwrap();
        }
    }

    return offensive_stats;
}

pub fn compute_target_stats(game_params: &GameParams, state: &State) -> TargetStats {
    let mut armor = game_params.initial_target_stats.armor;
    if let Some(carve_aura_app) = state.target_auras.get(&Aura::Carve) {
        let stacks = carve_aura_app.stacks.unwrap();
        armor *= 1.0 - (0.06 * stacks as f64);
    }
    // println!(
    //     "compute_target_stats armor: {:#?}, time: {:#?}",
    //     armor, state.time_ms
    // );

    return TargetStats {
        armor,
        max_health: game_params.initial_target_stats.max_health,
        current_health: game_params.initial_target_stats.current_health - state.total_damage,
        magic_resistance: game_params.initial_target_stats.magic_resistance,
    };
}

fn collect_passive_effects_stats(state: &State, game_params: &GameParams) -> AttackerStats {
    let mut offensive_stats: AttackerStats = AttackerStats {
        ..Default::default()
    };

    for passive_effect in game_params
        .items
        .iter()
        .flat_map(|&item_data| &item_data.passives)
    {
        let aura_stats: Option<AttackerStats> = passive_effect.offensive_stats(state, game_params);
        if aura_stats.is_some() {
            offensive_stats += aura_stats.unwrap();
        }
    }

    return offensive_stats;
}

pub fn convert_adaptive(adaptive_force: f64, damage_type: DamageType) -> f64 {
    match damage_type {
        DamageType::Magical => adaptive_force,
        DamageType::Physical => adaptive_force * 0.6,
        DamageType::True => panic!(),
        DamageType::Unknown => panic!(),
    }
}

fn apply_adaptive_force(offensive_stats: &mut AttackerStats, game_params: &GameParams) {
    if offensive_stats.ad_bonus > offensive_stats.ability_power {
        offensive_stats.ad_bonus +=
            convert_adaptive(offensive_stats.adaptive_force, DamageType::Physical);
    } else if offensive_stats.ad_bonus < offensive_stats.ability_power {
        offensive_stats.ability_power +=
            convert_adaptive(offensive_stats.adaptive_force, DamageType::Magical);
    } else {
        match game_params.champion_data.adaptive_type {
            AdaptiveType::Physical => {
                offensive_stats.ad_bonus +=
                    convert_adaptive(offensive_stats.adaptive_force, DamageType::Physical)
            }
            AdaptiveType::Magic => {
                offensive_stats.ability_power +=
                    convert_adaptive(offensive_stats.adaptive_force, DamageType::Magical)
            }
        };
    }
    offensive_stats.adaptive_force = 0.0;
}

pub fn apply_adaptive_damage(
    adaptive_damage: f64,
    offensive_stats: &AttackerStats,
    game_params: &GameParams,
) -> f64 {
    if offensive_stats.ad_bonus > offensive_stats.ability_power {
        convert_adaptive(adaptive_damage, DamageType::Physical)
    } else if offensive_stats.ad_bonus < offensive_stats.ability_power {
        convert_adaptive(adaptive_damage, DamageType::Magical)
    } else {
        let damage_type = match game_params.champion_data.adaptive_type {
            AdaptiveType::Physical => DamageType::Physical,
            AdaptiveType::Magic => DamageType::Magical,
        };
        convert_adaptive(adaptive_damage, damage_type)
    }
}

pub fn compile_passive_effects(game_params: &mut GameParams<'_>) {
    // iterate over effects from: items, runes and champions abilities
    let item_effects = game_params
        .items
        .iter()
        .map(|item: &&ItemData| item.passives.clone());

    let rune_effects = game_params
        .runes
        .iter()
        .map(|rune| rune.passive_effect())
        .filter(|passive_effect| passive_effect.is_some())
        .map(|passive_effect| vec![passive_effect.unwrap()]);

    // let champion_passives = game_params
    //     .abilities
    //     .iter()
    //     .map(|spell_data| spell_data.passive_effects.clone());

    let mut passive_effects = item_effects
        .chain(rune_effects)
        // .chain(champion_passives)
        .flat_map(|effects| effects)
        .collect_vec();

    passive_effects.sort_by(PassiveEffect::execution_order);

    game_params.passive_effects.append(&mut passive_effects);
}
