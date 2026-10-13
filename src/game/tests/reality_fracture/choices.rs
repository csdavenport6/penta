//! Choices a resolving spell or ability gives a player: an optional action
//! and the "if you do" that hangs on it, a pick among what is offered, and the
//! order in which several players answer.
use super::*;

/// Variable Chaser's Arc of Fortune on the stack, cast by player one, with
/// this many cards in each player's hand and ten in each library.
fn arc_of_fortune_cast(prepared_engine: bool, hands: [usize; 2]) -> Game {
    let mut game = setup(prepared_engine);
    let (_, arc) = prepared_permanent(&mut game, cards::VARIABLE_CHASER, "Arc of Fortune");
    for (player, hand) in [PlayerId::One, PlayerId::Two].into_iter().zip(hands) {
        game.players[player.index()].library =
            game.build_zone(player, &[cards::FOREST; 10]).unwrap();
        for _ in 0..hand {
            in_hand_of(&mut game, player, cards::MOUNTAIN);
        }
    }
    cast_copy(&mut game, arc, &[]);
    game
}

/// The player Arc of Fortune is asking next answers it: discard their hand
/// and draw seven, or keep it. Returns who was asked and what they were
/// offered.
fn answer_arc_of_fortune(game: &mut Game, discards: bool) -> (PlayerId, Vec<String>) {
    let question = run_to_decision(game).expect("a player is asked");
    let labels = question
        .options
        .iter()
        .map(|option| option.label.clone())
        .collect();
    game.apply(
        question.player,
        Action::ChooseDecision {
            decision: question.id,
            options: vec![question.options[usize::from(!discards)].id],
        },
    )
    .unwrap();
    (question.player, labels)
}

/// How many cards `player` has in hand, in their graveyard, and in their
/// library.
fn piles(game: &Game, player: PlayerId) -> (usize, usize, usize) {
    let player = &game.players[player.index()];
    (
        player.hand.len(),
        player.graveyard.len(),
        player.library.len(),
    )
}

/// "Each player may discard their hand and draw seven cards": both answer,
/// the active player first, and nothing moves until both have (CR 101.4).
/// The second player is told what the first chose.
#[test]
fn arc_of_fortune_takes_both_answers_in_turn_order_before_either_hand_moves() {
    for prepared_engine in [false, true] {
        let mut game = arc_of_fortune_cast(prepared_engine, [2, 3]);

        let (first, offered) = answer_arc_of_fortune(&mut game, true);
        assert_eq!(first, PlayerId::One, "the active player");
        assert_eq!(
            offered,
            ["Discard your hand and draw seven cards", "Keep your hand"]
        );
        assert_eq!(
            piles(&game, PlayerId::One),
            (2, 0, 10),
            "one answer moves nothing"
        );

        let (second, offered) = answer_arc_of_fortune(&mut game, true);
        assert_eq!(second, PlayerId::Two);
        assert_eq!(
            offered,
            [
                "Discard your hand and draw seven cards too",
                "Keep your hand"
            ]
        );
        drain_pending(&mut game);

        assert_eq!(piles(&game, PlayerId::One), (7, 2, 3));
        assert_eq!(piles(&game, PlayerId::Two), (7, 3, 3));
    }
}

#[test]
fn arc_of_fortune_moves_only_the_hands_of_players_who_chose_to() {
    for prepared_engine in [false, true] {
        for answers in [[true, false], [false, true], [false, false]] {
            let mut game = arc_of_fortune_cast(prepared_engine, [2, 3]);

            answer_arc_of_fortune(&mut game, answers[0]);
            let (_, offered) = answer_arc_of_fortune(&mut game, answers[1]);
            drain_pending(&mut game);

            if !answers[0] {
                assert_eq!(
                    offered,
                    [
                        "Discard your hand and draw seven cards",
                        "Keep your hand too"
                    ]
                );
            }
            for (player, kept) in [(PlayerId::One, (2, 0, 10)), (PlayerId::Two, (3, 0, 10))] {
                let discarded = (7, kept.0, 3);
                let expected = if answers[player.index()] {
                    discarded
                } else {
                    kept
                };
                assert_eq!(piles(&game, player), expected, "{answers:?}");
            }
        }
    }
}

/// The second question is asked inside the first one's answer. A game rebuilt
/// from a checkpoint taken between the two still knows which answer that was.
#[test]
fn arc_of_fortune_keeps_the_first_answer_across_a_checkpoint() {
    for prepared_engine in [false, true] {
        for first_discards in [true, false] {
            let mut game = arc_of_fortune_cast(prepared_engine, [2, 3]);
            answer_arc_of_fortune(&mut game, first_discards);
            run_to_decision(&mut game).expect("the second player is asked");

            let mut game = reconstructed(&game);
            let (second, _) = answer_arc_of_fortune(&mut game, false);
            drain_pending(&mut game);

            assert_eq!(second, PlayerId::Two);
            let expected = if first_discards {
                (7, 2, 3)
            } else {
                (2, 0, 10)
            };
            assert_eq!(piles(&game, PlayerId::One), expected);
            assert_eq!(piles(&game, PlayerId::Two), (3, 0, 10));
        }
    }
}

/// Discarding a hand of no cards is still discarding your hand.
#[test]
fn arc_of_fortune_lets_a_player_with_no_cards_in_hand_draw_seven() {
    for prepared_engine in [false, true] {
        let mut game = arc_of_fortune_cast(prepared_engine, [0, 1]);

        answer_arc_of_fortune(&mut game, true);
        answer_arc_of_fortune(&mut game, false);
        drain_pending(&mut game);

        assert_eq!(piles(&game, PlayerId::One), (7, 0, 3));
        assert_eq!(piles(&game, PlayerId::Two), (1, 0, 10));
    }
}

/// "Spells and abilities your opponents control can't cause you to discard
/// cards": a player who cannot discard their hand cannot take the option, so
/// choosing it leaves them where they were. Their own spell is no obstacle.
#[test]
fn arc_of_fortune_does_nothing_for_a_player_its_caster_cannot_make_discard() {
    for prepared_engine in [false, true] {
        let mut game = arc_of_fortune_cast(prepared_engine, [2, 3]);
        for player in [PlayerId::One, PlayerId::Two] {
            game.put_onto_battlefield(player, cards::TAMIYO_COLLECTOR_OF_TALES)
                .unwrap();
        }

        answer_arc_of_fortune(&mut game, true);
        answer_arc_of_fortune(&mut game, true);
        drain_pending(&mut game);

        assert_eq!(piles(&game, PlayerId::One), (7, 2, 3), "its caster");
        assert_eq!(piles(&game, PlayerId::Two), (3, 0, 10));
    }
}

/// Turn order is the active player's, whoever cast the spell: a sorcery
/// given flash resolves on the other player's turn.
#[test]
fn arc_of_fortune_asks_the_active_player_first_when_that_is_not_its_caster() {
    for prepared_engine in [false, true] {
        let mut game = arc_of_fortune_cast(prepared_engine, [2, 3]);
        game.active_player = PlayerId::Two;

        let (first, _) = answer_arc_of_fortune(&mut game, false);
        let (second, _) = answer_arc_of_fortune(&mut game, true);
        drain_pending(&mut game);

        assert_eq!((first, second), (PlayerId::Two, PlayerId::One));
        assert_eq!(piles(&game, PlayerId::One), (7, 2, 3));
        assert_eq!(piles(&game, PlayerId::Two), (3, 0, 10));
    }
}

/// "You may sacrifice a planeswalker. If you do, search your library for a
/// planeswalker card, put it onto the battlefield, then shuffle."
#[test]
fn entrust_the_spark_trades_a_planeswalker_for_one_from_your_library() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let jace = settled(&mut game, cards::JACE_MEMORY_ADEPT);
        game.players[0].library = game
            .build_zone(
                PlayerId::One,
                &[cards::FOREST, cards::CHANDRA_TORCH_OF_DEFIANCE],
            )
            .unwrap();

        cast(&mut game, cards::ENTRUST_THE_SPARK, &[]);
        run_to_decision(&mut game).expect("sacrificing a planeswalker is offered");
        choose_decision_by_label(&mut game, PlayerId::One, "Sacrifice Jace, Memory Adept");
        let search = run_to_decision(&mut game).expect("the search chooses its card");
        let found = search
            .options
            .iter()
            .filter(|option| option.card.is_some())
            .map(|option| option.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(found, ["Chandra, Torch of Defiance"], "not the Forest");
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, jace));
        assert_eq!(controlled(&game, cards::CHANDRA_TORCH_OF_DEFIANCE).len(), 1);
    }
}

/// Guiding Hydra with two +1/+1 counters, a creature beside it and one across
/// the table, stopped at its beginning-of-combat offer.
fn guiding_hydra_at_combat(prepared_engine: bool) -> (Game, [GameObjectId; 3]) {
    let mut game = setup(prepared_engine);
    let hydra = game
        .put_onto_battlefield(PlayerId::One, cards::GUIDING_HYDRA)
        .unwrap();
    permanent_mut(&mut game, hydra).add_counters(CounterKind::PlusOnePlusOne, 2);
    let bears = settled(&mut game, cards::GRIZZLY_BEARS);
    let theirs = game
        .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
        .unwrap();
    game.step = Step::PrecombatMain;
    game.advance_step();
    assert_eq!(game.step, Step::BeginningOfCombat);
    run_to_decision(&mut game).expect("removing a counter is offered");
    (game, [hydra, bears, theirs])
}

/// "You may remove a +1/+1 counter from this creature. If you do, put a +1/+1
/// counter on each other creature you control."
#[test]
fn guiding_hydra_spends_a_counter_to_put_one_on_each_other_creature_you_control() {
    for prepared_engine in [false, true] {
        let (mut game, [hydra, bears, theirs]) = guiding_hydra_at_combat(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Do it");
        drain_pending(&mut game);

        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(counters(hydra), 1, "one of its two is spent");
        assert_eq!(counters(bears), 1);
        assert_eq!(counters(theirs), 0, "not a creature you control");
    }
}

/// The counter comes off "this creature" whoever controls it by then, and
/// the new ones go to the creatures of the player whose trigger it is.
#[test]
fn guiding_hydra_taken_in_response_still_gives_up_its_counter() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let hydra = game
            .put_onto_battlefield(PlayerId::One, cards::GUIDING_HYDRA)
            .unwrap();
        permanent_mut(&mut game, hydra).add_counters(CounterKind::PlusOnePlusOne, 2);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        game.step = Step::PrecombatMain;
        game.advance_step();

        permanent_mut(&mut game, hydra).controller = PlayerId::Two;
        run_to_decision(&mut game).expect("removing a counter is still offered");
        choose_decision_by_label(&mut game, PlayerId::One, "Do it");
        drain_pending(&mut game);

        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(counters(hydra), 1);
        assert_eq!(counters(bears), 1);
    }
}

/// A Hydra kept alive by something else has no counter to remove, so "if
/// you do" never comes true.
#[test]
fn guiding_hydra_with_no_counter_to_remove_gives_none() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::CRUSADE);
        let hydra = settled(&mut game, cards::GUIDING_HYDRA);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);

        game.step = Step::PrecombatMain;
        game.advance_step();
        drain_pending(&mut game);

        assert!(on_battlefield(&game, hydra), "a white 2/1 under Crusade");
        assert_eq!(
            permanent(&game, bears).counters(CounterKind::PlusOnePlusOne),
            0
        );
    }
}

/// "Equipped creature gets +2/+0 and has 'Whenever this creature attacks, it
/// gains your choice of trample or deathtouch until end of turn.'"
#[test]
fn hunters_axe_gives_the_equipped_creature_its_controllers_choice_as_it_attacks() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let axe = settled(&mut game, cards::HUNTER_S_AXE);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let other = settled(&mut game, cards::GRIZZLY_BEARS);
        permanent_mut(&mut game, axe).attached_to = Some(bears);
        assert_eq!(game.power(permanent(&game, bears)), Some(4));

        declare_attack(&mut game, bears);
        run_to_decision(&mut game).expect("the keyword is chosen as the trigger resolves");
        choose_decision_by_label(&mut game, PlayerId::One, "Deathtouch");
        drain_pending(&mut game);

        let keyword =
            |id, keyword| game.permanent_has_executable_keyword(permanent(&game, id), keyword);
        assert!(keyword(bears, KeywordAbility::Deathtouch));
        assert!(!keyword(bears, KeywordAbility::Trample));
        assert!(!keyword(other, KeywordAbility::Deathtouch));
    }
}

/// Pia cast with a Forest in hand and a library holding an enchantment, as
/// far as the offer to discard.
fn pia_aether_ascetic_offering(prepared_engine: bool) -> Game {
    let mut game = setup(prepared_engine);
    game.players[0].library = game
        .build_zone(
            PlayerId::One,
            &[cards::FOREST, cards::GLORIOUS_ANTHEM, cards::GRIZZLY_BEARS],
        )
        .unwrap();
    hand(&mut game, cards::FOREST);
    cast(&mut game, cards::PIA_AETHER_ASCETIC, &[]);
    run_to_decision(&mut game).expect("the discard is offered");
    game
}

/// "You may discard a card. If you do, search your library for an
/// enchantment card, reveal it, put it into your hand, then shuffle."
#[test]
fn pia_aether_ascetic_trades_a_discard_for_an_enchantment_from_your_library() {
    for prepared_engine in [false, true] {
        let mut game = pia_aether_ascetic_offering(prepared_engine);
        let forest = game.players[0].hand[0].id;

        choose(&mut game, PlayerId::One, forest);
        let decision = run_to_decision(&mut game).expect("the search chooses");
        assert_eq!(decision.options.len(), 1, "only the enchantment card");
        drain_pending(&mut game);

        let hand = &game.players[0].hand;
        assert_eq!(hand.len(), 1);
        assert_eq!(hand[0].definition, cards::GLORIOUS_ANTHEM);
        assert_eq!(game.players[0].library.len(), 2);
    }
}

/// Something Worth Saving cast over a library whose top four cards are two
/// instants, a creature and a land, stopped at its choice among them.
fn something_worth_saving_choosing(prepared_engine: bool) -> (Game, DecisionObservation) {
    let mut game = setup(prepared_engine);
    game.players[0].library = game
        .build_zone(
            PlayerId::One,
            &[
                cards::GRIZZLY_BEARS,
                cards::LIGHTNING_BOLT,
                cards::FOREST,
                cards::GRIZZLY_BEARS,
                cards::LIGHTNING_BOLT,
            ],
        )
        .unwrap();
    cast(&mut game, cards::SOMETHING_WORTH_SAVING, &[]);
    let choice = run_to_decision(&mut game).expect("a milled permanent card may be taken");
    (game, choice)
}

/// "Mill four cards. You may put a permanent card from among them into your
/// hand. You gain 1 life."
#[test]
fn something_worth_saving_returns_one_milled_permanent_card() {
    for prepared_engine in [false, true] {
        let (mut game, choice) = something_worth_saving_choosing(prepared_engine);
        let life = game.players[0].life;
        assert_eq!(choice.options.len(), 2, "the land and the creature");
        assert_eq!((choice.minimum, choice.maximum), (0, 1));

        let taken = offered(&game, PlayerId::One)[0];
        let definition = game.players[0]
            .graveyard
            .iter()
            .find(|card| card.id == taken)
            .expect("it was milled")
            .definition;
        choose(&mut game, PlayerId::One, taken);
        drain_pending(&mut game);

        let player = &game.players[0];
        assert_eq!(player.library.len(), 1, "the fifth card stays");
        let hand = player.hand.iter().map(|card| card.definition);
        assert!(hand.eq([definition]));
        assert_eq!(
            player.graveyard.len(),
            4,
            "three milled cards and the spell"
        );
        assert_eq!(player.life, life + 1);
    }
}

#[test]
fn something_worth_saving_may_leave_every_card_and_still_gains_life() {
    for prepared_engine in [false, true] {
        let (mut game, choice) = something_worth_saving_choosing(prepared_engine);
        let life = game.players[0].life;

        game.apply(
            PlayerId::One,
            Action::ChooseDecision {
                decision: choice.id,
                options: Vec::new(),
            },
        )
        .unwrap();
        drain_pending(&mut game);

        assert!(game.players[0].hand.is_empty());
        assert_eq!(game.players[0].graveyard.len(), 5);
        assert_eq!(game.players[0].life, life + 1);
    }
}
