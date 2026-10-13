//! Reality Fracture card inventory.

use super::CardRecord;
use super::PrintingRecord;
use crate::Binding;
use crate::ParentBinding;
use crate::TargetIndex;
use crate::card::AbilityDef;
use crate::card::AbilityPredicateDef;
use crate::card::AbilityTargetDef;
use crate::card::AbilityTargetPredicate;
use crate::card::ActivationTimingDef;
use crate::card::AddManaEffectDef;
use crate::card::AggregateOperationDef;
use crate::card::AlternativeCastKindDef;
use crate::card::AppliedEffectDef;
use crate::card::AppliedRuleDef;
use crate::card::AttackEventMatcherDef;
use crate::card::BattlefieldArrivalDef;
use crate::card::BattlefieldEntryModificationDef;
use crate::card::BattlefieldEntryScalarChoiceDef;
use crate::card::CardArt;
use crate::card::CardRules;
use crate::card::CardSupertype;
use crate::card::CardType;
use crate::card::CardTypeSet;
use crate::card::ChoiceVisibilityDef;
use crate::card::ChooseDef;
use crate::card::ComparisonDef;
use crate::card::ConditionValueDef;
use crate::card::ConditionalStaticEffectDef;
use crate::card::CopyAbilityDef;
use crate::card::CopyExceptionsDef;
use crate::card::CostDef;
use crate::card::CostModificationDef;
use crate::card::CostQuantityDef;
use crate::card::CountConditionDef;
use crate::card::CounterKind;
use crate::card::CreateTokenDef;
use crate::card::CreatedTokensDef;
use crate::card::CreatureTypeSetDef;
use crate::card::DamageAssignmentDef;
use crate::card::DiscardFollowUpDef;
use crate::card::DiscardSelectionDef;
use crate::card::EffectChoiceDef;
use crate::card::EffectDef;
use crate::card::EffectRecipientDef;
use crate::card::EmblemCharacteristics;
use crate::card::IfNoObjectsDef;
use crate::card::InstalledTriggerDef;
use crate::card::InstalledTriggerLifetimeDef;
use crate::card::KeywordAbility;
use crate::card::ManaColor;
use crate::card::ManaRestrictionDef;
use crate::card::ManaTypeDef;
use crate::card::MoveObjectsDef;
use crate::card::ObjectChoiceBindingDef;
use crate::card::ObjectCollectionSourceDef;
use crate::card::ObjectCounterValueDef;
use crate::card::ObjectPredicateDef;
use crate::card::ObjectQueryDef;
use crate::card::ObjectRefDef;
use crate::card::ObjectSetCountConditionDef;
use crate::card::ObjectSetDef;
use crate::card::ObjectSetFilterDef;
use crate::card::ObjectSetPredicateDef;
use crate::card::ObjectValueAggregateDef;
use crate::card::ObjectValueDef;
use crate::card::PayOrDef;
use crate::card::PlayerRefDef;
use crate::card::PlayerRelation;
use crate::card::PlayerRuleDef;
use crate::card::PlayerSetDef;
use crate::card::QuantifierDef;
use crate::card::QuotientValueDef;
use crate::card::RandomizeObjectOrderDef;
use crate::card::ReplacementChoiceDef;
use crate::card::ReplacementEffectDef;
use crate::card::ResolvedEffectDurationDef;
use crate::card::RevealAndClassifyCardsDef;
use crate::card::RoundingDef;
use crate::card::ScaledValueDef;
use crate::card::SpellCastQueryDef;
use crate::card::StaticApplyDef;
use crate::card::SubtypeDef;
use crate::card::SumValueDef;
use crate::card::TokenCharacteristics;
use crate::card::TokenCopyDef;
use crate::card::TokenCountersDef;
use crate::card::TokenDef;
use crate::card::TriggerConditionDef;
use crate::card::TriggerEventDef;
use crate::card::TriggerModificationDef;
use crate::card::TriggerModificationKindDef;
use crate::card::TurnStepDef;
use crate::card::ValueComparisonDef;
use crate::card::ValueDef;
use crate::card::ZoneKind;
use crate::card::ZonePlacement;
use crate::card::abilities;
use crate::mana_cost;

use crate::card::sets::y1993::alpha as catalog_lea;
use crate::card::sets::y2001::odyssey::threshold;
use crate::card::sets::y2005::ravnica_city_of_guilds as catalog_rav;
use crate::card::sets::y2007::future_sight as catalog_fut;
use crate::card::sets::y2016::kaladesh as catalog_kld;
use crate::card::sets::y2018::dominaria as catalog_dom;
use crate::card::sets::y2021::innistrad_midnight_hunt as catalog_mid;
use crate::card::sets::y2023::phyrexia_all_will_be_one as catalog_one;
use crate::card::sets::y2025::aetherdrift::exhaust;
use crate::card::sets::y2026::secrets_of_strixhaven::become_prepared;
use crate::card::sets::y2026::secrets_of_strixhaven::become_unprepared;
use crate::card::sets::y2026::secrets_of_strixhaven::enters_prepared;
use crate::card::sets::y2026::secrets_of_strixhaven::enters_prepared_with_reminder;
use crate::card::sets::y2026::secrets_of_strixhaven::reprepare_at_upkeep;
use crate::card::sets::y2026::secrets_of_strixhaven::reprepare_at_upkeep_with_reminder;

/// Printed set identity and stable catalog slug.
pub const SET: crate::card::CardSet = crate::card::CardSet::new(&crate::card::CardSetMetadata {
    code: "FRA",
    slug: "reality-fracture",
});

pub(in crate::card::sets) const DEFINITION: crate::card::sets::SetDefinition =
    crate::card::sets::SetDefinition::new(SET, CARDS, ADDITIONAL_PRINTINGS, file!());

// Token identity compares the ability list by address, so the one Jace token
// every Empower card creates has to point at this one list.
static JACE_TOKEN_ABILITIES: [AbilityDef; 2] = [
    AbilityDef::activated(
        "−1: Surveil 1.",
        &[CostDef::Loyalty(ValueDef::Constant(-1))],
        abilities::surveil(ValueDef::Constant(1)),
    ),
    AbilityDef::activated(
        "−3: Draw a card.",
        &[CostDef::Loyalty(ValueDef::Constant(-3))],
        EffectDef::DrawCards {
            recipient: EffectRecipientDef::Controller,
            amount: ValueDef::Constant(1),
        },
    ),
];

/// The blue Jace planeswalker token Empower Jace creates. It has no printed
/// loyalty, so it enters with none (CR 701.71a).
const JACE_TOKEN: TokenCharacteristics = TokenCharacteristics::new(
    CardTypeSet::single(CardType::Planeswalker),
    &["Jace"],
    &[ManaColor::Blue],
    None,
)
.with_abilities(&JACE_TOKEN_ABILITIES)
.with_art(CardArt::new(
    "8d1e424f-6407-4f8c-bed2-eaf148237c81",
    "Manuel Castañón",
));

const CADET_TOKEN: TokenCharacteristics =
    TokenCharacteristics::creature(&["Wizard", "Soldier"], &[], 2, 2)
        .with_name("Cadet")
        .with_art(CardArt::new(
            "8f4534d8-2783-484f-8ebf-a47b1cc4c6df",
            "Elizabeth Peiró",
        ));

// One list for the same reason: several cards create this token.
static BEAST_TOKEN_ABILITIES: [AbilityDef; 1] = [abilities::trample()];
const BEAST_TOKEN: TokenCharacteristics =
    TokenCharacteristics::creature(&["Beast"], &[ManaColor::Green], 4, 4)
        .with_abilities(&BEAST_TOKEN_ABILITIES)
        .with_art(CardArt::new(
            "859bda9a-fa90-4ad3-b0c1-6fc62e27c12f",
            "Simon Dominic",
        ));

// Likewise the Thopter that both Pia and Saheeli create.
static THOPTER_TOKEN_ABILITIES: [AbilityDef; 1] = [abilities::flying()];
const THOPTER_TOKEN: TokenCharacteristics =
    TokenCharacteristics::artifact_creature(&["Thopter"], &[], 1, 1)
        .with_abilities(&THOPTER_TOKEN_ABILITIES)
        .with_art(CardArt::new(
            "bfd6132f-c96b-4ce0-ac4d-c46356afc767",
            "Diego Gisbert",
        ));

const HEARTWOOD_TOKEN: TokenCharacteristics = crate::card::tokens::heartwood().with_art(
    CardArt::new("eb4bf635-04d7-4994-9f70-4ff573a767e7", "Tianxing Xu"),
);

const TREASURE_TOKEN: TokenCharacteristics = crate::card::tokens::treasure().with_art(
    CardArt::new("03992f88-7a15-4234-9bed-b7617d7ff09c", "Alayna Danner"),
);

// Each prepare spell below is printed on three cards. One constant apiece
// keeps the copies the same spell. A card that is still unsupported gives
// neither part rules, since an unsupported card exposes no clause; it takes
// its spell's constant when its creature gains rules.

/// The prepare spell of Blossom-Blessed Angel, Emergency Phytomedic, and
/// Vigorbloom Vanguard.
const SEED_SUTURE: CardRules =
    CardRules::new_sorcery(mana_cost!("{G/W}")).with_ability(AbilityDef::spell_with_targets(
        "Put a +1/+1 counter on target creature. You gain 1 life.",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::HasType(CardType::Creature),
        )],
        EffectDef::Sequence(&[
            EffectDef::AddCounters {
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                kind: CounterKind::PlusOnePlusOne,
                amount: ValueDef::Constant(1),
            },
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(1),
            },
        ]),
    ));

/// The prepare spell of Semester Foreseer, Fatehold Chronologist, and, once it
/// is supported, Prudent Fateseer.
const PEER_REVIEW: CardRules =
    CardRules::new_sorcery(mana_cost!("{2}{W/U}")).with_ability(AbilityDef::spell(
        "Create a 2/2 colorless Wizard Soldier creature token named Cadet. Surveil 1.",
        EffectDef::Sequence(&[
            EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(CADET_TOKEN))),
            abilities::surveil(ValueDef::Constant(1)),
        ]),
    ));

/// The prepare spell of Void Extrapolator, Paradox Shaper, and Theorix
/// Metamage.
const OMIT_VARIABLES: CardRules =
    CardRules::new_sorcery(mana_cost!("{U/B}")).with_ability(AbilityDef::spell(
        "Mill three cards. (Put the top three cards of your library into your graveyard.)",
        EffectDef::Mill {
            player: EffectRecipientDef::Controller,
            amount: ValueDef::Constant(3),
        },
    ));

/// The prepare spell of Hallway Heckler, Stingerquill Voxmancer, and, once it
/// is supported, Whiplash Wordsmith.
const VICIOUS_VERSE: CardRules =
    CardRules::new_sorcery(mana_cost!("{B/R}")).with_ability(AbilityDef::spell_with_targets(
        "Vicious Verse deals 1 damage to target opponent.",
        &[AbilityTargetDef::exactly_one(
            AbilityTargetPredicate::Player(PlayerRelation::Opponent),
        )],
        EffectDef::damage(
            EffectRecipientDef::Target(TargetIndex::PRIMARY),
            ValueDef::Constant(1),
        ),
    ));

/// The prepare spell of Konstrari Improviser, Woodwork Prodigy, and, once it
/// is supported, Heartwood Crafter.
const SOUL_TETHER: CardRules =
    CardRules::new_sorcery(mana_cost!("{2}{R/G}")).with_ability(AbilityDef::spell(
        "Create a Heartwood token. (It's a red and green artifact with \"{T}: Add {R} or {G}.\")",
        EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(HEARTWOOD_TOKEN))),
    ));

/// "Creature or planeswalker", as a target or as a group to count over.
const A_CREATURE_OR_PLANESWALKER: ObjectPredicateDef = ObjectPredicateDef::AnyOf(&[
    ObjectPredicateDef::HasType(CardType::Creature),
    ObjectPredicateDef::HasType(CardType::Planeswalker),
]);

/// What basic landcycling searches for.
const A_BASIC_LAND_CARD: ObjectPredicateDef = ObjectPredicateDef::All(&[
    ObjectPredicateDef::HasType(CardType::Land),
    ObjectPredicateDef::Supertype(CardSupertype::Basic),
]);

/// A permanent card, wherever it is: one of the types that can be on the battlefield.
const A_PERMANENT_CARD: ObjectPredicateDef = ObjectPredicateDef::AnyOf(&[
    ObjectPredicateDef::HasType(CardType::Artifact),
    ObjectPredicateDef::HasType(CardType::Creature),
    ObjectPredicateDef::HasType(CardType::Enchantment),
    ObjectPredicateDef::HasType(CardType::Land),
    ObjectPredicateDef::HasType(CardType::Planeswalker),
]);

/// Any Jace planeswalker token you control, however you came by it. A token
/// copy of a nontoken Jace counts; the nontoken Jace does not.
const A_JACE_TOKEN_YOU_CONTROL: ObjectQueryDef = ObjectQueryDef::controlled_by(
    ObjectPredicateDef::All(&[
        ObjectPredicateDef::Token,
        ObjectPredicateDef::HasType(CardType::Planeswalker),
        ObjectPredicateDef::Subtype(SubtypeDef::from_name("Jace")),
    ]),
    &[ZoneKind::Battlefield],
    PlayerSetDef::Related(PlayerRelation::You),
);

const EMPOWER_AMOUNT: Binding = Binding!("empower_jace_amount");
const EMPOWERED_JACE: Binding = Binding!("empowered_jace");

/// Empower Jace N (CR 701.71a): "If you don't control a Jace planeswalker
/// token, create a blue Jace planeswalker token with 0 loyalty, '[−1]:
/// Surveil 1,' and '[−3]: Draw a card.' Choose a Jace planeswalker token you
/// control. Put N loyalty counters on it."
///
/// N is worked out before the token arrives, so "where X is the number of
/// ..." counts the board the instruction started from. A token created here
/// sits at 0 loyalty only until the counters land in the same resolution;
/// state-based actions are not checked in between.
pub(in crate::card::sets) const fn empower_jace(amount: ValueDef) -> EffectDef {
    EffectDef::BindValue {
        binding: EMPOWER_AMOUNT,
        value: amount,
        // A const block rather than a plain reference: the body calls const fns,
        // which a temporary inside a const fn body will not promote to `'static`.
        effect: &const {
            EffectDef::Sequence(&[
                EffectDef::IfCondition {
                    condition: &TriggerConditionDef::ObjectCount {
                        query: A_JACE_TOKEN_YOU_CONTROL,
                        comparison: ComparisonDef::Equal,
                        amount: 0,
                    },
                    then: &EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                        JACE_TOKEN,
                    ))),
                },
                EffectDef::Choose(ChooseDef {
                    binding: ObjectChoiceBindingDef::Object(EMPOWERED_JACE),
                    unchosen: None,
                    chooser: PlayerRefDef::EffectController,
                    candidates: ObjectSetDef::Query(A_JACE_TOKEN_YOU_CONTROL),
                    exclude: None,
                    minimum: 1,
                    maximum: 1,
                    visibility: ChoiceVisibilityDef::Public,
                    then: &EffectDef::AddCounters {
                        object: EffectRecipientDef::object(ObjectRefDef::Binding(EMPOWERED_JACE)),
                        kind: CounterKind::Loyalty,
                        amount: ValueDef::BoundValue(EMPOWER_AMOUNT),
                    },
                }),
            ])
        },
    }
}

/// "Planeswalkers you control have '[cost]: ...'". The grant is an ordinary
/// static ability-adding effect, so the loyalty ability it adds shares its
/// planeswalker's one activation each turn with the printed ones (CR 606.3).
const fn planeswalkers_you_control_have(
    text: &'static str,
    ability: &'static AbilityDef,
) -> AbilityDef {
    AbilityDef::static_ability(
        text,
        EffectDef::StaticApply {
            recipient: EffectRecipientDef::matching_objects(
                ObjectPredicateDef::HasType(CardType::Planeswalker),
                &[ZoneKind::Battlefield],
                PlayerRelation::You,
            ),
            effect: AppliedEffectDef::add_ability(ability),
        },
    )
}

/// The second half of both Teyos' entry clause: "Put a +1/+1 counter on it if
/// it's a creature. Put a loyalty counter on it if it's a planeswalker." A
/// permanent that is somehow both gets both kinds.
const TEYO_S_COUNTERS: EffectDef = EffectDef::Sequence(&[
    EffectDef::IfCondition {
        condition: &TriggerConditionDef::TargetMatches {
            slot: TargetIndex::PRIMARY,
            object: ObjectPredicateDef::HasType(CardType::Creature),
        },
        then: &EffectDef::AddCounters {
            object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
            kind: CounterKind::PlusOnePlusOne,
            amount: ValueDef::Constant(1),
        },
    },
    EffectDef::IfCondition {
        condition: &TriggerConditionDef::TargetMatches {
            slot: TargetIndex::PRIMARY,
            object: ObjectPredicateDef::HasType(CardType::Planeswalker),
        },
        then: &EffectDef::AddCounters {
            object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
            kind: CounterKind::Loyalty,
            amount: ValueDef::Constant(1),
        },
    },
]);

/// The entry clause of the set's ten two-color lands. It is answered as the
/// land enters, so a planeswalker arriving alongside it does not count.
const fn enters_tapped_unless_you_control_a_planeswalker() -> AbilityDef {
    abilities::enters_tapped_unless_you_control(
        "This land enters tapped unless you control a planeswalker.",
        ObjectPredicateDef::HasType(CardType::Planeswalker),
    )
}

// FRA 1 — Emrakul, the Exigent Doom
// Audit: unsupported — Needs an activation from hand that exiles the card as a cost, and a granted
// land ability lasting until that card is cast from exile.
pub(in crate::card::sets) static EMRAKUL_THE_EXIGENT_DOOM: CardRecord = CardRecord::new(
    "Emrakul, the Exigent Doom",
    "c3ff8dd3-88a8-49dc-a59b-e2748680623c",
    "Cristi Balanescu",
    CardRules::unsupported(),
);

// FRA 2 — Academic Ascent
pub(in crate::card::sets) static ACADEMIC_ASCENT: CardRecord = CardRecord::new(
    "Academic Ascent",
    "730d8c28-1e58-4b8e-89e9-445d154d2e83",
    "Randy Gallegos",
    CardRules::new_instant(mana_cost!("{1}{W}")).with_abilities(&[
        AbilityDef::spell_with_targets(
            "Target creature gets +2/+2 and gains flying until end of turn.",
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::HasType(CardType::Creature),
            )],
            EffectDef::Apply {
                recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                effect: AppliedEffectDef::Composite(&[
                    AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(2),
                        ValueDef::Constant(2),
                    ),
                    AppliedEffectDef::add_ability(&abilities::flying()),
                ]),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
        ),
        AbilityDef::spell(
            "Empower Jace 2. (Put two loyalty counters on a Jace token you control. If you \
             don't control one, first create a blue Jace planeswalker token with \
             \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(2)),
        ),
    ]),
);

// FRA 3 — Blossom-Blessed Angel // Seed Suture
pub(in crate::card::sets) static BLOSSOM_BLESSED_ANGEL: CardRecord = CardRecord::new_prepare(
    "Blossom-Blessed Angel // Seed Suture",
    "5e77fbf0-9d1f-4e7d-a02e-43065d11b0d9",
    "Joshua Raphael",
    &[
        (
            "Blossom-Blessed Angel",
            CardRules::new_creature(mana_cost!("{3}{W}"), &["Angel", "Cleric"], 2, 4)
                .with_abilities(&[
                    abilities::flying(),
                    abilities::vigilance(),
                    enters_prepared_with_reminder(),
                ]),
        ),
        ("Seed Suture", SEED_SUTURE),
    ],
);

// FRA 4 — Campus Crier
pub(in crate::card::sets) static CAMPUS_CRIER: CardRecord = CardRecord::new(
    "Campus Crier",
    "6047b14c-91d5-4f8e-af3f-057a541e2546",
    "Johan Grenier",
    CardRules::new_creature(mana_cost!("{1}{W}"), &["Human", "Advisor"], 3, 1).with_ability(
        AbilityDef::activated(
            "{1}, Exile this card from your graveyard: Empower Jace 2. (Put two loyalty \
             counters on a Jace token you control. If you don't control one, first create a \
             blue Jace planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            &[CostDef::Mana(mana_cost!("{1}")), CostDef::ExileSource],
            empower_jace(ValueDef::Constant(2)),
        )
        .with_source_zones(&[ZoneKind::Graveyard]),
    ),
);

// FRA 5 — Enlightened Confidant
// Audit: unsupported — Needs a surveil outcome binding naming the cards put into the graveyard
// this way.
pub(in crate::card::sets) static ENLIGHTENED_CONFIDANT: CardRecord = CardRecord::new(
    "Enlightened Confidant",
    "483fcc58-cc6e-4452-a696-7b38e117c837",
    "Denman Rooke",
    CardRules::unsupported(),
);

// FRA 6 — Fateshaper Aspirant
pub(in crate::card::sets) static FATESHAPER_ASPIRANT: CardRecord = CardRecord::new(
    "Fateshaper Aspirant",
    "f0c8400d-824f-4d79-84bc-7615a0deb831",
    "Paul Dainton",
    CardRules::new_creature(mana_cost!("{4}{W}"), &["Rhino", "Cleric"], 3, 4).with_ability(
        AbilityDef::modal_triggered(
            "When this creature enters, choose one —",
            TriggerEventDef::zone_changed(
                ObjectPredicateDef::Source,
                None,
                Some(ZoneKind::Battlefield),
            ),
            &[
                AbilityDef::spell_with_targets(
                    "Return target legendary card from your graveyard to your hand.",
                    &[AbilityTargetDef::exactly_one(
                        AbilityTargetPredicate::Object {
                            object: ObjectPredicateDef::Supertype(CardSupertype::Legendary),
                            zones: &[ZoneKind::Graveyard],
                            controller: None,
                            owner: Some(PlayerRelation::You),
                        },
                    )],
                    EffectDef::move_to_zone(
                        EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        ZoneKind::Hand,
                        ZonePlacement::Top,
                    ),
                ),
                AbilityDef::spell_with_targets(
                    "Put a +1/+1 counter on target creature. It gains vigilance and \
                     indestructible until end of turn. (Damage and effects that say \"destroy\" \
                     don't destroy it.)",
                    &[AbilityTargetDef::exactly_one_permanent(
                        ObjectPredicateDef::HasType(CardType::Creature),
                    )],
                    EffectDef::Sequence(&[
                        EffectDef::AddCounters {
                            object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            kind: CounterKind::PlusOnePlusOne,
                            amount: ValueDef::Constant(1),
                        },
                        EffectDef::Apply {
                            recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            effect: AppliedEffectDef::Composite(&[
                                AppliedEffectDef::add_ability(&abilities::vigilance()),
                                AppliedEffectDef::add_ability(&abilities::indestructible()),
                            ]),
                            duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                        },
                    ]),
                ),
            ],
        ),
    ),
);

// FRA 7 — Flickering Hound
pub(in crate::card::sets) static FLICKERING_HOUND: CardRecord = CardRecord::new(
    "Flickering Hound",
    "686f3a25-305d-4f02-8972-eba7b8e9635f",
    "Chris Rahn",
    CardRules::new_creature(mana_cost!("{3}{W}"), &["Dog"], 2, 2).with_ability(
        AbilityDef::triggered_with_targets(
            "Whenever you cast a creature spell, exile up to one other target creature you \
             control, then return that card to the battlefield under its owner's control.",
            TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                ObjectPredicateDef::HasType(CardType::Creature),
                ObjectPredicateDef::ControlledBy(PlayerRelation::You),
            ])),
            &[AbilityTargetDef::up_to(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::HasType(CardType::Creature),
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::You),
                    owner: None,
                },
                1,
            )
            .excluding_source()],
            // The exile is linked to the Hound so the return can name the card it made.
            EffectDef::Sequence(&[
                EffectDef::ExileLinkedToSource {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    face_down: false,
                    until_source_leaves: false,
                    then: None,
                },
                EffectDef::ReturnLinkedExiles {
                    object: ObjectPredicateDef::Any,
                    zone: ZoneKind::Battlefield,
                    grant: None,
                    counters: None,
                    transformed: false,
                    controller: None,
                },
            ]),
        ),
    ),
);

// FRA 8 — Generous Revival
pub(in crate::card::sets) static GENEROUS_REVIVAL: CardRecord = CardRecord::new(
    "Generous Revival",
    "f82f181b-a43b-4cec-b8f6-f6c1dd4c64fd",
    "Alessandra Pisano",
    CardRules::new_sorcery(mana_cost!("{2}{W}")).with_abilities(&[
        AbilityDef::spell_with_targets(
            "Return target creature card with mana value 3 or less from your graveyard to the \
             battlefield with an additional +1/+1 counter on it.",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::ManaValueAtMost(3),
                    ]),
                    zones: &[ZoneKind::Graveyard],
                    controller: None,
                    owner: Some(PlayerRelation::You),
                },
            )],
            EffectDef::WithBattlefieldArrival {
                effect: &EffectDef::move_to_zone(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ZoneKind::Battlefield,
                    ZonePlacement::Top,
                ),
                arrival: BattlefieldArrivalDef {
                    counters: Some(TokenCountersDef {
                        kind: CounterKind::PlusOnePlusOne,
                        amount: ValueDef::Constant(1),
                    }),
                    ..BattlefieldArrivalDef::DEFAULT
                },
            },
        ),
        abilities::flashback(&[CostDef::Mana(mana_cost!("{4}{W}"))]),
    ]),
);

// FRA 9 — Germinate Recruits
pub(in crate::card::sets) static GERMINATE_RECRUITS: CardRecord = CardRecord::new(
    "Germinate Recruits",
    "1b5d7d19-b32a-4786-ae9a-00da5e6658ad",
    "Rovina Cai",
    CardRules::new_instant(mana_cost!("{2}{W}")).with_ability(AbilityDef::spell(
        "Create X 2/2 colorless Wizard Soldier creature tokens named Cadet, where X is the \
         amount of life you gained this turn.",
        EffectDef::CreateToken(
            CreateTokenDef::new(TokenDef::Literal(CADET_TOKEN))
                .with_count(ValueDef::LifeGainedThisTurn(PlayerRelation::You)),
        ),
    )),
);

// FRA 10 — Graft Surgeon
pub(in crate::card::sets) static GRAFT_SURGEON: CardRecord = CardRecord::new(
    "Graft Surgeon",
    "32a7a905-11bf-4b66-a28e-1066a0e372b8",
    "Bram Sels",
    CardRules::new_creature(mana_cost!("{2}{W}"), &["Human", "Cleric"], 2, 2).with_abilities(&[
        AbilityDef::as_enters(
            "This creature enters with a +1/+1 counter on it.",
            ReplacementEffectDef::ModifyBattlefieldEntry(
                BattlefieldEntryModificationDef::AddCounters {
                    kind: CounterKind::PlusOnePlusOne,
                    amount: 1,
                },
            ),
        ),
        abilities::dies_trigger_with_targets(
            "When this creature dies, put its counters on up to one target creature you \
             control.",
            &[AbilityTargetDef::up_to(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::HasType(CardType::Creature),
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::You),
                    owner: None,
                },
                1,
            )],
            EffectDef::AddCountersFrom {
                from: ObjectRefDef::Source,
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
            },
        ),
    ]),
);

// FRA 11 — Guiding Hydra
pub(in crate::card::sets) static GUIDING_HYDRA: CardRecord = CardRecord::new(
    "Guiding Hydra",
    "a53eb840-039d-4c45-b701-d58cb26b1a6c",
    "Jason Mowry",
    CardRules::new_creature(mana_cost!("{X}{W}"), &["Hydra", "Horror"], 1, 0).with_abilities(&[
        AbilityDef::as_enters(
            "This creature enters with X +1/+1 counters on it.",
            ReplacementEffectDef::ModifyBattlefieldEntry(
                BattlefieldEntryModificationDef::AddCastXCounters {
                    kind: CounterKind::PlusOnePlusOne,
                },
            ),
        ),
        AbilityDef::triggered(
            "At the beginning of combat on your turn, you may remove a +1/+1 counter from this \
             creature. If you do, put a +1/+1 counter on each other creature you control.",
            TriggerEventDef::StepBegins {
                step: TurnStepDef::BeginningOfCombat,
                player: PlayerRelation::You,
            },
            // Not a `PayOrDef`: a resolving payment cannot take a counter off its own
            // source without requiring that you still control it. Offered only while the
            // Hydra is there with a counter to give up, so taking the offer is "if you do".
            EffectDef::IfCondition {
                condition: &TriggerConditionDef::All(&[
                    TriggerConditionDef::SourceOnBattlefield,
                    TriggerConditionDef::SourceCounters {
                        kind: CounterKind::PlusOnePlusOne,
                        comparison: ComparisonDef::GreaterOrEqual,
                        amount: 1,
                    },
                ]),
                then: &EffectDef::May {
                    player: EffectRecipientDef::Controller,
                    effect: &EffectDef::Sequence(&[
                        EffectDef::RemoveCounters {
                            object: EffectRecipientDef::Source,
                            kind: CounterKind::PlusOnePlusOne,
                            amount: ValueDef::Constant(1),
                        },
                        EffectDef::AddCounters {
                            object: EffectRecipientDef::matching_objects(
                                ObjectPredicateDef::All(&[
                                    ObjectPredicateDef::HasType(CardType::Creature),
                                    ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                                ]),
                                &[ZoneKind::Battlefield],
                                PlayerRelation::You,
                            ),
                            kind: CounterKind::PlusOnePlusOne,
                            amount: ValueDef::Constant(1),
                        },
                    ]),
                },
            },
        ),
    ]),
);

// FRA 12 — Hexhaven Battalion
pub(in crate::card::sets) static HEXHAVEN_BATTALION: CardRecord = CardRecord::new(
    "Hexhaven Battalion",
    "3b6ac80e-c726-4bd0-893a-e666041a04a6",
    "Julia Griffin",
    CardRules::new_sorcery(mana_cost!("{4}{W}{W}")).with_abilities(&[
        AbilityDef::spell(
            "Create three 2/2 colorless Wizard Soldier creature tokens named Cadet. Empower \
             Jace 2. (Put two loyalty counters on a Jace token you control. If you don't \
             control one, first create a blue Jace planeswalker token with \
             \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            EffectDef::Sequence(&[
                EffectDef::CreateToken(
                    CreateTokenDef::new(TokenDef::Literal(CADET_TOKEN)).with_amount(3),
                ),
                empower_jace(ValueDef::Constant(2)),
            ]),
        ),
        abilities::typecycling!(
            "Basic landcycling {2} ({2}, Discard this card: Search your library for a basic \
             land card, reveal it, put it into your hand, then shuffle.)",
            &[CostDef::Mana(mana_cost!("{2}"))],
            A_BASIC_LAND_CARD,
        ),
    ]),
);

// FRA 13 — Kindred Judgment
// Audit: unsupported — Needs a creature type chosen at resolution that later predicates in the
// same resolution can read.
pub(in crate::card::sets) static KINDRED_JUDGMENT: CardRecord = CardRecord::new(
    "Kindred Judgment",
    "6f9f814b-8249-4e48-a05e-4c84060fe6fb",
    "Ekaterina Burmak",
    CardRules::unsupported(),
);

// FRA 14 — Loyal Tutor
pub(in crate::card::sets) static LOYAL_TUTOR: CardRecord = CardRecord::new(
    "Loyal Tutor",
    "490dae91-94ce-42a9-a11f-6c5e77c4e486",
    "Darren Tan",
    CardRules::new_instant(mana_cost!("{W}")).with_ability(AbilityDef::spell(
        "Search your library for a planeswalker card, reveal it, then shuffle and put that card \
         on top.",
        EffectDef::SearchZone {
            exile_face_down: false,
            player: EffectRecipientDef::Controller,
            source: ZoneKind::Library,
            object: ObjectPredicateDef::HasType(CardType::Planeswalker),
            minimum: 0,
            maximum: ValueDef::Constant(1),
            reveal: true,
            destination: ZoneKind::Library,
            placement: ZonePlacement::Top,
            shuffle: true,
            enters_tapped: false,
            attachment: None,
            binding: None,
            then: None,
        },
    )),
);

// FRA 15 — Memory Trap
pub(in crate::card::sets) static MEMORY_TRAP: CardRecord = CardRecord::new(
    "Memory Trap",
    "2f5345ae-4489-4d05-b2d5-c71285254f05",
    "Ekaterina Burmak",
    CardRules::new_enchantment(mana_cost!("{2}{W}")).with_ability(
        abilities::enters_trigger_with_targets(
            "When this enchantment enters, exile target nonland permanent an opponent controls \
             until this enchantment leaves the battlefield.",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Land)),
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::Opponent),
                    owner: None,
                },
            )],
            EffectDef::ExileLinkedToSource {
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                face_down: false,
                until_source_leaves: true,
                then: None,
            },
        ),
    ),
);

// FRA 16 — Predictive Preparations
pub(in crate::card::sets) static PREDICTIVE_PREPARATIONS: CardRecord = CardRecord::new(
    "Predictive Preparations",
    "50a0e5f0-3c39-4f16-9a73-eec8ef71f12e",
    "Nathaniel Himawan",
    CardRules::new_sorcery(mana_cost!("{1}{W}")).with_abilities(&[
        AbilityDef::spell_with_targets(
            "Put a +1/+1 counter on each of one or two target creatures.",
            &[AbilityTargetDef {
                minimum: 1,
                ..AbilityTargetDef::up_to(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Creature),
                        zones: &[ZoneKind::Battlefield],
                        controller: None,
                        owner: None,
                    },
                    2,
                )
            }],
            EffectDef::AddCounters {
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                kind: CounterKind::PlusOnePlusOne,
                amount: ValueDef::Constant(1),
            },
        ),
        abilities::flashback(&[CostDef::Mana(mana_cost!("{3}{W}"))]),
    ]),
);

// FRA 17 — Prophesied End
pub(in crate::card::sets) static PROPHESIED_END: CardRecord = CardRecord::new(
    "Prophesied End",
    "1f95399a-9766-4f3d-aa6a-ece55e0530d9",
    "A. M. Sartor",
    CardRules::new_instant(mana_cost!("{1}{W}")).with_ability(AbilityDef::spell_with_targets(
        "Destroy target creature. If it wasn't attacking, its controller draws a card.",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::HasType(CardType::Creature),
        )],
        // Whether it was attacking is read while it is still there to ask; the draw is
        // owed whether or not the destruction then happens.
        EffectDef::IfElseCondition {
            condition: &TriggerConditionDef::TargetMatches {
                slot: TargetIndex::PRIMARY,
                object: ObjectPredicateDef::Attacking,
            },
            then: &EffectDef::destroy_target(TargetIndex::PRIMARY),
            otherwise: &EffectDef::Sequence(&[
                EffectDef::destroy_target(TargetIndex::PRIMARY),
                EffectDef::DrawCards {
                    recipient: EffectRecipientDef::ControllerOfTarget(TargetIndex::PRIMARY),
                    amount: ValueDef::Constant(1),
                },
            ]),
        },
    )),
);

// FRA 18 — Refute Destiny
pub(in crate::card::sets) static REFUTE_DESTINY: CardRecord = CardRecord::new(
    "Refute Destiny",
    "2c588954-c6eb-4aae-a2fa-0651ccf2d90a",
    "Billy Christian",
    CardRules::new_sorcery(mana_cost!("{1}{W}")).with_ability(AbilityDef::spell_with_targets(
        "Exile target creature or planeswalker that's green or blue. Surveil 1. (Look at the \
         top card of your library. You may put it into your graveyard.)",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::All(&[
                A_CREATURE_OR_PLANESWALKER,
                ObjectPredicateDef::AnyOf(&[
                    ObjectPredicateDef::Color(ManaColor::Green),
                    ObjectPredicateDef::Color(ManaColor::Blue),
                ]),
            ]),
        )],
        EffectDef::Sequence(&[
            EffectDef::move_to_zone(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ZoneKind::Exile,
                ZonePlacement::Top,
            ),
            abilities::surveil(ValueDef::Constant(1)),
        ]),
    )),
);

// FRA 19 — Repurposed Enforcer
pub(in crate::card::sets) static REPURPOSED_ENFORCER: CardRecord = CardRecord::new(
    "Repurposed Enforcer",
    "35000e93-85d3-44f8-976a-5918ee4c71e0",
    "Chris Rallis",
    CardRules::new_creature(mana_cost!("{1}{W}"), &["Human", "Soldier"], 3, 2).with_ability(
        AbilityDef::triggered(
            "Whenever this creature attacks, empower Jace X, where X is the number of \
             creatures you control. (Put that many loyalty counters on a Jace token you \
             control. If you don't control one, first create a blue Jace planeswalker token \
             with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            TriggerEventDef::attacks(ObjectPredicateDef::Source),
            empower_jace(ValueDef::CountMatchingObjects(
                &ObjectQueryDef::controlled_by(
                    ObjectPredicateDef::HasType(CardType::Creature),
                    &[ZoneKind::Battlefield],
                    PlayerSetDef::Related(PlayerRelation::You),
                ),
            )),
        ),
    ),
);

// FRA 20 — Return to the Light Realms
// Audit: unsupported — Needs the CR 303.4f host choice for an Aura card put onto the battlefield
// without being cast. A plain zone move brings the Aura in unattached, so state-based actions
// put it into the graveyard instead of it entering attached to a legal permanent or player, or
// staying in the graveyard when there is none (CR 303.4g).
pub(in crate::card::sets) static RETURN_TO_THE_LIGHT_REALMS: CardRecord = CardRecord::new(
    "Return to the Light Realms",
    "9e72f397-2384-40f1-882b-f627664d97df",
    "Anna Steinbauer",
    CardRules::unsupported(),
);

// FRA 21 — Shatterwing Pegasus
pub(in crate::card::sets) static SHATTERWING_PEGASUS: CardRecord = CardRecord::new(
    "Shatterwing Pegasus",
    "e29095de-59ec-4562-ba8e-73f952e457ae",
    "Antonio José Manzanedo",
    CardRules::new_creature(mana_cost!("{2}{W}"), &["Pegasus"], 2, 3).with_abilities(&[
        abilities::flying(),
        AbilityDef::activated(
            "{4}{W}: Creatures you control get +1/+1 until end of turn.",
            &[CostDef::Mana(mana_cost!("{4}{W}"))],
            EffectDef::Apply {
                recipient: EffectRecipientDef::matching_objects(
                    ObjectPredicateDef::HasType(CardType::Creature),
                    &[ZoneKind::Battlefield],
                    PlayerRelation::You,
                ),
                effect: AppliedEffectDef::modify_power_toughness(
                    ValueDef::Constant(1),
                    ValueDef::Constant(1),
                ),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
        ),
    ]),
);

// FRA 22 — Surgical Precision
pub(in crate::card::sets) static SURGICAL_PRECISION: CardRecord = CardRecord::new(
    "Surgical Precision",
    "d3acf176-ef02-4729-88c4-0f0dfbfdada4",
    "Nathaniel Himawan",
    CardRules::new_sorcery(mana_cost!("{1}{W}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "Destroy target creature with toughness 4 or greater. You gain 1 life.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::ToughnessGreaterThan(ValueDef::Constant(3)),
                    ]),
                )],
                EffectDef::Sequence(&[
                    EffectDef::destroy_target(TargetIndex::PRIMARY),
                    EffectDef::GainLife {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(1),
                    },
                ]),
            ),
            AbilityDef::spell(
                "You draw a card and gain 2 life.",
                EffectDef::Sequence(&[
                    abilities::draw_cards(ValueDef::Constant(1)),
                    EffectDef::GainLife {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(2),
                    },
                ]),
            ),
        ],
    )),
);

// FRA 23 — Unflinching Hortimancer
pub(in crate::card::sets) static UNFLINCHING_HORTIMANCER: CardRecord = CardRecord::new(
    "Unflinching Hortimancer",
    "63f82985-c9c2-4d0a-ac4f-560166bebd9f",
    "Manuel Castañón",
    CardRules::new_creature(mana_cost!("{1}{W}"), &["Human", "Cleric"], 2, 1).with_abilities(&[
        abilities::ward(
            &[CostDef::Mana(mana_cost!("{1}"))],
            "Ward {1} (Whenever this creature becomes the target of a spell or ability an \
             opponent controls, counter it unless that player pays {1}.)",
        ),
        AbilityDef::triggered(
            "Whenever you gain life, put a +1/+1 counter on this creature.",
            TriggerEventDef::LifeGained(PlayerRelation::You),
            EffectDef::AddCounters {
                object: EffectRecipientDef::Source,
                kind: CounterKind::PlusOnePlusOne,
                amount: ValueDef::Constant(1),
            },
        ),
    ]),
);

// FRA 24 — Your Fate Ends Here
pub(in crate::card::sets) static YOUR_FATE_ENDS_HERE: CardRecord = CardRecord::new(
    "Your Fate Ends Here",
    "25000a17-b701-4d69-b2ef-2c74029199d3",
    "Andrew Mar",
    CardRules::new_instant(mana_cost!("{2}{W}")).with_ability(AbilityDef::spell_with_targets(
        "Destroy target creature or planeswalker with mana value 3 or greater. Surveil 1. \
         (Look at the top card of your library. You may put it into your graveyard.)",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::All(&[
                A_CREATURE_OR_PLANESWALKER,
                ObjectPredicateDef::Not(&ObjectPredicateDef::ManaValueAtMost(2)),
            ]),
        )],
        EffectDef::Sequence(&[
            EffectDef::destroy_target(TargetIndex::PRIMARY),
            abilities::surveil(ValueDef::Constant(1)),
        ]),
    )),
);

// FRA 25 — Countersculpt
pub(in crate::card::sets) static COUNTERSCULPT: CardRecord = CardRecord::new(
    "Countersculpt",
    "145b928d-a7ff-4fe5-ae4d-bbae7b1d955b",
    "Anna Podedworna",
    CardRules::new_instant(mana_cost!("{U}{U}")).with_ability(
        AbilityDef::spell_with_additional_cost(
            "As an additional cost to cast this spell, behold a Jace or pay {1}. (To behold a \
             Jace, choose a Jace you control or reveal a Jace card from your hand.)\nCounter \
             target spell. Empower Jace 1. (Put a loyalty counter on a Jace token you control. \
             If you don't control one, first create a blue Jace planeswalker token with \
             \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::Spell,
                    zones: &[ZoneKind::Stack],
                    controller: None,
                    owner: None,
                },
            )],
            CostDef::choice(&[
                CostDef::Behold {
                    object: ObjectPredicateDef::Subtype(SubtypeDef::from_name("Jace")),
                    count: 1,
                    choose_creature_type: false,
                },
                CostDef::pay_mana(mana_cost!("{1}")),
            ]),
            EffectDef::Sequence(&[
                EffectDef::Counter {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    zone: ZoneKind::Graveyard,
                    placement: ZonePlacement::Top,
                },
                empower_jace(ValueDef::Constant(1)),
            ]),
        ),
    ),
);

// FRA 26 — Cruel Calculations
// Audit: unsupported — Needs a per-turn count of cards put into a player's graveyard from their
// library.
pub(in crate::card::sets) static CRUEL_CALCULATIONS: CardRecord = CardRecord::new(
    "Cruel Calculations",
    "f76c4d8e-3e1f-4264-99af-1b8adb9a06be",
    "Lindsey Look",
    CardRules::unsupported(),
);

// FRA 27 — Cryotheory Adept
pub(in crate::card::sets) static CRYOTHEORY_ADEPT: CardRecord = CardRecord::new(
    "Cryotheory Adept",
    "9ba1f7ce-3404-4932-9795-22967707f762",
    "Kai Carpenter",
    CardRules::new_creature(mana_cost!("{1}{U}"), &["Human", "Wizard"], 2, 1).with_abilities(&[
        abilities::prowess(),
        AbilityDef::activated_with_targets(
            "{3}{U}, Exile this card from your graveyard: Tap target creature and put a stun \
             counter on it. Activate only as a sorcery. (If a permanent with a stun counter \
             would become untapped, remove one from it instead.)",
            &[CostDef::Mana(mana_cost!("{3}{U}")), CostDef::ExileSource],
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::HasType(CardType::Creature),
            )],
            EffectDef::Sequence(&[
                EffectDef::Tap {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                },
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    kind: CounterKind::Stun,
                    amount: ValueDef::Constant(1),
                },
            ]),
        )
        .with_source_zones(&[ZoneKind::Graveyard])
        .with_activation_timing(ActivationTimingDef::SorcerySpeed),
    ]),
);

// FRA 28 — Diviner of Victory // Unwind History
// Audit: unsupported — Diviner of Victory needs a trigger event for scrying or surveilling,
// published wherever a scry or surveil of at least one card is performed.
pub(in crate::card::sets) static DIVINER_OF_VICTORY: CardRecord = CardRecord::new_prepare(
    "Diviner of Victory // Unwind History",
    "0853bb80-8664-432a-8457-600139fd96d5",
    "Justyna Dura",
    &[
        ("Diviner of Victory", CardRules::unsupported()),
        ("Unwind History", CardRules::unsupported()),
    ],
);

// FRA 29 — Divining Duelist
pub(in crate::card::sets) static DIVINING_DUELIST: CardRecord = CardRecord::new(
    "Divining Duelist",
    "960c7335-331d-488b-be68-2ad1c1c695dc",
    "David Auden Nash",
    CardRules::new_creature(mana_cost!("{2}{U}"), &["Merfolk", "Wizard"], 3, 2).with_abilities(&[
        abilities::flash(),
        AbilityDef::modal_triggered(
            "When this creature enters, choose one —",
            TriggerEventDef::zone_changed(
                ObjectPredicateDef::Source,
                None,
                Some(ZoneKind::Battlefield),
            ),
            &[
                AbilityDef::spell_with_targets(
                    "Tap target creature.",
                    &[AbilityTargetDef::exactly_one_permanent(
                        ObjectPredicateDef::HasType(CardType::Creature),
                    )],
                    EffectDef::Tap {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    },
                ),
                AbilityDef::spell_with_targets(
                    "Untap target creature.",
                    &[AbilityTargetDef::exactly_one_permanent(
                        ObjectPredicateDef::HasType(CardType::Creature),
                    )],
                    EffectDef::Untap {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    },
                ),
                AbilityDef::spell(
                    "Draw a card, then discard a card.",
                    EffectDef::Sequence(&[
                        abilities::draw_cards(ValueDef::Constant(1)),
                        EffectDef::Discard {
                            recipient: EffectRecipientDef::Controller,
                            amount: ValueDef::Constant(1),
                            selection: DiscardSelectionDef::RecipientChooses,
                            then: None,
                        },
                    ]),
                ),
            ],
        ),
    ]),
);

// FRA 30 — Icy Reception
pub(in crate::card::sets) static ICY_RECEPTION: CardRecord = CardRecord::new(
    "Icy Reception",
    "8d754b96-5e44-45af-9c7a-b0da59fbe4c3",
    "Borja Pindado",
    CardRules::new_instant(mana_cost!("{1}{U}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "Counter target creature or legendary spell unless its controller pays {3}.",
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::All(&[
                            ObjectPredicateDef::Spell,
                            ObjectPredicateDef::AnyOf(&[
                                ObjectPredicateDef::HasType(CardType::Creature),
                                ObjectPredicateDef::Supertype(CardSupertype::Legendary),
                            ]),
                        ]),
                        zones: &[ZoneKind::Stack],
                        controller: None,
                        owner: None,
                    },
                )],
                abilities::counter_target_unless_paid(&[CostDef::Mana(mana_cost!("{3}"))]),
            ),
            AbilityDef::spell_with_targets(
                "Target creature gets -5/-0 until end of turn.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(-5),
                        ValueDef::Constant(0),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ],
    )),
);

// FRA 31 — Infinite Coursework
pub(in crate::card::sets) static INFINITE_COURSEWORK: CardRecord = CardRecord::new(
    "Infinite Coursework",
    "a5988272-faaa-463d-a0a1-a8e96b946bad",
    "Randy Vargas",
    CardRules::new_enchantment(mana_cost!("{2}{U}"))
        .with_subtypes(&["Aura"])
        .with_abilities(&[
            abilities::aura_spell(
                "Enchant creature",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
            ),
            abilities::enters_trigger(
                "When this Aura enters, tap enchanted creature. It becomes unprepared.",
                EffectDef::Sequence(&[
                    EffectDef::Tap {
                        object: EffectRecipientDef::AttachedPermanent,
                    },
                    become_unprepared(EffectRecipientDef::AttachedPermanent),
                ]),
            ),
            AbilityDef::static_ability(
                "Enchanted creature loses all abilities and doesn't untap during its \
                 controller's untap step.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::AttachedPermanent,
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::remove_abilities(AbilityPredicateDef::Any),
                        AppliedEffectDef::Rule(AppliedRuleDef::DoesNotUntapDuringUntapStep),
                    ]),
                },
            ),
        ]),
);

// FRA 32 — Jace's Machinations
// Audit: unsupported — Needs a player-scoped permission to activate loyalty abilities at instant
// speed; an effect locked to the Jaces controlled at resolution would miss the token this spell
// creates afterwards.
pub(in crate::card::sets) static JACE_S_MACHINATIONS: CardRecord = CardRecord::new(
    "Jace's Machinations",
    "282588b9-3656-453b-aa25-2419e078ddc1",
    "Serena Malyon",
    CardRules::unsupported(),
);

// FRA 33 — Mindseeker Oculus
pub(in crate::card::sets) static MINDSEEKER_OCULUS: CardRecord = CardRecord::new(
    "Mindseeker Oculus",
    "f5324741-353a-4a70-adb2-b631b00806dd",
    "Ioannis Fiore",
    CardRules::new_creature(mana_cost!("{2}{U}"), &["Homunculus"], 2, 1).with_ability(
        abilities::enters_trigger(
            "When this creature enters, empower Jace 4. (Put four loyalty counters on a Jace \
             token you control. If you don't control one, first create a blue Jace \
             planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(4)),
        ),
    ),
);

// FRA 34 — Perfected Theory
pub(in crate::card::sets) static PERFECTED_THEORY: CardRecord = CardRecord::new(
    "Perfected Theory",
    "d0ecae06-bc5a-4886-84df-c2900816f226",
    "Marta Nael",
    CardRules::new_instant(mana_cost!("{U}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "Target creature has base power and toughness 1/1 until end of turn.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::set_base_power_toughness(
                        ValueDef::Constant(1),
                        ValueDef::Constant(1),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
            AbilityDef::spell_with_targets(
                "Target creature has base power and toughness 4/5 until end of turn.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::set_base_power_toughness(
                        ValueDef::Constant(4),
                        ValueDef::Constant(5),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ],
    )),
);

// FRA 35 — Plan for All Outcomes
pub(in crate::card::sets) static PLAN_FOR_ALL_OUTCOMES: CardRecord = CardRecord::new(
    "Plan for All Outcomes",
    "c4effc17-0d0e-423a-b5f2-597ea6c71f67",
    "Chris Cold",
    CardRules::new_enchantment(mana_cost!("{3}{U}")).with_abilities(&[
        abilities::enters_trigger_with_targets(
            "When this enchantment enters, the owner of up to one other target nonland \
             permanent puts it on their choice of the top or bottom of their library.",
            &[AbilityTargetDef::up_to(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Land)),
                    zones: &[ZoneKind::Battlefield],
                    controller: None,
                    owner: None,
                },
                1,
            )
            .excluding_source()],
            EffectDef::ChooseEffect {
                player: EffectRecipientDef::player(PlayerRefDef::OwnerOf(ObjectRefDef::Target(
                    TargetIndex::PRIMARY,
                ))),
                choices: &[
                    EffectChoiceDef {
                        label: "Top",
                        effect: EffectDef::move_to_zone(
                            EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            ZoneKind::Library,
                            ZonePlacement::Top,
                        ),
                    },
                    EffectChoiceDef {
                        label: "Bottom",
                        effect: EffectDef::move_to_zone(
                            EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            ZoneKind::Library,
                            ZonePlacement::Bottom,
                        ),
                    },
                ],
            },
        ),
        AbilityDef::triggered(
            "Whenever you cast your first noncreature spell each turn, empower Jace 1. (Put a \
             loyalty counter on a Jace token you control. If you don't control one, first \
             create a blue Jace planeswalker token with \"[−1]: Surveil 1\" and \
             \"[−3]: Draw a card.\")",
            // "Your first" is part of the event, fixed as the spell is cast: a second
            // noncreature spell cast in response does not take this trigger back.
            TriggerEventDef::While {
                event: &TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                    ObjectPredicateDef::NoncreatureSpell,
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ])),
                condition: &TriggerConditionDef::ValueComparison(&ValueComparisonDef {
                    left: ValueDef::CountSpellsCastThisTurn(&SpellCastQueryDef {
                        player: PlayerRelation::You,
                        spell: ObjectPredicateDef::NoncreatureSpell,
                    }),
                    comparison: ComparisonDef::Equal,
                    right: ValueDef::Constant(1),
                }),
            },
            empower_jace(ValueDef::Constant(1)),
        ),
    ]),
);

// FRA 36 — Precise Redaction
pub(in crate::card::sets) static PRECISE_REDACTION: CardRecord = CardRecord::new(
    "Precise Redaction",
    "9244efad-35ab-45c0-b173-4bc68276cb67",
    "Mathias Kollros",
    CardRules::new_instant(mana_cost!("{1}{U}")).with_ability(AbilityDef::spell_with_targets(
        "Counter target white or black spell.",
        &[AbilityTargetDef::exactly_one(
            AbilityTargetPredicate::Object {
                object: ObjectPredicateDef::All(&[
                    ObjectPredicateDef::Spell,
                    ObjectPredicateDef::AnyOf(&[
                        ObjectPredicateDef::Color(ManaColor::White),
                        ObjectPredicateDef::Color(ManaColor::Black),
                    ]),
                ]),
                zones: &[ZoneKind::Stack],
                controller: None,
                owner: None,
            },
        )],
        EffectDef::counter_target(TargetIndex::PRIMARY),
    )),
);

// FRA 37 — Protege's Awakening
pub(in crate::card::sets) static PROTEGE_S_AWAKENING: CardRecord = CardRecord::new(
    "Protege's Awakening",
    "a08c7ec2-4c6a-4db2-85a7-41afe8731523",
    "Cynthia Sheppard",
    CardRules::new_sorcery(mana_cost!("{3}{U}")).with_abilities(&[
        AbilityDef::spell(
            "Empower Jace 6. (Put six loyalty counters on a Jace token you control. If you \
                don't control one, first create a blue Jace planeswalker token with \
                \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(6)),
        ),
        AbilityDef::spell(
            "Draw a card.",
            EffectDef::DrawCards {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(1),
            },
        ),
    ]),
);

// FRA 38 — Seasoned Cryomancer
// A target count cannot read how many nonland cards the discard turned up, but two cards are
// discarded, so that number is one or two and each gets the same trigger with its own limit.
const fn seasoned_cryomancer_freeze<const CREATURES: u8>() -> AbilityDef {
    AbilityDef::triggered_with_targets(
        "When you discard one or more nonland cards this way, tap up to that many target \
         creatures and put a stun counter on each of them.",
        TriggerEventDef::Reflexive,
        &const {
            [AbilityTargetDef::up_to(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::HasType(CardType::Creature),
                    zones: &[ZoneKind::Battlefield],
                    controller: None,
                    owner: None,
                },
                CREATURES,
            )]
        },
        EffectDef::Sequence(
            &const {
                [
                    EffectDef::Tap {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    },
                    EffectDef::AddCounters {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        kind: CounterKind::Stun,
                        amount: ValueDef::Constant(1),
                    },
                ]
            },
        ),
    )
}
pub(in crate::card::sets) static SEASONED_CRYOMANCER: CardRecord = CardRecord::new(
    "Seasoned Cryomancer",
    "51d86875-420d-4e82-b69c-4feeb99c9428",
    "Andrea Piparo",
    CardRules::new_creature(mana_cost!("{1}{U}{U}"), &["Human", "Wizard"], 2, 2).with_abilities(&[
        abilities::enters_trigger(
            "When this creature enters, draw two cards, then discard two cards. When you \
             discard one or more nonland cards this way, tap up to that many target creatures \
             and put a stun counter on each of them.",
            EffectDef::Sequence(&[
                abilities::draw_cards(ValueDef::Constant(2)),
                EffectDef::Discard {
                    recipient: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(2),
                    selection: DiscardSelectionDef::RecipientChooses,
                    then: Some(DiscardFollowUpDef {
                        counted: ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(
                            CardType::Land,
                        )),
                        bound: Some(Binding!("discarded")),
                        effect: &EffectDef::IfElseCondition {
                            condition: &TriggerConditionDef::ValueComparison(&ValueComparisonDef {
                                left: ValueDef::BoundObjectCount(Binding!("discarded")),
                                comparison: ComparisonDef::GreaterOrEqual,
                                right: ValueDef::Constant(2),
                            }),
                            then: &EffectDef::ReflexiveTrigger(&seasoned_cryomancer_freeze::<2>()),
                            otherwise: &EffectDef::IfCondition {
                                condition: &TriggerConditionDef::ValueComparison(
                                    &ValueComparisonDef {
                                        left: ValueDef::BoundObjectCount(Binding!("discarded")),
                                        comparison: ComparisonDef::GreaterOrEqual,
                                        right: ValueDef::Constant(1),
                                    },
                                ),
                                then: &EffectDef::ReflexiveTrigger(
                                    &seasoned_cryomancer_freeze::<1>(),
                                ),
                            },
                        },
                    }),
                },
            ]),
        ),
        AbilityDef::activated(
            "{3}{U}{U}, Exile this card from your graveyard: Draw two cards.",
            &[CostDef::Mana(mana_cost!("{3}{U}{U}")), CostDef::ExileSource],
            abilities::draw_cards(ValueDef::Constant(2)),
        )
        .with_source_zones(&[ZoneKind::Graveyard]),
    ]),
);

// FRA 39 — Semester Foreseer // Peer Review
pub(in crate::card::sets) static SEMESTER_FORESEER: CardRecord = CardRecord::new_prepare(
    "Semester Foreseer // Peer Review",
    "7e5a640b-fd88-4b5a-9dfc-1d8f5a5cef41",
    "Alix Branwyn",
    &[
        (
            "Semester Foreseer",
            CardRules::new_creature(mana_cost!("{3}{U}"), &["Human", "Wizard"], 3, 4)
                .with_abilities(&[
                    enters_prepared_with_reminder(),
                    abilities::enters_trigger(
                        "When this creature enters, surveil 1.",
                        abilities::surveil(ValueDef::Constant(1)),
                    ),
                ]),
        ),
        ("Peer Review", PEER_REVIEW),
    ],
);

// FRA 40 — Sphinx of False Conclusions
pub(in crate::card::sets) static SPHINX_OF_FALSE_CONCLUSIONS: CardRecord = CardRecord::new(
    "Sphinx of False Conclusions",
    "08ffbd51-2bd3-4262-8809-09576ce2b6f5",
    "Wylie Beckert",
    CardRules::new_creature(mana_cost!("{2}{U}{U}"), &["Sphinx", "Illusion"], 4, 2).with_abilities(
        &[
            abilities::flash(),
            abilities::flying(),
            AbilityDef::triggered(
                "Whenever this creature attacks, draw a card, then discard a card.",
                TriggerEventDef::attacks(ObjectPredicateDef::Source),
                EffectDef::Sequence(&[
                    abilities::draw_cards(ValueDef::Constant(1)),
                    EffectDef::Discard {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(1),
                        selection: DiscardSelectionDef::RecipientChooses,
                        then: None,
                    },
                ]),
            ),
            AbilityDef::triggered_if(
                "When this creature dies, if it isn't a token, create a token that's a copy of \
                 it.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::Source,
                    Some(ZoneKind::Battlefield),
                    Some(ZoneKind::Graveyard),
                ),
                &TriggerConditionDef::SourceMatches {
                    object: ObjectPredicateDef::Not(&ObjectPredicateDef::Token),
                },
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Copy(&TokenCopyDef {
                    object: &EffectRecipientDef::Source,
                    exceptions: CopyExceptionsDef::NONE,
                }))),
            ),
        ],
    ),
);

// FRA 41 — Sphinx's Approach
// Audit: unsupported — Needs a card-declared any-number-of-copies deck rule, and an optional
// payment at resolution that exiles the spell itself.
pub(in crate::card::sets) static SPHINX_S_APPROACH: CardRecord = CardRecord::new(
    "Sphinx's Approach",
    "f49be090-c745-40e5-bc1c-605b8d98acdf",
    "Nathaniel Himawan",
    CardRules::unsupported(),
);

// FRA 42 — Surveillance Phantasm
// Audit: unsupported — Needs a per-turn history flag recording that a player has scried or
// surveilled this turn.
pub(in crate::card::sets) static SURVEILLANCE_PHANTASM: CardRecord = CardRecord::new(
    "Surveillance Phantasm",
    "9df8a06d-c7de-49af-8c01-06dca3dfef4b",
    "Randy Vargas",
    CardRules::unsupported(),
);

// FRA 43 — The Theorist, Jace Beleren
pub(in crate::card::sets) static THE_THEORIST_JACE_BELEREN: CardRecord = CardRecord::new(
    "The Theorist, Jace Beleren",
    "20bb8c55-4b0b-425f-8201-b54fa2fdde86",
    "Ekaterina Burmak",
    CardRules::new_planeswalker(mana_cost!("{2}{U}{U}"), &["Jace"], 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::triggered(
                "At the beginning of each opponent's draw step, you draw a card.",
                TriggerEventDef::StepBegins {
                    step: TurnStepDef::Draw,
                    player: PlayerRelation::Opponent,
                },
                abilities::draw_cards(ValueDef::Constant(1)),
            ),
            AbilityDef::activated(
                "+1: Create a 1/1 blue Illusion creature token.",
                &[CostDef::Loyalty(ValueDef::Constant(1))],
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                    TokenCharacteristics::creature(&["Illusion"], &[ManaColor::Blue], 1, 1)
                        .with_art(CardArt::new(
                            "7bda0b63-fd0f-495d-9eb0-20057571c0ee",
                            "Daniel Cotter",
                        )),
                ))),
            ),
            // One opponent in this engine, so "for each opponent, up to one target ... that
            // player controls" is a single slot over what that opponent controls.
            AbilityDef::activated_with_targets(
                "−2: For each opponent, return up to one target artifact or creature that \
                 player controls to its owner's hand.",
                &[CostDef::Loyalty(ValueDef::Constant(-2))],
                &[AbilityTargetDef::up_to(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::AnyOf(&[
                            ObjectPredicateDef::HasType(CardType::Artifact),
                            ObjectPredicateDef::HasType(CardType::Creature),
                        ]),
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::Opponent),
                        owner: None,
                    },
                    1,
                )],
                EffectDef::move_to_zone(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ZoneKind::Hand,
                    ZonePlacement::Top,
                ),
            ),
            AbilityDef::activated(
                "−6: Draw three cards. Then put X +1/+1 counters on each creature you control, \
                 where X is the number of cards in your hand.",
                &[CostDef::Loyalty(ValueDef::Constant(-6))],
                EffectDef::Sequence(&[
                    abilities::draw_cards(ValueDef::Constant(3)),
                    EffectDef::AddCounters {
                        object: EffectRecipientDef::matching_objects(
                            ObjectPredicateDef::HasType(CardType::Creature),
                            &[ZoneKind::Battlefield],
                            PlayerRelation::You,
                        ),
                        kind: CounterKind::PlusOnePlusOne,
                        amount: ValueDef::CardsInHandAbove {
                            player: PlayerRelation::You,
                            threshold: 0,
                        },
                    },
                ]),
            ),
        ]),
);

// FRA 44 — Theorist's Proxy
pub(in crate::card::sets) static THEORIST_S_PROXY: CardRecord = CardRecord::new(
    "Theorist's Proxy",
    "710302ca-c4be-4069-8ce1-f531414c74e9",
    "Matt Stewart",
    CardRules::new_creature(mana_cost!("{1}{U}"), &["Illusion"], 0, 3).with_abilities(&[
        abilities::flash(),
        abilities::enters_trigger(
            "When this creature enters, empower Jace 3. (Put three loyalty counters on a Jace \
             token you control. If you don't control one, first create a blue Jace \
             planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(3)),
        ),
        AbilityDef::activated(
            "{U}, Sacrifice this creature: The next spell you cast this turn can't be countered.",
            &[CostDef::Mana(mana_cost!("{U}")), CostDef::SacrificeSource],
            EffectDef::Apply {
                recipient: EffectRecipientDef::Controller,
                effect: AppliedEffectDef::Rule(AppliedRuleDef::PlayerRule(
                    PlayerRuleDef::ApplyToMatchingSpell {
                        object: ObjectPredicateDef::Any,
                        effect: &AppliedEffectDef::Rule(AppliedRuleDef::CannotBeCountered),
                    },
                )),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn
                    .or(ResolvedEffectDurationDef::UntilNextMatchingCast),
            },
        ),
    ]),
);

// FRA 45 — Undulating Witness
pub(in crate::card::sets) static UNDULATING_WITNESS: CardRecord = CardRecord::new(
    "Undulating Witness",
    "0adbb4b2-a142-48da-8f4b-fa91529dbac4",
    "Alexander Ostrowski",
    CardRules::new_creature(mana_cost!("{4}{U}"), &["Serpent"], 3, 5).with_abilities(&[
        abilities::flying(),
        abilities::apply_to_self_until_end_of_turn(
            "{2}: This creature gets +1/-1 until end of turn.",
            &[CostDef::Mana(mana_cost!("{2}"))],
            AppliedEffectDef::modify_power_toughness(ValueDef::Constant(1), ValueDef::Constant(-1)),
        ),
        abilities::typecycling!(
            "Basic landcycling {2} ({2}, Discard this card: Search your library for a basic \
             land card, reveal it, put it into your hand, then shuffle.)",
            &[CostDef::Mana(mana_cost!("{2}"))],
            A_BASIC_LAND_CARD
        ),
    ]),
);

// FRA 46 — Unsummon (reprint)
const UNSUMMON_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_lea::UNSUMMON,
    "ddad9f16-52d5-49de-82b0-b1a5294a9c44",
    "Ioannis Fiore",
);

// FRA 47 — Variable Chaser // Arc of Fortune
const THE_ACTIVE_PLAYER: EffectRecipientDef =
    EffectRecipientDef::players(PlayerSetDef::Related(PlayerRelation::ActivePlayer));
const THE_NONACTIVE_PLAYER: EffectRecipientDef =
    EffectRecipientDef::players(PlayerSetDef::Related(PlayerRelation::NonactivePlayer));

/// "Discard their hand", for the players who chose to.
const fn discard_hand(players: EffectRecipientDef) -> EffectDef {
    EffectDef::Discard {
        recipient: players,
        // `Discard` stops at the size of the hand, so the largest amount
        // there is says "their hand".
        amount: ValueDef::Constant(i32::MAX),
        selection: DiscardSelectionDef::RecipientChooses,
        then: None,
    }
}

/// "... and draw seven cards", for a player whose hand is gone. A player an
/// opponent's spell cannot make discard (Tamiyo, Collector of Tales) cannot
/// take the option at all; their hand is still there, so having taken it does
/// nothing for them.
const fn draw_seven_once_hand_is_gone<const ACTIVE: bool>() -> EffectDef {
    EffectDef::IfCondition {
        condition: &const {
            TriggerConditionDef::ObjectCount {
                query: ObjectQueryDef::matching(
                    ObjectPredicateDef::Any,
                    &[ZoneKind::Hand],
                    if ACTIVE {
                        PlayerRelation::ActivePlayer
                    } else {
                        PlayerRelation::NonactivePlayer
                    },
                ),
                comparison: ComparisonDef::Equal,
                amount: 0,
            }
        },
        then: &const {
            EffectDef::DrawCards {
                recipient: if ACTIVE {
                    THE_ACTIVE_PLAYER
                } else {
                    THE_NONACTIVE_PLAYER
                },
                amount: ValueDef::Constant(7),
            }
        },
    }
}

pub(in crate::card::sets) static VARIABLE_CHASER: CardRecord = CardRecord::new_prepare(
    "Variable Chaser // Arc of Fortune",
    "e3afedb1-bf9d-4e31-9700-433514cc29b1",
    "Kai Carpenter",
    &[
        (
            "Variable Chaser",
            CardRules::new_creature(mana_cost!("{2}{U}"), &["Human", "Wizard"], 2, 3)
                .with_abilities(&[
                    abilities::flying(),
                    abilities::prowess(),
                    enters_prepared_with_reminder(),
                ]),
        ),
        (
            "Arc of Fortune",
            CardRules::new_sorcery(mana_cost!("{2}{U}")).with_ability(AbilityDef::spell(
                "Each player may discard their hand and draw seven cards.",
                // Both players answer, the active player first, before either
                // hand moves (CR 101.4), so the effect is two questions and
                // then one of four outcomes rather than one player's whole
                // turn followed by the other's. The first answer is public,
                // and no answer is published on its own, so the second
                // player's options are worded to say what it was.
                EffectDef::ChooseEffect {
                    player: THE_ACTIVE_PLAYER,
                    choices: &[
                        EffectChoiceDef {
                            label: "Discard your hand and draw seven cards",
                            effect: EffectDef::ChooseEffect {
                                player: THE_NONACTIVE_PLAYER,
                                choices: &[
                                    EffectChoiceDef {
                                        label: "Discard your hand and draw seven cards too",
                                        effect: EffectDef::Sequence(&[
                                            discard_hand(EffectRecipientDef::EachPlayer),
                                            draw_seven_once_hand_is_gone::<true>(),
                                            draw_seven_once_hand_is_gone::<false>(),
                                        ]),
                                    },
                                    EffectChoiceDef {
                                        label: "Keep your hand",
                                        effect: EffectDef::Sequence(&[
                                            discard_hand(THE_ACTIVE_PLAYER),
                                            draw_seven_once_hand_is_gone::<true>(),
                                        ]),
                                    },
                                ],
                            },
                        },
                        EffectChoiceDef {
                            label: "Keep your hand",
                            effect: EffectDef::ChooseEffect {
                                player: THE_NONACTIVE_PLAYER,
                                choices: &[
                                    EffectChoiceDef {
                                        label: "Discard your hand and draw seven cards",
                                        effect: EffectDef::Sequence(&[
                                            discard_hand(THE_NONACTIVE_PLAYER),
                                            draw_seven_once_hand_is_gone::<false>(),
                                        ]),
                                    },
                                    EffectChoiceDef {
                                        label: "Keep your hand too",
                                        effect: EffectDef::None,
                                    },
                                ],
                            },
                        },
                    ],
                },
            )),
        ),
    ],
);

// FRA 48 — Apex Witchstalker
pub(in crate::card::sets) static APEX_WITCHSTALKER: CardRecord = CardRecord::new(
    "Apex Witchstalker",
    "cd56f047-6bdc-4e83-8a7c-923ebad26302",
    "Jason Mowry",
    CardRules::new_creature(mana_cost!("{4}{B}{B}"), &["Wolf"], 6, 4).with_abilities(&[
        abilities::menace(),
        AbilityDef::triggered(
            "When this creature enters or dies, you gain 2 life.",
            TriggerEventDef::AnyOf(&[
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::Source,
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::Source,
                    Some(ZoneKind::Battlefield),
                    Some(ZoneKind::Graveyard),
                ),
            ]),
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(2),
            },
        ),
        abilities::typecycling!(
            "Basic landcycling {2} ({2}, Discard this card: Search your library for a basic \
             land card, reveal it, put it into your hand, then shuffle.)",
            &[CostDef::Mana(mana_cost!("{2}"))],
            A_BASIC_LAND_CARD
        ),
    ]),
);

// FRA 49 — Bloodline Recollector // Ancestral Craving
// Audit: unsupported — Bloodline Recollector needs the creatures-died-this-turn count readable in
// an intervening-if condition; the runtime evaluates it as 0 there.
pub(in crate::card::sets) static BLOODLINE_RECOLLECTOR: CardRecord = CardRecord::new_prepare(
    "Bloodline Recollector // Ancestral Craving",
    "4fcc913e-f736-460a-b24b-022fa2e861b9",
    "Cynthia Sheppard",
    &[
        ("Bloodline Recollector", CardRules::unsupported()),
        ("Ancestral Craving", CardRules::unsupported()),
    ],
);

// FRA 50 — Break Under Pressure
pub(in crate::card::sets) static BREAK_UNDER_PRESSURE: CardRecord = CardRecord::new(
    "Break Under Pressure",
    "46974d94-e900-43e4-92b5-4fb9b9f7cf46",
    "Jeff Miracola",
    CardRules::new_instant(mana_cost!("{2}{B}")).with_ability(AbilityDef::spell_with_targets(
        "Target opponent sacrifices a creature or planeswalker with the greatest mana value \
         among creatures and planeswalkers they control. You gain 2 life.",
        &[AbilityTargetDef::exactly_one(
            AbilityTargetPredicate::Player(PlayerRelation::Opponent),
        )],
        EffectDef::Sequence(&[
            // Creatures and planeswalkers are one group: the greatest mana value is read
            // across both, and only a permanent that has it can be chosen.
            EffectDef::Choose(ChooseDef {
                binding: ObjectChoiceBindingDef::Object(Binding!("sacrificed")),
                unchosen: None,
                chooser: PlayerRefDef::Target(TargetIndex::PRIMARY),
                candidates: ObjectSetDef::Query(ObjectQueryDef::controlled_by(
                    ObjectPredicateDef::All(&[
                        A_CREATURE_OR_PLANESWALKER,
                        ObjectPredicateDef::ManaValueEqualTo(ValueDef::AggregateObjectValues(
                            &ObjectValueAggregateDef {
                                objects: ObjectSetDef::Query(ObjectQueryDef::controlled_by(
                                    A_CREATURE_OR_PLANESWALKER,
                                    &[ZoneKind::Battlefield],
                                    PlayerSetDef::One(PlayerRefDef::Target(TargetIndex::PRIMARY)),
                                )),
                                select: ObjectValueDef::ManaValue,
                                operation: AggregateOperationDef::Maximum,
                            },
                        )),
                    ]),
                    &[ZoneKind::Battlefield],
                    PlayerSetDef::One(PlayerRefDef::Target(TargetIndex::PRIMARY)),
                )),
                exclude: None,
                minimum: 1,
                maximum: 1,
                visibility: ChoiceVisibilityDef::Public,
                then: &EffectDef::sacrifice(EffectRecipientDef::object(ObjectRefDef::Binding(
                    Binding!("sacrificed"),
                ))),
            }),
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(2),
            },
        ]),
    )),
);

// FRA 51 — Cast Away Doubt
pub(in crate::card::sets) static CAST_AWAY_DOUBT: CardRecord = CardRecord::new(
    "Cast Away Doubt",
    "3b5b28a4-0abd-4dc2-9856-c9a8d27f6a2d",
    "Zara Alfonso",
    CardRules::new_sorcery(mana_cost!("{2}{B}")).with_ability(AbilityDef::spell(
        "Draw two cards. Cast Away Doubt deals 2 damage to each player.",
        EffectDef::Sequence(&[
            abilities::draw_cards(ValueDef::Constant(2)),
            EffectDef::damage(EffectRecipientDef::EachPlayer, ValueDef::Constant(2)),
        ]),
    )),
);

// FRA 52 — Dark Matter Manipulator
pub(in crate::card::sets) static DARK_MATTER_MANIPULATOR: CardRecord = CardRecord::new(
    "Dark Matter Manipulator",
    "4ec912d5-cbe7-4d07-9ece-b03ac02d3055",
    "Steven Belledin",
    CardRules::new_creature(mana_cost!("{B}"), &["Human", "Warlock"], 1, 2).with_abilities(&[
        abilities::enters_trigger(
            "When this creature enters, mill three cards.",
            EffectDef::Mill {
                player: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(3),
            },
        ),
        AbilityDef::static_ability(
            "This creature gets +2/+0 for every seven cards in your graveyard.",
            EffectDef::StaticApply {
                recipient: EffectRecipientDef::Source,
                effect: AppliedEffectDef::modify_power_toughness(
                    ValueDef::Scaled(&ScaledValueDef::new(
                        ValueDef::Quotient(&QuotientValueDef::new(
                            ValueDef::CountMatchingObjects(&ObjectQueryDef::owned_by(
                                ObjectPredicateDef::Any,
                                &[ZoneKind::Graveyard],
                                PlayerSetDef::Related(PlayerRelation::You),
                            )),
                            ValueDef::Constant(7),
                            RoundingDef::Down,
                        )),
                        2,
                    )),
                    ValueDef::Constant(0),
                ),
            },
        ),
    ]),
);

// FRA 53 — Darklight Phoenix
// Audit: unsupported — Needs the creatures-died-this-turn count readable in an intervening-if
// condition; the runtime evaluates it as 0 there.
pub(in crate::card::sets) static DARKLIGHT_PHOENIX: CardRecord = CardRecord::new(
    "Darklight Phoenix",
    "ec454979-3839-4be3-a34a-9d25482948ba",
    "Wayne Reynolds",
    CardRules::unsupported(),
);

// FRA 54 — Extended Absence
pub(in crate::card::sets) static EXTENDED_ABSENCE: CardRecord = CardRecord::new(
    "Extended Absence",
    "eb4b6ed8-782e-4473-abc9-d50bf2275c6a",
    "Nathaniel Himawan",
    CardRules::new_instant(mana_cost!("{3}{B}")).with_ability(AbilityDef::spell_with_targets(
        "Exile target creature or planeswalker. Extended Absence deals 1 damage to each \
         opponent and you gain 1 life.",
        &[AbilityTargetDef::exactly_one_permanent(
            A_CREATURE_OR_PLANESWALKER,
        )],
        EffectDef::Sequence(&[
            EffectDef::move_to_zone(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ZoneKind::Exile,
                ZonePlacement::Top,
            ),
            EffectDef::damage(EffectRecipientDef::Opponent, ValueDef::Constant(1)),
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(1),
            },
        ]),
    )),
);

// FRA 55 — Extrapolate the Impossible
// Audit: unsupported — Needs revealing exactly two differently named cards from the sideboard,
// with an opponent choosing between them.
pub(in crate::card::sets) static EXTRAPOLATE_THE_IMPOSSIBLE: CardRecord = CardRecord::new(
    "Extrapolate the Impossible",
    "17fb6538-493c-41aa-ad13-3e63d3ad3317",
    "Volkan Baǵa",
    CardRules::unsupported(),
);

// FRA 56 — Last Gasp (reprint)
const LAST_GASP_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_rav::LAST_GASP,
    "2381d123-d8c7-4822-98fe-b1c365beb5ed",
    "Jabari Weathers",
);

// FRA 57 — Lich's Relic
pub(in crate::card::sets) static LICH_S_RELIC: CardRecord = CardRecord::new(
    "Lich's Relic",
    "b105511d-5022-4a84-b6ce-4bb433e93a62",
    "Gaboleps",
    CardRules::new_artifact(mana_cost!("{B}"))
        .with_subtypes(&["Equipment"])
        .with_abilities(&[
            abilities::enters_trigger(
                "When this Equipment enters, you may pay {2}. When you do, for each opponent, \
                 destroy up to one target creature or planeswalker that player controls.",
                EffectDef::PayOr(PayOrDef::optional(
                    &[CostDef::Mana(mana_cost!("{2}"))],
                    // One opponent in this engine, so "for each opponent, up to one target
                    // ... that player controls" is a single slot over what they control.
                    &EffectDef::ReflexiveTrigger(&AbilityDef::triggered_with_targets(
                        "When you do, for each opponent, destroy up to one target creature or \
                         planeswalker that player controls.",
                        TriggerEventDef::Reflexive,
                        &[AbilityTargetDef::up_to(
                            AbilityTargetPredicate::Object {
                                object: A_CREATURE_OR_PLANESWALKER,
                                zones: &[ZoneKind::Battlefield],
                                controller: Some(PlayerRelation::Opponent),
                                owner: None,
                            },
                            1,
                        )],
                        EffectDef::destroy_target(TargetIndex::PRIMARY),
                    )),
                )),
            ),
            AbilityDef::static_ability(
                "Equipped creature gets +2/+1.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::AttachedPermanent,
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(2),
                        ValueDef::Constant(1),
                    ),
                },
            ),
            abilities::equip(&[CostDef::Mana(mana_cost!("{2}"))], "Equip {2}"),
        ]),
);

// FRA 58 — Multiply by Zero
pub(in crate::card::sets) static MULTIPLY_BY_ZERO: CardRecord = CardRecord::new(
    "Multiply by Zero",
    "90d684a4-9639-4792-8760-2011a7a85370",
    "Florian Herold",
    CardRules::new_instant(mana_cost!("{1}{B}")).with_ability(AbilityDef::spell_with_targets(
        "Target creature has base power and toughness 0/0 until end of turn.",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::HasType(CardType::Creature),
        )],
        EffectDef::Apply {
            recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
            effect: AppliedEffectDef::set_base_power_toughness(
                ValueDef::Constant(0),
                ValueDef::Constant(0),
            ),
            duration: ResolvedEffectDurationDef::UntilEndOfTurn,
        },
    )),
);

// FRA 59 — Overwrite the Multiverse
// Audit: unsupported — Needs a count of the creatures the exile actually put into exile. A
// zone-move result binds every completed move, including one a replacement effect redirected to
// another zone, so "the number of creatures exiled this way" would count a creature that was
// never exiled.
pub(in crate::card::sets) static OVERWRITE_THE_MULTIVERSE: CardRecord = CardRecord::new(
    "Overwrite the Multiverse",
    "c4554f5b-791b-48f6-bf54-ad28699e1beb",
    "Ryan Pancoast",
    CardRules::unsupported(),
);

// FRA 60 — Rampart Hunter
pub(in crate::card::sets) static RAMPART_HUNTER: CardRecord = CardRecord::new(
    "Rampart Hunter",
    "f4a80225-7459-4151-86bb-8fdea31c39a6",
    "Néstor Ossandón Leal",
    CardRules::new_creature(mana_cost!("{3}{B}"), &["Horror"], 3, 3).with_abilities(&[
        abilities::deathtouch(),
        abilities::enters_trigger_with_targets(
            "When this creature enters, target creature gets +2/+2 and gains deathtouch until \
             end of turn.",
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::HasType(CardType::Creature),
            )],
            EffectDef::Apply {
                recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                effect: AppliedEffectDef::Composite(&[
                    AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(2),
                        ValueDef::Constant(2),
                    ),
                    AppliedEffectDef::add_ability(&abilities::deathtouch()),
                ]),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
        ),
    ]),
);

// FRA 61 — Rank Rat
pub(in crate::card::sets) static RANK_RAT: CardRecord = CardRecord::new(
    "Rank Rat",
    "4ff6da82-d7dd-4b59-b7e6-30670cea7169",
    "Johann Bodin",
    CardRules::new_creature(mana_cost!("{1}{B}"), &["Zombie", "Rat"], 1, 1).with_ability(
        abilities::enters_trigger(
            "When this creature enters, each opponent discards a card.",
            EffectDef::Discard {
                recipient: EffectRecipientDef::Opponent,
                amount: ValueDef::Constant(1),
                selection: DiscardSelectionDef::RecipientChooses,
                then: None,
            },
        ),
    ),
);

// FRA 62 — Rewrite Regrets
pub(in crate::card::sets) static REWRITE_REGRETS: CardRecord = CardRecord::new(
    "Rewrite Regrets",
    "453cfde7-c460-4b55-9472-b714e16f24bb",
    "Matt Stewart",
    CardRules::new_sorcery(mana_cost!("{3}{B}")).with_abilities(&[
        AbilityDef::spell_with_targets(
            "Return target creature or planeswalker card with mana value 6 or less from your \
             graveyard to the battlefield.",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::All(&[
                        A_CREATURE_OR_PLANESWALKER,
                        ObjectPredicateDef::ManaValueAtMost(6),
                    ]),
                    zones: &[ZoneKind::Graveyard],
                    controller: None,
                    owner: Some(PlayerRelation::You),
                },
            )],
            EffectDef::move_to_zone(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ZoneKind::Battlefield,
                ZonePlacement::Top,
            ),
        ),
        AbilityDef::spell(
            "Empower Jace 2. (Put two loyalty counters on a Jace token you control. If you \
             don't control one, first create a blue Jace planeswalker token with \
             \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(2)),
        ),
    ]),
);

// FRA 63 — Rise of the Deathbringer
// Audit: unsupported — Needs a value for the number of cards actually drawn this way.
pub(in crate::card::sets) static RISE_OF_THE_DEATHBRINGER: CardRecord = CardRecord::new(
    "Rise of the Deathbringer",
    "811719ad-b5a3-4d31-8c6f-5dbdfccf7c1f",
    "Lie Setiawan",
    CardRules::unsupported(),
);

// FRA 64 — Sanctum Lurker
// Audit: unsupported — Needs suppression of the 0-loyalty state-based action for planeswalkers you
// control.
pub(in crate::card::sets) static SANCTUM_LURKER: CardRecord = CardRecord::new(
    "Sanctum Lurker",
    "2185c08f-bb4d-49d5-8b6c-c629a48bb61c",
    "Joshua Raphael",
    CardRules::unsupported(),
);

// FRA 65 — Screeching Soulbreaker
pub(in crate::card::sets) static SCREECHING_SOULBREAKER: CardRecord = CardRecord::new(
    "Screeching Soulbreaker",
    "738667a1-c184-43ea-829f-49fbb69b6fc0",
    "Pavel Kolomeyets",
    CardRules::new_creature(mana_cost!("{2}{B}"), &["Siren", "Bard"], 1, 4).with_abilities(&[
        abilities::flying(),
        AbilityDef::triggered(
            "Whenever this creature attacks, it deals 1 damage to each opponent and you gain 1 \
             life.",
            TriggerEventDef::attacks(ObjectPredicateDef::Source),
            EffectDef::Sequence(&[
                EffectDef::damage(EffectRecipientDef::Opponent, ValueDef::Constant(1)),
                EffectDef::GainLife {
                    recipient: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(1),
                },
            ]),
        ),
    ]),
);

// FRA 66 — Silence the Echo
pub(in crate::card::sets) static SILENCE_THE_ECHO: CardRecord = CardRecord::new(
    "Silence the Echo",
    "f1d274db-751b-4414-a38d-762198168e91",
    "Pavel Kolomeyets",
    CardRules::new_sorcery(mana_cost!("{1}{B}")).with_ability(
        AbilityDef::spell_with_targets(
            "As an additional cost to cast this spell, sacrifice a creature or planeswalker or \
             pay {3}.\nDestroy target creature or planeswalker.",
            &[AbilityTargetDef::exactly_one_permanent(
                A_CREATURE_OR_PLANESWALKER,
            )],
            EffectDef::destroy_target(TargetIndex::PRIMARY),
        )
        .with_spell_additional_cost(&CostDef::Choice(&[
            CostDef::Sacrifice {
                object: A_CREATURE_OR_PLANESWALKER,
                quantity: CostQuantityDef::Fixed(1),
            },
            CostDef::Mana(mana_cost!("{3}")),
        ])),
    ),
);

// FRA 67 — Solve for Disappointment
pub(in crate::card::sets) static SOLVE_FOR_DISAPPOINTMENT: CardRecord = CardRecord::new(
    "Solve for Disappointment",
    "7beaa8c9-1a2c-4c88-b579-91e371d8d9e3",
    "Eli Minaya",
    CardRules::new_sorcery(mana_cost!("{1}{B}")).with_abilities(&[
        AbilityDef::spell_with_targets(
            "Target opponent reveals their hand. You choose a nonland permanent card from it. \
             That player discards that card.",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Player(PlayerRelation::Opponent),
            )],
            EffectDef::Sequence(&abilities::reveal_hand_and_discard_chosen_card(
                PlayerRefDef::Target(TargetIndex::PRIMARY),
                // A permanent card that is not a land. An artifact land is a permanent
                // card and still a land.
                ObjectPredicateDef::All(&[
                    ObjectPredicateDef::AnyOf(&[
                        ObjectPredicateDef::HasType(CardType::Artifact),
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::HasType(CardType::Enchantment),
                        ObjectPredicateDef::HasType(CardType::Planeswalker),
                    ]),
                    ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Land)),
                ]),
            )),
        ),
        AbilityDef::spell(
            "Empower Jace 1. (Put a loyalty counter on a Jace token you control. If you \
             don't control one, first create a blue Jace planeswalker token with \
             \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(1)),
        ),
    ]),
);

// FRA 68 — Terminal Criticism
pub(in crate::card::sets) static TERMINAL_CRITICISM: CardRecord = CardRecord::new(
    "Terminal Criticism",
    "7ebd7e38-b27c-4c6e-aaea-e8ee5ba5e5df",
    "Borja Pindado",
    CardRules::new_instant(mana_cost!("{1}{B}")).with_ability(AbilityDef::spell_with_targets(
        "Destroy target creature or planeswalker that's blue or red. You gain 1 life.",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::All(&[
                A_CREATURE_OR_PLANESWALKER,
                ObjectPredicateDef::AnyOf(&[
                    ObjectPredicateDef::Color(ManaColor::Blue),
                    ObjectPredicateDef::Color(ManaColor::Red),
                ]),
            ]),
        )],
        EffectDef::Sequence(&[
            EffectDef::destroy_target(TargetIndex::PRIMARY),
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(1),
            },
        ]),
    )),
);

// FRA 69 — Theoretical Necromancer
pub(in crate::card::sets) static THEORETICAL_NECROMANCER: CardRecord = CardRecord::new(
    "Theoretical Necromancer",
    "e36a7908-1e22-494b-adb4-e72ac0974d62",
    "Chris Rallis",
    CardRules::new_creature(mana_cost!("{2}{B}"), &["Vampire", "Warlock"], 4, 1).with_ability(
        AbilityDef::activated_with_targets(
            "{3}{B}, Exile this card from your graveyard: Return another target creature card \
             from your graveyard to your hand.",
            &[CostDef::Mana(mana_cost!("{3}{B}")), CostDef::ExileSource],
            &[
                AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::HasType(CardType::Creature),
                    zones: &[ZoneKind::Graveyard],
                    controller: None,
                    owner: Some(PlayerRelation::You),
                })
                .excluding_source(),
            ],
            EffectDef::move_to_zone(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ZoneKind::Hand,
                ZonePlacement::Top,
            ),
        )
        .with_source_zones(&[ZoneKind::Graveyard]),
    ),
);

// FRA 70 — Void Extrapolator // Omit Variables
pub(in crate::card::sets) static VOID_EXTRAPOLATOR: CardRecord = CardRecord::new_prepare(
    "Void Extrapolator // Omit Variables",
    "0eae2efb-bf25-48ee-9c07-9098008110ad",
    "Valera Lutfullina",
    &[
        (
            "Void Extrapolator",
            CardRules::new_creature(mana_cost!("{1}{B}"), &["Aetherborn", "Warlock"], 2, 2)
                .with_abilities(&crate::ability_list![
                    [enters_prepared_with_reminder()],
                    threshold(&[AbilityDef::static_ability(
                        "Threshold — This creature gets +1/+1 as long as there are seven or \
                         more cards in your graveyard.",
                        EffectDef::StaticApply {
                            recipient: EffectRecipientDef::Source,
                            effect: AppliedEffectDef::modify_power_toughness(
                                ValueDef::Constant(1),
                                ValueDef::Constant(1),
                            ),
                        },
                    )]),
                ]),
        ),
        ("Omit Variables", OMIT_VARIABLES),
    ],
);

// FRA 71 — Vraska's Final Mercy
pub(in crate::card::sets) static VRASKA_S_FINAL_MERCY: CardRecord = CardRecord::new(
    "Vraska's Final Mercy",
    "992bd991-7cfb-459f-bafd-9a44f3c925c5",
    "Magali Villeneuve",
    CardRules::new_sorcery(mana_cost!("{B}{B}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "You lose 2 life. Destroy target creature or planeswalker.",
                &[AbilityTargetDef::exactly_one_permanent(
                    A_CREATURE_OR_PLANESWALKER,
                )],
                EffectDef::Sequence(&[
                    EffectDef::LoseLife {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(2),
                    },
                    EffectDef::destroy_target(TargetIndex::PRIMARY),
                ]),
            ),
            AbilityDef::spell(
                "You lose 2 life. Empower Jace 6. (Put six loyalty counters on a Jace token \
                 you control. If you don't control one, first create a blue Jace planeswalker \
                 token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
                EffectDef::Sequence(&[
                    EffectDef::LoseLife {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(2),
                    },
                    empower_jace(ValueDef::Constant(6)),
                ]),
            ),
        ],
    )),
);

// FRA 72 — Ajani's Anguish
pub(in crate::card::sets) static AJANI_S_ANGUISH: CardRecord = CardRecord::new(
    "Ajani's Anguish",
    "d9039a58-2f17-4b8a-b714-3a2f0b46f057",
    "Mathias Kollros",
    CardRules::new_enchantment(mana_cost!("{X}{R}")).with_abilities(&[
        abilities::enters_trigger_with_targets(
            "When this enchantment enters, it deals X damage to any target.",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::AnyTarget,
            )],
            EffectDef::damage(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ValueDef::SourceCastX,
            ),
        ),
        AbilityDef::static_ability(
            "Creatures you control have trample.",
            EffectDef::StaticApply {
                recipient: EffectRecipientDef::matching_objects(
                    ObjectPredicateDef::HasType(CardType::Creature),
                    &[ZoneKind::Battlefield],
                    PlayerRelation::You,
                ),
                effect: AppliedEffectDef::add_ability(&abilities::trample()),
            },
        ),
    ]),
);

// FRA 73 — Artifist Acumen
pub(in crate::card::sets) static ARTIFIST_ACUMEN: CardRecord = CardRecord::new(
    "Artifist Acumen",
    "7d3b720d-f27c-462a-8f80-15748e5086e1",
    "Anna Podedworna",
    CardRules::new_sorcery(mana_cost!("{R}")).with_abilities(&[
        AbilityDef::spell(
            "Creatures you control gain first strike until end of turn.",
            EffectDef::Apply {
                recipient: EffectRecipientDef::matching_objects(
                    ObjectPredicateDef::HasType(CardType::Creature),
                    &[ZoneKind::Battlefield],
                    PlayerRelation::You,
                ),
                effect: AppliedEffectDef::add_ability(&abilities::first_strike()),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
        ),
        AbilityDef::spell("Draw a card.", abilities::draw_cards(ValueDef::Constant(1))),
    ]),
);

// FRA 74 — Awaken the Inferno
pub(in crate::card::sets) static AWAKEN_THE_INFERNO: CardRecord = CardRecord::new(
    "Awaken the Inferno",
    "c596c4ec-8480-4be9-a45d-700398a126f6",
    "Kai Carpenter",
    CardRules::new_sorcery(mana_cost!("{4}{R}")).with_abilities(&[
        AbilityDef::spell_with_targets(
            "Awaken the Inferno deals 6 damage to target creature or planeswalker an opponent \
             controls. Put a +1/+1 counter on up to one target creature you control.",
            &[
                AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                    object: A_CREATURE_OR_PLANESWALKER,
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::Opponent),
                    owner: None,
                }),
                AbilityTargetDef::up_to(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Creature),
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    },
                    1,
                ),
            ],
            EffectDef::Sequence(&[
                EffectDef::damage(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ValueDef::Constant(6),
                ),
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Target(TargetIndex(1)),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
            ]),
        ),
        abilities::typecycling!(
            "Basic landcycling {2} ({2}, Discard this card: Search your library for a basic \
             land card, reveal it, put it into your hand, then shuffle.)",
            &[CostDef::Mana(mana_cost!("{2}"))],
            A_BASIC_LAND_CARD,
        ),
    ]),
);

// FRA 75 — Blazing Crescendo (reprint)
const BLAZING_CRESCENDO_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_one::BLAZING_CRESCENDO,
    "8414a98c-0c79-4884-bc9b-061a6456b392",
    "Billy Christian",
);

// FRA 76 — Chandra's Emberling
pub(in crate::card::sets) static CHANDRA_S_EMBERLING: CardRecord = CardRecord::new(
    "Chandra's Emberling",
    "3a64c5f4-9cfc-4d19-bd99-13d619b1aa9d",
    "Borja Pindado",
    CardRules::new_creature(mana_cost!("{2}{R}"), &["Gremlin", "Elemental"], 2, 2).with_abilities(
        &[
            abilities::haste(),
            AbilityDef::triggered(
                "Whenever you cast a noncreature spell, put a +1/+1 counter on this creature.",
                TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                    ObjectPredicateDef::NoncreatureSpell,
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ])),
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Source,
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
            ),
        ],
    ),
);

// FRA 77 — Command the Stage
// Audit: unsupported — Needs a history flag recording that an opponent was dealt noncombat damage
// during the previous turn.
pub(in crate::card::sets) static COMMAND_THE_STAGE: CardRecord = CardRecord::new(
    "Command the Stage",
    "f3307da2-6dad-4ef2-9614-a7d34f38088e",
    "Jarel Threat",
    CardRules::unsupported(),
);

// FRA 78 — Craterclaw Colossus
pub(in crate::card::sets) static CRATERCLAW_COLOSSUS: CardRecord = CardRecord::new(
    "Craterclaw Colossus",
    "47793a51-08c6-4ad2-a7e5-a4484d83a5cd",
    "Mark Zug",
    CardRules::new_artifact_creature(mana_cost!("{4}{R}{R}{R}"), &["Beast", "Construct"], 5, 5)
        .with_abilities(&[
            abilities::haste(),
            abilities::enters_trigger(
                "When this creature enters, creatures you control gain trample and get +X/+0 \
                 until end of turn, where X is the number of artifacts you control.",
                EffectDef::Apply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::HasType(CardType::Creature),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::add_ability(&abilities::trample()),
                        AppliedEffectDef::modify_power_toughness(
                            ValueDef::CountMatchingObjects(&ObjectQueryDef::controlled_by(
                                ObjectPredicateDef::HasType(CardType::Artifact),
                                &[ZoneKind::Battlefield],
                                PlayerSetDef::Related(PlayerRelation::You),
                            )),
                            ValueDef::Constant(0),
                        ),
                    ]),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ]),
);

// FRA 79 — Curse-Marred Demon
pub(in crate::card::sets) static CURSE_MARRED_DEMON: CardRecord = CardRecord::new(
    "Curse-Marred Demon",
    "8f827e50-0a08-4bc8-98b1-b26c9af15ef2",
    "Kev Walker",
    CardRules::new_creature(mana_cost!("{2}{R}{R}"), &["Demon"], 4, 4).with_abilities(&[
        abilities::flying(),
        abilities::trample(),
        abilities::enters_trigger(
            "When this creature enters, search your library for a card, put it into your hand, \
             shuffle, then discard a card at random.",
            EffectDef::Sequence(&[
                EffectDef::SearchZone {
                    exile_face_down: false,
                    player: EffectRecipientDef::Controller,
                    source: ZoneKind::Library,
                    object: ObjectPredicateDef::Any,
                    minimum: 1,
                    maximum: ValueDef::Constant(1),
                    reveal: false,
                    destination: ZoneKind::Hand,
                    placement: ZonePlacement::Top,
                    shuffle: true,
                    enters_tapped: false,
                    attachment: None,
                    binding: None,
                    then: None,
                },
                EffectDef::Discard {
                    recipient: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(1),
                    selection: DiscardSelectionDef::Random,
                    then: None,
                },
            ]),
        ),
    ]),
);

// FRA 80 — Draconic Visitor
// Audit: unsupported — Needs a token-creation replacement that substitutes different token
// characteristics.
pub(in crate::card::sets) static DRACONIC_VISITOR: CardRecord = CardRecord::new(
    "Draconic Visitor",
    "112f8478-bd89-4a14-9721-8ab750613129",
    "Alexander Mokhov",
    CardRules::unsupported(),
);

// FRA 81 — Eardrum Rattler
pub(in crate::card::sets) static EARDRUM_RATTLER: CardRecord = CardRecord::new(
    "Eardrum Rattler",
    "5f8771f9-8128-4818-a11d-41ea368cf697",
    "Javier Charro",
    CardRules::new_creature(mana_cost!("{1}{R}"), &["Human", "Bard"], 2, 2).with_ability(
        AbilityDef::activated_with_targets(
            "{1}, {T}: Another target creature you control with power 2 or less can't be \
             blocked this turn.",
            &[CostDef::Mana(mana_cost!("{1}")), CostDef::TapSource],
            &[
                AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::Not(&ObjectPredicateDef::PowerAtLeast(3)),
                    ]),
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::You),
                    owner: None,
                })
                .excluding_source(),
            ],
            EffectDef::Apply {
                recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                effect: AppliedEffectDef::Rule(AppliedRuleDef::cannot_be_blocked_by(
                    ObjectPredicateDef::Any,
                )),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
        ),
    ),
);

// FRA 82 — Essence Burn
pub(in crate::card::sets) static ESSENCE_BURN: CardRecord = CardRecord::new(
    "Essence Burn",
    "d2e958de-70de-4156-8f9b-b2c0c1ba704a",
    "Javier Charro",
    CardRules::new_instant(mana_cost!("{1}{R}")).with_ability(AbilityDef::spell_with_targets(
        "Essence Burn deals 5 damage to target black or green creature or planeswalker. If \
         that permanent would die this turn, exile it instead.",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::All(&[
                A_CREATURE_OR_PLANESWALKER,
                ObjectPredicateDef::AnyOf(&[
                    ObjectPredicateDef::Color(ManaColor::Black),
                    ObjectPredicateDef::Color(ManaColor::Green),
                ]),
            ]),
        )],
        EffectDef::Sequence(&[
            EffectDef::damage(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ValueDef::Constant(5),
            ),
            EffectDef::Apply {
                recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                effect: AppliedEffectDef::Rule(AppliedRuleDef::ExileInsteadOfDying),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
        ]),
    )),
);

// FRA 83 — Face Yourself
pub(in crate::card::sets) static FACE_YOURSELF: CardRecord = CardRecord::new(
    "Face Yourself",
    "3ccf8f64-19bd-4fdf-b70a-30a042bacf2f",
    "Ekaterina Burmak",
    CardRules::new_sorcery(mana_cost!("{5}{R}{R}")).with_ability(AbilityDef::spell_with_targets(
        "For each creature target player controls, create a token that's a copy of that \
         creature, except it has haste and \"At the beginning of the end step, if you don't \
         control a planeswalker, sacrifice this creature.\"",
        &[AbilityTargetDef::exactly_one(
            AbilityTargetPredicate::Player(PlayerRelation::Any),
        )],
        EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Copy(&TokenCopyDef {
            object: &EffectRecipientDef::objects(ObjectSetDef::Query(
                ObjectQueryDef::controlled_by(
                    ObjectPredicateDef::HasType(CardType::Creature),
                    &[ZoneKind::Battlefield],
                    PlayerSetDef::LegalTargets(TargetIndex::PRIMARY),
                ),
            )),
            exceptions: CopyExceptionsDef {
                added_abilities: &[
                    CopyAbilityDef::Ability(&abilities::haste()),
                    CopyAbilityDef::Ability(&AbilityDef::triggered_if(
                        "At the beginning of the end step, if you don't control a planeswalker, \
                         sacrifice this creature.",
                        TriggerEventDef::StepBegins {
                            step: TurnStepDef::End,
                            player: PlayerRelation::Any,
                        },
                        &TriggerConditionDef::ObjectCount {
                            query: ObjectQueryDef::controlled_by(
                                ObjectPredicateDef::HasType(CardType::Planeswalker),
                                &[ZoneKind::Battlefield],
                                PlayerSetDef::Related(PlayerRelation::You),
                            ),
                            comparison: ComparisonDef::Equal,
                            amount: 0,
                        },
                        EffectDef::sacrifice(EffectRecipientDef::Source),
                    )),
                ],
                ..CopyExceptionsDef::NONE
            },
        }))),
    )),
);

// FRA 84 — Fulminous Forte
pub(in crate::card::sets) static FULMINOUS_FORTE: CardRecord = CardRecord::new(
    "Fulminous Forte",
    "19acb2b5-3b3e-43f0-bd81-8426ed3d9c55",
    "Andre Garcia",
    CardRules::new_instant(mana_cost!("{2}{R}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell(
                "Fulminous Forte deals 1 damage to each creature and planeswalker your opponents \
                 control.",
                EffectDef::damage(
                    EffectRecipientDef::matching_objects(
                        A_CREATURE_OR_PLANESWALKER,
                        &[ZoneKind::Battlefield],
                        PlayerRelation::Opponent,
                    ),
                    ValueDef::Constant(1),
                ),
            ),
            AbilityDef::spell_with_targets(
                "Fulminous Forte deals 5 damage to target creature or planeswalker.",
                &[AbilityTargetDef::exactly_one_permanent(
                    A_CREATURE_OR_PLANESWALKER,
                )],
                EffectDef::damage(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ValueDef::Constant(5),
                ),
            ),
        ],
    )),
);

// FRA 85 — Hallway Heckler // Vicious Verse
pub(in crate::card::sets) static HALLWAY_HECKLER: CardRecord = CardRecord::new_prepare(
    "Hallway Heckler // Vicious Verse",
    "7e324816-552f-455d-97c4-5ea6b26d2e6e",
    "Gustavo Pelissari",
    &[
        (
            "Hallway Heckler",
            CardRules::new_creature(mana_cost!("{2}{R}"), &["Elemental", "Sorcerer"], 2, 3)
                .with_abilities(&[
                    enters_prepared_with_reminder(),
                    AbilityDef::activated(
                        "{T}, Discard a card: Draw a card.",
                        &[
                            CostDef::TapSource,
                            CostDef::discard(ObjectPredicateDef::Any),
                        ],
                        abilities::draw_cards(ValueDef::Constant(1)),
                    ),
                ]),
        ),
        ("Vicious Verse", VICIOUS_VERSE),
    ],
);

// FRA 86 — Heartstring Puller
pub(in crate::card::sets) static HEARTSTRING_PULLER: CardRecord = CardRecord::new(
    "Heartstring Puller",
    "cbfe3354-7ced-4773-9a4e-a937ae9f94f8",
    "Jessica Liu",
    CardRules::new_creature(mana_cost!("{3}{R}"), &["Elf", "Sorcerer"], 3, 1).with_abilities(&[
        abilities::trample(),
        abilities::enters_trigger(
            "When this creature enters, create a 2/2 colorless Wizard Soldier creature token \
             named Cadet.",
            EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(CADET_TOKEN))),
        ),
    ]),
);

// FRA 87 — Identity Echo
pub(in crate::card::sets) static IDENTITY_ECHO: CardRecord = CardRecord::new(
    "Identity Echo",
    "e600b33b-8916-43dd-95d3-d7cbf874933d",
    "Aldo Domínguez",
    CardRules::new_enchantment(mana_cost!("{2}{R}")).with_ability(
        AbilityDef::activated_with_targets(
            "{3}{R}: Exile target creature or planeswalker you control. Reveal cards from the \
             top of your library until you reveal a creature or planeswalker card. Put that \
             card onto the battlefield and the rest on the bottom of your library in a random \
             order. Activate only as a sorcery.",
            &[CostDef::Mana(mana_cost!("{3}{R}"))],
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: A_CREATURE_OR_PLANESWALKER,
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::You),
                    owner: None,
                },
            )],
            EffectDef::Sequence(&[
                EffectDef::move_to_zone(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ZoneKind::Exile,
                    ZonePlacement::Top,
                ),
                EffectDef::RevealAndClassifyCards(RevealAndClassifyCardsDef {
                    source: ObjectCollectionSourceDef::TopCardsThroughFirstMatching {
                        player: PlayerRefDef::EffectController,
                        object: A_CREATURE_OR_PLANESWALKER,
                    },
                    object: A_CREATURE_OR_PLANESWALKER,
                    matching: Binding!("found"),
                    remainder: Binding!("rest"),
                    then: &EffectDef::Sequence(&[
                        EffectDef::move_to_zone(
                            EffectRecipientDef::objects(ObjectSetDef::Binding(Binding!("found"))),
                            ZoneKind::Battlefield,
                            ZonePlacement::Top,
                        ),
                        EffectDef::RandomizeObjectOrder(RandomizeObjectOrderDef {
                            input: ObjectSetDef::Binding(Binding!("rest")),
                            randomized: Binding!("bottom"),
                            then: &EffectDef::move_to_zone(
                                EffectRecipientDef::objects(ObjectSetDef::Binding(Binding!(
                                    "bottom"
                                ))),
                                ZoneKind::Library,
                                ZonePlacement::Bottom,
                            ),
                        }),
                    ]),
                }),
            ]),
        )
        .with_activation_timing(ActivationTimingDef::SorcerySpeed),
    ),
);

// FRA 88 — Master of Barbs
// Audit: unsupported — Needs a batched trigger for one or more opponents being dealt noncombat
// damage.
pub(in crate::card::sets) static MASTER_OF_BARBS: CardRecord = CardRecord::new(
    "Master of Barbs",
    "4404d9d4-9cdd-4dad-a4f6-574d90db5052",
    "Aldo Domínguez",
    CardRules::unsupported(),
);

// FRA 89 — No Admittance
pub(in crate::card::sets) static NO_ADMITTANCE: CardRecord = CardRecord::new(
    "No Admittance",
    "11ba4fdd-cc03-4bb6-a493-91a9785771d0",
    "Serena Malyon",
    CardRules::new_sorcery(mana_cost!("{1}{R}")).with_abilities(&[
        AbilityDef::spell_with_targets(
            "No Admittance deals 3 damage to any target.",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::AnyTarget,
            )],
            EffectDef::damage(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ValueDef::Constant(3),
            ),
        ),
        AbilityDef::spell(
            "Empower Jace 1. (Put a loyalty counter on a Jace token you control. If you \
             don't control one, first create a blue Jace planeswalker token with \
             \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(1)),
        ),
    ]),
);

// FRA 90 — Pompous Battlemage // Improvised Act
pub(in crate::card::sets) static POMPOUS_BATTLEMAGE: CardRecord = CardRecord::new_prepare(
    "Pompous Battlemage // Improvised Act",
    "aa0f77ac-741a-444a-8bf0-a42c644726bf",
    "Justin Gerard",
    &[
        (
            "Pompous Battlemage",
            CardRules::new_creature(mana_cost!("{R}"), &["Goblin", "Sorcerer"], 1, 1)
                .with_abilities(&[abilities::prowess(), enters_prepared_with_reminder()]),
        ),
        (
            "Improvised Act",
            CardRules::new_sorcery(mana_cost!("{R}")).with_ability(AbilityDef::spell(
                "You may discard a card. If you do, draw a card.",
                EffectDef::PayOr(PayOrDef::optional(
                    &[CostDef::discard(ObjectPredicateDef::Any)],
                    &abilities::draw_cards(ValueDef::Constant(1)),
                )),
            )),
        ),
    ],
);

// FRA 91 — Pyre Rhymer // Molten Tide
// Audit: unsupported — Molten Tide needs an until-end-of-turn triggered mana ability that adds
// an additional {R} whenever its controller taps a Mountain for mana.
pub(in crate::card::sets) static PYRE_RHYMER: CardRecord = CardRecord::new_prepare(
    "Pyre Rhymer // Molten Tide",
    "2b0ebea0-86de-4da4-9fe8-dacc1e75c161",
    "Andrea Piparo",
    &[
        ("Pyre Rhymer", CardRules::unsupported()),
        ("Molten Tide", CardRules::unsupported()),
    ],
);

// FRA 92 — Skilled Battlecarver
pub(in crate::card::sets) static SKILLED_BATTLECARVER: CardRecord = CardRecord::new(
    "Skilled Battlecarver",
    "0e44f959-1322-4abd-b6eb-dea992307c0c",
    "Evyn Fong",
    CardRules::new_creature(mana_cost!("{1}{R}"), &["Human", "Warrior"], 2, 1).with_abilities(&[
        AbilityDef::static_ability(
            "During your turn, this creature has first strike.",
            EffectDef::IfCondition {
                condition: &TriggerConditionDef::ActivePlayer(PlayerRelation::You),
                then: &EffectDef::StaticApply {
                    recipient: EffectRecipientDef::Source,
                    effect: AppliedEffectDef::add_ability(&abilities::first_strike()),
                },
            },
        ),
        abilities::apply_to_self_until_end_of_turn(
            "{1}{R}: This creature gets +1/+0 until end of turn.",
            &[CostDef::Mana(mana_cost!("{1}{R}"))],
            AppliedEffectDef::modify_power_toughness(ValueDef::Constant(1), ValueDef::Constant(0)),
        ),
    ]),
);

// FRA 93 — Stingcaster Mage
pub(in crate::card::sets) static STINGCASTER_MAGE: CardRecord = CardRecord::new(
    "Stingcaster Mage",
    "2d8e8e5f-3bf5-490d-aa9c-9df2b26f1460",
    "Matt Stewart",
    CardRules::new_creature(mana_cost!("{1}{R}"), &["Human", "Wizard"], 2, 1).with_abilities(&[
        abilities::haste(),
        abilities::enters_trigger_with_targets(
            "When this creature enters, target instant or sorcery card in your graveyard gains \
             flashback until end of turn. The flashback cost is equal to its mana cost. (You \
             may cast that card from your graveyard for its flashback cost. Then exile it.)",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::AnyOf(&[
                        ObjectPredicateDef::HasType(CardType::Instant),
                        ObjectPredicateDef::HasType(CardType::Sorcery),
                    ]),
                    zones: &[ZoneKind::Graveyard],
                    controller: None,
                    owner: Some(PlayerRelation::You),
                },
            )],
            EffectDef::Apply {
                recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                effect: AppliedEffectDef::add_ability(&abilities::flashback_for_card_mana_cost()),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
        ),
    ]),
);

// FRA 94 — Tether Technician
pub(in crate::card::sets) static TETHER_TECHNICIAN: CardRecord = CardRecord::new(
    "Tether Technician",
    "b75bbf46-a421-467a-9433-6cf22398a3a5",
    "Victor Maury",
    CardRules::new_creature(mana_cost!("{4}{R}"), &["Minotaur", "Artificer"], 4, 5).with_abilities(
        &[
            abilities::reach(),
            abilities::enters_trigger(
                "When this creature enters, you may discard a card. When you do, this creature \
                 deals 2 damage to any target.",
                EffectDef::May {
                    player: EffectRecipientDef::Controller,
                    effect: &EffectDef::Discard {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(1),
                        selection: DiscardSelectionDef::RecipientChooses,
                        then: Some(DiscardFollowUpDef {
                            counted: ObjectPredicateDef::Any,
                            bound: Some(Binding!("discarded")),
                            effect: &EffectDef::IfNoObjects(IfNoObjectsDef {
                                input: ObjectSetDef::Binding(Binding!("discarded")),
                                if_empty: &EffectDef::None,
                                otherwise: &EffectDef::ReflexiveTrigger(
                                    &AbilityDef::triggered_with_targets(
                                        "When you do, this creature deals 2 damage to any target.",
                                        TriggerEventDef::Reflexive,
                                        &[AbilityTargetDef::exactly_one(
                                            AbilityTargetPredicate::AnyTarget,
                                        )],
                                        EffectDef::damage(
                                            EffectRecipientDef::Target(TargetIndex::PRIMARY),
                                            ValueDef::Constant(2),
                                        ),
                                    ),
                                ),
                            }),
                        }),
                    },
                },
            ),
        ],
    ),
);

// FRA 95 — Violent Echoes
// Audit: unsupported — Needs the excess damage dealt by an ordinary damage instruction as a value.
pub(in crate::card::sets) static VIOLENT_ECHOES: CardRecord = CardRecord::new(
    "Violent Echoes",
    "ad03ba90-2442-4a71-94df-2088b5b63662",
    "Andrea Piparo",
    CardRules::unsupported(),
);

// FRA 96 — Wrath of the Bloodmane
pub(in crate::card::sets) static WRATH_OF_THE_BLOODMANE: CardRecord = CardRecord::new(
    "Wrath of the Bloodmane",
    "b5b55617-684a-4036-be9b-a3b24fc9cd5a",
    "Manuel Castañón",
    CardRules::new_instant(mana_cost!("{2}{R}")).with_abilities(&[
        abilities::this_spell_cost_reduction(
            "This spell costs {1} less to cast if you control a legendary creature.",
            ValueDef::IfMatchingObjectCount(&CountConditionDef {
                query: ObjectQueryDef::matching(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::Supertype(CardSupertype::Legendary),
                        ObjectPredicateDef::HasType(CardType::Creature),
                    ]),
                    &[ZoneKind::Battlefield],
                    PlayerRelation::You,
                ),
                comparison: ComparisonDef::GreaterOrEqual,
                amount: 1,
                then: ValueDef::Constant(1),
                otherwise: ValueDef::Constant(0),
            }),
        ),
        AbilityDef::spell_with_targets(
            "Wrath of the Bloodmane deals 4 damage to target creature or planeswalker.",
            &[AbilityTargetDef::exactly_one_permanent(
                A_CREATURE_OR_PLANESWALKER,
            )],
            EffectDef::damage(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ValueDef::Constant(4),
            ),
        ),
    ]),
);

// FRA 97 — Arcane Amphisbaena
pub(in crate::card::sets) static ARCANE_AMPHISBAENA: CardRecord = CardRecord::new(
    "Arcane Amphisbaena",
    "1bf923c4-f0b7-4271-978c-fd2e79fe1cc8",
    "Jason Mowry",
    CardRules::new_creature(mana_cost!("{1}{G}"), &["Snake"], 1, 1).with_abilities(&[
        abilities::deathtouch(),
        abilities::enters_trigger(
            "When this creature enters, empower Jace 2. (Put two loyalty counters on a Jace \
             token you control. If you don't control one, first create a blue Jace \
             planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(2)),
        ),
    ]),
);

// FRA 98 — Bestial Incursion
pub(in crate::card::sets) static BESTIAL_INCURSION: CardRecord = CardRecord::new(
    "Bestial Incursion",
    "e0de5f66-f0df-4866-9f73-104ce50411b4",
    "Justin Gerard",
    CardRules::new_sorcery(mana_cost!("{3}{G}")).with_abilities(&[
        AbilityDef::spell(
            "Create a 4/4 green Beast creature token with trample.",
            EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(BEAST_TOKEN))),
        ),
        abilities::flashback(&[CostDef::Mana(mana_cost!("{5}{G}"))]),
    ]),
);

// FRA 99 — Budding Insurgent
pub(in crate::card::sets) static BUDDING_INSURGENT: CardRecord = CardRecord::new(
    "Budding Insurgent",
    "18c59d60-2640-4576-9375-3ba38aa3ecb7",
    "Jeff Miracola",
    CardRules::new_creature(mana_cost!("{2}{G}"), &["Dryad", "Scout"], 3, 3).with_abilities(&[
        abilities::vigilance(),
        AbilityDef::activated_with_targets(
            "Sacrifice this creature: Destroy target artifact or enchantment. If that permanent \
             was a legendary enchantment, draw a card. Activate only as a sorcery.",
            &[CostDef::SacrificeSource],
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::AnyOf(&[
                    ObjectPredicateDef::HasType(CardType::Artifact),
                    ObjectPredicateDef::HasType(CardType::Enchantment),
                ]),
            )],
            // What it was is read while it is still there to ask; the draw is owed whether
            // or not the destruction then happens.
            EffectDef::IfElseCondition {
                condition: &TriggerConditionDef::TargetMatches {
                    slot: TargetIndex::PRIMARY,
                    object: ObjectPredicateDef::All(&[
                        ObjectPredicateDef::Supertype(CardSupertype::Legendary),
                        ObjectPredicateDef::HasType(CardType::Enchantment),
                    ]),
                },
                then: &EffectDef::Sequence(&[
                    EffectDef::destroy_target(TargetIndex::PRIMARY),
                    abilities::draw_cards(ValueDef::Constant(1)),
                ]),
                otherwise: &EffectDef::destroy_target(TargetIndex::PRIMARY),
            },
        )
        .with_activation_timing(ActivationTimingDef::SorcerySpeed),
    ]),
);

// FRA 100 — Carnivorous Cultivator // Enroot
pub(in crate::card::sets) static CARNIVOROUS_CULTIVATOR: CardRecord = CardRecord::new_prepare(
    "Carnivorous Cultivator // Enroot",
    "79dd5c54-5ea5-47b5-8f9b-50ed57a5ea45",
    "Martina Fačková",
    &[
        (
            "Carnivorous Cultivator",
            CardRules::new_creature(mana_cost!("{1}{G}"), &["Elf", "Warlock"], 2, 3)
                .with_abilities(&[
                    abilities::deathtouch(),
                    enters_prepared(),
                    AbilityDef::triggered_with_targets(
                        "Whenever this creature deals combat damage to a player, return target \
                         land card from your graveyard to your hand.",
                        TriggerEventDef::combat_damage_to_player(ObjectPredicateDef::Source),
                        &[AbilityTargetDef::exactly_one(
                            AbilityTargetPredicate::Object {
                                object: ObjectPredicateDef::HasType(CardType::Land),
                                zones: &[ZoneKind::Graveyard],
                                controller: None,
                                owner: Some(PlayerRelation::You),
                            },
                        )],
                        EffectDef::move_to_zone(
                            EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            ZoneKind::Hand,
                            ZonePlacement::Top,
                        ),
                    ),
                ]),
        ),
        (
            "Enroot",
            CardRules::new_sorcery(mana_cost!("{G}")).with_ability(AbilityDef::spell(
                "Search your library for a land card, put it into your graveyard, then shuffle.",
                EffectDef::SearchZone {
                    exile_face_down: false,
                    player: EffectRecipientDef::Controller,
                    source: ZoneKind::Library,
                    object: ObjectPredicateDef::HasType(CardType::Land),
                    minimum: 0,
                    maximum: ValueDef::Constant(1),
                    reveal: false,
                    destination: ZoneKind::Graveyard,
                    placement: ZonePlacement::Top,
                    shuffle: true,
                    enters_tapped: false,
                    attachment: None,
                    binding: None,
                    then: None,
                },
            )),
        ),
    ],
);

// FRA 101 — Compel Brutality
pub(in crate::card::sets) static COMPEL_BRUTALITY: CardRecord = CardRecord::new(
    "Compel Brutality",
    "bd32d736-7a58-46b9-90b4-2cac3c3e80a1",
    "Danny Schwartz",
    CardRules::new_instant(mana_cost!("{1}{G}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "Target creature you control deals damage equal to its power to target creature \
                 or planeswalker an opponent controls.",
                &[
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Creature),
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    }),
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: A_CREATURE_OR_PLANESWALKER,
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::Opponent),
                        owner: None,
                    }),
                ],
                EffectDef::damage_from(
                    ObjectRefDef::Target(TargetIndex::PRIMARY),
                    EffectRecipientDef::Target(TargetIndex(1)),
                    ValueDef::TargetPower(TargetIndex::PRIMARY),
                ),
            ),
            AbilityDef::spell_with_targets(
                "Target planeswalker you control deals damage equal to its loyalty to target \
                 creature or planeswalker an opponent controls.",
                &[
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Planeswalker),
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    }),
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: A_CREATURE_OR_PLANESWALKER,
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::Opponent),
                        owner: None,
                    }),
                ],
                // A planeswalker's loyalty is the number of loyalty counters on it (CR 306.5c).
                EffectDef::damage_from(
                    ObjectRefDef::Target(TargetIndex::PRIMARY),
                    EffectRecipientDef::Target(TargetIndex(1)),
                    ValueDef::CountersOnObject(&ObjectCounterValueDef::new(
                        ObjectRefDef::Target(TargetIndex::PRIMARY),
                        CounterKind::Loyalty,
                    )),
                ),
            ),
        ],
    )),
);

// FRA 101b — Compel Brutality (alternate printing)
const COMPEL_BRUTALITY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &COMPEL_BRUTALITY,
    1,
    "e8f47203-40a9-4a2c-8440-c0354f894c61",
    "Danny Schwartz",
);

// FRA 102 — Flourishing Grapple
pub(in crate::card::sets) static FLOURISHING_GRAPPLE: CardRecord = CardRecord::new(
    "Flourishing Grapple",
    "f71958e9-6d6d-4393-8b49-567103b50877",
    "Omar Rayyan",
    CardRules::new_instant(mana_cost!("{G}")).with_ability(AbilityDef::spell_with_targets(
        "Target creature or planeswalker an opponent controls that's red or white loses all \
         abilities until end of turn. Target creature you control deals damage equal to its \
         power to that permanent.",
        &[
            AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                object: ObjectPredicateDef::All(&[
                    A_CREATURE_OR_PLANESWALKER,
                    ObjectPredicateDef::AnyOf(&[
                        ObjectPredicateDef::Color(ManaColor::Red),
                        ObjectPredicateDef::Color(ManaColor::White),
                    ]),
                ]),
                zones: &[ZoneKind::Battlefield],
                controller: Some(PlayerRelation::Opponent),
                owner: None,
            }),
            AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                object: ObjectPredicateDef::HasType(CardType::Creature),
                zones: &[ZoneKind::Battlefield],
                controller: Some(PlayerRelation::You),
                owner: None,
            }),
        ],
        EffectDef::Sequence(&[
            EffectDef::Apply {
                recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                effect: AppliedEffectDef::remove_abilities(AbilityPredicateDef::Any),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
            EffectDef::damage_from(
                ObjectRefDef::Target(TargetIndex(1)),
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ValueDef::TargetPower(TargetIndex(1)),
            ),
        ]),
    )),
);

// FRA 103 — Gardenize
pub(in crate::card::sets) static GARDENIZE: CardRecord = CardRecord::new(
    "Gardenize",
    "930b89c3-4433-48de-829f-20fc3dbfced9",
    "Néstor Ossandón Leal",
    CardRules::new_enchantment(mana_cost!("{1}{G}{G}")).with_abilities(&[
        abilities::dies_trigger_matching(
            "Whenever a creature you control dies, put a charge counter on this enchantment.",
            ObjectPredicateDef::All(&[
                ObjectPredicateDef::HasType(CardType::Creature),
                ObjectPredicateDef::ControlledBy(PlayerRelation::You),
            ]),
            EffectDef::AddCounters {
                object: EffectRecipientDef::Source,
                kind: CounterKind::named("charge"),
                amount: ValueDef::Constant(1),
            },
        ),
        AbilityDef::triggered(
            "At the beginning of your first main phase, add {G} for each charge counter on this \
             enchantment.",
            TriggerEventDef::StepBegins {
                step: TurnStepDef::PrecombatMain,
                player: PlayerRelation::You,
            },
            EffectDef::AddMana(
                AddManaEffectDef::one(ManaColor::Green)
                    .with_variable_amount(ValueDef::CountersOnSource(CounterKind::named("charge"))),
            ),
        ),
    ]),
);

// FRA 104 — Greenhouse Propagator
pub(in crate::card::sets) static GREENHOUSE_PROPAGATOR: CardRecord = CardRecord::new(
    "Greenhouse Propagator",
    "a56e0f91-b128-4693-a949-53cb403f4fbf",
    "Ekaterina Burmak",
    CardRules::new_creature(mana_cost!("{2}{G}"), &["Cat", "Druid"], 2, 3).with_abilities(&[
        AbilityDef::triggered(
            "Whenever another creature you control enters, you gain 1 life.",
            TriggerEventDef::zone_changed(
                ObjectPredicateDef::All(&[
                    ObjectPredicateDef::HasType(CardType::Creature),
                    ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ]),
                None,
                Some(ZoneKind::Battlefield),
            ),
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(1),
            },
        ),
        AbilityDef::activated_mana(
            "{T}: Add {G}.",
            &[CostDef::TapSource],
            EffectDef::AddMana(AddManaEffectDef::one(ManaColor::Green)),
        ),
    ]),
);

// FRA 105 — Heartwood Crafter // Soul Tether
// Audit: unsupported — Heartwood Crafter needs a mana restriction that forbids one use, casting a
// spell from your hand; restrictions name the uses mana is permitted for.
pub(in crate::card::sets) static HEARTWOOD_CRAFTER: CardRecord = CardRecord::new_prepare(
    "Heartwood Crafter // Soul Tether",
    "910a1f41-17fd-4ab0-9597-7151e79dc760",
    "Anastasia Ovchinnikova",
    &[
        ("Heartwood Crafter", CardRules::unsupported()),
        ("Soul Tether", CardRules::unsupported()),
    ],
);

// FRA 106 — Hexhaven Invigorator
pub(in crate::card::sets) static HEXHAVEN_INVIGORATOR: CardRecord = CardRecord::new(
    "Hexhaven Invigorator",
    "9a446cae-e93c-4574-8ffd-7688f9729a8a",
    "Olivier Bernard",
    CardRules::new_creature(mana_cost!("{G}{G}{G}{G}"), &["Chimera", "Horror"], 6, 6).with_abilities(&[
        abilities::vigilance(),
        AbilityDef::triggered(
            "Whenever this creature is dealt damage, you may search your library for up to that \
             many land cards, put them onto the battlefield tapped, then shuffle.",
            TriggerEventDef::damage_to_source(),
            EffectDef::May {
                player: EffectRecipientDef::Controller,
                effect: &EffectDef::SearchZone {
                    exile_face_down: false,
                    player: EffectRecipientDef::Controller,
                    source: ZoneKind::Library,
                    object: ObjectPredicateDef::HasType(CardType::Land),
                    minimum: 0,
                    maximum: ValueDef::TriggerEventAmount,
                    reveal: false,
                    destination: ZoneKind::Battlefield,
                    placement: ZonePlacement::Top,
                    shuffle: true,
                    enters_tapped: true,
                    attachment: None,
                    binding: None,
                    then: None,
                },
            },
        ),
    ]),
);

// FRA 107 — Hungering Puppetbeast
pub(in crate::card::sets) static HUNGERING_PUPPETBEAST: CardRecord = CardRecord::new(
    "Hungering Puppetbeast",
    "3db2da7a-8088-4117-916b-f9c905d1b45b",
    "Simon Dominic",
    CardRules::new_artifact_creature(mana_cost!("{3}{G}{G}"), &["Beast", "Construct"], 5, 5)
        .with_abilities(&[
            abilities::enters_trigger(
                "When this creature enters, create a Heartwood token. (It's a red and green \
                 artifact with \"{T}: Add {R} or {G}.\")",
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(HEARTWOOD_TOKEN))),
            ),
            AbilityDef::activated(
                "{1}, Sacrifice another artifact: Put a +1/+1 counter on this creature. It gains \
                 your choice of trample, hexproof, or haste until end of turn.",
                &[
                    CostDef::Mana(mana_cost!("{1}")),
                    CostDef::sacrifice_permanent(ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Artifact),
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                    ])),
                ],
                EffectDef::Sequence(&[
                    EffectDef::AddCounters {
                        object: EffectRecipientDef::Source,
                        kind: CounterKind::PlusOnePlusOne,
                        amount: ValueDef::Constant(1),
                    },
                    EffectDef::ChooseEffect {
                        player: EffectRecipientDef::Controller,
                        choices: &[
                            EffectChoiceDef {
                                label: "Trample",
                                effect: EffectDef::Apply {
                                    recipient: EffectRecipientDef::Source,
                                    effect: AppliedEffectDef::add_ability(&abilities::trample()),
                                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                                },
                            },
                            EffectChoiceDef {
                                label: "Hexproof",
                                effect: EffectDef::Apply {
                                    recipient: EffectRecipientDef::Source,
                                    effect: AppliedEffectDef::add_ability(&abilities::hexproof()),
                                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                                },
                            },
                            EffectChoiceDef {
                                label: "Haste",
                                effect: EffectDef::Apply {
                                    recipient: EffectRecipientDef::Source,
                                    effect: AppliedEffectDef::add_ability(&abilities::haste()),
                                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                                },
                            },
                        ],
                    },
                ]),
            ),
        ]),
);

// FRA 108 — Hunter's Axe
pub(in crate::card::sets) static HUNTER_S_AXE: CardRecord = CardRecord::new(
    "Hunter's Axe",
    "a2cc5d0e-9643-4ee2-81de-fa2c3bd1ab09",
    "Richard Kane Ferguson",
    CardRules::new_artifact(mana_cost!("{G}"))
        .with_subtypes(&["Equipment"])
        .with_abilities(&[
            AbilityDef::static_ability(
                "Equipped creature gets +2/+0 and has \"Whenever this creature attacks, it gains \
                 your choice of trample or deathtouch until end of turn.\"",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::AttachedPermanent,
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::modify_power_toughness(
                            ValueDef::Constant(2),
                            ValueDef::Constant(0),
                        ),
                        AppliedEffectDef::add_ability(&AbilityDef::triggered(
                            "Whenever this creature attacks, it gains your choice of trample or \
                             deathtouch until end of turn.",
                            TriggerEventDef::attacks(ObjectPredicateDef::Source),
                            EffectDef::ChooseEffect {
                                player: EffectRecipientDef::Controller,
                                choices: &[
                                    EffectChoiceDef {
                                        label: "Trample",
                                        effect: EffectDef::Apply {
                                            recipient: EffectRecipientDef::Source,
                                            effect: AppliedEffectDef::add_ability(
                                                &abilities::trample(),
                                            ),
                                            duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                                        },
                                    },
                                    EffectChoiceDef {
                                        label: "Deathtouch",
                                        effect: EffectDef::Apply {
                                            recipient: EffectRecipientDef::Source,
                                            effect: AppliedEffectDef::add_ability(
                                                &abilities::deathtouch(),
                                            ),
                                            duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                                        },
                                    },
                                ],
                            },
                        )),
                    ]),
                },
            ),
            abilities::equip(
                &[CostDef::Mana(mana_cost!("{2}"))],
                "Equip {2} ({2}: Attach to target creature you control. Equip only as a sorcery.)",
            ),
        ]),
);

// FRA 109 — Inspired Tethermage
// Audit: unsupported — Needs a trigger event for a player putting loyalty counters on a
// planeswalker.
pub(in crate::card::sets) static INSPIRED_TETHERMAGE: CardRecord = CardRecord::new(
    "Inspired Tethermage",
    "073f4998-a204-447b-93d5-746ae87fd6a1",
    "Cristi Balanescu",
    CardRules::unsupported(),
);

// FRA 110 — Omnipresence
pub(in crate::card::sets) static OMNIPRESENCE: CardRecord = CardRecord::new(
    "Omnipresence",
    "eaf9dc77-c83b-49cf-84be-6bd791cb925e",
    "Bryan Sola",
    CardRules::new_enchantment(mana_cost!("{5}{G}{G}{G}")).with_ability(
        AbilityDef::static_ability(
            "You may cast spells with mana value less than or equal to the number of creatures \
             you control from your hand without paying their mana costs.",
            EffectDef::ModifyCost(CostModificationDef::SpellAlternative {
                spell: ObjectPredicateDef::ManaValueAtMostValue(ValueDef::CountMatchingObjects(
                    &ObjectQueryDef::controlled_by(
                        ObjectPredicateDef::HasType(CardType::Creature),
                        &[ZoneKind::Battlefield],
                        PlayerSetDef::Related(PlayerRelation::You),
                    ),
                )),
                caster: PlayerRelation::You,
                zones: &[ZoneKind::Hand],
                costs: crate::NO_COSTS,
            }),
        ),
    ),
);

// FRA 111 — Puppet Crafting
pub(in crate::card::sets) static PUPPET_CRAFTING: CardRecord = CardRecord::new(
    "Puppet Crafting",
    "6b8789a6-3b63-4198-af5f-c2f2f49fafd9",
    "Anna Pavleeva",
    CardRules::new_enchantment(mana_cost!("{1}{G}"))
        .with_subtypes(&["Aura"])
        .with_abilities(&[
            abilities::aura_spell(
                "Enchant artifact or non-Aura enchantment",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::AnyOf(&[
                        ObjectPredicateDef::HasType(CardType::Artifact),
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Enchantment),
                            ObjectPredicateDef::Not(&ObjectPredicateDef::Subtype(
                                SubtypeDef::from_name("Aura"),
                            )),
                        ]),
                    ]),
                )],
            ),
            AbilityDef::static_ability(
                "Enchanted permanent is a Construct creature with base power and toughness 5/5 \
                 in addition to its other types.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::AttachedPermanent,
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::add_card_types(CardTypeSet::single(CardType::Creature)),
                        AppliedEffectDef::add_creature_types(CreatureTypeSetDef::named(&[
                            "Construct",
                        ])),
                        AppliedEffectDef::set_base_power_toughness(
                            ValueDef::Constant(5),
                            ValueDef::Constant(5),
                        ),
                    ]),
                },
            ),
            AbilityDef::activated(
                "{4}{G}: Return this card from your graveyard to your hand.",
                &[CostDef::Mana(mana_cost!("{4}{G}"))],
                EffectDef::move_to_zone(
                    EffectRecipientDef::Source,
                    ZoneKind::Hand,
                    ZonePlacement::Top,
                ),
            )
            .with_source_zones(&[ZoneKind::Graveyard]),
        ]),
);

// FRA 112 — Restore with Empathy
pub(in crate::card::sets) static RESTORE_WITH_EMPATHY: CardRecord = CardRecord::new(
    "Restore with Empathy",
    "3546b93b-a7d1-451d-a369-22cc8ddcd00d",
    "Eelis Kyttanen",
    CardRules::new_instant(mana_cost!("{2}{G}")).with_ability(AbilityDef::spell_with_targets(
        "Return target permanent card from your graveyard to your hand. You gain 4 life.",
        &[AbilityTargetDef::exactly_one(
            AbilityTargetPredicate::Object {
                object: A_PERMANENT_CARD,
                zones: &[ZoneKind::Graveyard],
                controller: None,
                owner: Some(PlayerRelation::You),
            },
        )],
        EffectDef::Sequence(&[
            EffectDef::move_to_zone(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ZoneKind::Hand,
                ZonePlacement::Top,
            ),
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(4),
            },
        ]),
    )),
);

// FRA 113 — Simulacrum Shaper
pub(in crate::card::sets) static SIMULACRUM_SHAPER: CardRecord = CardRecord::new(
    "Simulacrum Shaper",
    "7c725702-8696-4e5a-8318-62f5e2616d52",
    "Alix Branwyn",
    CardRules::new_creature(mana_cost!("{1}{G}{G}"), &["Elf", "Druid"], 2, 2).with_abilities(&[
        abilities::enters_trigger(
            "When this creature enters, you may search your library for a basic land card, put \
             that card onto the battlefield tapped, then shuffle.",
            EffectDef::May {
                player: EffectRecipientDef::Controller,
                effect: &EffectDef::SearchZone {
                    exile_face_down: false,
                    player: EffectRecipientDef::Controller,
                    source: ZoneKind::Library,
                    object: A_BASIC_LAND_CARD,
                    minimum: 0,
                    maximum: ValueDef::Constant(1),
                    reveal: false,
                    destination: ZoneKind::Battlefield,
                    placement: ZonePlacement::Top,
                    shuffle: true,
                    enters_tapped: true,
                    attachment: None,
                    binding: None,
                    then: None,
                },
            },
        ),
        abilities::dies_trigger(
            "When this creature dies, draw a card.",
            abilities::draw_cards(ValueDef::Constant(1)),
        ),
    ]),
);

// FRA 114 — Something Worth Saving
pub(in crate::card::sets) static SOMETHING_WORTH_SAVING: CardRecord = CardRecord::new(
    "Something Worth Saving",
    "02ee7817-40af-4fcf-a2df-eb218b669281",
    "PINDURSKI",
    CardRules::new_instant(mana_cost!("{1}{G}")).with_ability(AbilityDef::spell(
        "Mill four cards. You may put a permanent card from among them into your hand. You gain \
         1 life. (To mill four cards, put the top four cards of your library into your \
         graveyard.)",
        EffectDef::Sequence(&[
            EffectDef::BindOutput {
                binding: Binding!("milled"),
                effect: &EffectDef::Mill {
                    player: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(4),
                },
            },
            EffectDef::Choose(ChooseDef {
                binding: ObjectChoiceBindingDef::Objects(Binding!("kept")),
                unchosen: None,
                chooser: PlayerRefDef::EffectController,
                candidates: ObjectSetDef::Matching {
                    objects: &ObjectSetDef::Binding(Binding!("milled")),
                    object: ObjectSetFilterDef::Predicate(&A_PERMANENT_CARD),
                },
                exclude: None,
                minimum: 0,
                maximum: 1,
                visibility: ChoiceVisibilityDef::Public,
                then: &EffectDef::move_to_zone(
                    EffectRecipientDef::objects(ObjectSetDef::Binding(Binding!("kept"))),
                    ZoneKind::Hand,
                    ZonePlacement::Top,
                ),
            }),
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(1),
            },
        ]),
    )),
);

// FRA 115 — Sureshot Sower
pub(in crate::card::sets) static SURESHOT_SOWER: CardRecord = CardRecord::new(
    "Sureshot Sower",
    "b635389c-e286-4edb-80d1-23dbe4a18857",
    "Zara Alfonso",
    CardRules::new_creature(mana_cost!("{1}{G}"), &["Human", "Archer"], 3, 1).with_abilities(&[
        abilities::reach(),
        AbilityDef::activated_with_targets(
            "{3}{G}, Discard this card: Destroy target creature with flying.",
            &[CostDef::Mana(mana_cost!("{3}{G}")), CostDef::DiscardSource],
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::All(&[
                    ObjectPredicateDef::HasType(CardType::Creature),
                    ObjectPredicateDef::HasKeyword(KeywordAbility::Flying),
                ]),
            )],
            EffectDef::destroy_target(TargetIndex::PRIMARY),
        )
        .with_source_zones(&[ZoneKind::Hand]),
    ]),
);

// FRA 116 — Tarmogoyf (reprint)
const TARMOGOYF_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_fut::TARMOGOYF,
    "8be5ee47-8f8a-4e3c-b1a1-9ca0e1fdec7f",
    "Lucas Graciano",
);

// FRA 117 — Tethermage's Advantage
pub(in crate::card::sets) static TETHERMAGE_S_ADVANTAGE: CardRecord = CardRecord::new(
    "Tethermage's Advantage",
    "38589a7c-9cfb-4bcc-845e-9dc205095853",
    "Javier Charro",
    CardRules::new_instant(mana_cost!("{G}")).with_ability(AbilityDef::spell_with_targets(
        "Target creature gets +2/+2 and gains reach until end of turn. Untap it.",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::HasType(CardType::Creature),
        )],
        EffectDef::Sequence(&[
            EffectDef::Apply {
                recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                effect: AppliedEffectDef::Composite(&[
                    AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(2),
                        ValueDef::Constant(2),
                    ),
                    AppliedEffectDef::add_ability(&abilities::reach()),
                ]),
                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
            },
            EffectDef::Untap {
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
            },
        ]),
    )),
);

// FRA 118 — Verdant Kraken
pub(in crate::card::sets) static VERDANT_KRAKEN: CardRecord = CardRecord::new(
    "Verdant Kraken",
    "2bb7a8eb-227f-410b-859f-750ef0aea2f0",
    "Wayne Reynolds",
    CardRules::new_creature(mana_cost!("{4}{G}{G}{G}"), &["Plant", "Kraken"], 6, 6).with_ability(
        AbilityDef::triggered(
            "At the beginning of each player's upkeep, you create a 3/3 green Forest Tentacle \
             land creature token. (It has \"{T}: Add {G}.\" It's affected by summoning sickness \
             until your next turn.)",
            TriggerEventDef::StepBegins {
                step: TurnStepDef::Upkeep,
                player: PlayerRelation::Any,
            },
            // The reminder text's "{T}: Add {G}" is the Forest's own (CR 305.6), and being a
            // creature is what makes the token wait a turn to use it (CR 302.6).
            EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                TokenCharacteristics::creature(&["Forest", "Tentacle"], &[ManaColor::Green], 3, 3)
                    .with_type(CardType::Land)
                    .with_art(CardArt::new(
                        "fc924972-5014-45d1-9676-6a76d862b205",
                        "Deruchenko Alexander",
                    )),
            ))),
        ),
    ),
);

// FRA 119 — Vinelasher Adept
pub(in crate::card::sets) static VINELASHER_ADEPT: CardRecord = CardRecord::new(
    "Vinelasher Adept",
    "e5ed142b-2b61-4ef5-8b23-2db2a0a0319d",
    "Christina Kraus",
    CardRules::new_creature(mana_cost!("{4}{G}{G}"), &["Rhino", "Soldier"], 2, 4).with_abilities(
        &[
            abilities::reach(),
            abilities::enters_trigger_with_targets(
                "When this creature enters, put three +1/+1 counters on target creature.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(3),
                },
            ),
            abilities::typecycling!(
                "Basic landcycling {2} ({2}, Discard this card: Search your library for a basic \
                 land card, reveal it, put it into your hand, then shuffle.)",
                &[CostDef::Mana(mana_cost!("{2}"))],
                A_BASIC_LAND_CARD,
            ),
        ],
    ),
);

// FRA 120 — Wrecking Gecko
pub(in crate::card::sets) static WRECKING_GECKO: CardRecord = CardRecord::new(
    "Wrecking Gecko",
    "3d693cb0-681e-480a-8f70-07e94c39225c",
    "Raph Lomotan",
    CardRules::new_artifact_creature(mana_cost!("{4}{G}"), &["Lizard", "Construct"], 5, 5)
        .with_abilities(&[
            abilities::ward(
                &[CostDef::Mana(mana_cost!("{2}"))],
                "Ward {2} (Whenever this creature becomes the target of a spell or ability an \
                 opponent controls, counter it unless that player pays {2}.)",
            ),
            abilities::apply_to_self_until_end_of_turn(
                "{6}{G}{G}: This creature gets +4/+4 and gains trample until end of turn.",
                &[CostDef::Mana(mana_cost!("{6}{G}{G}"))],
                AppliedEffectDef::Composite(&[
                    AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(4),
                        ValueDef::Constant(4),
                    ),
                    AppliedEffectDef::add_ability(&abilities::trample()),
                ]),
            ),
        ]),
);

// FRA 121 — Aerid Konstrari
pub(in crate::card::sets) static AERID_KONSTRARI: CardRecord = CardRecord::new(
    "Aerid Konstrari",
    "f17d2792-b075-4c47-ad38-e7a7eaee5f8c",
    "Chris Rahn",
    CardRules::new_creature(mana_cost!("{1}{R}{G}{G}"), &["Elder", "Sphinx"], 5, 4)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flying(),
            AbilityDef::triggered(
                "When Aerid Konstrari enters or dies, create a Heartwood token. (It's a red and \
                 green artifact with \"{T}: Add {R} or {G}.\")",
                TriggerEventDef::AnyOf(&[
                    TriggerEventDef::zone_changed(
                        ObjectPredicateDef::Source,
                        None,
                        Some(ZoneKind::Battlefield),
                    ),
                    TriggerEventDef::zone_changed(
                        ObjectPredicateDef::Source,
                        Some(ZoneKind::Battlefield),
                        Some(ZoneKind::Graveyard),
                    ),
                ]),
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(HEARTWOOD_TOKEN))),
            ),
            AbilityDef::activated(
                "{6}: Create a Heartwood token. Then Aerid Konstrari gets +X/+0 until end of turn, \
                 where X is the number of artifacts you control.",
                &[CostDef::Mana(mana_cost!("{6}"))],
                EffectDef::Sequence(&[
                    EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(HEARTWOOD_TOKEN))),
                    EffectDef::Apply {
                        recipient: EffectRecipientDef::Source,
                        effect: AppliedEffectDef::modify_power_toughness(
                            ValueDef::CountMatchingObjects(&ObjectQueryDef::controlled_by(
                                ObjectPredicateDef::HasType(CardType::Artifact),
                                &[ZoneKind::Battlefield],
                                PlayerSetDef::Related(PlayerRelation::You),
                            )),
                            ValueDef::Constant(0),
                        ),
                        duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                    },
                ]),
            ),
        ]),
);

// FRA 122 — Avatar of Burgeoning Echoes
pub(in crate::card::sets) static AVATAR_OF_BURGEONING_ECHOES: CardRecord = CardRecord::new(
    "Avatar of Burgeoning Echoes",
    "5905995b-7a20-4602-a7cc-90aa5089a082",
    "Eli Minaya",
    CardRules::new_creature(mana_cost!("{G}{U}"), &["Avatar"], 2, 3).with_abilities(&[
        AbilityDef::triggered(
            "Landfall — Whenever a land you control enters, empower Jace 2. (Put two loyalty \
             counters on a Jace token you control. If you don't control one, first create a \
             blue Jace planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            TriggerEventDef::zone_changed(
                ObjectPredicateDef::All(&[
                    ObjectPredicateDef::HasType(CardType::Land),
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ]),
                None,
                Some(ZoneKind::Battlefield),
            ),
            empower_jace(ValueDef::Constant(2)),
        ),
        planeswalkers_you_control_have(
            "Planeswalkers you control have \"[−10]: Put a +1/+1 counter on target creature \
             for each land you control.\"",
            &AbilityDef::activated_with_targets(
                "−10: Put a +1/+1 counter on target creature for each land you control.",
                &[CostDef::Loyalty(ValueDef::Constant(-10))],
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::CountMatchingObjects(&ObjectQueryDef::controlled_by(
                        ObjectPredicateDef::HasType(CardType::Land),
                        &[ZoneKind::Battlefield],
                        PlayerSetDef::Related(PlayerRelation::You),
                    )),
                },
            ),
        ),
    ]),
);

// FRA 123 — Blessed Ghoul
pub(in crate::card::sets) static BLESSED_GHOUL: CardRecord = CardRecord::new(
    "Blessed Ghoul",
    "bb975803-9bf2-401e-9414-d272df314398",
    "Igor Grechanyi",
    CardRules::new_creature(mana_cost!("{W/B}"), &["Zombie", "Cleric"], 1, 1).with_abilities(&[
        abilities::lifelink(),
        AbilityDef::activated(
            "{2}{W/B}: Return this card from your graveyard to your hand.",
            &[CostDef::Mana(mana_cost!("{2}{W/B}"))],
            EffectDef::move_to_zone(
                EffectRecipientDef::Source,
                ZoneKind::Hand,
                ZonePlacement::Top,
            ),
        )
        .with_source_zones(&[ZoneKind::Graveyard]),
    ]),
);

// FRA 124 — Bloombrute
pub(in crate::card::sets) static BLOOMBRUTE: CardRecord = CardRecord::new(
    "Bloombrute",
    "6b804503-9c70-4b1f-bb13-a65fb6dd3ef8",
    "Antonio José Manzanedo",
    CardRules::new_creature(mana_cost!("{2}{G}{W}"), &["Plant", "Elemental"], 4, 4).with_abilities(
        &[
            AbilityDef::triggered(
                "Whenever you gain life, draw a card. This ability triggers only once each turn.",
                TriggerEventDef::LifeGained(PlayerRelation::You),
                abilities::draw_cards(ValueDef::Constant(1)),
            )
            .triggering_at_most(1),
            AbilityDef::activated_with_targets(
                "{4}{G}{W}: Target creature gains trample and lifelink until end of turn.",
                &[CostDef::Mana(mana_cost!("{4}{G}{W}"))],
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::add_ability(&abilities::trample()),
                        AppliedEffectDef::add_ability(&abilities::lifelink()),
                    ]),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ],
    ),
);

// FRA 125 — Charge the Sanctum
pub(in crate::card::sets) static CHARGE_THE_SANCTUM: CardRecord = CardRecord::new(
    "Charge the Sanctum",
    "87b40df5-5c0a-41f5-a09c-a04f17066a91",
    "Néstor Ossandón Leal",
    CardRules::new_instant(mana_cost!("{2}{R/W}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell(
                "Creatures you control get +2/+0 until end of turn.",
                EffectDef::Apply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::HasType(CardType::Creature),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(2),
                        ValueDef::Constant(0),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
            AbilityDef::spell_with_targets(
                "Target creature gets +2/+0 and gains first strike until end of turn. Put a +1/+1 \
                 counter on it.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Sequence(&[
                    EffectDef::Apply {
                        recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        effect: AppliedEffectDef::Composite(&[
                            AppliedEffectDef::modify_power_toughness(
                                ValueDef::Constant(2),
                                ValueDef::Constant(0),
                            ),
                            AppliedEffectDef::add_ability(&abilities::first_strike()),
                        ]),
                        duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                    },
                    EffectDef::AddCounters {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        kind: CounterKind::PlusOnePlusOne,
                        amount: ValueDef::Constant(1),
                    },
                ]),
            ),
        ],
    )),
);

// FRA 126 — Clash of Elements
pub(in crate::card::sets) static CLASH_OF_ELEMENTS: CardRecord = CardRecord::new(
    "Clash of Elements",
    "b61bcef7-5832-45e6-a2bc-26d4f23707fc",
    "Anna Pavleeva",
    CardRules::new_instant(mana_cost!("{1}{U}{R}")).with_ability(AbilityDef::spell_with_targets(
        "Choose target nonland permanent. Its owner may put it on top of their library. If they \
         do, Clash of Elements deals 2 damage to them. If they didn't put the card on top of their \
         library, they put it on the bottom.",
        &[AbilityTargetDef::exactly_one_permanent(
            ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Land)),
        )],
        EffectDef::ChooseEffect {
            player: EffectRecipientDef::player(PlayerRefDef::OwnerOf(ObjectRefDef::Target(
                TargetIndex::PRIMARY,
            ))),
            choices: &[
                EffectChoiceDef {
                    label: "Top",
                    // "If they do" asks where the permanent went: one that a replacement sent
                    // somewhere else was not put on top, and costs its owner nothing. A token
                    // was put there, though nothing of it is left to find (CR 704.5d).
                    effect: EffectDef::WithZoneMoveResult {
                        effect: &EffectDef::move_to_zone(
                            EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            ZoneKind::Library,
                            ZonePlacement::Top,
                        ),
                        binding: Binding!("put_on_top"),
                        then: &EffectDef::IfNoObjects(IfNoObjectsDef {
                            input: ObjectSetDef::Union(&[
                                ObjectSetDef::InZone {
                                    objects: &ObjectSetDef::ZoneChangeSuccessorsOfBinding(
                                        Binding!("put_on_top"),
                                    ),
                                    zone: ZoneKind::Library,
                                },
                                ObjectSetDef::Matching {
                                    objects: &ObjectSetDef::Binding(Binding!("put_on_top")),
                                    object: ObjectSetFilterDef::Predicate(
                                        &ObjectPredicateDef::Token,
                                    ),
                                },
                            ]),
                            if_empty: &EffectDef::None,
                            otherwise: &EffectDef::damage(
                                EffectRecipientDef::player(PlayerRefDef::OwnerOf(
                                    ObjectRefDef::Target(TargetIndex::PRIMARY),
                                )),
                                ValueDef::Constant(2),
                            ),
                        }),
                    },
                },
                EffectChoiceDef {
                    label: "Bottom",
                    effect: EffectDef::move_to_zone(
                        EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        ZoneKind::Library,
                        ZonePlacement::Bottom,
                    ),
                },
            ],
        },
    )),
);

// FRA 127 — Craftwork Crusher
// Audit: unsupported — Needs a triggered ability that chooses two modes; a trigger carries at most
// one mode.
pub(in crate::card::sets) static CRAFTWORK_CRUSHER: CardRecord = CardRecord::new(
    "Craftwork Crusher",
    "03f9839c-aa07-4ee7-847b-091e47ab80c4",
    "Ilse Gort",
    CardRules::unsupported(),
);

// FRA 128 — Denzilore Fatehold
// Audit: unsupported — Needs a trigger event for scrying or surveilling, published wherever a scry
// or surveil of at least one card is performed.
pub(in crate::card::sets) static DENZILORE_FATEHOLD: CardRecord = CardRecord::new(
    "Denzilore Fatehold",
    "986f9e98-9d8d-428b-9187-860745cf3269",
    "Dmitry Burmak",
    CardRules::unsupported(),
);

// FRA 129 — Desperate Futurescribe
// Audit: unsupported — Needs a per-turn history flag recording that a player has scried or
// surveilled this turn.
pub(in crate::card::sets) static DESPERATE_FUTURESCRIBE: CardRecord = CardRecord::new(
    "Desperate Futurescribe",
    "cfaa3ebd-5c21-4e99-a0dd-8426d19b53a5",
    "Sidharth Chaturvedi",
    CardRules::unsupported(),
);

// FRA 130 — Emergency Phytomedic // Seed Suture
pub(in crate::card::sets) static EMERGENCY_PHYTOMEDIC: CardRecord = CardRecord::new_prepare(
    "Emergency Phytomedic // Seed Suture",
    "de94d388-919d-44ff-baef-8c90a417ac6d",
    "Dmitry Burmak",
    &[
        (
            "Emergency Phytomedic",
            CardRules::new_creature(mana_cost!("{G/W}"), &["Dryad", "Cleric"], 1, 1)
                .with_abilities(&[enters_prepared_with_reminder()]),
        ),
        ("Seed Suture", SEED_SUTURE),
    ],
);

// FRA 131 — Entrust the Spark
pub(in crate::card::sets) static ENTRUST_THE_SPARK: CardRecord = CardRecord::new(
    "Entrust the Spark",
    "ca894d25-b9fc-4cd6-8746-70d8c2868721",
    "Justyna Dura",
    CardRules::new_sorcery(mana_cost!("{3}{G}{U}")).with_ability(AbilityDef::spell(
        "You may sacrifice a planeswalker. If you do, search your library for a planeswalker card, \
         put it onto the battlefield, then shuffle.",
        EffectDef::PayOr(PayOrDef::optional(
            &[CostDef::sacrifice_permanent(ObjectPredicateDef::HasType(
                CardType::Planeswalker,
            ))],
            &EffectDef::SearchZone {
                exile_face_down: false,
                player: EffectRecipientDef::Controller,
                source: ZoneKind::Library,
                object: ObjectPredicateDef::HasType(CardType::Planeswalker),
                minimum: 0,
                maximum: ValueDef::Constant(1),
                reveal: false,
                destination: ZoneKind::Battlefield,
                placement: ZonePlacement::Top,
                shuffle: true,
                enters_tapped: false,
                attachment: None,
                binding: None,
                then: None,
            },
        )),
    )),
);

// FRA 132 — Fatehold Charm
pub(in crate::card::sets) static FATEHOLD_CHARM: CardRecord = CardRecord::new(
    "Fatehold Charm",
    "cfc54011-647e-4428-bcdb-59400e1da49d",
    "Rovina Cai",
    CardRules::new_instant(mana_cost!("{W}{U}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell(
                "Draw a card. Empower Jace 2.",
                EffectDef::Sequence(&[
                    EffectDef::DrawCards {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(1),
                    },
                    empower_jace(ValueDef::Constant(2)),
                ]),
            ),
            AbilityDef::spell_with_targets(
                "Return target spell or creature to its owner's hand.",
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::AnyOf(&[
                        AbilityTargetPredicate::Object {
                            object: ObjectPredicateDef::Spell,
                            zones: &[ZoneKind::Stack],
                            controller: None,
                            owner: None,
                        },
                        AbilityTargetPredicate::Object {
                            object: ObjectPredicateDef::HasType(CardType::Creature),
                            zones: &[ZoneKind::Battlefield],
                            controller: None,
                            owner: None,
                        },
                    ]),
                )],
                EffectDef::move_to_zone(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ZoneKind::Hand,
                    ZonePlacement::Top,
                ),
            ),
            AbilityDef::spell(
                "Creatures you control get +1/+2 until end of turn.",
                EffectDef::Apply {
                    recipient: EffectRecipientDef::objects(ObjectSetDef::Query(
                        ObjectQueryDef::matching(
                            ObjectPredicateDef::HasType(CardType::Creature),
                            &[ZoneKind::Battlefield],
                            PlayerRelation::You,
                        ),
                    )),
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(1),
                        ValueDef::Constant(2),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ],
    )),
);

// FRA 133 — Fatehold Chronologist // Peer Review
pub(in crate::card::sets) static FATEHOLD_CHRONOLOGIST: CardRecord = CardRecord::new_prepare(
    "Fatehold Chronologist // Peer Review",
    "29e7ec16-0c16-48aa-8e09-ce6e0d5bd40b",
    "Alexander Mokhov",
    &[
        (
            "Fatehold Chronologist",
            CardRules::new_creature(mana_cost!("{1}{W/U}"), &["Bird", "Wizard"], 1, 2)
                .with_abilities(&[abilities::flying(), enters_prepared_with_reminder()]),
        ),
        ("Peer Review", PEER_REVIEW),
    ],
);

// FRA 134 — Ferocity of the Hunt
pub(in crate::card::sets) static FEROCITY_OF_THE_HUNT: CardRecord = CardRecord::new(
    "Ferocity of the Hunt",
    "a9793ce9-5a0b-41fe-b9ad-02f6f7da2481",
    "Elliot Lang",
    CardRules::new_enchantment(mana_cost!("{1}{B/G}"))
        .with_subtypes(&["Aura"])
        .with_abilities(&[
            abilities::flash(),
            abilities::enchant_creature(),
            AbilityDef::static_ability(
                "Enchanted creature gets +1/+0 and has deathtouch.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::AttachedPermanent,
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::modify_power_toughness(
                            ValueDef::Constant(1),
                            ValueDef::Constant(0),
                        ),
                        AppliedEffectDef::add_ability(&abilities::deathtouch()),
                    ]),
                },
            ),
            abilities::dies_trigger_matching(
                "When enchanted creature dies, return that card to the battlefield tapped under \
                 its owner's control.",
                ObjectPredicateDef::AttachedToSource,
                EffectDef::WithBattlefieldArrival {
                    effect: &EffectDef::move_to_zone(
                        EffectRecipientDef::TriggeringZoneChangeResult,
                        ZoneKind::Battlefield,
                        ZonePlacement::Top,
                    ),
                    arrival: BattlefieldArrivalDef {
                        modifications: &[BattlefieldEntryModificationDef::Tapped],
                        ..BattlefieldArrivalDef::DEFAULT
                    },
                },
            ),
        ]),
);

// FRA 135 — Frostbite Pyromental
pub(in crate::card::sets) static FROSTBITE_PYROMENTAL: CardRecord = CardRecord::new(
    "Frostbite Pyromental",
    "7a44581f-8fc4-457d-888a-1e211090ee7e",
    "Wayne Reynolds",
    CardRules::new_creature(mana_cost!("{U}{R}{R}"), &["Elemental"], 4, 4).with_abilities(&[
        abilities::trample(),
        abilities::haste(),
        AbilityDef::triggered(
            "Whenever this creature deals combat damage to a player, draw two cards.",
            TriggerEventDef::combat_damage_to_player(ObjectPredicateDef::Source),
            abilities::draw_cards(ValueDef::Constant(2)),
        ),
        AbilityDef::triggered(
            "At the beginning of the end step, sacrifice this creature.",
            TriggerEventDef::StepBegins {
                step: TurnStepDef::End,
                player: PlayerRelation::Any,
            },
            EffectDef::sacrifice(EffectRecipientDef::Source),
        ),
    ]),
);

// FRA 136 — Grim Repriser
// Audit: unsupported — Needs a per-turn history flag recording that an opponent has been dealt
// noncombat damage this turn.
pub(in crate::card::sets) static GRIM_REPRISER: CardRecord = CardRecord::new(
    "Grim Repriser",
    "8295c48c-b4dd-4bc1-a206-04cf12b79bbd",
    "Wayne Reynolds",
    CardRules::unsupported(),
);

// FRA 137 — Ingris Stingerquill
pub(in crate::card::sets) static INGRIS_STINGERQUILL: CardRecord = CardRecord::new(
    "Ingris Stingerquill",
    "6471b135-33a8-4005-9a07-ebb74e0bf145",
    "Chase Stone",
    CardRules::new_creature(mana_cost!("{B}{R}{R}"), &["Elder", "Sphinx"], 1, 4)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flying(),
            AbilityDef::triggered(
                "Whenever a creature you control attacks, that creature deals 1 damage to each \
                 opponent.",
                TriggerEventDef::attacks(ObjectPredicateDef::All(&[
                    ObjectPredicateDef::HasType(CardType::Creature),
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ])),
                EffectDef::damage_from(
                    ObjectRefDef::TriggeringObject,
                    EffectRecipientDef::Opponent,
                    ValueDef::Constant(1),
                ),
            ),
            AbilityDef::activated(
                "{4}: Create a 2/2 colorless Wizard Soldier creature token named Cadet. Then \
                 creatures you control gain haste until end of turn.",
                &[CostDef::Mana(mana_cost!("{4}"))],
                EffectDef::Sequence(&[
                    EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(CADET_TOKEN))),
                    EffectDef::Apply {
                        recipient: EffectRecipientDef::matching_objects(
                            ObjectPredicateDef::HasType(CardType::Creature),
                            &[ZoneKind::Battlefield],
                            PlayerRelation::You,
                        ),
                        effect: AppliedEffectDef::add_ability(&abilities::haste()),
                        duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                    },
                ]),
            ),
        ]),
);

// FRA 138 — Konstrari Charm
pub(in crate::card::sets) static KONSTRARI_CHARM: CardRecord = CardRecord::new(
    "Konstrari Charm",
    "7d29dfa1-9582-47bc-8f42-62b611bdcc4e",
    "Matheus Graef",
    CardRules::new_instant(mana_cost!("{R}{G}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "Konstrari Charm deals 6 damage to target creature with flying.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::HasKeyword(KeywordAbility::Flying),
                    ]),
                )],
                EffectDef::damage(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ValueDef::Constant(6),
                ),
            ),
            AbilityDef::spell_with_targets(
                "Put two +1/+1 counters on target creature. It gains trample until end of turn.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Sequence(&[
                    EffectDef::AddCounters {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        kind: CounterKind::PlusOnePlusOne,
                        amount: ValueDef::Constant(2),
                    },
                    EffectDef::Apply {
                        recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        effect: AppliedEffectDef::add_ability(&abilities::trample()),
                        duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                    },
                ]),
            ),
            AbilityDef::spell(
                "Add {C}{C}{C}.",
                EffectDef::AddMana(AddManaEffectDef::one(ManaColor::Colorless).with_amount(3)),
            ),
        ],
    )),
);

// FRA 139 — Konstrari Improviser // Soul Tether
pub(in crate::card::sets) static KONSTRARI_IMPROVISER: CardRecord = CardRecord::new_prepare(
    "Konstrari Improviser // Soul Tether",
    "42e28bd2-486b-45d4-8840-6e33c19c2d57",
    "Matheus Graef",
    &[
        (
            "Konstrari Improviser",
            CardRules::new_creature(mana_cost!("{1}{R/G}"), &["Human", "Artificer"], 2, 2)
                .with_abilities(&[enters_prepared_with_reminder()]),
        ),
        ("Soul Tether", SOUL_TETHER),
    ],
);

// FRA 140 — Kwia Vigorbloom
pub(in crate::card::sets) static KWIA_VIGORBLOOM: CardRecord = CardRecord::new(
    "Kwia Vigorbloom",
    "2d6ff182-a853-4898-895b-072c89324ca7",
    "Ryan Pancoast",
    CardRules::new_creature(mana_cost!("{3}{G}{W}{W}"), &["Elder", "Sphinx"], 6, 6)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flying(),
            abilities::vigilance(),
            abilities::lifelink(),
            abilities::ward(&[CostDef::Mana(mana_cost!("{2}"))], "Ward {2}"),
            AbilityDef::triggered(
                "Whenever you gain life, create a colorless artifact token named Lotus with \"{T}, \
                 Sacrifice this token: Add three mana of any one color.\" This ability triggers \
                 only once each turn.",
                TriggerEventDef::LifeGained(PlayerRelation::You),
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                    TokenCharacteristics::artifact(&[], &[])
                        .with_name("Lotus")
                        .with_abilities(&[AbilityDef::activated_mana(
                            "{T}, Sacrifice this token: Add three mana of any one color.",
                            &[CostDef::TapSource, CostDef::SacrificeSource],
                            EffectDef::AddMana(AddManaEffectDef::any_color().with_amount(3)),
                        )])
                        .with_art(CardArt::new(
                            "c59b7223-00a9-4c0d-896d-1cd463860c40",
                            "Alex V. Ngo",
                        )),
                ))),
            )
            .triggering_at_most(1),
        ]),
);

// FRA 141 — Mind Meanderer
pub(in crate::card::sets) static MIND_MEANDERER: CardRecord = CardRecord::new(
    "Mind Meanderer",
    "94c290ce-252c-42b3-bcb0-c1ef621df566",
    "Jason Mowry",
    CardRules::new_creature(
        mana_cost!("{3}{G}{U}{U}"),
        &["Bird", "Fish", "Illusion"],
        4,
        4,
    )
    .with_abilities(&[
        abilities::flying(),
        AbilityDef::static_ability(
            "This creature has vigilance as long as you control a Jace planeswalker.",
            EffectDef::ConditionalStatic(ConditionalStaticEffectDef {
                condition: ObjectSetCountConditionDef {
                    objects: &ObjectSetDef::Query(ObjectQueryDef::matching(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Planeswalker),
                            ObjectPredicateDef::Subtype(SubtypeDef::from_name("Jace")),
                        ]),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    )),
                    predicate: ObjectSetPredicateDef {
                        filter: None,
                        comparison: ComparisonDef::GreaterOrEqual,
                        amount: 1,
                    },
                },
                then: StaticApplyDef {
                    recipient: EffectRecipientDef::Source,
                    effect: AppliedEffectDef::add_ability(&abilities::vigilance()),
                },
            }),
        ),
        abilities::enters_trigger_with_targets(
            "When this creature enters, it fights up to one target creature an opponent controls. \
             (Each deals damage equal to its power to the other.)",
            &[AbilityTargetDef::up_to(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::HasType(CardType::Creature),
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::Opponent),
                    owner: None,
                },
                1,
            )],
            EffectDef::Fight {
                first: ObjectRefDef::Source,
                second: ObjectRefDef::Target(TargetIndex::PRIMARY),
                excess: None,
            },
        ),
    ]),
);

// FRA 142 — Null Summoner
// Audit: unsupported — Needs a static, threshold-gated permission to cast a linked exiled card
// with mana of any type.
pub(in crate::card::sets) static NULL_SUMMONER: CardRecord = CardRecord::new(
    "Null Summoner",
    "3afdc75a-1bf5-4f2f-84eb-d82f77a095cd",
    "Magali Villeneuve",
    CardRules::unsupported(),
);

// FRA 143 — Paradox Shaper // Omit Variables
pub(in crate::card::sets) static PARADOX_SHAPER: CardRecord = CardRecord::new_prepare(
    "Paradox Shaper // Omit Variables",
    "e61b9d48-0ace-4453-afe0-a1024444bac0",
    "Mark Zug",
    &[
        (
            "Paradox Shaper",
            CardRules::new_creature(mana_cost!("{1}{U/B}"), &["Octopus", "Wizard"], 1, 3)
                .with_abilities(&[
                    reprepare_at_upkeep(),
                    AbilityDef::activated_with_targets(
                        "{2}: Put target card from your graveyard on the bottom of your library.",
                        &[CostDef::Mana(mana_cost!("{2}"))],
                        &[AbilityTargetDef::exactly_one(
                            AbilityTargetPredicate::Object {
                                object: ObjectPredicateDef::Any,
                                zones: &[ZoneKind::Graveyard],
                                controller: None,
                                owner: Some(PlayerRelation::You),
                            },
                        )],
                        EffectDef::move_to_zone(
                            EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            ZoneKind::Library,
                            ZonePlacement::Bottom,
                        ),
                    ),
                ]),
        ),
        ("Omit Variables", OMIT_VARIABLES),
    ],
);

// FRA 144 — Primal Witchstalker
pub(in crate::card::sets) static PRIMAL_WITCHSTALKER: CardRecord = CardRecord::new(
    "Primal Witchstalker",
    "04e64af7-cca1-499e-8951-f386e84c8b5b",
    "Kaitlyn McCulley",
    CardRules::new_creature(mana_cost!("{1}{B}{G}"), &["Wolf"], 2, 1).with_abilities(&[
        abilities::menace(),
        abilities::enters_trigger(
            "When this creature enters, mill four cards. When you do, return target land card from \
             your graveyard to the battlefield tapped. (To mill four cards, put the top four cards \
             of your library into your graveyard.)",
            // "When you do" asks for all four. A library of fewer than four cards has as
            // many milled as it holds, which is not milling four (CR 701.17b): The Royal
            // Scions is ruled the same way for drawing fewer than four, and surveil is the
            // exception only because CR 701.25d makes it one. The library is counted rather
            // than the cards in the graveyard afterwards, because a milled card that goes
            // somewhere else instead was still milled (CR 701.17c).
            EffectDef::IfElseCondition {
                condition: &TriggerConditionDef::ValueComparison(&ValueComparisonDef {
                    left: ValueDef::LibrarySize(PlayerRelation::You),
                    comparison: ComparisonDef::GreaterOrEqual,
                    right: ValueDef::Constant(4),
                }),
                then: &EffectDef::Sequence(&[
                    EffectDef::Mill {
                        player: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(4),
                    },
                    EffectDef::ReflexiveTrigger(&AbilityDef::triggered_with_targets(
                        "When you do, return target land card from your graveyard to the \
                         battlefield tapped.",
                        TriggerEventDef::Reflexive,
                        &[AbilityTargetDef::exactly_one(
                            AbilityTargetPredicate::Object {
                                object: ObjectPredicateDef::HasType(CardType::Land),
                                zones: &[ZoneKind::Graveyard],
                                controller: None,
                                owner: Some(PlayerRelation::You),
                            },
                        )],
                        EffectDef::WithBattlefieldArrival {
                            effect: &EffectDef::move_to_zone(
                                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                                ZoneKind::Battlefield,
                                ZonePlacement::Top,
                            ),
                            arrival: BattlefieldArrivalDef {
                                modifications: &[BattlefieldEntryModificationDef::Tapped],
                                ..BattlefieldArrivalDef::DEFAULT
                            },
                        },
                    )),
                ]),
                otherwise: &EffectDef::Mill {
                    player: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(4),
                },
            },
        ),
    ]),
);

// FRA 145 — Proctor of Potential
// Audit: unsupported — Needs a per-turn history flag recording that a player has scried or
// surveilled this turn.
pub(in crate::card::sets) static PROCTOR_OF_POTENTIAL: CardRecord = CardRecord::new(
    "Proctor of Potential",
    "cf0eec8c-0475-4050-8144-481a9bb13a0f",
    "Andrey Kuzinskiy",
    CardRules::unsupported(),
);

// FRA 146 — Prudent Fateseer // Peer Review
// Audit: unsupported — Prudent Fateseer needs a trigger event for scrying or surveilling,
// published wherever a scry or surveil of at least one card is performed.
pub(in crate::card::sets) static PRUDENT_FATESEER: CardRecord = CardRecord::new_prepare(
    "Prudent Fateseer // Peer Review",
    "6c1c790b-9e0e-4964-9ea3-554843907f06",
    "Gustavo Pelissari",
    &[
        ("Prudent Fateseer", CardRules::unsupported()),
        ("Peer Review", CardRules::unsupported()),
    ],
);

// FRA 147 — Recursive Recruitment
pub(in crate::card::sets) static RECURSIVE_RECRUITMENT: CardRecord = CardRecord::new(
    "Recursive Recruitment",
    "68fddb6a-86d4-4ebb-907d-fdcaadebc4b3",
    "Dominik Mayer",
    CardRules::new_sorcery(mana_cost!("{2}{U}{B}")).with_abilities(&[
        AbilityDef::spell(
            "Create two 2/2 colorless Wizard Soldier creature tokens named Cadet. If this spell \
             was cast from a graveyard, put a +1/+1 counter on each of them for every three cards \
             in your graveyard.",
            EffectDef::CreateToken(
                CreateTokenDef::new(TokenDef::Literal(CADET_TOKEN))
                    .with_amount(2)
                    .with_created_tokens(CreatedTokensDef {
                        binding: Binding!("cadets"),
                        then: &EffectDef::IfCondition {
                            condition: &TriggerConditionDef::SourceCastFrom(ZoneKind::Graveyard),
                            then: &EffectDef::AddCounters {
                                object: EffectRecipientDef::objects(ObjectSetDef::Binding(
                                    Binding!("cadets"),
                                )),
                                kind: CounterKind::PlusOnePlusOne,
                                amount: ValueDef::Quotient(&QuotientValueDef::new(
                                    ValueDef::CountMatchingObjects(&ObjectQueryDef::owned_by(
                                        ObjectPredicateDef::Any,
                                        &[ZoneKind::Graveyard],
                                        PlayerSetDef::Related(PlayerRelation::You),
                                    )),
                                    ValueDef::Constant(3),
                                    RoundingDef::Down,
                                )),
                            },
                        },
                    }),
            ),
        ),
        abilities::flashback(&[CostDef::Mana(mana_cost!("{6}{U}{B}"))]),
    ]),
);

// FRA 148 — Solarium Sentry
pub(in crate::card::sets) static SOLARIUM_SENTRY: CardRecord = CardRecord::new(
    "Solarium Sentry",
    "1ef12dcf-df50-4da6-8c4c-e2937ba9698e",
    "Chris Rallis",
    CardRules::new_creature(mana_cost!("{G}{W}"), &["Cat", "Soldier"], 3, 3).with_ability(
        AbilityDef::triggered(
            "Whenever an opponent casts a spell with mana value 2 or less, you gain 2 life.",
            TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                ObjectPredicateDef::ManaValueAtMost(2),
                ObjectPredicateDef::ControlledBy(PlayerRelation::Opponent),
            ])),
            EffectDef::GainLife {
                recipient: EffectRecipientDef::Controller,
                amount: ValueDef::Constant(2),
            },
        ),
    ),
);

// FRA 149 — Solitary Cell
pub(in crate::card::sets) static SOLITARY_CELL: CardRecord = CardRecord::new(
    "Solitary Cell",
    "5142bbb6-194c-4b12-b11a-1a21c9fe81a6",
    "Andrea Piparo",
    CardRules::new_artifact(mana_cost!("{R}{W}")).with_abilities(&[
        abilities::enters_trigger_with_targets(
            "When this artifact enters, exile target nonland permanent an opponent controls with \
             mana value 3 or less until this artifact leaves the battlefield.",
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::All(&[
                        ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Land)),
                        ObjectPredicateDef::ManaValueAtMost(3),
                    ]),
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::Opponent),
                    owner: None,
                },
            )],
            EffectDef::ExileLinkedToSource {
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                face_down: false,
                until_source_leaves: true,
                then: None,
            },
        ),
        AbilityDef::activated(
            "{1}, {T}, Discard a legendary card: Draw a card.",
            &[
                CostDef::Mana(mana_cost!("{1}")),
                CostDef::TapSource,
                CostDef::discard(ObjectPredicateDef::Supertype(CardSupertype::Legendary)),
            ],
            abilities::draw_cards(ValueDef::Constant(1)),
        ),
    ]),
);

// FRA 150 — Stingerquill Charm
pub(in crate::card::sets) static STINGERQUILL_CHARM: CardRecord = CardRecord::new(
    "Stingerquill Charm",
    "81733ff7-e611-43ee-bf38-6bb700676017",
    "Danny Schwartz",
    CardRules::new_instant(mana_cost!("{B}{R}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "Stingerquill Charm deals 3 damage to any target.",
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::AnyTarget,
                )],
                EffectDef::damage(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ValueDef::Constant(3),
                ),
            ),
            AbilityDef::spell_with_targets(
                "Target creature gains first strike and deathtouch until end of turn.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::add_ability(&abilities::first_strike()),
                        AppliedEffectDef::add_ability(&abilities::deathtouch()),
                    ]),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
            AbilityDef::spell(
                "Create a 2/2 colorless Wizard Soldier creature token named Cadet. It gains haste \
                 until end of turn.",
                EffectDef::CreateToken(
                    CreateTokenDef::new(TokenDef::Literal(CADET_TOKEN)).with_created_tokens(
                        CreatedTokensDef {
                            binding: Binding!("cadet"),
                            then: &EffectDef::Apply {
                                recipient: EffectRecipientDef::objects(ObjectSetDef::Binding(
                                    Binding!("cadet"),
                                )),
                                effect: AppliedEffectDef::add_ability(&abilities::haste()),
                                duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                            },
                        },
                    ),
                ),
            ),
        ],
    )),
);

// FRA 151 — Stingerquill Voxmancer // Vicious Verse
pub(in crate::card::sets) static STINGERQUILL_VOXMANCER: CardRecord = CardRecord::new_prepare(
    "Stingerquill Voxmancer // Vicious Verse",
    "84b1c268-3b8a-41b6-92e3-a2ce0cc3d738",
    "Igor Grechanyi",
    &[
        (
            "Stingerquill Voxmancer",
            CardRules::new_creature(mana_cost!("{B/R}"), &["Goblin", "Sorcerer"], 1, 2)
                .with_abilities(&[reprepare_at_upkeep_with_reminder()]),
        ),
        ("Vicious Verse", VICIOUS_VERSE),
    ],
);

// FRA 152 — Stinging Vitriol
pub(in crate::card::sets) static STINGING_VITRIOL: CardRecord = CardRecord::new(
    "Stinging Vitriol",
    "a7d78297-7411-4ec5-8931-a25146869d5b",
    "Nathaniel Himawan",
    CardRules::new_sorcery(mana_cost!("{B}{R}")).with_ability(AbilityDef::spell_with_targets(
        "Stinging Vitriol deals 2 damage to target opponent. That player reveals their hand. You \
         choose a nonland card from it. They discard that card.",
        &[AbilityTargetDef::exactly_one(
            AbilityTargetPredicate::Player(PlayerRelation::Opponent),
        )],
        EffectDef::Sequence(&[
            EffectDef::damage(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ValueDef::Constant(2),
            ),
            EffectDef::Sequence(&abilities::reveal_hand_and_discard_chosen_card(
                PlayerRefDef::Target(TargetIndex::PRIMARY),
                ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Land)),
            )),
        ]),
    )),
);

// FRA 153 — Tam's Resistance
pub(in crate::card::sets) static TAM_S_RESISTANCE: CardRecord = CardRecord::new(
    "Tam's Resistance",
    "b3d33df2-a77b-4c2e-ba7f-2cc9c1505f9b",
    "Samuel Perin",
    CardRules::new_sorcery(mana_cost!("{1}{G/U}")).with_abilities(&[
        AbilityDef::spell_with_targets(
            "Put a +1/+1 counter on up to one target creature. It gains vigilance until end \
             of turn.",
            &[AbilityTargetDef::up_to(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::HasType(CardType::Creature),
                    zones: &[ZoneKind::Battlefield],
                    controller: None,
                    owner: None,
                },
                1,
            )],
            EffectDef::Sequence(&[
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::add_ability(&abilities::vigilance()),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ]),
        ),
        AbilityDef::spell(
            "Empower Jace 4. (Put four loyalty counters on a Jace token you control. If you \
             don't control one, first create a blue Jace planeswalker token with \
             \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(4)),
        ),
    ]),
);

// FRA 154 — Tenured Tethermage
pub(in crate::card::sets) static TENURED_TETHERMAGE: CardRecord = CardRecord::new(
    "Tenured Tethermage",
    "1703306d-6a3d-4ab8-bf58-a9992236ef0f",
    "Javier Charro",
    CardRules::new_creature(mana_cost!("{1}{R}{G}"), &["Human", "Artificer"], 1, 1).with_abilities(
        &[
            abilities::enters_trigger(
                "When this creature enters, you may sacrifice a land. If you do, create two tapped \
                 Heartwood tokens. (They're red and green artifacts with \"{T}: Add {R} or {G}.\")",
                EffectDef::PayOr(PayOrDef::optional(
                    &[CostDef::sacrifice_permanent(ObjectPredicateDef::HasType(
                        CardType::Land,
                    ))],
                    &EffectDef::CreateToken(
                        CreateTokenDef::new(TokenDef::Literal(HEARTWOOD_TOKEN))
                            .with_amount(2)
                            .entering_tapped(),
                    ),
                )),
            ),
            AbilityDef::activated(
                "Tap two untapped artifacts you control: Put two +1/+1 counters on this creature.",
                &[CostDef::TapPermanents {
                    object: ObjectPredicateDef::HasType(CardType::Artifact),
                    controller: PlayerRelation::You,
                    count: 2,
                }],
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Source,
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(2),
                },
            ),
        ],
    ),
);

// FRA 155 — Theorix Charm
pub(in crate::card::sets) static THEORIX_CHARM: CardRecord = CardRecord::new(
    "Theorix Charm",
    "2835c9aa-0904-44db-8da2-e8c4e04201aa",
    "Elliot Lang",
    CardRules::new_instant(mana_cost!("{U}{B}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "Counter target noncreature spell unless its controller pays {2}.",
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::All(&[
                            ObjectPredicateDef::Spell,
                            ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(
                                CardType::Creature,
                            )),
                        ]),
                        zones: &[ZoneKind::Stack],
                        controller: None,
                        owner: None,
                    },
                )],
                abilities::counter_target_unless_paid(&[CostDef::Mana(mana_cost!("{2}"))]),
            ),
            AbilityDef::spell_with_targets(
                "Target creature gets -2/-2 until end of turn.",
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(-2),
                        ValueDef::Constant(-2),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
            AbilityDef::spell(
                "Mill three cards, then draw a card. (To mill three cards, put the top three cards \
                 of your library into your graveyard.)",
                EffectDef::Sequence(&[
                    EffectDef::Mill {
                        player: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(3),
                    },
                    abilities::draw_cards(ValueDef::Constant(1)),
                ]),
            ),
        ],
    )),
);

// FRA 156 — Theorix Metamage // Omit Variables
pub(in crate::card::sets) static THEORIX_METAMAGE: CardRecord = CardRecord::new_prepare(
    "Theorix Metamage // Omit Variables",
    "fb6bad96-841d-4738-8e62-92f346f914fd",
    "Slawomir Maniak",
    &[
        (
            "Theorix Metamage",
            CardRules::new_creature(mana_cost!("{2}{U/B}"), &["Shade", "Wizard"], 2, 3)
                .with_abilities(&crate::ability_list![
                    [enters_prepared_with_reminder()],
                    threshold(&[AbilityDef::static_ability(
                        "Threshold — This creature gets +1/+0 and has flying as long as there \
                         are seven or more cards in your graveyard.",
                        EffectDef::StaticApply {
                            recipient: EffectRecipientDef::Source,
                            effect: AppliedEffectDef::Composite(&[
                                AppliedEffectDef::modify_power_toughness(
                                    ValueDef::Constant(1),
                                    ValueDef::Constant(0),
                                ),
                                AppliedEffectDef::add_ability(&abilities::flying()),
                            ]),
                        },
                    )]),
                ]),
        ),
        ("Omit Variables", OMIT_VARIABLES),
    ],
);

// FRA 157 — Twinned Vision
pub(in crate::card::sets) static TWINNED_VISION: CardRecord = CardRecord::new(
    "Twinned Vision",
    "55f85984-0137-4899-8993-bbc8c4794d33",
    "Andreas Zafiratos",
    CardRules::new_instant(mana_cost!("{1}{U/R}")).with_abilities(&[
        AbilityDef::spell(
            "Draw a card. If this spell wasn't cast from your hand, draw two cards instead.",
            // A copy was not cast from anywhere, so it draws two as well.
            EffectDef::IfElseCondition {
                condition: &TriggerConditionDef::SourceCastFrom(ZoneKind::Hand),
                then: &abilities::draw_cards(ValueDef::Constant(1)),
                otherwise: &abilities::draw_cards(ValueDef::Constant(2)),
            },
        ),
        AbilityDef::alternative_cast(
            &[
                CostDef::Mana(mana_cost!("{1}{U/R}{U/R}")),
                CostDef::DiscardCards(1),
            ],
            AlternativeCastKindDef::Flashback,
            Some(
                "Flashback—{1}{U/R}{U/R}, Discard a card. (You may cast this card from your \
                 graveyard for its flashback cost. Then exile it.)",
            ),
            EffectDef::None,
        ),
    ]),
);

// FRA 158 — Twisted Fates
pub(in crate::card::sets) static TWISTED_FATES: CardRecord = CardRecord::new(
    "Twisted Fates",
    "c7c0765d-38fd-4d7b-bfb4-49b10ff5939b",
    "Lorenzo Mastroianni",
    CardRules::new_sorcery(mana_cost!("{2}{W}{W}{B}")).with_ability(
        AbilityDef::spell_with_targets(
            "Destroy target nonland permanent. Put a +1/+1 counter on each creature target player \
             controls.",
            &[
                AbilityTargetDef::exactly_one_permanent(ObjectPredicateDef::Not(
                    &ObjectPredicateDef::HasType(CardType::Land),
                )),
                AbilityTargetDef::exactly_one(AbilityTargetPredicate::Player(PlayerRelation::Any)),
            ],
            EffectDef::Sequence(&[
                EffectDef::destroy_target(TargetIndex::PRIMARY),
                EffectDef::AddCounters {
                    object: EffectRecipientDef::objects(ObjectSetDef::Query(
                        ObjectQueryDef::controlled_by(
                            ObjectPredicateDef::HasType(CardType::Creature),
                            &[ZoneKind::Battlefield],
                            PlayerSetDef::LegalTargets(TargetIndex(1)),
                        ),
                    )),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
            ]),
        ),
    ),
);

// FRA 159 — Uldaros Theorix
// Audit: unsupported — Needs copying exiled cards and casting the copies without paying their mana
// costs under a total-mana-value budget, with one target per card type.
pub(in crate::card::sets) static ULDAROS_THEORIX: CardRecord = CardRecord::new(
    "Uldaros Theorix",
    "a7ad622a-42ff-48fa-ae95-12e0a5bd9387",
    "Alexander Mokhov",
    CardRules::unsupported(),
);

// FRA 160 — Vigorbloom Charm
pub(in crate::card::sets) static VIGORBLOOM_CHARM: CardRecord = CardRecord::new(
    "Vigorbloom Charm",
    "2b198e10-b507-4314-a29c-a219f06e48b7",
    "Kaitlyn McCulley",
    CardRules::new_instant(mana_cost!("{G}{W}")).with_ability(AbilityDef::modal_spell(
        "Choose one —",
        &[
            AbilityDef::spell_with_targets(
                "Target permanent you control gains hexproof and indestructible until end of turn.",
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::Any,
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    },
                )],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::add_ability(&abilities::hexproof()),
                        AppliedEffectDef::add_ability(&abilities::indestructible()),
                    ]),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
            AbilityDef::spell(
                "You draw a card and gain 3 life.",
                EffectDef::Sequence(&[
                    abilities::draw_cards(ValueDef::Constant(1)),
                    EffectDef::GainLife {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(3),
                    },
                ]),
            ),
            AbilityDef::spell_with_targets(
                "Put a +1/+1 counter on target creature you control. Then it fights target \
                 creature an opponent controls. (Each deals damage equal to its power to the \
                 other.)",
                &[
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Creature),
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    }),
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Creature),
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::Opponent),
                        owner: None,
                    }),
                ],
                EffectDef::Sequence(&[
                    EffectDef::AddCounters {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        kind: CounterKind::PlusOnePlusOne,
                        amount: ValueDef::Constant(1),
                    },
                    EffectDef::Fight {
                        first: ObjectRefDef::Target(TargetIndex::PRIMARY),
                        second: ObjectRefDef::Target(TargetIndex(1)),
                        excess: None,
                    },
                ]),
            ),
        ],
    )),
);

// FRA 161 — Vigorbloom Vanguard // Seed Suture
pub(in crate::card::sets) static VIGORBLOOM_VANGUARD: CardRecord = CardRecord::new_prepare(
    "Vigorbloom Vanguard // Seed Suture",
    "acefc515-bf97-4dc0-b0f7-ae8ae5a61671",
    "Denman Rooke",
    &[
        (
            "Vigorbloom Vanguard",
            CardRules::new_creature(mana_cost!("{1}{G/W}"), &["Troll", "Druid"], 2, 2)
                .with_abilities(&[
                    enters_prepared_with_reminder(),
                    AbilityDef::static_ability(
                        "Each creature you control with a +1/+1 counter on it has vigilance.",
                        EffectDef::StaticApply {
                            recipient: EffectRecipientDef::matching_objects(
                                ObjectPredicateDef::All(&[
                                    ObjectPredicateDef::HasType(CardType::Creature),
                                    ObjectPredicateDef::HasCounter(CounterKind::PlusOnePlusOne),
                                ]),
                                &[ZoneKind::Battlefield],
                                PlayerRelation::You,
                            ),
                            effect: AppliedEffectDef::add_ability(&abilities::vigilance()),
                        },
                    ),
                ]),
        ),
        ("Seed Suture", SEED_SUTURE),
    ],
);

// FRA 162 — Vindictive Triumph
pub(in crate::card::sets) static VINDICTIVE_TRIUMPH: CardRecord = CardRecord::new(
    "Vindictive Triumph",
    "a803dbe7-153a-4e92-ad4d-c2babebe003d",
    "Johan Grenier",
    CardRules::new_instant(mana_cost!("{W}{B}{B}")).with_ability(AbilityDef::spell_with_targets(
        "Exile target creature or planeswalker. If that permanent's mana value was 3 or less, \
         return it to the battlefield tapped under your control. Exile it at the beginning of the \
         next end step.",
        &[AbilityTargetDef::exactly_one_permanent(
            A_CREATURE_OR_PLANESWALKER,
        )],
        // Its mana value is read while it is still a permanent with one to ask about.
        EffectDef::IfElseCondition {
            condition: &TriggerConditionDef::TargetMatches {
                slot: TargetIndex::PRIMARY,
                object: ObjectPredicateDef::ManaValueAtMost(3),
            },
            then: &EffectDef::WithZoneMoveResult {
                effect: &EffectDef::move_to_zone(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ZoneKind::Exile,
                    ZonePlacement::Top,
                ),
                binding: Binding!("exiled"),
                then: &EffectDef::WithZoneMoveResult {
                    effect: &EffectDef::WithBattlefieldArrival {
                        effect: &EffectDef::move_to_zone(
                            EffectRecipientDef::objects(ObjectSetDef::InZone {
                                objects: &ObjectSetDef::ZoneChangeSuccessorsOfBinding(Binding!(
                                    "exiled"
                                )),
                                zone: ZoneKind::Exile,
                            }),
                            ZoneKind::Battlefield,
                            ZonePlacement::Top,
                        ),
                        arrival: BattlefieldArrivalDef {
                            controller: Some(PlayerRelation::You),
                            modifications: &[BattlefieldEntryModificationDef::Tapped],
                            ..BattlefieldArrivalDef::DEFAULT
                        },
                    },
                    binding: Binding!("returned"),
                    then: &EffectDef::InstallTrigger(InstalledTriggerDef::once(
                        &AbilityDef::triggered(
                            "At the beginning of the next end step, exile it.",
                            TriggerEventDef::StepBegins {
                                step: TurnStepDef::End,
                                player: PlayerRelation::Any,
                            },
                            EffectDef::move_to_zone(
                                EffectRecipientDef::binding_zone_change_successors(Binding!(
                                    "returned"
                                )),
                                ZoneKind::Exile,
                                ZonePlacement::Top,
                            ),
                        ),
                    )),
                },
            },
            otherwise: &EffectDef::move_to_zone(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ZoneKind::Exile,
                ZonePlacement::Top,
            ),
        },
    )),
);

// FRA 163 — Warrior's Blades
// Audit: unsupported — Needs ability-cost pricing that reads the counters on the announced target.
pub(in crate::card::sets) static WARRIOR_S_BLADES: CardRecord = CardRecord::new(
    "Warrior's Blades",
    "c63d5b0e-ee72-42ed-aa7e-484ba84507cd",
    "Richard Kane Ferguson",
    CardRules::unsupported(),
);

// FRA 164 — Whiplash Wordsmith // Vicious Verse
// Audit: unsupported — Whiplash Wordsmith needs a per-turn history flag recording that an opponent
// has been dealt noncombat damage this turn.
pub(in crate::card::sets) static WHIPLASH_WORDSMITH: CardRecord = CardRecord::new_prepare(
    "Whiplash Wordsmith // Vicious Verse",
    "8096bc9a-a610-448f-bef2-7230e17e9777",
    "Anastasia Ovchinnikova",
    &[
        ("Whiplash Wordsmith", CardRules::unsupported()),
        ("Vicious Verse", CardRules::unsupported()),
    ],
);

// FRA 165 — Woodwork Prodigy // Soul Tether
pub(in crate::card::sets) static WOODWORK_PRODIGY: CardRecord = CardRecord::new_prepare(
    "Woodwork Prodigy // Soul Tether",
    "7d17f7e3-7b63-4674-9024-4fd1827f40ec",
    "Lius Lasahido",
    &[
        (
            "Woodwork Prodigy",
            CardRules::new_creature(mana_cost!("{2}{R/G}"), &["Cat", "Druid"], 3, 3)
                .with_abilities(&[reprepare_at_upkeep_with_reminder()]),
        ),
        ("Soul Tether", SOUL_TETHER),
    ],
);

// FRA 166 — Afterthought Sentry
pub(in crate::card::sets) static AFTERTHOUGHT_SENTRY: CardRecord = CardRecord::new(
    "Afterthought Sentry",
    "4d4b3bf7-a149-4099-b97d-4e36a87dfa60",
    "Andrew Mar",
    CardRules::new_artifact_creature(mana_cost!("{2}"), &["Gargoyle"], 2, 2).with_abilities(&[
        abilities::apply_to_self_until_end_of_turn(
            "{2}: This creature gains flying until end of turn.",
            &[CostDef::Mana(mana_cost!("{2}"))],
            AppliedEffectDef::add_ability(&abilities::flying()),
        ),
        AbilityDef::triggered_with_targets(
            "Whenever this creature attacks, exile up to one target card from a graveyard.",
            TriggerEventDef::attacks(ObjectPredicateDef::Source),
            &[AbilityTargetDef::up_to(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::Any,
                    zones: &[ZoneKind::Graveyard],
                    controller: None,
                    owner: None,
                },
                1,
            )],
            EffectDef::move_to_zone(
                EffectRecipientDef::Target(TargetIndex::PRIMARY),
                ZoneKind::Exile,
                ZonePlacement::Top,
            ),
        ),
    ]),
);

// FRA 167 — Archive Arbiter
pub(in crate::card::sets) static ARCHIVE_ARBITER: CardRecord = CardRecord::new(
    "Archive Arbiter",
    "024bce1e-a5f3-4292-bc17-d0355a5d65e1",
    "Valera Lutfullina",
    CardRules::new_artifact_creature(mana_cost!("{6}"), &["Sphinx"], 4, 4).with_abilities(&[
        abilities::flying(),
        AbilityDef::modal_triggered(
            "When this creature enters, choose one —",
            TriggerEventDef::zone_changed(
                ObjectPredicateDef::Source,
                None,
                Some(ZoneKind::Battlefield),
            ),
            &[
                AbilityDef::spell_with_targets(
                    "Destroy target noncreature, nonland permanent.",
                    &[AbilityTargetDef::exactly_one_permanent(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(
                                CardType::Creature,
                            )),
                            ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Land)),
                        ]),
                    )],
                    EffectDef::destroy_target(TargetIndex::PRIMARY),
                ),
                AbilityDef::spell(
                    "You gain 4 life.",
                    EffectDef::GainLife {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(4),
                    },
                ),
            ],
        ),
    ]),
);

// FRA 168 — Codie, Ravenous Codex
// Audit: unsupported — Needs a trigger event for casting a prepared spell.
pub(in crate::card::sets) static CODIE_RAVENOUS_CODEX: CardRecord = CardRecord::new(
    "Codie, Ravenous Codex",
    "c3192390-1518-49fc-8716-f2c7a0384f39",
    "Steve Prescott",
    CardRules::unsupported(),
);

// FRA 169 — The Echoverse Fulcrum
pub(in crate::card::sets) static THE_ECHOVERSE_FULCRUM: CardRecord = CardRecord::new(
    "The Echoverse Fulcrum",
    "d71d250f-c0e0-44b2-877c-76f3bcab4f34",
    "Adam Paquette",
    CardRules::new_artifact(mana_cost!("{2}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When The Echoverse Fulcrum enters, draw a card, then discard a card.",
                EffectDef::Sequence(&[
                    abilities::draw_cards(ValueDef::Constant(1)),
                    EffectDef::Discard {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(1),
                        selection: DiscardSelectionDef::RecipientChooses,
                        then: None,
                    },
                ]),
            ),
            AbilityDef::activated(
                "{5}, {T}, Exile The Echoverse Fulcrum: Destroy all creatures. Activate only as a \
                 sorcery.",
                &[
                    CostDef::Mana(mana_cost!("{5}")),
                    CostDef::TapSource,
                    CostDef::ExileSource,
                ],
                EffectDef::Destroy {
                    object: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::HasType(CardType::Creature),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::Any,
                    ),
                    then: None,
                },
            )
            .with_activation_timing(ActivationTimingDef::SorcerySpeed),
        ]),
);

// FRA 170 — Eye of Jace
pub(in crate::card::sets) static EYE_OF_JACE: CardRecord = CardRecord::new(
    "Eye of Jace",
    "0edba64a-39cf-4a8d-ba20-4f7da10b6c3d",
    "Ioannis Fiore",
    CardRules::new_artifact(mana_cost!("{1}")).with_ability(AbilityDef::triggered(
        "At the beginning of your upkeep, surveil 1. Then if there are seven or more cards in your \
         graveyard, sacrifice this artifact, it deals 2 damage to each opponent, and you gain 2 \
         life. (To surveil 1, look at the top card of your library. You may put it into your \
         graveyard.)",
        TriggerEventDef::StepBegins {
            step: TurnStepDef::Upkeep,
            player: PlayerRelation::You,
        },
        EffectDef::Sequence(&[
            abilities::surveil(ValueDef::Constant(1)),
            // Three instructions, none waiting on another: the damage and the life are owed
            // whether or not the Eye was still there to sacrifice.
            EffectDef::IfCondition {
                condition: &TriggerConditionDef::ObjectCount {
                    query: ObjectQueryDef::owned_by(
                        ObjectPredicateDef::Any,
                        &[ZoneKind::Graveyard],
                        PlayerSetDef::Related(PlayerRelation::You),
                    ),
                    comparison: ComparisonDef::GreaterOrEqual,
                    amount: 7,
                },
                then: &EffectDef::Sequence(&[
                    EffectDef::sacrifice(EffectRecipientDef::Source),
                    EffectDef::damage(EffectRecipientDef::Opponent, ValueDef::Constant(2)),
                    EffectDef::GainLife {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(2),
                    },
                ]),
            },
        ]),
    )),
);

// FRA 171 — Keeper of the Quiet Hour
pub(in crate::card::sets) static KEEPER_OF_THE_QUIET_HOUR: CardRecord = CardRecord::new(
    "Keeper of the Quiet Hour",
    "b6331218-dbb8-44a9-8ba5-5fb1ab41c5c0",
    "John Tedrick",
    CardRules::new_artifact_creature(mana_cost!("{3}"), &["Chimera"], 3, 2).with_ability(
        abilities::enters_trigger(
            "When this creature enters, empower Jace 2. (Put two loyalty counters on a Jace \
             token you control. If you don't control one, first create a blue Jace \
             planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
            empower_jace(ValueDef::Constant(2)),
        ),
    ),
);

// FRA 172 — Living Library
pub(in crate::card::sets) static LIVING_LIBRARY: CardRecord = CardRecord::new(
    "Living Library",
    "5d4a8e5f-0024-4da3-a2f5-edb48b12e733",
    "Jarel Threat",
    CardRules::new_artifact_creature(mana_cost!("{2}"), &["Book", "Illusion"], 0, 4).with_ability(
        AbilityDef::activated_with_targets(
            "{6}, Sacrifice this creature: Choose target creature or planeswalker an opponent \
             controls. Its owner shuffles it into their library.",
            &[CostDef::Mana(mana_cost!("{6}")), CostDef::SacrificeSource],
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: A_CREATURE_OR_PLANESWALKER,
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::Opponent),
                    owner: None,
                },
            )],
            EffectDef::Sequence(&[
                EffectDef::move_to_zone(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ZoneKind::Library,
                    ZonePlacement::Top,
                ),
                EffectDef::ShuffleLibrary {
                    player: EffectRecipientDef::player(PlayerRefDef::OwnerOf(
                        ObjectRefDef::Target(TargetIndex::PRIMARY),
                    )),
                },
            ]),
        ),
    ),
);

// FRA 173 — Medic's Kitesail
pub(in crate::card::sets) static MEDIC_S_KITESAIL: CardRecord = CardRecord::new(
    "Medic's Kitesail",
    "8b07409a-1dce-461d-95e4-1130521ff4c4",
    "Paolo Parente",
    CardRules::new_artifact(mana_cost!("{2}"))
        .with_subtypes(&["Equipment"])
        .with_abilities(&[
            AbilityDef::static_ability(
                "Equipped creature gets +1/+0 and has flying and \"Whenever this creature attacks, \
                 you gain 1 life.\"",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::AttachedPermanent,
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::modify_power_toughness(
                            ValueDef::Constant(1),
                            ValueDef::Constant(0),
                        ),
                        AppliedEffectDef::add_ability(&abilities::flying()),
                        AppliedEffectDef::add_ability(&AbilityDef::triggered(
                            "Whenever this creature attacks, you gain 1 life.",
                            TriggerEventDef::attacks(ObjectPredicateDef::Source),
                            EffectDef::GainLife {
                                recipient: EffectRecipientDef::Controller,
                                amount: ValueDef::Constant(1),
                            },
                        )),
                    ]),
                },
            ),
            abilities::equip(
                &[CostDef::Mana(mana_cost!("{2}"))],
                "Equip {2} ({2}: Attach to target creature you control. Equip only as a sorcery.)",
            ),
        ]),
);

// FRA 174 — Murmuring Volume
pub(in crate::card::sets) static MURMURING_VOLUME: CardRecord = CardRecord::new(
    "Murmuring Volume",
    "d68eab2e-89dd-4377-b7af-01512b1804a0",
    "Zoltan Boros",
    CardRules::new_artifact(mana_cost!("{3}"))
        .with_subtypes(&["Book"])
        .with_abilities(&[
            abilities::tap_for_mana(
                "{T}: Add one mana of any color.",
                AddManaEffectDef::any_color(),
            ),
            AbilityDef::activated(
                "{2}, {T}, Discard a card: Draw a card.",
                &[
                    CostDef::Mana(mana_cost!("{2}")),
                    CostDef::TapSource,
                    CostDef::discard(ObjectPredicateDef::Any),
                ],
                abilities::draw_cards(ValueDef::Constant(1)),
            ),
        ]),
);

// FRA 175 — Dedicated Commons
pub(in crate::card::sets) static DEDICATED_COMMONS: CardRecord = CardRecord::new(
    "Dedicated Commons",
    "3223e5db-5cc4-42f9-ae9e-ff58abc7c390",
    "Josu Solano",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {R} or {W}.",
            AddManaEffectDef::choice(&[ManaColor::Red, ManaColor::White]),
        ),
    ]),
);

// FRA 176 — Deserted Beach (reprint)
const DESERTED_BEACH_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_mid::DESERTED_BEACH,
    "56dae4c4-3e71-4a32-979b-4e26d9c9e96c",
    "Sarah Finnigan",
);

// FRA 177 — Fatehold Annex
pub(in crate::card::sets) static FATEHOLD_ANNEX: CardRecord = CardRecord::new(
    "Fatehold Annex",
    "5140f962-62f3-40fd-a322-44896c7e2613",
    "Sergey Glushakov",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {W} or {U}.",
            AddManaEffectDef::choice(&[ManaColor::White, ManaColor::Blue]),
        ),
    ]),
);

// FRA 178 — Formidable Commons
pub(in crate::card::sets) static FORMIDABLE_COMMONS: CardRecord = CardRecord::new(
    "Formidable Commons",
    "e6ca6c3e-f145-42d6-8a17-90770c15afaf",
    "Marco Gorlei",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {B} or {G}.",
            AddManaEffectDef::choice(&[ManaColor::Black, ManaColor::Green]),
        ),
    ]),
);

// FRA 179 — Hall of Echoes
pub(in crate::card::sets) static HALL_OF_ECHOES: CardRecord = CardRecord::new(
    "Hall of Echoes",
    "4a771010-b397-4849-ac9b-08e4dd5d6a72",
    "Leon Tukker",
    CardRules::new_land(&[]).with_abilities(&[
        abilities::tap_for(ManaColor::Colorless),
        AbilityDef::activated_with_targets(
            "{5}: This land becomes a copy of target creature you control until end of turn. The \
             \"legend rule\" doesn't apply to permanents you control this turn.",
            &[CostDef::Mana(mana_cost!("{5}"))],
            &[AbilityTargetDef::exactly_one(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::HasType(CardType::Creature),
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::You),
                    owner: None,
                },
            )],
            // Both halves last until end of turn, so the Hall is a land again by the time the
            // legend rule could compare it with what it copied.
            EffectDef::Sequence(&[
                EffectDef::BecomeCopyOf {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    copier: None,
                    exceptions: &CopyExceptionsDef::NONE,
                    duration: Some(ResolvedEffectDurationDef::UntilEndOfTurn),
                },
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Controller,
                    effect: AppliedEffectDef::Rule(AppliedRuleDef::PlayerRule(
                        PlayerRuleDef::LegendRuleDoesNotApplyTo(&ObjectPredicateDef::Any),
                    )),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ]),
        ),
    ]),
);

// FRA 180 — Haunted Ridge (reprint)
const HAUNTED_RIDGE_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_mid::HAUNTED_RIDGE,
    "a4e4966b-8963-4fac-a8bf-e778e063c7dd",
    "Wayne Wu",
);

// FRA 181 — Hexhaven Dueling Arena
pub(in crate::card::sets) static HEXHAVEN_DUELING_ARENA: CardRecord = CardRecord::new(
    "Hexhaven Dueling Arena",
    "9128ce00-6744-4d36-bfbe-ef75d78110b0",
    "Constantin Marin",
    CardRules::new_land(&[]).with_abilities(&[
        abilities::tap_for(ManaColor::Colorless),
        AbilityDef::activated_with_targets(
            "{2}, {T}: Target creature that attacked this turn becomes prepared. Activate only as \
             a sorcery. (Only creatures with prepare spells can become prepared.)",
            &[CostDef::Mana(mana_cost!("{2}")), CostDef::TapSource],
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::All(&[
                    ObjectPredicateDef::HasType(CardType::Creature),
                    ObjectPredicateDef::AttackedThisTurn,
                ]),
            )],
            become_prepared(EffectRecipientDef::Target(TargetIndex::PRIMARY)),
        )
        .with_activation_timing(ActivationTimingDef::SorcerySpeed),
        AbilityDef::activated_with_targets(
            "{4}, {T}: Target creature becomes prepared.",
            &[CostDef::Mana(mana_cost!("{4}")), CostDef::TapSource],
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::HasType(CardType::Creature),
            )],
            become_prepared(EffectRecipientDef::Target(TargetIndex::PRIMARY)),
        ),
    ]),
);

// FRA 182 — Innovative Commons
pub(in crate::card::sets) static INNOVATIVE_COMMONS: CardRecord = CardRecord::new(
    "Innovative Commons",
    "84ea799a-faa2-4ff1-a933-432d4ee31a3b",
    "Grady Frederick",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {U} or {R}.",
            AddManaEffectDef::choice(&[ManaColor::Blue, ManaColor::Red]),
        ),
    ]),
);

// FRA 183 — Konstrari Annex
pub(in crate::card::sets) static KONSTRARI_ANNEX: CardRecord = CardRecord::new(
    "Konstrari Annex",
    "39c805e3-82cd-42a9-80fe-8d81712a94ea",
    "Sergey Glushakov",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {R} or {G}.",
            AddManaEffectDef::choice(&[ManaColor::Red, ManaColor::Green]),
        ),
    ]),
);

// FRA 184 — Meticulous Commons
pub(in crate::card::sets) static METICULOUS_COMMONS: CardRecord = CardRecord::new(
    "Meticulous Commons",
    "93ac525e-1919-43dd-aba4-073b7e4c1768",
    "Grady Frederick",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {W} or {B}.",
            AddManaEffectDef::choice(&[ManaColor::White, ManaColor::Black]),
        ),
    ]),
);

// FRA 185 — Overgrown Farmland (reprint)
const OVERGROWN_FARMLAND_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_mid::OVERGROWN_FARMLAND,
    "178e61e4-472f-42cd-9d3b-4880c2acc527",
    "Titus Lunter",
);

// FRA 186 — Rockfall Vale (reprint)
const ROCKFALL_VALE_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_mid::ROCKFALL_VALE,
    "e3c8a8b6-23ba-45ad-80d1-8e2dc79897f7",
    "Cristi Balanescu",
);

// FRA 187 — Roiling Canopy
// Audit: unsupported — Needs an object count that excludes the triggering object.
pub(in crate::card::sets) static ROILING_CANOPY: CardRecord = CardRecord::new(
    "Roiling Canopy",
    "db61361b-bd12-453e-abc2-bbe09b66e3d9",
    "Titus Lunter",
    CardRules::unsupported(),
);

// FRA 188 — Room of Refuge
pub(in crate::card::sets) static ROOM_OF_REFUGE: CardRecord = CardRecord::new(
    "Room of Refuge",
    "9a467560-6676-4fc2-9400-768a79650aa4",
    "Constantin Marin",
    CardRules::new_land(&[]).with_abilities(&[
        AbilityDef::replacement(
            "This land enters tapped. As it enters, choose a color.",
            ReplacementEffectDef::Sequence(&[
                ReplacementEffectDef::ModifyBattlefieldEntry(
                    BattlefieldEntryModificationDef::Tapped,
                ),
                ReplacementEffectDef::Choose(ReplacementChoiceDef::Scalar(
                    BattlefieldEntryScalarChoiceDef::COLOR,
                )),
            ]),
        ),
        abilities::tap_for_mana(
            "{T}: Add one mana of the chosen color.",
            AddManaEffectDef::one_of_type(ManaTypeDef::ChosenColor),
        ),
        AbilityDef::activated_with_targets(
            "{5}, {T}, Sacrifice this land: Put two +1/+1 counters on target creature. Activate \
             only as a sorcery.",
            &[
                CostDef::Mana(mana_cost!("{5}")),
                CostDef::TapSource,
                CostDef::SacrificeSource,
            ],
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::HasType(CardType::Creature),
            )],
            EffectDef::AddCounters {
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                kind: CounterKind::PlusOnePlusOne,
                amount: ValueDef::Constant(2),
            },
        )
        .with_activation_timing(ActivationTimingDef::SorcerySpeed),
    ]),
);

// FRA 189 — Shipwreck Marsh (reprint)
const SHIPWRECK_MARSH_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_mid::SHIPWRECK_MARSH,
    "9e944c5b-68ac-4a30-bbd4-09a4288319ce",
    "Darren Tan",
);

// FRA 190 — Stingerquill Annex
pub(in crate::card::sets) static STINGERQUILL_ANNEX: CardRecord = CardRecord::new(
    "Stingerquill Annex",
    "6ede3143-69ac-4cbe-922a-d25b07c26da7",
    "Sergey Glushakov",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {B} or {R}.",
            AddManaEffectDef::choice(&[ManaColor::Black, ManaColor::Red]),
        ),
    ]),
);

// FRA 191 — Theorist's Sanctum
// Audit: unsupported — Needs an entry-time behold choice; behold is available only as a cost.
pub(in crate::card::sets) static THEORIST_S_SANCTUM: CardRecord = CardRecord::new(
    "Theorist's Sanctum",
    "22db5bba-46c9-4a26-821d-303ddb386ea4",
    "Adam Paquette",
    CardRules::unsupported(),
);

// FRA 192 — Theorix Annex
pub(in crate::card::sets) static THEORIX_ANNEX: CardRecord = CardRecord::new(
    "Theorix Annex",
    "97bbbd23-ecb1-4407-ac14-dede08532a1e",
    "Sergey Glushakov",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {U} or {B}.",
            AddManaEffectDef::choice(&[ManaColor::Blue, ManaColor::Black]),
        ),
    ]),
);

// FRA 193 — Transformative Commons
pub(in crate::card::sets) static TRANSFORMATIVE_COMMONS: CardRecord = CardRecord::new(
    "Transformative Commons",
    "b57d5be7-3157-4b49-aeb8-d7368ca7e9dd",
    "Lius Lasahido",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {G} or {U}.",
            AddManaEffectDef::choice(&[ManaColor::Green, ManaColor::Blue]),
        ),
    ]),
);

// FRA 194 — Vigorbloom Annex
pub(in crate::card::sets) static VIGORBLOOM_ANNEX: CardRecord = CardRecord::new(
    "Vigorbloom Annex",
    "db8c7bdd-76cd-4be0-ae0d-d430e9a5fe7a",
    "Sergey Glushakov",
    CardRules::new_land(&[]).with_abilities(&[
        enters_tapped_unless_you_control_a_planeswalker(),
        abilities::tap_for_mana(
            "{T}: Add {G} or {W}.",
            AddManaEffectDef::choice(&[ManaColor::Green, ManaColor::White]),
        ),
    ]),
);

// FRA 195 — Ajani Resolute
pub(in crate::card::sets) static AJANI_RESOLUTE: CardRecord = CardRecord::new(
    "Ajani Resolute",
    "a5e1a7dd-8c49-4435-935c-bcc78704082b",
    "Tyler Jacobson",
    CardRules::new_planeswalker(mana_cost!("{1}{W}"), &["Ajani"], 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::triggered(
                "Whenever you gain life, put a loyalty counter on Ajani.",
                TriggerEventDef::LifeGained(PlayerRelation::You),
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Source,
                    kind: CounterKind::Loyalty,
                    amount: ValueDef::Constant(1),
                },
            ),
            AbilityDef::activated(
                "0: You gain 1 life.",
                &[CostDef::Loyalty(ValueDef::Constant(0))],
                EffectDef::GainLife {
                    recipient: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(1),
                },
            ),
            AbilityDef::activated(
                "−4: Create a 2/2 white Cat Soldier creature token named Ajani's Pridemate with \
                 \"Whenever you gain life, put a +1/+1 counter on this token.\"",
                &[CostDef::Loyalty(ValueDef::Constant(-4))],
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                    TokenCharacteristics::creature(&["Cat", "Soldier"], &[ManaColor::White], 2, 2)
                        .with_name("Ajani's Pridemate")
                        .with_abilities(&[AbilityDef::triggered(
                            "Whenever you gain life, put a +1/+1 counter on this token.",
                            TriggerEventDef::LifeGained(PlayerRelation::You),
                            EffectDef::AddCounters {
                                object: EffectRecipientDef::Source,
                                kind: CounterKind::PlusOnePlusOne,
                                amount: ValueDef::Constant(1),
                            },
                        )])
                        .with_art(CardArt::new(
                            "7801c326-d740-4cec-9268-05348ace8302",
                            "Diego Gisbert",
                        )),
                ))),
            ),
            AbilityDef::activated(
                "−10: You get an emblem with \"Creatures you control get +2/+2.\"",
                &[CostDef::Loyalty(ValueDef::Constant(-10))],
                EffectDef::CreateEmblem {
                    creature_type: None,
                    emblem: EmblemCharacteristics::new(
                        "Ajani Resolute Emblem",
                        &[AbilityDef::static_ability(
                            "Creatures you control get +2/+2.",
                            EffectDef::StaticApply {
                                recipient: EffectRecipientDef::matching_objects(
                                    ObjectPredicateDef::HasType(CardType::Creature),
                                    &[ZoneKind::Battlefield],
                                    PlayerRelation::You,
                                ),
                                effect: AppliedEffectDef::modify_power_toughness(
                                    ValueDef::Constant(2),
                                    ValueDef::Constant(2),
                                ),
                            },
                        )],
                    ),
                },
            ),
        ]),
);

// FRA 196 — Danitha, Sword of Hope
pub(in crate::card::sets) static DANITHA_SWORD_OF_HOPE: CardRecord = CardRecord::new(
    "Danitha, Sword of Hope",
    "d5cb9810-3c2d-4ae4-b8b6-0155ca3f47ad",
    "Bryan Sola",
    CardRules::new_creature(mana_cost!("{2}{W}"), &["Human", "Knight"], 2, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::first_strike(),
            AbilityDef::triggered(
                "Whenever you cast an Equipment spell or a spell that targets a creature you \
                 control, draw a card. This ability triggers only once each turn.",
                TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                    ObjectPredicateDef::AnyOf(&[
                        ObjectPredicateDef::Subtype(SubtypeDef::from_name("Equipment")),
                        ObjectPredicateDef::TargetsObjectMatching(&ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Creature),
                            ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                        ])),
                    ]),
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ])),
                abilities::draw_cards(ValueDef::Constant(1)),
            )
            .triggering_at_most(1),
        ]),
);

// FRA 197 — Ghalta the Immovable
pub(in crate::card::sets) static GHALTA_THE_IMMOVABLE: CardRecord = CardRecord::new(
    "Ghalta the Immovable",
    "a9f3aa55-908f-42db-8135-4201433df850",
    "Antonio José Manzanedo",
    CardRules::new_creature(mana_cost!("{8}{W}"), &["Elder", "Dinosaur"], 0, 7)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::this_spell_cost_reduction(
                "This spell costs {X} less to cast, where X is the greatest toughness among \
                 creatures you control.",
                ValueDef::AggregateObjectValues(&ObjectValueAggregateDef {
                    objects: ObjectSetDef::Query(ObjectQueryDef::matching(
                        ObjectPredicateDef::HasType(CardType::Creature),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    )),
                    select: ObjectValueDef::Toughness,
                    operation: AggregateOperationDef::Maximum,
                }),
            ),
            // A permission rather than an ability removal: the creatures keep defender.
            AbilityDef::static_ability(
                "Creatures you control can attack as though they didn't have defender.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::HasType(CardType::Creature),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::Rule(AppliedRuleDef::MayAttackDespiteDefender),
                },
            ),
            AbilityDef::static_ability(
                "Each creature you control with toughness greater than its power assigns combat \
                 damage equal to its toughness rather than its power.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Creature),
                            ObjectPredicateDef::ToughnessGreaterThanItsPower,
                        ]),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::Rule(
                        AppliedRuleDef::AssignsCombatDamageEqualToToughness,
                    ),
                },
            ),
        ]),
);

// FRA 198 — Gideon's Memorial
pub(in crate::card::sets) static GIDEON_S_MEMORIAL: CardRecord = CardRecord::new(
    "Gideon's Memorial",
    "768c0e64-9907-417a-a763-c836fdf36883",
    "Manuel Castañón",
    CardRules::new_artifact(mana_cost!("{1}{W}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::static_ability(
                "Creature tokens you control get +1/+0 and have vigilance.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::Token,
                            ObjectPredicateDef::HasType(CardType::Creature),
                        ]),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::Composite(&[
                        AppliedEffectDef::modify_power_toughness(
                            ValueDef::Constant(1),
                            ValueDef::Constant(0),
                        ),
                        AppliedEffectDef::add_ability(&abilities::vigilance()),
                    ]),
                },
            ),
            AbilityDef::activated_mana(
                "{T}: Add one mana of any color. Spend this mana only to cast a planeswalker \
                 spell.",
                &[CostDef::TapSource],
                EffectDef::AddMana(AddManaEffectDef::any_color().with_restrictions(&[
                    ManaRestrictionDef::CastSpell(ObjectPredicateDef::HasType(
                        CardType::Planeswalker,
                    )),
                ])),
            ),
            AbilityDef::activated_with_targets(
                "{1}{W}, Discard this card: It deals 4 damage to target attacking or blocking \
                 creature.",
                &[CostDef::Mana(mana_cost!("{1}{W}")), CostDef::DiscardSource],
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::AttackingOrBlocking,
                    ]),
                )],
                EffectDef::damage(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ValueDef::Constant(4),
                ),
            )
            .with_source_zones(&[ZoneKind::Hand]),
        ]),
);

// FRA 199 — Koth of the Homestead
pub(in crate::card::sets) static KOTH_OF_THE_HOMESTEAD: CardRecord = CardRecord::new(
    "Koth of the Homestead",
    "920703fd-2a2f-454b-8829-af8f2afda4f4",
    "Kieran Yanner",
    CardRules::new_creature(mana_cost!("{2}{W}"), &["Human", "Citizen"], 2, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::triggered(
                "Landfall — Whenever a land you control enters, you gain 1 life.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Land),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                EffectDef::GainLife {
                    recipient: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(1),
                },
            ),
            AbilityDef::triggered_with_targets(
                "Whenever a Plains you control enters, put a +1/+1 counter on target creature.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::Subtype(SubtypeDef::from_name("Plains")),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
            ),
        ]),
);

// FRA 200 — Liliana the Faultless
pub(in crate::card::sets) static LILIANA_THE_FAULTLESS: CardRecord = CardRecord::new(
    "Liliana the Faultless",
    "70d8c400-87dc-4f15-808f-e54a95d779fc",
    "Marta Nael",
    CardRules::new_creature(mana_cost!("{W}"), &["Human", "Cleric"], 1, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::triggered(
                "Whenever another creature or planeswalker you control enters, you gain 1 life.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        A_CREATURE_OR_PLANESWALKER,
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                EffectDef::GainLife {
                    recipient: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(1),
                },
            ),
            AbilityDef::activated_with_targets(
                "{1}, {T}, Discard a card: Another target creature or planeswalker you control \
                 gains hexproof until end of turn.",
                &[
                    CostDef::Mana(mana_cost!("{1}")),
                    CostDef::TapSource,
                    CostDef::discard(ObjectPredicateDef::Any),
                ],
                &[
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: A_CREATURE_OR_PLANESWALKER,
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    })
                    .excluding_source(),
                ],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::add_ability(&abilities::hexproof()),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ]),
);

// FRA 201 — Lyra, Archangel of Dawn
pub(in crate::card::sets) static LYRA_ARCHANGEL_OF_DAWN: CardRecord = CardRecord::new(
    "Lyra, Archangel of Dawn",
    "86a3866e-68a8-402c-baf0-1908e98e3995",
    "Victor Adame Minguez",
    CardRules::new_creature(mana_cost!("{2}{W}"), &["Angel", "Knight"], 3, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flying(),
            AbilityDef::triggered(
                "Whenever you gain life, put a +1/+1 counter on each Angel you control.",
                TriggerEventDef::LifeGained(PlayerRelation::You),
                EffectDef::AddCounters {
                    object: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::Subtype(SubtypeDef::from_name("Angel")),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
            ),
        ]),
);

// FRA 202 — Rescue Girl, First Responder
pub(in crate::card::sets) static RESCUE_GIRL_FIRST_RESPONDER: CardRecord = CardRecord::new(
    "Rescue Girl, First Responder",
    "699874e3-1ccf-4a6c-8371-61040de82d08",
    "Cristi Balanescu",
    CardRules::new_creature(mana_cost!("{2}{W}"), &["Human", "Cleric"], 1, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flying(),
            AbilityDef::activated_with_targets(
                "{T}: Return another target permanent you control to its owner's hand. Activate \
                 only during your turn.",
                &[CostDef::TapSource],
                &[
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::Any,
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    })
                    .excluding_source(),
                ],
                EffectDef::move_to_zone(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ZoneKind::Hand,
                    ZonePlacement::Top,
                ),
            )
            .with_activation_timing(ActivationTimingDef::YourTurn),
        ]),
);

// FRA 203 — Saheeli, Consul of Oversight
// Audit: unsupported — Needs a trigger event for scrying or surveilling, published wherever a scry
// or surveil of at least one card is performed.
pub(in crate::card::sets) static SAHEELI_CONSUL_OF_OVERSIGHT: CardRecord = CardRecord::new(
    "Saheeli, Consul of Oversight",
    "07572be0-6610-493c-a21e-14b78e9805c9",
    "Billy Christian",
    CardRules::unsupported(),
);

// FRA 204 — Teyo, Lightshield Expert
pub(in crate::card::sets) static TEYO_LIGHTSHIELD_EXPERT: CardRecord = CardRecord::new(
    "Teyo, Lightshield Expert",
    "5f7521d7-9f1f-4f03-b2ea-dd2a1b1e4e5b",
    "Anna Steinbauer",
    CardRules::new_creature(mana_cost!("{1}{W}"), &["Human", "Cleric"], 1, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flash(),
            abilities::enters_trigger_with_targets(
                "When Teyo enters, target permanent you control gains hexproof until end of \
                 turn. Put a +1/+1 counter on it if it's a creature. Put a loyalty counter on it \
                 if it's a planeswalker. (It can't be the target of spells or abilities your \
                 opponents control.)",
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::Any,
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    },
                )],
                EffectDef::Sequence(&[
                    EffectDef::Apply {
                        recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        effect: AppliedEffectDef::add_ability(&abilities::hexproof()),
                        duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                    },
                    TEYO_S_COUNTERS,
                ]),
            ),
        ]),
);

// FRA 205 — Thalia, the Survivor
pub(in crate::card::sets) static THALIA_THE_SURVIVOR: CardRecord = CardRecord::new(
    "Thalia, the Survivor",
    "80226231-9e70-430e-aabc-f262f70b9226",
    "Ryan Pancoast",
    CardRules::new_creature(mana_cost!("{3}{W}"), &["Human", "Soldier"], 3, 4)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::lifelink(),
            AbilityDef::static_ability(
                "Noncreature spells your opponents cast cost {1} more to cast.",
                EffectDef::ModifyCost(CostModificationDef::increase_spell(
                    ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Creature)),
                    PlayerRelation::Opponent,
                    mana_cost!("{1}"),
                )),
            ),
        ]),
);

// FRA 206 — Tomik, Orzhov Lawmage
// Audit: unsupported — Needs a per-defender attacker cap, and the attack-rules reader visiting
// static abilities granted to planeswalkers.
pub(in crate::card::sets) static TOMIK_ORZHOV_LAWMAGE: CardRecord = CardRecord::new(
    "Tomik, Orzhov Lawmage",
    "7ca95235-6e54-4ff8-bc2e-6a3d483ff007",
    "Randy Vargas",
    CardRules::unsupported(),
);

// FRA 207 — Way of the Healer
pub(in crate::card::sets) static WAY_OF_THE_HEALER: CardRecord = CardRecord::new(
    "Way of the Healer",
    "50326a2a-7e10-464b-a97e-e880bda0558c",
    "Pauline Voss",
    CardRules::new_enchantment(mana_cost!("{3}{W}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Way of the Healer enters, empower Jace 5. (Put five loyalty counters on \
                 a Jace token you control. If you don't control one, first create a blue Jace \
                 planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
                empower_jace(ValueDef::Constant(5)),
            ),
            planeswalkers_you_control_have(
                "Planeswalkers you control have \"[−2]: Create a 2/2 colorless Wizard Soldier \
                 creature token named Cadet. Surveil 1.\"",
                &AbilityDef::activated(
                    "−2: Create a 2/2 colorless Wizard Soldier creature token named Cadet. \
                     Surveil 1.",
                    &[CostDef::Loyalty(ValueDef::Constant(-2))],
                    EffectDef::Sequence(&[
                        EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(CADET_TOKEN))),
                        abilities::surveil(ValueDef::Constant(1)),
                    ]),
                ),
            ),
        ]),
);

// FRA 208 — Way of the Mentor
pub(in crate::card::sets) static WAY_OF_THE_MENTOR: CardRecord = CardRecord::new(
    "Way of the Mentor",
    "1a59d5b1-12d6-486b-bd29-ca371359addd",
    "Danny Schwartz",
    CardRules::new_enchantment(mana_cost!("{2}{W}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Way of the Mentor enters, empower Jace 5. (Put five loyalty counters on \
                 a Jace token you control. If you don't control one, first create a blue Jace \
                 planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
                empower_jace(ValueDef::Constant(5)),
            ),
            AbilityDef::triggered(
                "Whenever you gain life, put a loyalty counter on each planeswalker you control.",
                TriggerEventDef::LifeGained(PlayerRelation::You),
                EffectDef::AddCounters {
                    object: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::HasType(CardType::Planeswalker),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    kind: CounterKind::Loyalty,
                    amount: ValueDef::Constant(1),
                },
            ),
        ]),
);

// FRA 209 — Yoshimaru, Beloved Companion
// Audit: unsupported — Needs a counter-placement replacement event that changes how many counters
// are put on a permanent.
pub(in crate::card::sets) static YOSHIMARU_BELOVED_COMPANION: CardRecord = CardRecord::new(
    "Yoshimaru, Beloved Companion",
    "384f3b7d-8d7f-41bf-bebd-64e8babe7fca",
    "Brian Valeza",
    CardRules::unsupported(),
);

// FRA 210 — Yuriko, Blade of the Mighty
// Audit: unsupported — Needs a static cast-and-activate prohibition scoped to the combat phase.
pub(in crate::card::sets) static YURIKO_BLADE_OF_THE_MIGHTY: CardRecord = CardRecord::new(
    "Yuriko, Blade of the Mighty",
    "ccbe92a5-42bc-4228-9d5a-212df2f5dc15",
    "Lie Setiawan",
    CardRules::unsupported(),
);

// FRA 211 — Arni, Humble Scribe
pub(in crate::card::sets) static ARNI_HUMBLE_SCRIBE: CardRecord = CardRecord::new(
    "Arni, Humble Scribe",
    "8e3a2239-9348-4639-9318-e9e35b2cf86b",
    "Vincent Christiaens",
    CardRules::new_creature(mana_cost!("{2}{U}"), &["Human", "Wizard"], 3, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::triggered(
                "Whenever another nontoken creature you control enters, untap Arni.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Token),
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                EffectDef::Untap {
                    object: EffectRecipientDef::Source,
                },
            ),
            AbilityDef::activated(
                "{T}: Draw a card, then discard a card.",
                &[CostDef::TapSource],
                EffectDef::Sequence(&[
                    abilities::draw_cards(ValueDef::Constant(1)),
                    EffectDef::Discard {
                        recipient: EffectRecipientDef::Controller,
                        amount: ValueDef::Constant(1),
                        selection: DiscardSelectionDef::RecipientChooses,
                        then: None,
                    },
                ]),
            ),
        ]),
);

// FRA 212 — Chandra, Chill of Compliance
// Audit: unsupported — Needs a surveil outcome binding naming the cards put into the graveyard
// this way.
pub(in crate::card::sets) static CHANDRA_CHILL_OF_COMPLIANCE: CardRecord = CardRecord::new(
    "Chandra, Chill of Compliance",
    "240f58ab-944c-4f4c-9df9-5f40b132bf3e",
    "Kieran Yanner",
    CardRules::unsupported(),
);

// FRA 213 — Fblthp, Impossibly Lost
pub(in crate::card::sets) static FBLTHP_IMPOSSIBLY_LOST: CardRecord = CardRecord::new(
    "Fblthp, Impossibly Lost",
    "a3a2edbb-d144-4670-acad-17316cea98d2",
    "Simon Dominic",
    CardRules::new_creature(mana_cost!("{1}{U}"), &["Homunculus"], 1, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[AbilityDef::triggered(
            "When one or more of your opponents are dealt combat damage during your turn, draw \
             two cards. If your library has no cards in it, you win the game. Fblthp's owner \
             shuffles him into their library. (If you draw from an empty library this way, you \
             still win the game.)",
            // Whoever's creature dealt it: the clause asks only who was dealt the damage.
            TriggerEventDef::While {
                event: &TriggerEventDef::CombatDamageDealtToPlayers {
                    sources: ObjectPredicateDef::Any,
                    players: PlayerRelation::Opponent,
                },
                condition: &TriggerConditionDef::ActivePlayer(PlayerRelation::You),
            },
            EffectDef::Sequence(&[
                abilities::draw_cards(ValueDef::Constant(2)),
                // Winning here comes before the next state-based check, which is where a
                // draw from an empty library would otherwise lose the game (CR 704.5b).
                EffectDef::IfCondition {
                    condition: &TriggerConditionDef::ValueComparison(&ValueComparisonDef {
                        left: ValueDef::LibrarySize(PlayerRelation::You),
                        comparison: ComparisonDef::Equal,
                        right: ValueDef::Constant(0),
                    }),
                    then: &EffectDef::WinTheGame {
                        player: EffectRecipientDef::Controller,
                    },
                },
                EffectDef::move_to_zone(
                    EffectRecipientDef::Source,
                    ZoneKind::Library,
                    ZonePlacement::Top,
                ),
                // The library is shuffled even if Fblthp has already left (CR 701.24c).
                EffectDef::ShuffleLibrary {
                    player: EffectRecipientDef::player(PlayerRefDef::OwnerOf(ObjectRefDef::Source)),
                },
            ]),
        )]),
);

// FRA 214 — Geist of Saint Thalia
pub(in crate::card::sets) static GEIST_OF_SAINT_THALIA: CardRecord = CardRecord::new(
    "Geist of Saint Thalia",
    "9c334530-0880-46b5-a358-9603eee3cecf",
    "Ryan Pancoast",
    CardRules::new_creature(mana_cost!("{1}{U}"), &["Spirit", "Cleric"], 1, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flying(),
            AbilityDef::static_ability(
                "Noncreature spells you cast cost {1} less to cast.",
                EffectDef::ModifyCost(CostModificationDef::reduce_spell(
                    ObjectPredicateDef::Not(&ObjectPredicateDef::HasType(CardType::Creature)),
                    PlayerRelation::You,
                    ValueDef::Constant(1),
                )),
            ),
        ]),
);

// FRA 215 — Hapatra, the Desert Frost
pub(in crate::card::sets) static HAPATRA_THE_DESERT_FROST: CardRecord = CardRecord::new(
    "Hapatra, the Desert Frost",
    "85faaa9d-4656-4365-871d-7cba53ed0996",
    "Jodie Muir",
    CardRules::new_creature(mana_cost!("{3}{U}"), &["Human", "Wizard"], 4, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            // One opponent in this engine, so "for each opponent, up to one target creature
            // that player controls" is a single slot over what that opponent controls.
            abilities::enters_trigger_with_targets(
                "When Hapatra enters, for each opponent, tap up to one target creature that \
                 player controls. Put a stun counter on each of those creatures. (If a \
                 permanent with a stun counter would become untapped, remove one from it \
                 instead.)",
                &[AbilityTargetDef::up_to(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Creature),
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::Opponent),
                        owner: None,
                    },
                    1,
                )],
                EffectDef::Sequence(&[
                    EffectDef::Tap {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    },
                    EffectDef::AddCounters {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        kind: CounterKind::Stun,
                        amount: ValueDef::Constant(1),
                    },
                ]),
            ),
            AbilityDef::activated_with_targets(
                "{2}{U}: Untap target creature.",
                &[CostDef::Mana(mana_cost!("{2}{U}"))],
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Untap {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                },
            ),
        ]),
);

// FRA 216 — Jace, Reality Sculptor
pub(in crate::card::sets) static JACE_REALITY_SCULPTOR: CardRecord = CardRecord::new(
    "Jace, Reality Sculptor",
    "74087795-0b38-4fd2-9841-147583baca41",
    "Martina Fačková",
    CardRules::new_planeswalker(mana_cost!("{3}{U}{U}"), &["Jace"], 5)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::activated(
                "+1: Empower Jace X, where X is the number of Islands you control.",
                &[CostDef::Loyalty(ValueDef::Constant(1))],
                empower_jace(ValueDef::CountMatchingObjects(
                    &ObjectQueryDef::controlled_by(
                        ObjectPredicateDef::Subtype(SubtypeDef::from_name("Island")),
                        &[ZoneKind::Battlefield],
                        PlayerSetDef::Related(PlayerRelation::You),
                    ),
                )),
            ),
            AbilityDef::activated(
                "−3: Until your next turn, whenever a creature attacks you or a planeswalker \
                 you control, it gets -5/-0 until end of turn.",
                &[CostDef::Loyalty(ValueDef::Constant(-3))],
                EffectDef::InstallTrigger(InstalledTriggerDef {
                    ability: &AbilityDef::triggered(
                        "Whenever a creature attacks you or a planeswalker you control, it gets \
                         -5/-0 until end of turn.",
                        TriggerEventDef::Attacks(AttackEventMatcherDef::attacking(
                            ObjectPredicateDef::HasType(CardType::Creature),
                            PlayerRelation::You,
                        )),
                        EffectDef::Apply {
                            recipient: EffectRecipientDef::TriggeringObject,
                            effect: AppliedEffectDef::modify_power_toughness(
                                ValueDef::Constant(-5),
                                ValueDef::Constant(0),
                            ),
                            duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                        },
                    ),
                    lifetime: InstalledTriggerLifetimeDef::UntilNextTurn(
                        PlayerRefDef::EffectController,
                    ),
                }),
            ),
            AbilityDef::activated(
                "0: Exile all but the bottom card of each opponent's library. Activate only if \
                 there are twenty-five or more loyalty counters among Jaces you control.",
                &[CostDef::Loyalty(ValueDef::Constant(0))],
                // Everything above the bottom card: one fewer than the library holds, from
                // the top. An empty library has nothing above its bottom card.
                abilities::bind_top_cards_then(
                    PlayerRefDef::Opponent,
                    ValueDef::Sum(&SumValueDef::new(
                        ValueDef::LibrarySize(PlayerRelation::Opponent),
                        ValueDef::Constant(-1),
                    )),
                    &EffectDef::MoveObjects(MoveObjectsDef {
                        input: ObjectSetDef::Binding(ParentBinding),
                        from: Some(ZoneKind::Library),
                        zone: ZoneKind::Exile,
                        placement: ZonePlacement::Top,
                        moved: None,
                        then: &EffectDef::None,
                    }),
                ),
            )
            .with_activation_condition(&TriggerConditionDef::ValueComparison(
                &ValueComparisonDef {
                    left: ValueDef::AggregateObjectValues(&ObjectValueAggregateDef {
                        objects: ObjectSetDef::Query(ObjectQueryDef::controlled_by(
                            ObjectPredicateDef::Subtype(SubtypeDef::from_name("Jace")),
                            &[ZoneKind::Battlefield],
                            PlayerSetDef::Related(PlayerRelation::You),
                        )),
                        select: ObjectValueDef::Counters(CounterKind::Loyalty),
                        operation: AggregateOperationDef::Sum,
                    }),
                    comparison: ComparisonDef::GreaterOrEqual,
                    right: ValueDef::Constant(25),
                },
            )),
        ]),
);

// FRA 217 — Lyra, Tolarian Archangel
pub(in crate::card::sets) static LYRA_TOLARIAN_ARCHANGEL: CardRecord = CardRecord::new(
    "Lyra, Tolarian Archangel",
    "a5183681-447b-4023-91f7-00e9338f4417",
    "Victor Adame Minguez",
    CardRules::new_creature(mana_cost!("{1}{U}{U}"), &["Angel", "Wizard"], 3, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flying(),
            AbilityDef::triggered_if(
                "At the beginning of each end step, if you've drawn three or more cards this \
                 turn, create a 3/3 blue Angel creature token with flying.",
                TriggerEventDef::StepBegins {
                    step: TurnStepDef::End,
                    player: PlayerRelation::Any,
                },
                &TriggerConditionDef::ValueComparison(&ValueComparisonDef {
                    left: ValueDef::CardsDrawnThisTurn(PlayerRelation::You),
                    comparison: ComparisonDef::GreaterOrEqual,
                    right: ValueDef::Constant(3),
                }),
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                    TokenCharacteristics::creature(&["Angel"], &[ManaColor::Blue], 3, 3)
                        .with_abilities(&[abilities::flying()])
                        .with_art(CardArt::new(
                            "5165055a-89e1-471b-ac96-732e593c910b",
                            "Raluca Marinescu",
                        )),
                ))),
            ),
            // Each activation is its own listener, so two of them draw four cards.
            AbilityDef::activated(
                "{3}{U}{U}: Until end of turn, whenever Lyra deals combat damage to a player, \
                 draw two cards.",
                &[CostDef::Mana(mana_cost!("{3}{U}{U}"))],
                EffectDef::InstallTrigger(InstalledTriggerDef {
                    ability: &AbilityDef::triggered(
                        "Whenever Lyra deals combat damage to a player, draw two cards.",
                        TriggerEventDef::combat_damage_to_player(ObjectPredicateDef::Source),
                        abilities::draw_cards(ValueDef::Constant(2)),
                    ),
                    lifetime: InstalledTriggerLifetimeDef::ThisTurn,
                }),
            ),
        ]),
);

// FRA 218 — Proft, Consulting Detective
// Audit: unsupported — Needs a trigger event for scrying or surveilling, published wherever a scry
// or surveil of at least one card is performed.
pub(in crate::card::sets) static PROFT_CONSULTING_DETECTIVE: CardRecord = CardRecord::new(
    "Proft, Consulting Detective",
    "b8466593-40fe-4557-89b2-760c1c92087b",
    "Lindsey Look",
    CardRules::unsupported(),
);

// FRA 219 — Ruric Thar, Biomagus
pub(in crate::card::sets) static RURIC_THAR_BIOMAGUS: CardRecord = CardRecord::new(
    "Ruric Thar, Biomagus",
    "00af4e87-5576-4a43-9422-4c35b2b66775",
    "Olivier Bernard",
    CardRules::new_creature(mana_cost!("{4}{U}{U}"), &["Ogre", "Crab", "Wizard"], 4, 6)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flying(),
            // "Prowess, prowess": two instances, each triggering on its own.
            abilities::prowess(),
            abilities::prowess(),
            AbilityDef::triggered(
                "Whenever Ruric Thar becomes the target of a spell or ability an opponent \
                 controls, draw a card.",
                TriggerEventDef::becomes_targeted(ObjectPredicateDef::ControlledBy(
                    PlayerRelation::Opponent,
                )),
                abilities::draw_cards(ValueDef::Constant(1)),
            ),
        ]),
);

// FRA 220 — Samut, Tyrant of Naktamun
// Audit: unsupported — Needs split second granted to spells on the stack; the rule reads the
// printed keyword only.
pub(in crate::card::sets) static SAMUT_TYRANT_OF_NAKTAMUN: CardRecord = CardRecord::new(
    "Samut, Tyrant of Naktamun",
    "6d7d8fa7-ce69-4a8c-9af0-55571393a244",
    "Grzegorz Rutkowski",
    CardRules::unsupported(),
);

// FRA 221 — Tetsuko Umezawa, Fugitive (reprint)
const TETSUKO_UMEZAWA_FUGITIVE_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_dom::TETSUKO_UMEZAWA_FUGITIVE,
    "3983d71e-3c23-4b36-b331-08e0707d8245",
    "Tuan Duong Chu",
);

// FRA 222 — Traxos, Academy Guardian
pub(in crate::card::sets) static TRAXOS_ACADEMY_GUARDIAN: CardRecord = CardRecord::new(
    "Traxos, Academy Guardian",
    "a349800f-b634-4e74-a9d9-185df37ad909",
    "Aaron Miller",
    CardRules::new_artifact_creature(mana_cost!("{3}{U}"), &["Dragon", "Construct"], 1, 5)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::this_spell_cost_reduction(
                "This spell costs {2} less to cast if you've cast a noncreature spell this turn.",
                ValueDef::IfCondition(&ConditionValueDef {
                    condition: &TriggerConditionDef::ValueComparison(&ValueComparisonDef {
                        left: ValueDef::CountSpellsCastThisTurn(&SpellCastQueryDef {
                            player: PlayerRelation::You,
                            spell: ObjectPredicateDef::NoncreatureSpell,
                        }),
                        comparison: ComparisonDef::GreaterOrEqual,
                        right: ValueDef::Constant(1),
                    }),
                    then: ValueDef::Constant(2),
                    otherwise: ValueDef::Constant(0),
                }),
            ),
            abilities::flying(),
            abilities::vigilance(),
            abilities::prowess(),
        ]),
);

// FRA 223 — Way of the Cryomancer
// Audit: unsupported — Needs a delayed trigger for the next matching spell cast this turn.
pub(in crate::card::sets) static WAY_OF_THE_CRYOMANCER: CardRecord = CardRecord::new(
    "Way of the Cryomancer",
    "838b0efb-7398-4df9-8fdf-b8af43b47938",
    "Andrey Kuzinskiy",
    CardRules::unsupported(),
);

// FRA 224 — Way of the Mind Sculptor
// Audit: unsupported — Needs a trigger event for a player activating a loyalty ability that
// carries the number of loyalty counters removed to activate it.
pub(in crate::card::sets) static WAY_OF_THE_MIND_SCULPTOR: CardRecord = CardRecord::new(
    "Way of the Mind Sculptor",
    "5838af68-66c3-4fe8-ab89-0a1721b0cfeb",
    "Josh Hass",
    CardRules::unsupported(),
);

// FRA 225 — Yargle, Goliath of Otaria
pub(in crate::card::sets) static YARGLE_GOLIATH_OF_OTARIA: CardRecord = CardRecord::new(
    "Yargle, Goliath of Otaria",
    "f45ba926-6496-4bd4-96eb-663946d56bbf",
    "Jehan Choo",
    CardRules::new_creature(mana_cost!("{4}{U}"), &["Frog", "Spirit"], 3, 9)
        .with_supertype(CardSupertype::Legendary),
);

// FRA 226 — Yuriko, Hope from the Shadows
pub(in crate::card::sets) static YURIKO_HOPE_FROM_THE_SHADOWS: CardRecord = CardRecord::new(
    "Yuriko, Hope from the Shadows",
    "45e81487-8b8c-480b-922a-eaa9edc7201d",
    "Lie Setiawan",
    CardRules::new_creature(mana_cost!("{U}"), &["Human", "Ninja"], 1, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flash(),
            AbilityDef::modal_triggered(
                "When Yuriko enters, choose one —",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::Source,
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                &[
                    AbilityDef::spell_with_targets(
                        "Target creature gets -X/-0 until end of turn, where X is the number of \
                         cards in your graveyard.",
                        &[AbilityTargetDef::exactly_one_permanent(
                            ObjectPredicateDef::HasType(CardType::Creature),
                        )],
                        EffectDef::Apply {
                            recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            effect: AppliedEffectDef::modify_power_toughness(
                                ValueDef::Negate(&ValueDef::CountMatchingObjects(
                                    &ObjectQueryDef::owned_by(
                                        ObjectPredicateDef::Any,
                                        &[ZoneKind::Graveyard],
                                        PlayerSetDef::Related(PlayerRelation::You),
                                    ),
                                )),
                                ValueDef::Constant(0),
                            ),
                            duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                        },
                    ),
                    AbilityDef::spell(
                        "Surveil 2. (Look at the top two cards of your library, then put any \
                         number of them into your graveyard and the rest on top of your library \
                         in any order.)",
                        abilities::surveil(ValueDef::Constant(2)),
                    ),
                ],
            ),
        ]),
);

// FRA 227 — Danitha, Spear of Agony
// Audit: unsupported — Needs a cast trigger matching a spell that targets a player;
// HasDeclaredPlayerTarget is rejected in trigger predicates.
pub(in crate::card::sets) static DANITHA_SPEAR_OF_AGONY: CardRecord = CardRecord::new(
    "Danitha, Spear of Agony",
    "6489814b-3d10-423e-988c-324740d36748",
    "Bryan Sola",
    CardRules::unsupported(),
);

// FRA 228 — Gallia, Tragic Host
// Audit: unsupported — Needs exiling another creature card from your graveyard as a cost on an
// ability activated from the graveyard.
pub(in crate::card::sets) static GALLIA_TRAGIC_HOST: CardRecord = CardRecord::new(
    "Gallia, Tragic Host",
    "498fa810-8522-4020-b773-52ad404c9f65",
    "Andrea Piparo",
    CardRules::unsupported(),
);

// FRA 229 — Garruk, Veiled Butcher
// Audit: unsupported — Needs a static would-die replacement that exiles instead, filtered by
// controller; catalog validation rejects that shape.
pub(in crate::card::sets) static GARRUK_VEILED_BUTCHER: CardRecord = CardRecord::new(
    "Garruk, Veiled Butcher",
    "d48bfb8a-d135-45f3-be99-4694b4b9ab93",
    "Victor Adame Minguez",
    CardRules::unsupported(),
);

// FRA 230 — Gideon the Oathless
// Audit: unsupported — Needs a trigger event for a player activating a loyalty ability.
pub(in crate::card::sets) static GIDEON_THE_OATHLESS: CardRecord = CardRecord::new(
    "Gideon the Oathless",
    "c985b0d1-25bd-4069-aab7-a566ff27a8f6",
    "Manuel Castañón",
    CardRules::unsupported(),
);

// FRA 231 — Liliana the Repentant
pub(in crate::card::sets) static LILIANA_THE_REPENTANT: CardRecord = CardRecord::new(
    "Liliana the Repentant",
    "1eb25a6c-d6b4-465d-990e-f1ab86b26b69",
    "Marta Nael",
    CardRules::new_creature(mana_cost!("{1}{B}"), &["Human", "Warlock"], 2, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::triggered(
                "Whenever another creature or planeswalker you control enters, mill two cards.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        A_CREATURE_OR_PLANESWALKER,
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                EffectDef::Mill {
                    player: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(2),
                },
            ),
            exhaust(
                AbilityDef::activated_with_targets(
                    "Exhaust — {5}{B}: Return target creature or planeswalker card from your \
                     graveyard to the battlefield. Put a +1/+1 counter on Liliana. Activate only \
                     as a sorcery. (Activate each exhaust ability only once.)",
                    &[CostDef::Mana(mana_cost!("{5}{B}"))],
                    &[AbilityTargetDef::exactly_one(
                        AbilityTargetPredicate::Object {
                            object: A_CREATURE_OR_PLANESWALKER,
                            zones: &[ZoneKind::Graveyard],
                            controller: None,
                            owner: Some(PlayerRelation::You),
                        },
                    )],
                    EffectDef::Sequence(&[
                        EffectDef::move_to_zone(
                            EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            ZoneKind::Battlefield,
                            ZonePlacement::Top,
                        ),
                        EffectDef::AddCounters {
                            object: EffectRecipientDef::Source,
                            kind: CounterKind::PlusOnePlusOne,
                            amount: ValueDef::Constant(1),
                        },
                    ]),
                )
                .with_activation_timing(ActivationTimingDef::SorcerySpeed),
            ),
        ]),
);

// FRA 232 — Loot, the Anomaly
// Audit: unsupported — Needs combat damage assigned by magnitude for an attacker or blocker with
// negative power.
pub(in crate::card::sets) static LOOT_THE_ANOMALY: CardRecord = CardRecord::new(
    "Loot, the Anomaly",
    "4f6fd2fa-8bc8-4743-bbc8-b56475d64eff",
    "Filip Burburan",
    CardRules::unsupported(),
);

// FRA 233 — Mabel, Bitter Recluse
// Audit: unsupported — Needs removal of up to three counters of any kinds from one target.
pub(in crate::card::sets) static MABEL_BITTER_RECLUSE: CardRecord = CardRecord::new(
    "Mabel, Bitter Recluse",
    "b2a412b0-2ae4-4552-bc5e-70654b6b9b4e",
    "Matt Stewart",
    CardRules::unsupported(),
);

// FRA 234 — Massacre Girl, Most Wanted
// Audit: unsupported — Needs a damage event matcher restricted to noncombat damage.
pub(in crate::card::sets) static MASSACRE_GIRL_MOST_WANTED: CardRecord = CardRecord::new(
    "Massacre Girl, Most Wanted",
    "9028d31f-9c41-47e3-885b-6a869bca8178",
    "Cristi Balanescu",
    CardRules::unsupported(),
);

// FRA 235 — Proft, Sinister Mastermind
// Audit: unsupported — Needs a conditional restriction on casting the card itself.
pub(in crate::card::sets) static PROFT_SINISTER_MASTERMIND: CardRecord = CardRecord::new(
    "Proft, Sinister Mastermind",
    "d36b0e06-cb82-4c48-bf35-e76f109116f6",
    "Lindsey Look",
    CardRules::unsupported(),
);

// FRA 236 — Teyo, Diamondblade Mage
pub(in crate::card::sets) static TEYO_DIAMONDBLADE_MAGE: CardRecord = CardRecord::new(
    "Teyo, Diamondblade Mage",
    "100c3b67-0c92-4224-b5ed-67789c612df7",
    "Anna Steinbauer",
    CardRules::new_creature(mana_cost!("{3}{B}"), &["Human", "Warlock"], 3, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flash(),
            abilities::enters_trigger_with_targets(
                "When Teyo enters, target permanent you control gains deathtouch until end of \
                 turn. Put a +1/+1 counter on it if it's a creature. Put a loyalty counter on it \
                 if it's a planeswalker.",
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::Any,
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    },
                )],
                EffectDef::Sequence(&[
                    EffectDef::Apply {
                        recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        effect: AppliedEffectDef::add_ability(&abilities::deathtouch()),
                        duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                    },
                    TEYO_S_COUNTERS,
                ]),
            ),
        ]),
);

// FRA 237 — Tinybones, Pocket Nuisance
pub(in crate::card::sets) static TINYBONES_POCKET_NUISANCE: CardRecord = CardRecord::new(
    "Tinybones, Pocket Nuisance",
    "2f47ddf7-35b6-4205-8045-f057914c5f64",
    "Jesper Ejsing",
    CardRules::new_creature(mana_cost!("{2}{B}"), &["Skeleton", "Rogue"], 2, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Tinybones enters, each opponent discards a card.",
                EffectDef::Discard {
                    recipient: EffectRecipientDef::Opponent,
                    amount: ValueDef::Constant(1),
                    selection: DiscardSelectionDef::RecipientChooses,
                    then: None,
                },
            ),
            // One trigger for each player's discard, however many cards it took.
            AbilityDef::triggered(
                "Whenever a player discards one or more cards, Tinybones deals 1 damage to each \
                 opponent.",
                TriggerEventDef::DiscardedCards(PlayerRelation::Any),
                EffectDef::damage(EffectRecipientDef::Opponent, ValueDef::Constant(1)),
            ),
        ]),
);

// FRA 238 — Way of the Deathbringer
pub(in crate::card::sets) static WAY_OF_THE_DEATHBRINGER: CardRecord = CardRecord::new(
    "Way of the Deathbringer",
    "12dd46b2-e892-4660-b120-55766fd4d878",
    "Joshua Raphael",
    CardRules::new_enchantment(mana_cost!("{2}{B}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Way of the Deathbringer enters, empower Jace 5. (Put five loyalty \
                 counters on a Jace token you control. If you don't control one, first create \
                 a blue Jace planeswalker token with \"[−1]: Surveil 1\" and \
                 \"[−3]: Draw a card.\")",
                empower_jace(ValueDef::Constant(5)),
            ),
            planeswalkers_you_control_have(
                "Planeswalkers you control have \"[−2]: You may sacrifice a creature. If you \
                 do, create a 4/4 green Beast creature token with trample.\"",
                &AbilityDef::activated(
                    "−2: You may sacrifice a creature. If you do, create a 4/4 green Beast \
                     creature token with trample.",
                    &[CostDef::Loyalty(ValueDef::Constant(-2))],
                    EffectDef::PayOr(PayOrDef::optional(
                        &[CostDef::sacrifice_permanent(ObjectPredicateDef::HasType(
                            CardType::Creature,
                        ))],
                        &EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                            BEAST_TOKEN,
                        ))),
                    )),
                ),
            ),
        ]),
);

// FRA 239 — Way of the Necromancer
pub(in crate::card::sets) static WAY_OF_THE_NECROMANCER: CardRecord = CardRecord::new(
    "Way of the Necromancer",
    "a0ff9689-ea49-4fff-b37c-4abbaeb0f73d",
    "Pauline Voss",
    CardRules::new_enchantment(mana_cost!("{1}{B}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Way of the Necromancer enters, empower Jace 2. (Put two loyalty counters \
                 on a Jace token you control. If you don't control one, first create a blue \
                 Jace planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
                empower_jace(ValueDef::Constant(2)),
            ),
            AbilityDef::triggered(
                "Whenever a creature you control dies, put a loyalty counter on each \
                 planeswalker you control.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    Some(ZoneKind::Battlefield),
                    Some(ZoneKind::Graveyard),
                ),
                EffectDef::AddCounters {
                    object: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::HasType(CardType::Planeswalker),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    kind: CounterKind::Loyalty,
                    amount: ValueDef::Constant(1),
                },
            ),
        ]),
);

// FRA 240 — Winter, Tormented Loner
pub(in crate::card::sets) static WINTER_TORMENTED_LONER: CardRecord = CardRecord::new(
    "Winter, Tormented Loner",
    "9670f754-f41f-45ac-8e8b-025ad2c0f66b",
    "Aurore Folny",
    CardRules::new_creature(mana_cost!("{2}{B}"), &["Human", "Warlock"], 0, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Winter enters, you may sacrifice a creature or planeswalker. When you do, \
                 each opponent sacrifices a creature of their choice.",
                EffectDef::PayOr(PayOrDef::optional(
                    &[CostDef::sacrifice_permanent(A_CREATURE_OR_PLANESWALKER)],
                    &EffectDef::ReflexiveTrigger(&AbilityDef::triggered(
                        "When you do, each opponent sacrifices a creature of their choice.",
                        TriggerEventDef::Reflexive,
                        EffectDef::Choose(ChooseDef {
                            binding: ObjectChoiceBindingDef::Object(Binding!("sacrificed")),
                            unchosen: None,
                            chooser: PlayerRefDef::Opponent,
                            candidates: ObjectSetDef::Query(ObjectQueryDef::controlled_by(
                                ObjectPredicateDef::HasType(CardType::Creature),
                                &[ZoneKind::Battlefield],
                                PlayerSetDef::Related(PlayerRelation::Opponent),
                            )),
                            exclude: None,
                            minimum: 1,
                            maximum: 1,
                            visibility: ChoiceVisibilityDef::Public,
                            then: &EffectDef::sacrifice(EffectRecipientDef::object(
                                ObjectRefDef::Binding(Binding!("sacrificed")),
                            )),
                        }),
                    )),
                )),
            ),
            AbilityDef::static_ability(
                "Winter gets +1/+0 for each creature and planeswalker card in your graveyard.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::Source,
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::CountMatchingObjects(&ObjectQueryDef::owned_by(
                            A_CREATURE_OR_PLANESWALKER,
                            &[ZoneKind::Graveyard],
                            PlayerSetDef::Related(PlayerRelation::You),
                        )),
                        ValueDef::Constant(0),
                    ),
                },
            ),
        ]),
);

// FRA 241 — Yargle, Glutton of Urborg (reprint)
const YARGLE_GLUTTON_OF_URBORG_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_dom::YARGLE_GLUTTON_OF_URBORG,
    "04c816fb-5951-4db1-8834-ed3f0b36bfe1",
    "Jehan Choo",
);

// FRA 242 — Ajani Unrelenting
// Audit: unsupported — Needs a trigger event for a player activating a loyalty ability.
pub(in crate::card::sets) static AJANI_UNRELENTING: CardRecord = CardRecord::new(
    "Ajani Unrelenting",
    "bc3c096f-7f6c-474c-a2c8-75d3e6ddd6f5",
    "Tyler Jacobson",
    CardRules::unsupported(),
);

// FRA 243 — Arni, Renowned Champion
pub(in crate::card::sets) static ARNI_RENOWNED_CHAMPION: CardRecord = CardRecord::new(
    "Arni, Renowned Champion",
    "bd8db649-1dba-457d-8327-e1f1da1aab36",
    "Vincent Christiaens",
    CardRules::new_creature(mana_cost!("{3}{R}"), &["Human", "Berserker"], 1, 5)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::trample(),
            AbilityDef::triggered(
                "Whenever another creature you control enters, Arni gets +X/+0 until end of \
                 turn, where X is that creature's power.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Source,
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::TriggeringObjectPower,
                        ValueDef::Constant(0),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ]),
);

// FRA 244 — Chandra, Torch of Defiance (reprint)
const CHANDRA_TORCH_OF_DEFIANCE_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_kld::CHANDRA_TORCH_OF_DEFIANCE,
    "40cb22c8-cb03-45c9-bb0e-b8cabdcc43cd",
    "Kieran Yanner",
);

// FRA 245 — Gallia, the Merrymaker
pub(in crate::card::sets) static GALLIA_THE_MERRYMAKER: CardRecord = CardRecord::new(
    "Gallia, the Merrymaker",
    "f27d50f0-d76e-4ce1-a8d9-d997af6a5b41",
    "Andrea Piparo",
    CardRules::new_creature(mana_cost!("{1}{R}"), &["Satyr"], 2, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::haste(),
            AbilityDef::static_ability(
                "Each other creature you control with a +1/+1 counter on it has haste.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Creature),
                            ObjectPredicateDef::HasCounter(CounterKind::PlusOnePlusOne),
                            ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                        ]),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::add_ability(&abilities::haste()),
                },
            ),
            AbilityDef::activated_with_targets(
                "{1}{R}, {T}: Put a +1/+1 counter on target creature that entered this turn.",
                &[CostDef::Mana(mana_cost!("{1}{R}")), CostDef::TapSource],
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::EnteredThisTurn,
                    ]),
                )],
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
            ),
        ]),
);

// FRA 246 — Jiang Yanggu, Alone
// Audit: unsupported — Needs an attack matcher that counts only the creatures declared against
// the same player. The declaration-size range counts every attacker, so a creature attacking a
// player alone while another attacks that player's planeswalker would not be seen.
pub(in crate::card::sets) static JIANG_YANGGU_ALONE: CardRecord = CardRecord::new(
    "Jiang Yanggu, Alone",
    "e8c1ce21-b77d-40bf-9ed1-478604e71f5f",
    "Dmitry Burmak",
    CardRules::unsupported(),
);

// FRA 247 — Kiora of Fire and Ashes
// A static, so its ability list has one address and the Dragon either ability creates is the
// same token.
static DRAGON_TOKEN: TokenCharacteristics =
    TokenCharacteristics::creature(&["Dragon"], &[ManaColor::Red], 5, 5)
        .with_abilities(&[abilities::flying()])
        .with_art(CardArt::new(
            "991a5840-adc7-45b5-8d5f-b24dae384bab",
            "Loïc Canavaggia",
        ));
pub(in crate::card::sets) static KIORA_OF_FIRE_AND_ASHES: CardRecord = CardRecord::new(
    "Kiora of Fire and Ashes",
    "08657053-86f9-4c52-abf0-d9cdd443ae3b",
    "Chris Rahn",
    CardRules::new_creature(mana_cost!("{4}{R}{R}"), &["Merfolk", "Noble"], 2, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Kiora enters, create a 5/5 red Dragon creature token with flying.",
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(DRAGON_TOKEN))),
            ),
            AbilityDef::activated(
                "{8}: Create a 5/5 red Dragon creature token with flying.",
                &[CostDef::Mana(mana_cost!("{8}"))],
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(DRAGON_TOKEN))),
            ),
        ]),
);

// FRA 248 — Koth, the Geomancer
pub(in crate::card::sets) static KOTH_THE_GEOMANCER: CardRecord = CardRecord::new(
    "Koth, the Geomancer",
    "54f64e95-5a97-4d7c-9939-7f33a3165562",
    "Kieran Yanner",
    CardRules::new_creature(mana_cost!("{2}{R}"), &["Human", "Warrior"], 3, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::reach(),
            AbilityDef::triggered(
                "Landfall — Whenever a land you control enters, Koth deals 1 damage to each \
                 opponent. If that land is a Mountain, add {R}.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Land),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                EffectDef::Sequence(&[
                    EffectDef::damage(EffectRecipientDef::Opponent, ValueDef::Constant(1)),
                    EffectDef::IfCondition {
                        condition: &TriggerConditionDef::ObjectSetCount(
                            &ObjectSetCountConditionDef {
                                objects: &ObjectSetDef::One(ObjectRefDef::TriggeringObject),
                                predicate: ObjectSetPredicateDef::contains(
                                    &ObjectPredicateDef::Subtype(SubtypeDef::from_name("Mountain")),
                                ),
                            },
                        ),
                        then: &EffectDef::AddMana(AddManaEffectDef::one(ManaColor::Red)),
                    },
                ]),
            ),
        ]),
);

// FRA 249 — Marwyn, the Clearcutter
pub(in crate::card::sets) static MARWYN_THE_CLEARCUTTER: CardRecord = CardRecord::new(
    "Marwyn, the Clearcutter",
    "f93da73c-ca8b-438e-8387-6109dac3fc1a",
    "Quintin Gleim",
    CardRules::new_creature(mana_cost!("{R}"), &["Elf", "Warrior"], 2, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[AbilityDef::activated(
            "{2}, {T}, Sacrifice an artifact or land: Draw a card.",
            &[
                CostDef::Mana(mana_cost!("{2}")),
                CostDef::TapSource,
                CostDef::sacrifice_permanent(ObjectPredicateDef::AnyOf(&[
                    ObjectPredicateDef::HasType(CardType::Artifact),
                    ObjectPredicateDef::HasType(CardType::Land),
                ])),
            ],
            abilities::draw_cards(ValueDef::Constant(1)),
        )]),
);

// FRA 250 — Pia, Determined Rebuilder
pub(in crate::card::sets) static PIA_DETERMINED_REBUILDER: CardRecord = CardRecord::new(
    "Pia, Determined Rebuilder",
    "dd3faaf4-45ca-4714-8dbe-37102ec131cf",
    "Andreas Zafiratos",
    CardRules::new_creature(mana_cost!("{2}{R}"), &["Human", "Artificer"], 2, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Pia enters, create a 1/1 colorless Thopter artifact creature token with \
                 flying.",
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(THOPTER_TOKEN))),
            ),
            AbilityDef::activated_with_targets(
                "{5}{R}: Target creature gets +X/+0 until end of turn, where X is the number of \
                 artifacts you control.",
                &[CostDef::Mana(mana_cost!("{5}{R}"))],
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::HasType(CardType::Creature),
                )],
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::CountMatchingObjects(&ObjectQueryDef::matching(
                            ObjectPredicateDef::HasType(CardType::Artifact),
                            &[ZoneKind::Battlefield],
                            PlayerRelation::You,
                        )),
                        ValueDef::Constant(0),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ]),
);

// FRA 251 — Samut, Hazoret's Champion
pub(in crate::card::sets) static SAMUT_HAZORET_S_CHAMPION: CardRecord = CardRecord::new(
    "Samut, Hazoret's Champion",
    "ba920f23-f05c-410e-8516-c93abedf1d4d",
    "Grzegorz Rutkowski",
    CardRules::new_creature(mana_cost!("{1}{R}"), &["Human", "Warrior", "Cleric"], 2, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[AbilityDef::static_ability(
            "Creatures you control have haste.",
            EffectDef::StaticApply {
                recipient: EffectRecipientDef::matching_objects(
                    ObjectPredicateDef::HasType(CardType::Creature),
                    &[ZoneKind::Battlefield],
                    PlayerRelation::You,
                ),
                effect: AppliedEffectDef::add_ability(&abilities::haste()),
            },
        )]),
);

// FRA 252 — Tetsuko Umezawa, Pursuer
// Audit: unsupported — Needs a block event that fires once per blocking creature and is observed
// by another permanent.
pub(in crate::card::sets) static TETSUKO_UMEZAWA_PURSUER: CardRecord = CardRecord::new(
    "Tetsuko Umezawa, Pursuer",
    "df818900-ce5e-4b0d-a927-c975cbef7eda",
    "Tuan Duong Chu",
    CardRules::unsupported(),
);

// FRA 253 — Tomik, Izzet Sparkmage
// Audit: unsupported — Needs a damage event matcher restricted to noncombat damage, and a
// replacement that adds 1 to the matched damage.
pub(in crate::card::sets) static TOMIK_IZZET_SPARKMAGE: CardRecord = CardRecord::new(
    "Tomik, Izzet Sparkmage",
    "5c5afd5f-6f37-4c3e-83f0-68fdcea98810",
    "Randy Vargas",
    CardRules::unsupported(),
);

// FRA 254 — Way of the Pyromancer
pub(in crate::card::sets) static WAY_OF_THE_PYROMANCER: CardRecord = CardRecord::new(
    "Way of the Pyromancer",
    "c1a00020-7c14-4503-a057-5763704bb83e",
    "Andrey Kuzinskiy",
    CardRules::new_enchantment(mana_cost!("{1}{R}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Way of the Pyromancer enters, empower Jace 2. (Put two loyalty counters \
                 on a Jace token you control. If you don't control one, first create a blue \
                 Jace planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
                empower_jace(ValueDef::Constant(2)),
            ),
            planeswalkers_you_control_have(
                "Planeswalkers you control have \"[+1]: Add {R}.\"",
                // Not a mana ability: a loyalty ability never is (CR 605.1a), so this
                // one uses the stack.
                &AbilityDef::activated(
                    "+1: Add {R}.",
                    &[CostDef::Loyalty(ValueDef::Constant(1))],
                    EffectDef::AddMana(AddManaEffectDef::one(ManaColor::Red)),
                ),
            ),
        ]),
);

// FRA 255 — Way of the Warlord
pub(in crate::card::sets) static WAY_OF_THE_WARLORD: CardRecord = CardRecord::new(
    "Way of the Warlord",
    "6d86e410-20c4-4248-96bf-5780ece6274a",
    "Danny Schwartz",
    CardRules::new_enchantment(mana_cost!("{2}{R}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Way of the Warlord enters, empower Jace 5. (Put five loyalty counters on \
                 a Jace token you control. If you don't control one, first create a blue Jace \
                 planeswalker token with \"[−1]: Surveil 1\" and \"[−3]: Draw a card.\")",
                empower_jace(ValueDef::Constant(5)),
            ),
            planeswalkers_you_control_have(
                "Planeswalkers you control have \"[−4]: This planeswalker deals 2 damage to up \
                 to one target creature or planeswalker and 2 damage to target player.\"",
                &AbilityDef::activated_with_targets(
                    "−4: This planeswalker deals 2 damage to up to one target creature or \
                     planeswalker and 2 damage to target player.",
                    &[CostDef::Loyalty(ValueDef::Constant(-4))],
                    &[
                        AbilityTargetDef::up_to(
                            AbilityTargetPredicate::Object {
                                object: A_CREATURE_OR_PLANESWALKER,
                                zones: &[ZoneKind::Battlefield],
                                controller: None,
                                owner: None,
                            },
                            1,
                        ),
                        AbilityTargetDef::exactly_one(AbilityTargetPredicate::Player(
                            PlayerRelation::Any,
                        )),
                    ],
                    EffectDef::damage_simultaneously(&[
                        DamageAssignmentDef::from_effect(
                            EffectRecipientDef::Target(TargetIndex::PRIMARY),
                            ValueDef::Constant(2),
                        ),
                        DamageAssignmentDef::from_effect(
                            EffectRecipientDef::Target(TargetIndex(1)),
                            ValueDef::Constant(2),
                        ),
                    ]),
                ),
            ),
        ]),
);

// FRA 256 — Winter, Team Player
pub(in crate::card::sets) static WINTER_TEAM_PLAYER: CardRecord = CardRecord::new(
    "Winter, Team Player",
    "df8713cd-3f4b-43ef-adbd-e37c2617c617",
    "Aurore Folny",
    CardRules::new_creature(mana_cost!("{4}{R}"), &["Human", "Warrior"], 3, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::convoke(),
            AbilityDef::triggered(
                "Whenever you cast a noncreature spell, creatures you control get +1/+0 until \
                 end of turn.",
                TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                    ObjectPredicateDef::NoncreatureSpell,
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ])),
                EffectDef::Apply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::HasType(CardType::Creature),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::modify_power_toughness(
                        ValueDef::Constant(1),
                        ValueDef::Constant(0),
                    ),
                    duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                },
            ),
        ]),
);

// FRA 257 — Edgar, Moonlit Sovereign
pub(in crate::card::sets) static EDGAR_MOONLIT_SOVEREIGN: CardRecord = CardRecord::new(
    "Edgar, Moonlit Sovereign",
    "dad6afc9-8505-4cdd-bf79-e9ba4670f2bb",
    "Joshua Raphael",
    CardRules::new_creature(mana_cost!("{3}{G}{G}"), &["Werewolf", "Noble"], 4, 4)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::flash(),
            AbilityDef::triggered_if(
                "At the beginning of your end step, if you didn't cast a spell this turn, put \
                 two +1/+1 counters on Edgar.",
                TriggerEventDef::StepBegins {
                    step: TurnStepDef::End,
                    player: PlayerRelation::You,
                },
                &TriggerConditionDef::SpellsCastThisTurn {
                    quantifier: QuantifierDef::Every,
                    player: PlayerRelation::You,
                    comparison: ComparisonDef::Equal,
                    amount: 0,
                },
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Source,
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(2),
                },
            ),
            AbilityDef::activated(
                "{4}{G}: Put a +1/+1 counter on each creature you control with a +1/+1 counter \
                 on it.",
                &[CostDef::Mana(mana_cost!("{4}{G}"))],
                EffectDef::AddCounters {
                    object: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Creature),
                            ObjectPredicateDef::HasCounter(CounterKind::PlusOnePlusOne),
                        ]),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
            ),
        ]),
);

// FRA 258 — Fblthp, Knows the Way
// Audit: unsupported — Needs a library search for up to X cards with different names.
pub(in crate::card::sets) static FBLTHP_KNOWS_THE_WAY: CardRecord = CardRecord::new(
    "Fblthp, Knows the Way",
    "28fbb55a-5c9d-45ee-bf42-a84b1048f5d2",
    "Simon Dominic",
    CardRules::unsupported(),
);

// FRA 259 — Garruk, Curse Breaker
// Audit: unsupported — Needs an attack-batch trigger filtered to player defenders that fires once
// per attack rather than once per attacking creature.
pub(in crate::card::sets) static GARRUK_CURSE_BREAKER: CardRecord = CardRecord::new(
    "Garruk, Curse Breaker",
    "90ca5812-ceb5-46bd-b049-aed7ff10e6af",
    "Victor Adame Minguez",
    CardRules::unsupported(),
);

// FRA 260 — Ghalta the Unstoppable
pub(in crate::card::sets) static GHALTA_THE_UNSTOPPABLE: CardRecord = CardRecord::new(
    "Ghalta the Unstoppable",
    "1d535b5f-c916-4f16-89a7-9477578826d2",
    "Antonio José Manzanedo",
    CardRules::new_creature(mana_cost!("{8}{G}"), &["Elder", "Dinosaur"], 8, 8)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::this_spell_cost_reduction(
                "This spell costs {X} less to cast, where X is the greatest power among \
                 creatures you control.",
                abilities::greatest_power_you_control(),
            ),
            abilities::trample(),
            AbilityDef::static_ability(
                "Other creatures you control have trample.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Creature),
                            ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                        ]),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::add_ability(&abilities::trample()),
                },
            ),
        ]),
);

// FRA 261 — Jiang Yanggu, Never Alone
pub(in crate::card::sets) static JIANG_YANGGU_NEVER_ALONE: CardRecord = CardRecord::new(
    "Jiang Yanggu, Never Alone",
    "f5a0bb3e-8119-4739-8684-e61d1d607dcb",
    "Dmitry Burmak",
    CardRules::new_creature(mana_cost!("{3}{G}"), &["Human", "Druid"], 2, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Jiang Yanggu enters, create Mowu, a legendary 3/3 green Dog creature \
                 token.",
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                    TokenCharacteristics::creature(&["Dog"], &[ManaColor::Green], 3, 3)
                        .with_name("Mowu")
                        .with_supertype(CardSupertype::Legendary)
                        .with_art(CardArt::new(
                            "5868190d-18f6-43b6-af83-9e0ef585687d",
                            "Leonardo Santanna",
                        )),
                ))),
            ),
            AbilityDef::triggered(
                "At the beginning of your end step, untap all tokens you control.",
                TriggerEventDef::StepBegins {
                    step: TurnStepDef::End,
                    player: PlayerRelation::You,
                },
                EffectDef::Untap {
                    object: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::Token,
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                },
            ),
        ]),
);

// FRA 262 — Loot, the Nexus
// Audit: unsupported — Needs a count of distinct powers among creatures you control.
pub(in crate::card::sets) static LOOT_THE_NEXUS: CardRecord = CardRecord::new(
    "Loot, the Nexus",
    "3cfa4fc6-4d90-4576-a83f-6496c7f21104",
    "Filip Burburan",
    CardRules::unsupported(),
);

// FRA 263 — Marwyn, the Preserver
pub(in crate::card::sets) static MARWYN_THE_PRESERVER: CardRecord = CardRecord::new(
    "Marwyn, the Preserver",
    "90f33f99-7bc5-42e1-815e-bfb4c2b74107",
    "Quintin Gleim",
    CardRules::new_creature(mana_cost!("{1}{G}"), &["Elf", "Druid"], 3, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::static_ability(
                "Lands you control have hexproof. (They can't be the targets of spells or \
                 abilities your opponents control.)",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::HasType(CardType::Land),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::add_ability(&abilities::hexproof()),
                },
            ),
            AbilityDef::activated_with_targets(
                "{2}: Return target land card from your graveyard to your hand.",
                &[CostDef::Mana(mana_cost!("{2}"))],
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Land),
                        zones: &[ZoneKind::Graveyard],
                        controller: None,
                        owner: Some(PlayerRelation::You),
                    },
                )],
                EffectDef::move_to_zone(
                    EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    ZoneKind::Hand,
                    ZonePlacement::Top,
                ),
            ),
        ]),
);

// FRA 264 — Pia, Aether Ascetic
pub(in crate::card::sets) static PIA_AETHER_ASCETIC: CardRecord = CardRecord::new(
    "Pia, Aether Ascetic",
    "ff0bc30f-9d20-458e-808f-bdc2825905a5",
    "Andreas Zafiratos",
    CardRules::new_creature(mana_cost!("{2}{G}"), &["Human", "Druid"], 2, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[abilities::enters_trigger(
            "When Pia enters, you may discard a card. If you do, search your library for an \
             enchantment card, reveal it, put it into your hand, then shuffle.",
            EffectDef::PayOr(PayOrDef::optional(
                &[CostDef::discard(ObjectPredicateDef::Any)],
                &EffectDef::SearchZone {
                    exile_face_down: false,
                    player: EffectRecipientDef::Controller,
                    source: ZoneKind::Library,
                    object: ObjectPredicateDef::HasType(CardType::Enchantment),
                    minimum: 0,
                    maximum: ValueDef::Constant(1),
                    reveal: true,
                    destination: ZoneKind::Hand,
                    placement: ZonePlacement::Top,
                    shuffle: true,
                    enters_tapped: false,
                    attachment: None,
                    binding: None,
                    then: None,
                },
            )),
        )]),
);

// FRA 265 — Ruric Thar, Magecrusher
// Audit: unsupported — Needs per-permanent history of having dealt combat damage.
pub(in crate::card::sets) static RURIC_THAR_MAGECRUSHER: CardRecord = CardRecord::new(
    "Ruric Thar, Magecrusher",
    "eed83302-dc2c-45f4-a4bd-af9da51edef5",
    "Olivier Bernard",
    CardRules::unsupported(),
);

// FRA 266 — Titanbones, Towering Heart
// Audit: unsupported — Needs a discard event the discarded card itself hears, wherever the
// discard put it. The per-card discard event never reaches the card it discarded, and watching
// for its arrival in the graveyard from a hand instead would miss a discard that Rest in Peace
// or Leyline of the Void exiles, which is still a discard.
pub(in crate::card::sets) static TITANBONES_TOWERING_HEART: CardRecord = CardRecord::new(
    "Titanbones, Towering Heart",
    "edea6f70-a5a7-475d-b7f2-97933d0f32cf",
    "Jesper Ejsing",
    CardRules::unsupported(),
);

// FRA 267 — Way of the Paradox
// Audit: unsupported — Needs a trigger event for a player activating a loyalty ability, and an
// additional land play lasting one turn.
pub(in crate::card::sets) static WAY_OF_THE_PARADOX: CardRecord = CardRecord::new(
    "Way of the Paradox",
    "98dc5470-507a-4364-8480-42607255e56c",
    "Josh Hass",
    CardRules::unsupported(),
);

// FRA 268 — Way of the Wildspeaker
pub(in crate::card::sets) static WAY_OF_THE_WILDSPEAKER: CardRecord = CardRecord::new(
    "Way of the Wildspeaker",
    "a252cb01-537b-4afe-9abc-81a98c4a1439",
    "Joshua Raphael",
    CardRules::new_enchantment(mana_cost!("{4}{G}"))
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger(
                "When Way of the Wildspeaker enters, empower Jace 7. (Put seven loyalty \
                 counters on a Jace token you control. If you don't control one, first create \
                 a blue Jace planeswalker token with \"[−1]: Surveil 1\" and \
                 \"[−3]: Draw a card.\")",
                empower_jace(ValueDef::Constant(7)),
            ),
            planeswalkers_you_control_have(
                "Planeswalkers you control have \"[−4]: Create a 4/4 green Beast creature \
                 token with trample.\"",
                &AbilityDef::activated(
                    "−4: Create a 4/4 green Beast creature token with trample.",
                    &[CostDef::Loyalty(ValueDef::Constant(-4))],
                    EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(BEAST_TOKEN))),
                ),
            ),
        ]),
);

// FRA 269 — Yoshimaru, Scrappy Stray
pub(in crate::card::sets) static YOSHIMARU_SCRAPPY_STRAY: CardRecord = CardRecord::new(
    "Yoshimaru, Scrappy Stray",
    "b8dfd087-2434-42c6-ac4c-1decbcdde2db",
    "Brian Valeza",
    CardRules::new_creature(mana_cost!("{1}{G}"), &["Dog"], 1, 1)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::enters_trigger_with_targets(
                "When Yoshimaru enters, another target creature you control fights up to one \
                 target creature an opponent controls. (Each deals damage equal to its power to \
                 the other.)",
                &[
                    AbilityTargetDef::exactly_one(AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::HasType(CardType::Creature),
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::You),
                        owner: None,
                    })
                    .excluding_source(),
                    AbilityTargetDef::up_to(
                        AbilityTargetPredicate::Object {
                            object: ObjectPredicateDef::HasType(CardType::Creature),
                            zones: &[ZoneKind::Battlefield],
                            controller: Some(PlayerRelation::Opponent),
                            owner: None,
                        },
                        1,
                    ),
                ],
                EffectDef::Fight {
                    first: ObjectRefDef::Target(TargetIndex::PRIMARY),
                    second: ObjectRefDef::Target(TargetIndex(1)),
                    excess: None,
                },
            ),
            AbilityDef::activated_with_targets(
                "{6}: Put a +1/+1 counter on target nonlegendary creature.",
                &[CostDef::Mana(mana_cost!("{6}"))],
                &[AbilityTargetDef::exactly_one_permanent(
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Supertype(
                            CardSupertype::Legendary,
                        )),
                    ]),
                )],
                EffectDef::AddCounters {
                    object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    kind: CounterKind::PlusOnePlusOne,
                    amount: ValueDef::Constant(1),
                },
            ),
        ]),
);

// FRA 270 — Edgar, Ancient Bloodlord
pub(in crate::card::sets) static EDGAR_ANCIENT_BLOODLORD: CardRecord = CardRecord::new(
    "Edgar, Ancient Bloodlord",
    "7c619fed-2394-4efc-8cdc-6df5f51c1f57",
    "Joshua Raphael",
    CardRules::new_creature(mana_cost!("{W}{B}"), &["Vampire", "Noble"], 2, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::triggered(
                "Whenever another creature or planeswalker you control dies, you gain 1 life.",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::All(&[
                        A_CREATURE_OR_PLANESWALKER,
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                    Some(ZoneKind::Battlefield),
                    Some(ZoneKind::Graveyard),
                ),
                EffectDef::GainLife {
                    recipient: EffectRecipientDef::Controller,
                    amount: ValueDef::Constant(1),
                },
            ),
            AbilityDef::activated(
                "{2}, Sacrifice another creature or planeswalker: Put a +1/+1 counter on Edgar. \
                 He gains menace until end of turn. (He can't be blocked except by two or more \
                 creatures.)",
                &[
                    CostDef::Mana(mana_cost!("{2}")),
                    CostDef::sacrifice_permanent(ObjectPredicateDef::All(&[
                        A_CREATURE_OR_PLANESWALKER,
                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                    ])),
                ],
                EffectDef::Sequence(&[
                    EffectDef::AddCounters {
                        object: EffectRecipientDef::Source,
                        kind: CounterKind::PlusOnePlusOne,
                        amount: ValueDef::Constant(1),
                    },
                    EffectDef::Apply {
                        recipient: EffectRecipientDef::Source,
                        effect: AppliedEffectDef::add_ability(&abilities::menace()),
                        duration: ResolvedEffectDurationDef::UntilEndOfTurn,
                    },
                ]),
            ),
        ]),
);

// FRA 271 — Hapatra, the Desert Fang
pub(in crate::card::sets) static HAPATRA_THE_DESERT_FANG: CardRecord = CardRecord::new(
    "Hapatra, the Desert Fang",
    "cf7c1534-af41-4991-b3c3-f0a34ae330b5",
    "Jodie Muir",
    CardRules::new_creature(mana_cost!("{2}{B}{B}{G}"), &["Human", "Cleric"], 3, 3)
        .with_supertype(CardSupertype::Legendary)
        // One opponent in this engine, so "for each opponent, up to one target creature that
        // player controls" is a single slot over what that opponent controls.
        .with_abilities(&[abilities::enters_trigger_with_targets(
            "When Hapatra enters, for each opponent, put X -1/-1 counters on up to one target \
             creature that player controls, where X is the greatest mana value among cards in \
             your graveyard.",
            &[AbilityTargetDef::up_to(
                AbilityTargetPredicate::Object {
                    object: ObjectPredicateDef::HasType(CardType::Creature),
                    zones: &[ZoneKind::Battlefield],
                    controller: Some(PlayerRelation::Opponent),
                    owner: None,
                },
                1,
            )],
            EffectDef::AddCounters {
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                kind: CounterKind::MinusOneMinusOne,
                amount: ValueDef::AggregateObjectValues(&ObjectValueAggregateDef {
                    objects: ObjectSetDef::Query(ObjectQueryDef::owned_by(
                        ObjectPredicateDef::Any,
                        &[ZoneKind::Graveyard],
                        PlayerSetDef::Related(PlayerRelation::You),
                    )),
                    select: ObjectValueDef::ManaValue,
                    operation: AggregateOperationDef::Maximum,
                }),
            },
        )]),
);

// FRA 272 — Karn, Gilded Guardian
pub(in crate::card::sets) static KARN_GILDED_GUARDIAN: CardRecord = CardRecord::new(
    "Karn, Gilded Guardian",
    "3abcae65-5b21-4c98-adad-34b8bc76ea3a",
    "Scott M. Fischer",
    CardRules::new_artifact_creature(mana_cost!("{2/W}{2/U}{2/B}{2/R}{2/G}"), &["Golem"], 5, 5)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::vigilance(),
            abilities::trample(),
            abilities::enters_trigger(
                "When Karn enters, draw a card for each color among other artifacts you control.",
                // One for each color some other artifact you control is: a color counts
                // once however many artifacts share it, and Karn's own five do not count.
                abilities::draw_cards(ValueDef::Sum(&SumValueDef::new(
                    ValueDef::Sum(&SumValueDef::new(
                        ValueDef::Sum(&SumValueDef::new(
                            ValueDef::Sum(&SumValueDef::new(
                                ValueDef::AnyMatchingObject(&ObjectQueryDef::matching(
                                    ObjectPredicateDef::All(&[
                                        ObjectPredicateDef::HasType(CardType::Artifact),
                                        ObjectPredicateDef::Color(ManaColor::White),
                                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                                    ]),
                                    &[ZoneKind::Battlefield],
                                    PlayerRelation::You,
                                )),
                                ValueDef::AnyMatchingObject(&ObjectQueryDef::matching(
                                    ObjectPredicateDef::All(&[
                                        ObjectPredicateDef::HasType(CardType::Artifact),
                                        ObjectPredicateDef::Color(ManaColor::Blue),
                                        ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                                    ]),
                                    &[ZoneKind::Battlefield],
                                    PlayerRelation::You,
                                )),
                            )),
                            ValueDef::AnyMatchingObject(&ObjectQueryDef::matching(
                                ObjectPredicateDef::All(&[
                                    ObjectPredicateDef::HasType(CardType::Artifact),
                                    ObjectPredicateDef::Color(ManaColor::Black),
                                    ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                                ]),
                                &[ZoneKind::Battlefield],
                                PlayerRelation::You,
                            )),
                        )),
                        ValueDef::AnyMatchingObject(&ObjectQueryDef::matching(
                            ObjectPredicateDef::All(&[
                                ObjectPredicateDef::HasType(CardType::Artifact),
                                ObjectPredicateDef::Color(ManaColor::Red),
                                ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                            ]),
                            &[ZoneKind::Battlefield],
                            PlayerRelation::You,
                        )),
                    )),
                    ValueDef::AnyMatchingObject(&ObjectQueryDef::matching(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Artifact),
                            ObjectPredicateDef::Color(ManaColor::Green),
                            ObjectPredicateDef::Not(&ObjectPredicateDef::Source),
                        ]),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    )),
                ))),
            ),
        ]),
);

// FRA 273 — Kiora of Salt and Sand
// Audit: unsupported — Needs per-player history of loyalty abilities activated this turn, readable
// in an intervening-if condition.
pub(in crate::card::sets) static KIORA_OF_SALT_AND_SAND: CardRecord = CardRecord::new(
    "Kiora of Salt and Sand",
    "8151f5f5-e9f6-4fbe-b543-f456ebf22aa5",
    "Chris Rahn",
    CardRules::unsupported(),
);

// FRA 274 — Mabel, Valley Hero
pub(in crate::card::sets) static MABEL_VALLEY_HERO: CardRecord = CardRecord::new(
    "Mabel, Valley Hero",
    "47abea4b-9848-48aa-bc1b-f04f4799e920",
    "Matt Stewart",
    CardRules::new_creature(mana_cost!("{1}{R}{W}"), &["Mouse", "Soldier"], 1, 2)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[AbilityDef::triggered_with_targets(
            "Whenever Mabel or another creature you control enters, put a +1/+1 counter on \
             target creature that entered this turn.",
            TriggerEventDef::zone_changed(
                ObjectPredicateDef::AnyOf(&[
                    ObjectPredicateDef::Source,
                    ObjectPredicateDef::All(&[
                        ObjectPredicateDef::HasType(CardType::Creature),
                        ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                    ]),
                ]),
                None,
                Some(ZoneKind::Battlefield),
            ),
            &[AbilityTargetDef::exactly_one_permanent(
                ObjectPredicateDef::All(&[
                    ObjectPredicateDef::HasType(CardType::Creature),
                    ObjectPredicateDef::EnteredThisTurn,
                ]),
            )],
            EffectDef::AddCounters {
                object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                kind: CounterKind::PlusOnePlusOne,
                amount: ValueDef::Constant(1),
            },
        )]),
);

// FRA 275 — Saheeli, Jewel of Avishkar
pub(in crate::card::sets) static SAHEELI_JEWEL_OF_AVISHKAR: CardRecord = CardRecord::new(
    "Saheeli, Jewel of Avishkar",
    "28d84ef6-e190-46d4-882d-1cea5e111e2a",
    "Billy Christian",
    CardRules::new_creature(mana_cost!("{2}{U}{R}"), &["Human", "Artificer"], 2, 4)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::static_ability(
                "Thopters you control have haste.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::Subtype(SubtypeDef::from_name("Thopter")),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::add_ability(&abilities::haste()),
                },
            ),
            AbilityDef::triggered(
                "Whenever you cast a noncreature spell, create a 1/1 colorless Thopter artifact \
                 creature token with flying.",
                TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                    ObjectPredicateDef::NoncreatureSpell,
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ])),
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(THOPTER_TOKEN))),
            ),
        ]),
);

// FRA 276 — Tam, the Possibility
// Audit: unsupported — Needs an effect repeated a computed number of times, and a count of
// distinct planeswalker types among planeswalkers you control.
pub(in crate::card::sets) static TAM_THE_POSSIBILITY: CardRecord = CardRecord::new(
    "Tam, the Possibility",
    "6529d399-677e-45a6-ac3e-12a0b10f6c37",
    "Martina Fačková",
    CardRules::unsupported(),
);

// FRA 277 — Vraska, Soul of Stone
pub(in crate::card::sets) static VRASKA_SOUL_OF_STONE: CardRecord = CardRecord::new(
    "Vraska, Soul of Stone",
    "f3869752-eade-4e7a-8dd1-68cafb9e10be",
    "Kieran Yanner",
    CardRules::new_creature(mana_cost!("{U}{R}{W}"), &["Gorgon", "Wizard"], 3, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            AbilityDef::static_ability(
                "Artifact creatures you control have vigilance.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::matching_objects(
                        ObjectPredicateDef::All(&[
                            ObjectPredicateDef::HasType(CardType::Artifact),
                            ObjectPredicateDef::HasType(CardType::Creature),
                        ]),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    effect: AppliedEffectDef::add_ability(&abilities::vigilance()),
                },
            ),
            // Sculpture is the creature type and Treasure the artifact type.
            AbilityDef::triggered(
                "Whenever you cast a noncreature spell, create a 1/1 colorless Sculpture \
                 Treasure artifact creature token with \"{T}, Sacrifice this token: Add one \
                 mana of any color.\"",
                TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                    ObjectPredicateDef::NoncreatureSpell,
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ])),
                EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Literal(
                    TokenCharacteristics::artifact_creature(&["Sculpture", "Treasure"], &[], 1, 1)
                        .with_abilities(&[AbilityDef::activated_mana(
                            "{T}, Sacrifice this token: Add one mana of any color.",
                            &[CostDef::TapSource, CostDef::SacrificeSource],
                            EffectDef::AddMana(AddManaEffectDef::any_color()),
                        )])
                        .with_art(CardArt::new(
                            "a0fec445-c6fb-4ca2-b63f-1d608c418a22",
                            "Raluca Marinescu",
                        )),
                ))),
            ),
        ]),
);

// FRA 278 — Vraska, the Cutting Glare
pub(in crate::card::sets) static VRASKA_THE_CUTTING_GLARE: CardRecord = CardRecord::new(
    "Vraska, the Cutting Glare",
    "5c28b012-5efb-488f-a1c1-09e2dddfd6ee",
    "Kieran Yanner",
    CardRules::new_creature(mana_cost!("{B}{B}{G}"), &["Gorgon", "Assassin"], 4, 4)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::deathtouch(),
            AbilityDef::triggered_if_with_targets(
                "When Vraska enters, if you control six or more lands, destroy target permanent \
                 an opponent controls. They create a Treasure token. (It's an artifact with \
                 \"{T}, Sacrifice this token: Add one mana of any color.\")",
                TriggerEventDef::zone_changed(
                    ObjectPredicateDef::Source,
                    None,
                    Some(ZoneKind::Battlefield),
                ),
                &TriggerConditionDef::ObjectCount {
                    query: ObjectQueryDef::matching(
                        ObjectPredicateDef::HasType(CardType::Land),
                        &[ZoneKind::Battlefield],
                        PlayerRelation::You,
                    ),
                    comparison: ComparisonDef::GreaterOrEqual,
                    amount: 6,
                },
                &[AbilityTargetDef::exactly_one(
                    AbilityTargetPredicate::Object {
                        object: ObjectPredicateDef::Any,
                        zones: &[ZoneKind::Battlefield],
                        controller: Some(PlayerRelation::Opponent),
                        owner: None,
                    },
                )],
                EffectDef::Sequence(&[
                    EffectDef::Destroy {
                        object: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                        then: None,
                    },
                    EffectDef::CreateToken(
                        CreateTokenDef::new(TokenDef::Literal(TREASURE_TOKEN)).with_controller(
                            PlayerRefDef::ControllerOf(ObjectRefDef::Target(TargetIndex::PRIMARY)),
                        ),
                    ),
                ]),
            ),
        ]),
);

// FRA 279 — Karn, Argent Defender
pub(in crate::card::sets) static KARN_ARGENT_DEFENDER: CardRecord = CardRecord::new(
    "Karn, Argent Defender",
    "1ebbbddb-2dc3-4194-b72b-13bcebe2ab89",
    "Scott M. Fischer",
    CardRules::new_artifact_creature(mana_cost!("{2}"), &["Golem"], 1, 3)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[AbilityDef::static_ability(
            "Artifacts and creatures entering the battlefield don't cause abilities to trigger.",
            EffectDef::StaticApply {
                recipient: EffectRecipientDef::players(PlayerSetDef::All),
                effect: AppliedEffectDef::Rule(AppliedRuleDef::ModifyTriggers(
                    &TriggerModificationDef {
                        cause: TriggerEventDef::zone_changed(
                            ObjectPredicateDef::AnyOf(&[
                                ObjectPredicateDef::HasType(CardType::Artifact),
                                ObjectPredicateDef::HasType(CardType::Creature),
                            ]),
                            None,
                            Some(ZoneKind::Battlefield),
                        ),
                        permanent: None,
                        kind: TriggerModificationKindDef::Suppress,
                    },
                )),
            },
        )]),
);

// FRA 280 — Traxos, Scourge Eternal
pub(in crate::card::sets) static TRAXOS_SCOURGE_ETERNAL: CardRecord = CardRecord::new(
    "Traxos, Scourge Eternal",
    "05102c46-96f8-44a0-a1e6-e388fa5e0841",
    "Aaron Miller",
    CardRules::new_artifact_creature(mana_cost!("{4}"), &["Dragon", "Construct"], 5, 4)
        .with_supertype(CardSupertype::Legendary)
        .with_abilities(&[
            abilities::trample(),
            AbilityDef::static_ability(
                "Traxos doesn't untap during your untap step.",
                EffectDef::StaticApply {
                    recipient: EffectRecipientDef::Source,
                    effect: AppliedEffectDef::Rule(AppliedRuleDef::DoesNotUntapDuringUntapStep),
                },
            ),
            AbilityDef::triggered(
                "Whenever you cast an artifact or creature spell, untap Traxos.",
                TriggerEventDef::spell_cast(ObjectPredicateDef::All(&[
                    ObjectPredicateDef::AnyOf(&[
                        ObjectPredicateDef::HasType(CardType::Artifact),
                        ObjectPredicateDef::HasType(CardType::Creature),
                    ]),
                    ObjectPredicateDef::ControlledBy(PlayerRelation::You),
                ])),
                EffectDef::Untap {
                    object: EffectRecipientDef::Source,
                },
            ),
        ]),
);

// FRA 281 — Plains (reprint)
const PLAINS_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_lea::PLAINS,
    "a4eaecb8-066a-4c72-aa5d-5ed0614ba537",
    "Luc Courtois",
);

// FRA 282 — Plains (alternate printing)
const PLAINS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::PLAINS,
    1,
    "d5955c78-83ec-43d1-b7d6-214ab5d32ff2",
    "Florian Herold",
);

// FRA 283 — Island (reprint)
const ISLAND_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_lea::ISLAND,
    "d8184e92-54e6-4ddc-86e8-67f5c5eb079d",
    "Chris Ostrowski",
);

// FRA 284 — Island (alternate printing)
const ISLAND_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::ISLAND,
    1,
    "1c1b233f-ade1-40c8-a91f-e9d8cc63179b",
    "Zack Stella",
);

// FRA 285 — Swamp (reprint)
const SWAMP_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_lea::SWAMP,
    "30693b85-550d-4c98-8c5b-4fd4e91c9f28",
    "Josu Solano",
);

// FRA 286 — Swamp (alternate printing)
const SWAMP_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::SWAMP,
    1,
    "05465ed0-f511-4e6f-9561-49bf6608fb80",
    "Leon Tukker",
);

// FRA 287 — Mountain (reprint)
const MOUNTAIN_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_lea::MOUNTAIN,
    "16671d98-6f00-477b-a010-d2905c94eb65",
    "Jorge Jacinto",
);

// FRA 288 — Mountain (alternate printing)
const MOUNTAIN_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::MOUNTAIN,
    1,
    "ed8ebf4a-2879-4da2-93d8-b63d782a7786",
    "Chris Ostrowski",
);

// FRA 289 — Forest (reprint)
const FOREST_REPRINT: PrintingRecord = PrintingRecord::reprint(
    &catalog_lea::FOREST,
    "613bc075-2fe4-421f-bfec-3786a5e37797",
    "Randy Gallegos",
);

// FRA 290 — Forest (alternate printing)
const FOREST_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::FOREST,
    1,
    "25e36841-3242-436b-8e82-9099063fa461",
    "Chris Ostrowski",
);

// FRA 291 — Ajani Resolute (alternate printing)
const AJANI_RESOLUTE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &AJANI_RESOLUTE,
    1,
    "96aec358-5f18-44c7-8223-a41c9d583961",
    "Aleksi Briclot",
);

// FRA 292 — Gideon's Memorial (alternate printing)
const GIDEON_S_MEMORIAL_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &GIDEON_S_MEMORIAL,
    1,
    "b658f046-bacd-40f5-85ba-af2ab8071b71",
    "Lius Lasahido",
);

// FRA 293 — Liliana the Faultless (alternate printing)
const LILIANA_THE_FAULTLESS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &LILIANA_THE_FAULTLESS,
    1,
    "9a8ee54c-7b03-42d5-9143-f40c1016a67a",
    "Aleksi Briclot",
);

// FRA 294 — Lyra, Archangel of Dawn (alternate printing)
const LYRA_ARCHANGEL_OF_DAWN_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &LYRA_ARCHANGEL_OF_DAWN,
    1,
    "fc4635a1-1497-4643-b3e5-954808b07a54",
    "Aaron Miller",
);

// FRA 295 — Teyo, Lightshield Expert (alternate printing)
const TEYO_LIGHTSHIELD_EXPERT_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &TEYO_LIGHTSHIELD_EXPERT,
    1,
    "a2249911-fe70-4ad2-bb22-748ce4ac1998",
    "Fajareka Setiawan",
);

// FRA 296 — Thalia, the Survivor (alternate printing)
const THALIA_THE_SURVIVOR_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &THALIA_THE_SURVIVOR,
    1,
    "ce080177-aff6-41f2-b76b-c735701e6e98",
    "Luisa J. Preissler",
);

// FRA 297 — Chandra, Chill of Compliance (alternate printing)
const CHANDRA_CHILL_OF_COMPLIANCE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &CHANDRA_CHILL_OF_COMPLIANCE,
    1,
    "549cd466-d726-4ac9-a9e8-c1cd3ecda998",
    "Aleksi Briclot",
);

// FRA 298 — Fblthp, Impossibly Lost (alternate printing)
const FBLTHP_IMPOSSIBLY_LOST_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &FBLTHP_IMPOSSIBLY_LOST,
    1,
    "15139452-04fc-4870-8306-c9a6defba76d",
    "Tomek Larek",
);

// FRA 299 — Geist of Saint Thalia (alternate printing)
const GEIST_OF_SAINT_THALIA_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &GEIST_OF_SAINT_THALIA,
    1,
    "85082e55-cb7e-4fe2-86d5-f8accd48ba2c",
    "Luisa J. Preissler",
);

// FRA 300 — Jace, Reality Sculptor (alternate printing)
const JACE_REALITY_SCULPTOR_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &JACE_REALITY_SCULPTOR,
    1,
    "dafe1979-fda7-4164-a362-b9f789f25710",
    "Aleksi Briclot",
);

// FRA 301 — Lyra, Tolarian Archangel (alternate printing)
const LYRA_TOLARIAN_ARCHANGEL_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &LYRA_TOLARIAN_ARCHANGEL,
    1,
    "06796f03-18d4-4286-ab77-3cd9aa517090",
    "Aaron Miller",
);

// FRA 302 — Samut, Tyrant of Naktamun (alternate printing)
const SAMUT_TYRANT_OF_NAKTAMUN_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SAMUT_TYRANT_OF_NAKTAMUN,
    1,
    "7081f960-3476-4acd-8272-600c1e50b560",
    "Nicholas Elias",
);

// FRA 303 — Garruk, Veiled Butcher (alternate printing)
const GARRUK_VEILED_BUTCHER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &GARRUK_VEILED_BUTCHER,
    1,
    "9efdedea-c9a4-4085-9da9-4293200b99e0",
    "Aleksi Briclot",
);

// FRA 304 — Gideon the Oathless (alternate printing)
const GIDEON_THE_OATHLESS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &GIDEON_THE_OATHLESS,
    1,
    "aace6d83-d30f-4481-9c6f-c5b3e3794dea",
    "Lius Lasahido",
);

// FRA 305 — Liliana the Repentant (alternate printing)
const LILIANA_THE_REPENTANT_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &LILIANA_THE_REPENTANT,
    1,
    "ddefddbc-c769-4e53-a9b3-087578ae8bc0",
    "Aleksi Briclot",
);

// FRA 306 — Teyo, Diamondblade Mage (alternate printing)
const TEYO_DIAMONDBLADE_MAGE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &TEYO_DIAMONDBLADE_MAGE,
    1,
    "05cb9804-b2db-4de4-8897-7881536630f4",
    "Fajareka Setiawan",
);

// FRA 307 — Winter, Tormented Loner (alternate printing)
const WINTER_TORMENTED_LONER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &WINTER_TORMENTED_LONER,
    1,
    "926fadc8-2478-43ba-8e9a-ff608d651056",
    "Miranda Meeks",
);

// FRA 308 — Ajani Unrelenting (alternate printing)
const AJANI_UNRELENTING_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &AJANI_UNRELENTING,
    1,
    "2b293d8f-7877-4212-af6b-e898d759d013",
    "Aleksi Briclot",
);

// FRA 309 — Chandra, Torch of Defiance (alternate printing)
const CHANDRA_TORCH_OF_DEFIANCE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_kld::CHANDRA_TORCH_OF_DEFIANCE,
    1,
    "1913788b-36d3-4488-a744-71e52b1ecb5c",
    "Aleksi Briclot",
);

// FRA 310 — Marwyn, the Clearcutter (alternate printing)
const MARWYN_THE_CLEARCUTTER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &MARWYN_THE_CLEARCUTTER,
    1,
    "bc19a007-9de6-4e45-b458-cf7ad621e31f",
    "Jessica Liu",
);

// FRA 311 — Samut, Hazoret's Champion (alternate printing)
const SAMUT_HAZORET_S_CHAMPION_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SAMUT_HAZORET_S_CHAMPION,
    1,
    "f909fe3b-c349-4a19-90ad-aa51f8e73b66",
    "Nicholas Elias",
);

// FRA 312 — Winter, Team Player (alternate printing)
const WINTER_TEAM_PLAYER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &WINTER_TEAM_PLAYER,
    1,
    "e8d2c13b-d3b5-4f4c-825d-857710c4db76",
    "Miranda Meeks",
);

// FRA 313 — Fblthp, Knows the Way (alternate printing)
const FBLTHP_KNOWS_THE_WAY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &FBLTHP_KNOWS_THE_WAY,
    1,
    "0a41d10e-7dad-46fd-bd15-0ea6c89bc0aa",
    "Tomek Larek",
);

// FRA 314 — Garruk, Curse Breaker (alternate printing)
const GARRUK_CURSE_BREAKER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &GARRUK_CURSE_BREAKER,
    1,
    "90c84a1c-ee2b-4b63-8d81-d4e940a16162",
    "Aleksi Briclot",
);

// FRA 315 — Marwyn, the Preserver (alternate printing)
const MARWYN_THE_PRESERVER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &MARWYN_THE_PRESERVER,
    1,
    "447c324a-1e66-44fc-9d6e-5e1bbbd7cc03",
    "Jessica Liu",
);

// FRA 316 — Karn, Gilded Guardian (alternate printing)
const KARN_GILDED_GUARDIAN_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &KARN_GILDED_GUARDIAN,
    1,
    "3c3863a1-7c9f-4884-8915-84d088ce231c",
    "Billy Christian",
);

// FRA 317 — Tam, the Possibility (alternate printing)
const TAM_THE_POSSIBILITY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &TAM_THE_POSSIBILITY,
    1,
    "d30c5406-c30f-4c7b-8967-9eff3540f4de",
    "Aleksi Briclot",
);

// FRA 318 — Vraska, Soul of Stone (alternate printing)
const VRASKA_SOUL_OF_STONE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &VRASKA_SOUL_OF_STONE,
    1,
    "c4f56034-fab2-4f23-923d-cb5c395a0a87",
    "Syd Mills",
);

// FRA 319 — Vraska, the Cutting Glare (alternate printing)
const VRASKA_THE_CUTTING_GLARE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &VRASKA_THE_CUTTING_GLARE,
    1,
    "9e80737c-c228-4a69-acd8-c8bb2f6d8bd0",
    "Syd Mills",
);

// FRA 320 — Karn, Argent Defender (alternate printing)
const KARN_ARGENT_DEFENDER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &KARN_ARGENT_DEFENDER,
    1,
    "7deb091c-6fa8-44b5-a580-c0289112b6cb",
    "Billy Christian",
);

// FRA 321 — Enlightened Confidant (alternate printing)
const ENLIGHTENED_CONFIDANT_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &ENLIGHTENED_CONFIDANT,
    1,
    "64126446-8c25-4be9-8559-31f407d41578",
    "Jabari Weathers",
);

// FRA 322 — Kindred Judgment (alternate printing)
const KINDRED_JUDGMENT_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &KINDRED_JUDGMENT,
    1,
    "8cbcd273-a7eb-40b1-a7b6-7aafcb15c141",
    "Joshua Cairos",
);

// FRA 323 — Return to the Light Realms (alternate printing)
const RETURN_TO_THE_LIGHT_REALMS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &RETURN_TO_THE_LIGHT_REALMS,
    1,
    "c0d25cd5-1977-4303-b83c-0dd415c13077",
    "Magali Villeneuve",
);

// FRA 324 — Seasoned Cryomancer (alternate printing)
const SEASONED_CRYOMANCER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SEASONED_CRYOMANCER,
    1,
    "1b36b3d1-2e58-4036-9d91-5875ad667346",
    "Chris Rallis",
);

// FRA 325 — Darklight Phoenix (alternate printing)
const DARKLIGHT_PHOENIX_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &DARKLIGHT_PHOENIX,
    1,
    "a1009710-f953-46c9-88fe-d14e9ff1f4af",
    "Gonzalo Kenny",
);

// FRA 326 — Rise of the Deathbringer (alternate printing)
const RISE_OF_THE_DEATHBRINGER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &RISE_OF_THE_DEATHBRINGER,
    1,
    "bc6be107-b3e3-4f71-880c-79755fafe98d",
    "Kieran Yanner",
);

// FRA 327 — Craterclaw Colossus (alternate printing)
const CRATERCLAW_COLOSSUS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &CRATERCLAW_COLOSSUS,
    1,
    "8733a9a4-c184-46d9-afca-bf9048e2f25c",
    "Raluca Marinescu",
);

// FRA 328 — Curse-Marred Demon (alternate printing)
const CURSE_MARRED_DEMON_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &CURSE_MARRED_DEMON,
    1,
    "68732a25-9058-4df3-ac41-3cd40845a5cb",
    "Thomas M. Baxa",
);

// FRA 329 — Stingcaster Mage (alternate printing)
const STINGCASTER_MAGE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &STINGCASTER_MAGE,
    1,
    "086ef095-32ad-4311-a40b-996c5f7b86cf",
    "Gal Or",
);

// FRA 330 — Hexhaven Invigorator (alternate printing)
const HEXHAVEN_INVIGORATOR_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &HEXHAVEN_INVIGORATOR,
    1,
    "332030be-1a25-4f62-8a62-fa10b182a209",
    "David Auden Nash",
);

// FRA 331 — Omnipresence (alternate printing)
const OMNIPRESENCE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &OMNIPRESENCE,
    1,
    "87d2bb1a-7dbf-41bc-bbfd-20e25fe19f52",
    "Chase Stone",
);

// FRA 332 — Simulacrum Shaper (alternate printing)
const SIMULACRUM_SHAPER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SIMULACRUM_SHAPER,
    1,
    "071ed239-6875-4090-94b3-56b246a9407b",
    "Gustavo Pelissari",
);

// FRA 333 — Frostbite Pyromental (alternate printing)
const FROSTBITE_PYROMENTAL_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &FROSTBITE_PYROMENTAL,
    1,
    "79ff0b78-c220-4940-8957-77cf4fd820e2",
    "Paul Adam",
);

// FRA 334 — Germinate Recruits (alternate printing)
const GERMINATE_RECRUITS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &GERMINATE_RECRUITS,
    1,
    "dcb1b131-fee7-455f-a6e7-b94471288ed7",
    "Camille Alquier",
);

// FRA 335 — Repurposed Enforcer (alternate printing)
const REPURPOSED_ENFORCER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &REPURPOSED_ENFORCER,
    1,
    "ac0bd1be-36cb-483f-9c47-2714e534cb6c",
    "Vlad Petruchik",
);

// FRA 336 — Cruel Calculations (alternate printing)
const CRUEL_CALCULATIONS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &CRUEL_CALCULATIONS,
    1,
    "8bdd99c6-d0f1-4922-ada8-0c4566860e15",
    "Francis Tneh",
);

// FRA 337 — Jace's Machinations (alternate printing)
const JACE_S_MACHINATIONS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &JACE_S_MACHINATIONS,
    1,
    "aabdc0a6-5d60-44c4-96cb-692eeeac0630",
    "Rimas Valeikis",
);

// FRA 338 — Sphinx of False Conclusions (alternate printing)
const SPHINX_OF_FALSE_CONCLUSIONS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SPHINX_OF_FALSE_CONCLUSIONS,
    1,
    "a62e9d0c-a802-4a14-9627-d784d6694375",
    "Arif Wijaya",
);

// FRA 339 — Theorist's Proxy (alternate printing)
const THEORIST_S_PROXY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &THEORIST_S_PROXY,
    1,
    "a46d570d-2abb-4270-87bc-b4b3baf5ed3b",
    "Camille Alquier",
);

// FRA 340 — Dark Matter Manipulator (alternate printing)
const DARK_MATTER_MANIPULATOR_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &DARK_MATTER_MANIPULATOR,
    1,
    "5fd4f195-ee14-4aff-91b0-7ed5b552fd35",
    "Kevin Glint",
);

// FRA 341 — Overwrite the Multiverse (alternate printing)
const OVERWRITE_THE_MULTIVERSE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &OVERWRITE_THE_MULTIVERSE,
    1,
    "c318cd29-89db-4643-b3ef-5ff8492f5c1a",
    "Bengt Hampus Viklander",
);

// FRA 342 — Sanctum Lurker (alternate printing)
const SANCTUM_LURKER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SANCTUM_LURKER,
    1,
    "02b1bf77-f78b-4a48-8ff8-a0d3b0aeafaa",
    "Michael MacRae",
);

// FRA 343 — Vraska's Final Mercy (alternate printing)
const VRASKA_S_FINAL_MERCY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &VRASKA_S_FINAL_MERCY,
    1,
    "c9535252-1a05-4d3a-98e1-54bfd7832ce4",
    "Darius Zablockis",
);

// FRA 344 — Master of Barbs (alternate printing)
const MASTER_OF_BARBS_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &MASTER_OF_BARBS,
    1,
    "32dcc615-910c-413b-8774-d4db32e80e19",
    "Guglielmo Bonis",
);

// FRA 345 — Gardenize (alternate printing)
const GARDENIZE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &GARDENIZE,
    1,
    "c60d4862-9615-4522-8724-489065d6ef6b",
    "Jurijus Chitrovas",
);

// FRA 346 — Hungering Puppetbeast (alternate printing)
const HUNGERING_PUPPETBEAST_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &HUNGERING_PUPPETBEAST,
    1,
    "3cee6c29-e9ab-4e86-8fc9-9b9c287ffdfd",
    "Kevin Glint",
);

// FRA 347 — Aerid Konstrari (alternate printing)
const AERID_KONSTRARI_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &AERID_KONSTRARI,
    1,
    "66dcd789-d405-4fa6-855f-61064e2e4635",
    "Julia Vasilyeva",
);

// FRA 348 — Avatar of Burgeoning Echoes (alternate printing)
const AVATAR_OF_BURGEONING_ECHOES_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &AVATAR_OF_BURGEONING_ECHOES,
    1,
    "58910deb-a99f-43c6-acc8-46563682b486",
    "Francis Tneh",
);

// FRA 349 — Denzilore Fatehold (alternate printing)
const DENZILORE_FATEHOLD_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &DENZILORE_FATEHOLD,
    1,
    "75a3e059-d681-42f3-8a90-879c84323fe2",
    "Gintas Galvanauskas",
);

// FRA 350 — Entrust the Spark (alternate printing)
const ENTRUST_THE_SPARK_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &ENTRUST_THE_SPARK,
    1,
    "cec7b201-e3c4-4bde-bfc1-0049be22b890",
    "Mintautas Šukys",
);

// FRA 351 — Ingris Stingerquill (alternate printing)
const INGRIS_STINGERQUILL_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &INGRIS_STINGERQUILL,
    1,
    "164a17bc-789b-437f-9798-1854b5cd3dda",
    "Ovidio Cartagena",
);

// FRA 352 — Kwia Vigorbloom (alternate printing)
const KWIA_VIGORBLOOM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &KWIA_VIGORBLOOM,
    1,
    "4d5c27d8-b8c0-4a67-ad6e-3e21ace14e52",
    "Eglė Mosakaitė",
);

// FRA 353 — Null Summoner (alternate printing)
const NULL_SUMMONER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &NULL_SUMMONER,
    1,
    "fc488525-de0b-4a52-8a1a-6ee4e798474e",
    "Kevin Glint",
);

// FRA 354 — Proctor of Potential (alternate printing)
const PROCTOR_OF_POTENTIAL_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &PROCTOR_OF_POTENTIAL,
    1,
    "4589d911-1740-470d-9da5-7fbab448e184",
    "Arif Wijaya",
);

// FRA 355 — Solitary Cell (alternate printing)
const SOLITARY_CELL_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SOLITARY_CELL,
    1,
    "b7e8929c-416e-4649-9147-ec88001c6c1c",
    "Arif Wijaya",
);

// FRA 356 — Stinging Vitriol (alternate printing)
const STINGING_VITRIOL_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &STINGING_VITRIOL,
    1,
    "71fd89cc-bf57-4b6d-a788-b4c91b3db044",
    "Kevin Glint",
);

// FRA 357 — Tenured Tethermage (alternate printing)
const TENURED_TETHERMAGE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &TENURED_TETHERMAGE,
    1,
    "6ae471aa-74c3-416b-a884-bcf1bb2f6004",
    "Viktor Fetsch",
);

// FRA 358 — Uldaros Theorix (alternate printing)
const ULDAROS_THEORIX_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &ULDAROS_THEORIX,
    1,
    "001d33b2-5e6f-4233-bd03-be85642e76e4",
    "Eugene Maslovski",
);

// FRA 359 — Flickering Hound (alternate printing)
const FLICKERING_HOUND_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &FLICKERING_HOUND,
    1,
    "4ac7452d-92b0-4532-96ed-a510a354b31f",
    "Justin Hernandez & Alexis Hernandez",
);

// FRA 360 — Guiding Hydra (alternate printing)
const GUIDING_HYDRA_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &GUIDING_HYDRA,
    1,
    "cdf66481-bbc2-4fba-a88c-c19122bbb31a",
    "FeelFish",
);

// FRA 361 — Refute Destiny (alternate printing)
const REFUTE_DESTINY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &REFUTE_DESTINY,
    1,
    "839cac1b-c378-47dc-b005-bc412c4df718",
    "Wojtek Łebski",
);

// FRA 362 — Precise Redaction (alternate printing)
const PRECISE_REDACTION_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &PRECISE_REDACTION,
    1,
    "4b15b2c1-c3fd-474e-872f-464083a0e0bf",
    "Florencia Degraf",
);

// FRA 363 — The Theorist, Jace Beleren (alternate printing)
const THE_THEORIST_JACE_BELEREN_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &THE_THEORIST_JACE_BELEREN,
    1,
    "707f6fbb-94f6-459b-8a21-bec360b7b668",
    "Miles Johnston",
);

// FRA 364 — Extrapolate the Impossible (alternate printing)
const EXTRAPOLATE_THE_IMPOSSIBLE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &EXTRAPOLATE_THE_IMPOSSIBLE,
    1,
    "a20f408a-87b2-4ee3-a1b1-520b38f54d9e",
    "Sam Chivers",
);

// FRA 365 — Lich's Relic (alternate printing)
const LICH_S_RELIC_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &LICH_S_RELIC,
    1,
    "b66b0007-e973-4854-ac84-b1017b14186c",
    "Elena Masci",
);

// FRA 366 — Terminal Criticism (alternate printing)
const TERMINAL_CRITICISM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &TERMINAL_CRITICISM,
    1,
    "01eb1fc4-14bc-479e-91fb-83eae7e3d203",
    "Justin Hernandez & Alexis Hernandez",
);

// FRA 367 — Ajani's Anguish (alternate printing)
const AJANI_S_ANGUISH_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &AJANI_S_ANGUISH,
    1,
    "e0bad302-16af-48c9-a33b-2def7daf527d",
    "Molly Mendoza",
);

// FRA 368 — Draconic Visitor (alternate printing)
const DRACONIC_VISITOR_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &DRACONIC_VISITOR,
    1,
    "d061676d-093a-47ac-9064-3edcacbc4485",
    "Zoe Van Dijk",
);

// FRA 369 — Essence Burn (alternate printing)
const ESSENCE_BURN_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &ESSENCE_BURN,
    1,
    "3802a45a-0a6f-4a1b-85b5-276bbfd67d75",
    "Jeff Soto",
);

// FRA 370 — Face Yourself (alternate printing)
const FACE_YOURSELF_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &FACE_YOURSELF,
    1,
    "68930b71-d96e-4e08-9547-7f8160d7ab73",
    "Jack Hughes",
);

// FRA 371 — Identity Echo (alternate printing)
const IDENTITY_ECHO_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &IDENTITY_ECHO,
    1,
    "3ab341a1-525d-4493-b4d1-a4c025649205",
    "Maria Jesus Contreras",
);

// FRA 372 — Flourishing Grapple (alternate printing)
const FLOURISHING_GRAPPLE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &FLOURISHING_GRAPPLE,
    1,
    "afac18b1-08be-425c-a3ea-0556dc02a38c",
    "Gage Lindsten",
);

// FRA 373 — Puppet Crafting (alternate printing)
const PUPPET_CRAFTING_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &PUPPET_CRAFTING,
    1,
    "15126d14-3737-4377-8476-e5f05a54fc77",
    "Galen Dara",
);

// FRA 374 — Tarmogoyf (alternate printing)
const TARMOGOYF_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_fut::TARMOGOYF,
    1,
    "1b09f45d-5224-4b2b-83ce-4f1de6357fd8",
    "Micha Huigen",
);

// FRA 375 — Verdant Kraken (alternate printing)
const VERDANT_KRAKEN_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &VERDANT_KRAKEN,
    1,
    "decca9e3-5ca8-4b48-8ec9-23b26ca683cb",
    "Michael Koehler",
);

// FRA 376 — Solarium Sentry (alternate printing)
const SOLARIUM_SENTRY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SOLARIUM_SENTRY,
    1,
    "860263c1-f8d9-4557-a068-6e17ef555f0c",
    "Micha Huigen",
);

// FRA 377 — Vindictive Triumph (alternate printing)
const VINDICTIVE_TRIUMPH_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &VINDICTIVE_TRIUMPH,
    1,
    "70cbf43a-d6d0-4777-807d-537257879ca5",
    "Elena Masci",
);

// FRA 378 — The Echoverse Fulcrum (alternate printing)
const THE_ECHOVERSE_FULCRUM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &THE_ECHOVERSE_FULCRUM,
    1,
    "1b54a4c5-511f-439f-86d0-e84a30275462",
    "Jack Hughes",
);

// FRA 379 — Hall of Echoes (alternate printing)
const HALL_OF_ECHOES_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &HALL_OF_ECHOES,
    1,
    "fa63b3eb-292d-4d9d-9179-085566e82d37",
    "Serena Malyon",
);

// FRA 380 — Roiling Canopy (alternate printing)
const ROILING_CANOPY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &ROILING_CANOPY,
    1,
    "0e76872b-d271-4032-aca2-dde428712b3a",
    "FeelFish",
);

// FRA 381 — Theorist's Sanctum (alternate printing)
const THEORIST_S_SANCTUM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &THEORIST_S_SANCTUM,
    1,
    "e8f607b3-f37c-49e7-8eb8-a14b2153f30e",
    "Jaime A. Zuverza",
);

// FRA 382 — Plains (alternate printing)
const PLAINS_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::PLAINS,
    2,
    "796b72af-5078-427d-b566-9bccad17090f",
    "Alayna Danner",
);

// FRA 383 — Plains (alternate printing)
const PLAINS_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::PLAINS,
    3,
    "57999b70-74c6-415a-aef8-3afc546e7557",
    "Alayna Danner",
);

// FRA 384 — Plains (alternate printing)
const PLAINS_ALTERNATE_4: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::PLAINS,
    4,
    "48964496-cbbb-4646-a257-153497bbc83d",
    "Alayna Danner",
);

// FRA 385 — Island (alternate printing)
const ISLAND_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::ISLAND,
    2,
    "bc4cd881-9c34-4cf9-8afe-050e803f34a6",
    "Calder Moore",
);

// FRA 386 — Island (alternate printing)
const ISLAND_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::ISLAND,
    3,
    "6ff3f00e-4ec2-41ef-bd19-fa830480245d",
    "Calder Moore",
);

// FRA 387 — Island (alternate printing)
const ISLAND_ALTERNATE_4: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::ISLAND,
    4,
    "b256a981-ba4b-43b9-83a3-f31a105a2c26",
    "Calder Moore",
);

// FRA 388 — Swamp (alternate printing)
const SWAMP_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::SWAMP,
    2,
    "e8418418-10bb-48e1-83ed-0081390384b4",
    "Daniel Romanovsky",
);

// FRA 389 — Swamp (alternate printing)
const SWAMP_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::SWAMP,
    3,
    "140ba675-3922-48cf-b6ec-8639267dcab8",
    "Daniel Romanovsky",
);

// FRA 390 — Swamp (alternate printing)
const SWAMP_ALTERNATE_4: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::SWAMP,
    4,
    "f080e0af-cd86-488b-968e-e69b0fa0fe93",
    "Daniel Romanovsky",
);

// FRA 391 — Mountain (alternate printing)
const MOUNTAIN_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::MOUNTAIN,
    2,
    "ec2ceca8-2d90-4d8f-ac47-cd8134e8d859",
    "Leon Tukker",
);

// FRA 392 — Mountain (alternate printing)
const MOUNTAIN_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::MOUNTAIN,
    3,
    "ddc91faf-dd36-4c26-b3b4-0a6a624b5663",
    "Leon Tukker",
);

// FRA 393 — Mountain (alternate printing)
const MOUNTAIN_ALTERNATE_4: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::MOUNTAIN,
    4,
    "a49ae3b7-4b1f-41b6-8c05-cbdc44f070cb",
    "Leon Tukker",
);

// FRA 394 — Forest (alternate printing)
const FOREST_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::FOREST,
    2,
    "790dcf48-ab51-4b10-9933-6b7e3baf0f52",
    "Adam Paquette",
);

// FRA 395 — Forest (alternate printing)
const FOREST_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::FOREST,
    3,
    "1c2ac08e-3810-46f4-94ab-8b32986f9185",
    "Adam Paquette",
);

// FRA 396 — Forest (alternate printing)
const FOREST_ALTERNATE_4: PrintingRecord = PrintingRecord::alternate(
    &catalog_lea::FOREST,
    4,
    "a0d12fc5-c7bd-4734-9259-98fe65241f60",
    "Adam Paquette",
);

// FRA 397 — Deserted Beach (alternate printing)
const DESERTED_BEACH_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_mid::DESERTED_BEACH,
    1,
    "01bdab5b-c1a6-4459-8e93-37602c8548e2",
    "Piotr Dura",
);

// FRA 398 — Haunted Ridge (alternate printing)
const HAUNTED_RIDGE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_mid::HAUNTED_RIDGE,
    1,
    "9c94c165-ce61-4c2b-8c08-7df71049a8d9",
    "Piotr Dura",
);

// FRA 399 — Overgrown Farmland (alternate printing)
const OVERGROWN_FARMLAND_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_mid::OVERGROWN_FARMLAND,
    1,
    "047b4e1e-d9cb-4e6b-a1e4-bc5366ee0a23",
    "Piotr Dura",
);

// FRA 400 — Rockfall Vale (alternate printing)
const ROCKFALL_VALE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_mid::ROCKFALL_VALE,
    1,
    "1017fda6-ed4f-4989-9f46-d61398aade55",
    "Piotr Dura",
);

// FRA 401 — Shipwreck Marsh (alternate printing)
const SHIPWRECK_MARSH_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &catalog_mid::SHIPWRECK_MARSH,
    1,
    "2cac144f-fa8a-4f04-9007-f7b7ab27a4b3",
    "Piotr Dura",
);

// FRA 402 — Bloodline Recollector // Ancestral Craving (alternate printing)
const BLOODLINE_RECOLLECTOR_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &BLOODLINE_RECOLLECTOR,
    1,
    "9a5db4d2-035f-4f48-8752-75487267c222",
    "Mark Poole",
);

// FRA 403 — Emrakul, the Exigent Doom (alternate printing)
const EMRAKUL_THE_EXIGENT_DOOM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &EMRAKUL_THE_EXIGENT_DOOM,
    1,
    "c4061059-9fdc-4bc2-b099-a06ddf3869f6",
    "Ikumi Nakamura",
);

// FRA 404 — Loyal Tutor (alternate printing)
const LOYAL_TUTOR_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &LOYAL_TUTOR,
    1,
    "1eaec01c-8178-49d1-a140-4cf41ef1a074",
    "NAJI YANAGIDA",
);

// FRA 405 — The Theorist, Jace Beleren (alternate printing)
const THE_THEORIST_JACE_BELEREN_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &THE_THEORIST_JACE_BELEREN,
    2,
    "326894e1-469d-4586-b823-9d5038315078",
    "Yukie Tajima",
);

// FRA 406 — Gardenize (alternate printing)
const GARDENIZE_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &GARDENIZE,
    2,
    "1ed34863-2dca-444e-85ea-70f305b0d32a",
    "Toru Terada",
);

// FRA 407 — Aerid Konstrari (alternate printing)
const AERID_KONSTRARI_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &AERID_KONSTRARI,
    2,
    "dc29f777-a9ac-488f-b36a-51b4a4fabe73",
    "NgN",
);

// FRA 408 — Denzilore Fatehold (alternate printing)
const DENZILORE_FATEHOLD_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &DENZILORE_FATEHOLD,
    2,
    "b770dda5-d248-427b-a393-eecc0b325364",
    "konomura",
);

// FRA 409 — Ingris Stingerquill (alternate printing)
const INGRIS_STINGERQUILL_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &INGRIS_STINGERQUILL,
    2,
    "480c8ceb-41df-4f83-a6b3-f7f8518ff602",
    "AKAGI",
);

// FRA 410 — Kwia Vigorbloom (alternate printing)
const KWIA_VIGORBLOOM_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &KWIA_VIGORBLOOM,
    2,
    "081ce1a1-e6a1-40cf-b2ba-eb20d7d56061",
    "Haru Akasaka",
);

// FRA 411 — Uldaros Theorix (alternate printing)
const ULDAROS_THEORIX_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &ULDAROS_THEORIX,
    2,
    "96984957-a126-46cb-a753-1be89f74b9d4",
    "SUSHIZAWA",
);

// FRA 412 — Codie, Ravenous Codex (alternate printing)
const CODIE_RAVENOUS_CODEX_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &CODIE_RAVENOUS_CODEX,
    1,
    "19e9f3f3-2a6a-4787-8b8c-2e27ce38fb4c",
    "saemon",
);

// FRA 413 — Emrakul, the Exigent Doom (alternate printing)
const EMRAKUL_THE_EXIGENT_DOOM_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &EMRAKUL_THE_EXIGENT_DOOM,
    2,
    "be305eb6-13b6-4b5d-9ca0-2dd4cafd0ac7",
    "Ikumi Nakamura",
);

// FRA 414 — Loyal Tutor (alternate printing)
const LOYAL_TUTOR_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &LOYAL_TUTOR,
    2,
    "07cac7ce-0b8d-4b40-8096-caa8a165be16",
    "NAJI YANAGIDA",
);

// FRA 415 — The Theorist, Jace Beleren (alternate printing)
const THE_THEORIST_JACE_BELEREN_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &THE_THEORIST_JACE_BELEREN,
    3,
    "8bcdda05-4d27-4da7-9fdb-064195aca568",
    "Yukie Tajima",
);

// FRA 416 — Gardenize (alternate printing)
const GARDENIZE_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &GARDENIZE,
    3,
    "ab61eefe-19c3-4a53-b7cd-227290032367",
    "Toru Terada",
);

// FRA 417 — Aerid Konstrari (alternate printing)
const AERID_KONSTRARI_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &AERID_KONSTRARI,
    3,
    "d0b12e0f-de6e-4cc3-ad04-b2c4ee556fd3",
    "NgN",
);

// FRA 418 — Denzilore Fatehold (alternate printing)
const DENZILORE_FATEHOLD_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &DENZILORE_FATEHOLD,
    3,
    "3cf001dd-52b8-48d8-a13d-cd0dc16003d8",
    "konomura",
);

// FRA 419 — Ingris Stingerquill (alternate printing)
const INGRIS_STINGERQUILL_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &INGRIS_STINGERQUILL,
    3,
    "a0686b23-2220-405b-8bc5-980f18af5fef",
    "AKAGI",
);

// FRA 420 — Kwia Vigorbloom (alternate printing)
const KWIA_VIGORBLOOM_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &KWIA_VIGORBLOOM,
    3,
    "c5ba91b6-1678-4bd5-8f83-9d7f418562f5",
    "Haru Akasaka",
);

// FRA 421 — Uldaros Theorix (alternate printing)
const ULDAROS_THEORIX_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &ULDAROS_THEORIX,
    3,
    "c6b00951-3f30-4846-b5fa-1f1d45dfd340",
    "SUSHIZAWA",
);

// FRA 422 — Codie, Ravenous Codex (alternate printing)
const CODIE_RAVENOUS_CODEX_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &CODIE_RAVENOUS_CODEX,
    2,
    "53015f09-fcb8-448e-baf3-36524f4217a7",
    "saemon",
);

// FRA 423 — Emrakul, the Exigent Doom (alternate printing)
const EMRAKUL_THE_EXIGENT_DOOM_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &EMRAKUL_THE_EXIGENT_DOOM,
    3,
    "c7fc8ddd-e165-4539-91c8-8af0e3f2b67e",
    "Cristi Balanescu",
);

// FRA 424 — Loyal Tutor (alternate printing)
const LOYAL_TUTOR_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &LOYAL_TUTOR,
    3,
    "f2d44935-2473-464f-b402-ea45b0e80c99",
    "Darren Tan",
);

// FRA 425 — Diviner of Victory // Unwind History (alternate printing)
const DIVINER_OF_VICTORY_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &DIVINER_OF_VICTORY,
    1,
    "83582080-bbd8-421e-80d3-66a7144f03fd",
    "Justyna Dura",
);

// FRA 426 — Variable Chaser // Arc of Fortune (alternate printing)
const VARIABLE_CHASER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &VARIABLE_CHASER,
    1,
    "6330415d-324c-4170-9269-eb024dc9e5a3",
    "Kai Carpenter",
);

// FRA 427 — Bloodline Recollector // Ancestral Craving (alternate printing)
const BLOODLINE_RECOLLECTOR_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &BLOODLINE_RECOLLECTOR,
    2,
    "9fb84b18-0775-44eb-83af-5dcb69518360",
    "Cynthia Sheppard",
);

// FRA 428 — Pompous Battlemage // Improvised Act (alternate printing)
const POMPOUS_BATTLEMAGE_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &POMPOUS_BATTLEMAGE,
    1,
    "dd78dea9-901c-4ff0-a155-06e2fa75b5d6",
    "Justin Gerard",
);

// FRA 429 — Pyre Rhymer // Molten Tide (alternate printing)
const PYRE_RHYMER_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &PYRE_RHYMER,
    1,
    "5111c3cd-ccfb-4ed6-9311-741f9d1dc524",
    "Andrea Piparo",
);

// FRA 430 — Carnivorous Cultivator // Enroot (alternate printing)
const CARNIVOROUS_CULTIVATOR_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &CARNIVOROUS_CULTIVATOR,
    1,
    "b06313ea-bf64-407b-878a-a2764055c851",
    "Martina Fačková",
);

// FRA 431 — Codie, Ravenous Codex (alternate printing)
const CODIE_RAVENOUS_CODEX_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &CODIE_RAVENOUS_CODEX,
    3,
    "1ad53db3-57e2-416d-907d-a71f63009c06",
    "Steve Prescott",
);

// FRA 432 — Fatehold Charm (alternate printing)
const FATEHOLD_CHARM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &FATEHOLD_CHARM,
    1,
    "a2008313-659e-4132-85a2-e7b7be93bcdc",
    "Rovina Cai",
);

// FRA 433 — Konstrari Charm (alternate printing)
const KONSTRARI_CHARM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &KONSTRARI_CHARM,
    1,
    "2261c33e-1667-4554-ad31-0adf4805a69d",
    "Matheus Graef",
);

// FRA 434 — Stingerquill Charm (alternate printing)
const STINGERQUILL_CHARM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &STINGERQUILL_CHARM,
    1,
    "7c9013bc-bc85-49b2-ad9b-745b97d6775d",
    "Danny Schwartz",
);

// FRA 435 — Theorix Charm (alternate printing)
const THEORIX_CHARM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &THEORIX_CHARM,
    1,
    "984491e5-4005-42d4-b144-29a87e388f15",
    "Elliot Lang",
);

// FRA 436 — Vigorbloom Charm (alternate printing)
const VIGORBLOOM_CHARM_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &VIGORBLOOM_CHARM,
    1,
    "8f273559-b6af-4dc1-9980-b132988c68b1",
    "Kaitlyn McCulley",
);

// FRA 437 — Carnivorous Cultivator // Enroot (alternate printing)
const CARNIVOROUS_CULTIVATOR_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &CARNIVOROUS_CULTIVATOR,
    2,
    "e9e31f13-8a2c-4ae2-854e-27d6c62936f2",
    "Filip Burburan",
);

// FRA 438 — Emrakul, the Exigent Doom (alternate printing)
const EMRAKUL_THE_EXIGENT_DOOM_ALTERNATE_4: PrintingRecord = PrintingRecord::alternate(
    &EMRAKUL_THE_EXIGENT_DOOM,
    4,
    "ee609b68-5c9c-43e0-aff4-eb1bbc8b2911",
    "Princess Hidir",
);

// FRA 439 — Enlightened Confidant (alternate printing)
const ENLIGHTENED_CONFIDANT_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &ENLIGHTENED_CONFIDANT,
    2,
    "236b2024-9141-4a91-b5f8-2180ecc3a761",
    "James Martin",
);

// FRA 440 — Return to the Light Realms (alternate printing)
const RETURN_TO_THE_LIGHT_REALMS_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &RETURN_TO_THE_LIGHT_REALMS,
    2,
    "65e9f4df-02e5-4ac7-86da-795688bfb66f",
    "Heikala",
);

// FRA 441 — Seasoned Cryomancer (alternate printing)
const SEASONED_CRYOMANCER_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &SEASONED_CRYOMANCER,
    2,
    "f1a15a83-2f74-4513-ae4b-4cdc17016a84",
    "Mattias Lindström",
);

// FRA 442 — Sphinx's Approach (alternate printing)
const SPHINX_S_APPROACH_ALTERNATE_1: PrintingRecord = PrintingRecord::alternate(
    &SPHINX_S_APPROACH,
    1,
    "e57d1edd-8e8d-4bb7-8833-316549818efa",
    "Harry Conway",
);

// FRA 443 — The Theorist, Jace Beleren (alternate printing)
const THE_THEORIST_JACE_BELEREN_ALTERNATE_4: PrintingRecord = PrintingRecord::alternate(
    &THE_THEORIST_JACE_BELEREN,
    4,
    "29a7f3e8-efcd-42bb-b315-9e9c0794dddb",
    "ONIKU kuitai",
);

// FRA 444 — Darklight Phoenix (alternate printing)
const DARKLIGHT_PHOENIX_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &DARKLIGHT_PHOENIX,
    2,
    "16c5f0f3-2578-40d3-9012-6eaef17fa0b1",
    "Jun Ishimaru",
);

// FRA 445 — Overwrite the Multiverse (alternate printing)
const OVERWRITE_THE_MULTIVERSE_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &OVERWRITE_THE_MULTIVERSE,
    2,
    "f28b1518-8347-4385-9f34-a9c48818c770",
    "Pedro Correa",
);

// FRA 446 — Craterclaw Colossus (alternate printing)
const CRATERCLAW_COLOSSUS_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &CRATERCLAW_COLOSSUS,
    2,
    "60b9f969-694a-4445-9b0f-525c259edfc8",
    "Desmuncubic",
);

// FRA 447 — Stingcaster Mage (alternate printing)
const STINGCASTER_MAGE_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &STINGCASTER_MAGE,
    2,
    "3d580fa2-954d-4de6-a64c-369c5ab8e03f",
    "Ricardo Diseño",
);

// FRA 448 — Omnipresence (alternate printing)
const OMNIPRESENCE_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &OMNIPRESENCE,
    2,
    "91e787cb-a9b0-4353-86b4-a4340122fd2f",
    "Killoffer",
);

// FRA 449 — Enlightened Confidant (alternate printing)
const ENLIGHTENED_CONFIDANT_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &ENLIGHTENED_CONFIDANT,
    3,
    "81372104-e98c-45c9-9490-6ccbb0dac309",
    "Jabari Weathers",
);

// FRA 450 — Kindred Judgment (alternate printing)
const KINDRED_JUDGMENT_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &KINDRED_JUDGMENT,
    2,
    "e24ab744-0033-4793-b379-ae6b51114d17",
    "Joshua Cairos",
);

// FRA 451 — Return to the Light Realms (alternate printing)
const RETURN_TO_THE_LIGHT_REALMS_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &RETURN_TO_THE_LIGHT_REALMS,
    3,
    "0354dee4-bbc7-4ab4-9fab-4e7bbda887ba",
    "Magali Villeneuve",
);

// FRA 452 — Seasoned Cryomancer (alternate printing)
const SEASONED_CRYOMANCER_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &SEASONED_CRYOMANCER,
    3,
    "5421efaf-a6e1-4f91-927a-eea863a990d2",
    "Chris Rallis",
);

// FRA 453 — Darklight Phoenix (alternate printing)
const DARKLIGHT_PHOENIX_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &DARKLIGHT_PHOENIX,
    3,
    "033ed4e4-d40a-4f2d-851b-23df6312c5d8",
    "Gonzalo Kenny",
);

// FRA 454 — Rise of the Deathbringer (alternate printing)
const RISE_OF_THE_DEATHBRINGER_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &RISE_OF_THE_DEATHBRINGER,
    2,
    "a18ea67e-05eb-47c8-91ae-3ebc5b47aa0f",
    "Kieran Yanner",
);

// FRA 455 — Craterclaw Colossus (alternate printing)
const CRATERCLAW_COLOSSUS_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &CRATERCLAW_COLOSSUS,
    3,
    "3b67dc41-1157-47eb-b126-0149b477d551",
    "Raluca Marinescu",
);

// FRA 456 — Curse-Marred Demon (alternate printing)
const CURSE_MARRED_DEMON_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &CURSE_MARRED_DEMON,
    2,
    "a332a8ac-e7b4-4be8-b4de-c613d95885cb",
    "Thomas M. Baxa",
);

// FRA 457 — Stingcaster Mage (alternate printing)
const STINGCASTER_MAGE_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &STINGCASTER_MAGE,
    3,
    "38096f2a-7fae-43f1-a1e9-f9c670bc191d",
    "Gal Or",
);

// FRA 458 — Hexhaven Invigorator (alternate printing)
const HEXHAVEN_INVIGORATOR_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &HEXHAVEN_INVIGORATOR,
    2,
    "125b65ab-541e-414f-8da8-c80f6325db0a",
    "David Auden Nash",
);

// FRA 459 — Omnipresence (alternate printing)
const OMNIPRESENCE_ALTERNATE_3: PrintingRecord = PrintingRecord::alternate(
    &OMNIPRESENCE,
    3,
    "076a0b0f-e818-4e67-b786-16d389371c4c",
    "Chase Stone",
);

// FRA 460 — Simulacrum Shaper (alternate printing)
const SIMULACRUM_SHAPER_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &SIMULACRUM_SHAPER,
    2,
    "1b3e3d1c-c452-46ef-a41d-6dc14686bc4f",
    "Gustavo Pelissari",
);

// FRA 461 — Frostbite Pyromental (alternate printing)
const FROSTBITE_PYROMENTAL_ALTERNATE_2: PrintingRecord = PrintingRecord::alternate(
    &FROSTBITE_PYROMENTAL,
    2,
    "6c4846a6-49fa-4eba-ac5c-9c40e3949e0f",
    "Paul Adam",
);

pub(in crate::card::sets) static CARDS: &[&CardRecord] = &[
    &EMRAKUL_THE_EXIGENT_DOOM,
    &ACADEMIC_ASCENT,
    &BLOSSOM_BLESSED_ANGEL,
    &CAMPUS_CRIER,
    &ENLIGHTENED_CONFIDANT,
    &FATESHAPER_ASPIRANT,
    &FLICKERING_HOUND,
    &GENEROUS_REVIVAL,
    &GERMINATE_RECRUITS,
    &GRAFT_SURGEON,
    &GUIDING_HYDRA,
    &HEXHAVEN_BATTALION,
    &KINDRED_JUDGMENT,
    &LOYAL_TUTOR,
    &MEMORY_TRAP,
    &PREDICTIVE_PREPARATIONS,
    &PROPHESIED_END,
    &REFUTE_DESTINY,
    &REPURPOSED_ENFORCER,
    &RETURN_TO_THE_LIGHT_REALMS,
    &SHATTERWING_PEGASUS,
    &SURGICAL_PRECISION,
    &UNFLINCHING_HORTIMANCER,
    &YOUR_FATE_ENDS_HERE,
    &COUNTERSCULPT,
    &CRUEL_CALCULATIONS,
    &CRYOTHEORY_ADEPT,
    &DIVINER_OF_VICTORY,
    &DIVINING_DUELIST,
    &ICY_RECEPTION,
    &INFINITE_COURSEWORK,
    &JACE_S_MACHINATIONS,
    &MINDSEEKER_OCULUS,
    &PERFECTED_THEORY,
    &PLAN_FOR_ALL_OUTCOMES,
    &PRECISE_REDACTION,
    &PROTEGE_S_AWAKENING,
    &SEASONED_CRYOMANCER,
    &SEMESTER_FORESEER,
    &SPHINX_OF_FALSE_CONCLUSIONS,
    &SPHINX_S_APPROACH,
    &SURVEILLANCE_PHANTASM,
    &THE_THEORIST_JACE_BELEREN,
    &THEORIST_S_PROXY,
    &UNDULATING_WITNESS,
    &VARIABLE_CHASER,
    &APEX_WITCHSTALKER,
    &BLOODLINE_RECOLLECTOR,
    &BREAK_UNDER_PRESSURE,
    &CAST_AWAY_DOUBT,
    &DARK_MATTER_MANIPULATOR,
    &DARKLIGHT_PHOENIX,
    &EXTENDED_ABSENCE,
    &EXTRAPOLATE_THE_IMPOSSIBLE,
    &LICH_S_RELIC,
    &MULTIPLY_BY_ZERO,
    &OVERWRITE_THE_MULTIVERSE,
    &RAMPART_HUNTER,
    &RANK_RAT,
    &REWRITE_REGRETS,
    &RISE_OF_THE_DEATHBRINGER,
    &SANCTUM_LURKER,
    &SCREECHING_SOULBREAKER,
    &SILENCE_THE_ECHO,
    &SOLVE_FOR_DISAPPOINTMENT,
    &TERMINAL_CRITICISM,
    &THEORETICAL_NECROMANCER,
    &VOID_EXTRAPOLATOR,
    &VRASKA_S_FINAL_MERCY,
    &AJANI_S_ANGUISH,
    &ARTIFIST_ACUMEN,
    &AWAKEN_THE_INFERNO,
    &CHANDRA_S_EMBERLING,
    &COMMAND_THE_STAGE,
    &CRATERCLAW_COLOSSUS,
    &CURSE_MARRED_DEMON,
    &DRACONIC_VISITOR,
    &EARDRUM_RATTLER,
    &ESSENCE_BURN,
    &FACE_YOURSELF,
    &FULMINOUS_FORTE,
    &HALLWAY_HECKLER,
    &HEARTSTRING_PULLER,
    &IDENTITY_ECHO,
    &MASTER_OF_BARBS,
    &NO_ADMITTANCE,
    &POMPOUS_BATTLEMAGE,
    &PYRE_RHYMER,
    &SKILLED_BATTLECARVER,
    &STINGCASTER_MAGE,
    &TETHER_TECHNICIAN,
    &VIOLENT_ECHOES,
    &WRATH_OF_THE_BLOODMANE,
    &ARCANE_AMPHISBAENA,
    &BESTIAL_INCURSION,
    &BUDDING_INSURGENT,
    &CARNIVOROUS_CULTIVATOR,
    &COMPEL_BRUTALITY,
    &FLOURISHING_GRAPPLE,
    &GARDENIZE,
    &GREENHOUSE_PROPAGATOR,
    &HEARTWOOD_CRAFTER,
    &HEXHAVEN_INVIGORATOR,
    &HUNGERING_PUPPETBEAST,
    &HUNTER_S_AXE,
    &INSPIRED_TETHERMAGE,
    &OMNIPRESENCE,
    &PUPPET_CRAFTING,
    &RESTORE_WITH_EMPATHY,
    &SIMULACRUM_SHAPER,
    &SOMETHING_WORTH_SAVING,
    &SURESHOT_SOWER,
    &TETHERMAGE_S_ADVANTAGE,
    &VERDANT_KRAKEN,
    &VINELASHER_ADEPT,
    &WRECKING_GECKO,
    &AERID_KONSTRARI,
    &AVATAR_OF_BURGEONING_ECHOES,
    &BLESSED_GHOUL,
    &BLOOMBRUTE,
    &CHARGE_THE_SANCTUM,
    &CLASH_OF_ELEMENTS,
    &CRAFTWORK_CRUSHER,
    &DENZILORE_FATEHOLD,
    &DESPERATE_FUTURESCRIBE,
    &EMERGENCY_PHYTOMEDIC,
    &ENTRUST_THE_SPARK,
    &FATEHOLD_CHARM,
    &FATEHOLD_CHRONOLOGIST,
    &FEROCITY_OF_THE_HUNT,
    &FROSTBITE_PYROMENTAL,
    &GRIM_REPRISER,
    &INGRIS_STINGERQUILL,
    &KONSTRARI_CHARM,
    &KONSTRARI_IMPROVISER,
    &KWIA_VIGORBLOOM,
    &MIND_MEANDERER,
    &NULL_SUMMONER,
    &PARADOX_SHAPER,
    &PRIMAL_WITCHSTALKER,
    &PROCTOR_OF_POTENTIAL,
    &PRUDENT_FATESEER,
    &RECURSIVE_RECRUITMENT,
    &SOLARIUM_SENTRY,
    &SOLITARY_CELL,
    &STINGERQUILL_CHARM,
    &STINGERQUILL_VOXMANCER,
    &STINGING_VITRIOL,
    &TAM_S_RESISTANCE,
    &TENURED_TETHERMAGE,
    &THEORIX_CHARM,
    &THEORIX_METAMAGE,
    &TWINNED_VISION,
    &TWISTED_FATES,
    &ULDAROS_THEORIX,
    &VIGORBLOOM_CHARM,
    &VIGORBLOOM_VANGUARD,
    &VINDICTIVE_TRIUMPH,
    &WARRIOR_S_BLADES,
    &WHIPLASH_WORDSMITH,
    &WOODWORK_PRODIGY,
    &AFTERTHOUGHT_SENTRY,
    &ARCHIVE_ARBITER,
    &CODIE_RAVENOUS_CODEX,
    &THE_ECHOVERSE_FULCRUM,
    &EYE_OF_JACE,
    &KEEPER_OF_THE_QUIET_HOUR,
    &LIVING_LIBRARY,
    &MEDIC_S_KITESAIL,
    &MURMURING_VOLUME,
    &DEDICATED_COMMONS,
    &FATEHOLD_ANNEX,
    &FORMIDABLE_COMMONS,
    &HALL_OF_ECHOES,
    &HEXHAVEN_DUELING_ARENA,
    &INNOVATIVE_COMMONS,
    &KONSTRARI_ANNEX,
    &METICULOUS_COMMONS,
    &ROILING_CANOPY,
    &ROOM_OF_REFUGE,
    &STINGERQUILL_ANNEX,
    &THEORIST_S_SANCTUM,
    &THEORIX_ANNEX,
    &TRANSFORMATIVE_COMMONS,
    &VIGORBLOOM_ANNEX,
    &AJANI_RESOLUTE,
    &DANITHA_SWORD_OF_HOPE,
    &GHALTA_THE_IMMOVABLE,
    &GIDEON_S_MEMORIAL,
    &KOTH_OF_THE_HOMESTEAD,
    &LILIANA_THE_FAULTLESS,
    &LYRA_ARCHANGEL_OF_DAWN,
    &RESCUE_GIRL_FIRST_RESPONDER,
    &SAHEELI_CONSUL_OF_OVERSIGHT,
    &TEYO_LIGHTSHIELD_EXPERT,
    &THALIA_THE_SURVIVOR,
    &TOMIK_ORZHOV_LAWMAGE,
    &WAY_OF_THE_HEALER,
    &WAY_OF_THE_MENTOR,
    &YOSHIMARU_BELOVED_COMPANION,
    &YURIKO_BLADE_OF_THE_MIGHTY,
    &ARNI_HUMBLE_SCRIBE,
    &CHANDRA_CHILL_OF_COMPLIANCE,
    &FBLTHP_IMPOSSIBLY_LOST,
    &GEIST_OF_SAINT_THALIA,
    &HAPATRA_THE_DESERT_FROST,
    &JACE_REALITY_SCULPTOR,
    &LYRA_TOLARIAN_ARCHANGEL,
    &PROFT_CONSULTING_DETECTIVE,
    &RURIC_THAR_BIOMAGUS,
    &SAMUT_TYRANT_OF_NAKTAMUN,
    &TRAXOS_ACADEMY_GUARDIAN,
    &WAY_OF_THE_CRYOMANCER,
    &WAY_OF_THE_MIND_SCULPTOR,
    &YARGLE_GOLIATH_OF_OTARIA,
    &YURIKO_HOPE_FROM_THE_SHADOWS,
    &DANITHA_SPEAR_OF_AGONY,
    &GALLIA_TRAGIC_HOST,
    &GARRUK_VEILED_BUTCHER,
    &GIDEON_THE_OATHLESS,
    &LILIANA_THE_REPENTANT,
    &LOOT_THE_ANOMALY,
    &MABEL_BITTER_RECLUSE,
    &MASSACRE_GIRL_MOST_WANTED,
    &PROFT_SINISTER_MASTERMIND,
    &TEYO_DIAMONDBLADE_MAGE,
    &TINYBONES_POCKET_NUISANCE,
    &WAY_OF_THE_DEATHBRINGER,
    &WAY_OF_THE_NECROMANCER,
    &WINTER_TORMENTED_LONER,
    &AJANI_UNRELENTING,
    &ARNI_RENOWNED_CHAMPION,
    &GALLIA_THE_MERRYMAKER,
    &JIANG_YANGGU_ALONE,
    &KIORA_OF_FIRE_AND_ASHES,
    &KOTH_THE_GEOMANCER,
    &MARWYN_THE_CLEARCUTTER,
    &PIA_DETERMINED_REBUILDER,
    &SAMUT_HAZORET_S_CHAMPION,
    &TETSUKO_UMEZAWA_PURSUER,
    &TOMIK_IZZET_SPARKMAGE,
    &WAY_OF_THE_PYROMANCER,
    &WAY_OF_THE_WARLORD,
    &WINTER_TEAM_PLAYER,
    &EDGAR_MOONLIT_SOVEREIGN,
    &FBLTHP_KNOWS_THE_WAY,
    &GARRUK_CURSE_BREAKER,
    &GHALTA_THE_UNSTOPPABLE,
    &JIANG_YANGGU_NEVER_ALONE,
    &LOOT_THE_NEXUS,
    &MARWYN_THE_PRESERVER,
    &PIA_AETHER_ASCETIC,
    &RURIC_THAR_MAGECRUSHER,
    &TITANBONES_TOWERING_HEART,
    &WAY_OF_THE_PARADOX,
    &WAY_OF_THE_WILDSPEAKER,
    &YOSHIMARU_SCRAPPY_STRAY,
    &EDGAR_ANCIENT_BLOODLORD,
    &HAPATRA_THE_DESERT_FANG,
    &KARN_GILDED_GUARDIAN,
    &KIORA_OF_SALT_AND_SAND,
    &MABEL_VALLEY_HERO,
    &SAHEELI_JEWEL_OF_AVISHKAR,
    &TAM_THE_POSSIBILITY,
    &VRASKA_SOUL_OF_STONE,
    &VRASKA_THE_CUTTING_GLARE,
    &KARN_ARGENT_DEFENDER,
    &TRAXOS_SCOURGE_ETERNAL,
];

pub(in crate::card::sets) static ADDITIONAL_PRINTINGS: &[PrintingRecord] = &[
    UNSUMMON_REPRINT,
    LAST_GASP_REPRINT,
    BLAZING_CRESCENDO_REPRINT,
    COMPEL_BRUTALITY_ALTERNATE_1,
    TARMOGOYF_REPRINT,
    DESERTED_BEACH_REPRINT,
    HAUNTED_RIDGE_REPRINT,
    OVERGROWN_FARMLAND_REPRINT,
    ROCKFALL_VALE_REPRINT,
    SHIPWRECK_MARSH_REPRINT,
    TETSUKO_UMEZAWA_FUGITIVE_REPRINT,
    YARGLE_GLUTTON_OF_URBORG_REPRINT,
    CHANDRA_TORCH_OF_DEFIANCE_REPRINT,
    PLAINS_REPRINT,
    PLAINS_ALTERNATE_1,
    ISLAND_REPRINT,
    ISLAND_ALTERNATE_1,
    SWAMP_REPRINT,
    SWAMP_ALTERNATE_1,
    MOUNTAIN_REPRINT,
    MOUNTAIN_ALTERNATE_1,
    FOREST_REPRINT,
    FOREST_ALTERNATE_1,
    AJANI_RESOLUTE_ALTERNATE_1,
    GIDEON_S_MEMORIAL_ALTERNATE_1,
    LILIANA_THE_FAULTLESS_ALTERNATE_1,
    LYRA_ARCHANGEL_OF_DAWN_ALTERNATE_1,
    TEYO_LIGHTSHIELD_EXPERT_ALTERNATE_1,
    THALIA_THE_SURVIVOR_ALTERNATE_1,
    CHANDRA_CHILL_OF_COMPLIANCE_ALTERNATE_1,
    FBLTHP_IMPOSSIBLY_LOST_ALTERNATE_1,
    GEIST_OF_SAINT_THALIA_ALTERNATE_1,
    JACE_REALITY_SCULPTOR_ALTERNATE_1,
    LYRA_TOLARIAN_ARCHANGEL_ALTERNATE_1,
    SAMUT_TYRANT_OF_NAKTAMUN_ALTERNATE_1,
    GARRUK_VEILED_BUTCHER_ALTERNATE_1,
    GIDEON_THE_OATHLESS_ALTERNATE_1,
    LILIANA_THE_REPENTANT_ALTERNATE_1,
    TEYO_DIAMONDBLADE_MAGE_ALTERNATE_1,
    WINTER_TORMENTED_LONER_ALTERNATE_1,
    AJANI_UNRELENTING_ALTERNATE_1,
    CHANDRA_TORCH_OF_DEFIANCE_ALTERNATE_1,
    MARWYN_THE_CLEARCUTTER_ALTERNATE_1,
    SAMUT_HAZORET_S_CHAMPION_ALTERNATE_1,
    WINTER_TEAM_PLAYER_ALTERNATE_1,
    FBLTHP_KNOWS_THE_WAY_ALTERNATE_1,
    GARRUK_CURSE_BREAKER_ALTERNATE_1,
    MARWYN_THE_PRESERVER_ALTERNATE_1,
    KARN_GILDED_GUARDIAN_ALTERNATE_1,
    TAM_THE_POSSIBILITY_ALTERNATE_1,
    VRASKA_SOUL_OF_STONE_ALTERNATE_1,
    VRASKA_THE_CUTTING_GLARE_ALTERNATE_1,
    KARN_ARGENT_DEFENDER_ALTERNATE_1,
    ENLIGHTENED_CONFIDANT_ALTERNATE_1,
    KINDRED_JUDGMENT_ALTERNATE_1,
    RETURN_TO_THE_LIGHT_REALMS_ALTERNATE_1,
    SEASONED_CRYOMANCER_ALTERNATE_1,
    DARKLIGHT_PHOENIX_ALTERNATE_1,
    RISE_OF_THE_DEATHBRINGER_ALTERNATE_1,
    CRATERCLAW_COLOSSUS_ALTERNATE_1,
    CURSE_MARRED_DEMON_ALTERNATE_1,
    STINGCASTER_MAGE_ALTERNATE_1,
    HEXHAVEN_INVIGORATOR_ALTERNATE_1,
    OMNIPRESENCE_ALTERNATE_1,
    SIMULACRUM_SHAPER_ALTERNATE_1,
    FROSTBITE_PYROMENTAL_ALTERNATE_1,
    GERMINATE_RECRUITS_ALTERNATE_1,
    REPURPOSED_ENFORCER_ALTERNATE_1,
    CRUEL_CALCULATIONS_ALTERNATE_1,
    JACE_S_MACHINATIONS_ALTERNATE_1,
    SPHINX_OF_FALSE_CONCLUSIONS_ALTERNATE_1,
    THEORIST_S_PROXY_ALTERNATE_1,
    DARK_MATTER_MANIPULATOR_ALTERNATE_1,
    OVERWRITE_THE_MULTIVERSE_ALTERNATE_1,
    SANCTUM_LURKER_ALTERNATE_1,
    VRASKA_S_FINAL_MERCY_ALTERNATE_1,
    MASTER_OF_BARBS_ALTERNATE_1,
    GARDENIZE_ALTERNATE_1,
    HUNGERING_PUPPETBEAST_ALTERNATE_1,
    AERID_KONSTRARI_ALTERNATE_1,
    AVATAR_OF_BURGEONING_ECHOES_ALTERNATE_1,
    DENZILORE_FATEHOLD_ALTERNATE_1,
    ENTRUST_THE_SPARK_ALTERNATE_1,
    INGRIS_STINGERQUILL_ALTERNATE_1,
    KWIA_VIGORBLOOM_ALTERNATE_1,
    NULL_SUMMONER_ALTERNATE_1,
    PROCTOR_OF_POTENTIAL_ALTERNATE_1,
    SOLITARY_CELL_ALTERNATE_1,
    STINGING_VITRIOL_ALTERNATE_1,
    TENURED_TETHERMAGE_ALTERNATE_1,
    ULDAROS_THEORIX_ALTERNATE_1,
    FLICKERING_HOUND_ALTERNATE_1,
    GUIDING_HYDRA_ALTERNATE_1,
    REFUTE_DESTINY_ALTERNATE_1,
    PRECISE_REDACTION_ALTERNATE_1,
    THE_THEORIST_JACE_BELEREN_ALTERNATE_1,
    EXTRAPOLATE_THE_IMPOSSIBLE_ALTERNATE_1,
    LICH_S_RELIC_ALTERNATE_1,
    TERMINAL_CRITICISM_ALTERNATE_1,
    AJANI_S_ANGUISH_ALTERNATE_1,
    DRACONIC_VISITOR_ALTERNATE_1,
    ESSENCE_BURN_ALTERNATE_1,
    FACE_YOURSELF_ALTERNATE_1,
    IDENTITY_ECHO_ALTERNATE_1,
    FLOURISHING_GRAPPLE_ALTERNATE_1,
    PUPPET_CRAFTING_ALTERNATE_1,
    TARMOGOYF_ALTERNATE_1,
    VERDANT_KRAKEN_ALTERNATE_1,
    SOLARIUM_SENTRY_ALTERNATE_1,
    VINDICTIVE_TRIUMPH_ALTERNATE_1,
    THE_ECHOVERSE_FULCRUM_ALTERNATE_1,
    HALL_OF_ECHOES_ALTERNATE_1,
    ROILING_CANOPY_ALTERNATE_1,
    THEORIST_S_SANCTUM_ALTERNATE_1,
    PLAINS_ALTERNATE_2,
    PLAINS_ALTERNATE_3,
    PLAINS_ALTERNATE_4,
    ISLAND_ALTERNATE_2,
    ISLAND_ALTERNATE_3,
    ISLAND_ALTERNATE_4,
    SWAMP_ALTERNATE_2,
    SWAMP_ALTERNATE_3,
    SWAMP_ALTERNATE_4,
    MOUNTAIN_ALTERNATE_2,
    MOUNTAIN_ALTERNATE_3,
    MOUNTAIN_ALTERNATE_4,
    FOREST_ALTERNATE_2,
    FOREST_ALTERNATE_3,
    FOREST_ALTERNATE_4,
    DESERTED_BEACH_ALTERNATE_1,
    HAUNTED_RIDGE_ALTERNATE_1,
    OVERGROWN_FARMLAND_ALTERNATE_1,
    ROCKFALL_VALE_ALTERNATE_1,
    SHIPWRECK_MARSH_ALTERNATE_1,
    BLOODLINE_RECOLLECTOR_ALTERNATE_1,
    EMRAKUL_THE_EXIGENT_DOOM_ALTERNATE_1,
    LOYAL_TUTOR_ALTERNATE_1,
    THE_THEORIST_JACE_BELEREN_ALTERNATE_2,
    GARDENIZE_ALTERNATE_2,
    AERID_KONSTRARI_ALTERNATE_2,
    DENZILORE_FATEHOLD_ALTERNATE_2,
    INGRIS_STINGERQUILL_ALTERNATE_2,
    KWIA_VIGORBLOOM_ALTERNATE_2,
    ULDAROS_THEORIX_ALTERNATE_2,
    CODIE_RAVENOUS_CODEX_ALTERNATE_1,
    EMRAKUL_THE_EXIGENT_DOOM_ALTERNATE_2,
    LOYAL_TUTOR_ALTERNATE_2,
    THE_THEORIST_JACE_BELEREN_ALTERNATE_3,
    GARDENIZE_ALTERNATE_3,
    AERID_KONSTRARI_ALTERNATE_3,
    DENZILORE_FATEHOLD_ALTERNATE_3,
    INGRIS_STINGERQUILL_ALTERNATE_3,
    KWIA_VIGORBLOOM_ALTERNATE_3,
    ULDAROS_THEORIX_ALTERNATE_3,
    CODIE_RAVENOUS_CODEX_ALTERNATE_2,
    EMRAKUL_THE_EXIGENT_DOOM_ALTERNATE_3,
    LOYAL_TUTOR_ALTERNATE_3,
    DIVINER_OF_VICTORY_ALTERNATE_1,
    VARIABLE_CHASER_ALTERNATE_1,
    BLOODLINE_RECOLLECTOR_ALTERNATE_2,
    POMPOUS_BATTLEMAGE_ALTERNATE_1,
    PYRE_RHYMER_ALTERNATE_1,
    CARNIVOROUS_CULTIVATOR_ALTERNATE_1,
    CODIE_RAVENOUS_CODEX_ALTERNATE_3,
    FATEHOLD_CHARM_ALTERNATE_1,
    KONSTRARI_CHARM_ALTERNATE_1,
    STINGERQUILL_CHARM_ALTERNATE_1,
    THEORIX_CHARM_ALTERNATE_1,
    VIGORBLOOM_CHARM_ALTERNATE_1,
    CARNIVOROUS_CULTIVATOR_ALTERNATE_2,
    EMRAKUL_THE_EXIGENT_DOOM_ALTERNATE_4,
    ENLIGHTENED_CONFIDANT_ALTERNATE_2,
    RETURN_TO_THE_LIGHT_REALMS_ALTERNATE_2,
    SEASONED_CRYOMANCER_ALTERNATE_2,
    SPHINX_S_APPROACH_ALTERNATE_1,
    THE_THEORIST_JACE_BELEREN_ALTERNATE_4,
    DARKLIGHT_PHOENIX_ALTERNATE_2,
    OVERWRITE_THE_MULTIVERSE_ALTERNATE_2,
    CRATERCLAW_COLOSSUS_ALTERNATE_2,
    STINGCASTER_MAGE_ALTERNATE_2,
    OMNIPRESENCE_ALTERNATE_2,
    ENLIGHTENED_CONFIDANT_ALTERNATE_3,
    KINDRED_JUDGMENT_ALTERNATE_2,
    RETURN_TO_THE_LIGHT_REALMS_ALTERNATE_3,
    SEASONED_CRYOMANCER_ALTERNATE_3,
    DARKLIGHT_PHOENIX_ALTERNATE_3,
    RISE_OF_THE_DEATHBRINGER_ALTERNATE_2,
    CRATERCLAW_COLOSSUS_ALTERNATE_3,
    CURSE_MARRED_DEMON_ALTERNATE_2,
    STINGCASTER_MAGE_ALTERNATE_3,
    HEXHAVEN_INVIGORATOR_ALTERNATE_2,
    OMNIPRESENCE_ALTERNATE_3,
    SIMULACRUM_SHAPER_ALTERNATE_2,
    FROSTBITE_PYROMENTAL_ALTERNATE_2,
];
