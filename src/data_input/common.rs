use std::{
    collections::{HashMap, HashSet},
    ops::{Add, AddAssign},
};

use itertools::Itertools;

use crate::simulation::State;

use super::{
    abilities::SpellData,
    champions::{stat_increase, ChampionStats},
    items::Item,
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
    pub damage_physical_multiplier: f64,
    pub damage_magical_multiplier: f64,
    pub damage_true_multiplier: f64,
    pub adaptive_force: f64,
}

impl AddAssign for AttackerStats {
    fn add_assign(&mut self, other: AttackerStats) {
        self.ability_haste += other.ability_haste;
        self.ad_base += other.ad_base;
        self.ad_bonus += other.ad_bonus;
        self.lethality += other.lethality;
        self.armor_penetration_perc += other.armor_penetration_perc;
        self.crit_chance += other.crit_chance;
        self.attack_speed_base += other.attack_speed_base;
        self.attack_speed_bonus += other.attack_speed_bonus;
        // multipliers are combined multiplicatively
        self.damage_physical_multiplier *= other.damage_physical_multiplier;
        self.damage_magical_multiplier *= other.damage_magical_multiplier;
        self.damage_true_multiplier *= other.damage_true_multiplier;
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
#[derive(Debug)]
pub struct TargetStats {
    pub armor: f64,
    pub hp: f64,
}

// this is a container for data that is constant throughout the duration of each simulation
pub struct GameParams<'a> {
    pub champion: Champion,
    pub champion_stats: &'a ChampionStats,
    pub level: u64,
    pub items: &'a Vec<&'a Item>,
    pub initial_config: &'a HashMap<String, String>,
    pub abilities: &'a Vec<SpellData>,
    pub target_stats: &'a TargetStats,
    pub runes: &'a HashSet<Rune>,
    pub attacker_hp_perc: f64,
    pub runes_data: &'a RunesData,
    pub passive_effects: &'a mut Vec<PassiveEffect>,
    pub crit_handling: CritHandlingChoice,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum PassiveEffect {
    // Items
    Haunt,
    BitterCold,
    Eminence,
    IgnorePain,
    Defy,
    Hackshorn,
    Ichorshield,
    Lifeline,
    Preparation,
    Extraction,
    EverRisingMoon,
    Blackout,
    Extinguish,
    Cleave,
    GrievousWounds,
    Carve,
    Fervor,
    Death,
    Taxes,
    Annul,
    IonianInsight,

    // Runes
    DarkHarvest,
    SuddenImpact,

    // Auras
    SuddenImpactReady,
    Stealth,
}

impl PassiveEffect {
    pub fn from_string(name: &str) -> PassiveEffect {
        match name {
            "Haunt" => PassiveEffect::Haunt,
            "Bitter Cold" => PassiveEffect::BitterCold,
            "Eminence" => PassiveEffect::Eminence,
            "Ignore Pain" => PassiveEffect::IgnorePain,
            "Defy" => PassiveEffect::Defy,
            "Hackshorn" => PassiveEffect::Hackshorn,
            "Ichorshield" => PassiveEffect::Ichorshield,
            "Lifeline" => PassiveEffect::Lifeline,
            "Preparation" => PassiveEffect::Preparation,
            "Extraction" => PassiveEffect::Extraction,
            "Ever Rising Moon" => PassiveEffect::EverRisingMoon,
            "Blackout" => PassiveEffect::Blackout,
            "Extinguish" => PassiveEffect::Extinguish,
            "Cleave" => PassiveEffect::Cleave,
            "Grievous Wounds" => PassiveEffect::GrievousWounds,
            "Carve" => PassiveEffect::Carve,
            "Fervor" => PassiveEffect::Fervor,
            "Death" => PassiveEffect::Death,
            "Taxes" => PassiveEffect::Taxes,
            "Annul" => PassiveEffect::Annul,
            "Ionian Insight" => PassiveEffect::IonianInsight,
            &_ => todo!("missing {name}"),
        }
    }

    pub fn to_string(effect: PassiveEffect) -> &'static str {
        match effect {
            // items
            PassiveEffect::Haunt => "Haunt",
            PassiveEffect::BitterCold => "Bitter Cold",
            PassiveEffect::Eminence => "Eminence",
            PassiveEffect::IgnorePain => "Ignore Pain",
            PassiveEffect::Defy => "Defy",
            PassiveEffect::Hackshorn => "Hackshorn",
            PassiveEffect::Ichorshield => "Ichorshield",
            PassiveEffect::Lifeline => "Lifeline",
            PassiveEffect::Preparation => "Preparation",
            PassiveEffect::Extraction => "Extraction",
            PassiveEffect::EverRisingMoon => "Ever Rising Moon",
            PassiveEffect::Blackout => "Blackout",
            PassiveEffect::Extinguish => "Extinguish",
            PassiveEffect::Cleave => "Cleave",
            PassiveEffect::GrievousWounds => "Grievous Wounds",
            PassiveEffect::Carve => "Carve",
            PassiveEffect::Fervor => "Fervor",
            PassiveEffect::Death => "Death",
            PassiveEffect::Taxes => "Taxes",
            PassiveEffect::Annul => "Annul",
            PassiveEffect::IonianInsight => "Ionian Insight",

            // runes
            PassiveEffect::DarkHarvest => "",
            PassiveEffect::SuddenImpact => "",

            // auras
            PassiveEffect::SuddenImpactReady => "",
            PassiveEffect::Stealth => "",
        }
    }

    pub fn handle_on_post_damage(
        &self,
        damage: f64,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        match self {
            PassiveEffect::Haunt => todo!(),
            PassiveEffect::BitterCold => todo!(),
            PassiveEffect::Eminence => todo!(),
            PassiveEffect::IgnorePain => todo!(),
            PassiveEffect::Defy => todo!(),
            PassiveEffect::Hackshorn => todo!(),
            PassiveEffect::Ichorshield => todo!(),
            PassiveEffect::Lifeline => todo!(),
            PassiveEffect::Preparation => todo!(),
            PassiveEffect::Extraction => todo!(),
            PassiveEffect::EverRisingMoon => todo!(),
            PassiveEffect::Blackout => todo!(),
            PassiveEffect::Extinguish => todo!(),
            PassiveEffect::Cleave => todo!(),
            PassiveEffect::GrievousWounds => todo!(),
            PassiveEffect::Carve => todo!(),
            PassiveEffect::Fervor => todo!(),
            PassiveEffect::Death => todo!(),
            PassiveEffect::Taxes => todo!(),
            PassiveEffect::Annul => todo!(),
            PassiveEffect::IonianInsight => (),
            PassiveEffect::DarkHarvest => Rune::DarkHarvest.handle_on_post_damage(
                damage,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),

            PassiveEffect::SuddenImpact => (),
            PassiveEffect::SuddenImpactReady => Aura::SuddenImpactReady.handle_on_post_damage(
                damage,
                attacker_stats,
                state,
                game_params,
                event,
                events,
            ),
            PassiveEffect::Stealth => (),
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
            PassiveEffect::Haunt => todo!(),
            PassiveEffect::BitterCold => todo!(),
            PassiveEffect::Eminence => todo!(),
            PassiveEffect::IgnorePain => todo!(),
            PassiveEffect::Defy => todo!(),
            PassiveEffect::Hackshorn => todo!(),
            PassiveEffect::Ichorshield => todo!(),
            PassiveEffect::Lifeline => todo!(),
            PassiveEffect::Preparation => todo!(),
            PassiveEffect::Extraction => todo!(),
            PassiveEffect::EverRisingMoon => todo!(),
            PassiveEffect::Blackout => todo!(),
            PassiveEffect::Extinguish => todo!(),
            PassiveEffect::Cleave => todo!(),
            PassiveEffect::GrievousWounds => todo!(),
            PassiveEffect::Carve => todo!(),
            PassiveEffect::Fervor => todo!(),
            PassiveEffect::Death => todo!(),
            PassiveEffect::Taxes => todo!(),
            PassiveEffect::Annul => todo!(),
            PassiveEffect::IonianInsight => (),
            PassiveEffect::DarkHarvest => (),

            PassiveEffect::SuddenImpact => {
                Rune::SuddenImpact.handle_dash_event(event, events, state, game_params)
            }
            PassiveEffect::SuddenImpactReady => (),

            PassiveEffect::Stealth => (),
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
            PassiveEffect::Haunt => todo!(),
            PassiveEffect::BitterCold => todo!(),
            PassiveEffect::Eminence => todo!(),
            PassiveEffect::IgnorePain => todo!(),
            PassiveEffect::Defy => todo!(),
            PassiveEffect::Hackshorn => todo!(),
            PassiveEffect::Ichorshield => todo!(),
            PassiveEffect::Lifeline => todo!(),
            PassiveEffect::Preparation => todo!(),
            PassiveEffect::Extraction => todo!(),
            PassiveEffect::EverRisingMoon => todo!(),
            PassiveEffect::Blackout => todo!(),
            PassiveEffect::Extinguish => todo!(),
            PassiveEffect::Cleave => todo!(),
            PassiveEffect::GrievousWounds => todo!(),
            PassiveEffect::Carve => todo!(),
            PassiveEffect::Fervor => todo!(),
            PassiveEffect::Death => todo!(),
            PassiveEffect::Taxes => todo!(),
            PassiveEffect::Annul => todo!(),
            PassiveEffect::IonianInsight => (),
            PassiveEffect::DarkHarvest => (),

            PassiveEffect::SuddenImpact => {
                Rune::SuddenImpact.handle_stealth_exit_event(event, events, state, game_params)
            }
            PassiveEffect::SuddenImpactReady => (),

            PassiveEffect::Stealth => (),
        }
    }
}

pub enum DamageType {
    Physical,
    Magical,
    True,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Aura {
    SuddenImpactReady,
    Stealth, // includes Camouflage & Invisibility
}

impl Aura {
    pub fn passive_effects(&self) -> Vec<PassiveEffect> {
        match self {
            Aura::SuddenImpactReady => vec![PassiveEffect::SuddenImpactReady],
            Aura::Stealth => vec![],
        }
    }

    fn handle_on_post_damage(
        &self,
        damage: f64,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        match self {
            Aura::SuddenImpactReady => {
                game_params.runes_data.sudden_impact.handle_buff_triggered(
                    damage,
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
        damage_physical_multiplier: 0.0,
        damage_magical_multiplier: 0.0,
        damage_true_multiplier: 0.0,
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
    };
    offensive_stats += collect_runes_stats(state, game_params);

    apply_passives(&mut offensive_stats, items);
    apply_adaptive_force(&mut offensive_stats);

    offensive_stats
}

fn apply_passives(offensive_stats: &mut AttackerStats, items: &Vec<&Item>) {
    // todo: change this in a callback fashion
}

pub fn convert_adaptive(adaptive_force: f64, damage_type: DamageType) -> f64 {
    match damage_type {
        DamageType::Magical => adaptive_force,
        DamageType::Physical => adaptive_force * 0.6,
        DamageType::True => panic!(),
    }
}

fn apply_adaptive_force(offensive_stats: &mut AttackerStats) {
    if offensive_stats.ad_bonus >= offensive_stats.ability_power {
        offensive_stats.ad_bonus +=
            convert_adaptive(offensive_stats.adaptive_force, DamageType::Physical);
    } else if offensive_stats.ad_bonus < offensive_stats.ability_power {
        offensive_stats.ability_power +=
            convert_adaptive(offensive_stats.adaptive_force, DamageType::Magical);
    }
    offensive_stats.adaptive_force = 0.0;
}

pub fn apply_adaptive_damage(adaptive_damage: f64, offensive_stats: &AttackerStats) -> f64 {
    if offensive_stats.ad_bonus > offensive_stats.ability_power {
        convert_adaptive(adaptive_damage, DamageType::Physical)
    } else if offensive_stats.ad_bonus < offensive_stats.ability_power {
        convert_adaptive(adaptive_damage, DamageType::Magical)
    } else {
        // todo: the default depends on the champion
        // see "adaptiveType"
        convert_adaptive(adaptive_damage, DamageType::Physical)
    }
}

pub fn compile_passive_effects(game_params: &mut GameParams<'_>) {
    // iterate over effects from: items, runes and champions abilities
    let item_effects = game_params
        .items
        .iter()
        .map(|item: &&Item| item.passives.clone());

    let rune_effects = game_params
        .runes
        .iter()
        .map(|rune| rune.passive_effect())
        .filter(|passive_effect| passive_effect.is_some())
        .map(|passive_effect| vec![passive_effect.unwrap()]);

    let champion_passives = game_params
        .abilities
        .iter()
        .map(|spell_data| spell_data.passive_effects.clone());

    let mut passive_effects = item_effects
        .chain(rune_effects)
        .chain(champion_passives)
        .flat_map(|effects| effects)
        .collect_vec();

    game_params.passive_effects.append(&mut passive_effects);
}
