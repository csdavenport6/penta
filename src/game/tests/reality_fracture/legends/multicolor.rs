//! Reality Fracture's multicolor legends.
use super::*;
use crate::card::Subtype;

/// "Whenever another creature or planeswalker you control dies, you gain 1
/// life", which the one sacrificed to his own ability does.
#[test]
fn edgar_ancient_bloodlord_gains_life_for_the_creature_his_ability_sacrifices() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let edgar = settled(&mut game, cards::EDGAR_ANCIENT_BLOODLORD);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let jace = settled(&mut game, cards::JACE_MEMORY_ADEPT);
        settled(&mut game, cards::MOX_RUBY);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Black, 2);

        let mut payments = activations(&game, edgar, "{2}, Sacrifice another")
            .into_iter()
            .map(|action| match action {
                Action::ActivateAbility { cost_objects, .. } => cost_objects,
                _ => unreachable!("only activations are collected"),
            })
            .collect::<Vec<_>>();
        payments.sort();
        let mut wanted = [vec![bears], vec![jace]];
        wanted.sort();
        assert_eq!(payments, wanted, "not Edgar himself, and not an artifact");

        activate(&mut game, edgar, "{2}, Sacrifice another");
        drain_pending(&mut game);

        assert_eq!(game.players[0].life, 21);
        assert!(has_keyword(&game, edgar, KeywordAbility::Menace));
        assert_eq!(
            permanent(&game, edgar).counters(CounterKind::PlusOnePlusOne),
            1
        );
    }
}

/// An opponent's creature dying, or Edgar himself, gains nothing.
#[test]
fn edgar_ancient_bloodlord_gains_nothing_for_an_opponents_creature() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::EDGAR_ANCIENT_BLOODLORD);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        game.destroy_permanent(theirs);
        drain_pending(&mut game);

        assert_eq!(game.players[0].life, 20);
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

/// "{2/W}{2/U}{2/B}{2/R}{2/G}": each symbol is two generic mana or one mana
/// of its color.
#[test]
fn karn_gilded_guardian_is_cast_for_one_mana_of_each_color() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let karn = hand(&mut game, cards::KARN_GILDED_GUARDIAN);
        for color in [
            ManaColor::White,
            ManaColor::Blue,
            ManaColor::Black,
            ManaColor::Red,
        ] {
            game.add_unrestricted_mana(PlayerId::One, color, 1);
        }
        assert!(!can_cast(&game, PlayerId::One, karn));

        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 1);
        assert!(can_cast(&game, PlayerId::One, karn));
    }
}

/// "Whenever Mabel or another creature you control enters, put a +1/+1
/// counter on target creature that entered this turn."
#[test]
fn mabel_valley_hero_puts_a_counter_on_a_creature_that_entered_this_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let old = settled(&mut game, cards::GRIZZLY_BEARS);
        permanent_mut(&mut game, old).entered_turn = 0;

        cast(&mut game, cards::MABEL_VALLEY_HERO, &[]);
        run_to_decision(&mut game).expect("Mabel's own arrival triggers");
        let mabel = controlled(&game, cards::MABEL_VALLEY_HERO)[0];
        assert_eq!(offered(&game, PlayerId::One), [mabel], "not the old Bears");
        choose(&mut game, PlayerId::One, mabel);
        drain_pending(&mut game);

        cast(&mut game, cards::HILL_GIANT, &[]);
        run_to_decision(&mut game).expect("another creature of yours triggers");
        let giant = controlled(&game, cards::HILL_GIANT)[0];
        assert_eq!(offered(&game, PlayerId::One), [mabel, giant]);
        choose(&mut game, PlayerId::One, mabel);
        drain_pending(&mut game);

        assert_eq!(
            permanent(&game, mabel).counters(CounterKind::PlusOnePlusOne),
            2
        );
    }
}

/// An opponent's creature entering does not trigger Mabel, though it is a
/// creature that entered this turn for a trigger that does happen.
#[test]
fn mabel_valley_hero_ignores_an_opponents_creature_entering() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::MABEL_VALLEY_HERO);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        assert!(run_to_decision(&mut game).is_none());
    }
}

/// "Thopters you control have haste", and each noncreature spell makes one.
#[test]
fn saheeli_jewel_of_avishkar_makes_a_thopter_with_haste_for_a_noncreature_spell() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::SAHEELI_JEWEL_OF_AVISHKAR);

        cast(&mut game, cards::GRIZZLY_BEARS, &[]);
        drain_pending(&mut game);
        assert!(tokens(&game).is_empty(), "a creature spell");

        cast(&mut game, cards::MOX_RUBY, &[]);
        drain_pending(&mut game);

        let [thopter] = tokens(&game)[..] else {
            panic!("one Thopter");
        };
        assert!(has_keyword(&game, thopter.card.id, KeywordAbility::Flying));
        assert!(has_keyword(&game, thopter.card.id, KeywordAbility::Haste));
        assert!(game.can_attack(thopter), "the turn it was made");
    }
}

/// "Create a 1/1 colorless Sculpture Treasure artifact creature token with
/// '{T}, Sacrifice this token: Add one mana of any color.'" It is an artifact
/// creature, so Vraska gives it vigilance.
#[test]
fn vraska_soul_of_stone_makes_a_sculpture_treasure_that_is_a_vigilant_creature() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::VRASKA_SOUL_OF_STONE);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);

        cast(&mut game, cards::MOX_RUBY, &[]);
        drain_pending(&mut game);

        let [sculpture] = tokens(&game)[..] else {
            panic!("one token");
        };
        let id = sculpture.card.id;
        assert_eq!(
            (game.power(sculpture), game.toughness(sculpture)),
            (Some(1), Some(1))
        );
        assert!(colors(&game, sculpture).is_empty());
        let subtypes = game.effective_subtypes(sculpture);
        assert!(subtypes.contains(Subtype::from_name("Sculpture").unwrap()));
        assert!(subtypes.contains(Subtype::from_name("Treasure").unwrap()));
        assert!(has_keyword(&game, id, KeywordAbility::Vigilance));
        assert!(!has_keyword(&game, bears, KeywordAbility::Vigilance));

        permanent_mut(&mut game, id).entered_controller_turn = 0;
        let green = game.players[0].mana_pool.amount(ManaColor::Green);
        let tap = game
            .legal_actions(PlayerId::One)
            .into_iter()
            .find(|action| {
                matches!(action, Action::ActivateManaAbility { source, color, .. }
                    if *source == id && *color == ManaColor::Green)
            })
            .expect("it taps for any color");
        game.apply(PlayerId::One, tap).unwrap();
        assert!(!on_battlefield(&game, id), "sacrificed for the mana");
        assert_eq!(
            game.players[0].mana_pool.amount(ManaColor::Green),
            green + 1
        );
    }
}

/// Vraska cast with `lands` lands of player one's and a Serra Angel of the
/// opponent's on the battlefield, her trigger aimed at the Angel if it is
/// there to aim.
fn vraska_the_cutting_glare_entering(prepared_engine: bool, lands: usize) -> (Game, GameObjectId) {
    let mut game = setup(prepared_engine);
    for _ in 0..lands {
        settled(&mut game, cards::FOREST);
    }
    let angel = game
        .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
        .unwrap();
    cast(&mut game, cards::VRASKA_THE_CUTTING_GLARE, &[]);
    if run_to_decision(&mut game).is_some() {
        choose(&mut game, PlayerId::One, angel);
    }
    drain_pending(&mut game);
    (game, angel)
}

/// "When Vraska enters, if you control six or more lands, destroy target
/// permanent an opponent controls. They create a Treasure token."
#[test]
fn vraska_the_cutting_glare_destroys_a_permanent_and_its_controller_gets_a_treasure() {
    for prepared_engine in [false, true] {
        let (game, angel) = vraska_the_cutting_glare_entering(prepared_engine, 6);

        assert!(!on_battlefield(&game, angel));
        assert!(tokens(&game).is_empty(), "not for you");
        let treasures = game
            .battlefield
            .iter()
            .filter(|permanent| {
                permanent.controller == PlayerId::Two
                    && permanent.card.definition.is_token()
                    && game
                        .effective_subtypes(permanent)
                        .contains(Subtype::from_name("Treasure").unwrap())
            })
            .count();
        assert_eq!(treasures, 1);
    }
}

/// "If you don't, the ability won't trigger at all."
#[test]
fn vraska_the_cutting_glare_does_nothing_with_five_lands() {
    for prepared_engine in [false, true] {
        let (game, angel) = vraska_the_cutting_glare_entering(prepared_engine, 5);

        assert!(on_battlefield(&game, angel));
        assert!(
            !game
                .battlefield
                .iter()
                .any(|permanent| permanent.card.definition.is_token())
        );
    }
}
