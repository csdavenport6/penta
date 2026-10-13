// csdavenport6/penta#60: these shapes used to pass validation and panic when
// the payment was offered.
static REMOVE_SOURCE_COUNTER: [CostDef; 1] = [CostDef::RemoveCountersFromSource {
    kind: crate::card::CounterKind::PlusOnePlusOne,
    amount: 1,
}];

fn unresolvable_payment_error() -> CatalogError {
    CatalogError::UnsupportedAbilityEffectProgramContext {
        definition: CardDefinitionId::from_uuid("00000000-0000-0000-0000-000000000001"),
        part: CardPartId::PRIMARY,
        ability: AbilityId::PRIMARY,
        context: "cost program",
        operation: "unsupported resolving payment cost",
    }
}

#[test]
fn catalog_rejects_resolving_payments_the_runtime_cannot_resolve() {
    let ability = AbilityDef::triggered(
        "At the beginning of your upkeep, you may remove a +1/+1 counter from this creature.",
        TriggerEventDef::StepBegins {
            step: TurnStepDef::Upkeep,
            player: PlayerRelation::You,
        },
        EffectDef::PayOr(PayOrDef::optional(
            &REMOVE_SOURCE_COUNTER,
            &EffectDef::None,
        )),
    );

    assert_eq!(
        error(definition_with_ability(ability)),
        unresolvable_payment_error()
    );
}

#[test]
fn catalog_rejects_entry_payments_the_runtime_cannot_resolve() {
    let ability = AbilityDef::as_enters(
        "As this creature enters, you may remove a +1/+1 counter from it.",
        ReplacementEffectDef::PayOr {
            payment: crate::card::EffectPaymentDef::new(
                PlayerSetDef::Related(PlayerRelation::You),
                &REMOVE_SOURCE_COUNTER,
            ),
            if_paid: &[],
            if_declined: &[],
        },
    );

    assert_eq!(
        error(definition_with_ability(ability)),
        unresolvable_payment_error()
    );
}

#[test]
fn catalog_rejects_replacement_performed_payments_the_runtime_cannot_resolve() {
    static PAY: EffectDef = EffectDef::PayOr(PayOrDef::optional(
        &REMOVE_SOURCE_COUNTER,
        &EffectDef::None,
    ));
    static PROGRAM: [ReplacementEffectDef; 2] = [
        ReplacementEffectDef::ReplaceEventWithNothing,
        ReplacementEffectDef::Perform(&PAY),
    ];
    let ability = AbilityDef::replacement_for(
        "If you would draw a card, you may remove a +1/+1 counter from this creature instead.",
        ReplacementEventDef::WouldDraw {
            player: PlayerRelation::You,
            during_own_draw_step: false,
            except_first_in_draw_step: false,
        },
        ReplacementEffectDef::Sequence(&PROGRAM),
    );

    assert_eq!(
        error(definition_with_ability(ability)),
        unresolvable_payment_error()
    );
}

#[test]
fn catalog_rejects_unresolvable_payments_in_effect_created_abilities() {
    static PAY: EffectDef = EffectDef::PayOr(PayOrDef::optional(
        &REMOVE_SOURCE_COUNTER,
        &EffectDef::None,
    ));
    static DELAYED: AbilityDef = AbilityDef::triggered(
        "At the beginning of your next upkeep, you may remove a +1/+1 counter from this creature.",
        TriggerEventDef::StepBegins {
            step: TurnStepDef::Upkeep,
            player: PlayerRelation::You,
        },
        PAY,
    );
    static REFLEXIVE: AbilityDef = AbilityDef::triggered(
        "When you do, you may remove a +1/+1 counter from this creature.",
        TriggerEventDef::Reflexive,
        PAY,
    );
    for effect in [
        EffectDef::InstallTrigger(InstalledTriggerDef::once(&DELAYED)),
        EffectDef::ReflexiveTrigger(&REFLEXIVE),
    ] {
        let ability = AbilityDef::triggered(
            "At the beginning of your end step, set up a later payment.",
            TriggerEventDef::StepBegins {
                step: TurnStepDef::End,
                player: PlayerRelation::You,
            },
            effect,
        );
        assert_eq!(
            error(definition_with_ability(ability)),
            unresolvable_payment_error(),
            "{effect:?}"
        );
    }
}
