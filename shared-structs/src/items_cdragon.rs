use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemDataCdragon {
    // pub epicness: Option<i64>,
    #[serde(rename(deserialize = "itemID"))]
    pub item_id: i64,
    // #[serde(rename(deserialize = "mAbilityHasteMod"))]
    // pub m_ability_haste_mod: Option<i64>,
    // #[serde(rename(deserialize = "mFlatPhysicalDamageMod"))]
    // pub m_flat_physical_damage_mod: Option<i64>,
    // #[serde(rename(deserialize = "mFlatArmorMod"))]
    // pub m_flat_armor_mod: Option<i64>,
    // #[serde(rename(deserialize = "mFlatMovementSpeedMod"))]
    // pub m_flat_movement_speed_mod: Option<i64>,
    #[serde(rename(deserialize = "mItemGroups"))]
    pub m_item_groups: Vec<String>,
    // #[serde(rename(deserialize = "mPercentArmorPenetrationMod"))]
    // pub m_percent_armor_penetration_mod: Option<f64>,
    // #[serde(rename(deserialize = "mFlatCritChanceMod"))]
    // pub m_flat_crit_chance_mod: Option<f64>,
    // #[serde(rename(deserialize = "PhysicalLethality"))]
    // pub physical_lethality: Option<i64>,
    // #[serde(rename(deserialize = "mPercentAttackSpeedMod"))]
    // pub m_percent_attack_speed_mod: Option<f64>,
    // #[serde(rename(deserialize = "mPercentMovementSpeedMod"))]
    // pub m_percent_movement_speed_mod: Option<f64>,
}
