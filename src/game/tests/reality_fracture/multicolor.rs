//! Reality Fracture's multicolor cards.
use super::*;

/// "{6}: Create a Heartwood token. Then Aerid Konstrari gets +X/+0 until end
/// of turn, where X is the number of artifacts you control": the token just
/// made is one of them.
#[test]
fn aerid_konstrari_counts_the_heartwood_its_ability_just_created() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let aerid = settled(&mut game, cards::AERID_KONSTRARI);
        assert_eq!(tokens(&game).len(), 1, "one Heartwood for entering");
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 6);

        activate(&mut game, aerid, "{6}");
        drain_pending(&mut game);

        assert_eq!(tokens(&game).len(), 2);
        assert_eq!(game.power(permanent(&game, aerid)), Some(7));
    }
}

/// Clash of Elements cast at the opponent's permanent `target`, resolved as
/// far as its owner's choice. The opponent's library holds three cards.
fn clash_of_elements_asking(game: &mut Game, target: GameObjectId) {
    game.players[1].library = game.build_zone(PlayerId::Two, &[cards::FOREST; 3]).unwrap();
    cast(game, cards::CLASH_OF_ELEMENTS, &[Target::Permanent(target)]);
    run_to_decision(game).expect("the permanent's owner chooses");
}

/// "Its owner may put it on top of their library. If they do, Clash of
/// Elements deals 2 damage to them."
#[test]
fn clash_of_elements_deals_two_damage_to_an_owner_who_keeps_the_card_on_top() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        clash_of_elements_asking(&mut game, bears);

        choose_decision_by_label(&mut game, PlayerId::Two, "Top");
        drain_pending(&mut game);

        let library = &game.players[1].library;
        assert_eq!(library.len(), 4);
        assert_eq!(library.last().unwrap().definition, cards::GRIZZLY_BEARS);
        assert_eq!(game.players[1].life, 18);
    }
}

/// "If they didn't put the card on top of their library, they put it on the
/// bottom."
#[test]
fn clash_of_elements_puts_the_card_on_the_bottom_for_an_owner_who_declines() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        clash_of_elements_asking(&mut game, bears);

        choose_decision_by_label(&mut game, PlayerId::Two, "Bottom");
        drain_pending(&mut game);

        let library = &game.players[1].library;
        assert_eq!(library.len(), 4);
        assert_eq!(library.first().unwrap().definition, cards::GRIZZLY_BEARS);
        assert_eq!(game.players[1].life, 20);
    }
}

/// A token put on top of a library was put there, and is gone the next time
/// state-based actions are checked (CR 704.5d).
#[test]
fn clash_of_elements_deals_its_damage_for_a_token_put_on_top() {
    static SOLDIER: TokenCharacteristics =
        TokenCharacteristics::creature(&["Soldier"], &[ManaColor::White], 1, 1);
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let soldier = game.create_token_from(PlayerId::Two, SOLDIER, None);
        clash_of_elements_asking(&mut game, soldier);

        choose_decision_by_label(&mut game, PlayerId::Two, "Top");
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, soldier));
        assert_eq!(game.players[1].library.len(), 3);
        assert_eq!(game.players[1].life, 18);
    }
}

/// A commander its owner moves to the command zone instead (CR 903.9b) was
/// not put on top of their library, so it costs them nothing.
#[test]
fn clash_of_elements_deals_no_damage_for_a_commander_kept_out_of_the_library() {
    for prepared_engine in [false, true] {
        let mut game = commander_game(Format::DuelCommander);
        game.set_prepared_engine_enabled(prepared_engine);
        let commander = game.players[1].command[0].id;
        game.move_target_to_zone(
            Target::Card(commander),
            ZoneKind::Battlefield,
            ZoneMoveCause::Rules,
            None,
            ZonePlacement::Top,
        );
        let commander = game.battlefield.last().unwrap().card.id;
        let (library, life) = (game.players[1].library.len(), game.players[1].life);

        cast(
            &mut game,
            cards::CLASH_OF_ELEMENTS,
            &[Target::Permanent(commander)],
        );
        run_to_decision(&mut game).expect("the commander's owner chooses");
        choose_decision_by_label(&mut game, PlayerId::Two, "Top");
        choose_decision_by_label(&mut game, PlayerId::Two, "Move to the command zone");
        drain_pending(&mut game);

        assert_eq!(game.players[1].command.len(), 2);
        assert_eq!(game.players[1].library.len(), library);
        assert_eq!(game.players[1].life, life);
    }
}

/// "You may sacrifice a planeswalker. If you do, search your library for a
/// planeswalker card, put it onto the battlefield, then shuffle."
#[test]
fn entrust_the_spark_trades_a_planeswalker_for_one_from_your_library() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let jace = settled(&mut game, cards::JACE_MEMORY_ADEPT);
        game.players[0].library = game
            .build_zone(
                PlayerId::One,
                &[cards::FOREST, cards::CHANDRA_TORCH_OF_DEFIANCE],
            )
            .unwrap();

        cast(&mut game, cards::ENTRUST_THE_SPARK, &[]);
        run_to_decision(&mut game).expect("sacrificing a planeswalker is offered");
        choose_decision_by_label(&mut game, PlayerId::One, "Sacrifice Jace, Memory Adept");
        let search = run_to_decision(&mut game).expect("the search chooses its card");
        let found = search
            .options
            .iter()
            .filter(|option| option.card.is_some())
            .map(|option| option.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(found, ["Chandra, Torch of Defiance"], "not the Forest");
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, jace));
        assert_eq!(controlled(&game, cards::CHANDRA_TORCH_OF_DEFIANCE).len(), 1);
    }
}

/// "When enchanted creature dies, return that card to the battlefield tapped
/// under its owner's control": the creature's owner, not the Aura's.
#[test]
fn ferocity_of_the_hunt_returns_the_creature_tapped_to_its_owner() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        cast(
            &mut game,
            cards::FEROCITY_OF_THE_HUNT,
            &[Target::Permanent(theirs)],
        );
        drain_pending(&mut game);
        assert_eq!(game.power(permanent(&game, theirs)), Some(3));
        assert!(has_keyword(&game, theirs, KeywordAbility::Deathtouch));

        game.destroy_permanent(theirs);
        drain_pending(&mut game);

        let returned = game
            .battlefield
            .iter()
            .find(|permanent| permanent.card.definition == cards::GRIZZLY_BEARS)
            .expect("the card came back");
        assert_eq!(returned.controller, PlayerId::Two);
        assert!(returned.tapped);
        assert_eq!(
            game.power(returned),
            Some(2),
            "the Aura did not come with it"
        );
    }
}

/// "Whenever a creature you control attacks, that creature deals 1 damage to
/// each opponent": the attacker is the source, so its lifelink counts.
#[test]
fn ingris_stingerquill_has_the_attacking_creature_deal_the_damage() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::INGRIS_STINGERQUILL);
        let hawk = settled(&mut game, cards::HEALER_S_HAWK);

        attack_with(&mut game, hawk);

        assert_eq!(game.players[1].life, 19);
        assert_eq!(game.players[0].life, 21, "the Hawk has lifelink");
    }
}

/// "Create a ... Cadet. Then creatures you control gain haste until end of
/// turn": the Cadet is there to gain it, and a later arrival is not.
#[test]
fn ingris_stingerquill_gives_the_cadet_it_creates_haste() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ingris = settled(&mut game, cards::INGRIS_STINGERQUILL);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 4);

        activate(&mut game, ingris, "{4}");
        drain_pending(&mut game);

        let cadet = tokens(&game)[0].card.id;
        assert!(has_keyword(&game, cadet, KeywordAbility::Haste));
        let late = settled(&mut game, cards::GRIZZLY_BEARS);
        assert!(!has_keyword(&game, late, KeywordAbility::Haste));
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

/// "Add {C}{C}{C}": a mode of a spell, so the mana arrives as it resolves.
#[test]
fn konstrari_charm_adds_three_colorless_mana() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);

        cast(&mut game, cards::KONSTRARI_CHARM, &[]);
        assert_eq!(game.players[0].mana_pool.colorless, 0);
        drain_pending(&mut game);

        assert_eq!(game.players[0].mana_pool.colorless, 3);
    }
}

/// "Whenever you gain life, create a ... Lotus ... This ability triggers only
/// once each turn", and the Lotus is spent for three mana of one color.
#[test]
fn kwia_vigorbloom_makes_one_lotus_a_turn_worth_three_mana_of_one_color() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::KWIA_VIGORBLOOM);

        for _ in 0..2 {
            game.gain_life(PlayerId::One, 1);
            drain_pending(&mut game);
        }

        let lotuses = tokens(&game)
            .iter()
            .map(|token| token.card.id)
            .collect::<Vec<_>>();
        assert_eq!(lotuses.len(), 1, "only the first life gained this turn");
        let spend = game
            .legal_actions(PlayerId::One)
            .into_iter()
            .find(|action| {
                matches!(action, Action::ActivateManaAbility { source, .. } if *source == lotuses[0])
            })
            .expect("the Lotus can be tapped for mana");
        game.apply(PlayerId::One, spend).unwrap();
        assert!(!on_battlefield(&game, lotuses[0]));
        assert_eq!(game.players[0].mana_pool.total(), 3);
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

/// "Mill four cards. When you do, return target land card from your graveyard
/// to the battlefield tapped": the target is chosen after the mill, so a land
/// it just turned over can be the one.
#[test]
fn primal_witchstalker_returns_a_land_it_just_milled_tapped() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.players[0].library = game
            .build_zone(
                PlayerId::One,
                &[
                    cards::GRIZZLY_BEARS,
                    cards::GRIZZLY_BEARS,
                    cards::GRIZZLY_BEARS,
                    cards::GRIZZLY_BEARS,
                    cards::FOREST,
                ],
            )
            .unwrap();

        cast(&mut game, cards::PRIMAL_WITCHSTALKER, &[]);
        run_to_decision(&mut game).expect("the reflexive trigger chooses its target");
        let lands = offered(&game, PlayerId::One);
        assert_eq!(lands.len(), 1, "the one land in the graveyard");
        choose(&mut game, PlayerId::One, lands[0]);
        drain_pending(&mut game);

        assert_eq!(game.players[0].library.len(), 1);
        let forest = controlled(&game, cards::FOREST)[0];
        assert!(permanent(&game, forest).tapped);
    }
}

/// A milled card that goes somewhere else instead was still milled
/// (CR 701.17c), so four cards were: Blightsteel Colossus shuffles itself
/// back, and the land beside it is still returned.
#[test]
fn primal_witchstalker_counts_a_milled_card_that_went_elsewhere() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.players[0].library = game
            .build_zone(
                PlayerId::One,
                &[
                    cards::GRIZZLY_BEARS,
                    cards::FOREST,
                    cards::BLIGHTSTEEL_COLOSSUS,
                    cards::GRIZZLY_BEARS,
                ],
            )
            .unwrap();

        cast(&mut game, cards::PRIMAL_WITCHSTALKER, &[]);
        run_to_decision(&mut game).expect("the reflexive trigger chooses its target");
        let lands = offered(&game, PlayerId::One);
        choose(&mut game, PlayerId::One, lands[0]);
        drain_pending(&mut game);

        assert_eq!(game.players[0].library.len(), 1, "only the Colossus");
        assert_eq!(controlled(&game, cards::FOREST).len(), 1);
    }
}

/// A library of three cannot have four cards milled from it (CR 701.17b), so
/// "when you do" never happens, whatever is already in the graveyard.
#[test]
fn primal_witchstalker_returns_nothing_unless_four_cards_were_milled() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.players[0].library = game
            .build_zone(PlayerId::One, &[cards::GRIZZLY_BEARS; 3])
            .unwrap();
        put_in_graveyard(&mut game, &[cards::FOREST]);

        cast(&mut game, cards::PRIMAL_WITCHSTALKER, &[]);
        assert!(run_to_decision(&mut game).is_none(), "no trigger follows");

        assert!(game.players[0].library.is_empty(), "as many as possible");
        assert!(controlled(&game, cards::FOREST).is_empty());
    }
}

/// Player one casts the card with `definition` out of their graveyard with
/// mana to spare, leaving it on the stack.
fn cast_from_graveyard(game: &mut Game, definition: CardDefinitionId) {
    let card = put_in_graveyard(game, &[definition])[0];
    mana_to_spare(game);
    let action = casts(game, card)
        .into_iter()
        .next()
        .expect("the card can be cast from the graveyard");
    game.apply(PlayerId::One, action).unwrap();
}

/// The +1/+1 counters on each of player one's tokens.
fn token_counters(game: &Game) -> Vec<u16> {
    tokens(game)
        .iter()
        .map(|token| token.counters(CounterKind::PlusOnePlusOne))
        .collect()
}

/// "If this spell was cast from a graveyard, put a +1/+1 counter on each of
/// them for every three cards in your graveyard": the spell is on the stack,
/// not in the graveyard, while that is counted.
#[test]
fn recursive_recruitment_flashed_back_counts_the_graveyard_it_left() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        put_in_graveyard(&mut game, &[cards::MOUNTAIN; 8]);

        cast_from_graveyard(&mut game, cards::RECURSIVE_RECRUITMENT);
        drain_pending(&mut game);

        assert_eq!(token_counters(&game), [2, 2], "eight cards, not nine");
        assert_eq!(game.players[0].exile.len(), 1, "flashback exiles it");
    }
}

#[test]
fn recursive_recruitment_cast_from_hand_adds_no_counters() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        put_in_graveyard(&mut game, &[cards::MOUNTAIN; 8]);

        cast(&mut game, cards::RECURSIVE_RECRUITMENT, &[]);
        drain_pending(&mut game);

        assert_eq!(token_counters(&game), [0, 0]);
    }
}

/// "Whenever an opponent casts a spell with mana value 2 or less, you gain 2
/// life": not your own spell, and not a three-drop.
#[test]
fn solarium_sentry_gains_life_only_for_an_opponents_cheap_spell() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::SOLARIUM_SENTRY);
        cast(&mut game, cards::GRIZZLY_BEARS, &[]);
        drain_pending(&mut game);
        assert_eq!(game.players[0].life, 20, "your own two-drop");

        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        for (definition, life) in [(cards::GRAY_OGRE, 20), (cards::GRIZZLY_BEARS, 22)] {
            let spell = game
                .build_zone(PlayerId::Two, &[definition])
                .unwrap()
                .remove(0);
            let id = spell.id;
            game.players[1].hand.push(spell);
            for color in ManaColor::COLORS {
                game.add_unrestricted_mana(PlayerId::Two, color, 3);
            }
            let action = game
                .legal_actions(PlayerId::Two)
                .into_iter()
                .find(|action| matches!(action, Action::CastSpell { card, .. } if *card == id))
                .expect("the opponent can cast it");
            game.apply(PlayerId::Two, action).unwrap();
            drain_pending(&mut game);

            assert_eq!(game.players[0].life, life);
        }
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

/// "Create a ... Cadet. It gains haste until end of turn."
#[test]
fn stingerquill_charm_gives_the_cadet_it_creates_haste() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);

        cast(&mut game, cards::STINGERQUILL_CHARM, &[]);
        drain_pending(&mut game);

        let cadet = tokens(&game)[0].card.id;
        assert!(has_keyword(&game, cadet, KeywordAbility::Haste));
        assert!(!has_keyword(&game, bears, KeywordAbility::Haste));
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

/// "Draw a card. If this spell wasn't cast from your hand, draw two cards
/// instead."
#[test]
fn twinned_vision_draws_one_card_when_cast_from_hand() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);

        cast(&mut game, cards::TWINNED_VISION, &[]);
        drain_pending(&mut game);

        assert_eq!(game.players[0].hand.len(), 1);
    }
}

/// "Flashback—{1}{U/R}{U/R}, Discard a card."
#[test]
fn twinned_vision_flashed_back_costs_a_discard_and_draws_two_cards() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let vision = put_in_graveyard(&mut game, &[cards::TWINNED_VISION])[0];
        mana_to_spare(&mut game);
        assert!(casts(&game, vision).is_empty(), "nothing to discard");

        hand(&mut game, cards::MOUNTAIN);
        hand(&mut game, cards::MOUNTAIN);
        let action = casts(&game, vision).remove(0);
        game.apply(PlayerId::One, action).unwrap();
        drain_pending(&mut game);

        let player = &game.players[0];
        assert_eq!(player.hand.len(), 3, "one Mountain kept, two cards drawn");
        let graveyard = player.graveyard.iter().map(|card| card.definition);
        assert!(graveyard.eq([cards::MOUNTAIN]), "the other discarded");
        assert_eq!(player.exile.len(), 1, "flashback exiles it");
    }
}

/// A copy of a spell is not cast at all, so it was not cast from your hand
/// even when the spell it copies was.
#[test]
fn a_copy_of_twinned_vision_draws_two_cards() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let vision = hand(&mut game, cards::TWINNED_VISION);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 2);
        let action = casts(&game, vision).remove(0);
        game.apply(PlayerId::One, action).unwrap();

        let spell = game.stack.last().expect("it is on the stack").id;
        cast(&mut game, cards::TWINCAST, &[Target::Spell(spell)]);
        drain_pending(&mut game);

        assert_eq!(
            game.players[0].hand.len(),
            3,
            "two for the copy, one for the card"
        );
    }
}

/// "Destroy target nonland permanent. Put a +1/+1 counter on each creature
/// target player controls."
#[test]
fn twisted_fates_puts_its_counters_on_the_target_players_creatures() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let yours = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        let mox = game
            .put_onto_battlefield(PlayerId::Two, cards::MOX_RUBY)
            .unwrap();

        cast(
            &mut game,
            cards::TWISTED_FATES,
            &[Target::Permanent(mox), Target::Player(PlayerId::One)],
        );
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, mox));
        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(counters(yours), 1);
        assert_eq!(counters(theirs), 0);
    }
}

/// "Put a +1/+1 counter on target creature you control. Then it fights target
/// creature an opponent controls": the counter is on it when it fights.
#[test]
fn vigorbloom_charm_fights_with_the_counter_it_just_placed() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let wurm = game
            .put_onto_battlefield(PlayerId::Two, cards::CRAW_WURM)
            .unwrap();

        cast(
            &mut game,
            cards::VIGORBLOOM_CHARM,
            &[Target::Permanent(bears), Target::Permanent(wurm)],
        );
        drain_pending(&mut game);

        assert_eq!(permanent(&game, wurm).damage, 3);
        assert!(!on_battlefield(&game, bears), "six damage to a 3/3");
    }
}

/// "If that permanent's mana value was 3 or less, return it to the
/// battlefield tapped under your control. Exile it at the beginning of the
/// next end step."
#[test]
fn vindictive_triumph_borrows_a_cheap_permanent_until_the_next_end_step() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(
            &mut game,
            cards::VINDICTIVE_TRIUMPH,
            &[Target::Permanent(theirs)],
        );
        drain_pending(&mut game);

        let borrowed = controlled(&game, cards::GRIZZLY_BEARS);
        assert_eq!(borrowed.len(), 1, "it is yours now");
        assert!(permanent(&game, borrowed[0]).tapped);

        end_step_begins(&mut game);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, borrowed[0]));
        let exiled = game.players[1].exile.iter().map(|card| card.definition);
        assert!(exiled.eq([cards::GRIZZLY_BEARS]), "exiled to its owner");
    }
}

#[test]
fn vindictive_triumph_only_exiles_a_permanent_costing_more_than_three() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();

        cast(
            &mut game,
            cards::VINDICTIVE_TRIUMPH,
            &[Target::Permanent(angel)],
        );
        drain_pending(&mut game);

        assert!(controlled(&game, cards::SERRA_ANGEL).is_empty());
        assert_eq!(game.players[1].exile.len(), 1);
    }
}

/// "If the permanent had {X} in its mana cost, X is 0 for the purpose of
/// determining its mana value."
#[test]
fn vindictive_triumph_reads_x_as_zero_in_a_permanents_mana_value() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        cast_for_x(&mut game, cards::MUTAGEN_MAN_LIVING_OOZE, 3);
        drain_pending(&mut game);
        let ooze = controlled(&game, cards::MUTAGEN_MAN_LIVING_OOZE)[0];

        cast(
            &mut game,
            cards::VINDICTIVE_TRIUMPH,
            &[Target::Permanent(ooze)],
        );
        drain_pending(&mut game);

        let returned = controlled(&game, cards::MUTAGEN_MAN_LIVING_OOZE);
        assert_eq!(
            returned.len(),
            1,
            "{{X}}{{G}}{{G}} is two on the battlefield"
        );
        assert!(permanent(&game, returned[0]).tapped);
    }
}
