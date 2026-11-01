//! Ultra-fast zero-allocation hierarchical modular routing & tree composition engine in Rust.
//!
//! `modtree` provides a high-performance hierarchical routing model inspired by the Sewing Machine
//! Architecture (SMA). It models trees of `Module`, `Library`, `Plugin`, and `Command` nodes, enabling
//! deterministic, compile-time and runtime composed command dispatch.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

// Allows `::modtree` path references to resolve within this crate's own tests.
extern crate self as modtree;

pub use modtree_core::*;

#[cfg(feature = "macros")]
pub use modtree_macros::*;

/// Convenient prelude module containing all core traits, macros, and node constructors.
pub mod prelude {
    pub use modtree_core::prelude::*;
    #[cfg(feature = "macros")]
    pub use modtree_macros::*;
}
