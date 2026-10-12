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
