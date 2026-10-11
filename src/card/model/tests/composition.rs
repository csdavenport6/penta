use crate::card::{
    AbilityDef, AbilityTargetDef, AlternateSpellKind, CardComposition, CardEffectStatus, CardRules,
    CardStructure, CardType, DoubleFacedKind, ObjectPredicateDef, PlayActionKind, PlayRestriction,
    SpellForm, abilities,
};
use crate::{CardPartId, PlayOptionId, TargetSlotId};

#[test]
fn single_composition_derives_one_primary_cast_option() {
    let rules = CardRules::new_instant(mana_cost!("{R}"));
    let composition = CardComposition::single("Test spell", rules);

    assert_eq!(composition.parts.len(), 1);
    assert_eq!(composition.parts[0].id, CardPartId::PRIMARY);
    assert_eq!(composition.parts[0].rules, rules);
    assert!(matches!(
        composition.structure,
        CardStructure::Single {
            main: CardPartId::PRIMARY,
        }
    ));
    assert_eq!(composition.play_options.len(), 1);
    assert_eq!(composition.play_options[0].id, PlayOptionId::DEFAULT);
    assert_eq!(
        composition.play_options[0].action,
        PlayActionKind::CastSpell
    );
    assert_eq!(
        composition.play_options[0].form,
        SpellForm::Part(CardPartId::PRIMARY),
    );
}

#[test]
fn double_faced_kind_controls_whether_the_back_face_is_playable() {
    static FACES: [(&str, CardRules); 2] = [
        ("Front", CardRules::new_instant(mana_cost!("{W}"))),
        ("Back", CardRules::new_land(&[])),
    ];

    let transforming = CardComposition::double_faced(&FACES, DoubleFacedKind::Transforming);
    assert_eq!(transforming.play_options.len(), 1);

    let modal = CardComposition::double_faced(&FACES, DoubleFacedKind::Modal);
    assert_eq!(modal.play_options.len(), 2);
    assert_eq!(modal.play_options[0].action, PlayActionKind::CastSpell);
    assert_eq!(modal.play_options[1].action, PlayActionKind::PlayLand);
    assert_eq!(modal.play_options[1].form, SpellForm::Part(CardPartId(1)),);
}

#[test]
fn alternate_spell_composition_names_both_parts_and_casts_each_for_its_own_cost() {
    static PARTS: [(&str, CardRules); 2] = [
        (
            "Mage",
            CardRules::new_creature(mana_cost!("{3}{U}{U}"), &["Wizard"], 5, 5),
        ),
        (
            "Shock",
            CardRules::new_instant(mana_cost!("{R}")).with_ability(AbilityDef::destroy_target(
                "Destroy target creature.",
                &AbilityTargetDef::exactly_one_permanent(ObjectPredicateDef::HasType(
                    CardType::Creature,
                )),
            )),
        ),
    ];

    let composition = CardComposition::alternate_spell(&PARTS, AlternateSpellKind::Prepare);

    assert_eq!(composition.parts.len(), 2);
    assert_eq!(composition.parts[0].id, CardPartId::PRIMARY);
    assert_eq!(composition.parts[0].name, "Mage");
    assert_eq!(composition.parts[0].rules, PARTS[0].1);
    assert_eq!(composition.parts[1].id, CardPartId(1));
    assert_eq!(composition.parts[1].name, "Shock");
    assert_eq!(composition.parts[1].rules, PARTS[1].1);
    assert_eq!(
        composition.structure,
        CardStructure::AlternateSpell {
            main: CardPartId::PRIMARY,
            alternate: CardPartId(1),
            kind: AlternateSpellKind::Prepare,
        }
    );
    let [main, alternate] = composition.play_options.as_slice() else {
        panic!("each part is one cast option");
    };
    assert_eq!(main.id, PlayOptionId::DEFAULT);
    assert_eq!(main.label, "Mage");
    assert_eq!(main.form, SpellForm::Part(CardPartId::PRIMARY));
    assert_eq!(main.mana_cost, Some(mana_cost!("{3}{U}{U}")));
    assert_eq!(main.effect_status, CardEffectStatus::Implemented);
    assert!(main.targets.is_empty());
    assert_eq!(alternate.id, PlayOptionId(1));
    assert_eq!(alternate.label, "Shock");
    assert_eq!(alternate.form, SpellForm::Part(CardPartId(1)));
    assert_eq!(alternate.mana_cost, Some(mana_cost!("{R}")));
    assert_eq!(alternate.effect_status, CardEffectStatus::Implemented);
    assert_eq!(alternate.targets.len(), 1);
}

#[test]
fn alternate_spell_composition_accepts_a_part_whose_rules_do_not_exist_yet() {
    static UNSUPPORTED_SPELL: [(&str, CardRules); 2] = [
        (
            "Mage",
            CardRules::new_creature(mana_cost!("{1}{U}"), &["Wizard"], 2, 2),
        ),
        ("Unwritten Spell", CardRules::unsupported()),
    ];
    static UNSUPPORTED_PERMANENT: [(&str, CardRules); 2] = [
        ("Unwritten Mage", CardRules::unsupported()),
        ("Spell", CardRules::new_instant(mana_cost!("{R}"))),
    ];

    let spell = CardComposition::alternate_spell(&UNSUPPORTED_SPELL, AlternateSpellKind::Prepare);
    assert_eq!(spell.parts[1].name, "Unwritten Spell");
    assert_eq!(
        spell.play_options[0].effect_status,
        CardEffectStatus::Implemented
    );
    assert_eq!(
        spell.play_options[1].effect_status,
        CardEffectStatus::Unsupported
    );
    assert_eq!(spell.play_options[1].mana_cost, None);

    let permanent =
        CardComposition::alternate_spell(&UNSUPPORTED_PERMANENT, AlternateSpellKind::Prepare);
    assert_eq!(permanent.parts[0].name, "Unwritten Mage");
    assert_eq!(
        permanent.play_options[0].effect_status,
        CardEffectStatus::Unsupported
    );
    assert_eq!(
        permanent.play_options[1].effect_status,
        CardEffectStatus::Implemented
    );
}

#[test]
fn fused_split_composition_combines_targets_in_printed_order() {
    static HALVES: [(&str, CardRules); 2] = [
        (
            "First",
            CardRules::new_instant(mana_cost!("{1}{U}")).with_ability(AbilityDef::destroy_target(
                "Destroy target creature.",
                &AbilityTargetDef::exactly_one_permanent(ObjectPredicateDef::HasType(
                    CardType::Creature,
                )),
            )),
        ),
        (
            "Second",
            CardRules::new_instant(mana_cost!("{1}{R}")).with_ability(AbilityDef::destroy_target(
                "Destroy target artifact.",
                &AbilityTargetDef::exactly_one_permanent(ObjectPredicateDef::HasType(
                    CardType::Artifact,
                )),
            )),
        ),
    ];
    let composition = CardComposition::split(&HALVES, Some(mana_cost!("{2}{U}{R}")));

    assert!(matches!(
        composition.structure,
        CardStructure::Split {
            ref parts,
            fused: Some(PlayOptionId(2)),
        } if parts == &[CardPartId::PRIMARY, CardPartId(1)]
    ));
    let fused = &composition.play_options[2];
    assert_eq!(fused.restriction, PlayRestriction::FromHandOnly);
    assert_eq!(
        fused.form,
        SpellForm::Combined(vec![CardPartId::PRIMARY, CardPartId(1)]),
    );
    assert_eq!(fused.targets.len(), 2);
    assert_eq!(fused.targets[0].id, TargetSlotId(0));
    assert_eq!(fused.targets[1].id, TargetSlotId(1));
}

#[test]
fn optional_additional_costs_are_not_alternative_casts() {
    let rules = CardRules::new_instant(mana_cost!("{G}")).with_ability(abilities::buyback(&[
        crate::CostDef::Mana(mana_cost!("{3}")),
    ]));
    let composition = CardComposition::single("Buyback test", rules);
    let option = &composition.play_options[0];

    assert!(option.alternative_costs.is_empty());
    assert_eq!(option.additional_costs.len(), 1);
    assert_eq!(option.additional_costs[0].label, "Buyback");
    assert_eq!(
        option.additional_costs[0].mana_cost,
        Some(mana_cost!("{3}"))
    );
}
