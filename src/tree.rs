//! Hierarchical tree representation and node definitions.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

use crate::error::RoutingError;
use crate::flow::FlowControl;
use crate::node::{EnterHook, ExitHook, LeafHandler, NodeDescriptor};
use crate::taxonomy::NodeKind;

/// Terminal leaf node representing a concrete command or execution point.
#[derive(Clone)]
pub struct LeafNode<Ctx, Intent, Outcome, E> {
    /// Metadata descriptor for this leaf.
    pub descriptor: NodeDescriptor,
    /// Terminal execution handler.
    pub handler: LeafHandler<Ctx, Intent, Outcome, E>,
    /// Optional hook executed during descent.
    pub on_enter: Option<EnterHook<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during ascent.
    pub on_exit: Option<ExitHook<Ctx, Intent, Outcome, E>>,
}

impl<Ctx, Intent, Outcome, E> LeafNode<Ctx, Intent, Outcome, E> {
    /// Creates a new terminal leaf node with the given name and handler.
    #[inline]
    pub const fn new(name: &'static str, handler: LeafHandler<Ctx, Intent, Outcome, E>) -> Self {
        Self {
            descriptor: NodeDescriptor::new(name, NodeKind::Leaf),
            handler,
            on_enter: None,
            on_exit: None,
        }
    }

    /// Attaches an `on_enter` descent hook to this leaf.
    #[inline]
    pub const fn with_enter(mut self, hook: EnterHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_enter = Some(hook);
        self
    }

    /// Attaches an `on_exit` ascent hook to this leaf.
    #[inline]
    pub const fn with_exit(mut self, hook: ExitHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_exit = Some(hook);
        self
    }
}

/// Flat library node containing strictly terminal leaves.
///
/// # Structural Invariant
/// A `LibraryNode` can ONLY contain child [`LeafNode`] instances. It cannot contain
/// nested `ModuleNode`, `LibraryNode`, or `PluginNode` instances.
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct LibraryNode<Ctx, Intent, Outcome, E> {
    /// Metadata descriptor for this library.
    pub descriptor: NodeDescriptor,
    /// Flat collection of terminal leaves.
    pub leaves: Vec<LeafNode<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during descent.
    pub on_enter: Option<EnterHook<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during ascent.
    pub on_exit: Option<ExitHook<Ctx, Intent, Outcome, E>>,
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> LibraryNode<Ctx, Intent, Outcome, E> {
    /// Creates a new library container.
    #[inline]
    pub const fn new(name: &'static str) -> Self {
        Self {
            descriptor: NodeDescriptor::new(name, NodeKind::Library),
            leaves: Vec::new(),
            on_enter: None,
            on_exit: None,
        }
    }

    /// Appends a terminal leaf to this library.
    #[inline]
    pub fn add_leaf(&mut self, leaf: LeafNode<Ctx, Intent, Outcome, E>) {
        self.leaves.push(leaf);
    }

    /// Builder method to add a leaf.
    #[inline]
    pub fn with_leaf(mut self, leaf: LeafNode<Ctx, Intent, Outcome, E>) -> Self {
        self.leaves.push(leaf);
        self
    }

    /// Attaches an `on_enter` descent hook to this library.
    #[inline]
    pub const fn with_enter(mut self, hook: EnterHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_enter = Some(hook);
        self
    }

    /// Attaches an `on_exit` ascent hook to this library.
    #[inline]
    pub const fn with_exit(mut self, hook: ExitHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_exit = Some(hook);
        self
    }
}

/// Dynamic plugin mount node representing an isolated subsystem or runtime module.
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct PluginNode<Ctx, Intent, Outcome, E> {
    /// Metadata descriptor for this plugin mount.
    pub descriptor: NodeDescriptor,
    /// Root node of the plugin's subtree.
    pub root: Box<TreeNode<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during descent.
    pub on_enter: Option<EnterHook<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during ascent.
    pub on_exit: Option<ExitHook<Ctx, Intent, Outcome, E>>,
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> PluginNode<Ctx, Intent, Outcome, E> {
    /// Creates a new plugin mount with an underlying subtree.
    #[inline]
    pub fn new(name: &'static str, root: TreeNode<Ctx, Intent, Outcome, E>) -> Self {
        Self {
            descriptor: NodeDescriptor::new(name, NodeKind::Plugin),
            root: Box::new(root),
            on_enter: None,
            on_exit: None,
        }
    }

    /// Attaches an `on_enter` descent hook to this plugin.
    #[inline]
    pub const fn with_enter(mut self, hook: EnterHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_enter = Some(hook);
        self
    }

    /// Attaches an `on_exit` ascent hook to this plugin.
    #[inline]
    pub const fn with_exit(mut self, hook: ExitHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_exit = Some(hook);
        self
    }
}

/// Composite structural module containing child nodes of any valid type.
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct ModuleNode<Ctx, Intent, Outcome, E> {
    /// Metadata descriptor for this module.
    pub descriptor: NodeDescriptor,
    /// Ordered child nodes.
    pub children: Vec<TreeNode<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during descent.
    pub on_enter: Option<EnterHook<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during ascent.
    pub on_exit: Option<ExitHook<Ctx, Intent, Outcome, E>>,
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> ModuleNode<Ctx, Intent, Outcome, E> {
    /// Creates a new module node.
    #[inline]
    pub const fn new(name: &'static str) -> Self {
        Self {
            descriptor: NodeDescriptor::new(name, NodeKind::Module),
            children: Vec::new(),
            on_enter: None,
            on_exit: None,
        }
    }

    /// Appends a child node to this module.
    #[inline]
    pub fn add_child(&mut self, child: TreeNode<Ctx, Intent, Outcome, E>) {
        self.children.push(child);
    }

    /// Builder method to add a child node.
    #[inline]
    pub fn with_child(mut self, child: TreeNode<Ctx, Intent, Outcome, E>) -> Self {
        self.children.push(child);
        self
    }

    /// Attaches an `on_enter` descent hook to this module.
    #[inline]
    pub const fn with_enter(mut self, hook: EnterHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_enter = Some(hook);
        self
    }

    /// Attaches an `on_exit` ascent hook to this module.
    #[inline]
    pub const fn with_exit(mut self, hook: ExitHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_exit = Some(hook);
        self
    }
}

/// Generalized node variant in the hierarchy.
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub enum TreeNode<Ctx, Intent, Outcome, E> {
    /// Structural branch module.
    Module(ModuleNode<Ctx, Intent, Outcome, E>),
    /// Flat restricted container for leaves.
    Library(LibraryNode<Ctx, Intent, Outcome, E>),
    /// Dynamic mount point.
    Plugin(PluginNode<Ctx, Intent, Outcome, E>),
    /// Terminal executable leaf.
    Leaf(LeafNode<Ctx, Intent, Outcome, E>),
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> TreeNode<Ctx, Intent, Outcome, E> {
    /// Returns the node's descriptor.
    #[inline(always)]
    pub const fn descriptor(&self) -> &NodeDescriptor {
        match self {
            Self::Module(m) => &m.descriptor,
            Self::Library(l) => &l.descriptor,
            Self::Plugin(p) => &p.descriptor,
            Self::Leaf(leaf) => &leaf.descriptor,
        }
    }

    /// Returns the name of the node.
    #[inline(always)]
    pub const fn name(&self) -> &'static str {
        self.descriptor().name
    }

    /// Returns the taxonomic kind of the node.
    #[inline(always)]
    pub const fn kind(&self) -> NodeKind {
        self.descriptor().kind
    }

    /// Executes the `on_enter` hook if present.
    #[inline]
    pub fn on_enter(&self, ctx: &mut Ctx, intent: Intent) -> FlowControl<Intent, Outcome, E> {
        let hook = match self {
            Self::Module(m) => m.on_enter,
            Self::Library(l) => l.on_enter,
            Self::Plugin(p) => p.on_enter,
            Self::Leaf(leaf) => leaf.on_enter,
        };
        match hook {
            Some(f) => f(ctx, intent),
            None => FlowControl::Proceed(intent),
        }
    }

    /// Executes the `on_exit` hook if present.
    #[inline]
    pub fn on_exit(
        &self,
        ctx: &mut Ctx,
        intent: &Intent,
        outcome: &mut Result<Outcome, RoutingError<E>>,
    ) {
        let hook = match self {
            Self::Module(m) => m.on_exit,
            Self::Library(l) => l.on_exit,
            Self::Plugin(p) => p.on_exit,
            Self::Leaf(leaf) => leaf.on_exit,
        };
        if let Some(f) = hook {
            f(ctx, intent, outcome);
        }
    }
}
