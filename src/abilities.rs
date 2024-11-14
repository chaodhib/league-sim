use std::{collections::HashMap, fs::File, io::BufReader};

use serde_json::Value;

#[derive(Debug)]
pub struct SpellData {
    pub key: String,
    pub coefficient_ad: f64,
    pub coefficient_ap: f64,
    pub ad_damage: HashMap<u64, f64>,
    pub ap_damage: HashMap<u64, f64>,
    pub variation_name: Option<String>,
}

// fn has_desireable_stats(item: &Value) -> bool {
//     return (not_nul(&item, "abilityHaste")
//         || not_nul(&item, "armorPenetration")
//         || not_nul(&item, "attackDamage")
//         // || not_nul(&item, "cooldownReduction")
//         || not_nul(&item, "lethality"))
//         && !not_nul(&item, "abilityPower")
//         && !not_nul(&item, "criticalStrikeChance")
//         && !not_nul(&item, "attackSpeed");
// }

// fn not_nul(item: &Value, stat_category: &str) -> bool {
//     let values = item["stats"].clone()[stat_category].clone();
//     match values.as_object() {
//         Some(category) => {
//             for ele in category.iter() {
//                 if ele.1.as_number().unwrap().as_f64() != Some(0.0) {
//                     // println!("item: {:#?}", item);
//                     return true;
//                 }
//             }
//         }
//         None => (),
//     };
//     return false;
// }

pub fn pull_abilities_data() -> Vec<SpellData> {
    let file = File::open("/home/chaodhib/git/lolstaticdata/champions/Khazix.json").unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    let json_input: HashMap<String, Value> = serde_json::from_reader(reader).unwrap();
    let abilities = json_input.get("abilities").unwrap();

    let mut abilities_data = Vec::new();

    // Q (variation 1)
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            abilities["Q"][0]["effects"][1]["leveling"][0]["modifiers"][0]["values"][rank - 1]
                .as_f64()
                .unwrap(),
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage: ad_damage,
        ap_damage: ap_damage,
        coefficient_ad: abilities["Q"][0]["effects"][1]["leveling"][0]["modifiers"][1]["values"][0]
            .as_f64()
            .unwrap()
            * 0.01,
        coefficient_ap: 0.0f64,
        key: "Q".to_string(),
        variation_name: Some("Physical Damage".to_string()),
    });

    // Q (variation 2)
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            abilities["Q"][0]["effects"][1]["leveling"][1]["modifiers"][0]["values"][rank - 1]
                .as_f64()
                .unwrap(),
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage: ad_damage,
        ap_damage: ap_damage,
        coefficient_ad: abilities["Q"][0]["effects"][1]["leveling"][1]["modifiers"][1]["values"][0]
            .as_f64()
            .unwrap()
            * 0.01,
        coefficient_ap: 0.0f64,
        key: "Q".to_string(),
        variation_name: Some("Increased Damage".to_string()),
    });

    // W
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            abilities["W"][0]["effects"][0]["leveling"][0]["modifiers"][0]["values"][rank - 1]
                .as_f64()
                .unwrap(),
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage: ad_damage,
        ap_damage: ap_damage,
        coefficient_ad: abilities["W"][0]["effects"][0]["leveling"][0]["modifiers"][1]["values"][0]
            .as_f64()
            .unwrap()
            * 0.01,
        coefficient_ap: 0.0f64,
        key: "W".to_string(),
        variation_name: None,
    });

    // E
    let mut ad_damage: HashMap<u64, f64> = HashMap::new();
    for rank in 1..=5usize {
        ad_damage.insert(
            rank.try_into().unwrap(),
            abilities["E"][0]["effects"][0]["leveling"][0]["modifiers"][0]["values"][rank - 1]
                .as_f64()
                .unwrap(),
        );
    }

    let ap_damage: HashMap<u64, f64> = HashMap::new();
    abilities_data.push(SpellData {
        ad_damage: ad_damage,
        ap_damage: ap_damage,
        coefficient_ad: abilities["E"][0]["effects"][0]["leveling"][0]["modifiers"][1]["values"][0]
            .as_f64()
            .unwrap()
            * 0.01,
        coefficient_ap: 0.0f64,
        key: "E".to_string(),
        variation_name: None,
    });

    // R
    // not a damage ability

    return abilities_data;
}

pub fn find_ability<'a>(
    abilities: &'a Vec<SpellData>,
    spell_name: &str,
    configs: &HashMap<String, String>,
) -> &'a SpellData {
    // println!("abilities {:#?}", abilities);
    // println!("spell_name {:#?}", spell_name);
    // println!("configs {:#?}", configs);
    let mut variation_name: Option<String> = None;

    if spell_name == "Q" {
        // Khazix's Q
        if configs.get("CHAMPION_KHAZIX_ISOLATED_TARGET").unwrap() == "TRUE" {
            variation_name = Some("Increased Damage".to_string());
        } else {
            variation_name = Some("Physical Damage".to_string());
        }
    }

    let ability: &SpellData;
    if variation_name.is_some() {
        ability = abilities
            .iter()
            .find(|&x| x.key == spell_name.to_string() && x.variation_name == variation_name)
            .unwrap();
    } else {
        ability = abilities
            .iter()
            .find(|&x| x.key == spell_name.to_string())
            .unwrap();
    }

    return ability;
}
