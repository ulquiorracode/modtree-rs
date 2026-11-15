//! # Tier 2: Fluent Builder API for Hierarchical ModTree Routing
//!
//! Provides dedicated builder structs ([`TreeBuilder`], [`CommandBuilder`], [`BundleBuilder`])
//! isolated behind `feature = "builder"`.

#[cfg(feature = "alloc")]
pub use crate::bundle::BundleBuilder;
use crate::node::{CommandHandler, EnterHook, ExitHook};
#[cfg(feature = "alloc")]
use crate::tree::{CommandNode, IntoTreeNode, LibraryNode, ModuleNode, PluginNode, TreeNode};

/// Dedicated builder for constructing [`CommandNode`] instances.
pub struct CommandBuilder<Ctx, Intent, Outcome, E> {
    name: &'static str,
    handler: CommandHandler<Ctx, Intent, Outcome, E>,
    on_enter: Option<EnterHook<Ctx, Intent, Outcome, E>>,
    on_exit: Option<ExitHook<Ctx, Intent, Outcome, E>>,
}

impl<Ctx, Intent, Outcome, E> CommandBuilder<Ctx, Intent, Outcome, E> {
    /// Creates a new command builder.
    #[inline]
    pub const fn new(name: &'static str, handler: CommandHandler<Ctx, Intent, Outcome, E>) -> Self {
        Self {
            name,
            handler,
            on_enter: None,
            on_exit: None,
        }
    }

    /// Attaches an `on_enter` descent hook.
    #[inline]
    pub const fn on_enter(mut self, hook: EnterHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_enter = Some(hook);
        self
    }

    /// Attaches an `on_exit` ascent hook.
    #[inline]
    pub const fn on_exit(mut self, hook: ExitHook<Ctx, Intent, Outcome, E>) -> Self {
        self.on_exit = Some(hook);
        self
    }

    /// Builds the terminal [`CommandNode`].
    #[inline]
    pub fn build(self) -> CommandNode<Ctx, Intent, Outcome, E> {
        let mut node = CommandNode::new(self.name, self.handler);
        node.on_enter = self.on_enter;
        node.on_exit = self.on_exit;
        node
    }
}

/// Dedicated builder for constructing hierarchical [`ModuleNode`] routing trees.
#[cfg(feature = "alloc")]
pub struct TreeBuilder<Ctx, Intent, Outcome, E> {
    root: ModuleNode<Ctx, Intent, Outcome, E>,
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> TreeBuilder<Ctx, Intent, Outcome, E> {
    /// Creates a new tree builder rooted at a module with the specified name.
    #[inline]
    pub const fn new(root_name: &'static str) -> Self {
        Self {
            root: ModuleNode::new(root_name),
        }
    }

    /// Attaches an `on_enter` descent hook to the root module.
    #[inline]
    pub const fn on_enter(mut self, hook: EnterHook<Ctx, Intent, Outcome, E>) -> Self {
        self.root.on_enter = Some(hook);
        self
    }

    /// Attaches an `on_exit` ascent hook to the root module.
    #[inline]
    pub const fn on_exit(mut self, hook: ExitHook<Ctx, Intent, Outcome, E>) -> Self {
        self.root.on_exit = Some(hook);
        self
    }

    /// Appends a command to the root module.
    #[inline]
    pub fn command(mut self, cmd: CommandNode<Ctx, Intent, Outcome, E>) -> Self {
        self.root.add_child(TreeNode::Command(cmd));
        self
    }

    /// Appends a submodule to the root module.
    #[inline]
    pub fn module(mut self, module: ModuleNode<Ctx, Intent, Outcome, E>) -> Self {
        self.root.add_child(TreeNode::Module(module));
        self
    }

    /// Appends a library node to the root module.
    #[inline]
    pub fn library(mut self, lib: LibraryNode<Ctx, Intent, Outcome, E>) -> Self {
        self.root.add_child(TreeNode::Library(lib));
        self
    }

    /// Appends a plugin node to the root module.
    #[inline]
    pub fn plugin(mut self, plugin: PluginNode<Ctx, Intent, Outcome, E>) -> Self {
        self.root.add_child(TreeNode::Plugin(plugin));
        self
    }

    /// Mounts any item implementing [`IntoTreeNode`] into the root module.
    #[inline]
    pub fn mount<T>(mut self, item: T) -> Self
    where
        T: IntoTreeNode<Ctx, Intent, Outcome, E>,
    {
        self.root.mount(item);
        self
    }

    /// Finalizes construction and returns the root [`ModuleNode`].
    #[inline]
    pub fn build(self) -> ModuleNode<Ctx, Intent, Outcome, E> {
        self.root
    }
}
