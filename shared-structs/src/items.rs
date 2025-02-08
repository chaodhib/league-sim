use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSourceData {
    pub name: String,
    pub id: i64,
    pub tier: i64,
    pub rank: Vec<String>,
    // #[serde(rename(deserialize = "buildsFrom"))]
    // pub builds_from: Vec<i64>,
    // #[serde(rename(deserialize = "buildsInto"))]
    // pub builds_into: Vec<Value>,
    // #[serde(rename(deserialize = "specialRecipe"))]
    // pub special_recipe: i64,
    // #[serde(rename(deserialize = "noEffects"))]
    // pub no_effects: bool,
    pub removed: bool,
    // #[serde(rename(deserialize = "requiredChampion"))]
    // pub required_champion: String,
    // #[serde(rename(deserialize = "requiredAlly"))]
    // pub required_ally: String,
    pub icon: String,
    // #[serde(rename(deserialize = "simpleDescription"))]
    // pub simple_description: String,
    // pub nicknames: Vec<Value>,
    pub passives: Vec<Passive>,
    pub active: Vec<Active>,
    pub stats: Option<Stats>,
    pub shop: Shop,
    // #[serde(rename(deserialize = "iconOverlay"))]
    // pub icon_overlay: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Passive {
    pub unique: bool,
    pub mythic: bool,
    pub name: Option<String>,
    pub effects: String,
    pub range: Option<u64>,
    pub cooldown: Option<String>,
    pub stats: Option<Stats>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    #[serde(rename(deserialize = "abilityPower"))]
    pub ability_power: Option<StatDetails>,
    pub armor: Option<StatDetails>,
    #[serde(rename(deserialize = "armorPenetration"))]
    pub armor_penetration: Option<StatDetails>,
    #[serde(rename(deserialize = "attackDamage"))]
    pub attack_damage: Option<StatDetails>,
    #[serde(rename(deserialize = "attackSpeed"))]
    pub attack_speed: Option<StatDetails>,
    #[serde(rename(deserialize = "cooldownReduction"))]
    pub cooldown_reduction: Option<StatDetails>,
    #[serde(rename(deserialize = "criticalStrikeChance"))]
    pub critical_strike_chance: Option<StatDetails>,
    #[serde(rename(deserialize = "goldPer10"))]
    pub gold_per10: Option<StatDetails>,
    #[serde(rename(deserialize = "healAndShieldPower"))]
    pub heal_and_shield_power: Option<StatDetails>,
    pub health: Option<StatDetails>,
    #[serde(rename(deserialize = "healthRegen"))]
    pub health_regen: Option<StatDetails>,
    pub lethality: Option<StatDetails>,
    pub lifesteal: Option<StatDetails>,
    #[serde(rename(deserialize = "magicPenetration"))]
    pub magic_penetration: Option<StatDetails>,
    #[serde(rename(deserialize = "magicResistance"))]
    pub magic_resistance: Option<StatDetails>,
    pub mana: Option<StatDetails>,
    #[serde(rename(deserialize = "manaRegen"))]
    pub mana_regen: Option<StatDetails>,
    pub movespeed: Option<StatDetails>,
    #[serde(rename(deserialize = "abilityHaste"))]
    pub ability_haste: Option<StatDetails>,
    pub omnivamp: Option<StatDetails>,
    pub tenacity: Option<StatDetails>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatDetails {
    pub flat: f64,
    pub percent: f64,
    #[serde(rename(deserialize = "perLevel"))]
    pub per_level: f64,
    #[serde(rename(deserialize = "percentPerLevel"))]
    pub percent_per_level: f64,
    #[serde(rename(deserialize = "percentBase"))]
    pub percent_base: f64,
    #[serde(rename(deserialize = "percentBonus"))]
    pub percent_bonus: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Active {
    pub unique: bool,
    pub name: Option<String>,
    pub effects: String,
    pub range: Option<u64>,
    pub cooldown: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shop {
    pub prices: Option<Prices>,
    pub purchasable: bool,
    // pub tags: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prices {
    pub total: i64,
    // pub combined: i64,
    // pub sell: i64,
}
