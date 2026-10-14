//! What a spell or ability does once an object it reads has left the
//! battlefield or stopped being what it was: the value that object last had,
//! or nothing at all.
use super::*;

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

/// A Fblthp who has left before his trigger resolves is a new object where
/// he lies, so he stays there; the library is shuffled all the same.
#[test]
fn fblthp_impossibly_lost_stays_where_he_went_if_he_left_before_resolving() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        stock_library(&mut game);
        let fblthp = settled(&mut game, cards::FBLTHP_IMPOSSIBLY_LOST);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        connect_with(&mut game, bears);
        game.destroy_permanent(fblthp);
        drain_pending(&mut game);

        assert_eq!(game.players[0].hand.len(), 2);
        assert_eq!(game.players[0].library.len(), 8);
        assert_eq!(game.players[0].graveyard.len(), 1);
        assert_eq!(
            game.players[0].graveyard[0].definition,
            cards::FBLTHP_IMPOSSIBLY_LOST
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

/// "Graft Surgeon's second ability puts all counters that were on Graft
/// Surgeon onto the target creature, not just its +1/+1 counters."
#[test]
fn graft_surgeon_puts_every_kind_of_counter_it_died_with_on_its_target() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let surgeon = settled(&mut game, cards::GRAFT_SURGEON);
        assert_eq!(
            permanent(&game, surgeon).counters(CounterKind::PlusOnePlusOne),
            1,
            "it enters with one"
        );
        permanent_mut(&mut game, surgeon).add_counters(CounterKind::Stun, 2);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        let theirs = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(
            &mut game,
            cards::LIGHTNING_BOLT,
            &[Target::Permanent(surgeon)],
        );
        run_to_decision(&mut game).expect("the trigger chooses its target");
        assert_eq!(offered(&game, PlayerId::One), [bears], "one you control");
        choose(&mut game, PlayerId::One, bears);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, surgeon));
        let bears = permanent(&game, bears);
        assert_eq!(bears.counters(CounterKind::PlusOnePlusOne), 1);
        assert_eq!(bears.counters(CounterKind::Stun), 2);
        assert_eq!(
            permanent(&game, theirs).counters(CounterKind::PlusOnePlusOne),
            0
        );
    }
}

/// "If that land is a Mountain" asks about the land that entered, as it last
/// was if it has gone by the time the trigger resolves.
#[test]
fn koth_the_geomancer_adds_red_mana_for_a_mountain_that_has_already_left() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        settled(&mut game, cards::KOTH_THE_GEOMANCER);
        let mountain = game
            .put_onto_battlefield(PlayerId::One, cards::MOUNTAIN)
            .unwrap();
        game.destroy_permanent(mountain);

        drain_pending(&mut game);

        assert_eq!(game.players[1].life, 19);
        assert_eq!(game.players[0].mana_pool.amount(ManaColor::Red), 1);
    }
}

/// "If it wasn't attacking, its controller draws a card" asks about the
/// creature as it last was on the battlefield, and names whoever controlled
/// it there.
#[test]
fn prophesied_end_draws_for_the_controller_of_a_creature_that_was_not_attacking() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = game
            .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();

        cast(
            &mut game,
            cards::PROPHESIED_END,
            &[Target::Permanent(bears)],
        );
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, bears));
        assert_eq!(game.players[1].hand.len(), 1, "its controller draws");
        assert!(game.players[0].hand.is_empty(), "and you do not");
    }
}

#[test]
fn prophesied_end_draws_nothing_for_a_creature_that_was_attacking() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);
        attack_with(&mut game, bears);

        cast(
            &mut game,
            cards::PROPHESIED_END,
            &[Target::Permanent(bears)],
        );
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, bears));
        assert!(game.players[0].hand.is_empty(), "it was attacking");
    }
}

/// The draw does not depend on the destruction: an indestructible creature
/// that wasn't attacking still has its controller draw.
#[test]
fn prophesied_end_draws_even_when_the_creature_is_not_destroyed() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let tajic = game
            .put_onto_battlefield(PlayerId::Two, cards::TAJIC_BLADE_OF_THE_LEGION)
            .unwrap();

        cast(
            &mut game,
            cards::PROPHESIED_END,
            &[Target::Permanent(tajic)],
        );
        drain_pending(&mut game);

        assert!(on_battlefield(&game, tajic), "indestructible");
        assert_eq!(game.players[1].hand.len(), 1);
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
