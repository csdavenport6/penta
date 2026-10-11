//! Walk-In Closet // Forgotten Cellar, and the Room mechanic under it: two
//! doors on one enchantment, opened one at a time and paid for separately.

use super::*;

const CLOSET: CardPartId = CardPartId::PRIMARY;
const CELLAR: CardPartId = CardPartId(1);
const BOTH: CardPartId = CardPartId(2);
const NEITHER: CardPartId = CardPartId(3);

/// A main phase with the Room in hand, a Mountain and a Lightning Bolt in
/// the graveyard, and enough mana for either door.
fn staged() -> (Game, GameObjectId) {
    let mut game = ready_game();
    game.battlefield.clear();
    game.players[0].hand.clear();
    game.players[0].graveyard.clear();
    game.players[0]
        .graveyard
        .push(card(88_000, cards::MOUNTAIN, PlayerId::One));
    game.players[0]
        .graveyard
        .push(card(88_001, cards::LIGHTNING_BOLT, PlayerId::One));
    let room = card(
        88_010,
        cards::WALK_IN_CLOSET_FORGOTTEN_CELLAR,
        PlayerId::One,
    );
    let room_id = room.id;
    game.players[0].hand.push(room);
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Green, 5);
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Colorless, 8);
    // Enough red to cast the Bolt if something lets it be cast, so the
    // graveyard offers below turn on the permission rather than on mana.
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 3);
    (game, room_id)
}

fn cast_door(game: &mut Game, room: GameObjectId, option: PlayOptionId) {
    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| {
            matches!(action, Action::CastSpell { card, choices, .. }
                if *card == room && choices.play_option() == option)
        })
        .expect("the door is castable");
    game.apply(PlayerId::One, action)
        .expect("the cast is legal");
    drain_pending(game);
}

fn the_room(game: &Game) -> &Permanent {
    game.battlefield
        .iter()
        .find(|permanent| permanent.card.definition == cards::WALK_IN_CLOSET_FORGOTTEN_CELLAR)
        .expect("the Room is on the battlefield")
}

fn unlocks(game: &Game) -> Vec<CardPartId> {
    game.legal_actions(PlayerId::One)
        .into_iter()
        .filter_map(|action| match action {
            Action::UnlockDoor { door, .. } => Some(door),
            _ => None,
        })
        .collect()
}

fn land_plays(game: &Game) -> Vec<GameObjectId> {
    game.legal_actions(PlayerId::One)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayLand { card, .. } => Some(card),
            _ => None,
        })
        .collect()
}

/// Which cards in your graveyard are castable, each named once however many
/// targets the spell is offered for.
fn graveyard_casts(game: &Game) -> Vec<GameObjectId> {
    let graveyard = game.players[0]
        .graveyard
        .iter()
        .map(|card| card.id)
        .collect::<Vec<_>>();
    let mut castable = Vec::new();
    for action in game.legal_actions(PlayerId::One) {
        if let Action::CastSpell { card, .. } = action
            && graveyard.contains(&card)
            && !castable.contains(&card)
        {
            castable.push(card);
        }
    }
    castable
}

fn in_graveyard(game: &Game, definition: CardDefinitionId) -> bool {
    game.players[0]
        .graveyard
        .iter()
        .any(|card| card.definition == definition)
}

fn in_exile(game: &Game, definition: CardDefinitionId) -> bool {
    game.players[0]
        .exile
        .iter()
        .any(|card| card.definition == definition)
}

/// Casts the cellar, then returns the Room to its owner's hand, leaving the
/// turn's two effects with no Room on the battlefield to belong to.
fn open_the_cellar_then_bounce_the_room(game: &mut Game, room: GameObjectId) {
    cast_door(game, room, PlayOptionId(1));
    let permanent = the_room(game).card.id;
    game.return_permanent_to_hand(permanent);
    drain_pending(game);
    assert!(
        game.battlefield
            .iter()
            .all(|permanent| permanent.card.definition != cards::WALK_IN_CLOSET_FORGOTTEN_CELLAR),
        "the Room is gone",
    );
}

/// Both halves are offered from hand, each for its own cost.
#[test]
fn either_door_can_be_cast() {
    let (game, room) = staged();
    let options = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .filter_map(|action| match action {
            Action::CastSpell { card, choices, .. } if card == room => Some(choices.play_option()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(options.contains(&PlayOptionId::DEFAULT), "Walk-In Closet");
    assert!(options.contains(&PlayOptionId(1)), "Forgotten Cellar");
    assert_eq!(options.len(), 2, "two doors, and no way to cast both");
}

/// Casting a half puts the Room onto the battlefield with that door open and
/// the other one shut.
#[test]
fn casting_a_door_unlocks_only_that_door() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId::DEFAULT);

    assert_eq!(the_room(&game).presented, CLOSET, "the closet is open");
    assert_eq!(unlocks(&game), vec![CELLAR], "and the cellar is not");
}

/// The closet's own line is Crucible of Worlds', and it works as soon as the
/// door does.
#[test]
fn the_closet_opens_the_graveyard_to_lands() {
    let (mut game, room) = staged();
    assert!(land_plays(&game).is_empty(), "nothing yet");

    cast_door(&mut game, room, PlayOptionId::DEFAULT);

    assert_eq!(
        land_plays(&game),
        vec![GameObjectId(88_000)],
        "the Mountain, and not the Bolt",
    );
}

/// A locked door is bought at sorcery speed for its own printed cost, and
/// opening it leaves the Room presenting both halves.
#[test]
fn unlocking_the_other_door_costs_its_own_mana() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId::DEFAULT);
    let before = game.players[0].mana_pool.total();

    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| matches!(action, Action::UnlockDoor { door, .. } if *door == CELLAR))
        .expect("the cellar is offered");
    game.apply(PlayerId::One, action).expect("it unlocks");
    drain_pending(&mut game);

    assert_eq!(the_room(&game).presented, BOTH, "both doors are open");
    assert_eq!(
        before - game.players[0].mana_pool.total(),
        5,
        "five mana, which is what the cellar costs",
    );
    assert!(unlocks(&game).is_empty(), "nothing left to open");
}

/// With both open the Room has both text boxes: the closet's permission
/// survives the cellar being unlocked next to it.
#[test]
fn both_doors_give_both_abilities() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId(1));
    assert!(
        land_plays(&game).is_empty(),
        "the cellar alone says nothing about lands",
    );

    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| matches!(action, Action::UnlockDoor { door, .. } if *door == CLOSET))
        .expect("the closet is offered");
    game.apply(PlayerId::One, action).expect("it unlocks");
    drain_pending(&mut game);

    assert_eq!(the_room(&game).presented, BOTH);
    assert_eq!(
        land_plays(&game),
        vec![GameObjectId(88_000)],
        "the closet's line came with the door",
    );
}

/// A Room that arrives without anyone casting a half arrives shut, with no
/// name of its own and nothing in either text box (CR 714.3d).
#[test]
fn a_room_that_was_not_cast_enters_locked() {
    let (mut game, _room) = staged();
    game.put_onto_battlefield(PlayerId::One, cards::WALK_IN_CLOSET_FORGOTTEN_CELLAR)
        .expect("cataloged");
    drain_pending(&mut game);

    assert_eq!(the_room(&game).presented, NEITHER, "both doors shut");
    assert!(land_plays(&game).is_empty(), "and no abilities");
    let mut offered = unlocks(&game);
    offered.sort_by_key(|door| door.0);
    assert_eq!(offered, vec![CLOSET, CELLAR], "either one may be bought");
}

/// It is a special action with sorcery timing: no opponent's turn, and no
/// unlocking with something on the stack.
#[test]
fn unlocking_keeps_sorcery_timing() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId::DEFAULT);

    game.step = Step::DeclareAttackers;
    assert!(unlocks(&game).is_empty(), "not outside a main phase");

    game.step = Step::PrecombatMain;
    game.active_player = PlayerId::Two;
    assert!(unlocks(&game).is_empty(), "not on their turn");
}

/// "When you unlock this door": the cellar's trigger fires as the Room
/// enters, because casting that half is what opened it.
#[test]
fn casting_the_cellar_fires_its_unlock_trigger() {
    let (mut game, room) = staged();
    assert!(graveyard_casts(&game).is_empty(), "nothing yet");

    cast_door(&mut game, room, PlayOptionId(1));

    assert_eq!(
        graveyard_casts(&game),
        vec![GameObjectId(88_001)],
        "the Bolt is castable out of the graveyard",
    );
}

/// The closet's door carries no such clause, so opening it opens nothing.
#[test]
fn the_closet_does_not_open_the_graveyard_to_spells() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId::DEFAULT);

    assert!(
        graveyard_casts(&game).is_empty(),
        "lands are what the closet said",
    );
}

/// Unlocking the cellar on the battlefield fires it the same way casting it
/// did -- and the closet's door, which has no such clause, does not fire it.
#[test]
fn unlocking_the_cellar_fires_it_too() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId::DEFAULT);
    assert!(graveyard_casts(&game).is_empty(), "the closet is silent");

    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| matches!(action, Action::UnlockDoor { door, .. } if *door == CELLAR))
        .expect("the cellar is offered");
    game.apply(PlayerId::One, action).expect("it unlocks");
    drain_pending(&mut game);

    assert_eq!(
        graveyard_casts(&game),
        vec![GameObjectId(88_001)],
        "the trigger came with the door",
    );
}

/// The other half of the cellar's sentence: a card headed for your graveyard
/// is exiled instead, for the rest of the turn.
#[test]
fn the_cellar_exiles_what_would_reach_your_graveyard() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId(1));
    let bear = game
        .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
        .expect("cataloged");
    drain_pending(&mut game);
    let exiled_before = game.players[0].exile.len();

    game.move_permanents_to_graveyard(&[bear]);
    drain_pending(&mut game);

    assert!(
        game.players[0]
            .graveyard
            .iter()
            .all(|card| card.definition != cards::GRIZZLY_BEARS),
        "it did not reach the graveyard",
    );
    assert_eq!(
        game.players[0].exile.len(),
        exiled_before + 1,
        "it went to exile instead",
    );
}

/// "Your graveyard": theirs fills up as it always would.
#[test]
fn the_cellar_leaves_their_graveyard_alone() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId(1));
    let bear = game
        .put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
        .expect("cataloged");
    drain_pending(&mut game);

    game.move_permanents_to_graveyard(&[bear]);
    drain_pending(&mut game);

    assert!(
        game.players[1]
            .graveyard
            .iter()
            .any(|card| card.definition == cards::GRIZZLY_BEARS),
        "the clause names your own graveyard",
    );
}

/// Both halves of the cellar's sentence last only the turn.
#[test]
fn the_cellars_effects_end_with_the_turn() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId(1));
    assert!(!graveyard_casts(&game).is_empty(), "on this turn it works");

    game.complete_cleanup();
    drain_pending(&mut game);
    game.active_player = PlayerId::One;
    game.step = Step::PrecombatMain;
    game.priority = PlayerId::One;
    game.add_unrestricted_mana(PlayerId::One, ManaColor::Red, 3);

    assert!(
        graveyard_casts(&game).is_empty(),
        "the permission lasted the turn it was given for",
    );
    assert!(
        game.ongoing_effects.is_empty(),
        "and so did the replacement"
    );
    let bear = game
        .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
        .expect("cataloged");
    drain_pending(&mut game);
    game.move_permanents_to_graveyard(&[bear]);
    drain_pending(&mut game);
    assert!(
        in_graveyard(&game, cards::GRIZZLY_BEARS),
        "a creature dying next turn reaches the graveyard again",
    );
}

/// The two clauses meet: a spell cast out of the graveyard resolves and is
/// exiled rather than falling back in to be cast again.
#[test]
fn a_spell_cast_from_the_graveyard_is_exiled_afterwards() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId(1));
    let bolt = GameObjectId(88_001);

    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| matches!(action, Action::CastSpell { card, .. } if *card == bolt))
        .expect("the Bolt is castable out of the graveyard");
    game.apply(PlayerId::One, action)
        .expect("the cast is legal");
    drain_pending(&mut game);

    assert!(
        game.players[0]
            .graveyard
            .iter()
            .all(|card| card.definition != cards::LIGHTNING_BOLT),
        "it did not fall back into the graveyard",
    );
    assert!(
        game.players[0]
            .exile
            .iter()
            .any(|card| card.definition == cards::LIGHTNING_BOLT),
        "the other half of the sentence caught it",
    );
    assert!(
        graveyard_casts(&game).is_empty(),
        "and so it cannot be cast a second time",
    );
}

/// The replacement is the player's for the turn, not the Room's. With the Room
/// back in hand, a creature that dies and a card that is discarded are both
/// still exiled: the battlefield exit and the move from anywhere else each
/// find the effect with no Room left to carry it.
#[test]
fn the_room_leaves_mid_turn_and_cards_are_still_exiled() {
    let (mut game, room) = staged();
    let bear = game
        .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
        .expect("cataloged");
    drain_pending(&mut game);
    let growth = card(88_020, cards::GIANT_GROWTH, PlayerId::One);
    let growth_id = growth.id;
    game.players[0].hand.push(growth);

    open_the_cellar_then_bounce_the_room(&mut game, room);

    game.move_permanents_to_graveyard(&[bear]);
    drain_pending(&mut game);
    assert!(
        !in_graveyard(&game, cards::GRIZZLY_BEARS),
        "the dying creature did not reach the graveyard",
    );
    assert!(
        in_exile(&game, cards::GRIZZLY_BEARS),
        "it was exiled instead",
    );

    game.discard_cards(PlayerId::One, &[growth_id]);
    drain_pending(&mut game);
    assert!(
        !in_graveyard(&game, cards::GIANT_GROWTH),
        "the discarded card did not reach the graveyard",
    );
    assert!(
        in_exile(&game, cards::GIANT_GROWTH),
        "it was exiled instead",
    );
}

/// The permission is the player's for the turn too: the Bolt stays castable
/// from the graveyard with the Room back in hand, and is exiled once cast.
#[test]
fn the_room_leaves_mid_turn_and_the_graveyard_is_still_castable() {
    let (mut game, room) = staged();
    open_the_cellar_then_bounce_the_room(&mut game, room);
    let bolt = GameObjectId(88_001);

    assert_eq!(
        graveyard_casts(&game),
        vec![bolt],
        "the Bolt is still castable out of the graveyard",
    );
    assert!(
        land_plays(&game).is_empty(),
        "spells only: the cellar never opened the graveyard to lands",
    );

    let action = game
        .legal_actions(PlayerId::One)
        .into_iter()
        .find(|action| matches!(action, Action::CastSpell { card, .. } if *card == bolt))
        .expect("the Bolt is castable out of the graveyard");
    game.apply(PlayerId::One, action)
        .expect("the cast is legal");
    drain_pending(&mut game);

    assert!(
        !in_graveyard(&game, cards::LIGHTNING_BOLT),
        "it did not fall back into the graveyard",
    );
    assert!(
        in_exile(&game, cards::LIGHTNING_BOLT),
        "the replacement caught it without the Room",
    );
}

/// A Room destroyed that turn is itself a card headed for your graveyard, so
/// its own clause exiles it, and goes on exiling afterwards.
#[test]
fn a_room_destroyed_that_turn_is_exiled_by_its_own_clause() {
    let (mut game, room) = staged();
    cast_door(&mut game, room, PlayOptionId(1));
    let bear = game
        .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
        .expect("cataloged");
    drain_pending(&mut game);

    let permanent = the_room(&game).card.id;
    game.destroy_permanent(permanent);
    drain_pending(&mut game);
    assert!(
        !in_graveyard(&game, cards::WALK_IN_CLOSET_FORGOTTEN_CELLAR),
        "the Room did not reach the graveyard",
    );
    assert!(
        in_exile(&game, cards::WALK_IN_CLOSET_FORGOTTEN_CELLAR),
        "it was exiled by the effect it made",
    );

    game.move_permanents_to_graveyard(&[bear]);
    drain_pending(&mut game);
    assert!(
        in_exile(&game, cards::GRIZZLY_BEARS),
        "and the effect outlived it",
    );
}

/// A Room card in a library or a hand is both halves at once, which is what
/// makes its mana value eight there (CR 714.2a).
#[test]
fn the_card_is_both_halves_outside_the_battlefield() {
    let game = ready_game();
    let definition = game
        .catalog
        .get(cards::WALK_IN_CLOSET_FORGOTTEN_CELLAR)
        .expect("cataloged");
    let parts = crate::card::applicable_part_ids(definition, &CharacteristicContext::Hand)
        .expect("a Room card is legible in hand");

    assert_eq!(parts, vec![CLOSET, CELLAR], "the doors, and only the doors");
}

/// "Walk-In Closet's ability doesn't allow you to activate abilities (such
/// as cycling) of land cards in your graveyard." The permission is to play a
/// land, and a Triome in the graveyard is a land to play and not a card to
/// cycle -- its cycling is a hand ability and stays one.
#[test]
fn the_closet_opens_the_land_drop_and_not_the_cycling() {
    let (mut game, room) = staged();
    let triome = card(88_100, cards::RAUGRIN_TRIOME, PlayerId::One);
    let triome_id = triome.id;
    game.players[0].graveyard.push(triome);
    game.players[0].lands_played_this_turn = 0;

    cast_door(&mut game, room, PlayOptionId::DEFAULT);

    assert!(
        land_plays(&game).contains(&triome_id),
        "the Triome is a land the closet lets you play",
    );
    assert!(
        !game.legal_actions(PlayerId::One).into_iter().any(
            |action| matches!(action, Action::ActivateAbility { source, .. } if source == triome_id)
        ),
        "and its cycling is not an ability the closet reaches",
    );

    // The same card in hand cycles as it always did, so what is shut is the
    // graveyard rather than the clause.
    let held = card(88_101, cards::RAUGRIN_TRIOME, PlayerId::One);
    let held_id = held.id;
    game.players[0].hand.push(held);
    assert!(
        game.legal_actions(PlayerId::One).into_iter().any(
            |action| matches!(action, Action::ActivateAbility { source, .. } if source == held_id)
        ),
        "a Triome in hand cycles for its three all the same",
    );
}

/// A checkpoint taken mid-turn carries the cellar's replacement with it. The
/// ability sits beneath a door part's unlock trigger and is a replacement, not
/// an activated ability; the rebuilt game still exiles, and still lets go at
/// cleanup.
#[test]
fn a_checkpoint_mid_turn_keeps_the_cellars_replacement() {
    let (mut game, room) = staged();
    let doomed = game
        .put_onto_battlefield(PlayerId::One, cards::GRIZZLY_BEARS)
        .expect("cataloged");
    let survivor = game
        .put_onto_battlefield(PlayerId::One, cards::SAVANNAH_LIONS)
        .expect("cataloged");
    drain_pending(&mut game);
    let growth = card(88_030, cards::GIANT_GROWTH, PlayerId::One);
    let growth_id = growth.id;
    game.players[0].hand.push(growth);
    cast_door(&mut game, room, PlayOptionId(1));
    assert_eq!(game.ongoing_effects.len(), 1, "the replacement is live");

    let (wire, hidden) = checkpoint_fixture(&game, PlayerId::One);
    assert_eq!(
        wire["checkpoint"]["hasDeferredState"],
        serde_json::Value::Bool(false),
        "the replacement has stable catalog semantics",
    );
    let mut rebuilt =
        Game::from_observation_checkpoint(game.catalog.clone(), game.format, &wire, &hidden, 4_242)
            .expect("the cellar's replacement reconstructs");
    assert_eq!(rebuilt.ongoing_effects, game.ongoing_effects);

    rebuilt.move_permanents_to_graveyard(&[doomed]);
    drain_pending(&mut rebuilt);
    assert!(
        !in_graveyard(&rebuilt, cards::GRIZZLY_BEARS),
        "the dying creature did not reach the rebuilt graveyard",
    );
    assert!(
        in_exile(&rebuilt, cards::GRIZZLY_BEARS),
        "the rebuilt effect exiled it",
    );
    rebuilt.discard_cards(PlayerId::One, &[growth_id]);
    drain_pending(&mut rebuilt);
    assert!(
        in_exile(&rebuilt, cards::GIANT_GROWTH),
        "and a discarded card with it",
    );

    rebuilt.complete_cleanup();
    drain_pending(&mut rebuilt);
    assert!(
        rebuilt.ongoing_effects.is_empty(),
        "the rebuilt effect still ends with the turn",
    );
    rebuilt.move_permanents_to_graveyard(&[survivor]);
    drain_pending(&mut rebuilt);
    assert!(
        in_graveyard(&rebuilt, cards::SAVANNAH_LIONS),
        "and a creature dying afterwards reaches the graveyard",
    );
}
