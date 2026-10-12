//! Reality Fracture's black cards.
use super::*;

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

/// "This creature gets +2/+0 for every seven cards in your graveyard": whole
/// sevens, and only your own graveyard.
#[test]
fn dark_matter_manipulator_grows_for_each_full_seven_cards_in_your_graveyard() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        // Nothing to mill, so its own enters trigger leaves the count alone.
        game.players[0].library.clear();
        let manipulator = settled(&mut game, cards::DARK_MATTER_MANIPULATOR);
        game.players[1].graveyard = game.build_zone(PlayerId::Two, &[cards::FOREST; 7]).unwrap();

        for (cards_in_graveyard, power) in [(6, 1), (7, 3), (13, 3), (14, 5)] {
            game.players[0].graveyard.clear();
            put_in_graveyard(&mut game, &vec![cards::FOREST; cards_in_graveyard]);
            let manipulator = permanent(&game, manipulator);
            assert_eq!(
                (game.power(manipulator), game.toughness(manipulator)),
                (Some(power), Some(2)),
                "{cards_in_graveyard} cards"
            );
        }
    }
}

/// Lich's Relic cast with mana to spare, stopped at its offer to pay {2}.
/// Returns your creature, then the opponent's creature and planeswalker.
fn lichs_relic_offering(prepared_engine: bool) -> (Game, [GameObjectId; 3]) {
    let mut game = setup(prepared_engine);
    let yours = settled(&mut game, cards::GRIZZLY_BEARS);
    let bears = game
        .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
        .unwrap();
    let jace = game
        .put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
        .unwrap();
    game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
        .unwrap();
    cast(&mut game, cards::LICH_S_RELIC, &[]);
    run_to_decision(&mut game).expect("paying {2} is offered");
    (game, [yours, bears, jace])
}

/// "You may pay {2}. When you do, for each opponent, destroy up to one target
/// creature or planeswalker that player controls."
#[test]
fn lichs_relic_paid_for_destroys_a_creature_or_planeswalker_an_opponent_controls() {
    for prepared_engine in [false, true] {
        let (mut game, [yours, bears, jace]) = lichs_relic_offering(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Pay the cost");
        let decision = run_to_decision(&mut game).expect("the reflexive trigger chooses");
        assert_eq!((decision.minimum, decision.maximum), (0, 1), "up to one");
        assert_eq!(offered(&game, PlayerId::One), [bears, jace]);
        choose(&mut game, PlayerId::One, jace);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, jace));
        assert!(on_battlefield(&game, bears));
        assert!(on_battlefield(&game, yours));
    }
}

#[test]
fn lichs_relic_left_unpaid_destroys_nothing() {
    for prepared_engine in [false, true] {
        let (mut game, permanents) = lichs_relic_offering(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Decline");

        assert!(run_to_decision(&mut game).is_none(), "no trigger to aim");
        for permanent in permanents {
            assert!(on_battlefield(&game, permanent));
        }
    }
}

/// The additional cost of Silence the Echo as its caster means to pay it.
#[derive(Clone, Copy, Debug)]
enum EchoCost {
    Creature,
    Planeswalker,
    ThreeMana,
}

/// "As an additional cost to cast this spell, sacrifice a creature or
/// planeswalker or pay {3}."
#[test]
fn silence_the_echo_costs_a_creature_a_planeswalker_or_three_more_mana() {
    for prepared_engine in [false, true] {
        for cost in [
            EchoCost::Creature,
            EchoCost::Planeswalker,
            EchoCost::ThreeMana,
        ] {
            let mut game = setup(prepared_engine);
            let target = game
                .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
                .unwrap();
            settled(&mut game, cards::FOREST);
            let spell = hand(&mut game, cards::SILENCE_THE_ECHO);
            game.add_unrestricted_mana(PlayerId::One, ManaColor::Black, 2);
            assert!(
                casts(&game, spell).is_empty(),
                "a land is nothing to sacrifice and there is no {{3}}"
            );

            let sacrificed = match cost {
                EchoCost::Creature => Some(settled(&mut game, cards::SAVANNAH_LIONS)),
                EchoCost::Planeswalker => Some(settled(&mut game, cards::JACE_MEMORY_ADEPT)),
                EchoCost::ThreeMana => {
                    game.add_unrestricted_mana(PlayerId::One, ManaColor::Black, 3);
                    None
                }
            };
            let ways = casts(&game, spell)
                .into_iter()
                .filter(|action| {
                    matches!(action, Action::CastSpell { choices, .. }
                        if choices.iter_targets().eq(&[Target::Permanent(target)]))
                })
                .collect::<Vec<_>>();
            assert_eq!(ways.len(), 1, "exactly the one way to pay: {cost:?}");
            game.apply(PlayerId::One, ways[0].clone()).unwrap();
            drain_pending(&mut game);

            assert_eq!(game.players[0].mana_pool.total(), 0, "{cost:?}");
            if let Some(sacrificed) = sacrificed {
                assert!(!on_battlefield(&game, sacrificed), "{cost:?}");
            }
            assert!(!on_battlefield(&game, target), "destroyed: {cost:?}");
        }
    }
}

/// "Return another target creature card from your graveyard to your hand":
/// the Necromancer is still in the graveyard while targets are chosen, and
/// is not one of them.
#[test]
fn theoretical_necromancer_returns_a_creature_card_other_than_itself() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let cards = put_in_graveyard(
            &mut game,
            &[
                cards::THEORETICAL_NECROMANCER,
                cards::GRIZZLY_BEARS,
                cards::FOREST,
            ],
        );
        let (necromancer, bears) = (cards[0], cards[1]);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Black, 4);

        let mut activations = game
            .legal_actions(PlayerId::One)
            .into_iter()
            .filter(|action| {
                matches!(action, Action::ActivateAbility { source, .. } if *source == necromancer)
            })
            .collect::<Vec<_>>();
        assert_eq!(activations.len(), 1, "one creature card to aim at");
        let action = activations.remove(0);
        let Action::ActivateAbility { targets, .. } = &action else {
            unreachable!("only activations are collected");
        };
        assert!(
            targets
                .iter()
                .flat_map(TargetSelection::targets)
                .eq(&[Target::Card(bears)])
        );
        // Naming the Necromancer anyway is refused where the declaration is checked.
        let mut at_itself = action.clone();
        if let Action::ActivateAbility { targets, .. } = &mut at_itself {
            *targets = vec![TargetSelection::single(
                targets[0].slot(),
                Target::Card(necromancer),
            )];
        }
        assert!(game.clone().apply(PlayerId::One, at_itself).is_err());

        game.apply(PlayerId::One, action).unwrap();
        drain_pending(&mut game);

        assert!(
            game.players[0]
                .hand
                .iter()
                .any(|card| card.definition == cards::GRIZZLY_BEARS)
        );
        assert_eq!(game.players[0].exile.len(), 1, "itself, as the cost");
    }
}
