//! Reality Fracture's green cards.
use super::*;
use crate::card::Subtype;

/// Budding Insurgent sacrificed to destroy the opponent's permanent made from
/// `definition`. Returns that permanent and how many cards you drew.
fn budding_insurgent_destroying(
    game: &mut Game,
    definition: CardDefinitionId,
) -> (GameObjectId, usize) {
    stock_library(game);
    let insurgent = settled(game, cards::BUDDING_INSURGENT);
    let target = game
        .put_onto_battlefield(PlayerId::Two, definition)
        .unwrap();
    let hand = game.players[0].hand.len();
    activate_at(
        game,
        insurgent,
        "Sacrifice this creature",
        &[Target::Permanent(target)],
    );
    drain_pending(game);
    assert!(!on_battlefield(game, insurgent), "sacrificed to pay for it");
    (target, game.players[0].hand.len() - hand)
}

/// "If that permanent was a legendary enchantment, draw a card": both words,
/// read off the permanent that was destroyed.
#[test]
fn budding_insurgent_draws_only_for_a_legendary_enchantment() {
    for prepared_engine in [false, true] {
        for (definition, drawn) in [
            (cards::GUILD_ARTISAN, 1),
            (cards::CRUSADE, 0),
            (cards::MOX_OPAL, 0),
        ] {
            let mut game = setup(prepared_engine);

            let (target, cards) = budding_insurgent_destroying(&mut game, definition);

            assert!(!on_battlefield(&game, target));
            assert_eq!(cards, drawn);
        }
    }
}

/// The draw asks what the permanent was, not whether it was destroyed.
#[test]
fn budding_insurgent_draws_for_a_legendary_enchantment_it_cannot_destroy() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);

        let (god, cards) =
            budding_insurgent_destroying(&mut game, cards::PURPHOROS_GOD_OF_THE_FORGE);

        assert!(on_battlefield(&game, god), "indestructible");
        assert_eq!(cards, 1);
    }
}

static PLANESWALKER: TokenCharacteristics =
    TokenCharacteristics::new(CardTypeSet::single(CardType::Planeswalker), &[], &[], None);

/// A planeswalker of player one's holding exactly `loyalty`.
fn planeswalker_with(game: &mut Game, loyalty: u16) -> GameObjectId {
    let planeswalker = game.create_token_from(PlayerId::One, PLANESWALKER, None);
    permanent_mut(game, planeswalker).add_counters(CounterKind::Loyalty, loyalty);
    planeswalker
}

/// "Target planeswalker you control deals damage equal to its loyalty to
/// target creature or planeswalker an opponent controls."
#[test]
fn compel_brutality_has_a_planeswalker_deal_damage_equal_to_its_loyalty() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let planeswalker = planeswalker_with(&mut game, 3);
        let wurm = game
            .put_onto_battlefield(PlayerId::Two, cards::CRAW_WURM)
            .unwrap();

        cast(
            &mut game,
            cards::COMPEL_BRUTALITY,
            &[Target::Permanent(planeswalker), Target::Permanent(wurm)],
        );
        drain_pending(&mut game);

        assert_eq!(permanent(&game, wurm).damage, 3);
        assert_eq!(
            permanent(&game, planeswalker).counters(CounterKind::Loyalty),
            3,
            "dealing the damage costs it nothing"
        );
    }
}

/// "If that creature or planeswalker is no longer on the battlefield at the
/// time the spell resolves, no damage is dealt."
#[test]
fn compel_brutality_deals_no_damage_once_its_planeswalker_has_left() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let planeswalker = planeswalker_with(&mut game, 3);
        let wurm = game
            .put_onto_battlefield(PlayerId::Two, cards::CRAW_WURM)
            .unwrap();

        cast(
            &mut game,
            cards::COMPEL_BRUTALITY,
            &[Target::Permanent(planeswalker), Target::Permanent(wurm)],
        );
        game.destroy_permanent(planeswalker);
        drain_pending(&mut game);

        assert_eq!(permanent(&game, wurm).damage, 0);
    }
}

/// "Target creature you control deals damage equal to its power ...": the
/// first mode takes a creature and the second a planeswalker, never the
/// other's.
#[test]
fn compel_brutality_has_a_creature_deal_damage_equal_to_its_power() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let planeswalker = planeswalker_with(&mut game, 3);
        let wurm = game
            .put_onto_battlefield(PlayerId::Two, cards::CRAW_WURM)
            .unwrap();
        let compel = hand(&mut game, cards::COMPEL_BRUTALITY);
        game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 2);

        assert_eq!(
            castable_targets(&game, compel),
            [
                vec![Target::Permanent(bears), Target::Permanent(wurm)],
                vec![Target::Permanent(planeswalker), Target::Permanent(wurm)],
            ]
        );
        let action = casts(&game, compel).remove(0);
        game.apply(PlayerId::One, action).unwrap();
        drain_pending(&mut game);

        assert_eq!(permanent(&game, wurm).damage, 2);
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

/// "... loses all abilities until end of turn. Target creature you control
/// deals damage equal to its power to that permanent."
#[test]
fn flourishing_grapple_strips_abilities_and_has_your_creature_deal_its_power() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();

        cast(
            &mut game,
            cards::FLOURISHING_GRAPPLE,
            &[Target::Permanent(angel), Target::Permanent(bears)],
        );
        drain_pending(&mut game);

        let angel = permanent(&game, angel);
        assert!(!game.permanent_has_executable_keyword(angel, KeywordAbility::Flying));
        assert!(!game.permanent_has_executable_keyword(angel, KeywordAbility::Vigilance));
        assert_eq!(angel.damage, 2);
        assert_eq!(permanent(&game, bears).damage, 0, "it is not a fight");
    }
}

/// The two sentences have a target each. With your creature gone the first
/// still happens and nothing deals the damage.
#[test]
fn flourishing_grapple_still_strips_abilities_when_your_creature_has_left() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let angel = game
            .put_onto_battlefield(PlayerId::Two, cards::SERRA_ANGEL)
            .unwrap();

        cast(
            &mut game,
            cards::FLOURISHING_GRAPPLE,
            &[Target::Permanent(angel), Target::Permanent(bears)],
        );
        game.destroy_permanent(bears);
        drain_pending(&mut game);

        let angel = permanent(&game, angel);
        assert!(!game.permanent_has_executable_keyword(angel, KeywordAbility::Flying));
        assert_eq!(angel.damage, 0);
    }
}

const CHARGE: CounterKind = CounterKind::named("charge");

/// Gardenize with three charge counters as `active`'s first main phase
/// begins, its trigger waiting if there is one.
fn gardenize_at_first_main_phase(game: &mut Game, active: PlayerId) -> GameObjectId {
    let gardenize = settled(game, cards::GARDENIZE);
    permanent_mut(game, gardenize).add_counters(CHARGE, 3);
    game.active_player = active;
    game.priority = active;
    game.step = Step::Draw;
    game.advance_step();
    assert_eq!(game.step, Step::PrecombatMain);
    gardenize
}

/// "At the beginning of your first main phase, add {G} for each charge
/// counter on this enchantment."
#[test]
fn gardenize_adds_green_mana_for_each_charge_counter_on_your_turn_only() {
    for prepared_engine in [false, true] {
        for (active, mana) in [(PlayerId::One, 3), (PlayerId::Two, 0)] {
            let mut game = setup(prepared_engine);

            gardenize_at_first_main_phase(&mut game, active);
            drain_pending(&mut game);

            assert_eq!(game.players[0].mana_pool.amount(ManaColor::Green), mana);
            assert_eq!(game.players[0].mana_pool.total(), mana);
        }
    }
}

/// "If Gardenize is no longer on the battlefield at the time its last ability
/// resolves, use the number of counters it had on it the last time it was on
/// the battlefield to determine the amount of {G} to add."
#[test]
fn gardenize_that_has_left_adds_mana_for_the_counters_it_last_had() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);

        let gardenize = gardenize_at_first_main_phase(&mut game, PlayerId::One);
        game.destroy_permanent(gardenize);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, gardenize));
        assert_eq!(game.players[0].mana_pool.amount(ManaColor::Green), 3);
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

/// The colors of mana player one could tap `source` for.
fn mana_colors(game: &Game, source: GameObjectId) -> Vec<ManaColor> {
    game.legal_actions(PlayerId::One)
        .into_iter()
        .filter_map(|action| match action {
            Action::ActivateManaAbility {
                source: tapped,
                color,
                ..
            } if tapped == source => Some(color),
            _ => None,
        })
        .collect()
}

/// "Create a Heartwood token. (It's a red and green artifact with '{T}: Add
/// {R} or {G}.')"
#[test]
fn hungering_puppetbeast_makes_a_heartwood_that_taps_for_red_or_green() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);

        cast(&mut game, cards::HUNGERING_PUPPETBEAST, &[]);
        drain_pending(&mut game);

        let made = tokens(&game);
        let [heartwood] = made[..] else {
            panic!("one token is made");
        };
        assert_eq!(
            game.permanent_types(heartwood),
            Some(CardTypeSet::single(CardType::Artifact))
        );
        assert!(
            game.effective_subtypes(heartwood)
                .contains(Subtype::from_name("Heartwood").unwrap())
        );
        assert_eq!(colors(&game, heartwood), [ManaColor::Red, ManaColor::Green]);
        assert_eq!(
            mana_colors(&game, heartwood.card.id),
            [ManaColor::Red, ManaColor::Green]
        );
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

/// "Equipped creature gets +2/+0 and has 'Whenever this creature attacks, it
/// gains your choice of trample or deathtouch until end of turn.'"
#[test]
fn hunters_axe_gives_the_equipped_creature_its_controllers_choice_as_it_attacks() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let axe = settled(&mut game, cards::HUNTER_S_AXE);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let other = settled(&mut game, cards::GRIZZLY_BEARS);
        permanent_mut(&mut game, axe).attached_to = Some(bears);
        assert_eq!(game.power(permanent(&game, bears)), Some(4));

        declare_attack(&mut game, bears);
        run_to_decision(&mut game).expect("the keyword is chosen as the trigger resolves");
        choose_decision_by_label(&mut game, PlayerId::One, "Deathtouch");
        drain_pending(&mut game);

        let keyword =
            |id, keyword| game.permanent_has_executable_keyword(permanent(&game, id), keyword);
        assert!(keyword(bears, KeywordAbility::Deathtouch));
        assert!(!keyword(bears, KeywordAbility::Trample));
        assert!(!keyword(other, KeywordAbility::Deathtouch));
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

/// "Enchanted permanent is a Construct creature with base power and toughness
/// 5/5 in addition to its other types."
#[test]
fn puppet_crafting_makes_its_permanent_a_five_five_construct_that_keeps_its_types() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let mox = settled(&mut game, cards::MOX_RUBY);

        cast(&mut game, cards::PUPPET_CRAFTING, &[Target::Permanent(mox)]);
        drain_pending(&mut game);

        let mox = permanent(&game, mox);
        let types = game.permanent_types(mox).expect("it has types");
        assert!(types.contains(CardType::Creature) && types.contains(CardType::Artifact));
        assert!(
            game.effective_subtypes(mox)
                .contains(Subtype::from_name("Construct").unwrap())
        );
        assert_eq!((game.power(mox), game.toughness(mox)), (Some(5), Some(5)));
        assert_eq!(mana_colors(&game, mox.card.id), [ManaColor::Red]);
    }
}

/// Something Worth Saving cast over a library whose top four cards are two
/// instants, a creature and a land, stopped at its choice among them.
fn something_worth_saving_choosing(prepared_engine: bool) -> (Game, DecisionObservation) {
    let mut game = setup(prepared_engine);
    game.players[0].library = game
        .build_zone(
            PlayerId::One,
            &[
                cards::GRIZZLY_BEARS,
                cards::LIGHTNING_BOLT,
                cards::FOREST,
                cards::GRIZZLY_BEARS,
                cards::LIGHTNING_BOLT,
            ],
        )
        .unwrap();
    cast(&mut game, cards::SOMETHING_WORTH_SAVING, &[]);
    let choice = run_to_decision(&mut game).expect("a milled permanent card may be taken");
    (game, choice)
}

/// "Mill four cards. You may put a permanent card from among them into your
/// hand. You gain 1 life."
#[test]
fn something_worth_saving_returns_one_milled_permanent_card() {
    for prepared_engine in [false, true] {
        let (mut game, choice) = something_worth_saving_choosing(prepared_engine);
        let life = game.players[0].life;
        assert_eq!(choice.options.len(), 2, "the land and the creature");
        assert_eq!((choice.minimum, choice.maximum), (0, 1));

        let taken = offered(&game, PlayerId::One)[0];
        let definition = game.players[0]
            .graveyard
            .iter()
            .find(|card| card.id == taken)
            .expect("it was milled")
            .definition;
        choose(&mut game, PlayerId::One, taken);
        drain_pending(&mut game);

        let player = &game.players[0];
        assert_eq!(player.library.len(), 1, "the fifth card stays");
        let hand = player.hand.iter().map(|card| card.definition);
        assert!(hand.eq([definition]));
        assert_eq!(
            player.graveyard.len(),
            4,
            "three milled cards and the spell"
        );
        assert_eq!(player.life, life + 1);
    }
}

#[test]
fn something_worth_saving_may_leave_every_card_and_still_gains_life() {
    for prepared_engine in [false, true] {
        let (mut game, choice) = something_worth_saving_choosing(prepared_engine);
        let life = game.players[0].life;

        game.apply(
            PlayerId::One,
            Action::ChooseDecision {
                decision: choice.id,
                options: Vec::new(),
            },
        )
        .unwrap();
        drain_pending(&mut game);

        assert!(game.players[0].hand.is_empty());
        assert_eq!(game.players[0].graveyard.len(), 5);
        assert_eq!(game.players[0].life, life + 1);
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

/// `active`'s upkeep begins and whatever it triggers resolves.
fn take_an_upkeep(game: &mut Game, active: PlayerId) {
    game.active_player = active;
    game.priority = active;
    game.step = Step::Upkeep;
    game.handle_upkeep_triggers();
    drain_pending(game);
}

/// "At the beginning of each player's upkeep, you create a 3/3 green Forest
/// Tentacle land creature token."
#[test]
fn verdant_kraken_makes_you_a_forest_tentacle_at_each_players_upkeep() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::VERDANT_KRAKEN);

        take_an_upkeep(&mut game, PlayerId::One);
        assert_eq!(tokens(&game).len(), 1);
        take_an_upkeep(&mut game, PlayerId::Two);
        let made = tokens(&game);
        assert_eq!(made.len(), 2, "the opponent's upkeep makes you one too");

        let tentacle = made[0];
        let types = game.permanent_types(tentacle).expect("it has types");
        assert!(types.contains(CardType::Land) && types.contains(CardType::Creature));
        let subtypes = game.effective_subtypes(tentacle);
        assert!(subtypes.contains(Subtype::from_name("Forest").unwrap()));
        assert!(subtypes.contains(Subtype::from_name("Tentacle").unwrap()));
        assert_eq!(
            (game.power(tentacle), game.toughness(tentacle)),
            (Some(3), Some(3))
        );
        assert_eq!(colors(&game, tentacle), [ManaColor::Green]);
    }
}

/// "(It has '{T}: Add {G}.' It's affected by summoning sickness until your
/// next turn.)"
#[test]
fn verdant_krakens_forest_tentacle_taps_for_green_once_your_next_turn_has_begun() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::VERDANT_KRAKEN);
        take_an_upkeep(&mut game, PlayerId::Two);
        let tentacle = tokens(&game)[0].card.id;
        game.active_player = PlayerId::One;
        game.priority = PlayerId::One;
        game.step = Step::PrecombatMain;

        assert!(
            mana_colors(&game, tentacle).is_empty(),
            "a creature that just arrived"
        );

        game.turns_started[0] += 1;
        assert_eq!(mana_colors(&game, tentacle), [ManaColor::Green]);
    }
}
