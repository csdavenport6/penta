//! What a spell or ability may target, and the limits on when an ability may
//! be activated at all.
use super::*;

/// The target lists of player one's legal activations of `source`'s ability
/// whose text starts with `prefix`, one per activation.
fn activation_targets(game: &Game, source: GameObjectId, prefix: &str) -> Vec<Vec<Target>> {
    activations(game, source, prefix)
        .iter()
        .map(announced_targets)
        .collect()
}

/// "Destroy target noncreature, nonland permanent": both words.
#[test]
fn archive_arbiter_destroys_only_a_noncreature_nonland_permanent() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
            .unwrap();
        let mox = game
            .put_onto_battlefield(PlayerId::Two, cards::MOX_RUBY)
            .unwrap();

        cast(&mut game, cards::ARCHIVE_ARBITER, &[]);
        run_to_decision(&mut game).expect("the trigger chooses its mode");
        choose_decision_by_label(
            &mut game,
            PlayerId::One,
            "Destroy target noncreature, nonland permanent.",
        );
        assert_eq!(
            offered(&game, PlayerId::One),
            [mox],
            "not a creature, a land, or the Arbiter itself"
        );
        choose(&mut game, PlayerId::One, mox);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, mox));
    }
}

/// "Another target creature you control with power 2 or less."
#[test]
fn eardrum_rattler_targets_only_your_other_creatures_with_power_two_or_less() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let rattler = settled(&mut game, cards::EARDRUM_RATTLER);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        settled(&mut game, cards::SERRA_ANGEL);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);

        assert_eq!(
            activation_targets(&game, rattler, "{1}, {T}"),
            [vec![Target::Permanent(bears)]],
            "not itself, not a 4/4, not the opponent's"
        );
    }
}

/// "Target black or green creature or planeswalker."
#[test]
fn essence_burn_targets_only_black_or_green_creatures_and_planeswalkers() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let knight = game
            .put_onto_battlefield(PlayerId::Two, cards::BLACK_KNIGHT)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
            .unwrap();
        let burn = hand(&mut game, cards::ESSENCE_BURN);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 2);

        assert_eq!(
            castable_targets(&game, burn),
            [
                vec![Target::Permanent(bears)],
                vec![Target::Permanent(knight)]
            ],
            "a white creature, a blue planeswalker and a land are not offered"
        );
    }
}

/// "Target creature or planeswalker an opponent controls that's red or
/// white."
#[test]
fn flourishing_grapple_targets_only_red_or_white_permanents_an_opponent_controls() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();
        let dragon = game
            .put_onto_battlefield(PlayerId::Two, cards::SHIVAN_DRAGON)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
            .unwrap();
        let grapple = hand(&mut game, cards::FLOURISHING_GRAPPLE);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 1);

        assert_eq!(
            castable_targets(&game, grapple),
            [
                vec![Target::Permanent(angel), Target::Permanent(bears)],
                vec![Target::Permanent(dragon), Target::Permanent(bears)],
            ],
            "not their green creature, their blue planeswalker, or your own creature"
        );
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

/// "Konstrari Charm deals 6 damage to target creature with flying" takes only
/// a flier; the counters take any creature.
#[test]
fn konstrari_charm_burns_only_a_creature_with_flying() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        let charm = hand(&mut game, cards::KONSTRARI_CHARM);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 1);

        let offered = castable_targets(&game, charm);
        let aimed_at = |id| {
            offered
                .iter()
                .filter(|targets| **targets == [Target::Permanent(id)])
                .count()
        };
        assert_eq!(aimed_at(angel), 2, "the damage or the counters");
        assert_eq!(aimed_at(bears), 1, "the counters only");
    }
}

/// "Another target creature or planeswalker you control gains hexproof."
#[test]
fn liliana_the_faultless_protects_another_creature_or_planeswalker_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let liliana = settled(&mut game, cards::LILIANA_THE_FAULTLESS);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let jace = settled(&mut game, cards::JACE_MEMORY_ADEPT);
        settled(&mut game, cards::MOX_RUBY);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        hand(&mut game, cards::FOREST);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 1);

        let mut targets = activation_targets(&game, liliana, "{1}, {T}, Discard");
        targets.sort();
        let mut wanted = [
            vec![Target::Permanent(bears)],
            vec![Target::Permanent(jace)],
        ];
        wanted.sort();
        assert_eq!(targets, wanted);

        activate_at(
            &mut game,
            liliana,
            "{1}, {T}, Discard",
            &[Target::Permanent(jace)],
        );
        drain_pending(&mut game);

        assert!(has_keyword(&game, jace, KeywordAbility::Hexproof));
        assert!(game.players[0].hand.is_empty(), "the card was discarded");
    }
}

/// "Exhaust — {5}{B}: Return target creature or planeswalker card from your
/// graveyard to the battlefield. Put a +1/+1 counter on Liliana. Activate
/// only as a sorcery. (Activate each exhaust ability only once.)"
#[test]
fn liliana_the_repentant_returns_a_creature_or_planeswalker_card_once() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let liliana = settled(&mut game, cards::LILIANA_THE_REPENTANT);
        let [bears, jace, _] = put_in_graveyard(
            &mut game,
            &[
                cards::GRIZZLY_BEARS,
                cards::JACE_MEMORY_ADEPT,
                cards::FOREST,
            ],
        )[..] else {
            unreachable!("three cards were put there");
        };
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Black, 12);

        let mut targets = activation_targets(&game, liliana, "Exhaust");
        targets.sort();
        let mut wanted = [vec![Target::Card(bears)], vec![Target::Card(jace)]];
        wanted.sort();
        assert_eq!(targets, wanted, "not the land card");

        activate_at(&mut game, liliana, "Exhaust", &[Target::Card(jace)]);
        drain_pending(&mut game);

        assert_eq!(controlled(&game, cards::JACE_MEMORY_ADEPT).len(), 1);
        assert_eq!(
            permanent(&game, liliana).counters(CounterKind::PlusOnePlusOne),
            1
        );
        assert!(
            activations(&game, liliana, "Exhaust").is_empty(),
            "exhausted, with the mana and a target still there"
        );
    }
}

/// "When this creature enters, it fights up to one target creature an
/// opponent controls."
#[test]
fn mind_meanderer_fights_the_opposing_creature_it_targets() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::MIND_MEANDERER, &[]);
        run_to_decision(&mut game).expect("the trigger chooses its target");
        assert_eq!(offered(&game, PlayerId::One), [theirs], "not your own");
        choose(&mut game, PlayerId::One, theirs);
        drain_pending(&mut game);

        let meanderer = controlled(&game, cards::MIND_MEANDERER)[0];
        assert!(!on_battlefield(&game, theirs), "four damage to a 2/2");
        assert_eq!(permanent(&game, meanderer).damage, 2);
    }
}

/// "Enchant artifact or non-Aura enchantment."
#[test]
fn puppet_crafting_enchants_an_artifact_or_an_enchantment_that_is_not_an_aura() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let mox = settled(&mut game, cards::MOX_RUBY);
        let crusade = game
            .put_onto_battlefield(PlayerId::Two, cards::CRUSADE)
            .unwrap();
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let aura = settled(&mut game, cards::HOLY_STRENGTH);
        permanent_mut(&mut game, aura).attached_to = Some(bears);
        settled(&mut game, cards::FOREST);
        let crafting = hand(&mut game, cards::PUPPET_CRAFTING);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 2);

        assert_eq!(
            castable_targets(&game, crafting),
            [
                vec![Target::Permanent(mox)],
                vec![Target::Permanent(crusade)]
            ],
            "not a creature, an Aura, or a land"
        );
    }
}

/// "Return another target permanent you control to its owner's hand.
/// Activate only during your turn."
#[test]
fn rescue_girl_returns_another_permanent_of_yours_only_during_your_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let rescue = settled(&mut game, cards::RESCUE_GIRL_FIRST_RESPONDER);
        let mox = settled(&mut game, cards::MOX_RUBY);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        assert_eq!(
            activation_targets(&game, rescue, "{T}:"),
            [vec![Target::Permanent(mox)]]
        );

        game.active_player = PlayerId::Two;
        assert!(activations(&game, rescue, "{T}:").is_empty());

        game.active_player = PlayerId::One;
        activate(&mut game, rescue, "{T}:");
        drain_pending(&mut game);
        assert!(!on_battlefield(&game, mox));
        assert_eq!(game.players[0].hand.len(), 1);
    }
}

/// "Exile target nonland permanent an opponent controls with mana value 3 or
/// less."
#[test]
fn solitary_cell_exiles_only_a_cheap_nonland_permanent_an_opponent_controls() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::GRIZZLY_BEARS);
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
            .unwrap();

        cast(&mut game, cards::SOLITARY_CELL, &[]);
        run_to_decision(&mut game).expect("the trigger chooses its target");

        assert_eq!(
            offered(&game, PlayerId::One),
            [bears],
            "not yours, not a five-drop, not a land"
        );
    }
}

/// "Stinging Vitriol deals 2 damage to target opponent. That player reveals
/// their hand. You choose a nonland card from it. They discard that card."
#[test]
fn stinging_vitriol_burns_an_opponent_and_takes_a_nonland_card_from_their_hand() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.players[1].hand = game
            .build_zone(
                PlayerId::Two,
                &[cards::FOREST, cards::LIGHTNING_BOLT, cards::GRIZZLY_BEARS],
            )
            .unwrap();
        let (bolt, bears) = (game.players[1].hand[1].id, game.players[1].hand[2].id);
        let vitriol = hand(&mut game, cards::STINGING_VITRIOL);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Black, 1);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);
        assert_eq!(
            castable_targets(&game, vitriol),
            [vec![Target::Player(PlayerId::Two)]],
            "an opponent, never yourself"
        );

        let action = casts(&game, vitriol).remove(0);
        game.apply(PlayerId::One, action).unwrap();
        run_to_decision(&mut game).expect("you choose from the revealed hand");
        assert_eq!(offered(&game, PlayerId::One), [bolt, bears], "not the land");
        choose(&mut game, PlayerId::One, bolt);
        drain_pending(&mut game);

        assert_eq!(game.players[1].life, 18);
        assert_eq!(game.players[1].hand.len(), 2);
        let graveyard = game.players[1].graveyard.iter().map(|card| card.definition);
        assert!(graveyard.eq([cards::LIGHTNING_BOLT]));
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

/// "For each opponent, return up to one target artifact or creature that
/// player controls to its owner's hand."
#[test]
fn the_theorist_bounces_only_an_artifact_or_creature_an_opponent_controls() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let jace = settled(&mut game, cards::THE_THEORIST_JACE_BELEREN);
        settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
            .unwrap();

        let mut targets = activation_targets(&game, jace, "−2:");
        targets.sort_by_key(Vec::len);
        assert_eq!(targets, [vec![], vec![Target::Permanent(theirs)]]);
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
