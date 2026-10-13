/// One cost a resolving payment settles as written: not repeated, not
/// labeled, and not a scalar alternative. Those lanes have their own
/// procedures and their own support checks.
///
/// [`CostDef::resolving_payment_cost`] is the only way to obtain one. Catalog
/// validation rejects a cost it declines, and the payment procedure matches
/// this type exhaustively, so the two cannot disagree about what is payable.
/// Each variant keeps the name of the [`CostDef`] variant it narrows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResolvingPaymentCostDef {
    Mana(ManaCost),
    GenericMana(ValueDef),
    ColoredMana {
        color: ManaColor,
        amount: ValueDef,
    },
    ObjectManaCostReducedBy {
        object: &'static EffectRecipientDef,
        generic: u16,
    },
    PayLife(u16),
    Life(ValueDef),
    Energy(u16),
    MillCards(u16),
    DiscardCards(u16),
    /// One matching card from the payer's hand.
    Discard(ObjectPredicateDef),
    /// One matching permanent the payer controls.
    SacrificePermanent(ObjectPredicateDef),
    SacrificeCreaturesWithTotalPower(u16),
    MovePermanentMatching {
        object: ObjectPredicateDef,
        zone: ZoneKind,
    },
    ChosenGenericMana,
    ChosenEnergy,
    RemoveAnyNumberOfCounters {
        object: &'static EffectRecipientDef,
        kind: CounterKind,
    },
}

impl CostDef {
    /// Narrow a cost to the forms a resolving payment can settle as written.
    /// Composition (`Perform`, `All`, `Parameter`, `Repeated`, `Choice`) is
    /// interpreted before this is asked, so it is never one of them.
    #[must_use]
    pub(crate) const fn resolving_payment_cost(self) -> Option<ResolvingPaymentCostDef> {
        use ResolvingPaymentCostDef as Resolving;
        Some(match self {
            Self::Mana(cost) => Resolving::Mana(cost),
            Self::GenericMana(amount) => Resolving::GenericMana(amount),
            Self::ColoredMana { color, amount } => Resolving::ColoredMana { color, amount },
            Self::ObjectManaCostReducedBy { object, generic } => {
                Resolving::ObjectManaCostReducedBy { object, generic }
            }
            Self::PayLife(amount) => Resolving::PayLife(amount),
            Self::Life(amount) => Resolving::Life(amount),
            Self::Energy(amount) => Resolving::Energy(amount),
            Self::MillCards(amount) => Resolving::MillCards(amount),
            Self::DiscardCards(amount) => Resolving::DiscardCards(amount),
            Self::Discard {
                object,
                quantity: CostQuantityDef::Fixed(1),
            } => Resolving::Discard(object),
            Self::SacrificePermanent {
                object,
                controller: PlayerRelation::You,
            } => Resolving::SacrificePermanent(object),
            Self::SacrificeCreaturesWithTotalPower(total) => {
                Resolving::SacrificeCreaturesWithTotalPower(total)
            }
            Self::MovePermanentMatching { object, zone } => {
                Resolving::MovePermanentMatching { object, zone }
            }
            Self::ChosenGenericMana => Resolving::ChosenGenericMana,
            Self::ChosenEnergy => Resolving::ChosenEnergy,
            Self::RemoveAnyNumberOfCounters { object, kind } => {
                Resolving::RemoveAnyNumberOfCounters { object, kind }
            }
            _ => return None,
        })
    }
}
