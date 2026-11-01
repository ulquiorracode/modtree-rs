//! Traversal and routing engine implementing the Sewing Machine Architecture (SMA).

#[cfg(feature = "alloc")]
use crate::error::RoutingError;
#[cfg(feature = "alloc")]
use crate::flow::FlowControl;
#[cfg(feature = "alloc")]
use crate::tree::{LibraryNode, ModuleNode, PluginNode, TreeNode};

/// Helper for splitting string paths into zero-allocation segment iterators.
pub struct PathSegments<'a> {
    remaining: &'a str,
    delimiter: char,
}

impl<'a> PathSegments<'a> {
    /// Creates a new path segment iterator over the given path and delimiter.
    #[inline]
    pub const fn new(path: &'a str, delimiter: char) -> Self {
        Self {
            remaining: path,
            delimiter,
        }
    }
}

impl<'a> Iterator for PathSegments<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let trimmed = self.remaining.trim_start_matches(self.delimiter);
        if trimmed.is_empty() {
            self.remaining = "";
            return None;
        }

        match trimmed.find(self.delimiter) {
            Some(idx) => {
                let segment = &trimmed[..idx];
                self.remaining = &trimmed[idx + 1..];
                Some(segment)
            }
            None => {
                let segment = trimmed;
                self.remaining = "";
                Some(segment)
            }
        }
    }
}

/// Executes a U-cycle traversal down the tree towards the terminal leaf identified by `path`.
///
/// # Traversal Phases
/// 1. **Descent**: At each ancestor node, `on_enter` is evaluated. If any node yields
///    [`FlowControl::Halt`], execution halts and ascends immediately. If [`FlowControl::ShortCircuit`]
///    is yielded, descent ceases and ascent begins with the short-circuited outcome.
/// 2. **Terminal**: The target leaf executes its handler.
/// 3. **Ascent**: Traversal rewinds up the ancestor chain, calling `on_exit` in reverse order.
#[cfg(feature = "alloc")]
pub fn dispatch<Ctx, Intent: Clone, Outcome, E>(
    root: &TreeNode<Ctx, Intent, Outcome, E>,
    path: &[&str],
    ctx: &mut Ctx,
    intent: Intent,
) -> Result<Outcome, RoutingError<E>> {
    if path.is_empty() {
        return Err(RoutingError::EmptyPath);
    }

    // If root name matches first segment, consume it; otherwise route from root's children.
    let remaining_path = if !path.is_empty() && path[0] == root.name() {
        &path[1..]
    } else {
        path
    };

    dispatch_node(root, remaining_path, ctx, intent)
}

#[cfg(feature = "alloc")]
fn dispatch_node<Ctx, Intent: Clone, Outcome, E>(
    node: &TreeNode<Ctx, Intent, Outcome, E>,
    path: &[&str],
    ctx: &mut Ctx,
    intent: Intent,
) -> Result<Outcome, RoutingError<E>> {
    // 1. Descent Phase (on_enter)
    let next_intent = match node.on_enter(ctx, intent.clone()) {
        FlowControl::Proceed(i) => i,
        FlowControl::ShortCircuit(outcome) => {
            let mut res = Ok(outcome);
            node.on_exit(ctx, &intent, &mut res);
            return res;
        }
        FlowControl::Halt(err) => {
            let mut res = Err(RoutingError::Halted(err));
            node.on_exit(ctx, &intent, &mut res);
            return res;
        }
    };

    // 2. Routing & Terminal Resolution
    let mut outcome = match node {
        TreeNode::Leaf(leaf) => {
            if !path.is_empty() {
                Err(RoutingError::UnexpectedTerminalLeaf)
            } else {
                let mut leaf_res = (leaf.handler)(ctx, next_intent).map_err(RoutingError::Halted);
                if let Some(exit_hook) = leaf.on_exit {
                    exit_hook(ctx, &intent, &mut leaf_res);
                }
                leaf_res
            }
        }
        TreeNode::Library(library) => dispatch_library(library, path, ctx, next_intent),
        TreeNode::Module(module) => dispatch_module(module, path, ctx, next_intent),
        TreeNode::Plugin(plugin) => dispatch_plugin(plugin, path, ctx, next_intent),
    };

    // 3. Ascent Phase (on_exit)
    node.on_exit(ctx, &intent, &mut outcome);

    outcome
}

#[cfg(feature = "alloc")]
fn dispatch_library<Ctx, Intent: Clone, Outcome, E>(
    library: &LibraryNode<Ctx, Intent, Outcome, E>,
    path: &[&str],
    ctx: &mut Ctx,
    intent: Intent,
) -> Result<Outcome, RoutingError<E>> {
    if path.is_empty() {
        return Err(RoutingError::ExpectedTerminalLeaf);
    }

    let target_name = path[0];
    let remaining_path = &path[1..];

    if !remaining_path.is_empty() {
        return Err(RoutingError::UnexpectedTerminalLeaf);
    }

    for leaf in &library.leaves {
        if leaf.descriptor.name == target_name {
            let leaf_intent = match leaf
                .on_enter
                .map_or(FlowControl::Proceed(intent.clone()), |f| {
                    f(ctx, intent.clone())
                }) {
                FlowControl::Proceed(i) => i,
                FlowControl::ShortCircuit(outcome) => {
                    let mut res = Ok(outcome);
                    if let Some(exit_hook) = leaf.on_exit {
                        exit_hook(ctx, &intent, &mut res);
                    }
                    return res;
                }
                FlowControl::Halt(err) => {
                    let mut res = Err(RoutingError::Halted(err));
                    if let Some(exit_hook) = leaf.on_exit {
                        exit_hook(ctx, &intent, &mut res);
                    }
                    return res;
                }
            };

            let mut leaf_res = (leaf.handler)(ctx, leaf_intent).map_err(RoutingError::Halted);
            if let Some(exit_hook) = leaf.on_exit {
                exit_hook(ctx, &intent, &mut leaf_res);
            }
            return leaf_res;
        }
    }

    Err(RoutingError::NodeNotFound)
}

#[cfg(feature = "alloc")]
fn dispatch_module<Ctx, Intent: Clone, Outcome, E>(
    module: &ModuleNode<Ctx, Intent, Outcome, E>,
    path: &[&str],
    ctx: &mut Ctx,
    intent: Intent,
) -> Result<Outcome, RoutingError<E>> {
    if path.is_empty() {
        return Err(RoutingError::ExpectedTerminalLeaf);
    }

    let target_name = path[0];
    let remaining_path = &path[1..];

    for child in &module.children {
        if child.name() == target_name {
            return dispatch_node(child, remaining_path, ctx, intent);
        }
    }

    Err(RoutingError::NodeNotFound)
}

#[cfg(feature = "alloc")]
fn dispatch_plugin<Ctx, Intent: Clone, Outcome, E>(
    plugin: &PluginNode<Ctx, Intent, Outcome, E>,
    path: &[&str],
    ctx: &mut Ctx,
    intent: Intent,
) -> Result<Outcome, RoutingError<E>> {
    if path.is_empty() {
        return Err(RoutingError::ExpectedTerminalLeaf);
    }

    // If path starts with plugin root name, route with subpath, else pass full path
    let child_path = if path[0] == plugin.root.name() {
        &path[1..]
    } else {
        path
    };

    dispatch_node(&plugin.root, child_path, ctx, intent)
}
