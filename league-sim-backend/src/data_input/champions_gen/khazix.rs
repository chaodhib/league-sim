Champion {id: 121i64,key: "Khazix".into(),name: "Kha\'Zix".into(),title: "the Voidreaver".into(),full_name: "".into(),icon: "http://ddragon.leagueoflegends.com/cdn/15.4.1/img/champion/Khazix.png".into(),resource: "MANA".into(),attack_type: "MELEE".into(),adaptive_type: "PHYSICAL_DAMAGE".into(),stats: Stats {health: Health {flat: 643i64,percent: 0f64,per_level: 99i64,percent_per_level: 0f64},health_regen: HealthRegen {flat: 7.5f64,percent: 0f64,per_level: 0.75f64,percent_per_level: 0f64},mana: Mana {flat: 327i64,percent: 0f64,per_level: 40i64,percent_per_level: 0f64},mana_regen: ManaRegen {flat: 7.59f64,percent: 0f64,per_level: 0.5f64,percent_per_level: 0f64},armor: Armor {flat: 32i64,percent: 0f64,per_level: 4.2f64,percent_per_level: 0f64},magic_resistance: MagicResistance {flat: 32i64,percent: 0f64,per_level: 2.05f64,percent_per_level: 0f64},attack_damage: AttackDamage {flat: 60i64,percent: 0f64,per_level: 3.1f64,percent_per_level: 0f64},movespeed: Movespeed {flat: 350i64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},acquisition_radius: AcquisitionRadius {flat: 400i64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},selection_radius: SelectionRadius {flat: 130i64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},pathing_radius: PathingRadius {flat: 35i64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},gameplay_radius: GameplayRadius {flat: 65i64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},critical_strike_damage: CriticalStrikeDamage {flat: 175i64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},critical_strike_damage_modifier: CriticalStrikeDamageModifier {flat: 1f64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},attack_speed: AttackSpeed {flat: 0.668f64,percent: 0f64,per_level: 2.7f64,percent_per_level: 0f64},attack_speed_ratio: AttackSpeedRatio {flat: 0.667999982833862f64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},attack_cast_time: AttackCastTime {flat: 0.3f64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},attack_total_time: AttackTotalTime {flat: 1.6f64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},attack_delay_offset: AttackDelayOffset {flat: -0.0994652435183525f64,percent: 0f64,per_level: 0f64,percent_per_level: 0f64},attack_range: AttackRange {flat: 125i64,percent: 0f64,per_level: 0i64,percent_per_level: 0f64}},abilities: Abilities {p: vec![Ability {name: "Unseen Threat".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/p".into(),effects: vec![Effect {description: "Innate: Kha\'Zix gains Unseen Threat whenever the enemy loses sight of him or he activates Void Assault. Unseen Threat: Kha\'Zix empowers his next basic attack against an enemy champion to deal 14 : 116 (based on level) (+ 40% bonus AD) bonus magic damage and slow them by 25% for 2 seconds.".into(),leveling: vec![].into_iter().collect()}].into_iter().collect(),cost: None,cooldown: None,targeting: "Passive".into(),affects: "Enemies".into(),spellshieldable: Some("False".into()),resource: None,damage_type: Some("MAGIC_DAMAGE".into()),spell_effects: Some("spell".into()),projectile: None,notes: "Any form of vision loss may trigger Unseen Threat, such as  Curse of the Black Mist,  nearsight and  Brushmaker.\n Void Assault grants Unseen Threat even if Kha\'Zix never becomes unseen (e.g. affected by  true sight).\nOther  stealth such as  Senna\'s  Curse of the Black Mist does not do this.\nBoth the attack\'s damage and bonus spell damage are grouped under the same Spell ID.\nBecause of this, a single Unseen Threat attack does not trigger two  Electrocute stacks.".into(),blurb: "Innate:  Kha\'Zix gains Unseen Threat whenever the enemy loses  sight of him causing his next basic attack against an enemy Champion to deal bonus magic damage and slow.".into(),speed: None,width: None,cast_time: None,effect_radius: None,target_range: None}].into_iter().collect(),q: vec![Ability {name: "Taste Their Fear".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/q".into(),effects: vec![Effect {description: "Passive: Kha\'Zix considers any enemy unit to be Isolated if they are not nearby to one of their allies. Taste Their Fear, Evolved Reaper Claws, and Evolved Spike Racks have special interactions against Isolated targets.".into(),leveling: vec![].into_iter().collect()},Effect {description: "Active: Kha\'Zix slashes the target enemy, dealing physical damage, increased by 110% against Isolated targets.".into(),leveling: vec![Leveling {attribute: "Physical Damage".into(),modifiers: vec![Modifier {values: vec![80f64,105f64,130f64,155f64,180f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()},Modifier {values: vec![110f64,110f64,110f64,110f64,110f64].into_iter().collect(),units: vec!["% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into()].into_iter().collect()}].into_iter().collect()},Leveling {attribute: "Increased Damage".into(),modifiers: vec![Modifier {values: vec![168f64,220.5f64,273f64,325.5f64,378f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()},Modifier {values: vec![231f64,231f64,231f64,231f64,231f64].into_iter().collect(),units: vec!["% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into()].into_iter().collect()}].into_iter().collect()}].into_iter().collect()}].into_iter().collect(),cost: Some(Cost {modifiers: vec![Modifier {values: vec![20f64,20f64,20f64,20f64,20f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()}].into_iter().collect()}),cooldown: Some(Cooldown {modifiers: vec![Modifier {values: vec![4f64,4f64,4f64,4f64,4f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()}].into_iter().collect(),affected_by_cdr: true}),targeting: "Unit".into(),affects: "Enemies".into(),spellshieldable: Some("True".into()),resource: Some("MANA".into()),damage_type: Some("PHYSICAL_DAMAGE".into()),spell_effects: Some("Single target".into()),projectile: None,notes: "A team\'s allies are  champions,  pets,  minions and outer  turrets.\n Monsters are considered allies for other monsters.\n Wards do not prevent Isolation.\nA number of targetable champion summoned units are specifically tagged to not be a valid ally of a potentially Isolated target. These units are:\n Gangplank\'s  Powder Keg\n Illaoi\'s  Prophet of an Elder God\n Jhin\'s  Captive Audience\n Nidalee\'s  Bushwhack\n Rek\'Sai\'s  Tunnel\n Senna\'s  Absolution\'s Mist Wraiths\n Teemo\'s  Noxious Trap\n Viego\'s  Sovereign\'s Domination\'s Mist Wraiths\n Yorick\'s  Dark Procession\nIsolated targets are marked by an  indicator to Kha\'Zix when visible and within 2500 units of him. However, Isolation bonuses are evaluated when the relevant spells hit and thus enemies do not need to show the indicator for them to receive the effects.\nIf the target becomes  untargetable,  dies, or is too far away or no longer in  sight during the cast time, this ability will cancel but does not go on  cooldown nor pay its cost (if applicable).".into(),blurb: "Passive:  Kha\'Zix considers any enemy unit to be Isolated if they are not nearby to one of their allies. His abilities have special interactions against Isolated targets.".into(),speed: None,width: None,cast_time: Some("0.25".into()),effect_radius: Some("375".into()),target_range: Some("325".into())},Ability {name: "Evolved Reaper Claws".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/q".into(),effects: vec![Effect {description: "Passive: Kha\'Zix gains 50 bonus range on his basic attacks and Taste Their Fear. Evolved Bonus: If the target is Isolated, the cooldown is reduced by 45%.".into(),leveling: vec![].into_iter().collect()}].into_iter().collect(),cost: None,cooldown: None,targeting: "Unit".into(),affects: "Enemies".into(),spellshieldable: Some("True".into()),resource: None,damage_type: Some("PHYSICAL_DAMAGE".into()),spell_effects: Some("Single target".into()),projectile: None,notes: "No additional details.".into(),blurb: "Passive: Kha\'Zix gains  bonus range on his basic attacks and Taste Their Fear.".into(),speed: None,width: None,cast_time: None,effect_radius: None,target_range: Some("375".into())}].into_iter().collect(),w: vec![Ability {name: "Void Spike".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/w".into(),effects: vec![Effect {description: "Active: Kha\'Zix fires a bolt of spikes in the target direction that explodes upon hitting an enemy, dealing physical damage to nearby enemies.".into(),leveling: vec![Leveling {attribute: "Physical Damage".into(),modifiers: vec![Modifier {values: vec![85f64,115f64,145f64,175f64,205f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()},Modifier {values: vec![100f64,100f64,100f64,100f64,100f64].into_iter().collect(),units: vec!["% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into()].into_iter().collect()}].into_iter().collect()}].into_iter().collect()},Effect {description: "Kha\'Zix heals himself if he is within the explosion.".into(),leveling: vec![Leveling {attribute: "Heal".into(),modifiers: vec![Modifier {values: vec![55f64,75f64,95f64,115f64,135f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()},Modifier {values: vec![50f64,50f64,50f64,50f64,50f64].into_iter().collect(),units: vec!["% AP".into(),"% AP".into(),"% AP".into(),"% AP".into(),"% AP".into()].into_iter().collect()}].into_iter().collect()}].into_iter().collect()}].into_iter().collect(),cost: Some(Cost {modifiers: vec![Modifier {values: vec![55f64,60f64,65f64,70f64,75f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()}].into_iter().collect()}),cooldown: Some(Cooldown {modifiers: vec![Modifier {values: vec![9f64,9f64,9f64,9f64,9f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()}].into_iter().collect(),affected_by_cdr: true}),targeting: "Direction".into(),affects: "Enemies, Self".into(),spellshieldable: Some("True".into()),resource: Some("MANA".into()),damage_type: Some("PHYSICAL_DAMAGE".into()),spell_effects: Some("spellaoe".into()),projectile: Some("TRUE".into()),notes: "This ability will cast from wherever the caster is at the end of the cast time.\nVoid Spike\'s effect radius is centered around the location of the missile as it collides.".into(),blurb: "Active:  Kha\'Zix fires a bolt of spikes in the target direction that explodes upon hitting an enemy, dealing physical damage to nearby enemies.".into(),speed: Some("1700".into()),width: Some("140".into()),cast_time: Some("0.25".into()),effect_radius: Some("275".into()),target_range: Some("1025".into())},Ability {name: "Evolved Spike Racks".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/w".into(),effects: vec![Effect {description: "Evolved Bonus: Void Spike now fires three clusters in a cone, slows by 40% and reveals enemy champions hit for 2 seconds. Multiple explosions do not deal extra damage to the same target nor provide Kha\'Zix with additional healing.".into(),leveling: vec![].into_iter().collect()},Effect {description: "Isolated targets hit by Evolved Spike Racks are slowed by 60% instead.".into(),leveling: vec![].into_iter().collect()}].into_iter().collect(),cost: None,cooldown: None,targeting: "Direction".into(),affects: "Enemies, Self".into(),spellshieldable: Some("special".into()),resource: None,damage_type: Some("PHYSICAL_DAMAGE".into()),spell_effects: Some("aoe".into()),projectile: Some("TRUE".into()),notes: "This ability will cast from wherever the caster is at the end of the cast time.\nEvolved Spike Racks\' effect radius is centered around the location of the missile as it collides.\n Spell shield will not block the  reveal.".into(),blurb: "Evolved Bonus: Void Spike now fires three clusters in a cone,  slowing and  revealing enemy champions hit for a short time.".into(),speed: None,width: None,cast_time: None,effect_radius: None,target_range: None}].into_iter().collect(),e: vec![Ability {name: "Leap".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/e".into(),effects: vec![Effect {description: "Active: Kha\'Zix leaps to the target location, dealing physical damage to nearby enemies upon arrival.".into(),leveling: vec![Leveling {attribute: "Physical Damage".into(),modifiers: vec![Modifier {values: vec![65f64,100f64,135f64,170f64,205f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()},Modifier {values: vec![20f64,20f64,20f64,20f64,20f64].into_iter().collect(),units: vec!["% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into(),"% bonus AD".into()].into_iter().collect()}].into_iter().collect()}].into_iter().collect()},Effect {description: "Taste Their Fear can be cast during the dash. Leap will cast at max range if cast beyond that.".into(),leveling: vec![].into_iter().collect()}].into_iter().collect(),cost: Some(Cost {modifiers: vec![Modifier {values: vec![50f64,50f64,50f64,50f64,50f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()}].into_iter().collect()}),cooldown: Some(Cooldown {modifiers: vec![Modifier {values: vec![20f64,18f64,16f64,14f64,12f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into(),"".into(),"".into()].into_iter().collect()}].into_iter().collect(),affected_by_cdr: true}),targeting: "Location".into(),affects: "Enemies".into(),spellshieldable: Some("True".into()),resource: Some("MANA".into()),damage_type: Some("PHYSICAL_DAMAGE".into()),spell_effects: Some("spellaoe".into()),projectile: None,notes: "Kha\'Zix can use his summoner spells and item actives while  leaping.\nBuffering Interactions\n\n Taste Their Fear can be buffered to cast after the  leap ends if it is cast while the target is out of range. There is no check for the target coming in range during the  leap.\nIf the target is still out of range after landing, there will be a 0.5 second delay  before Kha\'Zix starts moving towards the target compared to manually casting  Taste Their Fear after landing.\n Void Spike &  Void Assault can be buffered to cast after the  leap ends.".into(),blurb: "Active:  Kha\'Zix  leaps to the target location, dealing physical damage to nearby enemies upon landing.".into(),speed: None,width: None,cast_time: Some("none".into()),effect_radius: Some("300".into()),target_range: Some("700".into())},Ability {name: "Evolved Wings".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/e".into(),effects: vec![Effect {description: "Evolved Bonus: Leap gains 200 bonus cast range, and the cooldown resets upon scoring a champion takedown.".into(),leveling: vec![].into_iter().collect()}].into_iter().collect(),cost: None,cooldown: None,targeting: "Location".into(),affects: "Enemies".into(),spellshieldable: Some("True".into()),resource: None,damage_type: Some("PHYSICAL_DAMAGE".into()),spell_effects: Some("aoe".into()),projectile: None,notes: "Kha\'Zix can use his summoner spells and item actives while  leaping.\nBuffering Interactions\n\n Taste Their Fear can be buffered to cast after the  leap ends if it is cast while the target is out of range. There is no check for the target coming in range during the  leap.\nIf the target is still out of range after landing, there will be a 0.5 second delay  before Kha\'Zix starts moving towards the target compared to manually casting  Taste Their Fear after landing.\n Void Spike &  Void Assault can be buffered to cast after the  leap ends.".into(),blurb: "Evolved Bonus: Leap has increased range, and the cooldown  resets upon scoring a champion  takedown.".into(),speed: None,width: None,cast_time: None,effect_radius: None,target_range: Some("900".into())}].into_iter().collect(),r: vec![Ability {name: "Void Assault".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/r".into(),effects: vec![Effect {description: "Passive: Each rank in Void Assault allows Kha\'Zix to evolve one of his abilities, granting it additional effects. Evolving an ability causes him to enter a 2-second cast time. Kha\'Zix cannot evolve while he is unable to cast abilities.".into(),leveling: vec![].into_iter().collect()},Effect {description: "Active: Kha\'Zix becomes invisible for 1.25 seconds, during which he gains 40% bonus movement speed.".into(),leveling: vec![].into_iter().collect()},Effect {description: "After 2 seconds of leaving invisibility, and for the next 12 seconds, Void Assault can be recast at no additional cost.".into(),leveling: vec![].into_iter().collect()},Effect {description: "Recast: Kha\'Zix mimics the first cast\'s effects.".into(),leveling: vec![].into_iter().collect()}].into_iter().collect(),cost: Some(Cost {modifiers: vec![Modifier {values: vec![100f64,100f64,100f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into()].into_iter().collect()}].into_iter().collect()}),cooldown: Some(Cooldown {modifiers: vec![Modifier {values: vec![100f64,85f64,70f64].into_iter().collect(),units: vec!["".into(),"".into(),"".into()].into_iter().collect()}].into_iter().collect(),affected_by_cdr: true}),targeting: "Auto".into(),affects: "Self".into(),spellshieldable: None,resource: Some("MANA".into()),damage_type: None,spell_effects: None,projectile: None,notes: "Each cast counts as an ability activation for the purposes of on-cast effects such as  Spellblade and triggering  Force Pulse\'s passive.\nKha\'Zix receives the evolution even if he dies while in cast time.\nEvery time Void Assault is ranked, a secondary menu will pop up for Kha\'Zix to select an ability to evolve, this can only be done once per ability.\nThe only way for Kha\'Zix to evolve all of his abilities is to gain the fourth evolution point by being victorious in The Hunt is On! (by scoring a  takedown on  Rengar).\nUsing a basic attack breaks the stealth at the end of the attack windup.".into(),blurb: "Passive: Each rank in Void Assault allows Kha\'Zix to evolve one of his abilities, granting it additional effects.".into(),speed: None,width: None,cast_time: Some("none".into()),effect_radius: None,target_range: None},Ability {name: "Evolved Adaptive Cloaking".into(),icon: "https://cdn.communitydragon.org/latest/champion/Khazix/ability-icon/r".into(),effects: vec![Effect {description: "Evolved Bonus: The invisibility now lasts 2 seconds, and Void Assault can be recast twice.".into(),leveling: vec![].into_iter().collect()}].into_iter().collect(),cost: None,cooldown: None,targeting: "Auto".into(),affects: "Self".into(),spellshieldable: None,resource: None,damage_type: None,spell_effects: None,projectile: None,notes: "No additional details.".into(),blurb: "Evolved Bonus: The  invisibility is extended, and Void Assault can be cast up to 3 times.".into(),speed: None,width: None,cast_time: None,effect_radius: None,target_range: None}].into_iter().collect()},patch_last_changed: "14.23".into()}