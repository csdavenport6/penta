//! A total over an object set, read where no effect is resolving.
//!
//! An intervening-if is asked as its trigger is placed, for the player the
//! event happened to. A total has to read that player's objects the way a
//! count of the same query does.

use super::*;
use crate::card::{
    AggregateOperationDef, ObjectValueAggregateDef, ObjectValueDef, PlayerRefDef,
    ValueComparisonDef,
};

static RULES: [AbilityDef; 1] = [AbilityDef::triggered_if(
    "At the beginning of each player's upkeep, if creatures that player controls have total \
     power 4 or greater, you gain 1 life.",
    TriggerEventDef::StepBegins {
        step: TurnStepDef::Upkeep,
        player: PlayerRelation::Any,
    },
    &TriggerConditionDef::ValueComparison(&ValueComparisonDef {
        left: ValueDef::AggregateObjectValues(&ObjectValueAggregateDef {
            objects: ObjectSetDef::Query(ObjectQueryDef::controlled_by(
                ObjectPredicateDef::HasType(CardType::Creature),
                &[ZoneKind::Battlefield],
                PlayerSetDef::One(PlayerRefDef::EventPlayer),
            )),
            select: ObjectValueDef::Power,
            operation: AggregateOperationDef::Sum,
        }),
        comparison: ComparisonDef::GreaterOrEqual,
        right: ValueDef::Constant(4),
    }),
    EffectDef::GainLife {
        recipient: EffectRecipientDef::Controller,
        amount: ValueDef::Constant(1),
    },
)];

/// Player one controls the 2/2 that asks; player two controls two more.
#[test]
fn an_intervening_if_totals_the_objects_of_the_player_the_event_names() {
    for (active, gained) in [(PlayerId::One, 0), (PlayerId::Two, 1)] {
        let (mut game, _) = super::composed_mechanic_programs::staged(&RULES);
        for _ in 0..2 {
            game.put_onto_battlefield(PlayerId::Two, cards::GRIZZLY_BEARS)
                .unwrap();
        }
        let life = game.players[0].life;
        game.active_player = active;

        game.handle_upkeep_triggers();
        game.finish_rules_procedure();
        drain_pending(&mut game);

        assert_eq!(game.players[0].life, life + gained, "{active:?}'s upkeep");
    }
}
