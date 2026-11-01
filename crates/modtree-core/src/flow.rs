//! Flow control decisions for `modtree` traversal pipelines.

/// Decision made by a node during the descent phase of the U-cycle traversal.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl<TIntent = (), TOutcome = (), E = ()> {
    /// Proceed to the next child in the traversal path, passing the (optionally enriched) intent.
    Proceed(TIntent),

    /// Short-circuit the descent with an early successful outcome (e.g., cached result).
    ///
    /// Downstream nodes and the terminal command are not executed. The outcome immediately
    /// begins the ascent phase through ancestor nodes via `on_exit`.
    ShortCircuit(TOutcome),

    /// Halt execution immediately with an explicit error/reason.
    ///
    /// Downstream nodes are not executed. The error immediately begins the ascent
    /// phase through visited ancestor nodes via `on_exit`.
    Halt(E),
}

impl<TIntent, TOutcome, E> FlowControl<TIntent, TOutcome, E> {
    /// Returns `true` if the decision is [`FlowControl::Proceed`].
    #[inline(always)]
    pub const fn is_proceed(&self) -> bool {
        matches!(self, Self::Proceed(_))
    }

    /// Returns `true` if the decision is [`FlowControl::ShortCircuit`].
    #[inline(always)]
    pub const fn is_short_circuit(&self) -> bool {
        matches!(self, Self::ShortCircuit(_))
    }

    /// Returns `true` if the decision is [`FlowControl::Halt`].
    #[inline(always)]
    pub const fn is_halt(&self) -> bool {
        matches!(self, Self::Halt(_))
    }
}
