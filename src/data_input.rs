use std::collections::HashMap;

use abilities::{pull_abilities_data, SpellData};
use champions::{get_base_champion_stats, ChampionStats};
use items::{pull_items_data, Item};

pub mod abilities;
pub mod champions;
pub mod common;
pub mod items;

pub struct StaticData {
    pub items_map: HashMap<u64, Item>,
    pub base_champion_stats: ChampionStats,
    pub abilities: Vec<SpellData>,
}

pub fn parse_files(item_ids: &[u64]) -> StaticData {
    StaticData {
        items_map: pull_items_data(item_ids),
        base_champion_stats: get_base_champion_stats(),
        abilities: pull_abilities_data(),
    }
}
