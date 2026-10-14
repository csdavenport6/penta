//! Which events make an ability trigger: whose spell, permanent, or turn it
//! has to be, how many times one event or one turn triggers it, and what the
//! trigger reads from the event that caused it.
use super::*;

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

/// "When one or more of your opponents are dealt combat damage during your
/// turn": combat damage dealt to you, on an opponent's turn, is neither.
#[test]
fn fblthp_impossibly_lost_ignores_combat_damage_dealt_to_you_on_another_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let fblthp = settled(&mut game, cards::FBLTHP_IMPOSSIBLY_LOST);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        permanent_mut(&mut game, theirs).entered_controller_turn = 0;
        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        game.step = Step::DeclareAttackers;

        game.apply(
            PlayerId::Two,
            Action::DeclareAttacker {
                attacker: theirs,
                defender: AttackDefender::Player(PlayerId::One),
            },
        )
        .unwrap();
        game.finish_declaring_attackers();
        game.finish_declaring_blockers();
        game.deal_combat_damage();
        drain_pending(&mut game);

        assert_eq!(game.players[0].life, 18);
        assert!(game.players[0].hand.is_empty());
        assert!(on_battlefield(&game, fblthp));
    }
}

/// "During your turn" is its own condition: an opponent dealt combat damage
/// while it is somebody else's turn does not trigger Fblthp.
#[test]
fn fblthp_impossibly_lost_ignores_an_opponents_combat_damage_outside_your_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let fblthp = settled(&mut game, cards::FBLTHP_IMPOSSIBLY_LOST);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        // No rule has your creature attacking on another player's turn, so the
        // turn changes hands between the declaration and the damage to ask
        // the condition on its own.
        attack_with(&mut game, bears);
        game.finish_declaring_blockers();
        game.active_player = PlayerId::Two;

        game.deal_combat_damage();
        drain_pending(&mut game);

        assert_eq!(game.players[1].life, 18, "the damage was dealt");
        assert!(game.players[0].hand.is_empty());
        assert!(on_battlefield(&game, fblthp));
    }
}

/// "Whenever this creature is dealt damage, you may search your library for
/// up to that many land cards, put them onto the battlefield tapped, then
/// shuffle": the damage of the event that triggered it sizes the search.
#[test]
fn hexhaven_invigorator_fetches_up_to_as_many_lands_as_the_damage_it_was_dealt() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let invigorator = settled(&mut game, cards::HEXHAVEN_INVIGORATOR);
        game.players[0].library = game
            .build_zone(
                PlayerId::One,
                &[
                    cards::FOREST,
                    cards::GRIZZLY_BEARS,
                    cards::FOREST,
                    cards::MOUNTAIN,
                    cards::FOREST,
                    cards::FOREST,
                ],
            )
            .unwrap();

        cast(
            &mut game,
            cards::LIGHTNING_BOLT,
            &[Target::Permanent(invigorator)],
        );
        run_to_decision(&mut game).expect("searching is offered");
        choose_decision_by_label(&mut game, PlayerId::One, "Do it");
        let search = run_to_decision(&mut game).expect("the search chooses its cards");
        assert_eq!(search.options.len(), 5, "every land card, not the creature");
        assert_eq!(search.maximum, 3, "the Bolt dealt 3 damage");
        game.apply(
            PlayerId::One,
            Action::ChooseDecision {
                decision: search.id,
                options: search.options[..3].iter().map(|option| option.id).collect(),
            },
        )
        .unwrap();
        drain_pending(&mut game);

        let lands = game
            .battlefield
            .iter()
            .filter(|permanent| {
                game.permanent_types(permanent)
                    .is_some_and(|types| types.contains(CardType::Land))
            })
            .collect::<Vec<_>>();
        assert_eq!(lands.len(), 3);
        assert!(lands.iter().all(|land| land.tapped));
        assert_eq!(game.players[0].library.len(), 3);
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

/// "Artifacts and creatures entering the battlefield don't cause abilities to
/// trigger": "if an artifact land or creature land you control enters, that
/// ability won't trigger. If a nonartifact, noncreature land you control
/// enters, the ability will trigger."
#[test]
fn karn_argent_defender_silences_only_what_an_artifact_or_creature_entering_triggers() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::KARN_ARGENT_DEFENDER);
        settled(&mut game, cards::KOTH_OF_THE_HOMESTEAD);

        settled(&mut game, cards::ANCIENT_DEN);
        assert_eq!(game.players[0].life, 20, "an artifact land");

        settled(&mut game, cards::FOREST);
        assert_eq!(game.players[0].life, 21);
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

/// "Landfall — Whenever a land you control enters, Koth deals 1 damage to
/// each opponent. If that land is a Mountain, add {R}."
#[test]
fn koth_the_geomancer_adds_red_mana_only_for_a_mountain() {
    for (land, red) in [
        (cards::FOREST, 0),
        (cards::MOUNTAIN, 1),
        (cards::BADLANDS, 1),
    ] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            settled(&mut game, cards::KOTH_THE_GEOMANCER);

            settled(&mut game, land);

            assert_eq!(game.players[1].life, 19);
            assert_eq!(game.players[0].life, 20);
            assert_eq!(game.players[0].mana_pool.amount(ManaColor::Red), red);
        }
    }
}

/// "Until end of turn, whenever Lyra deals combat damage to a player, draw
/// two cards": nothing without the activation, and two for each time it was
/// activated.
#[test]
fn lyra_tolarian_archangel_draws_two_for_her_combat_damage_once_activated() {
    for (activations, drawn) in [(0, 0), (1, 2), (2, 4)] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            stock_library(&mut game);
            let lyra = settled(&mut game, cards::LYRA_TOLARIAN_ARCHANGEL);
            for _ in 0..activations {
                game.add_unrestricted_mana(PlayerId::One, ManaColor::Blue, 5);
                activate(&mut game, lyra, "{3}{U}{U}:");
                drain_pending(&mut game);
            }

            connect_with(&mut game, lyra);
            drain_pending(&mut game);

            assert_eq!(game.players[1].life, 17);
            assert_eq!(game.players[0].hand.len(), drawn);
        }
    }
}

/// "Whenever Mabel or another creature you control enters, put a +1/+1
/// counter on target creature that entered this turn."
#[test]
fn mabel_valley_hero_puts_a_counter_on_a_creature_that_entered_this_turn() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let old = settled(&mut game, cards::GRIZZLY_BEARS);
        permanent_mut(&mut game, old).entered_turn = 0;

        cast(&mut game, cards::MABEL_VALLEY_HERO, &[]);
        run_to_decision(&mut game).expect("Mabel's own arrival triggers");
        let mabel = controlled(&game, cards::MABEL_VALLEY_HERO)[0];
        assert_eq!(offered(&game, PlayerId::One), [mabel], "not the old Bears");
        choose(&mut game, PlayerId::One, mabel);
        drain_pending(&mut game);

        cast(&mut game, cards::HILL_GIANT, &[]);
        run_to_decision(&mut game).expect("another creature of yours triggers");
        let giant = controlled(&game, cards::HILL_GIANT)[0];
        assert_eq!(offered(&game, PlayerId::One), [mabel, giant]);
        choose(&mut game, PlayerId::One, mabel);
        drain_pending(&mut game);

        assert_eq!(
            permanent(&game, mabel).counters(CounterKind::PlusOnePlusOne),
            2
        );
    }
}

/// An opponent's creature entering does not trigger Mabel, though it is a
/// creature that entered this turn for a trigger that does happen.
#[test]
fn mabel_valley_hero_ignores_an_opponents_creature_entering() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::MABEL_VALLEY_HERO);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        assert!(run_to_decision(&mut game).is_none());
    }
}

/// Pompous Battlemage's Improvised Act on the stack, cast with `hand_size`
/// cards in hand. Returns the Battlemage.
fn improvised_act_cast(prepared_engine: bool, hand_size: usize) -> (Game, GameObjectId) {
    let mut game = setup(prepared_engine);
    stock_library(&mut game);
    let (battlemage, act) =
        prepared_permanent(&mut game, cards::POMPOUS_BATTLEMAGE, "Improvised Act");
    for _ in 0..hand_size {
        hand(&mut game, cards::MOUNTAIN);
    }
    cast_copy(&mut game, act, &[]);
    (game, battlemage)
}

/// The copy of a prepare spell is cast, so the creature it came from sees a
/// noncreature spell cast like any other.
#[test]
fn pompous_battlemage_grows_from_casting_its_own_prepare_spell() {
    for prepared_engine in [false, true] {
        let (mut game, battlemage) = improvised_act_cast(prepared_engine, 0);
        drain_pending(&mut game);

        let battlemage = permanent(&game, battlemage);
        assert_eq!(
            (game.power(battlemage), game.toughness(battlemage)),
            (Some(2), Some(2)),
            "prowess"
        );
    }
}

/// "Prowess, prowess": two instances, each its own trigger.
#[test]
fn ruric_thar_biomagus_gets_plus_two_plus_two_for_a_noncreature_spell() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let ruric = settled(&mut game, cards::RURIC_THAR_BIOMAGUS);

        cast(&mut game, cards::MOX_RUBY, &[]);
        drain_pending(&mut game);

        let ruric = permanent(&game, ruric);
        assert_eq!(
            (game.power(ruric), game.toughness(ruric)),
            (Some(6), Some(8))
        );
    }
}

/// "Whenever Ruric Thar becomes the target of a spell or ability an opponent
/// controls, draw a card": your own spell aimed at him draws nothing.
#[test]
fn ruric_thar_biomagus_draws_when_an_opponent_targets_him() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let ruric = settled(&mut game, cards::RURIC_THAR_BIOMAGUS);

        cast(&mut game, cards::GIANT_GROWTH, &[Target::Permanent(ruric)]);
        drain_pending(&mut game);
        assert!(game.players[0].hand.is_empty(), "your own spell");

        game.active_player = PlayerId::Two;
        game.priority = PlayerId::Two;
        let bolt = in_hand_of(&mut game, PlayerId::Two, cards::LIGHTNING_BOLT);
        game.add_unrestricted_mana(PlayerId::Two, ManaColor::Red, 1);
        let action = game
            .legal_actions(PlayerId::Two)
            .into_iter()
            .find(|action| {
                matches!(action, Action::CastSpell { card, choices, .. }
                    if *card == bolt && choices.iter_targets().eq(&[Target::Permanent(ruric)]))
            })
            .expect("the opponent can aim it at Ruric Thar");
        game.apply(PlayerId::Two, action).unwrap();
        drain_pending(&mut game);

        assert_eq!(game.players[0].hand.len(), 1);
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
            let id = in_hand_of(&mut game, PlayerId::Two, definition);
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

/// "When Tinybones enters, each opponent discards a card. Whenever a player
/// discards one or more cards, Tinybones deals 1 damage to each opponent":
/// the discard his own arrival causes is one he sees.
#[test]
fn tinybones_pocket_nuisance_deals_damage_for_the_discard_his_arrival_causes() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        in_hand_of(&mut game, PlayerId::Two, cards::FOREST);

        cast(&mut game, cards::TINYBONES_POCKET_NUISANCE, &[]);
        drain_pending(&mut game);

        assert!(game.players[1].hand.is_empty());
        assert_eq!(game.players[1].life, 19);
    }
}
