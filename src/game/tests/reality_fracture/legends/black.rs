//! Reality Fracture's black legends.
use super::*;

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

#[test]
fn liliana_the_repentant_exhausts_only_as_a_sorcery() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let liliana = settled(&mut game, cards::LILIANA_THE_REPENTANT);
        put_in_graveyard(&mut game, &[cards::GRIZZLY_BEARS]);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Black, 6);
        assert_eq!(activations(&game, liliana, "Exhaust").len(), 1);

        game.step = Step::BeginningOfCombat;
        assert!(activations(&game, liliana, "Exhaust").is_empty());
    }
}

/// "Target permanent you control gains deathtouch until end of turn. ... Put
/// a loyalty counter on it if it's a planeswalker."
#[test]
fn teyo_diamondblade_mage_gives_a_planeswalker_deathtouch_and_a_loyalty_counter() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let jace = settled(&mut game, cards::JACE_MEMORY_ADEPT);
        let before = permanent(&game, jace).counters(CounterKind::Loyalty);

        teyo_entering_at(&mut game, cards::TEYO_DIAMONDBLADE_MAGE, jace);

        assert!(has_keyword(&game, jace, KeywordAbility::Deathtouch));
        let jace = permanent(&game, jace);
        assert_eq!(jace.counters(CounterKind::Loyalty), before + 1);
        assert_eq!(jace.counters(CounterKind::PlusOnePlusOne), 0);
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

/// "One or more cards": one trigger for a discard of two, whoever discarded
/// them, and the damage is always to the opponents.
#[test]
fn tinybones_pocket_nuisance_triggers_once_for_each_players_discard_of_any_size() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::TINYBONES_POCKET_NUISANCE);
        hand(&mut game, cards::FOREST);
        hand(&mut game, cards::FOREST);

        // You discard two, to a spell aimed at yourself.
        cast(&mut game, cards::MIND_ROT, &[Target::Player(PlayerId::One)]);
        drain_pending(&mut game);

        assert!(game.players[0].hand.is_empty());
        assert_eq!(game.players[0].life, 20);
        assert_eq!(game.players[1].life, 19);
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

#[test]
fn winter_tormented_loner_asks_nothing_of_the_opponent_when_you_decline() {
    for prepared_engine in [false, true] {
        let (mut game, bears, theirs) = winter_offering(prepared_engine);

        choose_decision_by_label(&mut game, PlayerId::One, "Decline");

        assert!(run_to_decision(&mut game).is_none());
        for permanent in [bears, theirs[0], theirs[1]] {
            assert!(on_battlefield(&game, permanent));
        }
    }
}

/// "Winter gets +1/+0 for each creature and planeswalker card in your
/// graveyard": cards of yours, of either type, and nothing else.
#[test]
fn winter_tormented_loner_counts_creature_and_planeswalker_cards_in_your_graveyard() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let winter = settled(&mut game, cards::WINTER_TORMENTED_LONER);
        put_in_graveyard(
            &mut game,
            &[
                cards::GRIZZLY_BEARS,
                cards::JACE_MEMORY_ADEPT,
                cards::FOREST,
                cards::LIGHTNING_BOLT,
            ],
        );
        let theirs = game
            .build_zone(PlayerId::Two, &[cards::GRIZZLY_BEARS])
            .unwrap();
        game.players[1].graveyard.extend(theirs);

        let winter = permanent(&game, winter);
        assert_eq!(
            (game.power(winter), game.toughness(winter)),
            (Some(2), Some(3))
        );
    }
}
