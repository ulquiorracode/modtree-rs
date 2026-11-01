//! Core node definitions and lifecycle hooks for `modtree`.

use crate::error::RoutingError;
use crate::flow::FlowControl;
use crate::taxonomy::NodeKind;

/// Hook executed during the descent phase of the U-cycle traversal.
pub type EnterHook<Ctx, Intent, Outcome, E> =
    fn(&mut Ctx, Intent) -> FlowControl<Intent, Outcome, E>;

/// Hook executed during the ascent phase of the U-cycle traversal.
pub type ExitHook<Ctx, Intent, Outcome, E> =
    fn(&mut Ctx, &Intent, &mut Result<Outcome, RoutingError<E>>);

/// Execution handler invoked at a terminal command.
pub type CommandHandler<Ctx, Intent, Outcome, E> = fn(&mut Ctx, Intent) -> Result<Outcome, E>;

/// Metadata describing a node in the tree hierarchy.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeDescriptor {
    /// Human-readable identifier and path segment name of the node.
    pub name: &'static str,
    /// Taxonomic classification of the node.
    pub kind: NodeKind,
}

impl NodeDescriptor {
    /// Creates a new node descriptor.
    #[inline(always)]
    pub const fn new(name: &'static str, kind: NodeKind) -> Self {
        Self { name, kind }
    }
}

/// Lifecycle contract for traversing and executing nodes within `modtree`.
pub trait NodeHook<Ctx, Intent, Outcome, E> {
    /// Descent phase: called when traversal enters this node.
    #[inline(always)]
    fn on_enter(&self, _ctx: &mut Ctx, intent: Intent) -> FlowControl<Intent, Outcome, E> {
        FlowControl::Proceed(intent)
    }

    /// Ascent phase: called when traversal rewinds back up through this node.
    #[inline(always)]
    fn on_exit(
        &self,
        _ctx: &mut Ctx,
        _intent: &Intent,
        _outcome: &mut Result<Outcome, RoutingError<E>>,
    ) {
    }
}

/// Blanket implementation of [`NodeHook`] for unit types.
impl<Ctx, Intent, Outcome, E> NodeHook<Ctx, Intent, Outcome, E> for () {}
