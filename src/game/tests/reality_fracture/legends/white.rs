//! Reality Fracture's white legends.
use super::*;

/// "Whenever you gain life, put a loyalty counter on Ajani" watches the life
/// his own "0: You gain 1 life" gains.
#[test]
fn ajani_resolute_gains_a_loyalty_counter_for_the_life_his_own_ability_gains() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ajani = settled(&mut game, cards::AJANI_RESOLUTE);
        assert_eq!(permanent(&game, ajani).counters(CounterKind::Loyalty), 2);

        activate(&mut game, ajani, "0:");
        drain_pending(&mut game);

        assert_eq!(game.players[0].life, 21);
        assert_eq!(permanent(&game, ajani).counters(CounterKind::Loyalty), 3);
    }
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

/// "Whenever you cast an Equipment spell or a spell that targets a creature
/// you control, draw a card. This ability triggers only once each turn."
#[test]
fn danitha_sword_of_hope_draws_once_a_turn_for_equipment_or_a_spell_aimed_at_your_creature() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let danitha = settled(&mut game, cards::DANITHA_SWORD_OF_HOPE);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::GIANT_GROWTH, &[Target::Permanent(theirs)]);
        drain_pending(&mut game);
        assert!(
            game.players[0].hand.is_empty(),
            "not a creature you control"
        );

        cast(&mut game, cards::BONESPLITTER, &[]);
        drain_pending(&mut game);
        assert_eq!(game.players[0].hand.len(), 1, "an Equipment spell");

        cast(
            &mut game,
            cards::GIANT_GROWTH,
            &[Target::Permanent(danitha)],
        );
        drain_pending(&mut game);
        assert_eq!(game.players[0].hand.len(), 1, "only once each turn");
    }
}

#[test]
fn danitha_sword_of_hope_draws_for_a_spell_that_targets_a_creature_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let danitha = settled(&mut game, cards::DANITHA_SWORD_OF_HOPE);

        cast(
            &mut game,
            cards::GIANT_GROWTH,
            &[Target::Permanent(danitha)],
        );
        drain_pending(&mut game);

        assert_eq!(game.players[0].hand.len(), 1);
    }
}

/// "Whenever you cast": an opponent's Equipment spell is not yours.
#[test]
fn danitha_sword_of_hope_draws_nothing_for_an_opponents_equipment_spell() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        settled(&mut game, cards::DANITHA_SWORD_OF_HOPE);
        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        let bonesplitter = in_hand_of(&mut game, PlayerId::Two, cards::BONESPLITTER);
        game.add_unrestricted_mana(PlayerId::Two, ManaColor::Colorless, 1);

        let action = game
            .legal_actions(PlayerId::Two)
            .into_iter()
            .find(
                |action| matches!(action, Action::CastSpell { card, .. } if *card == bonesplitter),
            )
            .expect("the opponent can cast it");
        game.apply(PlayerId::Two, action).unwrap();
        drain_pending(&mut game);

        assert!(game.players[0].hand.is_empty());
    }
}

/// "This spell costs {X} less to cast, where X is the greatest toughness
/// among creatures you control", and never less than {W}.
#[test]
fn ghalta_the_immovable_costs_less_by_the_greatest_toughness_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ghalta = hand(&mut game, cards::GHALTA_THE_IMMOVABLE);
        settled(&mut game, cards::GRIZZLY_BEARS);
        settled(&mut game, cards::WALL_OF_WOOD);
        game.put_onto_battlefield(PlayerId::Two, cards::WALL_OF_STONE)
            .unwrap();

        // The 0/3 Wall is the greatest of yours; the opponent's 0/8 is not.
        game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 5);
        assert!(
            !can_cast(&game, PlayerId::One, ghalta),
            "{{8}}{{W}} less {{3}}"
        );
        game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 1);
        assert!(can_cast(&game, PlayerId::One, ghalta));
    }
}

#[test]
fn ghalta_the_immovable_never_costs_less_than_its_white_mana() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ghalta = hand(&mut game, cards::GHALTA_THE_IMMOVABLE);
        settled(&mut game, cards::WALL_OF_STONE);
        settled(&mut game, cards::WALL_OF_STONE);

        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 1);
        assert!(
            !can_cast(&game, PlayerId::One, ghalta),
            "{{W}} is still owed"
        );
        game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 1);
        assert!(can_cast(&game, PlayerId::One, ghalta));
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

/// The damage Ghalta itself assigns is its own toughness: a 0/7 hits for 7.
#[test]
fn ghalta_the_immovable_assigns_its_own_toughness() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ghalta = settled(&mut game, cards::GHALTA_THE_IMMOVABLE);

        connect_with(&mut game, ghalta);

        assert_eq!(game.players[1].life, 13);
    }
}

/// "Creature tokens you control get +1/+0 and have vigilance": not a
/// creature card, and not the opponent's token.
#[test]
fn gideons_memorial_strengthens_only_creature_tokens_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::GIDEON_S_MEMORIAL);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let yours = game.create_token_from(PlayerId::One, SOLDIER, None);
        let theirs = game.create_token_from(PlayerId::Two, SOLDIER, None);

        let stats = |id| {
            let permanent = permanent(&game, id);
            (
                game.power(permanent),
                game.toughness(permanent),
                has_keyword(&game, id, KeywordAbility::Vigilance),
            )
        };
        assert_eq!(stats(yours), (Some(2), Some(1), true));
        assert_eq!(stats(theirs), (Some(1), Some(1), false));
        assert_eq!(stats(bears), (Some(2), Some(2), false));
    }
}

/// "Spend this mana only to cast a planeswalker spell."
#[test]
fn gideons_memorial_mana_casts_a_planeswalker_spell_and_nothing_else() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let memorial = settled(&mut game, cards::GIDEON_S_MEMORIAL);
        let ajani = hand(&mut game, cards::AJANI_RESOLUTE);
        let lions = hand(&mut game, cards::SAVANNAH_LIONS);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Colorless, 1);

        tap_for(&mut game, memorial, ManaColor::White);

        assert!(can_cast(&game, PlayerId::One, ajani), "{{1}}{{W}}");
        assert!(!can_cast(&game, PlayerId::One, lions), "{{W}}");
    }
}

/// "{1}{W}, Discard this card: It deals 4 damage to target attacking or
/// blocking creature." The ability is the card's own, used from hand.
#[test]
fn gideons_memorial_discarded_from_hand_deals_four_damage_to_an_attacking_creature() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let giant = settled(&mut game, cards::HILL_GIANT);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        attack_with(&mut game, giant);
        let on_battlefield_already = settled(&mut game, cards::GIDEON_S_MEMORIAL);
        let memorial = hand(&mut game, cards::GIDEON_S_MEMORIAL);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::White, 2);
        assert!(
            activations(&game, on_battlefield_already, "{1}{W}, Discard").is_empty(),
            "only a Memorial in hand can be discarded"
        );

        let actions = game
            .legal_actions(PlayerId::One)
            .into_iter()
            .filter(|action| {
                matches!(action, Action::ActivateAbility { source, .. } if *source == memorial)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actions.iter().map(announced_targets).collect::<Vec<_>>(),
            [vec![Target::Permanent(giant)]],
            "the attacker, and not the creature standing by"
        );
        game.apply(PlayerId::One, actions[0].clone()).unwrap();
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, giant), "4 damage to a 3/3");
        assert_eq!(
            game.players[0].graveyard[0].definition,
            cards::GIDEON_S_MEMORIAL
        );
    }
}

/// "When a Plains you control enters, both of Koth's abilities will trigger."
#[test]
fn koth_of_the_homestead_triggers_twice_for_a_plains_and_once_for_another_land() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let koth = settled(&mut game, cards::KOTH_OF_THE_HOMESTEAD);

        settled(&mut game, cards::FOREST);
        assert_eq!(game.players[0].life, 21);
        assert_eq!(
            permanent(&game, koth).counters(CounterKind::PlusOnePlusOne),
            0
        );

        settled(&mut game, cards::PLAINS);
        assert_eq!(game.players[0].life, 22);
        assert_eq!(
            permanent(&game, koth).counters(CounterKind::PlusOnePlusOne),
            1,
            "the only creature to target"
        );
    }
}

/// "Whenever another creature or planeswalker you control enters, you gain 1
/// life": not Liliana herself, and not an opponent's creature.
#[test]
fn liliana_the_faultless_gains_life_for_another_creature_or_planeswalker_of_yours() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::LILIANA_THE_FAULTLESS);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        drain_pending(&mut game);
        settled(&mut game, cards::MOX_RUBY);
        assert_eq!(game.players[0].life, 20);

        settled(&mut game, cards::GRIZZLY_BEARS);
        settled(&mut game, cards::JACE_MEMORY_ADEPT);

        assert_eq!(game.players[0].life, 22);
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

/// "Whenever you gain life, put a +1/+1 counter on each Angel you control":
/// Lyra is one of them, and an opponent's Angel is not.
#[test]
fn lyra_archangel_of_dawn_puts_a_counter_on_each_angel_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let lyra = settled(&mut game, cards::LYRA_ARCHANGEL_OF_DAWN);
        let angel = settled(&mut game, cards::SERRA_ANGEL);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();

        game.gain_life(PlayerId::One, 2);
        drain_pending(&mut game);

        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(
            [lyra, angel, bears, theirs].map(counters),
            [1, 1, 0, 0],
            "one trigger for one life gain, however much life"
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

/// "Noncreature spells your opponents cast cost {1} more to cast": yours
/// cost what they always did.
#[test]
fn thalia_the_survivor_taxes_only_your_opponents_noncreature_spells() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::THALIA_THE_SURVIVOR);
        let yours = hand(&mut game, cards::LIGHTNING_BOLT);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);
        assert!(can_cast(&game, PlayerId::One, yours));

        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        let bolt = in_hand_of(&mut game, PlayerId::Two, cards::LIGHTNING_BOLT);
        let goblin = in_hand_of(&mut game, PlayerId::Two, cards::MONSS_GOBLIN_RAIDERS);
        game.add_unrestricted_mana(PlayerId::Two, ManaColor::Red, 1);
        assert!(
            !can_cast(&game, PlayerId::Two, bolt),
            "{{R}} and {{1}} more"
        );
        assert!(can_cast(&game, PlayerId::Two, goblin), "a creature spell");
        game.add_unrestricted_mana(PlayerId::Two, ManaColor::Red, 1);
        assert!(can_cast(&game, PlayerId::Two, bolt));
    }
}
