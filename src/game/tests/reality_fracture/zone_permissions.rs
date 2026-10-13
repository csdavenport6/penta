//! Spells cast and abilities activated from a graveyard or a hand, and where
//! an effect that moves a card between zones puts it.
use super::*;

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

/// Identity Echo and a creature of yours on the battlefield over a library
/// holding `library`, bottom card first, with the ability activated at that
/// creature and resolved. Returns the creature.
fn identity_echoed(game: &mut Game, library: &[CardDefinitionId]) -> GameObjectId {
    let echo = settled(game, cards::IDENTITY_ECHO);
    let bears = settled(game, cards::GRIZZLY_BEARS);
    game.players[0].library = game.build_zone(PlayerId::One, library).unwrap();
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 4);
    activate_at(game, echo, "{3}{R}", &[Target::Permanent(bears)]);
    drain_pending(game);
    bears
}

/// "Reveal cards from the top of your library until you reveal a creature or
/// planeswalker card. Put that card onto the battlefield and the rest on the
/// bottom of your library in a random order."
#[test]
fn identity_echo_trades_your_permanent_for_the_first_creature_or_planeswalker_card() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);

        let bears = identity_echoed(
            &mut game,
            &[
                cards::SERRA_ANGEL,
                cards::JACE_MEMORY_ADEPT,
                cards::FOREST,
                cards::MOUNTAIN,
            ],
        );

        assert!(!on_battlefield(&game, bears));
        assert_eq!(game.players[0].exile.len(), 1);
        assert_eq!(controlled(&game, cards::JACE_MEMORY_ADEPT).len(), 1);
        assert!(controlled(&game, cards::SERRA_ANGEL).is_empty());
        let library = game.players[0]
            .library
            .iter()
            .map(|card| card.definition)
            .collect::<Vec<_>>();
        assert!(
            library[..2].contains(&cards::FOREST) && library[..2].contains(&cards::MOUNTAIN),
            "the rest go underneath"
        );
        assert_eq!(library[2], cards::SERRA_ANGEL, "never revealed");
    }
}

/// With no such card to find, the whole library is revealed and put back,
/// and the exiled permanent is still gone.
#[test]
fn identity_echo_finding_nothing_still_exiles_the_permanent() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);

        let bears = identity_echoed(&mut game, &[cards::FOREST, cards::MOUNTAIN]);

        assert!(!on_battlefield(&game, bears));
        assert_eq!(game.players[0].exile.len(), 1);
        assert_eq!(game.players[0].library.len(), 2);
        assert_eq!(
            game.battlefield
                .iter()
                .filter(|permanent| permanent.controller == PlayerId::One)
                .count(),
            1,
            "only Identity Echo is left"
        );
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

/// Player one's legal activations of the card or permanent `source`,
/// wherever it is.
fn activations_of(game: &Game, source: GameObjectId) -> Vec<Action> {
    game.legal_actions(PlayerId::One)
        .into_iter()
        .filter(
            |action| matches!(action, Action::ActivateAbility { source: activated, .. } if *activated == source),
        )
        .collect()
}

/// "{3}{G}, Discard this card: Destroy target creature with flying."
#[test]
fn sureshot_sower_is_discarded_from_hand_to_destroy_a_creature_with_flying() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        let on_the_battlefield = settled(&mut game, cards::SURESHOT_SOWER);
        let sower = hand(&mut game, cards::SURESHOT_SOWER);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 4);

        let mut from_hand = activations_of(&game, sower);
        let action = from_hand.remove(0);
        assert_eq!(announced_targets(&action), [Target::Permanent(angel)]);
        assert!(from_hand.is_empty(), "only a creature with flying");
        assert!(
            activations_of(&game, on_the_battlefield).is_empty(),
            "the ability works only from hand"
        );

        game.apply(PlayerId::One, action).unwrap();
        assert!(game.players[0].hand.is_empty(), "discarded as a cost");
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, angel));
        assert_eq!(game.players[0].graveyard.len(), 1);
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
