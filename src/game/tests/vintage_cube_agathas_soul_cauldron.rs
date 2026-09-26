//! Agatha's Soul Cauldron: a graveyard eater that hands what it ate to
//! whichever of your creatures is carrying a counter, and pays for their
//! abilities with whatever mana you happen to have.
//!
//! "When a creature card is exiled this way" is a reflexive trigger made by
//! the exile itself. It names its creature only on the way to the stack,
//! after the card has gone, and it does not need the Cauldron to still be on
//! the battlefield.

use super::*;

/// Player One with a Cauldron on the battlefield, `graveyard` in Player Two's
/// graveyard, and `battlefield` under Player One beside the Cauldron.
fn staged(
    graveyard: &[CardDefinitionId],
    battlefield: &[CardDefinitionId],
) -> (Game, GameObjectId, Vec<GameObjectId>) {
    let mut game = ready_game();
    game.battlefield.clear();
    game.players[1].graveyard.clear();
    game.players[1].exile.clear();
    for definition in graveyard {
        let card = game
            .build_zone(PlayerId::Two, &[*definition])
            .expect("cataloged")
            .into_iter()
            .next()
            .expect("one card");
        game.players[1].graveyard.push(card);
    }
    let cauldron = game
        .put_onto_battlefield(PlayerId::One, cards::AGATHAS_SOUL_CAULDRON)
        .expect("cataloged");
    let mut theirs = Vec::new();
    for definition in battlefield {
        theirs.push(
            game.put_onto_battlefield(PlayerId::One, *definition)
                .expect("cataloged"),
        );
    }
    for permanent in &mut game.battlefield {
        permanent.entered_controller_turn = 0;
    }
    drain_pending(&mut game);
    game.active_player = PlayerId::One;
    game.step = Step::PrecombatMain;
    game.priority = PlayerId::One;
    (game, cauldron, theirs)
}

fn permanent(game: &Game, id: GameObjectId) -> &Permanent {
    game.battlefield
        .iter()
        .find(|candidate| candidate.card.id == id)
        .expect("it is on the battlefield")
}

fn counters(game: &Game, id: GameObjectId) -> u16 {
    permanent(game, id).counters(CounterKind::PlusOnePlusOne)
}

fn put_a_counter_on(game: &mut Game, id: GameObjectId) {
    game.battlefield
        .iter_mut()
        .find(|candidate| candidate.card.id == id)
        .expect("it is on the battlefield")
        .add_counters(CounterKind::PlusOnePlusOne, 1);
}

fn prompts(game: &Game) -> Vec<String> {
    game.pending_decisions
        .iter()
        .map(|pending| pending.observation.prompt.clone())
        .collect()
}

fn exiled(game: &Game, definition: CardDefinitionId) -> bool {
    game.players[1]
        .exile
        .iter()
        .any(|card| card.definition == definition)
}

/// Taps the Cauldron for the card of `definition` in Player Two's graveyard
/// and leaves the activation on the stack.
fn tap_the_cauldron_for(game: &mut Game, cauldron: GameObjectId, definition: CardDefinitionId) {
    let target = game.players[1]
        .graveyard
        .iter()
        .find(|card| card.definition == definition)
        .expect("it is in the graveyard")
        .id;
    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| match action {
            Action::ActivateAbility {
                source, targets, ..
            } => {
                *source == cauldron
                    && targets
                        .iter()
                        .any(|selection| selection.targets().contains(&Target::Card(target)))
            }
            _ => false,
        })
        .expect("a card in a graveyard is a legal target");
    game.apply(PlayerId::One, action).expect("it activates");
}

/// Resolves the activation and stops at the question the reflexive trigger
/// asks on its way to the stack, if it asks one.
fn resolve_to_the_target(game: &mut Game) -> Option<DecisionObservation> {
    pass_until_decision(game);
    game.pending_decisions
        .first()
        .map(|pending| pending.observation.clone())
        .filter(|decision| decision.kind == DecisionKind::TriggerPlacement)
}

fn name_the_target(game: &mut Game, targeting: &DecisionObservation, target: GameObjectId) {
    let option = targeting
        .options
        .iter()
        .find(|option| option.card.is_some_and(|(id, _)| id == target))
        .expect("the creature is there to be named")
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

/// Taps the Cauldron for the card of `definition` and lets everything that
/// follows resolve. `grow` is the creature the reflexive trigger names, and
/// is `None` exactly when the test expects no such trigger to ask.
fn eat(
    game: &mut Game,
    cauldron: GameObjectId,
    definition: CardDefinitionId,
    grow: Option<GameObjectId>,
) {
    tap_the_cauldron_for(game, cauldron, definition);
    match (resolve_to_the_target(game), grow) {
        (Some(targeting), Some(grow)) => name_the_target(game, &targeting, grow),
        (None, None) => {}
        (Some(targeting), None) => panic!(
            "no counter was expected, but one asks for its creature: {:?}",
            targeting.options,
        ),
        (None, Some(_)) => panic!(
            "a counter was expected, but nothing asks for its creature: {:?}",
            prompts(game),
        ),
    }
    drain_pending(game);
}

/// Every activated ability the permanent currently has, by its rules text.
fn activated_ability_texts(game: &Game, id: GameObjectId) -> Vec<&'static str> {
    let mut texts = Vec::new();
    let _ = game.visit_effective_abilities(permanent(game, id), |effective| {
        if matches!(
            effective.ability.definition,
            crate::card::DeclarativeAbilityDef::Activated(_)
        ) {
            texts.push(effective.ability.text);
        }
        std::ops::ControlFlow::Continue(())
    });
    texts
}

/// Activates the ability whose printed text starts with `prefix`.
fn activate(game: &mut Game, source: GameObjectId, prefix: &str) {
    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| match action {
            Action::ActivateAbility {
                source: candidate, ..
            } => *candidate == source && ability_text(game, action).starts_with(prefix),
            _ => false,
        })
        .unwrap_or_else(|| panic!("{prefix} is activatable"));
    game.apply(PlayerId::One, action).expect("it activates");
    drain_pending(game);
}

/// The printed text behind one activation action, so a test can name an
/// ability the way the card prints it rather than by index.
fn ability_text(game: &Game, action: &Action) -> &'static str {
    let Action::ActivateAbility {
        source, ability, ..
    } = action
    else {
        return "";
    };
    let mut text = "";
    let _ = game.visit_effective_abilities(permanent(game, *source), |effective| {
        if effective.origin == *ability {
            text = effective.ability.text;
            return std::ops::ControlFlow::Break(());
        }
        std::ops::ControlFlow::Continue(())
    });
    text
}

fn power(game: &Game, id: GameObjectId) -> i16 {
    game.power(permanent(game, id)).expect("it is a creature")
}

/// The reflexive trigger fires for a creature card and grows the creature it
/// names once the card has gone.
#[test]
fn eating_a_creature_card_grows_the_named_creature() {
    let (mut game, cauldron, mine) = staged(
        &[cards::ORDER_OF_THE_EBON_HAND],
        &[cards::SAVANNAH_LIONS, cards::GRIZZLY_BEARS],
    );
    let (lion, bears) = (mine[0], mine[1]);

    eat(
        &mut game,
        cauldron,
        cards::ORDER_OF_THE_EBON_HAND,
        Some(bears),
    );

    assert_eq!(counters(&game, bears), 1, "a creature card was exiled");
    assert_eq!(counters(&game, lion), 0, "and only the named creature grew");
    assert!(exiled(&game, cards::ORDER_OF_THE_EBON_HAND));
}

/// "When a creature card is exiled this way" is a real condition: a land
/// leaves the graveyard just the same, and nothing grows.
#[test]
fn eating_a_noncreature_card_grows_nothing() {
    let (mut game, cauldron, mine) = staged(&[cards::PLAINS], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];

    eat(&mut game, cauldron, cards::PLAINS, None);

    assert_eq!(counters(&game, lion), 0, "no creature card was exiled");
    assert!(
        game.players[1].graveyard.is_empty(),
        "the land was exiled either way",
    );
    assert!(exiled(&game, cards::PLAINS));
}

/// The grant is gated on counters, not on being a creature: the creature the
/// counter did not go to reads only its own text.
#[test]
fn a_creature_without_counters_gains_nothing() {
    let (mut game, cauldron, mine) = staged(
        &[cards::ORDER_OF_THE_EBON_HAND],
        &[cards::SAVANNAH_LIONS, cards::GRIZZLY_BEARS],
    );
    let (lion, bears) = (mine[0], mine[1]);

    eat(
        &mut game,
        cauldron,
        cards::ORDER_OF_THE_EBON_HAND,
        Some(bears),
    );

    assert_eq!(counters(&game, lion), 0, "nothing named it");
    assert!(
        activated_ability_texts(&game, lion).is_empty(),
        "a Savannah Lions with no counters prints no activated abilities: {:?}",
        activated_ability_texts(&game, lion),
    );
    assert_eq!(
        activated_ability_texts(&game, bears).len(),
        2,
        "while the bear beside it, carrying the counter, reads the Order's",
    );
}

/// Both of the exiled card's activated abilities land on the creature the
/// counter went to.
#[test]
fn a_countered_creature_gains_every_exiled_activated_ability() {
    let (mut game, cauldron, mine) =
        staged(&[cards::ORDER_OF_THE_EBON_HAND], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];

    eat(
        &mut game,
        cauldron,
        cards::ORDER_OF_THE_EBON_HAND,
        Some(lion),
    );

    let texts = activated_ability_texts(&game, lion);
    assert_eq!(
        texts.len(),
        2,
        "the Order prints two activated abilities: {texts:?}",
    );
    assert!(
        texts
            .iter()
            .any(|text| text.starts_with("{B}{B}: This creature gets +1/+0")),
        "the pump came along: {texts:?}",
    );
}

/// Protection from white is a static ability, so it stays behind: only
/// activated abilities are handed out.
#[test]
fn the_exiled_cards_static_abilities_stay_behind() {
    let (mut game, cauldron, mine) =
        staged(&[cards::ORDER_OF_THE_EBON_HAND], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];

    eat(
        &mut game,
        cauldron,
        cards::ORDER_OF_THE_EBON_HAND,
        Some(lion),
    );

    let mut texts = Vec::new();
    let _ = game.visit_effective_abilities(permanent(&game, lion), |effective| {
        texts.push(effective.ability.text);
        std::ops::ControlFlow::Continue(())
    });
    assert!(
        !texts.iter().any(|text| text.contains("Protection from")),
        "protection from white is static and was not granted: {texts:?}",
    );
    assert_eq!(
        activated_ability_texts(&game, lion).len(),
        2,
        "the two activated abilities did come across",
    );
}

/// The granted ability is not decoration: it can be activated, and it does
/// what the exiled card says.
#[test]
fn a_granted_ability_can_be_activated() {
    let (mut game, cauldron, mine) =
        staged(&[cards::ORDER_OF_THE_EBON_HAND], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];
    eat(
        &mut game,
        cauldron,
        cards::ORDER_OF_THE_EBON_HAND,
        Some(lion),
    );
    let before = power(&game, lion);
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Black, 2);

    activate(&mut game, lion, "{B}{B}: This creature gets +1/+0");

    assert_eq!(
        power(&game, lion),
        before + 1,
        "the granted pump resolved on the creature that has it",
    );
}

/// The Cauldron's other half: white mana pays a black activation cost,
/// because the ability belongs to a creature its controller controls.
#[test]
fn white_mana_pays_a_creatures_black_ability() {
    let (mut game, _cauldron, mine) = staged(&[], &[cards::ORDER_OF_THE_EBON_HAND]);
    let order = mine[0];
    let before = power(&game, order);
    game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 2);

    activate(&mut game, order, "{B}{B}: This creature gets +1/+0");

    assert_eq!(
        power(&game, order),
        before + 1,
        "the permission let two white mana pay {{B}}{{B}}",
    );
    assert_eq!(
        game.players[0].mana_pool.total(),
        0,
        "and both of them were spent on it",
    );
}

/// Without the Cauldron the same white mana pays for nothing, which is what
/// makes the permission the reason the activation above was offered.
#[test]
fn without_the_cauldron_white_mana_pays_nothing_black() {
    let (mut game, cauldron, mine) = staged(&[], &[cards::ORDER_OF_THE_EBON_HAND]);
    let order = mine[0];
    game.battlefield
        .retain(|permanent| permanent.card.id != cauldron);
    game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 2);

    assert!(
        !game.legal_actions(PlayerId::One).iter().any(|action| {
            matches!(action, Action::ActivateAbility { source, .. } if *source == order)
                && ability_text(&game, action).starts_with("{B}{B}")
        }),
        "white mana does not pay a black cost on its own",
    );
}

/// The permission is about abilities, not spells: a black card in hand is
/// still uncastable off white mana.
#[test]
fn the_permission_does_not_reach_spells() {
    let (mut game, _cauldron, _mine) = staged(&[], &[]);
    game.players[0].hand.clear();
    let card = game
        .build_zone(PlayerId::One, &[cards::ORDER_OF_THE_EBON_HAND])
        .expect("cataloged")
        .into_iter()
        .next()
        .expect("one card");
    let order = card.id;
    game.players[0].hand.push(card);
    game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 2);

    assert!(
        !game
            .legal_actions(PlayerId::One)
            .iter()
            .any(|action| matches!(action, Action::CastSpell { card, .. } if *card == order)),
        "{{B}}{{B}} for a creature spell is not what the Cauldron permits",
    );
}

/// "Creatures you control": the permission is its controller's alone, and a
/// black ability across the table still wants black mana.
#[test]
fn the_permission_does_not_reach_their_creatures() {
    let (mut game, _cauldron, _mine) = staged(&[], &[]);
    let order = game
        .put_onto_battlefield(PlayerId::Two, cards::ORDER_OF_THE_EBON_HAND)
        .expect("cataloged");
    drain_pending(&mut game);
    game.priority = PlayerId::Two;
    game.add_unrestricted_mana(PlayerId::Two, ManaColor::White, 2);

    assert!(
        !game.legal_actions(PlayerId::Two).iter().any(|action| {
            matches!(action, Action::ActivateAbility { source, .. } if *source == order)
                && ability_text(&game, action).starts_with("{B}{B}")
        }),
        "their white mana pays nothing black: the Cauldron is not theirs",
    );
}

/// The pile accumulates: a second creature card adds its abilities to what
/// the first one already handed out.
#[test]
fn a_second_exiled_creature_adds_its_abilities_too() {
    let (mut game, cauldron, mine) = staged(
        &[cards::ORDER_OF_THE_EBON_HAND, cards::PRODIGAL_SORCERER],
        &[cards::SAVANNAH_LIONS],
    );
    let lion = mine[0];

    eat(
        &mut game,
        cauldron,
        cards::ORDER_OF_THE_EBON_HAND,
        Some(lion),
    );
    let after_one = activated_ability_texts(&game, lion).len();
    for permanent in &mut game.battlefield {
        permanent.tapped = false;
    }
    eat(&mut game, cauldron, cards::PRODIGAL_SORCERER, Some(lion));

    assert_eq!(after_one, 2, "the Order printed two");
    assert_eq!(
        activated_ability_texts(&game, lion).len(),
        3,
        "the Sorcerer's tapper came along as well: {:?}",
        activated_ability_texts(&game, lion),
    );
    assert_eq!(
        counters(&game, lion),
        2,
        "one counter for each creature card"
    );
}

/// The grant is read fresh every time, so a creature that loses its last
/// counter hands the abilities straight back.
#[test]
fn losing_the_last_counter_takes_the_abilities_back() {
    let (mut game, cauldron, mine) =
        staged(&[cards::ORDER_OF_THE_EBON_HAND], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];
    eat(
        &mut game,
        cauldron,
        cards::ORDER_OF_THE_EBON_HAND,
        Some(lion),
    );
    assert_eq!(activated_ability_texts(&game, lion).len(), 2, "granted");

    game.battlefield
        .iter_mut()
        .find(|candidate| candidate.card.id == lion)
        .expect("it is on the battlefield")
        .remove_counters(CounterKind::PlusOnePlusOne, 1);

    assert!(
        activated_ability_texts(&game, lion).is_empty(),
        "the abilities went back with the counter: {:?}",
        activated_ability_texts(&game, lion),
    );
}

/// The counter does not have to be the Cauldron's own: any +1/+1 counter
/// makes a creature one the pile hands its abilities to.
#[test]
fn a_counter_from_anywhere_earns_the_abilities() {
    let (mut game, cauldron, mine) = staged(
        &[cards::ORDER_OF_THE_EBON_HAND],
        &[cards::SAVANNAH_LIONS, cards::GRIZZLY_BEARS],
    );
    let (lion, bears) = (mine[0], mine[1]);
    put_a_counter_on(&mut game, lion);

    eat(
        &mut game,
        cauldron,
        cards::ORDER_OF_THE_EBON_HAND,
        Some(bears),
    );

    assert_eq!(counters(&game, lion), 1, "the one it already had");
    assert_eq!(
        activated_ability_texts(&game, lion).len(),
        2,
        "which is all the grant asks about",
    );
}

/// "It doesn't grant keyword abilities, triggered abilities, or static
/// abilities." A Runewing in the pile has flying and a dies trigger and
/// nothing else: the Lion gains nothing at all, and dying draws no card.
#[test]
fn the_exiled_cards_triggered_abilities_stay_behind() {
    let (mut game, cauldron, mine) = staged(&[cards::RUNEWING], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];
    game.players[0].library.clear();
    game.players[0]
        .library
        .push(card(94_500, cards::ISLAND, PlayerId::One));
    let hand = game.players[0].hand.len();

    eat(&mut game, cauldron, cards::RUNEWING, Some(lion));
    assert_eq!(counters(&game, lion), 1, "a creature card, so it grew");

    assert_eq!(
        activated_ability_texts(&game, lion),
        Vec::<&str>::new(),
        "the Runewing prints no activated ability to hand over",
    );
    assert!(
        !game.has_flying(permanent(&game, lion)),
        "and flying is a keyword, which stays behind with the rest",
    );

    // Kill the Lion: if the dies trigger had come across, this would draw.
    if let Some(permanent) = game
        .battlefield
        .iter_mut()
        .find(|candidate| candidate.card.id == lion)
    {
        permanent.damage = 2;
    }
    game.check_state_based_actions();
    drain_pending(&mut game);

    assert!(
        !game
            .battlefield
            .iter()
            .any(|candidate| candidate.card.id == lion),
        "the Lion died",
    );
    assert_eq!(
        game.players[0].hand.len(),
        hand,
        "and drew nothing on the way out",
    );
    assert_eq!(game.players[0].library.len(), 1, "the library is untouched");
}

/// The activation names the card and nothing else. The creature is named by
/// a second, reflexive ability as it goes on the stack, by which time the
/// card is already in exile, and the counter waits there as its own object
/// for an answer.
#[test]
fn the_counter_target_is_chosen_after_the_exile() {
    let (mut game, cauldron, mine) =
        staged(&[cards::ORDER_OF_THE_EBON_HAND], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];

    tap_the_cauldron_for(&mut game, cauldron, cards::ORDER_OF_THE_EBON_HAND);
    assert!(
        game.pending_decisions.is_empty(),
        "the activation reaches the stack without a second question: {:?}",
        prompts(&game),
    );
    assert_eq!(game.stack.len(), 1);
    let named = &game.stack[0]
        .ability
        .as_ref()
        .expect("it is an ability")
        .targets;
    assert_eq!(named.len(), 1, "one target slot, for the card: {named:?}");
    assert!(
        named
            .iter()
            .all(|selection| !selection.targets().contains(&Target::Permanent(lion))),
        "and no creature is named with it",
    );

    let targeting = resolve_to_the_target(&mut game).expect("the exile asks for a creature");

    assert!(
        exiled(&game, cards::ORDER_OF_THE_EBON_HAND),
        "the card has gone before anything is named",
    );
    assert!(game.players[1].graveyard.is_empty());
    assert!(game.stack.is_empty(), "and the activation has finished");
    assert_eq!(counters(&game, lion), 0);

    name_the_target(&mut game, &targeting, lion);

    assert!(
        game.pending_decisions.is_empty(),
        "one target is all it asks for: {:?}",
        prompts(&game),
    );
    assert_eq!(
        game.stack.len(),
        1,
        "the counter is one object on the stack"
    );
    assert_eq!(game.stack[0].kind, StackObjectKind::TriggeredAbility);
    assert_eq!(game.stack[0].controller, PlayerId::One);
    assert!(
        game.stack[0].ability.as_ref().is_some_and(|ability| {
            ability
                .targets
                .iter()
                .any(|selection| selection.targets().contains(&Target::Permanent(lion)))
        }),
        "aimed at the Lion",
    );
    assert_eq!(
        counters(&game, lion),
        0,
        "and nothing has landed while it waits there",
    );

    game.apply(PlayerId::One, Action::PassPriority)
        .expect("you may pass with the counter on the stack");
    assert_eq!(game.priority, PlayerId::Two, "and they may answer it");
    assert_eq!(counters(&game, lion), 0);

    drain_pending(&mut game);

    assert_eq!(counters(&game, lion), 1);
}

/// The creature is named against the board as it stands after the exile. One
/// that arrived while the activation waited on the stack is there to be
/// named, which the old up-front target could never have allowed.
#[test]
fn a_creature_that_arrives_in_response_to_the_activation_is_a_legal_target() {
    let (mut game, cauldron, mine) =
        staged(&[cards::ORDER_OF_THE_EBON_HAND], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];
    tap_the_cauldron_for(&mut game, cauldron, cards::ORDER_OF_THE_EBON_HAND);
    assert_eq!(game.stack.len(), 1, "the activation is on the stack");

    let bears = game
        .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
        .expect("cataloged");
    game.check_state_based_actions();

    let targeting = resolve_to_the_target(&mut game).expect("the exile asks for a creature");
    assert_eq!(
        targeting.options.len(),
        2,
        "the Lion and the late arrival: {:?}",
        targeting.options,
    );

    name_the_target(&mut game, &targeting, bears);
    drain_pending(&mut game);

    assert_eq!(counters(&game, bears), 1, "the late arrival is what grew");
    assert_eq!(counters(&game, lion), 0);
    assert_eq!(
        activated_ability_texts(&game, bears).len(),
        2,
        "and it reads the Order's abilities for it",
    );
}

/// And the same window read the other way: a creature answered while the
/// activation waited is not there to be named, and with nobody left the
/// reflexive trigger never reaches the stack. The card is exiled all the same.
#[test]
fn a_creature_answered_in_response_to_the_activation_is_not_a_legal_target() {
    let (mut game, cauldron, mine) =
        staged(&[cards::ORDER_OF_THE_EBON_HAND], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];
    tap_the_cauldron_for(&mut game, cauldron, cards::ORDER_OF_THE_EBON_HAND);

    game.destroy_permanent(lion);
    game.check_state_based_actions();

    let targeting = resolve_to_the_target(&mut game);
    assert!(
        targeting.is_none(),
        "there is no creature of yours to ask about: {:?}",
        prompts(&game),
    );
    drain_pending(&mut game);

    assert!(game.stack.is_empty());
    assert!(game.pending_triggers.is_empty());
    assert!(
        exiled(&game, cards::ORDER_OF_THE_EBON_HAND),
        "the exile never depended on the counter having somewhere to go",
    );
}

/// The activation is legal with no creature of yours at all, since the
/// printed ability targets only the card.
#[test]
fn with_no_creature_of_yours_the_card_is_still_exiled() {
    let (mut game, cauldron, _mine) = staged(&[cards::ORDER_OF_THE_EBON_HAND], &[]);

    eat(&mut game, cauldron, cards::ORDER_OF_THE_EBON_HAND, None);

    assert!(exiled(&game, cards::ORDER_OF_THE_EBON_HAND));
    assert!(game.players[1].graveyard.is_empty());
}

/// A noncreature card makes no reflexive trigger at all: nothing is queued,
/// nothing is asked, and nothing reaches the stack once the exile is done.
#[test]
fn a_noncreature_card_creates_no_trigger() {
    let (mut game, cauldron, mine) = staged(&[cards::PLAINS], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];
    tap_the_cauldron_for(&mut game, cauldron, cards::PLAINS);

    let targeting = resolve_to_the_target(&mut game);

    assert!(
        targeting.is_none(),
        "a land is not a creature card: {:?}",
        prompts(&game),
    );
    assert!(game.pending_decisions.is_empty(), "{:?}", prompts(&game));
    assert!(game.pending_triggers.is_empty());
    assert!(game.stack.is_empty());
    assert!(exiled(&game, cards::PLAINS), "though the land did go");
    assert_eq!(counters(&game, lion), 0);
}

/// The reflexive trigger belongs to the exile that was made, not to the
/// Cauldron still being around to notice it. Destroying the Cauldron in
/// response to the activation leaves the exile standing, and the counter
/// still asks for its creature and lands. What goes with the Cauldron is the
/// grant: the pile is nobody's any more.
#[test]
fn the_trigger_survives_the_cauldron_leaving_in_response() {
    let (mut game, cauldron, mine) =
        staged(&[cards::ORDER_OF_THE_EBON_HAND], &[cards::SAVANNAH_LIONS]);
    let lion = mine[0];
    tap_the_cauldron_for(&mut game, cauldron, cards::ORDER_OF_THE_EBON_HAND);
    assert_eq!(game.stack.len(), 1, "the activation is on the stack");

    game.destroy_permanent(cauldron);
    game.check_state_based_actions();
    assert!(
        !game
            .battlefield
            .iter()
            .any(|candidate| candidate.card.id == cauldron),
        "it is gone before the exile is made",
    );

    let targeting =
        resolve_to_the_target(&mut game).expect("the target is still asked for without it");
    assert!(exiled(&game, cards::ORDER_OF_THE_EBON_HAND));

    name_the_target(&mut game, &targeting, lion);
    drain_pending(&mut game);

    assert_eq!(counters(&game, lion), 1, "the counter still lands");
    assert!(
        activated_ability_texts(&game, lion).is_empty(),
        "but there is no Cauldron left to hand the Order's abilities over: {:?}",
        activated_ability_texts(&game, lion),
    );
}
