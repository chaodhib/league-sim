use super::{
    champions::{AdaptiveType, AttackType, ChampionStats},
    common::Champion, ChampionData,
};
pub fn get_base_champion_stats(champion: Champion) -> (ChampionData, ChampionStats) {
    let champion_data = match champion {
        Champion::Khazix => {
            ChampionData {
                name: Champion::Khazix,
                id: 121u64,
                key: "Khazix".to_string(),
                attack_type: AttackType::from_str("MELEE"),
                adaptive_type: AdaptiveType::from_str("PHYSICAL_DAMAGE"),
            }
        }
    };
    let champion_data = match champion {
        Champion::Khazix => {
            ChampionStats {
                armor_flat: 32f64,
                armor_per_level: 4.2f64,
                attack_damage_flat: 60f64,
                attack_damage_per_level: 3.1f64,
                attack_speed_flat: 0.668f64,
                attack_speed_per_level: 0.027000000000000003f64,
                attack_speed_ratio: 0.667999982833862f64,
                attack_delay_offset: -0.0994652435183525f64,
                attack_cast_time: 0.3f64,
                attack_total_time: 1.6f64,
                base_movement_speed: 350f64,
            }
        }
    };
    (champion_data, champion_stats)
}
