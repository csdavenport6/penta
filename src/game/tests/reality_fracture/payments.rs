//! What a spell or ability costs: reductions and taxes, casting without
//! paying a mana cost, and which permanents or cards a sacrifice, tap, or
//! discard cost accepts.
use super::*;

/// Whether `player` could cast the card `id` with the mana they have.
fn can_cast(game: &Game, player: PlayerId, id: GameObjectId) -> bool {
    game.legal_actions(player)
        .iter()
        .any(|action| matches!(action, Action::CastSpell { card, .. } if *card == id))
}

/// What player one could sacrifice to activate `source`'s ability whose text
/// starts with `prefix`, one list per legal activation, in a fixed order.
fn sacrifices_offered(game: &Game, source: GameObjectId, prefix: &str) -> Vec<Vec<GameObjectId>> {
    let mut offered = activations(game, source, prefix)
        .into_iter()
        .map(|action| match action {
            Action::ActivateAbility { cost_objects, .. } => cost_objects,
            _ => unreachable!("only activations are collected"),
        })
        .collect::<Vec<_>>();
    offered.sort();
    offered
}

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

        let mut wanted = [vec![bears], vec![jace]];
        wanted.sort();
        assert_eq!(
            sacrifices_offered(&game, edgar, "{2}, Sacrifice another"),
            wanted,
            "not Edgar himself, and not an artifact"
        );

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

/// "Noncreature spells you cast cost {1} less to cast": not a creature spell,
/// and not an opponent's spell.
#[test]
fn geist_of_saint_thalia_discounts_only_your_noncreature_spells() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::GEIST_OF_SAINT_THALIA);
        let divination = hand(&mut game, cards::DIVINATION);
        let bears = hand(&mut game, cards::GRIZZLY_BEARS);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 1);
        assert!(!can_cast(&game, PlayerId::One, bears), "{{1}}{{G}} still");
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 1);
        assert!(
            can_cast(&game, PlayerId::One, divination),
            "{{2}}{{U}} less {{1}}"
        );

        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        let theirs = in_hand_of(&mut game, PlayerId::Two, cards::DIVINATION);
        game.add_unrestricted_mana(PlayerId::Two, ManaColor::Blue, 2);
        assert!(!can_cast(&game, PlayerId::Two, theirs));
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

/// "{1}, Sacrifice another artifact: Put a +1/+1 counter on this creature. It
/// gains your choice of trample, hexproof, or haste until end of turn."
#[test]
fn hungering_puppetbeast_sacrifices_another_artifact_for_a_counter_and_a_keyword() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let beast = settled(&mut game, cards::HUNGERING_PUPPETBEAST);
        let heartwood = tokens(&game)[0].card.id;
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 1);

        let payments = activations(&game, beast, "{1}, Sacrifice another artifact")
            .into_iter()
            .map(|action| match action {
                Action::ActivateAbility { cost_objects, .. } => cost_objects,
                _ => unreachable!("only activations are collected"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            payments,
            [vec![heartwood]],
            "itself is not another artifact"
        );

        activate(&mut game, beast, "{1}, Sacrifice another artifact");
        run_to_decision(&mut game).expect("the keyword is chosen as it resolves");
        choose_decision_by_label(&mut game, PlayerId::One, "Hexproof");
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, heartwood));
        let beast = permanent(&game, beast);
        assert_eq!(beast.counters(CounterKind::PlusOnePlusOne), 1);
        assert!(game.permanent_has_executable_keyword(beast, KeywordAbility::Hexproof));
        assert!(!game.permanent_has_executable_keyword(beast, KeywordAbility::Trample));
        assert!(!game.permanent_has_executable_keyword(beast, KeywordAbility::Haste));
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

/// "You may cast spells with mana value less than or equal to the number of
/// creatures you control from your hand without paying their mana costs."
#[test]
fn omnipresence_frees_spells_up_to_the_number_of_creatures_you_control() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::OMNIPRESENCE);
        settled(&mut game, cards::GRIZZLY_BEARS);
        settled(&mut game, cards::GRIZZLY_BEARS);
        // The opponent's creatures are not yours to count.
        for _ in 0..3 {
            game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
                .unwrap();
        }
        let two = hand(&mut game, cards::GRIZZLY_BEARS);
        let three = hand(&mut game, cards::HURLOON_MINOTAUR);

        assert_eq!(casts(&game, two).len(), 1, "mana value 2, two creatures");
        assert!(casts(&game, three).is_empty(), "mana value 3 is too much");

        let action = casts(&game, two).remove(0);
        game.apply(PlayerId::One, action).unwrap();
        drain_pending(&mut game);

        assert_eq!(controlled(&game, cards::GRIZZLY_BEARS).len(), 3);
        assert_eq!(casts(&game, three).len(), 1, "a third creature allows it");
    }
}

/// "If a spell has {X} in its mana cost, you must choose 0 as the value of X
/// when casting it without paying its mana cost."
#[test]
fn omnipresence_casts_an_x_spell_only_for_zero() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::OMNIPRESENCE);
        settled(&mut game, cards::GRIZZLY_BEARS);
        let anguish = hand(&mut game, cards::AJANI_S_ANGUISH);

        let chosen = casts(&game, anguish)
            .into_iter()
            .map(|action| match action {
                Action::CastSpell { choices, .. } => choices.x(),
                _ => unreachable!("only casts are collected"),
            })
            .collect::<Vec<_>>();

        assert_eq!(chosen, [0], "{{X}}{{R}} has mana value 1 in hand");
    }
}

/// "{1}, {T}, Discard a legendary card: Draw a card."
#[test]
fn solitary_cell_draws_only_for_discarding_a_legendary_card() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        let cell = settled(&mut game, cards::SOLITARY_CELL);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 1);
        hand(&mut game, cards::GRIZZLY_BEARS);
        assert!(activations(&game, cell, "{1}, {T}").is_empty());
        assert_eq!(game.priority, PlayerId::One, "and not for want of priority");

        hand(&mut game, cards::YARGLE_GLUTTON_OF_URBORG);
        activate(&mut game, cell, "{1}, {T}");
        drain_pending(&mut game);

        let player = &game.players[0];
        assert_eq!(player.hand.len(), 2, "the Bears and the card drawn");
        let graveyard = player.graveyard.iter().map(|card| card.definition);
        assert!(graveyard.eq([cards::YARGLE_GLUTTON_OF_URBORG]));
    }
}

/// "You may sacrifice a land. If you do, create two tapped Heartwood tokens",
/// and "Tap two untapped artifacts you control" cannot be paid with them
/// until they untap.
#[test]
fn tenured_tethermage_taps_two_artifacts_for_its_counters() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::FOREST);

        cast(&mut game, cards::TENURED_TETHERMAGE, &[]);
        run_to_decision(&mut game).expect("sacrificing a land is offered");
        choose_decision_by_label(&mut game, PlayerId::One, "Sacrifice Forest");
        drain_pending(&mut game);

        let tethermage = controlled(&game, cards::TENURED_TETHERMAGE)[0];
        let heartwoods = tokens(&game)
            .iter()
            .map(|token| token.card.id)
            .collect::<Vec<_>>();
        assert_eq!(heartwoods.len(), 2);
        assert!(controlled(&game, cards::FOREST).is_empty());
        assert!(activations(&game, tethermage, "Tap two").is_empty());

        for heartwood in &heartwoods {
            permanent_mut(&mut game, *heartwood).tapped = false;
        }
        activate(&mut game, tethermage, "Tap two");
        drain_pending(&mut game);

        assert!(
            heartwoods
                .iter()
                .all(|heartwood| permanent(&game, *heartwood).tapped)
        );
        assert_eq!(
            permanent(&game, tethermage).counters(CounterKind::PlusOnePlusOne),
            2
        );
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

/// "This spell costs {2} less to cast if you've cast a noncreature spell this
/// turn": a creature spell earlier in the turn does not count.
#[test]
fn traxos_academy_guardian_costs_two_less_after_a_noncreature_spell_this_turn() {
    for (earlier, discounted) in [
        (None, false),
        (Some(cards::MEMNITE), false),
        (Some(cards::MOX_RUBY), true),
    ] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            if let Some(earlier) = earlier {
                cast(&mut game, earlier, &[]);
                drain_pending(&mut game);
            }
            for player in &mut game.players {
                player.mana_pool = ManaPool::default();
                player.mana.clear();
            }
            let traxos = hand(&mut game, cards::TRAXOS_ACADEMY_GUARDIAN);

            game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 2);
            assert_eq!(can_cast(&game, PlayerId::One, traxos), discounted);
            game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 2);
            assert!(can_cast(&game, PlayerId::One, traxos));
        }
    }
}

/// "This spell costs {1} less to cast if you control a legendary creature":
/// a count read while the spell is being priced, over your permanents only.
#[test]
fn wrath_of_the_bloodmane_costs_one_less_while_you_control_a_legendary_creature() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.put_onto_battlefield(PlayerId::Two, cards::YARGLE_GLUTTON_OF_URBORG)
            .unwrap();
        settled(&mut game, cards::GRIZZLY_BEARS);
        let wrath = hand(&mut game, cards::WRATH_OF_THE_BLOODMANE);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 2);

        assert!(casts(&game, wrath).is_empty(), "two mana is one short");

        settled(&mut game, cards::YARGLE_GLUTTON_OF_URBORG);
        assert!(
            !casts(&game, wrath).is_empty(),
            "your Yargle makes up the difference"
        );
    }
}
