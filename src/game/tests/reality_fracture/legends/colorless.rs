//! Reality Fracture's colorless legends.
use super::*;

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

/// A creature's own enters trigger is one of the abilities its arrival would
/// cause to trigger, whoever controls it.
#[test]
fn karn_argent_defender_silences_a_creatures_own_enters_trigger() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.put_onto_battlefield(PlayerId::Two, cards::KARN_ARGENT_DEFENDER)
            .unwrap();
        in_hand_of(&mut game, PlayerId::Two, cards::FOREST);

        cast(&mut game, cards::TINYBONES_POCKET_NUISANCE, &[]);
        drain_pending(&mut game);

        assert_eq!(game.players[1].hand.len(), 1, "nobody discards");
    }
}

/// "Whenever you cast an artifact or creature spell, untap Traxos."
#[test]
fn traxos_scourge_eternal_untaps_for_an_artifact_or_creature_spell_you_cast() {
    for (spell, untapped) in [
        (cards::OPT, false),
        (cards::MOX_RUBY, true),
        (cards::GRIZZLY_BEARS, true),
    ] {
        for prepared_engine in [false, true] {
            let mut game = setup(prepared_engine);
            stock_library(&mut game);
            let traxos = settled(&mut game, cards::TRAXOS_SCOURGE_ETERNAL);
            permanent_mut(&mut game, traxos).tapped = true;

            cast(&mut game, spell, &[]);
            drain_pending(&mut game);

            assert_eq!(!permanent(&game, traxos).tapped, untapped);
        }
    }
}

/// "Traxos doesn't untap during your untap step."
#[test]
fn traxos_scourge_eternal_stays_tapped_through_your_untap_step() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        let traxos = settled(&mut game, cards::TRAXOS_SCOURGE_ETERNAL);
        let bears = settled(&mut game, cards::GRIZZLY_BEARS);

        assert!(game.skips_turn_based_untap(permanent(&game, traxos)));
        assert!(!game.skips_turn_based_untap(permanent(&game, bears)));
    }
}
