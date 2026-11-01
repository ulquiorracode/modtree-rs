//! # modtree
//!
//! Ultra-fast, zero-allocation hierarchical modular routing & tree composition engine in Rust.
//!
//! `modtree` models applications, routing tables, and command hierarchies as structured trees
//! executing under the Sewing Machine Architecture (SMA) U-cycle traversal contract:
//!
//! - **Descent (`on_enter`)**: Capability audits, scoped resource injection, and zero-allocation
//!   short-circuiting ([`flow::FlowControl`]).
//! - **Terminal (`on_execute`)**: Direct execution of the resolved target leaf handler.
//! - **Ascent (`on_exit`)**: Telemetry recording, auditing, output formatting, and transactional
//!   cleanup.
//!
//! ## Core Architecture
//!
//! - [`taxonomy::NodeKind`]: Structural taxonomy (`Module`, `Library`, `Plugin`, `Leaf`).
//! - [`flow::FlowControl`]: Zero-overhead descent flow control (`Proceed`, `ShortCircuit`, `Halt`).
//! - [`tree::TreeNode`]: Hierarchical node representation.
//! - [`router::dispatch`]: Zero-allocation U-cycle traversal dispatcher.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod flow;
pub mod node;
#[cfg(feature = "alloc")]
pub mod router;
pub mod taxonomy;
#[cfg(feature = "alloc")]
pub mod tree;

pub use error::RoutingError;
pub use flow::FlowControl;
pub use node::{EnterHook, ExitHook, LeafHandler, NodeDescriptor, NodeHook};
pub use taxonomy::NodeKind;

#[cfg(feature = "alloc")]
pub use router::{PathSegments, dispatch};
#[cfg(feature = "alloc")]
pub use tree::{LeafNode, LibraryNode, ModuleNode, PluginNode, TreeNode};

/// Unified prelude re-exporting key types and traits.
pub mod prelude {
    pub use crate::error::RoutingError;
    pub use crate::flow::FlowControl;
    pub use crate::node::{EnterHook, ExitHook, LeafHandler, NodeDescriptor, NodeHook};
    pub use crate::taxonomy::NodeKind;

    #[cfg(feature = "alloc")]
    pub use crate::router::{PathSegments, dispatch};
    #[cfg(feature = "alloc")]
    pub use crate::tree::{LeafNode, LibraryNode, ModuleNode, PluginNode, TreeNode};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use alloc::vec::Vec;

    #[derive(Default)]
    struct TestContext {
        visited: Vec<&'static str>,
    }

    #[test]
    fn test_taxonomy_constraints() {
        assert!(NodeKind::Module.can_have_children());
        assert!(NodeKind::Library.can_have_children());
        assert!(NodeKind::Plugin.can_have_children());
        assert!(!NodeKind::Leaf.can_have_children());

        assert!(NodeKind::Module.can_adopt(NodeKind::Leaf));
        assert!(NodeKind::Module.can_adopt(NodeKind::Module));
        assert!(NodeKind::Library.can_adopt(NodeKind::Leaf));
        assert!(!NodeKind::Library.can_adopt(NodeKind::Module));
        assert!(!NodeKind::Library.can_adopt(NodeKind::Library));
    }

    #[test]
    fn test_path_segments_iterator() {
        let path = "/admin/users/ban";
        let segments: Vec<&str> = PathSegments::new(path, '/').collect();
        assert_eq!(segments, &["admin", "users", "ban"]);

        let path2 = "root.math.add";
        let segments2: Vec<&str> = PathSegments::new(path2, '.').collect();
        assert_eq!(segments2, &["root", "math", "add"]);
    }

    #[test]
    fn test_successful_u_cycle_traversal() {
        let mut ctx = TestContext::default();

        let add_leaf: LeafNode<TestContext, (i32, i32), i32, &'static str> = LeafNode::new(
            "add",
            |_ctx: &mut TestContext, (a, b): (i32, i32)| Ok(a + b),
        )
        .with_enter(|ctx, intent| {
            ctx.visited.push("leaf_enter");
            FlowControl::Proceed(intent)
        })
        .with_exit(|ctx, _intent, outcome| {
            ctx.visited.push("leaf_exit");
            if let Ok(val) = outcome {
                *val *= 2; // Mutate outcome on ascent: (10 + 20) * 2 = 60
            }
        });

        let math_lib = LibraryNode::new("math")
            .with_leaf(add_leaf)
            .with_enter(|ctx, intent| {
                ctx.visited.push("library_enter");
                FlowControl::Proceed(intent)
            })
            .with_exit(|ctx, _intent, _outcome| {
                ctx.visited.push("library_exit");
            });

        let root = TreeNode::Module(
            ModuleNode::new("root")
                .with_child(TreeNode::Library(math_lib))
                .with_enter(|ctx, intent| {
                    ctx.visited.push("root_enter");
                    FlowControl::Proceed(intent)
                })
                .with_exit(|ctx, _intent, _outcome| {
                    ctx.visited.push("root_exit");
                }),
        );

        let res = dispatch(&root, &["root", "math", "add"], &mut ctx, (10, 20));
        assert_eq!(res, Ok(60));

        // Exact SMA U-cycle order
        assert_eq!(
            ctx.visited,
            &[
                "root_enter",
                "library_enter",
                "leaf_enter",
                "leaf_exit",
                "library_exit",
                "root_exit"
            ]
        );
    }

    #[test]
    fn test_descent_halt_short_circuiting() {
        let mut ctx = TestContext::default();

        let banned_leaf: LeafNode<TestContext, (), i32, &'static str> =
            LeafNode::new("secret", |_ctx: &mut TestContext, _intent: ()| {
                panic!("should never execute when halted in descent");
            });

        let secure_module = ModuleNode::new("secure")
            .with_child(TreeNode::Leaf(banned_leaf))
            .with_enter(|ctx, _intent| {
                ctx.visited.push("secure_enter");
                FlowControl::Halt("access_denied")
            })
            .with_exit(|ctx, _intent, _outcome| {
                ctx.visited.push("secure_exit");
            });

        let root = TreeNode::Module(
            ModuleNode::new("root")
                .with_child(TreeNode::Module(secure_module))
                .with_enter(|ctx, intent| {
                    ctx.visited.push("root_enter");
                    FlowControl::Proceed(intent)
                })
                .with_exit(|ctx, _intent, _outcome| {
                    ctx.visited.push("root_exit");
                }),
        );

        let res = dispatch(&root, &["root", "secure", "secret"], &mut ctx, ());
        assert_eq!(res, Err(RoutingError::Halted("access_denied")));

        // Halt on secure node immediately ascents through visited ancestors
        assert_eq!(
            ctx.visited,
            &["root_enter", "secure_enter", "secure_exit", "root_exit"]
        );
    }

    #[test]
    fn test_early_short_circuit() {
        let mut ctx = TestContext::default();

        let expensive_leaf: LeafNode<TestContext, (), i32, &'static str> =
            LeafNode::new("compute", |_ctx: &mut TestContext, _: ()| {
                panic!("should not compute on cache hit");
            });

        let cached_lib = LibraryNode::new("cache")
            .with_leaf(expensive_leaf)
            .with_enter(|ctx, _intent| {
                ctx.visited.push("cache_enter");
                FlowControl::ShortCircuit(42) // Early cached result
            })
            .with_exit(|ctx, _intent, _outcome| {
                ctx.visited.push("cache_exit");
            });

        let root =
            TreeNode::Module(ModuleNode::new("root").with_child(TreeNode::Library(cached_lib)));

        let res = dispatch(&root, &["root", "cache", "compute"], &mut ctx, ());
        assert_eq!(res, Ok(42));
        assert_eq!(ctx.visited, &["cache_enter", "cache_exit"]);
    }

    #[test]
    fn test_routing_errors() {
        let mut ctx = TestContext::default();
        let leaf: LeafNode<TestContext, (), &'static str, &'static str> =
            LeafNode::new("ping", |_ctx: &mut TestContext, _: ()| Ok("pong"));
        let root = TreeNode::Module(ModuleNode::new("root").with_child(TreeNode::Leaf(leaf)));

        // Empty path
        assert_eq!(
            dispatch(&root, &[], &mut ctx, ()),
            Err(RoutingError::EmptyPath)
        );

        // Node not found
        assert_eq!(
            dispatch(&root, &["root", "unknown"], &mut ctx, ()),
            Err(RoutingError::NodeNotFound)
        );

        // Expected terminal leaf (stopped on module)
        assert_eq!(
            dispatch(&root, &["root"], &mut ctx, ()),
            Err(RoutingError::ExpectedTerminalLeaf)
        );

        // Unexpected terminal leaf (path continues past leaf)
        assert_eq!(
            dispatch(&root, &["root", "ping", "extra"], &mut ctx, ()),
            Err(RoutingError::UnexpectedTerminalLeaf)
        );
    }
}
