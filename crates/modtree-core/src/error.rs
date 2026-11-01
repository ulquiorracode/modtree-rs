//! Error types for `modtree` routing and structural operations.

use core::fmt;

use crate::taxonomy::NodeKind;

/// Errors that can occur during routing, tree construction, or traversal execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingError<E = ()> {
    /// Provided route path is empty.
    EmptyPath,

    /// Node segment was not found in the current branch.
    NodeNotFound,

    /// Reached a terminal command while additional path segments remain unconsumed.
    UnexpectedTerminalCommand,

    /// Expected a terminal command at the end of the route path, but found a branch node.
    ExpectedTerminalCommand,

    /// Structural constraint violation: parent node cannot adopt the given child kind.
    InvalidChildAdoption {
        /// The kind of the parent attempting adoption.
        parent: NodeKind,
        /// The forbidden kind of the child node.
        child: NodeKind,
    },

    /// Traversal was aborted during descent by a node returning [`crate::flow::FlowControl::Halt`].
    Halted(E),
}

impl<E: fmt::Display> fmt::Display for RoutingError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "routing path is empty"),
            Self::NodeNotFound => write!(f, "route segment not found"),
            Self::UnexpectedTerminalCommand => {
                write!(
                    f,
                    "encountered terminal command before path was fully resolved"
                )
            }
            Self::ExpectedTerminalCommand => {
                write!(
                    f,
                    "path ended on a structural branch instead of a terminal command"
                )
            }
            Self::InvalidChildAdoption { parent, child } => {
                write!(
                    f,
                    "structural invariant violated: parent {:?} cannot adopt child {:?}",
                    parent, child
                )
            }
            Self::Halted(err) => write!(f, "traversal halted: {}", err),
        }
    }
}

#[cfg(feature = "std")]
impl<E: fmt::Display + fmt::Debug> std::error::Error for RoutingError<E> {}
