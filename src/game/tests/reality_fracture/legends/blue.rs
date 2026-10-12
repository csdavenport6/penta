//! Reality Fracture's blue legends.
use super::*;

/// "Whenever another nontoken creature you control enters, untap Arni."
#[test]
fn arni_humble_scribe_untaps_only_for_another_nontoken_creature_of_yours() {
    static SOLDIER: TokenCharacteristics =
        TokenCharacteristics::creature(&["Soldier"], &[ManaColor::White], 1, 1);
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let arni = settled(&mut game, cards::ARNI_HUMBLE_SCRIBE);
        permanent_mut(&mut game, arni).tapped = true;

        game.create_token_from(PlayerId::One, SOLDIER, None);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        settled(&mut game, cards::MOX_RUBY);
        assert!(permanent(&game, arni).tapped);

        settled(&mut game, cards::GRIZZLY_BEARS);
        assert!(!permanent(&game, arni).tapped);
    }
}

/// Fblthp under player one, whose library holds `library` cards, as a Grizzly
/// Bears of theirs deals combat damage to player two on their own turn.
fn fblthp_after_your_combat_damage(prepared_engine: bool, library: usize) -> (Game, GameObjectId) {
    let mut game = setup(prepared_engine);
    game.players[0].library = game
        .build_zone(PlayerId::One, &vec![cards::FOREST; library])
        .unwrap();
    let fblthp = settled(&mut game, cards::FBLTHP_IMPOSSIBLY_LOST);
    let bears = settled(&mut game, cards::GRIZZLY_BEARS);
    connect_with(&mut game, bears);
    drain_pending(&mut game);
    (game, fblthp)
}

/// "Draw two cards. If your library has no cards in it, you win the game."
#[test]
fn fblthp_impossibly_lost_wins_the_game_once_his_draws_empty_your_library() {
    for prepared_engine in [false, true] {
        let (game, _) = fblthp_after_your_combat_damage(prepared_engine, 2);

        assert_eq!(game.players[0].hand.len(), 2);
        assert!(matches!(
            game.result,
            Some(GameResult::Winner {
                winner: PlayerId::One,
                ..
            })
        ));
    }
}

/// "If you draw from an empty library this way, you still win the game": the
/// win happens while the ability is resolving, before state-based actions
/// would have you lose for the draw you could not make.
#[test]
fn fblthp_impossibly_lost_wins_before_a_draw_from_an_empty_library_can_lose() {
    for library in [0, 1] {
        for prepared_engine in [false, true] {
            let (game, _) = fblthp_after_your_combat_damage(prepared_engine, library);

            assert!(matches!(
                game.result,
                Some(GameResult::Winner {
                    winner: PlayerId::One,
                    ..
                })
            ));
        }
    }
}

/// "Fblthp's owner shuffles him into their library", when cards are left in
/// it and the game goes on.
#[test]
fn fblthp_impossibly_lost_is_shuffled_into_a_library_that_still_has_cards() {
    for prepared_engine in [false, true] {
        let (game, fblthp) = fblthp_after_your_combat_damage(prepared_engine, 5);

        assert_eq!(game.result, None);
        assert_eq!(game.players[0].hand.len(), 2);
        assert!(!on_battlefield(&game, fblthp));
        let library = &game.players[0].library;
        assert_eq!(library.len(), 4, "the three left and Fblthp");
        assert!(
            library
                .iter()
                .any(|card| card.definition == cards::FBLTHP_IMPOSSIBLY_LOST)
        );
    }
}

/// "When one or more of your opponents are dealt combat damage during your
/// turn": combat damage dealt to you, on an opponent's turn, is neither.
#[test]
fn fblthp_impossibly_lost_ignores_combat_damage_dealt_to_you_on_another_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let fblthp = settled(&mut game, cards::FBLTHP_IMPOSSIBLY_LOST);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        permanent_mut(&mut game, theirs).entered_controller_turn = 0;
        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        game.step = Step::DeclareAttackers;

        game.apply(
            PlayerId::Two,
            Action::DeclareAttacker {
                attacker: theirs,
                defender: AttackDefender::Player(PlayerId::One),
            },
        )
        .unwrap();
        game.finish_declaring_attackers();
        game.finish_declaring_blockers();
        game.deal_combat_damage();
        drain_pending(&mut game);

        assert_eq!(game.players[0].life, 18);
        assert!(game.players[0].hand.is_empty());
        assert!(on_battlefield(&game, fblthp));
    }
}

/// "When Hapatra enters, for each opponent, tap up to one target creature
/// that player controls. Put a stun counter on each of those creatures."
#[test]
fn hapatra_the_desert_frost_taps_and_stuns_up_to_one_creature_an_opponent_controls() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::HAPATRA_THE_DESERT_FROST, &[]);
        let decision = run_to_decision(&mut game).expect("the trigger chooses its target");
        assert_eq!((decision.minimum, decision.maximum), (0, 1), "up to one");
        assert_eq!(offered(&game, PlayerId::One), [theirs]);
        choose(&mut game, PlayerId::One, theirs);
        drain_pending(&mut game);

        let theirs = permanent(&game, theirs);
        assert!(theirs.tapped);
        assert_eq!(theirs.counters(CounterKind::Stun), 1);
    }
}

/// Lyra under player one at the end of `player`'s turn, player one having
/// drawn `drawn` cards during it.
fn lyra_tolarian_at_end_step_of(prepared_engine: bool, player: PlayerId, drawn: u16) -> Game {
    let mut game = setup(prepared_engine);
    stock_library(&mut game);
    settled(&mut game, cards::LYRA_TOLARIAN_ARCHANGEL);
    game.active_player = player;
    game.priority = player;
    game.draw_cards(PlayerId::One, drawn);
    end_step_begins(&mut game);
    drain_pending(&mut game);
    game
}

/// "At the beginning of each end step, if you've drawn three or more cards
/// this turn, create a 3/3 blue Angel creature token with flying."
#[test]
fn lyra_tolarian_archangel_makes_an_angel_at_any_end_step_after_three_draws() {
    for player in [PlayerId::One, PlayerId::Two] {
        for prepared_engine in [false, true] {
            let game = lyra_tolarian_at_end_step_of(prepared_engine, player, 3);

            let angels = tokens(&game);
            assert_eq!(angels.len(), 1);
            assert_eq!(
                (game.power(angels[0]), game.toughness(angels[0])),
                (Some(3), Some(3))
            );
            assert!(has_keyword(
                &game,
                angels[0].card.id,
                KeywordAbility::Flying
            ));
        }
    }
}

#[test]
fn lyra_tolarian_archangel_makes_nothing_after_only_two_draws() {
    for prepared_engine in [false, true] {
        let game = lyra_tolarian_at_end_step_of(prepared_engine, PlayerId::One, 2);

        assert!(tokens(&game).is_empty());
    }
}

/// "Until end of turn, whenever Lyra deals combat damage to a player, draw
/// two cards": nothing without the activation, and two for each time it was
/// activated.
#[test]
fn lyra_tolarian_archangel_draws_two_for_her_combat_damage_once_activated() {
    for (activations, drawn) in [(0, 0), (1, 2), (2, 4)] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            stock_library(&mut game);
            let lyra = settled(&mut game, cards::LYRA_TOLARIAN_ARCHANGEL);
            for _ in 0..activations {
                game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 5);
                activate(&mut game, lyra, "{3}{U}{U}:");
                drain_pending(&mut game);
            }

            connect_with(&mut game, lyra);
            drain_pending(&mut game);

            assert_eq!(game.players[1].life, 17);
            assert_eq!(game.players[0].hand.len(), drawn);
        }
    }
}

/// "Prowess, prowess": two instances, each its own trigger.
#[test]
fn ruric_thar_biomagus_gets_plus_two_plus_two_for_a_noncreature_spell() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ruric = settled(&mut game, cards::RURIC_THAR_BIOMAGUS);

        cast(&mut game, cards::MOX_RUBY, &[]);
        drain_pending(&mut game);

        let ruric = permanent(&game, ruric);
        assert_eq!(
            (game.power(ruric), game.toughness(ruric)),
            (Some(6), Some(8))
        );
    }
}

/// "Whenever Ruric Thar becomes the target of a spell or ability an opponent
/// controls, draw a card": your own spell aimed at him draws nothing.
#[test]
fn ruric_thar_biomagus_draws_when_an_opponent_targets_him() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let ruric = settled(&mut game, cards::RURIC_THAR_BIOMAGUS);

        cast(&mut game, cards::GIANT_GROWTH, &[Target::Permanent(ruric)]);
        drain_pending(&mut game);
        assert!(game.players[0].hand.is_empty(), "your own spell");

        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        let bolt = in_hand_of(&mut game, PlayerId::Two, cards::LIGHTNING_BOLT);
        game.add_unrestricted_mana(PlayerId::Two, ManaColor::Red, 1);
        let action = game
            .legal_actions(PlayerId::Two)
            .into_iter()
            .find(|action| {
                matches!(action, Action::CastSpell { card, choices, .. }
                    if *card == bolt && choices.iter_targets().eq(&[Target::Permanent(ruric)]))
            })
            .expect("the opponent can aim it at Ruric Thar");
        game.apply(PlayerId::Two, action).unwrap();
        drain_pending(&mut game);

        assert_eq!(game.players[0].hand.len(), 1);
    }
}

/// "This spell costs {2} less to cast if you've cast a noncreature spell this
/// turn": a creature spell earlier in the turn does not count.
#[test]
fn traxos_academy_guardian_costs_two_less_after_a_noncreature_spell_this_turn() {
    for (earlier, discounted) in [
        (None, false),
        (Some(cards::MEMNITE), false),
        (Some(cards::MOX_RUBY), true),
    ] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            if let Some(earlier) = earlier {
                cast(&mut game, earlier, &[]);
                drain_pending(&mut game);
            }
            for player in &mut game.players {
                player.mana_pool = ManaPool::default();
                player.mana.clear();
            }
            let traxos = hand(&mut game, cards::TRAXOS_ACADEMY_GUARDIAN);

            game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 2);
            assert_eq!(can_cast(&game, PlayerId::One, traxos), discounted);
            game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 2);
            assert!(can_cast(&game, PlayerId::One, traxos));
        }
    }
}

/// "Target creature gets -X/-0 until end of turn, where X is the number of
/// cards in your graveyard."
#[test]
fn yuriko_hope_from_the_shadows_shrinks_a_creature_by_your_graveyard() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        put_in_graveyard(
            &mut game,
            &[cards::FOREST, cards::GRIZZLY_BEARS, cards::OPT],
        );
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();

        cast(&mut game, cards::YURIKO_HOPE_FROM_THE_SHADOWS, &[]);
        run_to_decision(&mut game).expect("the trigger chooses its mode");
        choose_decision_by_label(
            &mut game,
            PlayerId::One,
            "Target creature gets -X/-0 until end of turn, where X is the number of cards in \
             your graveyard.",
        );
        run_to_decision(&mut game).expect("and then its target");
        choose(&mut game, PlayerId::One, angel);
        drain_pending(&mut game);

        let angel = permanent(&game, angel);
        assert_eq!(
            (game.power(angel), game.toughness(angel)),
            (Some(1), Some(4))
        );
    }
}

/// "Noncreature spells you cast cost {1} less to cast": not a creature spell,
/// and not an opponent's spell.
#[test]
fn geist_of_saint_thalia_discounts_only_your_noncreature_spells() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::GEIST_OF_SAINT_THALIA);
        let divination = hand(&mut game, cards::DIVINATION);
        let bears = hand(&mut game, cards::GRIZZLY_BEARS);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 1);
        assert!(!can_cast(&game, PlayerId::One, bears), "{{1}}{{G}} still");
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 1);
        assert!(
            can_cast(&game, PlayerId::One, divination),
            "{{2}}{{U}} less {{1}}"
        );

        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        let theirs = in_hand_of(&mut game, PlayerId::Two, cards::DIVINATION);
        game.add_unrestricted_mana(PlayerId::Two, ManaColor::Blue, 2);
        assert!(!can_cast(&game, PlayerId::Two, theirs));
    }
}

/// A Fblthp who has left before his trigger resolves is a new object where
/// he lies, so he stays there; the library is shuffled all the same.
#[test]
fn fblthp_impossibly_lost_stays_where_he_went_if_he_left_before_resolving() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let fblthp = settled(&mut game, cards::FBLTHP_IMPOSSIBLY_LOST);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        connect_with(&mut game, bears);
        game.destroy_permanent(fblthp);
        drain_pending(&mut game);

        assert_eq!(game.players[0].hand.len(), 2);
        assert_eq!(game.players[0].library.len(), 8);
        assert_eq!(game.players[0].graveyard.len(), 1);
        assert_eq!(
            game.players[0].graveyard[0].definition,
            cards::FBLTHP_IMPOSSIBLY_LOST
        );
    }
}
