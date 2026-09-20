//! Inti, Seneschal of the Sun: two clauses that feed each other, since the
//! discard one asks for is the discard the other is watching for.
//!
//! "When you do" is a reflexive trigger made by the card actually discarded.
//! It names its attacking creature only on the way to the stack, after the
//! discard, and it does not need Inti to still be on the battlefield.

use super::*;

/// Inti and a bear ready to attack, with cards in hand to pitch.
fn staged(hand: usize, library: usize) -> (Game, GameObjectId, GameObjectId) {
    let mut game = ready_game();
    game.battlefield.clear();
    game.players[0].hand.clear();
    game.players[0].library.clear();
    for index in 0..hand {
        game.players[0].hand.push(card(
            97_000 + u32::try_from(index).expect("a small hand"),
            cards::MOUNTAIN,
            PlayerId::One,
        ));
    }
    for index in 0..library {
        game.players[0].library.push(card(
            97_100 + u32::try_from(index).expect("a small library"),
            cards::LIGHTNING_BOLT,
            PlayerId::One,
        ));
    }
    let inti = game
        .put_onto_battlefield(PlayerId::One, cards::INTI_SENESCHAL_OF_THE_SUN)
        .expect("cataloged");
    let bears = game
        .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
        .expect("cataloged");
    for permanent in &mut game.battlefield {
        permanent.entered_controller_turn = 0;
    }
    game.turns_started = [2, 1];
    drain_pending(&mut game);
    game.active_player = PlayerId::One;
    game.step = Step::DeclareAttackers;
    game.attackers_declared = false;
    game.priority = PlayerId::One;
    (game, inti, bears)
}

/// Declares the attack, which puts Inti's first trigger on the stack.
fn attack(game: &mut Game, attackers: &[GameObjectId]) {
    for attacker in attackers {
        game.apply(
            PlayerId::One,
            Action::DeclareAttacker {
                attacker: *attacker,
                defender: AttackDefender::Player(PlayerId::Two),
            },
        )
        .expect("the creature attacks");
    }
    game.apply(PlayerId::One, Action::FinishDeclaringAttackers)
        .expect("the declaration finishes");
}

fn first_decision(game: &Game) -> Option<DecisionObservation> {
    game.pending_decisions
        .first()
        .map(|pending| pending.observation.clone())
}

fn prompts(game: &Game) -> Vec<String> {
    game.pending_decisions
        .iter()
        .map(|pending| pending.observation.prompt.clone())
        .collect()
}

/// Passes priority until the attack trigger resolves into its offer.
fn resolve_to_the_offer(game: &mut Game) -> DecisionObservation {
    pass_until_decision(game);
    first_decision(game).expect("the attack trigger offers the discard")
}

fn answer_the_offer(game: &mut Game, offer: &DecisionObservation, discard: bool) {
    let option = offer
        .options
        .iter()
        .find(|option| (option.label != "Decline") == discard)
        .expect("the offer can be taken or declined")
        .id;
    game.apply(
        offer.player,
        Action::ChooseDecision {
            decision: offer.id,
            options: vec![option],
        },
    )
    .expect("answering the offer is legal");
}

/// Accepts the offer and answers what the discard itself asks: which card
/// goes, and the order of the two triggers the discard makes. It stops at
/// the question the reflexive trigger asks, if it asks one.
fn discard_up_to_the_target(game: &mut Game) -> Option<DecisionObservation> {
    let offer = resolve_to_the_offer(game);
    answer_the_offer(game, &offer, true);
    for _ in 0..4 {
        let decision = first_decision(game)?;
        if decision.kind == DecisionKind::TriggerPlacement {
            return Some(decision);
        }
        let options = decision
            .options
            .iter()
            .map(|option| option.id)
            .take(decision.minimum.max(1).min(decision.maximum))
            .collect();
        game.apply(
            decision.player,
            Action::ChooseDecision {
                decision: decision.id,
                options,
            },
        )
        .expect("the discard's own questions have legal answers");
    }
    None
}

fn name_the_target(game: &mut Game, targeting: &DecisionObservation, target: GameObjectId) {
    let option = targeting
        .options
        .iter()
        .find(|option| option.card.is_some_and(|(id, _)| id == target))
        .expect("the attacking creature is there to be named")
        .id;
    game.apply(
        targeting.player,
        Action::ChooseDecision {
            decision: targeting.id,
            options: vec![option],
        },
    )
    .expect("naming it is legal");
}

/// Attacks with the bear, answers the "you may discard" offer, and lets
/// everything that follows resolve.
fn attack_and_answer(game: &mut Game, bears: GameObjectId, discard: bool) {
    attack(game, &[bears]);
    let offer = resolve_to_the_offer(game);
    answer_the_offer(game, &offer, discard);
    drain_pending(game);
}

fn counters_on(game: &Game, id: GameObjectId) -> u16 {
    game.battlefield
        .iter()
        .find(|permanent| permanent.card.id == id)
        .map_or(0, |permanent| {
            permanent.counters(CounterKind::PlusOnePlusOne)
        })
}

fn on_battlefield(game: &Game, id: GameObjectId) -> bool {
    game.battlefield
        .iter()
        .any(|permanent| permanent.card.id == id)
}

/// Declining the discard ends the clause.
#[test]
fn declining_the_discard_does_nothing() {
    let (mut game, _inti, bears) = staged(2, 3);

    attack_and_answer(&mut game, bears, false);

    assert_eq!(counters_on(&game, bears), 0);
    assert_eq!(game.players[0].hand.len(), 2, "the hand is intact");
    assert!(game.players[0].exile.is_empty(), "and nothing was exiled");
}

/// Discarding grows the attacker and gives it trample.
#[test]
fn discarding_grows_the_attacker() {
    let (mut game, _inti, bears) = staged(2, 3);

    attack_and_answer(&mut game, bears, true);

    assert_eq!(counters_on(&game, bears), 1);
    let grown = game
        .battlefield
        .iter()
        .find(|permanent| permanent.card.id == bears)
        .expect("it is still attacking");
    assert_eq!(game.power(grown), Some(3));
    assert!(game.permanent_has_executable_keyword(grown, KeywordAbility::Trample));
}

/// The discard he asks for is the discard his other half watches, so one
/// attack both grows a creature and finds a card.
#[test]
fn the_discard_feeds_the_other_half() {
    let (mut game, _inti, bears) = staged(2, 3);
    let library = game.players[0].library.len();

    attack_and_answer(&mut game, bears, true);

    assert_eq!(counters_on(&game, bears), 1);
    assert_eq!(game.players[0].library.len(), library - 1);
    assert_eq!(game.players[0].exile.len(), 1, "one card, not one per half");
    assert_eq!(game.players[0].graveyard.len(), 1, "the card he pitched");
}

/// A discard of two is one trigger, not two.
#[test]
fn a_discard_of_two_exiles_one_card() {
    let (mut game, _inti, _bears) = staged(3, 4);
    game.step = Step::PrecombatMain;
    let cards = game.players[0]
        .hand
        .iter()
        .take(2)
        .map(|card| card.id)
        .collect::<Vec<_>>();

    game.discard_cards(PlayerId::One, &cards);
    drain_pending(&mut game);

    assert_eq!(
        game.players[0].exile.len(),
        1,
        "\"one or more cards\" is one trigger",
    );
}

/// What he finds is playable, and still costs its mana.
#[test]
fn the_exiled_card_is_playable_for_its_cost() {
    let (mut game, _inti, bears) = staged(2, 3);
    attack_and_answer(&mut game, bears, true);
    let exiled = game.players[0].exile[0].id;
    game.step = Step::PostcombatMain;
    game.priority = PlayerId::One;

    assert!(
        game.legal_actions(PlayerId::One)
            .into_iter()
            .all(|action| !matches!(action, Action::CastSpell { card, .. } if card == exiled)),
        "there is no red mana yet",
    );

    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);

    assert!(
        game.legal_actions(PlayerId::One)
            .into_iter()
            .any(|action| matches!(action, Action::CastSpell { card, .. } if card == exiled)),
    );
}

/// "Until your next end step" reaches into your own turn when the discard
/// happened on somebody else's.
#[test]
fn a_discard_on_their_turn_lasts_into_yours() {
    let (mut game, _inti, _bears) = staged(2, 3);
    game.active_player = PlayerId::Two;
    game.step = Step::PrecombatMain;
    let pitched = game.players[0].hand[0].id;

    game.discard_cards(PlayerId::One, &[pitched]);
    drain_pending(&mut game);
    let exiled = game.players[0].exile[0].id;
    game.priority = PlayerId::One;
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);
    assert!(
        game.legal_actions(PlayerId::One)
            .into_iter()
            .any(|action| matches!(action, Action::CastSpell { card, .. } if card == exiled)),
        "playable already, on their turn",
    );

    // Your turn arrives; the permission is still there.
    game.active_player = PlayerId::One;
    game.turns_started[PlayerId::One.index()] += 1;
    game.priority = PlayerId::One;
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);

    assert!(
        game.legal_actions(PlayerId::One)
            .into_iter()
            .any(|action| matches!(action, Action::CastSpell { card, .. } if card == exiled)),
        "and still playable on yours, which is what the end step names",
    );
}

/// And no further: once that turn is over the permission is gone.
#[test]
fn it_lapses_after_your_next_end_step() {
    let (mut game, _inti, _bears) = staged(2, 3);
    game.step = Step::PrecombatMain;
    let pitched = game.players[0].hand[0].id;
    game.discard_cards(PlayerId::One, &[pitched]);
    drain_pending(&mut game);
    let exiled = game.players[0].exile[0].id;

    game.turns_started[PlayerId::One.index()] += 1;
    game.priority = PlayerId::One;
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);

    assert!(
        game.legal_actions(PlayerId::One)
            .into_iter()
            .all(|action| !matches!(action, Action::CastSpell { card, .. } if card == exiled)),
        "your next end step came and went",
    );
}

/// "You pay all costs and follow all normal timing rules for cards played
/// due to the last ability. For example, if the exiled card is a land card,
/// you may play it only during your main phase while the stack is empty."
#[test]
fn a_land_he_exiles_still_wants_a_land_drop() {
    let (mut game, _inti, bears) = staged(2, 0);
    game.players[0]
        .library
        .push(card(97_200, cards::MOUNTAIN, PlayerId::One));

    attack_and_answer(&mut game, bears, true);
    let exiled = game.players[0].exile[0].id;
    assert_eq!(
        game.players[0].exile[0].definition,
        cards::MOUNTAIN,
        "the land off the top is what he found",
    );

    assert!(
        game.legal_actions(PlayerId::One)
            .into_iter()
            .all(|action| !matches!(action, Action::PlayLand { card, .. } if card == exiled)),
        "a land is not played in the middle of an attack",
    );

    game.step = Step::PostcombatMain;
    game.priority = PlayerId::One;
    game.players[0].lands_played_this_turn = 1;
    assert!(
        game.legal_actions(PlayerId::One)
            .into_iter()
            .all(|action| !matches!(action, Action::PlayLand { card, .. } if card == exiled)),
        "nor once the turn's land drop is spent",
    );

    game.players[0].lands_played_this_turn = 0;
    assert!(
        game.legal_actions(PlayerId::One)
            .into_iter()
            .any(|action| matches!(action, Action::PlayLand { card, .. } if card == exiled)),
        "and with a main phase and a land drop it is playable",
    );
}

/// "You don't choose a target for Inti's first ability at the time it
/// triggers. Rather, a second 'reflexive' ability triggers when you discard a
/// card this way. You choose a target for that ability as it goes on the
/// stack." So the attack trigger carries no target, the card is already in
/// the graveyard when the attacker is named, and the counter waits on the
/// stack as its own object for an answer.
#[test]
fn the_target_is_chosen_after_the_discard() {
    let (mut game, _inti, bears) = staged(2, 3);
    let held = game.players[0].hand.len();

    attack(&mut game, &[bears]);
    assert!(
        game.pending_decisions.is_empty(),
        "the attack trigger reaches the stack without a question: {:?}",
        prompts(&game),
    );
    assert_eq!(game.stack.len(), 1);
    assert!(
        game.stack[0]
            .ability
            .as_ref()
            .expect("it is an ability")
            .targets
            .is_empty(),
        "and it carries no target",
    );

    let targeting = discard_up_to_the_target(&mut game).expect("the discard asks for an attacker");

    assert_eq!(
        game.players[0].hand.len(),
        held - 1,
        "the card has left the hand before anything is named",
    );
    assert_eq!(game.players[0].graveyard.len(), 1);
    assert_eq!(counters_on(&game, bears), 0);

    name_the_target(&mut game, &targeting, bears);

    assert!(
        game.pending_decisions.is_empty(),
        "one target is all it asks for: {:?}",
        prompts(&game),
    );
    assert!(
        game.stack
            .iter()
            .all(|object| object.kind == StackObjectKind::TriggeredAbility),
    );
    assert!(
        game.stack.iter().any(|object| {
            object.ability.as_ref().is_some_and(|ability| {
                ability
                    .targets
                    .iter()
                    .any(|selection| selection.targets().contains(&Target::Permanent(bears)))
            })
        }),
        "the counter is its own object on the stack, aimed at the bear",
    );
    assert_eq!(
        counters_on(&game, bears),
        0,
        "and nothing has landed while it waits there",
    );

    drain_pending(&mut game);

    assert_eq!(counters_on(&game, bears), 1);
}

/// The attacking creature is named against the board as it stands after the
/// discard. One that was removed while the attack trigger waited is not
/// there to be named, which the old up-front target would have allowed.
#[test]
fn an_attacker_removed_before_the_discard_is_not_a_legal_target() {
    let (mut game, inti, bears) = staged(2, 3);
    attack(&mut game, &[inti, bears]);
    assert_eq!(game.stack.len(), 1, "the attack trigger is on the stack");

    game.destroy_permanent(bears);
    game.check_state_based_actions();
    assert!(!on_battlefield(&game, bears));

    let targeting = discard_up_to_the_target(&mut game).expect("Inti is still attacking");

    assert_eq!(
        targeting.options.len(),
        1,
        "only the attacker that is left: {:?}",
        targeting.options,
    );
    assert!(
        targeting
            .options
            .iter()
            .all(|option| option.card.is_some_and(|(id, _)| id == inti)),
    );

    name_the_target(&mut game, &targeting, inti);
    drain_pending(&mut game);

    assert_eq!(counters_on(&game, inti), 1);
}

/// With no attacker left at all the discard may still be made, and it still
/// feeds his other half; the reflexive trigger simply has nothing to name
/// and never reaches the stack.
#[test]
fn a_discard_with_no_attacker_left_still_finds_a_card() {
    let (mut game, inti, bears) = staged(2, 3);
    attack(&mut game, &[bears]);

    game.destroy_permanent(bears);
    game.check_state_based_actions();

    let targeting = discard_up_to_the_target(&mut game);
    assert!(
        targeting.is_none(),
        "there is no attacking creature to ask about: {:?}",
        prompts(&game),
    );
    drain_pending(&mut game);

    assert_eq!(game.players[0].hand.len(), 1, "the card was still pitched");
    assert_eq!(game.players[0].exile.len(), 1, "and the other half saw it");
    assert_eq!(counters_on(&game, inti), 0, "but nothing grew");
}

/// Declining ends the matter without anybody ever being named.
#[test]
fn declining_asks_for_no_target() {
    let (mut game, _inti, bears) = staged(2, 3);
    attack(&mut game, &[bears]);
    assert!(
        game.pending_decisions.is_empty(),
        "nothing is named as the attack trigger goes on the stack: {:?}",
        prompts(&game),
    );

    let offer = resolve_to_the_offer(&mut game);
    assert_eq!(offer.options.len(), 2, "discard a card, or do not");
    assert!(
        offer
            .options
            .iter()
            .all(|option| option.card.is_none_or(|(id, _)| id != bears)),
        "the offer names no creature: {:?}",
        offer.options,
    );

    answer_the_offer(&mut game, &offer, false);

    assert!(
        game.pending_decisions.is_empty(),
        "declining asks for nothing more: {:?}",
        prompts(&game),
    );
    assert!(game.pending_triggers.is_empty());
    assert!(game.stack.is_empty());
    assert_eq!(game.players[0].hand.len(), 2);
    assert_eq!(counters_on(&game, bears), 0);
}

/// The discard is made before the target is named, so answering the target
/// afterwards loses the counter and nothing else: the card stays discarded
/// and the other half still finds its card.
#[test]
fn an_answered_target_loses_the_counter_not_the_discard() {
    let (mut game, inti, bears) = staged(2, 3);
    let held = game.players[0].hand.len();
    attack(&mut game, &[bears]);
    let targeting = discard_up_to_the_target(&mut game).expect("the discard asks for an attacker");
    name_the_target(&mut game, &targeting, bears);

    game.destroy_permanent(bears);
    game.check_state_based_actions();
    drain_pending(&mut game);

    assert_eq!(
        game.players[0].hand.len(),
        held - 1,
        "the discard had already happened",
    );
    assert!(
        game.players[0]
            .graveyard
            .iter()
            .any(|card| card.definition == cards::MOUNTAIN),
        "and the pitched card stays where it went",
    );
    assert_eq!(
        game.players[0].exile.len(),
        1,
        "the other half saw the discard all the same",
    );
    assert_eq!(counters_on(&game, inti), 0, "the counter goes nowhere else");
    assert!(game.stack.is_empty());
}

/// The reflexive trigger belongs to the discard that was made, not to Inti
/// still being around to notice it. Removing him in response to the attack
/// trigger leaves the offer standing, and accepting it still grows the
/// attacker. His other half is gone with him, so nothing is exiled.
#[test]
fn the_counter_still_lands_when_inti_has_left() {
    let (mut game, inti, bears) = staged(2, 3);
    attack(&mut game, &[bears]);
    assert_eq!(game.stack.len(), 1, "the attack trigger is on the stack");

    game.destroy_permanent(inti);
    game.check_state_based_actions();
    assert!(!on_battlefield(&game, inti), "he is gone before the offer");

    let targeting =
        discard_up_to_the_target(&mut game).expect("the target is still asked for without him");
    name_the_target(&mut game, &targeting, bears);
    drain_pending(&mut game);

    assert_eq!(counters_on(&game, bears), 1);
    let grown = game
        .battlefield
        .iter()
        .find(|permanent| permanent.card.id == bears)
        .expect("it is still attacking");
    assert!(game.permanent_has_executable_keyword(grown, KeywordAbility::Trample));
    assert!(
        game.players[0].exile.is_empty(),
        "nobody was watching the discard",
    );
}
