//! Reality Fracture's red cards.
use super::*;

/// "When this enchantment enters, it deals X damage to any target": the X the
/// spell was cast for, read by a trigger that is a different object.
#[test]
fn ajanis_anguish_deals_the_x_it_was_cast_for() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();

        cast_for_x(&mut game, cards::AJANI_S_ANGUISH, 3);
        run_to_decision(&mut game).expect("the trigger chooses its target");
        choose(&mut game, PlayerId::One, angel);
        drain_pending(&mut game);

        assert_eq!(permanent(&game, angel).damage, 3);
    }
}

/// X is the spell's, so the trigger still knows it after the enchantment is
/// gone.
#[test]
fn ajanis_anguish_keeps_its_x_after_leaving_the_battlefield() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();

        cast_for_x(&mut game, cards::AJANI_S_ANGUISH, 3);
        run_to_decision(&mut game).expect("the trigger chooses its target");
        choose(&mut game, PlayerId::One, angel);
        let anguish = controlled(&game, cards::AJANI_S_ANGUISH)[0];
        game.destroy_permanent(anguish);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, anguish));
        assert_eq!(permanent(&game, angel).damage, 3);
    }
}

/// "Creatures you control gain trample and get +X/+0 until end of turn, where
/// X is the number of artifacts you control." X and the creatures it reaches
/// are both settled as the trigger resolves (CR 608.2h, 611.2c).
#[test]
fn craterclaw_colossus_fixes_its_bonus_and_its_creatures_as_the_trigger_resolves() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let mox = settled(&mut game, cards::MOX_RUBY);
        game.put_onto_battlefield(PlayerId::Two, cards::MOX_RUBY)
            .unwrap();
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(&mut game, cards::CRATERCLAW_COLOSSUS, &[]);
        drain_pending(&mut game);
        let colossus = controlled(&game, cards::CRATERCLAW_COLOSSUS)[0];

        // The Colossus is an artifact too: your Mox and itself make two.
        let power = |game: &Game, id| game.power(permanent(game, id));
        assert_eq!(power(&game, bears), Some(4));
        assert_eq!(power(&game, colossus), Some(7));
        assert!(has_keyword(&game, bears, KeywordAbility::Trample));
        assert_eq!(power(&game, theirs), Some(2), "not a creature you control");

        // Losing an artifact afterwards takes nothing back, and a creature
        // that arrives afterwards was not there to be given anything.
        game.destroy_permanent(mox);
        let late = settled(&mut game, cards::GRIZZLY_BEARS);
        assert_eq!(power(&game, bears), Some(4));
        assert_eq!(power(&game, late), Some(2));
        assert!(!has_keyword(&game, late, KeywordAbility::Trample));
    }
}

/// "Search your library for a card, put it into your hand, shuffle, then
/// discard a card at random": the card just found is in hand to be lost.
#[test]
fn curse_marred_demon_can_discard_the_card_it_searched_for() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.players[0].library = game
            .build_zone(PlayerId::One, &[cards::FOREST, cards::LIGHTNING_BOLT])
            .unwrap();

        cast(&mut game, cards::CURSE_MARRED_DEMON, &[]);
        let search = run_to_decision(&mut game).expect("the search chooses its card");
        assert_eq!((search.minimum, search.maximum), (1, 1), "any card will do");
        let bolt = search
            .options
            .iter()
            .find(|option| option.label == "Lightning Bolt")
            .expect("every card in the library is offered")
            .id;
        game.apply(
            PlayerId::One,
            Action::ChooseDecision {
                decision: search.id,
                options: vec![bolt],
            },
        )
        .unwrap();
        drain_pending(&mut game);

        let player = &game.players[0];
        assert!(player.hand.is_empty(), "the only card in hand is discarded");
        assert_eq!(player.library.len(), 1);
        let graveyard = player.graveyard.iter().map(|card| card.definition);
        assert!(graveyard.eq([cards::LIGHTNING_BOLT]));
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

/// Face Yourself resolved at the opponent, who controls a 2/2, a 4/4 and a
/// land. Returns the tokens it made for player one.
fn facing_the_opponent(game: &mut Game) -> Vec<GameObjectId> {
    game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
        .unwrap();
    game.put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
        .unwrap();
    game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
        .unwrap();
    cast(game, cards::FACE_YOURSELF, &[Target::Player(PlayerId::Two)]);
    drain_pending(game);
    tokens(game)
        .iter()
        .map(|permanent| permanent.card.id)
        .collect()
}

/// "For each creature target player controls, create a token that's a copy of
/// that creature, except it has haste ...": that player's creatures, not
/// yours, and the copies are yours.
#[test]
fn face_yourself_copies_each_creature_the_target_player_controls() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::CRAW_WURM);

        let copies = facing_the_opponent(&mut game);

        let mut powers = Vec::new();
        for copy in &copies {
            let copy = permanent(&game, *copy);
            powers.push(game.power(copy));
            assert!(game.permanent_has_executable_keyword(copy, KeywordAbility::Haste));
        }
        powers.sort_unstable();
        assert_eq!(powers, [Some(2), Some(4)], "not your Wurm, not their land");
    }
}

/// "At the beginning of the end step, if you don't control a planeswalker,
/// sacrifice this creature."
#[test]
fn face_yourself_copies_are_sacrificed_at_the_end_step() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let copies = facing_the_opponent(&mut game);
        assert_eq!(copies.len(), 2);

        end_step_begins(&mut game);
        drain_pending(&mut game);

        assert!(tokens(&game).is_empty());
    }
}

/// The condition is the copies' controller's, and a planeswalker there as the
/// end step begins means the ability never triggers.
#[test]
fn face_yourself_copies_stay_while_you_control_a_planeswalker() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let copies = facing_the_opponent(&mut game);
        settled(&mut game, cards::JACE_MEMORY_ADEPT);

        end_step_begins(&mut game);
        assert!(game.pending_triggers.is_empty() && game.stack.is_empty());
        drain_pending(&mut game);

        assert_eq!(tokens(&game).len(), copies.len());
    }
}

/// The condition is checked again as each trigger resolves, and the
/// opponent's planeswalker is not yours.
#[test]
fn face_yourself_copies_stay_if_a_planeswalker_arrives_before_the_trigger_resolves() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let copies = facing_the_opponent(&mut game);
        game.put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
            .unwrap();

        end_step_begins(&mut game);
        assert!(!game.pending_triggers.is_empty() || !game.pending_decisions.is_empty());
        game.put_onto_battlefield(PlayerId::One, cards::JACE_MEMORY_ADEPT)
            .unwrap();
        drain_pending(&mut game);

        assert_eq!(tokens(&game).len(), copies.len());
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

/// Tether Technician cast with one card left in hand, stopped at its offer
/// to discard. Returns the opponent's 4/4.
fn tether_technician_offering(prepared_engine: bool) -> (Game, GameObjectId) {
    let mut game = setup(prepared_engine);
    let angel = game
        .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
        .unwrap();
    cast(&mut game, cards::TETHER_TECHNICIAN, &[]);
    hand(&mut game, cards::FOREST);
    run_to_decision(&mut game).expect("discarding is offered");
    (game, angel)
}

/// "You may discard a card. When you do, this creature deals 2 damage to any
/// target."
#[test]
fn tether_technician_deals_two_damage_when_you_discard() {
    for prepared_engine in [false, true] {
        let (mut game, angel) = tether_technician_offering(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Do it");
        while !offered(&game, PlayerId::One).contains(&angel) {
            let forest = offered(&game, PlayerId::One)[0];
            choose(&mut game, PlayerId::One, forest);
            run_to_decision(&mut game).expect("the reflexive trigger chooses its target");
        }
        choose(&mut game, PlayerId::One, angel);
        drain_pending(&mut game);

        assert!(game.players[0].hand.is_empty());
        assert_eq!(permanent(&game, angel).damage, 2);
    }
}

#[test]
fn tether_technician_that_keeps_its_cards_deals_no_damage() {
    for prepared_engine in [false, true] {
        let (mut game, angel) = tether_technician_offering(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Decline");
        assert!(run_to_decision(&mut game).is_none(), "no trigger follows");

        assert_eq!(game.players[0].hand.len(), 1);
        assert_eq!(permanent(&game, angel).damage, 0);
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
