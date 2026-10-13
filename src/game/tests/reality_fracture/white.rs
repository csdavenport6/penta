//! Reality Fracture's white cards.
use super::*;

/// "If it wasn't attacking, its controller draws a card" asks about the
/// creature as it last was on the battlefield, and names whoever controlled
/// it there.
#[test]
fn prophesied_end_draws_for_the_controller_of_a_creature_that_was_not_attacking() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(
            &mut game,
            cards::PROPHESIED_END,
            &[Target::Permanent(bears)],
        );
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, bears));
        assert_eq!(game.players[1].hand.len(), 1, "its controller draws");
        assert!(game.players[0].hand.is_empty(), "and you do not");
    }
}

#[test]
fn prophesied_end_draws_nothing_for_a_creature_that_was_attacking() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        attack_with(&mut game, bears);

        cast(
            &mut game,
            cards::PROPHESIED_END,
            &[Target::Permanent(bears)],
        );
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, bears));
        assert!(game.players[0].hand.is_empty(), "it was attacking");
    }
}

/// The draw does not depend on the destruction: an indestructible creature
/// that wasn't attacking still has its controller draw.
#[test]
fn prophesied_end_draws_even_when_the_creature_is_not_destroyed() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let tajic = game
            .put_onto_battlefield(PlayerId::Two, cards::TAJIC_BLADE_OF_THE_LEGION)
            .unwrap();

        cast(
            &mut game,
            cards::PROPHESIED_END,
            &[Target::Permanent(tajic)],
        );
        drain_pending(&mut game);

        assert!(on_battlefield(&game, tajic), "indestructible");
        assert_eq!(game.players[1].hand.len(), 1);
    }
}

/// The Hound is never its own "other target creature you control", and what
/// comes back is a new object.
#[test]
fn flickering_hound_blinks_another_creature_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let hound = settled(&mut game, cards::FLICKERING_HOUND);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::SAVANNAH_LIONS, &[]);
        let decision = run_to_decision(&mut game).expect("the trigger chooses its target");
        assert_eq!((decision.minimum, decision.maximum), (0, 1), "up to one");
        assert_eq!(offered(&game, PlayerId::One), [bears]);
        choose(&mut game, PlayerId::One, bears);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, bears), "the exiled object is gone");
        assert_eq!(controlled(&game, cards::GRIZZLY_BEARS).len(), 1);
        assert!(on_battlefield(&game, hound));
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

/// "Graft Surgeon's second ability puts all counters that were on Graft
/// Surgeon onto the target creature, not just its +1/+1 counters."
#[test]
fn graft_surgeon_puts_every_kind_of_counter_it_died_with_on_its_target() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let surgeon = settled(&mut game, cards::GRAFT_SURGEON);
        assert_eq!(
            permanent(&game, surgeon).counters(CounterKind::PlusOnePlusOne),
            1,
            "it enters with one"
        );
        permanent_mut(&mut game, surgeon).add_counters(CounterKind::Stun, 2);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(
            &mut game,
            cards::LIGHTNING_BOLT,
            &[Target::Permanent(surgeon)],
        );
        run_to_decision(&mut game).expect("the trigger chooses its target");
        assert_eq!(offered(&game, PlayerId::One), [bears], "one you control");
        choose(&mut game, PlayerId::One, bears);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, surgeon));
        let bears = permanent(&game, bears);
        assert_eq!(bears.counters(CounterKind::PlusOnePlusOne), 1);
        assert_eq!(bears.counters(CounterKind::Stun), 2);
        assert_eq!(
            permanent(&game, theirs).counters(CounterKind::PlusOnePlusOne),
            0
        );
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

#[test]
fn guiding_hydra_that_keeps_its_counter_gives_none() {
    for prepared_engine in [false, true] {
        let (mut game, [hydra, bears, _]) = guiding_hydra_at_combat(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Decline");
        drain_pending(&mut game);

        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(counters(hydra), 2);
        assert_eq!(counters(bears), 0);
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

/// "Destroy target creature with toughness 4 or greater."
#[test]
fn surgical_precision_targets_only_toughness_four_or_greater() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::HILL_GIANT)
            .unwrap();
        let spell = hand(&mut game, cards::SURGICAL_PRECISION);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 2);

        let mut targets = castable_targets(&game, spell);
        targets.sort_by_key(Vec::len);
        assert_eq!(
            targets,
            [vec![], vec![Target::Permanent(angel)]],
            "the drawing mode, and the 4/4 but not the 3/3"
        );
    }
}
