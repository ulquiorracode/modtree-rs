//! Hierarchical tree representation and node definitions.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

use crate::bundle::BundleError;
use crate::error::RoutingError;
use crate::flow::FlowControl;
use crate::node::{CommandHandler, EnterHook, ExitHook, NodeDescriptor};
use crate::taxonomy::NodeKind;

/// Terminal command node representing a concrete execution point or handler.
pub struct CommandNode<Ctx, Intent, Outcome, E> {
    /// Metadata descriptor for this command.
    pub descriptor: NodeDescriptor,
    /// Terminal execution handler.
    pub handler: CommandHandler<Ctx, Intent, Outcome, E>,
    /// Optional hook executed during descent.
    pub on_enter: Option<EnterHook<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during ascent.
    pub on_exit: Option<ExitHook<Ctx, Intent, Outcome, E>>,
}

impl<Ctx, Intent, Outcome, E> Clone for CommandNode<Ctx, Intent, Outcome, E> {
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor,
            handler: self.handler,
            on_enter: self.on_enter,
            on_exit: self.on_exit,
        }
    }
}

impl<Ctx, Intent, Outcome, E> CommandNode<Ctx, Intent, Outcome, E> {
    /// Creates a new terminal command node with the given name and handler.
    #[inline]
    pub const fn new(name: &'static str, handler: CommandHandler<Ctx, Intent, Outcome, E>) -> Self {
        Self {
            descriptor: NodeDescriptor::new(name, NodeKind::Command),
            handler,
            on_enter: None,
            on_exit: None,
        }
    }

    /// Ergonomic alias for [`CommandNode::new`].
    #[inline]
    pub const fn named(
        name: &'static str,
        handler: CommandHandler<Ctx, Intent, Outcome, E>,
    ) -> Self {
        Self::new(name, handler)
    }

    /// Returns the name of the command.
    #[inline(always)]
    pub const fn name(&self) -> &'static str {
        self.descriptor.name
    }

    /// Attaches an `on_enter` descent hook to this command.
    #[inline]
    pub const fn with_enter(mut self, hook: EnterHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_enter = Some(hook);
        self
    }

    /// Attaches an `on_exit` ascent hook to this command.
    #[inline]
    pub const fn with_exit(mut self, hook: ExitHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_exit = Some(hook);
        self
    }
}

/// Flat library node containing strictly terminal commands.
///
/// # Structural Invariant
/// A `LibraryNode` can ONLY contain child [`CommandNode`] instances. It cannot contain
/// nested `ModuleNode`, `LibraryNode`, or `PluginNode` instances.
#[cfg(feature = "alloc")]
pub struct LibraryNode<Ctx, Intent, Outcome, E> {
    /// Metadata descriptor for this library.
    pub descriptor: NodeDescriptor,
    /// Flat collection of terminal commands.
    pub commands: Vec<CommandNode<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during descent.
    pub on_enter: Option<EnterHook<Ctx, Intent, Outcome, E>>,
    /// Optional hook executed during ascent.
    pub on_exit: Option<ExitHook<Ctx, Intent, Outcome, E>>,
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> Clone for LibraryNode<Ctx, Intent, Outcome, E> {
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor,
            commands: self.commands.clone(),
            on_enter: self.on_enter,
            on_exit: self.on_exit,
        }
    }
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> LibraryNode<Ctx, Intent, Outcome, E> {
    /// Creates a new library container.
    #[inline]
    pub const fn new(name: &'static str) -> Self {
        Self {
            descriptor: NodeDescriptor::new(name, NodeKind::Library),
            commands: Vec::new(),
            on_enter: None,
            on_exit: None,
        }
    }

    /// Ergonomic alias for [`LibraryNode::new`].
    #[inline]
    pub const fn named(name: &'static str) -> Self {
        Self::new(name)
    }

    /// Returns the name of the library.
    #[inline(always)]
    pub const fn name(&self) -> &'static str {
        self.descriptor.name
    }

    /// Appends a terminal command to this library.
    #[inline]
    pub fn add_command(&mut self, cmd: CommandNode<Ctx, Intent, Outcome, E>) {
        self.commands.push(cmd);
    }

    /// Builder method to add a terminal command.
    #[inline]
    pub fn with_command(mut self, cmd: CommandNode<Ctx, Intent, Outcome, E>) -> Self {
        self.commands.push(cmd);
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
impl<Ctx, Intent, Outcome, E> Clone for PluginNode<Ctx, Intent, Outcome, E> {
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor,
            root: self.root.clone(),
            on_enter: self.on_enter,
            on_exit: self.on_exit,
        }
    }
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

    /// Ergonomic alias for [`PluginNode::new`].
    #[inline]
    pub fn named(name: &'static str, root: TreeNode<Ctx, Intent, Outcome, E>) -> Self {
        Self::new(name, root)
    }

    /// Returns the name of the plugin mount.
    #[inline(always)]
    pub const fn name(&self) -> &'static str {
        self.descriptor.name
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
impl<Ctx, Intent, Outcome, E> Clone for ModuleNode<Ctx, Intent, Outcome, E> {
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor,
            children: self.children.clone(),
            on_enter: self.on_enter,
            on_exit: self.on_exit,
        }
    }
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

    /// Ergonomic alias for [`ModuleNode::new`].
    #[inline]
    pub const fn named(name: &'static str) -> Self {
        Self::new(name)
    }

    /// Returns the name of the module.
    #[inline(always)]
    pub const fn name(&self) -> &'static str {
        self.descriptor.name
    }

    /// Appends a child node to this module.
    #[inline]
    pub fn add_child(&mut self, child: TreeNode<Ctx, Intent, Outcome, E>) {
        self.children.push(child);
    }

    /// Mounts any item implementing [`IntoTreeNode`] into this module.
    #[inline]
    pub fn mount<T>(&mut self, child: T)
    where
        T: IntoTreeNode<Ctx, Intent, Outcome, E>,
    {
        self.children.push(child.into_tree_node());
    }

    /// Builder method to add a child node.
    #[inline]
    pub fn with_child(mut self, child: TreeNode<Ctx, Intent, Outcome, E>) -> Self {
        self.children.push(child);
        self
    }

    /// Builder method to mount any item implementing [`IntoTreeNode`].
    #[inline]
    pub fn with_mounted<T>(mut self, child: T) -> Self
    where
        T: IntoTreeNode<Ctx, Intent, Outcome, E>,
    {
        self.mount(child);
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

    /// Finds a direct child by its name.
    #[inline]
    pub fn find_child_mut(&mut self, name: &str) -> Option<&mut TreeNode<Ctx, Intent, Outcome, E>> {
        self.children.iter_mut().find(|c| c.name() == name)
    }

    /// Recursively traverses a hierarchy path to locate a mutable reference to a target module.
    pub fn find_module_mut(
        &mut self,
        path: &[&str],
    ) -> Option<&mut ModuleNode<Ctx, Intent, Outcome, E>> {
        let path = if !path.is_empty() && path[0] == self.descriptor.name {
            &path[1..]
        } else {
            path
        };

        if path.is_empty() {
            return Some(self);
        }

        let target = path[0];
        let remaining = &path[1..];

        for child in &mut self.children {
            match child {
                TreeNode::Module(m) if m.name() == target => {
                    return m.find_module_mut(remaining);
                }
                _ => {}
            }
        }
        None
    }

    /// Mounts a node into a target parent module specified by `parent_path`.
    /// Intermediate modules are automatically created if they do not exist.
    pub fn mount_into(
        &mut self,
        parent_path: &[&'static str],
        node: TreeNode<Ctx, Intent, Outcome, E>,
    ) -> Result<(), BundleError> {
        let path = if !parent_path.is_empty() && parent_path[0] == self.descriptor.name {
            &parent_path[1..]
        } else {
            parent_path
        };

        if path.is_empty() {
            let name = node.name();
            if self.find_child_mut(name).is_some() {
                return Err(BundleError::MountConflict {
                    target: name,
                    reason: "child with the same name already exists in target module",
                });
            }
            self.children.push(node);
            return Ok(());
        }

        let segment = path[0];
        let remaining = &path[1..];

        let child_idx = match self.children.iter().position(|c| c.name() == segment) {
            Some(idx) => {
                if !matches!(self.children[idx], TreeNode::Module(_)) {
                    return Err(BundleError::MountConflict {
                        target: segment,
                        reason: "path traversal encountered a non-module branch",
                    });
                }
                idx
            }
            None => {
                let new_module = TreeNode::Module(ModuleNode::new(segment));
                self.children.push(new_module);
                self.children.len() - 1
            }
        };

        if let TreeNode::Module(ref mut m) = self.children[child_idx] {
            m.mount_into(remaining, node)
        } else {
            unreachable!()
        }
    }

    /// Mounts a node at an explicit path including its own terminal name.
    /// Intermediate modules are automatically created if they do not exist.
    pub fn mount_at(
        &mut self,
        full_path: &[&'static str],
        node: TreeNode<Ctx, Intent, Outcome, E>,
    ) -> Result<(), BundleError> {
        if full_path.is_empty() {
            return Err(BundleError::InvalidMountPath("empty mount path"));
        }
        let parent_path = &full_path[..full_path.len() - 1];
        self.mount_into(parent_path, node)
    }
}

/// Generalized node variant in the hierarchy.
#[cfg(feature = "alloc")]
pub enum TreeNode<Ctx, Intent, Outcome, E> {
    /// Structural branch module.
    Module(ModuleNode<Ctx, Intent, Outcome, E>),
    /// Flat restricted container for commands.
    Library(LibraryNode<Ctx, Intent, Outcome, E>),
    /// Dynamic mount point.
    Plugin(PluginNode<Ctx, Intent, Outcome, E>),
    /// Terminal executable command.
    Command(CommandNode<Ctx, Intent, Outcome, E>),
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> Clone for TreeNode<Ctx, Intent, Outcome, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Module(m) => Self::Module(m.clone()),
            Self::Library(l) => Self::Library(l.clone()),
            Self::Plugin(p) => Self::Plugin(p.clone()),
            Self::Command(cmd) => Self::Command(cmd.clone()),
        }
    }
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
            Self::Command(cmd) => &cmd.descriptor,
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
            Self::Command(cmd) => cmd.on_enter,
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
            Self::Command(cmd) => cmd.on_exit,
        };
        if let Some(f) = hook {
            f(ctx, intent, outcome);
        }
    }
}

/// Conversion trait turning a structural domain type into a [`TreeNode`].
#[cfg(feature = "alloc")]
pub trait IntoTreeNode<Ctx, Intent, Outcome, E> {
    /// Converts this item into a [`TreeNode`].
    fn into_tree_node(self) -> TreeNode<Ctx, Intent, Outcome, E>;
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> IntoTreeNode<Ctx, Intent, Outcome, E>
    for TreeNode<Ctx, Intent, Outcome, E>
{
    #[inline(always)]
    fn into_tree_node(self) -> TreeNode<Ctx, Intent, Outcome, E> {
        self
    }
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> IntoTreeNode<Ctx, Intent, Outcome, E>
    for ModuleNode<Ctx, Intent, Outcome, E>
{
    #[inline(always)]
    fn into_tree_node(self) -> TreeNode<Ctx, Intent, Outcome, E> {
        TreeNode::Module(self)
    }
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> IntoTreeNode<Ctx, Intent, Outcome, E>
    for LibraryNode<Ctx, Intent, Outcome, E>
{
    #[inline(always)]
    fn into_tree_node(self) -> TreeNode<Ctx, Intent, Outcome, E> {
        TreeNode::Library(self)
    }
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> IntoTreeNode<Ctx, Intent, Outcome, E>
    for PluginNode<Ctx, Intent, Outcome, E>
{
    #[inline(always)]
    fn into_tree_node(self) -> TreeNode<Ctx, Intent, Outcome, E> {
        TreeNode::Plugin(self)
    }
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> IntoTreeNode<Ctx, Intent, Outcome, E>
    for CommandNode<Ctx, Intent, Outcome, E>
{
    #[inline(always)]
    fn into_tree_node(self) -> TreeNode<Ctx, Intent, Outcome, E> {
        TreeNode::Command(self)
    }
}

/// Reference-based trait constructing a [`TreeNode`] representation.
#[cfg(feature = "alloc")]
pub trait AsTreeNode<Ctx, Intent, Outcome, E> {
    /// Constructs a [`TreeNode`] from this reference.
    fn as_tree_node(&self) -> TreeNode<Ctx, Intent, Outcome, E>;
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E, T> AsTreeNode<Ctx, Intent, Outcome, E> for T
where
    T: Clone + IntoTreeNode<Ctx, Intent, Outcome, E>,
{
    #[inline(always)]
    fn as_tree_node(&self) -> TreeNode<Ctx, Intent, Outcome, E> {
        self.clone().into_tree_node()
    }
}
