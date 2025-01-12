use std::collections::HashMap;

use abilities::{pull_abilities_data, AbilitiesExtraData, SpellData};
use champions::{get_base_champion_stats, ChampionData, ChampionStats};
use common::Champion;
use items::{pull_items_data, ItemData};
use runes::{pull_runes, RunesData};

pub mod abilities;
pub mod champions;
pub mod common;
pub mod items;
pub mod runes;

pub struct StaticData {
    pub items_map: HashMap<u64, ItemData>,
    pub champion_data: ChampionData,
    pub base_champion_stats: ChampionStats,
    pub abilities: Vec<SpellData>,
    pub abilities_extra_data: AbilitiesExtraData,
    pub runes_data: RunesData,
}

pub fn parse_files(
    champion: Champion,
    item_ids: &[u64],
    config: &HashMap<String, String>,
) -> StaticData {
    let (champion_data, base_champion_stats) = get_base_champion_stats(champion);
    let items_map = pull_items_data(item_ids);
    let (abilities, abilities_extra_data) = pull_abilities_data(config);
    let runes_data = pull_runes();

    // println!("base_champion_stats: {:#?}", base_champion_stats);
    // println!("items_map: {:#?}", items_map);
    // println!("abilities: {:#?}", abilities);

    StaticData {
        items_map,
        champion_data,
        base_champion_stats,
        abilities,
        abilities_extra_data,
        runes_data,
    }
}
