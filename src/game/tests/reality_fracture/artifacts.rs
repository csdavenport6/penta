//! Reality Fracture's artifacts.
use super::*;

/// Eye of Jace's upkeep trigger over a graveyard already holding `graveyard`
/// cards, stopped at its surveil.
fn eye_of_jace_surveilling(prepared_engine: bool, graveyard: usize) -> (Game, GameObjectId) {
    let mut game = setup(prepared_engine);
    stock_library(&mut game);
    let eye = settled(&mut game, cards::EYE_OF_JACE);
    put_in_graveyard(&mut game, &vec![cards::MOUNTAIN; graveyard]);
    game.commit_next_turn(PlayerId::One, Vec::new());
    run_to_decision(&mut game).expect("the surveil asks about the top card");
    (game, eye)
}

/// "Surveil 1. Then if there are seven or more cards in your graveyard,
/// sacrifice this artifact, it deals 2 damage to each opponent, and you gain
/// 2 life": the count is taken after the surveil, so the card it just put
/// there is the seventh, and the Eye deals its damage from the graveyard.
#[test]
fn eye_of_jace_counts_the_card_its_own_surveil_put_into_the_graveyard() {
    for prepared_engine in [false, true] {
        let (mut game, eye) = eye_of_jace_surveilling(prepared_engine, 6);

        surveil(&mut game, true);

        assert!(!on_battlefield(&game, eye));
        assert_eq!(game.players[1].life, 18);
        assert_eq!(game.players[0].life, 22);
    }
}

#[test]
fn eye_of_jace_stays_while_the_graveyard_is_short_of_seven() {
    for prepared_engine in [false, true] {
        let (mut game, eye) = eye_of_jace_surveilling(prepared_engine, 6);

        surveil(&mut game, false);

        assert!(on_battlefield(&game, eye));
        assert_eq!(game.players[1].life, 20);
        assert_eq!(game.players[0].life, 20);
    }
}

/// "Destroy target noncreature, nonland permanent": both words.
#[test]
fn archive_arbiter_destroys_only_a_noncreature_nonland_permanent() {
    for prepared_engine in [false, true] {
        let mut game = setup(prepared_engine);
        game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
            .unwrap();
        game.put_onto_battlefield(PlayerId::Two, cards::FOREST)
            .unwrap();
        let mox = game
            .put_onto_battlefield(PlayerId::Two, cards::MOX_RUBY)
            .unwrap();

        cast(&mut game, cards::ARCHIVE_ARBITER, &[]);
        run_to_decision(&mut game).expect("the trigger chooses its mode");
        choose_decision_by_label(
            &mut game,
            PlayerId::One,
            "Destroy target noncreature, nonland permanent.",
        );
        assert_eq!(
            offered(&game, PlayerId::One),
            [mox],
            "not a creature, a land, or the Arbiter itself"
        );
        choose(&mut game, PlayerId::One, mox);
        drain_pending(&mut game);

        assert!(!on_battlefield(&game, mox));
    }
}
