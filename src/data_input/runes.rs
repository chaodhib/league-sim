use super::common::{self, AttackerStats, Aura, GameParams, PassiveEffect};
use crate::{
    attack::AttackType,
    simulation::{self, State},
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Rune {
    DarkHarvest,
    SuddenImpact,
    EyeballCollection,
    AbsoluteFocus,
    GatheringStorm,
    AdaptiveForce1,
    AdaptiveForce2,
    AttackSpeed,
    AbilityHaste,
}

impl Rune {
    fn offensive_stats(
        &self,
        state: &State<'_>,
        game_params: &GameParams<'_>,
    ) -> Option<AttackerStats> {
        // the following code could maybe be replaced by something generic using reflection?
        // step one, look for the struct based on the enum value
        // call function 'offensive_stats' if the struct implements it
        match self {
            Rune::DarkHarvest => None,
            Rune::SuddenImpact => None,
            Rune::EyeballCollection => Some(
                game_params
                    .runes_data
                    .eyeball_collection
                    .offensive_stats(state, game_params),
            ),
            Rune::AbsoluteFocus => Some(
                game_params
                    .runes_data
                    .absolute_focus
                    .offensive_stats(state, game_params),
            ),
            Rune::GatheringStorm => Some(
                game_params
                    .runes_data
                    .gathering_storm
                    .offensive_stats(state, game_params),
            ),
            Rune::AdaptiveForce1 => Some(AttackerStats {
                adaptive_force: 9.0,
                ..Default::default()
            }),
            Rune::AdaptiveForce2 => Some(AttackerStats {
                adaptive_force: 9.0,
                ..Default::default()
            }),
            Rune::AttackSpeed => Some(AttackerStats {
                attack_speed_bonus: 0.1,
                ..Default::default()
            }),
            Rune::AbilityHaste => Some(AttackerStats {
                ability_haste: 8.0,
                ..Default::default()
            }),
        }
    }

    pub fn passive_effect(&self) -> Option<PassiveEffect> {
        match self {
            Rune::DarkHarvest => Some(PassiveEffect::DarkHarvest),
            Rune::SuddenImpact => Some(PassiveEffect::SuddenImpact),

            Rune::EyeballCollection => None,
            Rune::AbsoluteFocus => None,
            Rune::GatheringStorm => None,
            Rune::AdaptiveForce1 => None,
            Rune::AdaptiveForce2 => None,
            Rune::AttackSpeed => None,
            Rune::AbilityHaste => None,
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
            Rune::DarkHarvest => {
                game_params.runes_data.dark_harvest.handle_on_post_damage(
                    damage,
                    attacker_stats,
                    state,
                    game_params,
                    event,
                    events,
                );
            }

            Rune::SuddenImpact => {
                game_params.runes_data.sudden_impact.handle_on_post_damage(
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

    pub(crate) fn handle_dash_event(
        &self,
        event: &simulation::Event,
        events: &mut std::collections::BinaryHeap<simulation::Event>,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
    ) {
        match self {
            Rune::SuddenImpact => {
                game_params.runes_data.sudden_impact.handle_dash_event(
                    event,
                    events,
                    state,
                    game_params,
                );
            }

            _ => (),
        }
    }

    pub(crate) fn handle_stealth_exit_event(
        &self,
        event: &simulation::Event,
        events: &mut std::collections::BinaryHeap<simulation::Event>,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
    ) {
        match self {
            Rune::SuddenImpact => {
                game_params
                    .runes_data
                    .sudden_impact
                    .handle_stealth_exit_event(event, events, state, game_params);
            }

            _ => (),
        }
    }
}

struct DarkHarvest {
    hp_perc_threshold: f64,
    base_damage: f64,
    damage_per_soul: f64,
    bonus_ad: f64,
    bonus_ap: f64,
    cooldown: u64,
}

impl DarkHarvest {
    fn handle_on_post_damage(
        &self,
        _damage: f64,
        attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &crate::simulation::Event,
        events: &mut std::collections::BinaryHeap<crate::simulation::Event>,
    ) {
        let target_stats = common::compute_target_stats(game_params, state);
        // check if it is in cooldown & hp requirement
        let current_hp = target_stats.current_health / target_stats.max_health * 100.0;
        if current_hp >= self.hp_perc_threshold
            || state
                .effects_cooldowns
                .get(&PassiveEffect::DarkHarvest)
                .is_some()
        {
            return;
        }

        simulation::insert_passive_triggered_event(
            events,
            event.time_ms,
            PassiveEffect::DarkHarvest,
        );

        // fetch the current of stacks
        let stack_count = state
            .config
            .get("RUNE_DARK_HARVEST_STACKS")
            .unwrap_or(
                game_params
                    .initial_config
                    .get("RUNE_DARK_HARVEST_STACKS")
                    .unwrap_or(&"0".to_string()),
            )
            .parse::<u64>()
            .unwrap();

        // trigger the damage
        let adaptive_damage = self.base_damage
            + self.damage_per_soul * stack_count as f64
            + self.bonus_ad * attacker_stats.ad_bonus
            + self.bonus_ap * attacker_stats.ability_power;

        let damage = common::apply_adaptive_damage(adaptive_damage, attacker_stats);
        simulation::on_damage_from_rune(&damage, state, event, Rune::DarkHarvest);

        // set new stacks value
        state.config.insert(
            "RUNE_DARK_HARVEST_STACKS".to_string(),
            (stack_count + 1).to_string(),
        );

        // set cooldown
        state
            .effects_cooldowns
            .insert(PassiveEffect::DarkHarvest, event.time_ms + self.cooldown);
    }
}

pub struct SuddenImpact {
    min_damage: f64,
    max_damage: f64,
    buff_duration: u64,
    cooldown: u64,
}
impl SuddenImpact {
    fn handle_dash_event(
        &self,
        event: &simulation::Event,
        events: &mut std::collections::BinaryHeap<simulation::Event>,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
    ) {
        // check if it is in cooldown
        if state
            .effects_cooldowns
            .get(&PassiveEffect::SuddenImpact)
            .is_some_and(|cooldown_end| event.time_ms < *cooldown_end)
        {
            return;
        }

        simulation::insert_passive_triggered_event(
            events,
            event.time_ms,
            PassiveEffect::SuddenImpact,
        );

        // apply buff
        state.add_attacker_aura(
            Aura::SuddenImpactReady,
            event.time_ms + self.buff_duration,
            game_params,
            event,
            events,
        );
    }

    pub fn handle_stealth_exit_event(
        &self,
        event: &simulation::Event,
        events: &mut std::collections::BinaryHeap<simulation::Event>,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
    ) {
        self.handle_dash_event(event, events, state, game_params);
    }

    pub fn handle_on_post_damage(
        &self,
        _damage: f64,
        _attacker_stats: &AttackerStats,
        state: &mut State<'_>,
        game_params: &GameParams<'_>,
        event: &simulation::Event,
        events: &mut std::collections::BinaryHeap<simulation::Event>,
    ) {
        if !state
            .attacker_auras
            .get(&Aura::SuddenImpactReady)
            .is_some_and(|end| state.time_ms < *end)
        {
            panic!();
        }

        // trigger the damage
        let true_damage: f64 = self.min_damage
            + (self.max_damage - self.min_damage) / 17.0 * (game_params.level as f64 - 1.0);

        simulation::on_damage_from_rune(&true_damage, state, event, Rune::SuddenImpact);

        // remove buff
        state.remove_attacker_aura(&Aura::SuddenImpactReady, game_params, event, events);

        // set cooldown
        state
            .effects_cooldowns
            .insert(PassiveEffect::SuddenImpact, state.time_ms + self.cooldown);
    }
}

struct EyeballCollection {
    per_stack_bonus: f64,
    max_stack: u64,
    max_stack_bonus: f64,
}

impl EyeballCollection {
    fn offensive_stats(&self, _state: &State<'_>, game_params: &GameParams<'_>) -> AttackerStats {
        let stack_count: u64 = game_params
            .initial_config
            .get("RUNE_EYEBALL_COLLECTION_STACKS")
            .unwrap_or(&"0".to_string())
            .parse::<u64>()
            .unwrap();
        let mut adaptive_force: f64 = self.per_stack_bonus * (stack_count as f64);
        if stack_count >= self.max_stack {
            adaptive_force += self.max_stack_bonus;
        }

        return AttackerStats {
            adaptive_force,
            ..Default::default()
        };
    }
}

struct AbsoluteFocus {
    hp_perc_threshold: f64,
    min_damage: f64,
    max_damage: f64,
}

impl AbsoluteFocus {
    fn offensive_stats(&self, _state: &State<'_>, game_params: &GameParams<'_>) -> AttackerStats {
        let adaptive_force: f64 = if game_params.attacker_hp_perc > self.hp_perc_threshold {
            self.min_damage
                + (self.max_damage - self.min_damage) / 17.0 * (game_params.level as f64 - 1.0)
        } else {
            0.0
        };

        return AttackerStats {
            adaptive_force,
            ..Default::default()
        };
    }
}

struct GatheringStorm {
    coefficient: f64,
}

impl GatheringStorm {
    fn offensive_stats(&self, state: &State<'_>, _game_params: &GameParams<'_>) -> AttackerStats {
        let x: u64 = 1 + state.time_ms / 600_000;
        return AttackerStats {
            adaptive_force: self.coefficient * ((x * (x - 1)) as f64),
            ..Default::default()
        };
    }
}

pub struct RunesData {
    pub dark_harvest: DarkHarvest,
    pub sudden_impact: SuddenImpact,
    pub eyeball_collection: EyeballCollection,
    pub absolute_focus: AbsoluteFocus,
    pub gathering_storm: GatheringStorm,
}

pub fn pull_runes() -> RunesData {
    // let file = File::open("source_1/perks_formated.json").unwrap();
    // let reader: BufReader<File> = BufReader::new(file);
    // let runes_data: Vec<Value> = serde_json::from_reader(reader).unwrap();

    // Dark Harvest
    // let dark_harvest_data = runes_data
    //     .iter()
    //     .find(|&x| x["name"].as_str().unwrap_or_default() == "Dark Harvest");

    // let rune_description = dark_harvest_data.unwrap()["longDesc"].as_str().unwrap();

    // println!("rune_description: {:#?}", rune_description);

    // let re = Regex::new(r"Hello (?<name>\w+)!").unwrap();
    // let Some(caps) = re.captures("Hello Murphy!") else {
    //     println!("no match!");
    //     return;
    // };
    // println!("The name is: {}", &caps["name"]);

    let dark_harvest = DarkHarvest {
        hp_perc_threshold: 50.0,
        base_damage: 20.0,
        damage_per_soul: 9.0,
        bonus_ad: 0.1,
        bonus_ap: 0.05,
        cooldown: 35000,
    };

    let sudden_impact = SuddenImpact {
        min_damage: 20.0,
        max_damage: 80.0,
        buff_duration: 4000,
        cooldown: 10_000,
    };

    let eyeball_collection = EyeballCollection {
        per_stack_bonus: 2.0,
        max_stack: 10,
        max_stack_bonus: 10.0,
    };

    let absolute_focus = AbsoluteFocus {
        hp_perc_threshold: 70.0,
        min_damage: 3.0,
        max_damage: 30.0,
    };

    let gathering_storm = GatheringStorm { coefficient: 4.0 };

    return RunesData {
        dark_harvest,
        sudden_impact,
        eyeball_collection,
        absolute_focus,
        gathering_storm,
    };
}

pub fn collect_runes_stats(state: &State, game_params: &GameParams) -> AttackerStats {
    let mut offensive_stats = AttackerStats {
        ..Default::default()
    };

    for selected_rune in game_params.runes.iter() {
        let rune_stats = selected_rune.offensive_stats(state, game_params);
        if rune_stats.is_some() {
            offensive_stats += rune_stats.unwrap();
        }
    }

    return offensive_stats;
}
