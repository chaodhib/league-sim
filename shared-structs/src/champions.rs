use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Champion {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub title: String,
    #[serde(rename(deserialize = "fullName"))]
    pub full_name: String,
    pub icon: String,
    pub resource: String,
    #[serde(rename(deserialize = "attackType"))]
    pub attack_type: String,
    #[serde(rename(deserialize = "adaptiveType"))]
    pub adaptive_type: String,
    pub stats: Stats,
    pub abilities: Abilities,
    #[serde(rename(deserialize = "patchLastChanged"))]
    pub patch_last_changed: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub health: Health,
    #[serde(rename(deserialize = "healthRegen"))]
    pub health_regen: HealthRegen,
    pub mana: Mana,
    #[serde(rename(deserialize = "manaRegen"))]
    pub mana_regen: ManaRegen,
    pub armor: Armor,
    #[serde(rename(deserialize = "magicResistance"))]
    pub magic_resistance: MagicResistance,
    #[serde(rename(deserialize = "attackDamage"))]
    pub attack_damage: AttackDamage,
    pub movespeed: Movespeed,
    #[serde(rename(deserialize = "acquisitionRadius"))]
    pub acquisition_radius: AcquisitionRadius,
    #[serde(rename(deserialize = "selectionRadius"))]
    pub selection_radius: SelectionRadius,
    #[serde(rename(deserialize = "pathingRadius"))]
    pub pathing_radius: PathingRadius,
    #[serde(rename(deserialize = "gameplayRadius"))]
    pub gameplay_radius: GameplayRadius,
    #[serde(rename(deserialize = "criticalStrikeDamage"))]
    pub critical_strike_damage: CriticalStrikeDamage,
    #[serde(rename(deserialize = "criticalStrikeDamageModifier"))]
    pub critical_strike_damage_modifier: CriticalStrikeDamageModifier,
    #[serde(rename(deserialize = "attackSpeed"))]
    pub attack_speed: AttackSpeed,
    #[serde(rename(deserialize = "attackSpeedRatio"))]
    pub attack_speed_ratio: AttackSpeedRatio,
    #[serde(rename(deserialize = "attackCastTime"))]
    pub attack_cast_time: AttackCastTime,
    #[serde(rename(deserialize = "attackTotalTime"))]
    pub attack_total_time: AttackTotalTime,
    #[serde(rename(deserialize = "attackDelayOffset"))]
    pub attack_delay_offset: AttackDelayOffset,
    #[serde(rename(deserialize = "attackRange"))]
    pub attack_range: AttackRange,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Health {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: i64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthRegen {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mana {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: i64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManaRegen {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Armor {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MagicResistance {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackDamage {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Movespeed {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcquisitionRadius {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionRadius {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathingRadius {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameplayRadius {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticalStrikeDamage {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticalStrikeDamageModifier {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackSpeed {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackSpeedRatio {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackCastTime {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackTotalTime {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackDelayOffset {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackRange {
    pub flat: i64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: i64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Abilities {
    #[serde(rename(deserialize = "P"))]
    pub p: Vec<Ability>,
    #[serde(rename(deserialize = "Q"))]
    pub q: Vec<Ability>,
    #[serde(rename(deserialize = "W"))]
    pub w: Vec<Ability>,
    #[serde(rename(deserialize = "E"))]
    pub e: Vec<Ability>,
    #[serde(rename(deserialize = "R"))]
    pub r: Vec<Ability>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    pub description: String,
    pub leveling: Vec<Leveling>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ability {
    pub name: String,
    pub icon: String,
    pub effects: Vec<Effect>,
    pub cost: Option<Cost>,
    pub cooldown: Option<Cooldown>,
    pub targeting: String,
    pub affects: String,
    pub spellshieldable: Option<String>,
    pub resource: Option<String>,
    #[serde(rename(deserialize = "damageType"))]
    pub damage_type: Option<String>,
    #[serde(rename(deserialize = "spellEffects"))]
    pub spell_effects: Option<String>,
    pub projectile: Option<String>,
    // #[serde(rename(deserialize = "onHitEffects"))]
    // pub on_hit_effects: Value,
    // pub occurrence: Value,
    pub notes: String,
    pub blurb: String,
    // #[serde(rename(deserialize = "missileSpeed"))]
    // pub missile_speed: Value,
    // #[serde(rename(deserialize = "rechargeRate"))]
    // pub recharge_rate: Value,
    // #[serde(rename(deserialize = "collisionRadius"))]
    // pub collision_radius: Value,
    // #[serde(rename(deserialize = "tetherRadius"))]
    // pub tether_radius: Value,
    // #[serde(rename(deserialize = "onTargetCdStatic"))]
    // pub on_target_cd_static: Value,
    // #[serde(rename(deserialize = "innerRadius"))]
    // pub inner_radius: Value,
    pub speed: Option<String>,
    pub width: Option<String>,
    // pub angle: Value,
    #[serde(rename(deserialize = "castTime"))]
    pub cast_time: Option<String>,
    #[serde(rename(deserialize = "effectRadius"))]
    pub effect_radius: Option<String>,
    #[serde(rename(deserialize = "targetRange"))]
    pub target_range: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Leveling {
    pub attribute: String,
    pub modifiers: Vec<Modifier>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    pub values: Vec<f64>,
    pub units: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cost {
    pub modifiers: Vec<Modifier>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cooldown {
    pub modifiers: Vec<Modifier>,
    #[serde(rename(deserialize = "affectedByCdr"))]
    pub affected_by_cdr: bool,
}
