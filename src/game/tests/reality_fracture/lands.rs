//! Reality Fracture's lands.
use super::*;

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

/// "{2}, {T}: Target creature that attacked this turn becomes prepared.
/// Activate only as a sorcery." The dearer ability asks neither.
#[test]
fn hexhaven_dueling_arena_prepares_cheaply_only_a_creature_that_attacked_this_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let arena = settled(&mut game, cards::HEXHAVEN_DUELING_ARENA);
        let attacker = settled(&mut game, cards::GRIZZLY_BEARS);
        let resting = settled(&mut game, cards::GRIZZLY_BEARS);
        permanent_mut(&mut game, attacker).attacked_this_turn = true;
        game.step = Step::PostcombatMain;
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 4);

        assert_eq!(
            activation_targets(&game, arena, "{2}, {T}"),
            [vec![Target::Permanent(attacker)]]
        );
        assert_eq!(
            activation_targets(&game, arena, "{4}, {T}"),
            [
                vec![Target::Permanent(attacker)],
                vec![Target::Permanent(resting)]
            ]
        );

        game.step = Step::End;
        assert!(activation_targets(&game, arena, "{2}, {T}").is_empty());
        assert_eq!(activation_targets(&game, arena, "{4}, {T}").len(), 2);
    }
}
