//! Reality Fracture's legendary cards, collector numbers 195 to 280.
use super::*;

mod black;
mod blue;
mod colorless;
mod green;
mod multicolor;
mod red;
mod white;

/// Teyo cast from hand with flash, his trigger aimed at `target`.
fn teyo_entering_at(game: &mut Game, teyo: CardDefinitionId, target: GameObjectId) {
    cast(game, teyo, &[]);
    run_to_decision(game).expect("the trigger chooses its target");
    choose(game, PlayerId::One, target);
    drain_pending(game);
}

/// A token for the clauses that ask whether a creature is one.
static SOLDIER: TokenCharacteristics =
    TokenCharacteristics::creature(&["Soldier"], &[ManaColor::White], 1, 1);

/// What player one could sacrifice to activate `source`'s ability whose text
/// starts with `prefix`, one list per legal activation, in a fixed order.
fn sacrifices_offered(game: &Game, source: GameObjectId, prefix: &str) -> Vec<Vec<GameObjectId>> {
    let mut offered = activations(game, source, prefix)
        .into_iter()
        .map(|action| match action {
            Action::ActivateAbility { cost_objects, .. } => cost_objects,
            _ => unreachable!("only activations are collected"),
        })
        .collect::<Vec<_>>();
    offered.sort();
    offered
}

/// Player one activates `source`'s mana ability for one mana of `color`.
fn tap_for(game: &mut Game, source: GameObjectId, color: ManaColor) {
    let tap = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| {
            matches!(action, Action::ActivateManaAbility { source: tapped, color: added, .. }
                if *tapped == source && *added == color)
        })
        .expect("the permanent taps for that color");
    game.apply(PlayerId::One, tap).unwrap();
}
