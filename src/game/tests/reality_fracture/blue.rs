//! Reality Fracture's blue cards.
use super::*;
use crate::card::PermanentDesignationDef;

/// Seasoned Cryomancer enters with these two cards on top of an otherwise
/// empty hand, so they are what it draws and then discards. Stops at the
/// reflexive trigger's target choice, or with nothing waiting if there is
/// none, and returns the three opposing creatures.
fn cryomancer_discarding(game: &mut Game, drawn: [CardDefinitionId; 2]) -> [GameObjectId; 3] {
    let creatures = [(); 3].map(|()| {
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap()
    });
    game.players[0].library = game.build_zone(PlayerId::One, &drawn).unwrap();
    game.put_onto_battlefield(PlayerId::One, cards::SEASONED_CRYOMANCER)
        .unwrap();
    while let Some(decision) = run_to_decision(game) {
        let picks_a_creature = decision.options.iter().any(|option| {
            option
                .card
                .as_ref()
                .is_some_and(|(id, _)| creatures.contains(id))
        });
        if picks_a_creature {
            break;
        }
        choose_all_offered(game, PlayerId::One);
    }
    assert_eq!(game.players[0].graveyard.len(), 2, "both cards discarded");
    creatures
}

/// "When you discard one or more nonland cards this way, tap up to that many
/// target creatures and put a stun counter on each of them."
#[test]
fn seasoned_cryomancer_targets_up_to_as_many_creatures_as_nonland_cards_discarded() {
    for prepared_engine in [false, true] {
        for (drawn, nonland) in [
            ([cards::LIGHTNING_BOLT, cards::GRIZZLY_BEARS], 2),
            ([cards::LIGHTNING_BOLT, cards::FOREST], 1),
        ] {
            let mut game = setup(prepared_engine);
            let creatures = cryomancer_discarding(&mut game, drawn);

            let decision = game
                .observe(PlayerId::One)
                .decision
                .expect("the reflexive trigger chooses its targets");
            assert_eq!((decision.minimum, decision.maximum), (0, nonland));
            let options = decision
                .options
                .iter()
                .filter(|option| {
                    option
                        .card
                        .as_ref()
                        .is_some_and(|(id, _)| creatures[..nonland].contains(id))
                })
                .map(|option| option.id)
                .collect();
            game.apply(
                PlayerId::One,
                Action::ChooseDecision {
                    decision: decision.id,
                    options,
                },
            )
            .unwrap();
            drain_pending(&mut game);

            for (index, creature) in creatures.into_iter().enumerate() {
                let frozen = index < nonland;
                let creature = permanent(&game, creature);
                assert_eq!(creature.tapped, frozen);
                assert_eq!(creature.counters(CounterKind::Stun), u16::from(frozen));
            }
        }
    }
}

#[test]
fn seasoned_cryomancer_discarding_only_lands_freezes_nothing() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let creatures = cryomancer_discarding(&mut game, [cards::FOREST, cards::FOREST]);

        assert!(game.observe(PlayerId::One).decision.is_none());
        assert!(game.stack.is_empty(), "no reflexive trigger");
        for creature in creatures {
            assert!(!permanent(&game, creature).tapped);
        }
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

/// "When this creature dies, if it isn't a token, create a token that's a
/// copy of it." The copy is of the Sphinx as it last was on the battlefield,
/// and being a token it leaves nothing behind in turn.
#[test]
fn sphinx_of_false_conclusions_leaves_one_token_copy_which_leaves_none() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let sphinx = settled(&mut game, cards::SPHINX_OF_FALSE_CONCLUSIONS);

        cast(
            &mut game,
            cards::LIGHTNING_BOLT,
            &[Target::Permanent(sphinx)],
        );
        drain_pending(&mut game);

        let copies = tokens(&game);
        assert_eq!(copies.len(), 1);
        let copy = copies[0];
        assert_eq!(
            game.object_card_name(copy.card.id).as_deref(),
            Some("Sphinx of False Conclusions")
        );
        assert_eq!((game.power(copy), game.toughness(copy)), (Some(4), Some(2)));
        assert!(game.permanent_has_executable_keyword(copy, KeywordAbility::Flying));
        let copy = copy.card.id;

        cast(&mut game, cards::LIGHTNING_BOLT, &[Target::Permanent(copy)]);
        drain_pending(&mut game);

        assert!(tokens(&game).is_empty(), "the copy was a token");
    }
}

/// "−6: Draw three cards. Then put X +1/+1 counters on each creature you
/// control, where X is the number of cards in your hand." X is read after
/// the draw.
#[test]
fn the_theorist_counts_your_hand_after_its_three_draws() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let jace = settled(&mut game, cards::THE_THEORIST_JACE_BELEREN);
        permanent_mut(&mut game, jace).add_counters(CounterKind::Loyalty, 3);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        hand(&mut game, cards::FOREST);

        activate(&mut game, jace, "−6:");
        drain_pending(&mut game);

        assert_eq!(
            game.players[0].hand.len(),
            4,
            "the one held and three drawn"
        );
        let counters = |id| permanent(&game, id).counters(CounterKind::PlusOnePlusOne);
        assert_eq!(counters(bears), 4);
        assert_eq!(counters(theirs), 0);
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

/// A game in `player`'s upkeep past the first turn, both libraries stocked,
/// with The Theorist under player one.
fn theorist_in_upkeep_of(prepared_engine: bool, player: PlayerId) -> Game {
    let mut game = setup(prepared_engine);
    stock_library(&mut game);
    game.players[1].library = game
        .build_zone(PlayerId::Two, &[cards::FOREST; 10])
        .unwrap();
    settled(&mut game, cards::THE_THEORIST_JACE_BELEREN);
    game.turn = 5;
    game.active_player = player;
    game.priority = player;
    game.step = Step::Upkeep;
    game
}

/// "Your opponent will draw the card they normally draw in their draw step
/// before you draw a card from Jace's first ability."
#[test]
fn the_theorist_draws_for_you_in_an_opponents_draw_step() {
    for prepared_engine in [false, true] {
        let mut game = theorist_in_upkeep_of(prepared_engine, PlayerId::Two);

        game.advance_step();
        assert_eq!(game.step, Step::Draw);
        let hands = |game: &Game| [game.players[0].hand.len(), game.players[1].hand.len()];
        assert_eq!(hands(&game), [0, 1], "their draw comes first");
        drain_pending(&mut game);

        assert_eq!(hands(&game), [1, 1]);
    }
}

#[test]
fn the_theorist_draws_nothing_extra_in_your_own_draw_step() {
    for prepared_engine in [false, true] {
        let mut game = theorist_in_upkeep_of(prepared_engine, PlayerId::One);

        game.advance_step();
        drain_pending(&mut game);

        assert_eq!(game.players[0].hand.len(), 1, "only the turn's own draw");
    }
}
