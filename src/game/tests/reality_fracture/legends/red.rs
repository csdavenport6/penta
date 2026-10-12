//! Reality Fracture's red legends.
use super::*;

/// "Whenever another creature you control enters, Arni gets +X/+0 until end
/// of turn, where X is that creature's power."
#[test]
fn arni_renowned_champion_gets_the_power_of_each_other_creature_that_enters() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let arni = settled(&mut game, cards::ARNI_RENOWNED_CHAMPION);
        game.put_onto_battlefield(PlayerId::Two, cards::CRAW_WURM)
            .unwrap();
        drain_pending(&mut game);
        assert_eq!(game.power(permanent(&game, arni)), Some(1));

        settled(&mut game, cards::HILL_GIANT);
        settled(&mut game, cards::GRIZZLY_BEARS);

        let arni = permanent(&game, arni);
        assert_eq!((game.power(arni), game.toughness(arni)), (Some(6), Some(5)));
    }
}

/// "Each other creature you control with a +1/+1 counter on it has haste."
#[test]
fn gallia_the_merrymaker_gives_haste_to_your_other_creatures_with_a_counter() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::GALLIA_THE_MERRYMAKER);
        let with = settled(&mut game, cards::GRIZZLY_BEARS);
        let without = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        for id in [with, theirs] {
            permanent_mut(&mut game, id).add_counters(CounterKind::PlusOnePlusOne, 1);
        }

        assert!(has_keyword(&game, with, KeywordAbility::Haste));
        assert!(!has_keyword(&game, without, KeywordAbility::Haste));
        assert!(!has_keyword(&game, theirs, KeywordAbility::Haste));
    }
}

/// "Put a +1/+1 counter on target creature that entered this turn": whoever
/// controls it, and Gallia's own counter gives her nothing she lacked.
#[test]
fn gallia_the_merrymaker_targets_only_a_creature_that_entered_this_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let gallia = settled(&mut game, cards::GALLIA_THE_MERRYMAKER);
        settled(&mut game, cards::GRIZZLY_BEARS);
        for permanent in &mut game.battlefield {
            permanent.entered_turn = 0;
        }
        let fresh = game
            .put_onto_battlefield(PlayerId::One, cards::HILL_GIANT)
            .unwrap();
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 2);

        let mut targets = activation_targets(&game, gallia, "{1}{R}, {T}:");
        targets.sort();
        let mut wanted = [
            vec![Target::Permanent(fresh)],
            vec![Target::Permanent(theirs)],
        ];
        wanted.sort();
        assert_eq!(targets, wanted);

        activate_at(
            &mut game,
            gallia,
            "{1}{R}, {T}:",
            &[Target::Permanent(fresh)],
        );
        drain_pending(&mut game);

        assert!(
            has_keyword(&game, fresh, KeywordAbility::Haste),
            "the counter is what Gallia's haste asks for"
        );
        assert!(game.can_attack(permanent(&game, fresh)));
    }
}

/// "Landfall — Whenever a land you control enters, Koth deals 1 damage to
/// each opponent. If that land is a Mountain, add {R}."
#[test]
fn koth_the_geomancer_adds_red_mana_only_for_a_mountain() {
    for (land, red) in [
        (cards::FOREST, 0),
        (cards::MOUNTAIN, 1),
        (cards::BADLANDS, 1),
    ] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            settled(&mut game, cards::KOTH_THE_GEOMANCER);

            settled(&mut game, land);

            assert_eq!(game.players[1].life, 19);
            assert_eq!(game.players[0].life, 20);
            assert_eq!(game.players[0].mana_pool.amount(ManaColor::Red), red);
        }
    }
}

/// "If that land is a Mountain" asks about the land that entered, as it last
/// was if it has gone by the time the trigger resolves.
#[test]
fn koth_the_geomancer_adds_red_mana_for_a_mountain_that_has_already_left() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::KOTH_THE_GEOMANCER);
        let mountain = game
            .put_onto_battlefield(PlayerId::One, cards::MOUNTAIN)
            .unwrap();
        game.destroy_permanent(mountain);

        drain_pending(&mut game);

        assert_eq!(game.players[1].life, 19);
        assert_eq!(game.players[0].mana_pool.amount(ManaColor::Red), 1);
    }
}

/// "{2}, {T}, Sacrifice an artifact or land: Draw a card."
#[test]
fn marwyn_the_clearcutter_sacrifices_an_artifact_or_a_land_to_draw() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let marwyn = settled(&mut game, cards::MARWYN_THE_CLEARCUTTER);
        let mox = settled(&mut game, cards::MOX_RUBY);
        let forest = settled(&mut game, cards::FOREST);
        settled(&mut game, cards::GRIZZLY_BEARS);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 2);

        let mut wanted = [vec![mox], vec![forest]];
        wanted.sort();
        assert_eq!(
            sacrifices_offered(&game, marwyn, "{2}, {T}, Sacrifice"),
            wanted,
            "not a creature, and not Marwyn"
        );

        activate(&mut game, marwyn, "{2}, {T}, Sacrifice");
        drain_pending(&mut game);
        assert_eq!(game.players[0].hand.len(), 1);
    }
}

/// "Target creature gets +X/+0 until end of turn, where X is the number of
/// artifacts you control": the Thopter Pia made is one of them.
#[test]
fn pia_determined_rebuilder_counts_her_thopter_among_your_artifacts() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let pia = settled(&mut game, cards::PIA_DETERMINED_REBUILDER);
        settled(&mut game, cards::MOX_RUBY);
        game.put_onto_battlefield(PlayerId::Two, cards::MOX_RUBY)
            .unwrap();
        let thopters = tokens(&game);
        assert_eq!(thopters.len(), 1);
        assert_eq!(
            (game.power(thopters[0]), game.toughness(thopters[0])),
            (Some(1), Some(1))
        );
        assert!(has_keyword(
            &game,
            thopters[0].card.id,
            KeywordAbility::Flying
        ));
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 6);

        activate_at(&mut game, pia, "{5}{R}:", &[Target::Permanent(pia)]);
        drain_pending(&mut game);

        let pia = permanent(&game, pia);
        assert_eq!((game.power(pia), game.toughness(pia)), (Some(4), Some(2)));
    }
}

/// "Whenever you cast a noncreature spell, creatures you control get +1/+0
/// until end of turn": the creatures there as the trigger resolves, which is
/// before the spell that caused it does.
#[test]
fn winter_team_player_strengthens_the_creatures_you_control_as_the_trigger_resolves() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let winter = settled(&mut game, cards::WINTER_TEAM_PLAYER);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::GRIZZLY_BEARS, &[]);
        drain_pending(&mut game);
        assert_eq!(
            game.power(permanent(&game, winter)),
            Some(3),
            "a creature spell"
        );

        cast(&mut game, cards::MOX_RUBY, &[]);
        drain_pending(&mut game);
        let late = settled(&mut game, cards::HILL_GIANT);

        let stats = |id| {
            let permanent = permanent(&game, id);
            (game.power(permanent), game.toughness(permanent))
        };
        assert_eq!(stats(winter), (Some(4), Some(3)));
        assert_eq!(stats(theirs), (Some(2), Some(2)));
        assert_eq!(stats(late), (Some(3), Some(3)), "it arrived afterwards");
    }
}
