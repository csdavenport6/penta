//! What a resolving spell or ability reads, and when: counts and greatest
//! values, what its own earlier instructions just changed, and which objects
//! its effect reaches.
use super::*;

/// Player one answers a surveil 1 by putting the card into their graveyard,
/// or by leaving it on top of their library.
fn surveil(game: &mut Game, into_graveyard: bool) {
    let decision = game
        .observe(PlayerId::One)
        .decision
        .expect("the surveil is waiting");
    let options = if into_graveyard {
        vec![decision.options[0].id]
    } else {
        Vec::new()
    };
    game.apply(
        PlayerId::One,
        Action::ChooseDecision {
            decision: decision.id,
            options,
        },
    )
    .unwrap();
    drain_pending(game);
}

/// "{6}: Create a Heartwood token. Then Aerid Konstrari gets +X/+0 until end
/// of turn, where X is the number of artifacts you control": the token just
/// made is one of them.
#[test]
fn aerid_konstrari_counts_the_heartwood_its_ability_just_created() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let aerid = settled(&mut game, cards::AERID_KONSTRARI);
        assert_eq!(tokens(&game).len(), 1, "one Heartwood for entering");
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 6);

        activate(&mut game, aerid, "{6}");
        drain_pending(&mut game);

        assert_eq!(tokens(&game).len(), 2);
        assert_eq!(game.power(permanent(&game, aerid)), Some(7));
    }
}

/// "Consider all creatures and planeswalkers the target opponent controls as
/// one group to see which one has the greatest mana value. For example, if
/// the opponent controls a creature with mana value 3 and a planeswalker with
/// mana value 5, they must sacrifice the planeswalker."
#[test]
fn break_under_pressure_takes_the_greatest_mana_value_across_both_types() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        let jace = game
            .put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
            .unwrap();
        let life = game.players[0].life;

        cast(
            &mut game,
            cards::BREAK_UNDER_PRESSURE,
            &[Target::Player(PlayerId::Two)],
        );
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, jace), "mana value five is greatest");
        assert!(on_battlefield(&game, bears), "mana value two is not");
        assert_eq!(game.players[0].life, life + 2, "you gain 2 life");
    }
}

/// A tie for the greatest mana value is the opponent's to break, and nothing
/// below it is offered.
#[test]
fn break_under_pressure_lets_the_opponent_break_a_tie_for_greatest() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();
        let jace = game
            .put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
            .unwrap();
        // Your own six-drop is not among "creatures and planeswalkers they
        // control" and must not raise the bar.
        let yours = game
            .put_onto_battlefield(PlayerId::One, cards::CRAW_WURM)
            .unwrap();

        cast(
            &mut game,
            cards::BREAK_UNDER_PRESSURE,
            &[Target::Player(PlayerId::Two)],
        );
        run_to_decision(&mut game).expect("the opponent chooses which");

        assert_eq!(offered(&game, PlayerId::Two), [angel, jace]);
        choose(&mut game, PlayerId::Two, angel);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, angel));
        assert!(on_battlefield(&game, jace));
        assert!(on_battlefield(&game, bears));
        assert_eq!(permanent(&game, yours).controller, PlayerId::One);
    }
}

/// With nothing to sacrifice the rest of the spell still happens.
#[test]
fn break_under_pressure_gains_life_when_the_opponent_has_nothing_to_sacrifice() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let forest = game
            .put_onto_battlefield(PlayerId::Two, cards::FOREST)
            .unwrap();
        let life = game.players[0].life;

        cast(
            &mut game,
            cards::BREAK_UNDER_PRESSURE,
            &[Target::Player(PlayerId::Two)],
        );
        drain_pending(&mut game);

        assert!(on_battlefield(&game, forest), "a land is neither");
        assert_eq!(game.players[0].life, life + 2);
    }
}

/// "Creatures you control gain trample and get +X/+0 until end of turn, where
/// X is the number of artifacts you control." X and the creatures it reaches
/// are both settled as the trigger resolves (CR 608.2h, 611.2c).
#[test]
fn craterclaw_colossus_fixes_its_bonus_and_its_creatures_as_the_trigger_resolves() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let mox = settled(&mut game, cards::MOX_RUBY);
        game.put_onto_battlefield(PlayerId::Two, cards::MOX_RUBY)
            .unwrap();
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::CRATERCLAW_COLOSSUS, &[]);
        drain_pending(&mut game);
        let colossus = controlled(&game, cards::CRATERCLAW_COLOSSUS)[0];

        // The Colossus is an artifact too: your Mox and itself make two.
        let power = |game: &Game, id| game.power(permanent(game, id));
        assert_eq!(power(&game, bears), Some(4));
        assert_eq!(power(&game, colossus), Some(7));
        assert!(has_keyword(&game, bears, KeywordAbility::Trample));
        assert_eq!(power(&game, theirs), Some(2), "not a creature you control");

        // Losing an artifact afterwards takes nothing back, and a creature
        // that arrives afterwards was not there to be given anything.
        game.destroy_permanent(mox);
        let late = settled(&mut game, cards::GRIZZLY_BEARS);
        assert_eq!(power(&game, bears), Some(4));
        assert_eq!(power(&game, late), Some(2));
        assert!(!has_keyword(&game, late, KeywordAbility::Trample));
    }
}

/// "Search your library for a card, put it into your hand, shuffle, then
/// discard a card at random": the card just found is in hand to be lost.
#[test]
fn curse_marred_demon_can_discard_the_card_it_searched_for() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.players[0].library = game
            .build_zone(PlayerId::One, &[cards::FOREST, cards::LIGHTNING_BOLT])
            .unwrap();

        cast(&mut game, cards::CURSE_MARRED_DEMON, &[]);
        let search = run_to_decision(&mut game).expect("the search chooses its card");
        assert_eq!((search.minimum, search.maximum), (1, 1), "any card will do");
        let bolt = search
            .options
            .iter()
            .find(|option| option.label == "Lightning Bolt")
            .expect("every card in the library is offered")
            .id;
        game.apply(
            PlayerId::One,
            Action::ChooseDecision {
                decision: search.id,
                options: vec![bolt],
            },
        )
        .unwrap();
        drain_pending(&mut game);

        let player = &game.players[0];
        assert!(player.hand.is_empty(), "the only card in hand is discarded");
        assert_eq!(player.library.len(), 1);
        let graveyard = player.graveyard.iter().map(|card| card.definition);
        assert!(graveyard.eq([cards::LIGHTNING_BOLT]));
    }
}

/// "{4}{G}: Put a +1/+1 counter on each creature you control with a +1/+1
/// counter on it."
#[test]
fn edgar_moonlit_sovereign_adds_to_each_of_your_creatures_that_already_has_a_counter() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let edgar = settled(&mut game, cards::EDGAR_MOONLIT_SOVEREIGN);
        let with = settled(&mut game, cards::GRIZZLY_BEARS);
        let without = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        for id in [with, theirs] {
            permanent_mut(&mut game, id).add_counters(CounterKind::PlusOnePlusOne, 1);
        }
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 5);

        activate(&mut game, edgar, "{4}{G}:");
        drain_pending(&mut game);

        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!([edgar, with, without, theirs].map(counters), [0, 2, 0, 1]);
    }
}

/// Eye of Jace's upkeep trigger over a graveyard already holding `graveyard`
/// cards, stopped at its surveil.
fn eye_of_jace_surveilling(prepared_engine: bool, graveyard: usize) -> (Game, GameObjectId) {
    let mut game = setup(prepared_engine);
    stock_library(&mut game);
    let eye = settled(&mut game, cards::EYE_OF_JACE);
    put_in_graveyard(&mut game, &vec![cards::MOUNTAIN; graveyard]);
    game.commit_next_turn(PlayerId::One, Vec::new());
    run_to_decision(&mut game).expect("the surveil asks about the top card");
    (game, eye)
}

/// "Surveil 1. Then if there are seven or more cards in your graveyard,
/// sacrifice this artifact, it deals 2 damage to each opponent, and you gain
/// 2 life": the count is taken after the surveil, so the card it just put
/// there is the seventh, and the Eye deals its damage from the graveyard.
#[test]
fn eye_of_jace_counts_the_card_its_own_surveil_put_into_the_graveyard() {
    for prepared_engine in [false, true] {
        let (mut game, eye) = eye_of_jace_surveilling(prepared_engine, 6);

        surveil(&mut game, true);

        assert!(!on_battlefield(&game, eye));
        assert_eq!(game.players[1].life, 18);
        assert_eq!(game.players[0].life, 22);
    }
}

#[test]
fn eye_of_jace_stays_while_the_graveyard_is_short_of_seven() {
    for prepared_engine in [false, true] {
        let (mut game, eye) = eye_of_jace_surveilling(prepared_engine, 6);

        surveil(&mut game, false);

        assert!(on_battlefield(&game, eye));
        assert_eq!(game.players[1].life, 20);
        assert_eq!(game.players[0].life, 20);
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

/// "Germinate Recruits counts the total amount of life gained without taking
/// into account any life you lost during that turn. For example, if you lost
/// 3 life and gained 3 life earlier in the turn, you'll create three tokens."
#[test]
fn germinate_recruits_counts_life_gained_whatever_was_lost() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.lose_life(PlayerId::One, 3);
        game.gain_life(PlayerId::One, 3);

        cast(&mut game, cards::GERMINATE_RECRUITS, &[]);
        drain_pending(&mut game);

        let cadets = tokens(&game);
        assert_eq!(cadets.len(), 3);
        for cadet in cadets {
            assert_eq!(
                (game.power(cadet), game.toughness(cadet)),
                (Some(2), Some(2))
            );
        }
    }
}

/// "Put X -1/-1 counters on up to one target creature that player controls,
/// where X is the greatest mana value among cards in your graveyard."
#[test]
fn hapatra_the_desert_fang_counts_the_greatest_mana_value_in_your_graveyard() {
    for (graveyard, counters) in [
        (
            &[cards::GRIZZLY_BEARS, cards::HILL_GIANT, cards::FOREST][..],
            4,
        ),
        (&[][..], 0),
    ] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            put_in_graveyard(&mut game, graveyard);
            let theirs = game.build_zone(PlayerId::Two, &[cards::CRAW_WURM]).unwrap();
            game.players[1].graveyard.extend(theirs);
            let wall = game
                .put_onto_battlefield(PlayerId::Two, cards::WALL_OF_STONE)
                .unwrap();

            cast(&mut game, cards::HAPATRA_THE_DESERT_FANG, &[]);
            let decision = run_to_decision(&mut game).expect("the trigger chooses its target");
            assert_eq!((decision.minimum, decision.maximum), (0, 1), "up to one");
            choose(&mut game, PlayerId::One, wall);
            drain_pending(&mut game);

            assert_eq!(
                permanent(&game, wall).counters(CounterKind::MinusOneMinusOne),
                counters
            );
        }
    }
}

/// "Create a ... Cadet. Then creatures you control gain haste until end of
/// turn": the Cadet is there to gain it, and a later arrival is not.
#[test]
fn ingris_stingerquill_gives_the_cadet_it_creates_haste() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ingris = settled(&mut game, cards::INGRIS_STINGERQUILL);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 4);

        activate(&mut game, ingris, "{4}");
        drain_pending(&mut game);

        let cadet = tokens(&game)[0].card.id;
        assert!(has_keyword(&game, cadet, KeywordAbility::Haste));
        let late = settled(&mut game, cards::GRIZZLY_BEARS);
        assert!(!has_keyword(&game, late, KeywordAbility::Haste));
    }
}

/// A token for the clauses that ask whether a creature is one.
static SOLDIER: TokenCharacteristics =
    TokenCharacteristics::creature(&["Soldier"], &[ManaColor::White], 1, 1);

/// "At the beginning of your end step, untap all tokens you control."
#[test]
fn jiang_yanggu_never_alone_untaps_your_tokens_at_your_end_step() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::JIANG_YANGGU_NEVER_ALONE);
        let mowu = tokens(&game)[0].card.id;
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game.create_token_from(PlayerId::Two, SOLDIER, None);
        for id in [mowu, bears, theirs] {
            permanent_mut(&mut game, id).tapped = true;
        }

        end_step_begins(&mut game);
        drain_pending(&mut game);

        let tapped = |id| permanent(&game, id).tapped;
        assert_eq!([mowu, bears, theirs].map(tapped), [false, true, true]);
    }
}

/// "Draw a card for each color among other artifacts you control": Karn is
/// all five colors himself and counts for none of them.
#[test]
fn karn_gilded_guardian_draws_for_each_color_among_your_other_artifacts() {
    for (artifacts, drawn) in [
        (&[][..], 0),
        (&[cards::MOX_RUBY, cards::GRIZZLY_BEARS][..], 0),
        (&[cards::VAULT_SKIRGE, cards::PORCELAIN_LEGIONNAIRE][..], 2),
        (&[cards::VAULT_SKIRGE, cards::BALEFUL_STRIX][..], 2),
    ] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            stock_library(&mut game);
            for artifact in artifacts {
                settled(&mut game, *artifact);
            }
            game.put_onto_battlefield(PlayerId::Two, cards::PORCELAIN_LEGIONNAIRE)
                .unwrap();
            game.players[0].hand.clear();

            settled(&mut game, cards::KARN_GILDED_GUARDIAN);

            assert_eq!(game.players[0].hand.len(), drawn, "{artifacts:?}");
        }
    }
}

/// Teyo cast from hand with flash, his trigger aimed at `target`.
fn teyo_entering_at(game: &mut Game, teyo: CardDefinitionId, target: GameObjectId) {
    cast(game, teyo, &[]);
    run_to_decision(game).expect("the trigger chooses its target");
    choose(game, PlayerId::One, target);
    drain_pending(game);
}

/// "Target permanent you control gains hexproof until end of turn. Put a
/// +1/+1 counter on it if it's a creature. Put a loyalty counter on it if
/// it's a planeswalker."
#[test]
fn teyo_lightshield_expert_gives_the_counter_that_fits_what_he_protects() {
    for (protected, plus_one, loyalty) in [
        (cards::GRIZZLY_BEARS, 1, 0),
        (cards::JACE_MEMORY_ADEPT, 0, 1),
        (cards::MOX_RUBY, 0, 0),
    ] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            let target = settled(&mut game, protected);
            let before = permanent(&game, target).counters(CounterKind::Loyalty);

            teyo_entering_at(&mut game, cards::TEYO_LIGHTSHIELD_EXPERT, target);

            assert!(has_keyword(&game, target, KeywordAbility::Hexproof));
            let target = permanent(&game, target);
            assert_eq!(target.counters(CounterKind::PlusOnePlusOne), plus_one);
            assert_eq!(target.counters(CounterKind::Loyalty), before + loyalty);
        }
    }
}

/// "−6: Draw three cards. Then put X +1/+1 counters on each creature you
/// control, where X is the number of cards in your hand." X is read after
/// the draw.
#[test]
fn the_theorist_counts_your_hand_after_its_three_draws() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let jace = settled(&mut game, cards::THE_THEORIST_JACE_BELEREN);
        permanent_mut(&mut game, jace).add_counters(CounterKind::Loyalty, 3);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        hand(&mut game, cards::FOREST);

        activate(&mut game, jace, "−6:");
        drain_pending(&mut game);

        assert_eq!(
            game.players[0].hand.len(),
            4,
            "the one held and three drawn"
        );
        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(counters(bears), 4);
        assert_eq!(counters(theirs), 0);
    }
}

/// "Destroy target nonland permanent. Put a +1/+1 counter on each creature
/// target player controls."
#[test]
fn twisted_fates_puts_its_counters_on_the_target_players_creatures() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let yours = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        let mox = game
            .put_onto_battlefield(PlayerId::Two, cards::MOX_RUBY)
            .unwrap();

        cast(
            &mut game,
            cards::TWISTED_FATES,
            &[Target::Permanent(mox), Target::Player(PlayerId::One)],
        );
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, mox));
        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(counters(yours), 1);
        assert_eq!(counters(theirs), 0);
    }
}

/// "Put a +1/+1 counter on target creature you control. Then it fights target
/// creature an opponent controls": the counter is on it when it fights.
#[test]
fn vigorbloom_charm_fights_with_the_counter_it_just_placed() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let wurm = game
            .put_onto_battlefield(PlayerId::Two, cards::CRAW_WURM)
            .unwrap();

        cast(
            &mut game,
            cards::VIGORBLOOM_CHARM,
            &[Target::Permanent(bears), Target::Permanent(wurm)],
        );
        drain_pending(&mut game);

        assert_eq!(permanent(&game, wurm).damage, 3);
        assert!(!on_battlefield(&game, bears), "six damage to a 3/3");
    }
}
