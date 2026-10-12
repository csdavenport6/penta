//! Reality Fracture cards whose clauses read a value or condition somewhere
//! the shared mechanic tests do not already reach.
//!
//! Catalog validation accepts shapes that a particular evaluator answers with
//! zero or with nothing, so each case here runs the printed card in both
//! engine modes and checks the outcome the Oracle text and rulings require.
use super::empower_jace::{
    activate, activations, cast, casts, permanent, permanent_mut, settled, stock_library,
};
use super::woe_hob_completion::{hand, setup};
use super::*;

mod black;
mod blue;
mod green;
mod red;
mod white;

/// Whether the permanent `id` is still on the battlefield.
fn on_battlefield(game: &Game, id: GameObjectId) -> bool {
    game.battlefield
        .iter()
        .any(|permanent| permanent.card.id == id)
}

/// The objects the waiting decision offers `player`, in order.
fn offered(game: &Game, player: PlayerId) -> Vec<GameObjectId> {
    game.observe(player)
        .decision
        .expect("the player is being asked")
        .options
        .iter()
        .filter_map(|option| option.card.as_ref().map(|(id, _)| *id))
        .collect()
}

/// `player` answers the waiting decision with the option standing for `id`.
fn choose(game: &mut Game, player: PlayerId, id: GameObjectId) {
    let decision = game
        .observe(player)
        .decision
        .expect("the player is being asked");
    let option = decision
        .options
        .iter()
        .find(|option| option.card.as_ref().is_some_and(|(card, _)| *card == id))
        .expect("the wanted object is offered")
        .id;
    game.apply(
        player,
        Action::ChooseDecision {
            decision: decision.id,
            options: vec![option],
        },
    )
    .unwrap();
}

/// The targets an activation was announced with, in slot order.
fn announced_targets(action: &Action) -> Vec<Target> {
    match action {
        Action::ActivateAbility { targets, .. } => targets
            .iter()
            .flat_map(TargetSelection::targets)
            .copied()
            .collect(),
        _ => unreachable!("only activations are collected"),
    }
}

/// Player one activates `source`'s ability whose text starts with `prefix` at
/// exactly `targets`, leaving it on the stack.
fn activate_at(game: &mut Game, source: GameObjectId, prefix: &str, targets: &[Target]) {
    let action = activations(game, source, prefix)
        .into_iter()
        .find(|action| announced_targets(action) == targets)
        .unwrap_or_else(|| panic!("{prefix} can be activated at those targets"));
    game.apply(PlayerId::One, action).unwrap();
}

/// Player one's creature `attacker` is declared as an attacker and the
/// declaration is finished, with whatever that triggered still waiting.
fn declare_attack(game: &mut Game, attacker: GameObjectId) {
    game.step = Step::DeclareAttackers;
    game.apply(
        PlayerId::One,
        Action::DeclareAttacker {
            attacker,
            defender: AttackDefender::Player(PlayerId::Two),
        },
    )
    .unwrap();
    game.finish_declaring_attackers();
}

/// Player one attacks with `attacker` and whatever that triggers resolves.
fn attack_with(game: &mut Game, attacker: GameObjectId) {
    declare_attack(game, attacker);
    drain_pending(game);
}

/// Puts cards with these definitions into player one's graveyard, oldest
/// first, and returns their ids in the same order.
fn put_in_graveyard(game: &mut Game, definitions: &[CardDefinitionId]) -> Vec<GameObjectId> {
    let cards = game.build_zone(PlayerId::One, definitions).unwrap();
    let ids = cards.iter().map(|card| card.id).collect();
    game.players[0].graveyard.extend(cards);
    ids
}

/// Passes priority until somebody is asked something, and returns the
/// question; `None` once nothing is left to resolve.
fn run_to_decision(game: &mut Game) -> Option<DecisionObservation> {
    for _ in 0..16 {
        if let Some(pending) = game.pending_decisions.first() {
            return Some(pending.observation.clone());
        }
        if game.stack.is_empty() && game.pending_triggers.is_empty() {
            return None;
        }
        let player = game.priority;
        game.apply(player, Action::PassPriority).unwrap();
    }
    panic!("the stack never settled");
}

/// Player one's permanents made from `definition`.
fn controlled(game: &Game, definition: CardDefinitionId) -> Vec<GameObjectId> {
    game.battlefield
        .iter()
        .filter(|permanent| {
            permanent.controller == PlayerId::One && permanent.card.definition == definition
        })
        .map(|permanent| permanent.card.id)
        .collect()
}

/// Player one's tokens.
fn tokens(game: &Game) -> Vec<&Permanent> {
    game.battlefield
        .iter()
        .filter(|permanent| {
            permanent.controller == PlayerId::One && permanent.card.definition.is_token()
        })
        .collect()
}

/// The target lists player one could cast `card` at, one per legal cast.
fn castable_targets(game: &Game, card: GameObjectId) -> Vec<Vec<Target>> {
    casts(game, card)
        .into_iter()
        .map(|action| match action {
            Action::CastSpell { choices, .. } => choices.iter_targets().copied().collect(),
            _ => unreachable!("only casts are collected"),
        })
        .collect()
}

/// The target lists of player one's legal activations of `source`'s ability
/// whose text starts with `prefix`, one per activation.
fn activation_targets(game: &Game, source: GameObjectId, prefix: &str) -> Vec<Vec<Target>> {
    activations(game, source, prefix)
        .iter()
        .map(announced_targets)
        .collect()
}
