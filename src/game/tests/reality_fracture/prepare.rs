//! Reality Fracture's preparation cards: each prepare spell cast from the
//! copy its permanent made, and each way one of these cards becomes prepared.
//!
//! The rulings every preparation card shares are held once in
//! `prepare_cards.rs`; what is here is what these cards' own clauses add.
use super::*;

/// Puts player one's preparation card `definition` onto the battlefield,
/// where it enters prepared, and returns it with the copy of `spell` that
/// made in exile.
fn prepared_on_entry(
    game: &mut Game,
    definition: CardDefinitionId,
    spell: &str,
) -> (GameObjectId, GameObjectId) {
    let before = exiled_copies(game, PlayerId::One);
    let permanent = settled(game, definition);
    assert!(is_prepared(game, permanent), "it enters prepared");
    let made = exiled_copies(game, PlayerId::One)
        .into_iter()
        .filter(|copy| !before.contains(copy))
        .collect::<Vec<_>>();
    assert_eq!(made.len(), 1, "one copy of its prepare spell");
    assert_eq!(made[0].1, spell);
    (permanent, made[0].0)
}

/// Player one casts the exiled prepare-spell copy `copy` at `targets` with
/// mana to spare, leaving it on the stack.
fn cast_copy(game: &mut Game, copy: GameObjectId, targets: &[Target]) {
    mana_to_spare(game);
    let action = casts(game, copy)
        .into_iter()
        .find(|action| {
            matches!(action, Action::CastSpell { choices, .. }
            if choices.iter_targets().eq(targets))
        })
        .expect("the copy can be cast at those targets");
    game.apply(PlayerId::One, action).unwrap();
}

/// "Put a +1/+1 counter on target creature. You gain 1 life."
#[test]
fn seed_suture_puts_a_counter_on_its_target_and_gains_its_caster_a_life() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let (angel, suture) =
            prepared_on_entry(&mut game, cards::BLOSSOM_BLESSED_ANGEL, "Seed Suture");
        let life = game.players[0].life;

        cast_copy(&mut game, suture, &[Target::Permanent(angel)]);
        assert!(
            !is_prepared(&game, angel),
            "casting its spell unprepares it"
        );
        drain_pending(&mut game);

        assert_eq!(
            permanent(&game, angel).counters(CounterKind::PlusOnePlusOne),
            1
        );
        assert_eq!(game.players[0].life, life + 1);
        assert!(exiled_copies(&game, PlayerId::One).is_empty());
    }
}

/// "Each creature you control with a +1/+1 counter on it has vigilance": the
/// Vanguard's own Seed Suture is what hands the counter out.
#[test]
fn vigorbloom_vanguard_gives_vigilance_to_your_creatures_that_carry_a_counter() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let (vanguard, suture) =
            prepared_on_entry(&mut game, cards::VIGORBLOOM_VANGUARD, "Seed Suture");
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        permanent_mut(&mut game, theirs).add_counters(CounterKind::PlusOnePlusOne, 1);
        assert!(!has_keyword(&game, bears, KeywordAbility::Vigilance));

        cast_copy(&mut game, suture, &[Target::Permanent(bears)]);
        drain_pending(&mut game);

        assert!(has_keyword(&game, bears, KeywordAbility::Vigilance));
        assert!(
            !has_keyword(&game, vanguard, KeywordAbility::Vigilance),
            "the Vanguard carries no counter of its own"
        );
        assert!(
            !has_keyword(&game, theirs, KeywordAbility::Vigilance),
            "not a creature you control"
        );
    }
}

/// "Create a 2/2 colorless Wizard Soldier creature token named Cadet. Surveil
/// 1."
#[test]
fn peer_review_creates_a_cadet_and_then_surveils() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let (_, review) = prepared_on_entry(&mut game, cards::FATEHOLD_CHRONOLOGIST, "Peer Review");

        cast_copy(&mut game, review, &[]);
        run_to_decision(&mut game).expect("the surveil asks about the top card");

        let made = tokens(&game);
        let [cadet] = made[..] else {
            panic!("the Cadet is made before the surveil");
        };
        assert_eq!(
            game.object_card_name(cadet.card.id).as_deref(),
            Some("Cadet")
        );
        assert_eq!(
            (game.power(cadet), game.toughness(cadet)),
            (Some(2), Some(2))
        );
        assert!(colors(&game, cadet).is_empty(), "colorless");
        surveil(&mut game, true);
        assert_eq!(game.players[0].graveyard.len(), 1);
    }
}

/// "Mill three cards", and what that does for the Threshold of the creature
/// that cast it: "This creature gets +1/+1 as long as there are seven or more
/// cards in your graveyard."
#[test]
fn omit_variables_mills_three_and_can_bring_void_extrapolator_to_threshold() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let (extrapolator, omit) =
            prepared_on_entry(&mut game, cards::VOID_EXTRAPOLATOR, "Omit Variables");
        put_in_graveyard(&mut game, &[cards::MOUNTAIN; 4]);
        let stats = |game: &Game| {
            let extrapolator = permanent(game, extrapolator);
            (game.power(extrapolator), game.toughness(extrapolator))
        };
        assert_eq!(stats(&game), (Some(2), Some(2)), "four cards is short");

        cast_copy(&mut game, omit, &[]);
        drain_pending(&mut game);

        assert_eq!(game.players[0].graveyard.len(), 7);
        assert_eq!(game.players[0].library.len(), 7);
        assert_eq!(stats(&game), (Some(3), Some(3)));
    }
}

/// "Threshold — This creature gets +1/+0 and has flying as long as there are
/// seven or more cards in your graveyard."
#[test]
fn theorix_metamage_grows_and_flies_only_at_threshold() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let metamage = settled(&mut game, cards::THEORIX_METAMAGE);
        let stats = |game: &Game| {
            let metamage = permanent(game, metamage);
            (game.power(metamage), game.toughness(metamage))
        };

        put_in_graveyard(&mut game, &[cards::MOUNTAIN; 6]);
        assert_eq!(stats(&game), (Some(2), Some(3)));
        assert!(!has_keyword(&game, metamage, KeywordAbility::Flying));

        put_in_graveyard(&mut game, &[cards::MOUNTAIN]);
        assert_eq!(stats(&game), (Some(3), Some(3)));
        assert!(has_keyword(&game, metamage, KeywordAbility::Flying));
    }
}

/// "At the beginning of your upkeep, if this creature isn't prepared, it
/// becomes prepared": the other way one of these cards becomes prepared. It
/// arrives without a copy of its spell and has one from its first upkeep.
#[test]
fn paradox_shaper_enters_unprepared_and_prepares_itself_at_upkeep() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let shaper = settled(&mut game, cards::PARADOX_SHAPER);
        assert!(!is_prepared(&game, shaper));
        assert!(exiled_copies(&game, PlayerId::One).is_empty());

        assert_eq!(take_an_upkeep(&mut game), 1);

        assert!(is_prepared(&game, shaper));
        let copies = exiled_copies(&game, PlayerId::One);
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].1, "Omit Variables");
    }
}

/// "{2}: Put target card from your graveyard on the bottom of your library."
#[test]
fn paradox_shaper_buries_a_card_from_your_own_graveyard_under_your_library() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let shaper = settled(&mut game, cards::PARADOX_SHAPER);
        let bolt = put_in_graveyard(&mut game, &[cards::LIGHTNING_BOLT])[0];
        let theirs = game
            .build_zone(PlayerId::Two, &[cards::LIGHTNING_BOLT])
            .unwrap();
        game.players[1].graveyard.extend(theirs);
        mana_to_spare(&mut game);

        assert_eq!(
            activation_targets(&game, shaper, "{2}"),
            [[Target::Card(bolt)]],
            "only a card in your graveyard"
        );
        activate_at(&mut game, shaper, "{2}", &[Target::Card(bolt)]);
        drain_pending(&mut game);

        let player = &game.players[0];
        assert!(player.graveyard.is_empty());
        assert_eq!(player.library.len(), 11);
        assert_eq!(
            player.library[0].definition,
            cards::LIGHTNING_BOLT,
            "under the ten Forests"
        );
    }
}

/// "Vicious Verse deals 1 damage to target opponent."
#[test]
fn vicious_verse_deals_one_damage_to_an_opponent_and_never_to_its_caster() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let (_, verse) = prepared_on_entry(&mut game, cards::HALLWAY_HECKLER, "Vicious Verse");
        mana_to_spare(&mut game);
        assert_eq!(
            castable_targets(&game, verse),
            [[Target::Player(PlayerId::Two)]]
        );

        cast_copy(&mut game, verse, &[Target::Player(PlayerId::Two)]);
        drain_pending(&mut game);

        assert_eq!(game.players[1].life, 19);
        assert_eq!(game.players[0].life, 20);
    }
}

/// Pompous Battlemage's Improvised Act cast with `hand` cards in hand, as far
/// as whatever it asks. Returns the Battlemage.
fn improvised_act_cast(prepared_engine: bool, hand_size: usize) -> (Game, GameObjectId) {
    let mut game = setup(prepared_engine);
    stock_library(&mut game);
    let (battlemage, act) =
        prepared_on_entry(&mut game, cards::POMPOUS_BATTLEMAGE, "Improvised Act");
    for _ in 0..hand_size {
        hand(&mut game, cards::MOUNTAIN);
    }
    cast_copy(&mut game, act, &[]);
    (game, battlemage)
}

/// "You may discard a card. If you do, draw a card."
#[test]
fn improvised_act_draws_a_card_for_the_one_discarded() {
    for prepared_engine in [false, true] {
        let (mut game, _) = improvised_act_cast(prepared_engine, 1);
        run_to_decision(&mut game).expect("the discard is offered");
        let mountain = game.players[0].hand[0].id;

        choose(&mut game, PlayerId::One, mountain);
        drain_pending(&mut game);

        let player = &game.players[0];
        assert_eq!(player.graveyard.len(), 1);
        assert_eq!(player.hand.len(), 1);
        assert_eq!(player.hand[0].definition, cards::FOREST, "the card drawn");
    }
}

#[test]
fn improvised_act_draws_nothing_when_no_card_is_discarded() {
    for prepared_engine in [false, true] {
        let (mut game, _) = improvised_act_cast(prepared_engine, 1);
        run_to_decision(&mut game).expect("the discard is offered");
        choose_decision_by_label(&mut game, PlayerId::One, "Decline");
        assert!(run_to_decision(&mut game).is_none());
        assert_eq!(game.players[0].hand.len(), 1);
        assert_eq!(game.players[0].library.len(), 10);

        let (mut game, _) = improvised_act_cast(prepared_engine, 0);
        drain_pending(&mut game);
        assert!(game.players[0].hand.is_empty(), "nothing to discard");
        assert_eq!(game.players[0].library.len(), 10);
    }
}

/// The copy of a prepare spell is cast, so the creature it came from sees a
/// noncreature spell cast like any other.
#[test]
fn pompous_battlemage_grows_from_casting_its_own_prepare_spell() {
    for prepared_engine in [false, true] {
        let (mut game, battlemage) = improvised_act_cast(prepared_engine, 0);
        drain_pending(&mut game);

        let battlemage = permanent(&game, battlemage);
        assert_eq!(
            (game.power(battlemage), game.toughness(battlemage)),
            (Some(2), Some(2)),
            "prowess"
        );
    }
}

/// "Search your library for a land card, put it into your graveyard, then
/// shuffle."
#[test]
fn enroot_puts_a_land_card_from_your_library_into_your_graveyard() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let (_, enroot) = prepared_on_entry(&mut game, cards::CARNIVOROUS_CULTIVATOR, "Enroot");
        game.players[0].library = game
            .build_zone(
                PlayerId::One,
                &[cards::LIGHTNING_BOLT, cards::FOREST, cards::GRIZZLY_BEARS],
            )
            .unwrap();

        cast_copy(&mut game, enroot, &[]);
        let search = run_to_decision(&mut game).expect("the search chooses its card");
        assert_eq!(search.options.len(), 1, "only the land card");
        drain_pending(&mut game);

        let player = &game.players[0];
        assert_eq!(player.graveyard.len(), 1);
        assert_eq!(player.graveyard[0].definition, cards::FOREST);
        assert_eq!(player.library.len(), 2);
    }
}

/// "Whenever this creature deals combat damage to a player, return target
/// land card from your graveyard to your hand."
#[test]
fn carnivorous_cultivator_returns_a_land_card_from_your_graveyard_when_it_connects() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let cultivator = settled(&mut game, cards::CARNIVOROUS_CULTIVATOR);
        let forest = put_in_graveyard(&mut game, &[cards::LIGHTNING_BOLT, cards::FOREST])[1];
        let theirs = game.build_zone(PlayerId::Two, &[cards::FOREST]).unwrap();
        game.players[1].graveyard.extend(theirs);

        connect_with(&mut game, cultivator);
        run_to_decision(&mut game).expect("the trigger chooses its target");
        assert_eq!(offered(&game, PlayerId::One), [forest]);
        choose(&mut game, PlayerId::One, forest);
        drain_pending(&mut game);

        let player = &game.players[0];
        assert_eq!(player.hand.len(), 1);
        assert_eq!(player.hand[0].definition, cards::FOREST);
        assert_eq!(player.graveyard.len(), 1, "the Bolt stays");
    }
}

/// "Create a Heartwood token."
#[test]
fn soul_tether_creates_a_heartwood_token() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let (_, tether) = prepared_on_entry(&mut game, cards::KONSTRARI_IMPROVISER, "Soul Tether");

        cast_copy(&mut game, tether, &[]);
        drain_pending(&mut game);

        let made = tokens(&game);
        let [heartwood] = made[..] else {
            panic!("one token is made");
        };
        assert_eq!(
            game.object_card_name(heartwood.card.id).as_deref(),
            Some("Heartwood")
        );
    }
}

/// Variable Chaser's Arc of Fortune on the stack, cast by player one, with
/// this many cards in each player's hand and ten in each library.
fn arc_of_fortune_cast(prepared_engine: bool, hands: [usize; 2]) -> Game {
    let mut game = setup(prepared_engine);
    let (_, arc) = prepared_on_entry(&mut game, cards::VARIABLE_CHASER, "Arc of Fortune");
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
