//! Reality Fracture's green legends.
use super::*;
use crate::card::Subtype;

/// "At the beginning of your end step, if you didn't cast a spell this turn,
/// put two +1/+1 counters on Edgar."
#[test]
fn edgar_moonlit_sovereign_grows_only_at_the_end_of_a_turn_you_cast_nothing() {
    for (cast_a_spell, counters) in [(false, 2), (true, 0)] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            let edgar = settled(&mut game, cards::EDGAR_MOONLIT_SOVEREIGN);
            if cast_a_spell {
                cast(&mut game, cards::MOX_RUBY, &[]);
                drain_pending(&mut game);
            }

            end_step_begins(&mut game);
            drain_pending(&mut game);

            assert_eq!(
                permanent(&game, edgar).counters(CounterKind::PlusOnePlusOne),
                counters
            );
        }
    }
}

/// Edgar cast with flash on your own turn is a spell you cast this turn.
#[test]
fn edgar_moonlit_sovereign_cast_this_turn_does_not_grow_at_its_end() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        cast(&mut game, cards::EDGAR_MOONLIT_SOVEREIGN, &[]);
        drain_pending(&mut game);
        let edgar = controlled(&game, cards::EDGAR_MOONLIT_SOVEREIGN)[0];

        end_step_begins(&mut game);
        drain_pending(&mut game);

        assert_eq!(
            permanent(&game, edgar).counters(CounterKind::PlusOnePlusOne),
            0
        );
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

/// "This spell costs {X} less to cast, where X is the greatest power among
/// creatures you control", and never less than {G}.
#[test]
fn ghalta_the_unstoppable_costs_less_by_the_greatest_power_you_control() {
    for (creature, generic) in [(cards::HILL_GIANT, 5), (cards::CRAW_WURM, 2)] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            let ghalta = hand(&mut game, cards::GHALTA_THE_UNSTOPPABLE);
            settled(&mut game, cards::GRIZZLY_BEARS);
            settled(&mut game, creature);
            game.put_onto_battlefield(PlayerId::Two, cards::SHIVAN_DRAGON)
                .unwrap();

            game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, generic);
            assert!(!can_cast(&game, PlayerId::One, ghalta));
            game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 1);
            assert!(can_cast(&game, PlayerId::One, ghalta));
        }
    }
}

/// "Create Mowu, a legendary 3/3 green Dog creature token."
#[test]
fn jiang_yanggu_never_alone_creates_mowu_a_legendary_green_dog() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::JIANG_YANGGU_NEVER_ALONE);

        let [mowu] = tokens(&game)[..] else {
            panic!("one token");
        };
        assert_eq!(game.effective_permanent_name(mowu).as_deref(), Some("Mowu"));
        assert_eq!((game.power(mowu), game.toughness(mowu)), (Some(3), Some(3)));
        assert_eq!(colors(&game, mowu), [ManaColor::Green]);
        assert!(
            game.effective_subtypes(mowu)
                .contains(Subtype::from_name("Dog").unwrap())
        );
        assert!(
            game.permanent_supertypes(mowu)
                .is_some_and(|supertypes| supertypes.contains(CardSupertype::Legendary))
        );
    }
}

/// "At the beginning of your end step, untap all tokens you control."
#[test]
fn jiang_yanggu_never_alone_untaps_your_tokens_at_your_end_step() {
    static SOLDIER: TokenCharacteristics =
        TokenCharacteristics::creature(&["Soldier"], &[ManaColor::White], 1, 1);
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

/// "{2}: Return target land card from your graveyard to your hand."
#[test]
fn marwyn_the_preserver_returns_a_land_card_from_your_graveyard() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let marwyn = settled(&mut game, cards::MARWYN_THE_PRESERVER);
        let [forest, _] = put_in_graveyard(&mut game, &[cards::FOREST, cards::GRIZZLY_BEARS])[..]
        else {
            unreachable!("two cards were put there");
        };
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 2);

        assert_eq!(
            activation_targets(&game, marwyn, "{2}:"),
            [vec![Target::Card(forest)]]
        );
        activate(&mut game, marwyn, "{2}:");
        drain_pending(&mut game);

        assert_eq!(game.players[0].hand.len(), 1);
        assert_eq!(game.players[0].hand[0].definition, cards::FOREST);
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

#[test]
fn pia_aether_ascetic_searches_for_nothing_when_you_keep_your_cards() {
    for prepared_engine in [false, true] {
        let mut game = pia_aether_ascetic_offering(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Decline");

        assert!(run_to_decision(&mut game).is_none());
        assert_eq!(game.players[0].hand.len(), 1);
        assert_eq!(game.players[0].library.len(), 3);
    }
}

/// "Another target creature you control fights up to one target creature an
/// opponent controls."
#[test]
fn yoshimaru_scrappy_stray_has_another_creature_of_yours_fight_an_opponents() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let giant = settled(&mut game, cards::HILL_GIANT);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::YOSHIMARU_SCRAPPY_STRAY, &[]);
        run_to_decision(&mut game).expect("the trigger chooses your creature");
        assert_eq!(offered(&game, PlayerId::One), [giant], "not Yoshimaru");
        choose(&mut game, PlayerId::One, giant);
        let decision = run_to_decision(&mut game).expect("and then the opponent's");
        assert_eq!((decision.minimum, decision.maximum), (0, 1), "up to one");
        choose(&mut game, PlayerId::One, theirs);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, theirs), "3 damage to a 2/2");
        assert_eq!(permanent(&game, giant).damage, 2);
    }
}

/// "{6}: Put a +1/+1 counter on target nonlegendary creature."
#[test]
fn yoshimaru_scrappy_stray_puts_its_counter_only_on_a_nonlegendary_creature() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        // Its arrival has the Bears fight nothing: no opponent's creature to name.
        let yoshimaru = settled(&mut game, cards::YOSHIMARU_SCRAPPY_STRAY);
        assert_eq!(permanent(&game, bears).damage, 0);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 6);

        assert_eq!(
            activation_targets(&game, yoshimaru, "{6}:"),
            [vec![Target::Permanent(bears)]],
            "Yoshimaru is legendary"
        );
    }
}
