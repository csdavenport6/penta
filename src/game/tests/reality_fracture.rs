//! Reality Fracture cards whose clauses read a value or condition somewhere
//! the shared mechanic tests do not already reach.
//!
//! Catalog validation accepts shapes that a particular evaluator answers with
//! zero or with nothing, so each case here runs the printed card in both
//! engine modes and checks the outcome the Oracle text and rulings require.
use super::empower_jace::{
    activate, activations, cast, casts, permanent, permanent_mut, settled, stock_library,
};
use super::mana_pool_doubling::commander_game;
use super::prepare_cards::{exiled_copies, is_prepared, take_an_upkeep};
use super::woe_hob_completion::{hand, setup};
use super::*;

mod artifacts;
mod black;
mod blue;
mod green;
mod lands;
mod legends;
mod multicolor;
mod prepare;
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

/// Player one's creature `attacker` attacks player two, nobody blocks, and
/// combat damage is dealt, with whatever that triggered still waiting.
fn connect_with(game: &mut Game, attacker: GameObjectId) {
    attack_with(game, attacker);
    game.finish_declaring_blockers();
    game.deal_combat_damage();
}

/// Puts a card with `definition` into `player`'s hand.
fn in_hand_of(game: &mut Game, player: PlayerId, definition: CardDefinitionId) -> GameObjectId {
    let card = game.build_zone(player, &[definition]).unwrap().remove(0);
    let id = card.id;
    game.players[player.index()].hand.push(card);
    id
}

/// Whether `player` could cast the card `id` with the mana they have.
fn can_cast(game: &Game, player: PlayerId, id: GameObjectId) -> bool {
    game.legal_actions(player)
        .iter()
        .any(|action| matches!(action, Action::CastSpell { card, .. } if *card == id))
}

/// Player one's turn moves from the second main phase to the end step, with
/// the mana left over from casting gone rather than burning anybody.
fn end_step_begins(game: &mut Game) {
    for player in &mut game.players {
        player.mana_pool = ManaPool::default();
        player.mana.clear();
    }
    game.step = Step::PostcombatMain;
    game.advance_step();
    assert_eq!(game.step, Step::End);
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

/// Player one answers a surveil 1 by putting the card into their graveyard,
/// or by leaving it on top of their library.
fn surveil(game: &mut Game, into_graveyard: bool) {
    let decision = game
        .observe(PlayerId::One)
        .decision
        .expect("the surveil is waiting");
    let options = if into_graveyard {
        vec![decision.options[0].id]
    } else {
        Vec::new()
    };
    game.apply(
        PlayerId::One,
        Action::ChooseDecision {
            decision: decision.id,
            options,
        },
    )
    .unwrap();
    drain_pending(game);
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

/// Whether the permanent `id` has `keyword`.
fn has_keyword(game: &Game, id: GameObjectId, keyword: KeywordAbility) -> bool {
    game.permanent_has_executable_keyword(permanent(game, id), keyword)
}

/// The colors `permanent` is.
fn colors(game: &Game, permanent: &Permanent) -> Vec<ManaColor> {
    ManaColor::COLORS
        .into_iter()
        .zip(game.permanent_colors(permanent))
        .filter_map(|(color, has)| has.then_some(color))
        .collect()
}

/// Gives player one six mana of each color, more than any card here costs.
fn mana_to_spare(game: &mut Game) {
    for color in ManaColor::COLORS {
        game.add_unrestricted_mana(PlayerId::One, color, 6);
    }
}

/// Player one casts a card with `definition` from hand with X chosen as `x`
/// and mana to spare, leaving it on the stack.
fn cast_for_x(game: &mut Game, definition: CardDefinitionId, x: u16) {
    let card = hand(game, definition);
    mana_to_spare(game);
    let action = casts(game, card)
        .into_iter()
        .find(|action| matches!(action, Action::CastSpell { choices, .. } if choices.x() == x))
        .expect("the spell can be cast for that X");
    game.apply(PlayerId::One, action).unwrap();
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
