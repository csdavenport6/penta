//! "Creatures you control with power or toughness 1 or less can't be blocked."
//!
//! A static group rule whose members are picked by either stat. Tetsuko
//! Umezawa, Fugitive is the only source: what needs covering is that each
//! half of the "or" qualifies on its own, that a creature with neither stays
//! blockable, and that the rule follows its controller rather than the table.

use super::*;

/// `attacker` attacking for player one, Grizzly Bears ready to block for
/// player two, and Tetsuko under `tetsuko_controller` unless she is the
/// attacker herself.
fn combat(attacker: CardDefinitionId, tetsuko_controller: PlayerId) -> (Game, GameObjectId) {
    let mut game = ready_game();
    game.step = Step::DeclareBlockers;
    game.attackers_declared = true;
    game.priority = PlayerId::Two;
    let mut attacking = creature(10_000, attacker, PlayerId::One);
    attacking.attacking = true;
    attacking.attack_defender = Some(AttackDefender::Player(PlayerId::Two));
    let attacker_id = attacking.card.id;
    game.battlefield.push(attacking);
    game.battlefield
        .push(creature(10_001, cards::GRIZZLY_BEARS, PlayerId::Two));
    if attacker != cards::TETSUKO_UMEZAWA_FUGITIVE {
        game.battlefield.push(creature(
            10_002,
            cards::TETSUKO_UMEZAWA_FUGITIVE,
            tetsuko_controller,
        ));
    }
    (game, attacker_id)
}

fn can_be_blocked(game: &Game, attacker: GameObjectId) -> bool {
    game.legal_actions(PlayerId::Two).iter().any(
        |action| matches!(action, Action::DeclareBlocker { attacker: actual, .. } if *actual == attacker),
    )
}

#[test]
fn power_one_or_less_is_enough() {
    // Tetsuko is a 1/3: her power qualifies her although her toughness does not.
    let (game, tetsuko) = combat(cards::TETSUKO_UMEZAWA_FUGITIVE, PlayerId::One);
    assert!(!can_be_blocked(&game, tetsuko));
}

#[test]
fn toughness_one_or_less_is_enough() {
    // Savannah Lions is a 2/1: its toughness qualifies it although its power does not.
    let (game, lions) = combat(cards::SAVANNAH_LIONS, PlayerId::One);
    assert!(!can_be_blocked(&game, lions));
}

#[test]
fn a_creature_with_neither_stat_that_low_can_be_blocked() {
    // Grizzly Bears is a 2/2.
    let (game, bears) = combat(cards::GRIZZLY_BEARS, PlayerId::One);
    assert!(can_be_blocked(&game, bears));
}

#[test]
fn an_opponents_tetsuko_does_not_help_your_creatures() {
    let (game, lions) = combat(cards::SAVANNAH_LIONS, PlayerId::Two);
    assert!(can_be_blocked(&game, lions));
}
