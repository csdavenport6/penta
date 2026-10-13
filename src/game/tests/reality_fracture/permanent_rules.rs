//! What a permanent is and does while it stays on the battlefield: static
//! abilities and the conditions on them, the characteristics of tokens and
//! copies, entering tapped, and the legend rule.
use super::*;
use crate::card::{PermanentDesignationDef, Subtype};

/// The colors `permanent` is.
fn colors(game: &Game, permanent: &Permanent) -> Vec<ManaColor> {
    ManaColor::COLORS
        .into_iter()
        .zip(game.permanent_colors(permanent))
        .filter_map(|(color, has)| has.then_some(color))
        .collect()
}

/// "Create a 2/2 white Cat Soldier creature token named Ajani's Pridemate
/// with 'Whenever you gain life, put a +1/+1 counter on this token.'"
#[test]
fn ajani_resolute_creates_a_pridemate_that_grows_whenever_you_gain_life() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ajani = settled(&mut game, cards::AJANI_RESOLUTE);
        permanent_mut(&mut game, ajani).add_counters(CounterKind::Loyalty, 2);

        activate(&mut game, ajani, "−4:");
        drain_pending(&mut game);
        let pridemate = tokens(&game)[0].card.id;
        assert_eq!(
            game.effective_permanent_name(permanent(&game, pridemate))
                .as_deref(),
            Some("Ajani's Pridemate")
        );
        assert_eq!(game.power(permanent(&game, pridemate)), Some(2));

        game.gain_life(PlayerId::One, 3);
        drain_pending(&mut game);

        let pridemate = permanent(&game, pridemate);
        assert_eq!(pridemate.counters(CounterKind::PlusOnePlusOne), 1);
        assert_eq!(
            (game.power(pridemate), game.toughness(pridemate)),
            (Some(3), Some(3))
        );
    }
}

/// "You get an emblem with 'Creatures you control get +2/+2.'" The emblem
/// outlasts Ajani and reaches creatures that arrive after it.
#[test]
fn ajani_resolute_emblem_gives_your_creatures_plus_two_plus_two() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ajani = settled(&mut game, cards::AJANI_RESOLUTE);
        permanent_mut(&mut game, ajani).add_counters(CounterKind::Loyalty, 8);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        activate(&mut game, ajani, "−10:");
        drain_pending(&mut game);
        assert!(!on_battlefield(&game, ajani), "no loyalty left");
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);

        let stats = |id| {
            let permanent = permanent(&game, id);
            (game.power(permanent), game.toughness(permanent))
        };
        assert_eq!(stats(bears), (Some(4), Some(4)));
        assert_eq!(stats(theirs), (Some(2), Some(2)));
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

/// "Creatures you control can attack as though they didn't have defender."
#[test]
fn ghalta_the_immovable_lets_your_defenders_attack_and_not_the_opponents() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let wall = settled(&mut game, cards::WALL_OF_WOOD);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::WALL_OF_WOOD)
            .unwrap();
        permanent_mut(&mut game, theirs).entered_controller_turn = 0;
        assert!(!game.can_attack(permanent(&game, wall)));

        settled(&mut game, cards::GHALTA_THE_IMMOVABLE);

        assert!(game.can_attack(permanent(&game, wall)));
        assert!(!game.can_attack(permanent(&game, theirs)));
    }
}

/// "Each creature you control with toughness greater than its power assigns
/// combat damage equal to its toughness rather than its power." A creature
/// whose power is at least its toughness keeps assigning its power.
#[test]
fn ghalta_the_immovable_has_only_creatures_with_more_toughness_than_power_assign_toughness() {
    for (attacker, damage) in [(cards::WALL_OF_WOOD, 3), (cards::SAVANNAH_LIONS, 2)] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            settled(&mut game, cards::GHALTA_THE_IMMOVABLE);
            let attacker = settled(&mut game, attacker);

            connect_with(&mut game, attacker);

            assert_eq!(game.players[1].life, 20 - damage);
        }
    }
}

/// Hall of Echoes having become a copy of your Yargle, a legendary 9/3.
/// Returns the Hall and that Yargle.
fn hall_of_echoes_copying_yargle(game: &mut Game) -> (GameObjectId, GameObjectId) {
    let hall = settled(game, cards::HALL_OF_ECHOES);
    let yargle = settled(game, cards::YARGLE_GLUTTON_OF_URBORG);
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 5);
    activate_at(game, hall, "{5}", &[Target::Permanent(yargle)]);
    drain_pending(game);
    (hall, yargle)
}

/// "This land becomes a copy of target creature you control until end of
/// turn. The 'legend rule' doesn't apply to permanents you control this
/// turn": the Hall and what it copied both stay, and so does a third that
/// arrives later in the turn. The opponent's legends get no such leave.
#[test]
fn hall_of_echoes_copies_a_legend_and_suspends_the_legend_rule_for_you_alone() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);

        let (hall, yargle) = hall_of_echoes_copying_yargle(&mut game);
        let third = settled(&mut game, cards::YARGLE_GLUTTON_OF_URBORG);
        for _ in 0..2 {
            game.put_onto_battlefield(PlayerId::Two, cards::YARGLE_GLUTTON_OF_URBORG)
                .unwrap();
        }
        game.check_state_based_actions();
        drain_pending(&mut game);

        assert_eq!(game.power(permanent(&game, hall)), Some(9));
        assert_eq!(
            game.effective_permanent_name(permanent(&game, hall))
                .as_deref(),
            Some("Yargle, Glutton of Urborg")
        );
        for yours in [hall, yargle, third] {
            assert!(on_battlefield(&game, yours));
        }
        let theirs = game
            .battlefield
            .iter()
            .filter(|permanent| permanent.controller == PlayerId::Two)
            .count();
        assert_eq!(theirs, 1, "the opponent keeps one Yargle");
    }
}

/// "Both effects of Hall of Echoes's last ability have the same duration ...
/// there's no moment at which Hall of Echoes is still a copy of the target
/// creature after the 'legend rule' begins to apply again."
#[test]
fn hall_of_echoes_is_a_land_again_by_the_time_the_legend_rule_returns() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let (hall, yargle) = hall_of_echoes_copying_yargle(&mut game);

        game.finish_cleanup();
        game.check_state_based_actions();
        drain_pending(&mut game);

        assert!(on_battlefield(&game, yargle));
        assert_eq!(game.power(permanent(&game, hall)), None, "a land again");
    }
}

/// "If you control more than one legendary permanent with the same name and
/// the 'legend rule' begins applying again, you'll immediately have to comply
/// with the rule."
#[test]
fn hall_of_echoes_suspends_the_legend_rule_only_until_end_of_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let (_, yargle) = hall_of_echoes_copying_yargle(&mut game);
        let second = settled(&mut game, cards::YARGLE_GLUTTON_OF_URBORG);
        assert!(on_battlefield(&game, yargle) && on_battlefield(&game, second));

        game.finish_cleanup();
        game.check_state_based_actions();
        drain_pending(&mut game);

        assert_ne!(
            on_battlefield(&game, yargle),
            on_battlefield(&game, second),
            "one of the two is kept"
        );
    }
}

/// The colors of mana player one could tap `source` for.
fn mana_colors(game: &Game, source: GameObjectId) -> Vec<ManaColor> {
    game.legal_actions(PlayerId::One)
        .into_iter()
        .filter_map(|action| match action {
            Action::ActivateManaAbility {
                source: tapped,
                color,
                ..
            } if tapped == source => Some(color),
            _ => None,
        })
        .collect()
}

/// "Create a Heartwood token. (It's a red and green artifact with '{T}: Add
/// {R} or {G}.')"
#[test]
fn hungering_puppetbeast_makes_a_heartwood_that_taps_for_red_or_green() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);

        cast(&mut game, cards::HUNGERING_PUPPETBEAST, &[]);
        drain_pending(&mut game);

        let made = tokens(&game);
        let [heartwood] = made[..] else {
            panic!("one token is made");
        };
        assert_eq!(
            game.permanent_types(heartwood),
            Some(CardTypeSet::single(CardType::Artifact))
        );
        assert!(
            game.effective_subtypes(heartwood)
                .contains(Subtype::from_name("Heartwood").unwrap())
        );
        assert_eq!(colors(&game, heartwood), [ManaColor::Red, ManaColor::Green]);
        assert_eq!(
            mana_colors(&game, heartwood.card.id),
            [ManaColor::Red, ManaColor::Green]
        );
    }
}

/// "When this Aura enters, tap enchanted creature. It becomes unprepared."
/// The copy of its prepare spell goes with the designation, and the static
/// ability then takes what the creature had.
#[test]
fn infinite_coursework_taps_and_unprepares_the_creature_it_enchants() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let emeritus = settled(&mut game, cards::EMERITUS_OF_IDEATION);
        let prepared = |game: &Game| {
            permanent(game, emeritus)
                .designations
                .contains(&PermanentDesignationDef::Prepared)
        };
        assert!(prepared(&game), "it enters prepared");
        assert_eq!(game.players[0].exile.len(), 1, "its Ancestral Recall");

        cast(
            &mut game,
            cards::INFINITE_COURSEWORK,
            &[Target::Permanent(emeritus)],
        );
        drain_pending(&mut game);

        assert!(permanent(&game, emeritus).tapped);
        assert!(!prepared(&game));
        assert!(game.players[0].exile.is_empty(), "the copy ceased to exist");
        assert!(
            !has_keyword(&game, emeritus, KeywordAbility::Flying),
            "it loses all abilities"
        );
    }
}

/// "This creature has vigilance as long as you control a Jace planeswalker":
/// yours, a planeswalker, and a Jace.
#[test]
fn mind_meanderer_has_vigilance_only_while_you_control_a_jace_planeswalker() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let meanderer = settled(&mut game, cards::MIND_MEANDERER);
        game.put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
            .unwrap();
        settled(&mut game, cards::CHANDRA_TORCH_OF_DEFIANCE);
        assert!(!has_keyword(&game, meanderer, KeywordAbility::Vigilance));

        settled(&mut game, cards::JACE_MEMORY_ADEPT);
        assert!(has_keyword(&game, meanderer, KeywordAbility::Vigilance));
    }
}

/// Player one plays a land with `definition` from hand, and returns the
/// permanent it became.
fn play_land(game: &mut Game, definition: CardDefinitionId) -> GameObjectId {
    let land = hand(game, definition);
    game.players[0].lands_played_this_turn = 0;
    let play = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| matches!(action, Action::PlayLand { card, .. } if *card == land))
        .expect("a land drop is available");
    game.apply(PlayerId::One, play).unwrap();
    drain_pending(game);
    controlled(game, definition)[0]
}

/// "This land enters tapped unless you control a planeswalker."
#[test]
fn a_planeswalker_land_enters_untapped_only_while_you_control_a_planeswalker() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
            .unwrap();

        let commons = play_land(&mut game, cards::DEDICATED_COMMONS);
        assert!(
            permanent(&game, commons).tapped,
            "the opponent's planeswalker is not yours"
        );

        settled(&mut game, cards::JACE_MEMORY_ADEPT);
        let annex = play_land(&mut game, cards::FATEHOLD_ANNEX);
        assert!(!permanent(&game, annex).tapped);
    }
}

/// "If this land enters at the same time as a planeswalker, that planeswalker
/// isn't considered when determining if this land enters the battlefield
/// tapped or untapped."
#[test]
fn a_planeswalker_land_ignores_a_planeswalker_entering_alongside_it() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.players[0].library = game
            .build_zone(
                PlayerId::One,
                &[cards::JACE_MEMORY_ADEPT, cards::DEDICATED_COMMONS],
            )
            .unwrap();

        cast_for_x(&mut game, cards::GENESIS_WAVE, 5);
        let revealed = run_to_decision(&mut game).expect("the revealed permanents are offered");
        let both = revealed
            .options
            .iter()
            .filter(|option| option.card.is_some())
            .map(|option| option.id)
            .collect::<Vec<_>>();
        assert_eq!(both.len(), 2);
        game.apply(
            PlayerId::One,
            Action::ChooseDecision {
                decision: revealed.id,
                options: both,
            },
        )
        .unwrap();
        drain_pending(&mut game);

        assert_eq!(controlled(&game, cards::JACE_MEMORY_ADEPT).len(), 1);
        let commons = controlled(&game, cards::DEDICATED_COMMONS)[0];
        assert!(permanent(&game, commons).tapped);
    }
}

/// "Enchanted permanent is a Construct creature with base power and toughness
/// 5/5 in addition to its other types."
#[test]
fn puppet_crafting_makes_its_permanent_a_five_five_construct_that_keeps_its_types() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let mox = settled(&mut game, cards::MOX_RUBY);

        cast(&mut game, cards::PUPPET_CRAFTING, &[Target::Permanent(mox)]);
        drain_pending(&mut game);

        let mox = permanent(&game, mox);
        let types = game.permanent_types(mox).expect("it has types");
        assert!(types.contains(CardType::Creature) && types.contains(CardType::Artifact));
        assert!(
            game.effective_subtypes(mox)
                .contains(Subtype::from_name("Construct").unwrap())
        );
        assert_eq!((game.power(mox), game.toughness(mox)), (Some(5), Some(5)));
        assert_eq!(mana_colors(&game, mox.card.id), [ManaColor::Red]);
    }
}

/// `active`'s upkeep begins and whatever it triggers resolves.
fn take_an_upkeep(game: &mut Game, active: PlayerId) {
    game.active_player = active;
    game.priority = active;
    game.step = Step::Upkeep;
    game.handle_upkeep_triggers();
    drain_pending(game);
}

/// "(It has '{T}: Add {G}.' It's affected by summoning sickness until your
/// next turn.)"
#[test]
fn verdant_krakens_forest_tentacle_taps_for_green_once_your_next_turn_has_begun() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::VERDANT_KRAKEN);
        take_an_upkeep(&mut game, PlayerId::Two);
        let tentacle = tokens(&game)[0].card.id;
        game.active_player = PlayerId::One;
        game.priority = PlayerId::One;
        game.step = Step::PrecombatMain;

        assert!(
            mana_colors(&game, tentacle).is_empty(),
            "a creature that just arrived"
        );

        game.turns_started[0] += 1;
        assert_eq!(mana_colors(&game, tentacle), [ManaColor::Green]);
    }
}

/// Player one activates `source`'s mana ability for one mana of `color`.
fn tap_for(game: &mut Game, source: GameObjectId, color: ManaColor) {
    let tap = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| {
            matches!(action, Action::ActivateManaAbility { source: tapped, color: added, .. }
                if *tapped == source && *added == color)
        })
        .expect("the permanent taps for that color");
    game.apply(PlayerId::One, tap).unwrap();
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
        tap_for(&mut game, id, ManaColor::Green);
        assert!(!on_battlefield(&game, id), "sacrificed for the mana");
        assert_eq!(
            game.players[0].mana_pool.amount(ManaColor::Green),
            green + 1
        );
    }
}

/// "Winter gets +1/+0 for each creature and planeswalker card in your
/// graveyard": cards of yours, of either type, and nothing else.
#[test]
fn winter_tormented_loner_counts_creature_and_planeswalker_cards_in_your_graveyard() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let winter = settled(&mut game, cards::WINTER_TORMENTED_LONER);
        put_in_graveyard(
            &mut game,
            &[
                cards::GRIZZLY_BEARS,
                cards::JACE_MEMORY_ADEPT,
                cards::FOREST,
                cards::LIGHTNING_BOLT,
            ],
        );
        let theirs = game
            .build_zone(PlayerId::Two, &[cards::GRIZZLY_BEARS])
            .unwrap();
        game.players[1].graveyard.extend(theirs);

        let winter = permanent(&game, winter);
        assert_eq!(
            (game.power(winter), game.toughness(winter)),
            (Some(2), Some(3))
        );
    }
}
