//! Triggers that depend on more than their event: an intervening "if" asked
//! as the event happens and again as the trigger resolves, and a reflexive
//! "when you do" that waits on an earlier instruction.
use super::*;
use crate::card::Subtype;

/// "At the beginning of your end step, if you didn't cast a spell this turn,
/// put two +1/+1 counters on Edgar."
#[test]
fn edgar_moonlit_sovereign_grows_only_at_the_end_of_a_turn_you_cast_nothing() {
    for (cast_a_spell, counters) in [(false, 2), (true, 0)] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            let edgar = settled(&mut game, cards::EDGAR_MOONLIT_SOVEREIGN);
            if cast_a_spell {
                cast(&mut game, cards::MOX_RUBY, &[]);
                drain_pending(&mut game);
            }

            end_step_begins(&mut game);
            drain_pending(&mut game);

            assert_eq!(
                permanent(&game, edgar).counters(CounterKind::PlusOnePlusOne),
                counters
            );
        }
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

/// Lich's Relic cast with mana to spare, stopped at its offer to pay {2}.
/// Returns your creature, then the opponent's creature and planeswalker.
fn lichs_relic_offering(prepared_engine: bool) -> (Game, [GameObjectId; 3]) {
    let mut game = setup(prepared_engine);
    let yours = settled(&mut game, cards::GRIZZLY_BEARS);
    let bears = game
        .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
        .unwrap();
    let jace = game
        .put_onto_battlefield(PlayerId::Two, cards::JACE_MEMORY_ADEPT)
        .unwrap();
    game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
        .unwrap();
    cast(&mut game, cards::LICH_S_RELIC, &[]);
    run_to_decision(&mut game).expect("paying {2} is offered");
    (game, [yours, bears, jace])
}

/// "You may pay {2}. When you do, for each opponent, destroy up to one target
/// creature or planeswalker that player controls."
#[test]
fn lichs_relic_paid_for_destroys_a_creature_or_planeswalker_an_opponent_controls() {
    for prepared_engine in [false, true] {
        let (mut game, [yours, bears, jace]) = lichs_relic_offering(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Pay the cost");
        let decision = run_to_decision(&mut game).expect("the reflexive trigger chooses");
        assert_eq!((decision.minimum, decision.maximum), (0, 1), "up to one");
        assert_eq!(offered(&game, PlayerId::One), [bears, jace]);
        choose(&mut game, PlayerId::One, jace);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, jace));
        assert!(on_battlefield(&game, bears));
        assert!(on_battlefield(&game, yours));
    }
}

/// Lyra under player one at the end of `player`'s turn, player one having
/// drawn `drawn` cards during it.
fn lyra_tolarian_at_end_step_of(prepared_engine: bool, player: PlayerId, drawn: u16) -> Game {
    let mut game = setup(prepared_engine);
    stock_library(&mut game);
    settled(&mut game, cards::LYRA_TOLARIAN_ARCHANGEL);
    game.active_player = player;
    game.priority = player;
    game.draw_cards(PlayerId::One, drawn);
    end_step_begins(&mut game);
    drain_pending(&mut game);
    game
}

/// "At the beginning of each end step, if you've drawn three or more cards
/// this turn, create a 3/3 blue Angel creature token with flying."
#[test]
fn lyra_tolarian_archangel_makes_an_angel_at_any_end_step_after_three_draws() {
    for player in [PlayerId::One, PlayerId::Two] {
        for prepared_engine in [false, true] {
            let game = lyra_tolarian_at_end_step_of(prepared_engine, player, 3);

            let angels = tokens(&game);
            assert_eq!(angels.len(), 1);
            assert_eq!(
                (game.power(angels[0]), game.toughness(angels[0])),
                (Some(3), Some(3))
            );
            assert!(has_keyword(
                &game,
                angels[0].card.id,
                KeywordAbility::Flying
            ));
        }
    }
}

#[test]
fn lyra_tolarian_archangel_makes_nothing_after_only_two_draws() {
    for prepared_engine in [false, true] {
        let game = lyra_tolarian_at_end_step_of(prepared_engine, PlayerId::One, 2);

        assert!(tokens(&game).is_empty());
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

/// Vraska cast with `lands` lands of player one's and a Serra Angel of the
/// opponent's on the battlefield, her trigger aimed at the Angel if it is
/// there to aim.
fn vraska_the_cutting_glare_entering(prepared_engine: bool, lands: usize) -> (Game, GameObjectId) {
    let mut game = setup(prepared_engine);
    for _ in 0..lands {
        settled(&mut game, cards::FOREST);
    }
    let angel = game
        .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
        .unwrap();
    cast(&mut game, cards::VRASKA_THE_CUTTING_GLARE, &[]);
    if run_to_decision(&mut game).is_some() {
        choose(&mut game, PlayerId::One, angel);
    }
    drain_pending(&mut game);
    (game, angel)
}

/// "When Vraska enters, if you control six or more lands, destroy target
/// permanent an opponent controls. They create a Treasure token."
#[test]
fn vraska_the_cutting_glare_destroys_a_permanent_and_its_controller_gets_a_treasure() {
    for prepared_engine in [false, true] {
        let (game, angel) = vraska_the_cutting_glare_entering(prepared_engine, 6);

        assert!(!on_battlefield(&game, angel));
        assert!(tokens(&game).is_empty(), "not for you");
        let treasures = game
            .battlefield
            .iter()
            .filter(|permanent| {
                permanent.controller == PlayerId::Two
                    && permanent.card.definition.is_token()
                    && game
                        .effective_subtypes(permanent)
                        .contains(Subtype::from_name("Treasure").unwrap())
            })
            .count();
        assert_eq!(treasures, 1);
    }
}

/// "If you don't, the ability won't trigger at all."
#[test]
fn vraska_the_cutting_glare_does_nothing_with_five_lands() {
    for prepared_engine in [false, true] {
        let (game, angel) = vraska_the_cutting_glare_entering(prepared_engine, 5);

        assert!(on_battlefield(&game, angel));
        assert!(
            !game
                .battlefield
                .iter()
                .any(|permanent| permanent.card.definition.is_token())
        );
    }
}

/// Winter cast with a Grizzly Bears of player one's and two creatures of the
/// opponent's on the battlefield, as far as the offer to sacrifice.
fn winter_offering(prepared_engine: bool) -> (Game, GameObjectId, [GameObjectId; 2]) {
    let mut game = setup(prepared_engine);
    let bears = settled(&mut game, cards::GRIZZLY_BEARS);
    let theirs = [cards::GRIZZLY_BEARS, cards::SERRA_ANGEL].map(|definition| {
        game.put_onto_battlefield(PlayerId::Two, definition)
            .unwrap()
    });
    cast(&mut game, cards::WINTER_TORMENTED_LONER, &[]);
    run_to_decision(&mut game).expect("the sacrifice is offered");
    (game, bears, theirs)
}

/// "You may sacrifice a creature or planeswalker. When you do, each opponent
/// sacrifices a creature of their choice."
#[test]
fn winter_tormented_loner_has_each_opponent_sacrifice_a_creature_of_their_choice() {
    for prepared_engine in [false, true] {
        let (mut game, bears, [their_bears, their_angel]) = winter_offering(prepared_engine);

        choose(&mut game, PlayerId::One, bears);
        let decision = run_to_decision(&mut game).expect("the opponent chooses");
        assert_eq!(decision.player, PlayerId::Two);
        choose(&mut game, PlayerId::Two, their_bears);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, bears));
        assert!(!on_battlefield(&game, their_bears));
        assert!(on_battlefield(&game, their_angel));
        // The Bears in your graveyard: "+1/+0 for each creature and planeswalker card".
        let winter = controlled(&game, cards::WINTER_TORMENTED_LONER)[0];
        assert_eq!(game.power(permanent(&game, winter)), Some(1));
    }
}

/// Winter is a creature he may sacrifice himself, and the reflexive trigger
/// still asks the opponent once he is gone.
#[test]
fn winter_tormented_loner_may_sacrifice_himself() {
    for prepared_engine in [false, true] {
        let (mut game, bears, [their_bears, their_angel]) = winter_offering(prepared_engine);
        let winter = controlled(&game, cards::WINTER_TORMENTED_LONER)[0];

        choose(&mut game, PlayerId::One, winter);
        run_to_decision(&mut game).expect("the opponent chooses");
        choose(&mut game, PlayerId::Two, their_angel);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, winter));
        assert!(on_battlefield(&game, bears));
        assert!(on_battlefield(&game, their_bears));
        assert!(!on_battlefield(&game, their_angel));
    }
}
