//! Preparation cards (CR 722) declared as a permanent and its prepare spell.
//!
//! "Engine mode" below is the prepared engine switched on or off. That engine
//! lowers ordinary effect programs and has nothing to do with the prepare
//! mechanic; every test here runs in both modes because the two must agree.
use super::woe_hob_completion::{hand, setup};
use super::*;
use crate::card::sets::secrets_of_strixhaven as sos;
use crate::card::{
    AlternateSpellKind, CopyExceptionsDef, CreateTokenDef, PermanentDesignationDef, TokenCopyDef,
    TokenDef,
};

const UPKEEP_MAGE: &str = "00000000-0000-0000-0000-000000722001";
const JOLT_MAGE: &str = "00000000-0000-0000-0000-000000722002";
const RECALL_STUDENT: &str = "00000000-0000-0000-0000-000000722003";
const READY: &str = "00000000-0000-0000-0000-000000722004";
const UNREADY: &str = "00000000-0000-0000-0000-000000722005";
const FROG_COUNTERPART: &str = "00000000-0000-0000-0000-000000722006";
const SHAPESHIFT: &str = "00000000-0000-0000-0000-000000722007";
const HUSH: &str = "00000000-0000-0000-0000-000000722008";

static A_CREATURE: [AbilityTargetDef; 1] = [AbilityTargetDef::exactly_one_permanent(
    ObjectPredicateDef::HasType(CardType::Creature),
)];
static TWO_CREATURES: [AbilityTargetDef; 2] = [A_CREATURE[0], A_CREATURE[0]];
static ENTERS_PREPARED: [AbilityDef; 1] = [sos::enters_prepared()];

/// Adds `definitions` to the game's catalog, keeping its engine mode.
fn add_definitions(game: &mut Game, definitions: Vec<CardDefinition>) {
    let prepared_engine = game.prepared_engine_enabled();
    game.catalog = CardCatalog::new(
        game.catalog
            .definitions()
            .into_iter()
            .cloned()
            .chain(definitions),
    )
    .expect("the test definitions are valid catalog entries");
    game.prepared_engine = crate::prepared_engine::PreparedEngine::compile(&game.catalog);
    game.set_prepared_engine_enabled(prepared_engine);
}

/// A ready board in the chosen engine mode whose catalog also holds
/// `definitions`.
fn game_with(prepared_engine: bool, definitions: Vec<CardDefinition>) -> Game {
    let mut game = setup(prepared_engine);
    add_definitions(&mut game, definitions);
    game
}

/// A preparation card composed the way `CardRecord::new_prepare` composes one.
fn prepare_card(uuid: &str, parts: &[(&'static str, CardRules); 2]) -> CardDefinition {
    let [(permanent_name, permanent), (spell_name, _)] = *parts;
    let composition = CardComposition::alternate_spell(parts, AlternateSpellKind::Prepare);
    let mut definition = CardDefinition::new(
        CardDefinitionId::from_uuid(uuid),
        format!("{permanent_name} // {spell_name}"),
        sos::SET,
        permanent,
    );
    definition.parts = composition.parts;
    definition.structure = composition.structure;
    definition.play_options = composition.play_options;
    definition
}

/// A one-blue-mana instant that does `effect` to its targets.
fn instant(
    uuid: &str,
    name: &str,
    text: &'static str,
    targets: &'static [AbilityTargetDef],
    effect: EffectDef,
) -> CardDefinition {
    CardDefinition::new(
        CardDefinitionId::from_uuid(uuid),
        name,
        sos::SET,
        CardRules::new_instant(mana_cost!("{U}"))
            .with_ability(AbilityDef::spell_with_targets(text, targets, effect)),
    )
}

/// A 2/2 that enters prepared with `spell` as its prepare spell.
fn mage_entering_prepared(
    uuid: &str,
    name: &'static str,
    spell_name: &'static str,
    spell: CardRules,
) -> CardDefinition {
    prepare_card(
        uuid,
        &[
            (
                name,
                CardRules::new_creature(mana_cost!("{1}{U}"), &["Wizard"], 2, 2)
                    .with_abilities(&ENTERS_PREPARED),
            ),
            (spell_name, spell),
        ],
    )
}

fn rules_of(game: &Game, definition: CardDefinitionId) -> CardRules {
    game.catalog
        .get(definition)
        .expect("the card is in the catalog")
        .rules
}

fn is_prepared(game: &Game, permanent: GameObjectId) -> bool {
    game.battlefield
        .iter()
        .find(|candidate| candidate.card.id == permanent)
        .expect("the permanent is on the battlefield")
        .designations
        .contains(&PermanentDesignationDef::Prepared)
}

/// Everything `player` has in exile, by name. Nothing but prepare-spell
/// copies is exiled in these tests.
fn exiled_copies(game: &Game, player: PlayerId) -> Vec<(GameObjectId, String)> {
    game.players[player.index()]
        .exile
        .iter()
        .map(|card| {
            (
                card.id,
                game.object_card_name(card.id)
                    .expect("an exiled copy has a name")
                    .into_owned(),
            )
        })
        .collect()
}

/// What the rest of the engine reads an exiled card as.
fn exiled_view(game: &Game, id: GameObjectId) -> TriggerEventObject {
    let card = game
        .players
        .iter()
        .flat_map(|player| &player.exile)
        .find(|card| card.id == id)
        .expect("the card is in exile");
    game.printed_trigger_event_object(
        id,
        card.definition,
        card.owner,
        &CharacteristicContext::Exile,
    )
    .expect("an exiled card has characteristics")
}

fn cast_action(game: &Game, card: GameObjectId, targets: &[Target]) -> Option<Action> {
    game.legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| {
            matches!(action, Action::CastSpell { card: cast, choices, .. }
            if *cast == card && choices.iter_targets().eq(targets))
        })
}

/// Player one casts `card` at `targets` with mana to spare, leaving it on the
/// stack.
fn cast(game: &mut Game, card: GameObjectId, targets: &[Target]) {
    for color in [ManaColor::Blue, ManaColor::Red] {
        game.add_unrestricted_mana(PlayerId::One, color, 1);
    }
    let action = cast_action(game, card, targets).expect("the spell can be cast at those targets");
    game.apply(PlayerId::One, action).unwrap();
}

/// Player one casts a card with `definition` from hand and it resolves.
fn resolve_from_hand(game: &mut Game, definition: &str, targets: &[Target]) {
    let card = hand(game, CardDefinitionId::from_uuid(definition));
    cast(game, card, targets);
    drain_pending(game);
}

/// Runs one of player one's upkeeps, reporting how many abilities it put on
/// the stack or queued before anything resolved.
fn take_an_upkeep(game: &mut Game) -> usize {
    game.commit_next_turn(PlayerId::One, Vec::new());
    let triggered = game.stack.len() + game.pending_triggers.len();
    drain_pending(game);
    triggered
}

#[test]
fn an_upkeep_reprepare_trigger_prepares_its_creature_only_while_it_is_unprepared() {
    static ABILITIES: [AbilityDef; 1] = [sos::reprepare_at_upkeep()];
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bolt = rules_of(&game, cards::LIGHTNING_BOLT);
        add_definitions(
            &mut game,
            vec![prepare_card(
                UPKEEP_MAGE,
                &[
                    (
                        "Upkeep Mage",
                        CardRules::new_creature(mana_cost!("{1}{R}"), &["Wizard"], 2, 2)
                            .with_abilities(&ABILITIES),
                    ),
                    ("Lightning Bolt", bolt),
                ],
            )],
        );
        let mage = game
            .put_onto_battlefield(PlayerId::One, CardDefinitionId::from_uuid(UPKEEP_MAGE))
            .unwrap();
        assert!(!is_prepared(&game, mage), "nothing prepared it on entry");
        assert!(exiled_copies(&game, PlayerId::One).is_empty());

        assert_eq!(take_an_upkeep(&mut game), 1, "unprepared, so it triggers");
        assert!(is_prepared(&game, mage));
        let copies = exiled_copies(&game, PlayerId::One);
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].1, "Lightning Bolt");

        assert_eq!(
            take_an_upkeep(&mut game),
            0,
            "the intervening-if keeps a prepared creature from triggering"
        );
        assert_eq!(exiled_copies(&game, PlayerId::One), copies);
    }
}

#[test]
fn becoming_prepared_and_unprepared_follow_the_recipient_named_by_the_effect() {
    for prepared_engine in [false, true] {
        let mut game = game_with(
            prepared_engine,
            vec![
                instant(
                    READY,
                    "Ready",
                    "Target creature becomes prepared.",
                    &A_CREATURE,
                    sos::become_prepared(EffectRecipientDef::Target(TargetIndex::PRIMARY)),
                ),
                instant(
                    UNREADY,
                    "Unready",
                    "Target creature becomes unprepared.",
                    &A_CREATURE,
                    sos::become_unprepared(EffectRecipientDef::Target(TargetIndex::PRIMARY)),
                ),
            ],
        );
        let emeritus = game
            .put_onto_battlefield(PlayerId::One, cards::EMERITUS_OF_IDEATION)
            .unwrap();
        let bears = game
            .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
            .unwrap();
        assert!(is_prepared(&game, emeritus), "it enters prepared");

        resolve_from_hand(&mut game, UNREADY, &[Target::Permanent(emeritus)]);
        assert!(!is_prepared(&game, emeritus));
        assert!(
            exiled_copies(&game, PlayerId::One).is_empty(),
            "the copy ceases to exist with the designation"
        );

        resolve_from_hand(&mut game, UNREADY, &[Target::Permanent(emeritus)]);
        assert!(!is_prepared(&game, emeritus), "nothing to remove");

        resolve_from_hand(&mut game, READY, &[Target::Permanent(emeritus)]);
        assert!(is_prepared(&game, emeritus));
        let copies = exiled_copies(&game, PlayerId::One);
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].1, "Ancestral Recall");

        resolve_from_hand(&mut game, READY, &[Target::Permanent(emeritus)]);
        assert_eq!(
            exiled_copies(&game, PlayerId::One),
            copies,
            "a prepared creature cannot become prepared again"
        );

        resolve_from_hand(&mut game, READY, &[Target::Permanent(bears)]);
        assert!(
            !is_prepared(&game, bears),
            "a creature without a prepare spell cannot become prepared"
        );
        assert_eq!(exiled_copies(&game, PlayerId::One), copies);
    }
}

/// "The token copy of Goblin Glasswright will enter prepared as a 1/1 green
/// Frog creature, but the copy of Craft with Pride in exile will still be a
/// red sorcery." (CR 722.3c)
#[test]
fn a_token_copy_with_exceptions_still_prepares_the_printed_prepare_spell() {
    static FROG: CreatureTypeSetDef = CreatureTypeSetDef::named(&["Frog"]);
    static COUNTERPART: EffectDef =
        EffectDef::CreateToken(CreateTokenDef::new(TokenDef::Copy(&TokenCopyDef {
            object: &EffectRecipientDef::Target(TargetIndex::PRIMARY),
            exceptions: CopyExceptionsDef {
                base_power_toughness: Some((1, 1)),
                colors: Some(ColorSet::from_colors(&[ManaColor::Green])),
                replaced_creature_types: Some(&FROG),
                ..CopyExceptionsDef::NONE
            },
        })));
    for prepared_engine in [false, true] {
        let mut game = game_with(
            prepared_engine,
            vec![instant(
                FROG_COUNTERPART,
                "Frog Counterpart",
                "Create a token that's a copy of target creature, except it's a 1/1 green Frog.",
                &A_CREATURE,
                COUNTERPART,
            )],
        );
        game.players[0].library = game
            .build_zone(PlayerId::One, &[cards::FOREST; 10])
            .unwrap();
        let emeritus = game
            .put_onto_battlefield(PlayerId::One, cards::EMERITUS_OF_IDEATION)
            .unwrap();
        let original_copy = exiled_copies(&game, PlayerId::One)[0].0;

        resolve_from_hand(&mut game, FROG_COUNTERPART, &[Target::Permanent(emeritus)]);

        let token = game
            .battlefield
            .iter()
            .find(|permanent| permanent.card.id != emeritus)
            .expect("the token copy entered");
        let frog = game.trigger_event_object(token);
        assert!(frog.token);
        assert_eq!((frog.power, frog.toughness), (Some(1), Some(1)));
        assert_eq!(frog.colors, [false, false, false, false, true], "green");
        let token = token.card.id;
        assert!(is_prepared(&game, token), "the copied clause prepared it");

        let copies = exiled_copies(&game, PlayerId::One);
        assert_eq!(copies.len(), 2);
        let (token_copy, name) = copies
            .iter()
            .find(|(id, _)| *id != original_copy)
            .cloned()
            .unwrap();
        assert_eq!(name, "Ancestral Recall");
        let spell = exiled_view(&game, token_copy);
        assert!(spell.types.contains(CardType::Instant));
        assert!(!spell.types.contains(CardType::Creature));
        assert_eq!(spell.colors, [false, true, false, false, false], "blue");
        assert_eq!(spell.subtypes, crate::card::SubtypeSet::EMPTY);
        assert_eq!(spell.mana_value, 1);
        assert_eq!((spell.power, spell.toughness), (None, None));

        cast(&mut game, token_copy, &[Target::Player(PlayerId::One)]);
        assert!(
            !is_prepared(&game, token),
            "casting it unprepares the token"
        );
        assert!(is_prepared(&game, emeritus), "and only the token");
        let hand_before = game.players[0].hand.len();
        drain_pending(&mut game);
        assert_eq!(game.players[0].hand.len(), hand_before + 3);
    }
}

/// "The same is true if a prepared creature becomes a copy of something
/// else, even if it's then a permanent without a prepare spell."
#[test]
fn a_prepared_permanent_that_becomes_a_copy_of_something_else_stays_prepared() {
    for prepared_engine in [false, true] {
        let mut game = game_with(
            prepared_engine,
            vec![instant(
                SHAPESHIFT,
                "Shapeshift",
                "Target creature becomes a copy of another target creature.",
                &TWO_CREATURES,
                EffectDef::BecomeCopyOf {
                    object: EffectRecipientDef::Target(TargetIndex(1)),
                    copier: Some(EffectRecipientDef::Target(TargetIndex::PRIMARY)),
                    exceptions: &CopyExceptionsDef::NONE,
                    duration: None,
                },
            )],
        );
        game.players[0].library = game
            .build_zone(PlayerId::One, &[cards::FOREST; 10])
            .unwrap();
        let emeritus = game
            .put_onto_battlefield(PlayerId::One, cards::EMERITUS_OF_IDEATION)
            .unwrap();
        let bears = game
            .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
            .unwrap();
        let copies = exiled_copies(&game, PlayerId::One);

        resolve_from_hand(
            &mut game,
            SHAPESHIFT,
            &[Target::Permanent(emeritus), Target::Permanent(bears)],
        );

        let shifted = game
            .battlefield
            .iter()
            .find(|permanent| permanent.card.id == emeritus)
            .unwrap();
        assert_eq!(
            game.object_card_name(emeritus).as_deref(),
            Some("Grizzly Bears")
        );
        assert_eq!(
            (game.power(shifted), game.toughness(shifted)),
            (Some(2), Some(2))
        );
        assert!(is_prepared(&game, emeritus));
        assert_eq!(exiled_copies(&game, PlayerId::One), copies);

        cast(&mut game, copies[0].0, &[Target::Player(PlayerId::One)]);
        assert!(!is_prepared(&game, emeritus));
        let hand_before = game.players[0].hand.len();
        drain_pending(&mut game);
        assert_eq!(game.players[0].hand.len(), hand_before + 3);
        assert!(exiled_copies(&game, PlayerId::One).is_empty());
    }
}

/// "If a prepared creature loses all abilities, it won't stop being
/// prepared, and nothing will happen to its alternative characteristics or
/// to the copy of its prepare spell in exile."
#[test]
fn a_prepared_permanent_that_loses_all_abilities_stays_prepared() {
    for prepared_engine in [false, true] {
        let mut game = game_with(
            prepared_engine,
            vec![instant(
                HUSH,
                "Test Hush",
                "Target creature loses all abilities.",
                &A_CREATURE,
                EffectDef::Apply {
                    recipient: EffectRecipientDef::Target(TargetIndex::PRIMARY),
                    effect: AppliedEffectDef::remove_abilities(AbilityPredicateDef::Any),
                    duration: ResolvedEffectDurationDef::Permanent,
                },
            )],
        );
        game.players[0].library = game
            .build_zone(PlayerId::One, &[cards::FOREST; 10])
            .unwrap();
        let emeritus = game
            .put_onto_battlefield(PlayerId::One, cards::EMERITUS_OF_IDEATION)
            .unwrap();
        let copies = exiled_copies(&game, PlayerId::One);

        resolve_from_hand(&mut game, HUSH, &[Target::Permanent(emeritus)]);

        let hushed = game
            .battlefield
            .iter()
            .find(|permanent| permanent.card.id == emeritus)
            .unwrap();
        assert!(game.effective_abilities(hushed).is_empty());
        assert!(is_prepared(&game, emeritus));
        assert_eq!(exiled_copies(&game, PlayerId::One), copies);
        assert_eq!(copies[0].1, "Ancestral Recall");

        cast(&mut game, copies[0].0, &[Target::Player(PlayerId::One)]);
        assert!(!is_prepared(&game, emeritus));
        let hand_before = game.players[0].hand.len();
        drain_pending(&mut game);
        assert_eq!(game.players[0].hand.len(), hand_before + 3);
    }
}

/// "If a prepare spell with one or more targets has no legal targets when it
/// tries to resolve, it won't resolve and none of its effects will happen.
/// Since the spell was cast, the associated permanent will still not be
/// prepared."
#[test]
fn a_prepare_spell_that_fizzles_leaves_its_permanent_unprepared() {
    static JOLT: [EffectDef; 2] = [
        EffectDef::damage(
            EffectRecipientDef::Target(TargetIndex::PRIMARY),
            ValueDef::Constant(1),
        ),
        EffectDef::DrawCards {
            recipient: EffectRecipientDef::Controller,
            amount: ValueDef::Constant(1),
        },
    ];
    for prepared_engine in [false, true] {
        let mut game = game_with(
            prepared_engine,
            vec![mage_entering_prepared(
                JOLT_MAGE,
                "Jolt Mage",
                "Test Jolt",
                CardRules::new_instant(mana_cost!("{R}")).with_ability(
                    AbilityDef::spell_with_targets(
                        "Test Jolt deals 1 damage to target creature. Draw a card.",
                        &A_CREATURE,
                        EffectDef::Sequence(&JOLT),
                    ),
                ),
            )],
        );
        let mage = game
            .put_onto_battlefield(PlayerId::One, CardDefinitionId::from_uuid(JOLT_MAGE))
            .unwrap();
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        let copy = exiled_copies(&game, PlayerId::One)[0].0;

        cast(&mut game, copy, &[Target::Permanent(bears)]);
        assert!(
            !is_prepared(&game, mage),
            "it is unprepared as the spell becomes cast"
        );
        game.sacrifice_permanent(bears);
        let hand_before = game.players[0].hand.len();
        drain_pending(&mut game);

        assert!(game.stack.is_empty());
        assert_eq!(
            game.players[0].hand.len(),
            hand_before,
            "none of its effects happen"
        );
        assert!(!is_prepared(&game, mage));
        assert!(exiled_copies(&game, PlayerId::One).is_empty());
        assert!(
            game.players[0].graveyard.is_empty(),
            "a copy that leaves the stack ceases to exist"
        );
        assert_eq!(game.prepared_spell_copies, []);
    }
}

/// Five Reality Fracture prepare spells are each printed on three cards, so
/// one player routinely holds same-named copies made by different permanents.
#[test]
fn two_same_named_prepare_spell_copies_coexist_in_exile_and_belong_to_their_own_permanents() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let recall = rules_of(&game, cards::ANCESTRAL_RECALL);
        add_definitions(
            &mut game,
            vec![mage_entering_prepared(
                RECALL_STUDENT,
                "Recall Student",
                "Ancestral Recall",
                recall,
            )],
        );
        game.players[0].library = game
            .build_zone(PlayerId::One, &[cards::FOREST; 10])
            .unwrap();
        let emeritus = game
            .put_onto_battlefield(PlayerId::One, cards::EMERITUS_OF_IDEATION)
            .unwrap();
        let emeritus_copy = exiled_copies(&game, PlayerId::One)[0].0;
        let student = game
            .put_onto_battlefield(PlayerId::One, CardDefinitionId::from_uuid(RECALL_STUDENT))
            .unwrap();

        let copies = exiled_copies(&game, PlayerId::One);
        assert_eq!(
            copies
                .iter()
                .map(|(_, name)| name.as_str())
                .collect::<Vec<_>>(),
            ["Ancestral Recall", "Ancestral Recall"]
        );
        let student_copy = copies[1].0;
        assert_ne!(student_copy, emeritus_copy);
        let (wire, hidden) = checkpoint_fixture(&game, PlayerId::One);
        let mut rebuilt =
            Game::from_observation_checkpoint(game.catalog.clone(), game.format, &wire, &hidden, 0)
                .expect("a checkpoint may hold two same-named copies");
        rebuilt.set_prepared_engine_enabled(prepared_engine);

        for state in [&mut game, &mut rebuilt] {
            for copy in [emeritus_copy, student_copy] {
                assert!(cast_action(state, copy, &[Target::Player(PlayerId::One)]).is_none());
            }
            state.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 1);
            for copy in [emeritus_copy, student_copy] {
                assert!(cast_action(state, copy, &[Target::Player(PlayerId::One)]).is_some());
            }
            state.players[0].mana_pool = ManaPool::default();
            state.players[0].mana.clear();

            cast(state, student_copy, &[Target::Player(PlayerId::One)]);
            assert!(!is_prepared(state, student));
            assert!(is_prepared(state, emeritus));
            let hand_before = state.players[0].hand.len();
            drain_pending(state);
            assert_eq!(state.players[0].hand.len(), hand_before + 3);
            assert_eq!(
                exiled_copies(state, PlayerId::One),
                [(emeritus_copy, "Ancestral Recall".to_string())]
            );
            state.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 1);
            assert!(cast_action(state, emeritus_copy, &[Target::Player(PlayerId::One)]).is_some());
        }
    }
}

/// "As a permanent with a prepare spell ... phases in prepared, its
/// controller creates a copy of that object in exile" (CR 722.3c).
#[test]
fn a_permanent_that_phases_in_prepared_gets_a_new_copy_of_its_prepare_spell() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let emeritus = game
            .put_onto_battlefield(PlayerId::One, cards::EMERITUS_OF_IDEATION)
            .unwrap();
        let original_copy = exiled_copies(&game, PlayerId::One)[0].0;

        game.phase_out(emeritus);
        assert!(
            exiled_copies(&game, PlayerId::One).is_empty(),
            "the copy lasts only while its permanent is on the battlefield"
        );
        assert_eq!(
            game.phased_out[0].designations,
            [PermanentDesignationDef::Prepared],
            "phasing out keeps the designation"
        );

        game.phase_in_for(PlayerId::One);
        assert!(is_prepared(&game, emeritus));
        let copies = exiled_copies(&game, PlayerId::One);
        assert_eq!(copies.len(), 1);
        assert_ne!(copies[0].0, original_copy, "a new object, not the old one");
        assert_eq!(copies[0].1, "Ancestral Recall");
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 1);
        assert!(cast_action(&game, copies[0].0, &[Target::Player(PlayerId::One)]).is_some());
    }
}
