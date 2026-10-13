// Every program an ability can run reaches the resolving payment procedure
// in its own scope: nothing outside the ability supplies a cost parameter.
fn validate_ability_cost_program(ability: &AbilityDef) -> Result<(), &'static str> {
    match (
        ability.declarative_effect(),
        ability.declarative_replacement(),
    ) {
        (Some(effect), _) => validate_cost_program(effect, None),
        (None, Some(program)) => validate_replacement_cost_program(program),
        (None, None) => Ok(()),
    }
}

// A parameter is valid only beneath its lexical supplier. Keep this check
// separate from effect traversal so copied/granted programs get the same rules.
fn validate_cost_program(
    effect: EffectDef,
    parameter: Option<&'static [CostDef]>,
) -> Result<(), &'static str> {
    // An effect-created ability resolves later as its own stack object, so
    // it is checked as an ability rather than as a child of this effect.
    match effect {
        EffectDef::InstallTrigger(crate::card::InstalledTriggerDef { ability, .. })
        | EffectDef::ReflexiveTrigger(ability) => return validate_ability_cost_program(ability),
        EffectDef::CreateOngoingEffect(ongoing) => {
            return validate_ability_cost_program(ongoing.ability);
        }
        _ => {}
    }
    if let EffectDef::WithCosts { costs, effect } = effect {
        if costs.iter().any(|cost| contains_cost_parameter(*cost)) {
            return Err("WithCosts cannot bind an unresolved parameter");
        }
        return validate_cost_program(*effect, Some(costs));
    }
    if let EffectDef::PayOr(payment) = effect {
        for cost in payment.payment.costs {
            validate_program_cost(*cost, parameter, payment.label.is_some(), false)?;
        }
    }
    for child in crate::card::child_effects(effect) {
        validate_cost_program(child, parameter)?;
    }
    Ok(())
}

fn validate_program_cost(
    cost: CostDef,
    parameter: Option<&'static [CostDef]>,
    repeated_or_labeled: bool,
    scalar_only: bool,
) -> Result<(), &'static str> {
    let (costs, repeated_or_labeled, scalar_only) = match cost {
        CostDef::Parameter => (
            parameter.ok_or("unbound cost parameter")?,
            repeated_or_labeled,
            scalar_only,
        ),
        CostDef::All(costs) => (costs, repeated_or_labeled, scalar_only),
        CostDef::Repeated { costs, .. } => (costs, true, scalar_only),
        CostDef::Choice(costs) => (costs, repeated_or_labeled, true),
        CostDef::Perform(program) if !scalar_only => {
            return if program.payment_program_supported() {
                Ok(())
            } else {
                Err("unsupported action payment program")
            };
        }
        _ if scalar_only && !scalar_batch_cost(cost) => {
            return Err("unsupported non-scalar cost choice");
        }
        _ if repeated_or_labeled && !repeatable_payment_cost(cost) => {
            return Err("unsupported repeated or labeled payment cost");
        }
        // Those lanes are settled by their own procedures.
        _ if scalar_only || repeated_or_labeled => return Ok(()),
        // Anything else is settled as written, through the same narrowing
        // the payment procedure consumes.
        _ if cost.resolving_payment_cost().is_some() => return Ok(()),
        _ => return Err("unsupported resolving payment cost"),
    };
    for cost in costs {
        validate_program_cost(*cost, parameter, repeated_or_labeled, scalar_only)?;
    }
    Ok(())
}

// A replacement program reaches the resolving payment procedure through its
// own `PayOr` and through any effect it performs. Neither has a label or an
// enclosing `WithCosts` to supply a parameter.
fn validate_replacement_cost_program(effect: ReplacementEffectDef) -> Result<(), &'static str> {
    let all = |effects: &[ReplacementEffectDef]| {
        effects
            .iter()
            .try_for_each(|effect| validate_replacement_cost_program(*effect))
    };
    match effect {
        ReplacementEffectDef::Perform(effect) => validate_cost_program(*effect, None),
        ReplacementEffectDef::PayOr {
            payment,
            if_paid,
            if_declined,
        } => {
            for cost in payment.costs {
                validate_program_cost(*cost, None, false, false)?;
            }
            all(if_paid)?;
            all(if_declined)
        }
        ReplacementEffectDef::BindOutput { effect, .. } => {
            validate_replacement_cost_program(*effect)
        }
        ReplacementEffectDef::Sequence(effects) => all(effects),
        ReplacementEffectDef::Conditional {
            if_true, if_false, ..
        } => {
            all(if_true)?;
            all(if_false)
        }
        ReplacementEffectDef::ReplaceEventWithNothing
        | ReplacementEffectDef::MoveToZone(_)
        | ReplacementEffectDef::RegenerateDestroyedObject
        | ReplacementEffectDef::RemoveDamageFromDestroyedObject
        | ReplacementEffectDef::ModifyBattlefieldEntry(_)
        | ReplacementEffectDef::PlaceCountersOnMovedObject { .. }
        | ReplacementEffectDef::SetEventAmount(_)
        | ReplacementEffectDef::SetManaType(_)
        | ReplacementEffectDef::MultiplyEventAmount(_)
        | ReplacementEffectDef::AddToEventAmount(_)
        | ReplacementEffectDef::Choose(_)
        | ReplacementEffectDef::LookAtHand(_)
        | ReplacementEffectDef::CopyEntering { .. } => Ok(()),
    }
}

fn contains_cost_parameter(cost: CostDef) -> bool {
    match cost {
        CostDef::Parameter => true,
        CostDef::All(costs) | CostDef::Choice(costs) | CostDef::Repeated { costs, .. } => {
            costs.iter().any(|cost| contains_cost_parameter(*cost))
        }
        _ => false,
    }
}

fn repeatable_payment_cost(cost: CostDef) -> bool {
    match cost {
        CostDef::All(costs) => costs.iter().all(|cost| repeatable_payment_cost(*cost)),
        CostDef::Choice(_) => scalar_batch_cost(cost),
        CostDef::Mana(cost) => !cost.variable_x,
        CostDef::SnowMana(_)
        | CostDef::PayLife(_)
        | CostDef::Energy(_)
        | CostDef::MillCards(_)
        | CostDef::DrawCards(_)
        | CostDef::DiscardCards(_)
        | CostDef::ExileTopCards(_)
        | CostDef::FlipCoins(_)
        | CostDef::PutCountersOnSource { .. }
        | CostDef::SacrificePermanents {
            controller: PlayerRelation::You,
            ..
        }
        | CostDef::GainLife {
            player: PlayerRelation::Opponent,
            ..
        }
        | CostDef::CreateTokens {
            player: PlayerRelation::Opponent,
            ..
        } => true,
        CostDef::AddMana(effect) => {
            matches!(
                effect.mana,
                crate::card::ManaSelectionDef::One(crate::card::ManaTypeDef::Fixed(_))
            ) && effect.also.is_none()
                && effect.variable_amount.is_none()
                && effect.amount_override.is_none()
                && effect.damage_to_controller == 0
                && effect.sacrifice_source_when_out_of.is_none()
                && effect.restrictions.is_empty()
                && effect.spend_effects.is_empty()
        }
        _ => false,
    }
}

fn scalar_batch_cost(cost: CostDef) -> bool {
    match cost {
        CostDef::Mana(cost) => !cost.variable_x,
        CostDef::PayLife(_) => true,
        CostDef::All(costs) | CostDef::Choice(costs) => {
            costs.iter().all(|cost| scalar_batch_cost(*cost))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{AbilityLabel, PayOrDef, actions};

    const SCALAR_CHOICES: &[CostDef] = &[
        CostDef::PayLife(1),
        CostDef::Mana(crate::ManaCost::new(1, 0)),
    ];

    static PAY_PARAMETER: EffectDef = EffectDef::PayOr(
        PayOrDef::optional(
            &[CostDef::repeated(
                &[CostDef::Parameter],
                &ValueDef::Constant(2),
            )],
            &EffectDef::None,
        )
        .labeled(AbilityLabel::from_name("test:purpose"))
        .with_visibility(crate::card::ChoiceVisibilityDef::Public),
    );

    #[test]
    fn named_actions_compose_with_payment_purpose_and_other_costs() {
        const ID: crate::card::MechanicId = crate::card::MechanicId::from_name("test:selection");
        const NAMED: CostDef = actions::choose_sacrifice(2).named(ID).as_cost();
        for costs in [
            &[NAMED][..],
            &[CostDef::All(&[NAMED])][..],
            &[NAMED, CostDef::PayLife(1)][..],
        ] {
            let costs = Box::leak(costs.to_vec().into_boxed_slice());
            assert!(
                validate_cost_program(
                    EffectDef::PayOr(PayOrDef::optional(costs, &EffectDef::None).labeled(ID)),
                    None
                )
                .is_ok()
            );
        }
    }

    #[test]
    fn game_action_programs_reject_unplannable_payments() {
        use crate::card::{EffectRecipientDef, GameActionDef};
        static DISCARD: GameActionDef = actions::choose_discard(3);
        static CHOICE: [CostDef; 1] = [DISCARD.as_cost()];
        const OPPONENT_CHOOSES: CostDef = actions::choose_discard(3)
            .with_chooser(crate::card::PlayerRefDef::Opponent)
            .as_cost();
        assert!(validate_program_cost(DISCARD.as_cost(), None, true, false).is_ok());
        assert!(validate_program_cost(CostDef::Choice(&CHOICE), None, false, false).is_err());
        assert!(
            validate_program_cost(
                GameActionDef::SacrificeYours {
                    object: EffectRecipientDef::Source,
                }
                .as_cost(),
                None,
                false,
                false
            )
            .is_err(),
            "an arbitrary action body has no complete payment plan"
        );
        let GameActionDef::Choose(mut choice) = DISCARD else {
            unreachable!()
        };
        choice.then = &GameActionDef::DiscardCards {
            object: EffectRecipientDef::Source,
        };
        let invalid = Box::leak(Box::new(GameActionDef::Choose(choice)));
        assert!(
            validate_program_cost(invalid.as_cost(), None, false, false).is_err(),
            "payment must execute the exact objects it selects"
        );
        assert!(
            validate_program_cost(OPPONENT_CHOOSES, None, false, false,).is_err(),
            "cost conversion preserves the payment planner's chooser boundary"
        );
    }

    // csdavenport6/penta#60: a leaf outside the repeated, labeled, and scalar
    // lanes is settled by the plain resolving-payment procedure.
    #[test]
    fn plain_payment_leaves_must_be_resolvable() {
        const REMOVE: CostDef = CostDef::RemoveCountersFromSource {
            kind: crate::card::CounterKind::PlusOnePlusOne,
            amount: 1,
        };
        const SACRIFICE: CostDef = CostDef::sacrifice_permanent(ObjectPredicateDef::Any);
        const DISCARD: CostDef = CostDef::discard(ObjectPredicateDef::Any);
        static PAY_SUPPLIED: EffectDef =
            EffectDef::PayOr(PayOrDef::optional(&[CostDef::Parameter], &EffectDef::None));
        let pay = |costs: &'static [CostDef]| {
            validate_cost_program(
                EffectDef::PayOr(PayOrDef::optional(costs, &EffectDef::None)),
                None,
            )
        };
        for costs in [
            &[REMOVE][..],
            &[CostDef::All(&[CostDef::PayLife(1), REMOVE])][..],
            &[CostDef::TapSource][..],
            &[SACRIFICE, REMOVE][..],
        ] {
            assert_eq!(pay(costs), Err("unsupported resolving payment cost"));
        }
        for costs in [
            &[CostDef::PayLife(1), CostDef::Energy(2)][..],
            &[SACRIFICE][..],
            &[DISCARD][..],
        ] {
            assert_eq!(pay(costs), Ok(()));
        }
        assert_eq!(
            validate_cost_program(
                EffectDef::WithCosts {
                    costs: &[REMOVE],
                    effect: &PAY_SUPPLIED,
                },
                None
            ),
            Err("unsupported resolving payment cost"),
            "a supplied parameter is held to the lane it expands in"
        );
    }

    #[test]
    fn replacement_programs_hold_payments_to_the_resolving_lanes() {
        use crate::card::{EffectPaymentDef, PlayerRefDef, PlayerSetDef};
        static PERFORM: [ReplacementEffectDef; 1] = [ReplacementEffectDef::Perform(&PAY_PARAMETER)];
        let entry = |costs: &'static [CostDef]| {
            validate_replacement_cost_program(ReplacementEffectDef::PayOr {
                payment: EffectPaymentDef::new(
                    PlayerSetDef::One(PlayerRefDef::EffectController),
                    costs,
                ),
                if_paid: &[],
                if_declined: &[],
            })
        };
        assert_eq!(entry(&[CostDef::PayLife(2)]), Ok(()));
        assert_eq!(entry(&[CostDef::Choice(SCALAR_CHOICES)]), Ok(()));
        assert_eq!(
            entry(&[CostDef::Parameter]),
            Err("unbound cost parameter"),
            "nothing encloses a replacement program to supply one"
        );
        assert_eq!(
            entry(&[CostDef::TapSource]),
            Err("unsupported resolving payment cost")
        );
        assert_eq!(
            validate_replacement_cost_program(ReplacementEffectDef::Conditional {
                condition: crate::card::ConditionDef::All(&[]),
                if_true: &[],
                if_false: &PERFORM,
            }),
            Err("unbound cost parameter"),
            "a performed effect is checked wherever the program nests it"
        );
    }

    #[test]
    fn composed_mechanic_programs_reject_unbound_and_unpayable_programs() {
        assert!(validate_cost_program(PAY_PARAMETER, None).is_err());
        assert!(
            validate_cost_program(
                EffectDef::WithCosts {
                    costs: &[CostDef::Choice(&[
                        CostDef::PayLife(1),
                        CostDef::DrawCards(1)
                    ])],
                    effect: &PAY_PARAMETER,
                },
                None
            )
            .is_err()
        );
        assert!(
            validate_cost_program(
                EffectDef::WithCosts {
                    costs: &[CostDef::Choice(SCALAR_CHOICES)],
                    effect: &PAY_PARAMETER,
                },
                None
            )
            .is_ok()
        );
        assert!(
            validate_cost_program(
                EffectDef::WithCosts {
                    costs: &[CostDef::PayLife(1)],
                    effect: &PAY_PARAMETER
                },
                None
            )
            .is_ok()
        );
        assert!(
            validate_cost_program(
                EffectDef::WithCosts {
                    costs: &[CostDef::Special("unsupported")],
                    effect: &PAY_PARAMETER
                },
                None
            )
            .is_err()
        );
    }
}
