//! Reality Fracture cards whose clauses read a value or condition somewhere
//! the shared mechanic tests do not already reach.
//!
//! Catalog validation accepts shapes that a particular evaluator answers with
//! zero or with nothing, so each case here runs the printed card in both
//! engine modes and checks the outcome the Oracle text and rulings require.
//!
//! The cases are grouped by the rule area they exercise, not by card: one
//! card's cases can sit in several submodules, and a card whose clauses the
//! shared mechanic tests already cover has none. The fixtures here are the
//! ones more than one submodule uses.
use super::empower_jace::{
    activate, activations, cast, casts, permanent, permanent_mut, reconstructed, settled,
    stock_library,
};
use super::mana_pool_doubling::commander_game;
use super::woe_hob_completion::{hand, setup};
use super::*;
use crate::card::PermanentDesignationDef;

mod choices;
mod last_known_information;
mod payments;
mod permanent_rules;
mod resolution;
mod targeting;
mod trigger_conditions;
mod triggers;
mod zone_permissions;

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

/// Whether `permanent` is prepared.
fn is_prepared(game: &Game, permanent: GameObjectId) -> bool {
    game.battlefield
        .iter()
        .find(|candidate| candidate.card.id == permanent)
        .expect("the permanent is on the battlefield")
        .designations
        .contains(&PermanentDesignationDef::Prepared)
}

/// Everything `player` has in exile, by name.
fn exiled_copies(game: &Game, player: PlayerId) -> Vec<(GameObjectId, String)> {
    game.players[player.index()]
        .exile
        .iter()
        .map(|card| {
            (
                card.id,
                game.object_card_name(card.id)
                    .expect("an exiled copy has a name")
                    .into_owned(),
            )
        })
        .collect()
}

/// Puts player one's preparation card `definition` onto the battlefield,
/// where it enters prepared, and returns that prepared permanent with the
/// copy of its prepare spell `spell` that this made in exile.
fn prepared_permanent(
    game: &mut Game,
    definition: CardDefinitionId,
    spell: &str,
) -> (GameObjectId, GameObjectId) {
    let permanent = settled(game, definition);
    assert!(is_prepared(game, permanent), "it enters prepared");
    let copies = exiled_copies(game, PlayerId::One);
    assert_eq!(copies.len(), 1, "one copy of its prepare spell");
    assert_eq!(copies[0].1, spell);
    (permanent, copies[0].0)
}

/// Player one casts the exiled prepare-spell copy `copy` at `targets` with
/// mana to spare, leaving it on the stack.
fn cast_copy(game: &mut Game, copy: GameObjectId, targets: &[Target]) {
    mana_to_spare(game);
    let action = casts(game, copy)
        .into_iter()
        .find(|action| {
            matches!(action, Action::CastSpell { choices, .. }
            if choices.iter_targets().eq(targets))
        })
        .expect("the copy can be cast at those targets");
    game.apply(PlayerId::One, action).unwrap();
}
