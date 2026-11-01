//! Domain taxonomy and node classification for `modtree`.

/// Structural role of a node within the hierarchy.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    /// Structural composite node containing child modules, libraries, plugins, or leaves.
    Module = 0,

    /// Flat container group exposing a collection of executable leaves without nested branching.
    ///
    /// # Invariant
    /// A `Library` node CANNOT contain child `Module`, `Library`, or `Plugin` nodes.
    /// Its children are strictly terminal `Leaf` instances.
    Library = 1,

    /// Dynamic mount point with runtime lifecycle hooks and capability-scoped isolation.
    Plugin = 2,

    /// Terminal execution point representing a concrete command, action, or handler.
    Leaf = 3,
}

impl NodeKind {
    /// Returns `true` if this node is permitted to have children.
    #[inline(always)]
    pub const fn can_have_children(self) -> bool {
        match self {
            Self::Module | Self::Library | Self::Plugin => true,
            Self::Leaf => false,
        }
    }

    /// Returns `true` if this node allows child branch nodes (`Module`, `Library`, `Plugin`).
    ///
    /// For [`NodeKind::Library`], this returns `false` because a library is strictly restricted
    /// to flat [`NodeKind::Leaf`] children only.
    #[inline(always)]
    pub const fn allows_branch_children(self) -> bool {
        match self {
            Self::Module | Self::Plugin => true,
            Self::Library | Self::Leaf => false,
        }
    }

    /// Validates whether a parent node of this kind can host a child of `child_kind`.
    #[inline(always)]
    pub const fn can_adopt(self, child_kind: Self) -> bool {
        match self {
            Self::Module => true,
            Self::Library => matches!(child_kind, Self::Leaf),
            Self::Plugin => true,
            Self::Leaf => false,
        }
    }
}
