//! Empower Jace (CR 701.71) and the Reality Fracture cards built on it.
//!
//! "If you don't control a Jace planeswalker token, create a blue Jace
//! planeswalker token with 0 loyalty, '[−1]: Surveil 1,' and '[−3]: Draw a
//! card.' Choose a Jace planeswalker token you control. Put N loyalty
//! counters on it."
use super::woe_hob_completion::{hand, setup};
use super::*;
use crate::card::Subtype;

const JACE: Subtype = Subtype::from_name("Jace").expect("Jace is a planeswalker type");

/// Player one's Jace planeswalker tokens and the loyalty on each, in
/// battlefield order.
fn jace_tokens(game: &Game) -> Vec<(GameObjectId, u16)> {
    game.battlefield
        .iter()
        .filter(|permanent| {
            permanent.controller == PlayerId::One
                && permanent.card.definition.is_token()
                && game
                    .permanent_types(permanent)
                    .is_some_and(|types| types.contains(CardType::Planeswalker))
                && game.effective_subtypes(permanent).contains(JACE)
        })
        .map(|permanent| (permanent.card.id, permanent.counters(CounterKind::Loyalty)))
        .collect()
}

/// Player one's only Jace token, created by whatever just empowered Jace.
fn the_jace_token(game: &Game) -> GameObjectId {
    let tokens = jace_tokens(game);
    assert_eq!(tokens.len(), 1);
    tokens[0].0
}

pub(super) fn permanent(game: &Game, id: GameObjectId) -> &Permanent {
    game.battlefield
        .iter()
        .find(|permanent| permanent.card.id == id)
        .expect("the permanent is on the battlefield")
}

pub(super) fn permanent_mut(game: &mut Game, id: GameObjectId) -> &mut Permanent {
    game.battlefield
        .iter_mut()
        .find(|permanent| permanent.card.id == id)
        .expect("the permanent is on the battlefield")
}

/// The loyalty counters on the planeswalker `id`.
fn loyalty(game: &Game, id: GameObjectId) -> u16 {
    permanent(game, id).counters(CounterKind::Loyalty)
}

/// A Jace planeswalker token no Empower card made, already holding `loyalty`.
fn another_jace_token(game: &mut Game, loyalty: u16) -> GameObjectId {
    static OTHER_JACE: TokenCharacteristics = TokenCharacteristics::new(
        CardTypeSet::single(CardType::Planeswalker),
        &["Jace"],
        &[ManaColor::Blue],
        None,
    );
    let token = game.create_token_from(PlayerId::One, OTHER_JACE, None);
    permanent_mut(game, token).add_counters(CounterKind::Loyalty, loyalty);
    token
}

/// Gives player one a library deep enough to draw and surveil from.
pub(super) fn stock_library(game: &mut Game) {
    game.players[0].library = game
        .build_zone(PlayerId::One, &[cards::FOREST; 10])
        .unwrap();
}

/// Puts a card with `definition` onto the battlefield under player one as
/// though it had been there since an earlier turn, resolving whatever its
/// arrival triggers.
pub(super) fn settled(game: &mut Game, definition: CardDefinitionId) -> GameObjectId {
    let id = game
        .put_onto_battlefield(PlayerId::One, definition)
        .expect("cataloged");
    permanent_mut(game, id).entered_controller_turn = 0;
    drain_pending(game);
    id
}

/// Player one casts a card with `definition` from hand at `targets` with mana
/// to spare, leaving it on the stack.
pub(super) fn cast(game: &mut Game, definition: CardDefinitionId, targets: &[Target]) {
    let card = hand(game, definition);
    for color in ManaColor::COLORS {
        game.add_unrestricted_mana(PlayerId::One, color, 6);
    }
    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| {
            matches!(action, Action::CastSpell { card: cast, choices, .. }
            if *cast == card && choices.iter_targets().eq(targets))
        })
        .expect("the spell can be cast at those targets");
    game.apply(PlayerId::One, action).unwrap();
}

/// Player one's legal ways to cast `card`.
pub(super) fn casts(game: &Game, card: GameObjectId) -> Vec<Action> {
    game.legal_actions(PlayerId::One)
        .into_iter()
        .filter(|action| matches!(action, Action::CastSpell { card: cast, .. } if *cast == card))
        .collect()
}

/// Player one puts a Lightning Bolt of their own on the stack, aimed at the
/// opponent, for something to counter.
fn bolt_on_the_stack(game: &mut Game) -> GameObjectId {
    let bolt = hand(game, cards::LIGHTNING_BOLT);
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);
    let action = casts(game, bolt).remove(0);
    game.apply(PlayerId::One, action).unwrap();
    bolt
}

/// Player one's legal activations of `source`'s ability whose text starts
/// with `prefix`, one per way of choosing its targets.
pub(super) fn activations(game: &Game, source: GameObjectId, prefix: &str) -> Vec<Action> {
    let permanent = permanent(game, source);
    game.legal_actions(PlayerId::One)
        .into_iter()
        .filter(|action| {
            let Action::ActivateAbility {
                source: activated,
                ability,
                ..
            } = action
            else {
                return false;
            };
            let mut named = false;
            game.for_each_effective_ability(permanent, |effective| {
                named |= effective.origin == *ability && effective.ability.text.starts_with(prefix);
            });
            *activated == source && named
        })
        .collect()
}

/// Player one activates `source`'s ability whose text starts with `prefix`,
/// leaving it on the stack.
pub(super) fn activate(game: &mut Game, source: GameObjectId, prefix: &str) {
    let action = activations(game, source, prefix)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{prefix} can be activated"));
    game.apply(PlayerId::One, action).unwrap();
}

/// Whether a decision option stands for the object `id`.
fn offers(option: &DecisionOption, id: GameObjectId) -> bool {
    option.card.as_ref().is_some_and(|(card, _)| *card == id)
}

/// Answers player one's pending decision with every option `wanted` picks.
fn answer(game: &mut Game, wanted: impl Fn(&DecisionOption) -> bool) {
    let decision = game
        .observe(PlayerId::One)
        .decision
        .expect("player one is being asked");
    let options = decision
        .options
        .iter()
        .filter(|option| wanted(option))
        .map(|option| option.id)
        .collect::<Vec<_>>();
    assert!(!options.is_empty(), "the wanted answer is offered");
    game.apply(
        PlayerId::One,
        Action::ChooseDecision {
            decision: decision.id,
            options,
        },
    )
    .unwrap();
}

/// Every permanent that has left the battlefield so far, in order.
fn departed(game: &Game) -> Vec<GameObjectId> {
    game.events()
        .iter()
        .filter_map(|event| match event {
            GameEvent::PermanentLeftBattlefield { card, .. } => Some(*card),
            _ => None,
        })
        .collect()
}

/// The opponent's library as card ids, bottom first.
fn opposing_library(game: &Game) -> Vec<GameObjectId> {
    game.players[1].library.iter().map(|card| card.id).collect()
}

/// The game rebuilt from what player one can see of it, in the same engine
/// mode.
pub(super) fn reconstructed(game: &Game) -> Game {
    let (wire, hidden) = checkpoint_fixture(game, PlayerId::One);
    let mut rebuilt =
        Game::from_observation_checkpoint(game.catalog.clone(), game.format, &wire, &hidden, 7)
            .expect("the board reconstructs from a checkpoint");
    rebuilt.set_prepared_engine_enabled(game.prepared_engine_enabled());
    rebuilt
}

#[test]
fn empowering_jace_with_no_jace_token_creates_one_holding_that_much_loyalty() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);

        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        let hand_before = game.players[0].hand.len();
        drain_pending(&mut game);

        let tokens = jace_tokens(&game);
        assert_eq!(tokens.len(), 1, "the token it had to create first");
        assert_eq!(tokens[0].1, 6, "empower Jace 6");
        assert_eq!(game.players[0].hand.len(), hand_before + 1, "draw a card");
    }
}

/// "If you control a Jace planeswalker token, you can't choose to create a
/// new one when you empower Jace."
#[test]
fn empowering_jace_again_adds_to_the_token_you_already_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);

        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        drain_pending(&mut game);
        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        drain_pending(&mut game);

        let tokens = jace_tokens(&game);
        assert_eq!(tokens.len(), 1, "no second token");
        assert_eq!(tokens[0].1, 12, "six, then six more on the same token");
    }
}

/// "Empower Jace won't let you put loyalty counters on nontoken Jace
/// planeswalkers."
#[test]
fn empowering_jace_never_chooses_a_nontoken_jace() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let nontoken = game
            .put_onto_battlefield(PlayerId::One, cards::JACE_THE_MIND_SCULPTOR)
            .unwrap();

        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        drain_pending(&mut game);

        assert_eq!(loyalty(&game, nontoken), 3, "his printed three, untouched");
        let tokens = jace_tokens(&game);
        assert_eq!(tokens.len(), 1, "he is not a token, so one is created");
        assert_eq!(tokens[0].1, 6);
    }
}

/// "In the rare case that you control multiple Jace planeswalker tokens,
/// you'll choose one of those tokens to put the loyalty counters on."
#[test]
fn empowering_jace_offers_every_jace_token_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let first = another_jace_token(&mut game, 1);
        let second = another_jace_token(&mut game, 2);
        let nontoken = game
            .put_onto_battlefield(PlayerId::One, cards::JACE_THE_MIND_SCULPTOR)
            .unwrap();

        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        pass_until_decision(&mut game);
        let decision = game
            .observe(PlayerId::One)
            .decision
            .expect("which Jace token is a choice");
        let offered = decision
            .options
            .iter()
            .filter_map(|option| option.card.as_ref().map(|(id, _)| *id))
            .collect::<Vec<_>>();
        assert_eq!(offered, [first, second], "and not the nontoken Jace");
        assert_eq!((decision.minimum, decision.maximum), (1, 1));
        answer(&mut game, |option| offers(option, second));
        drain_pending(&mut game);

        assert_eq!(jace_tokens(&game), [(first, 1), (second, 8)]);
        assert_eq!(loyalty(&game, nontoken), 3);
    }
}

/// "Empower Jace will put loyalty counters on" a Jace planeswalker token
/// "that you got another way (such as an effect that creates a token that's
/// a copy of a permanent)".
#[test]
fn empowering_jace_chooses_a_token_copy_of_a_nontoken_jace() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let copy = game.create_token_copy(
            PlayerId::One,
            copied_characteristics(cards::JACE_MEMORY_ADEPT),
            None,
            CardPartId::PRIMARY,
        );
        permanent_mut(&mut game, copy).add_counters(CounterKind::Loyalty, 4);

        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        drain_pending(&mut game);

        assert_eq!(jace_tokens(&game), [(copy, 10)]);
    }
}

/// Jace, Reality Sculptor counts Islands, and with none he empowers Jace 0:
/// the token is still created, with no loyalty to keep it (CR 704.5i).
#[test]
fn empowering_jace_zero_creates_a_token_that_state_based_actions_remove() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let sculptor = settled(&mut game, cards::JACE_REALITY_SCULPTOR);
        settled(&mut game, cards::FOREST);

        activate(&mut game, sculptor, "+1:");
        drain_pending(&mut game);

        assert_eq!(loyalty(&game, sculptor), 6, "the plus was paid");
        assert!(jace_tokens(&game).is_empty(), "nothing holds it there");
        let gone = departed(&game);
        assert_eq!(gone.len(), 1, "one permanent came and went");
        assert_ne!(gone[0], sculptor);
    }
}

/// "If each target chosen is an illegal target as that spell or ability
/// tries to resolve, it won't resolve. You won't empower Jace." No Admittance
/// prints its damage and its Empower as separate clauses of one spell.
#[test]
fn a_spell_whose_only_target_is_gone_does_not_empower_jace() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::NO_ADMITTANCE, &[Target::Permanent(bears)]);
        cast(
            &mut game,
            cards::LIGHTNING_BOLT,
            &[Target::Permanent(bears)],
        );
        drain_pending(&mut game);

        assert!(game.battlefield.is_empty(), "the Bolt got there first");
        assert!(departed(&game) == [bears], "and no Jace token came or went");
    }
}

#[test]
fn jace_reality_sculptor_empowers_for_each_island_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let sculptor = settled(&mut game, cards::JACE_REALITY_SCULPTOR);
        for land in [cards::ISLAND, cards::ISLAND, cards::FOREST] {
            settled(&mut game, land);
        }
        game.put_onto_battlefield(PlayerId::Two, cards::ISLAND)
            .unwrap();

        activate(&mut game, sculptor, "+1:");
        drain_pending(&mut game);

        let tokens = jace_tokens(&game);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].1, 2, "your two Islands, not the opponent's");
    }
}

/// "Activate only if there are twenty-five or more loyalty counters among
/// Jaces you control": his own and every other Jace's, token or not.
#[test]
fn jace_reality_sculptor_needs_twenty_five_loyalty_among_your_jaces() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let sculptor = settled(&mut game, cards::JACE_REALITY_SCULPTOR);
        permanent_mut(&mut game, sculptor).add_counters(CounterKind::Loyalty, 15);
        let token = another_jace_token(&mut game, 4);
        assert_eq!(loyalty(&game, sculptor) + loyalty(&game, token), 24);
        assert!(activations(&game, sculptor, "0:").is_empty());

        permanent_mut(&mut game, token).add_counters(CounterKind::Loyalty, 1);
        assert_eq!(activations(&game, sculptor, "0:").len(), 1);
    }
}

#[test]
fn jace_reality_sculptor_exiles_all_but_the_bottom_card_of_a_library() {
    for prepared_engine in [false, true] {
        for size in [0, 1, 4] {
            let mut game = setup(prepared_engine);
            let sculptor = settled(&mut game, cards::JACE_REALITY_SCULPTOR);
            permanent_mut(&mut game, sculptor).add_counters(CounterKind::Loyalty, 20);
            game.players[1].library = game
                .build_zone(PlayerId::Two, &vec![cards::FOREST; size])
                .unwrap();
            let before = opposing_library(&game);

            activate(&mut game, sculptor, "0:");
            drain_pending(&mut game);

            let kept = before.iter().take(1).copied().collect::<Vec<_>>();
            assert_eq!(opposing_library(&game), kept, "a library of {size}");
            assert_eq!(
                game.players[1].exile.len(),
                size.saturating_sub(1),
                "a library of {size}"
            );
            assert_eq!(loyalty(&game, sculptor), 25, "a zero costs nothing");
            assert!(game.result().is_none(), "nobody has had to draw yet");
        }
    }
}

/// Empower Jace from a source that is in no zone "you" control anything from:
/// Campus Crier is in exile by the time its ability asks which tokens are
/// yours.
#[test]
fn campus_crier_empowers_from_the_graveyard_by_exiling_itself() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let crier = game
            .build_zone(PlayerId::One, &[cards::CAMPUS_CRIER])
            .unwrap()
            .remove(0);
        let id = crier.id;
        game.players[0].graveyard.push(crier);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 1);

        let action = game
            .legal_actions(PlayerId::One)
            .into_iter()
            .find(
                |action| matches!(action, Action::ActivateAbility { source, .. } if *source == id),
            )
            .expect("it can be activated from the graveyard");
        game.apply(PlayerId::One, action).unwrap();
        assert!(game.players[0].graveyard.is_empty(), "exiled as a cost");
        assert_eq!(game.players[0].exile.len(), 1);
        drain_pending(&mut game);

        let tokens = jace_tokens(&game);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].1, 2);
    }
}

/// How Countersculpt's additional cost is met.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Payment {
    Mana,
    JaceToken,
    JaceCardInHand,
}

/// "As an additional cost to cast this spell, behold a Jace or pay {1}."
#[test]
fn countersculpt_is_paid_for_by_beholding_a_jace_or_with_one_more_mana() {
    for prepared_engine in [false, true] {
        for payment in [Payment::Mana, Payment::JaceToken, Payment::JaceCardInHand] {
            let mut game = setup(prepared_engine);
            let life = game.players[1].life;
            bolt_on_the_stack(&mut game);
            let counter = hand(&mut game, cards::COUNTERSCULPT);
            game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 2);
            assert!(
                casts(&game, counter).is_empty(),
                "no Jace to behold and nothing for the {{1}}"
            );

            let mut token = None;
            match payment {
                Payment::Mana => game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 1),
                Payment::JaceToken => token = Some(another_jace_token(&mut game, 3)),
                Payment::JaceCardInHand => {
                    hand(&mut game, cards::JACE_MEMORY_ADEPT);
                }
            }
            let ways = casts(&game, counter);
            assert_eq!(ways.len(), 1, "exactly the one way to pay: {payment:?}");
            game.apply(PlayerId::One, ways[0].clone()).unwrap();
            drain_pending(&mut game);

            assert_eq!(game.players[0].mana_pool.total(), 0, "{payment:?}");
            assert_eq!(game.players[1].life, life, "the Bolt was countered");
            let tokens = jace_tokens(&game);
            assert_eq!(tokens.len(), 1, "{payment:?}");
            match token {
                Some(token) => assert_eq!(tokens[0], (token, 4), "beheld, then empowered"),
                None => assert_eq!(tokens[0].1, 1, "the one it had to create"),
            }
            let revealed = payment == Payment::JaceCardInHand;
            assert_eq!(
                game.players[0].hand.len(),
                usize::from(revealed),
                "a revealed card stays in hand"
            );
        }
    }
}

/// "Whenever you cast your first noncreature spell each turn": decided as
/// the spell is cast, and only once a turn.
#[test]
fn plan_for_all_outcomes_empowers_for_the_first_noncreature_spell_of_a_turn_only() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::PLAN_FOR_ALL_OUTCOMES);

        cast(&mut game, cards::GRIZZLY_BEARS, &[]);
        drain_pending(&mut game);
        assert!(
            jace_tokens(&game).is_empty(),
            "a creature spell does not count"
        );

        // The first noncreature spell, answered by a second before the trigger resolves.
        let opponent = [Target::Player(PlayerId::Two)];
        cast(&mut game, cards::LIGHTNING_BOLT, &opponent);
        cast(&mut game, cards::LIGHTNING_BOLT, &opponent);
        drain_pending(&mut game);
        let tokens = jace_tokens(&game);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].1, 1, "the first triggered and the second did not");

        cast(&mut game, cards::LIGHTNING_BOLT, &opponent);
        drain_pending(&mut game);
        assert_eq!(jace_tokens(&game), tokens, "nor does a third");
    }
}

/// "The turn you cast Plan for All Outcomes, its second ability won't
/// trigger because you've already cast at least one noncreature spell."
#[test]
fn plan_for_all_outcomes_counts_itself_as_a_noncreature_spell_cast_this_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        cast(&mut game, cards::PLAN_FOR_ALL_OUTCOMES, &[]);
        drain_pending(&mut game);

        cast(
            &mut game,
            cards::LIGHTNING_BOLT,
            &[Target::Player(PlayerId::Two)],
        );
        drain_pending(&mut game);

        assert!(jace_tokens(&game).is_empty());
    }
}

/// "Only one loyalty ability of each planeswalker may be activated per turn,
/// no matter how many it has gained." A granted loyalty ability also keeps
/// loyalty timing (CR 606.3).
#[test]
fn a_granted_loyalty_ability_shares_its_planeswalkers_one_activation_each_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let adept = settled(&mut game, cards::JACE_MEMORY_ADEPT);
        settled(&mut game, cards::WAY_OF_THE_WILDSPEAKER);
        let token = the_jace_token(&game);
        assert_eq!((loyalty(&game, adept), loyalty(&game, token)), (4, 7));
        for walker in [adept, token] {
            assert_eq!(activations(&game, walker, "−4:").len(), 1, "granted");
        }
        assert!(!activations(&game, adept, "+1:").is_empty(), "printed");
        assert_eq!(activations(&game, token, "−3:").len(), 1, "the token's own");

        game.step = Step::Upkeep;
        assert!(
            activations(&game, token, "−4:").is_empty(),
            "main phase only"
        );
        game.step = Step::PrecombatMain;

        // Granted first: the token's own abilities are spent with it.
        activate(&mut game, token, "−4:");
        assert!(
            activations(&game, adept, "−4:").is_empty(),
            "the stack is not empty"
        );
        drain_pending(&mut game);
        assert_eq!(loyalty(&game, token), 3);
        assert!(
            game.battlefield
                .iter()
                .any(|permanent| game.power(permanent) == Some(4)),
            "the Beast it made"
        );
        for spent in ["−4:", "−3:", "−1:"] {
            assert!(activations(&game, token, spent).is_empty(), "{spent}");
        }

        // Printed first: the other planeswalker's granted ability goes with it.
        assert_eq!(
            activations(&game, adept, "−4:").len(),
            1,
            "a separate allowance"
        );
        let plus = activations(&game, adept, "+1:").remove(0);
        game.apply(PlayerId::One, plus).unwrap();
        drain_pending(&mut game);
        assert!(activations(&game, adept, "−4:").is_empty());
    }
}

/// Avatar of Burgeoning Echoes: landfall empowers, and the ability it grants
/// counts the lands of whoever controls the planeswalker using it.
#[test]
fn avatar_of_burgeoning_echoes_grants_a_counter_for_each_land_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let avatar = settled(&mut game, cards::AVATAR_OF_BURGEONING_ECHOES);
        for _ in 0..3 {
            settled(&mut game, cards::FOREST);
        }
        game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
            .unwrap();
        let token = the_jace_token(&game);
        assert_eq!(loyalty(&game, token), 6, "landfall three times");
        assert!(activations(&game, token, "−10:").is_empty());

        permanent_mut(&mut game, token).add_counters(CounterKind::Loyalty, 5);
        let action = activations(&game, token, "−10:")
            .into_iter()
            .find(|action| {
                matches!(action, Action::ActivateAbility { targets, .. }
                    if targets.iter().any(|slot| slot.targets() == [Target::Permanent(avatar)]))
            })
            .expect("the Avatar is a creature to aim at");
        game.apply(PlayerId::One, action).unwrap();
        drain_pending(&mut game);

        let counters = permanent(&game, avatar).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(counters, 3, "your three lands, not the opponent's");
        assert_eq!(loyalty(&game, token), 1);
    }
}

/// "Loyalty abilities can't be mana abilities. The ability granted by Way of
/// the Pyromancer uses the stack and can be countered or otherwise responded
/// to."
#[test]
fn way_of_the_pyromancer_grants_a_loyalty_ability_that_uses_the_stack() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::WAY_OF_THE_PYROMANCER);
        let token = the_jace_token(&game);

        activate(&mut game, token, "+1:");
        assert_eq!(game.stack.len(), 1, "it waits to resolve");
        assert_eq!(game.players[0].mana_pool.total(), 0);
        assert_eq!(
            loyalty(&game, token),
            3,
            "two from the Way, one for the cost"
        );
        drain_pending(&mut game);

        assert_eq!(game.players[0].mana_pool.amount(ManaColor::Red), 1);
        assert!(
            activations(&game, token, "−1:").is_empty(),
            "that was this turn's"
        );
    }
}

/// "This planeswalker deals 2 damage to up to one target creature or
/// planeswalker and 2 damage to target player."
#[test]
fn way_of_the_warlord_grants_damage_to_an_optional_permanent_and_a_player() {
    for prepared_engine in [false, true] {
        for both in [false, true] {
            let mut game = setup(prepared_engine);
            settled(&mut game, cards::WAY_OF_THE_WARLORD);
            let token = the_jace_token(&game);
            let bears = game
                .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
                .unwrap();
            let life = game.players[1].life;
            let mut wanted = vec![Target::Player(PlayerId::Two)];
            if both {
                wanted.insert(0, Target::Permanent(bears));
            }

            let action = activations(&game, token, "−4:")
                .into_iter()
                .find(|action| {
                    matches!(action, Action::ActivateAbility { targets, .. }
                        if targets.iter().flat_map(TargetSelection::targets).eq(&wanted))
                })
                .expect("the creature or planeswalker is optional");
            game.apply(PlayerId::One, action).unwrap();
            drain_pending(&mut game);

            assert_eq!(game.players[1].life, life - 2, "the player either way");
            let survived = game.battlefield.iter().any(|p| p.card.id == bears);
            assert_eq!(survived, !both, "the creature only when it was aimed at");
            assert_eq!(loyalty(&game, token), 1);
        }
    }
}

/// The Jace token and a loyalty ability granted to it both come back from a
/// checkpoint, and a different Empower card still finds that same token.
#[test]
fn the_jace_token_and_its_granted_ability_survive_a_checkpoint() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        drain_pending(&mut game);
        settled(&mut game, cards::WAY_OF_THE_WILDSPEAKER);
        let token = the_jace_token(&game);
        assert_eq!(loyalty(&game, token), 13);

        let mut game = reconstructed(&game);

        assert_eq!(jace_tokens(&game), [(token, 13)]);
        for ability in ["−1:", "−3:", "−4:"] {
            assert_eq!(activations(&game, token, ability).len(), 1, "{ability}");
        }
        cast(&mut game, cards::MINDSEEKER_OCULUS, &[]);
        drain_pending(&mut game);
        assert_eq!(
            jace_tokens(&game),
            [(token, 17)],
            "the same token, not a new one"
        );
    }
}

/// Empower Jace fixes its amount before asking which token, and both have to
/// outlast a checkpoint taken while the choice is pending.
#[test]
fn a_pending_empower_choice_survives_a_checkpoint() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        drain_pending(&mut game);
        let token = the_jace_token(&game);
        let copy = game.create_token_copy(
            PlayerId::One,
            copied_characteristics(cards::JACE_MEMORY_ADEPT),
            None,
            CardPartId::PRIMARY,
        );
        permanent_mut(&mut game, copy).add_counters(CounterKind::Loyalty, 4);

        cast(&mut game, cards::PROTEGE_S_AWAKENING, &[]);
        pass_until_decision(&mut game);
        let mut game = reconstructed(&game);
        answer(&mut game, |option| offers(option, copy));
        drain_pending(&mut game);

        assert_eq!(jace_tokens(&game), [(token, 6), (copy, 10)]);
    }
}
