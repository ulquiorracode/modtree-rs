# modtree-rs

[![CI](https://github.com/ulquiorracode/modtree-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/ulquiorracode/modtree-rs/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust: 2024](https://img.shields.io/badge/Language-Rust%202024-orange.svg)](https://www.rust-lang.org/)

Zero-allocation, hierarchical modular routing and tree composition
engine in Rust.

---

## Overview

Modern systems require composable architectures where units of functionality
can be grouped, nested, inspected, and executed with strict isolation and
zero runtime overhead.

`modtree` models applications, routing tables, and command hierarchies
as structured trees:

- **Compile-Time Static Branches**: Zero-cost monomorphic routing pipelines
  for built-in modules.
- **Dynamic Mount Points**: Capability-scoped runtime attachment for
  plugins, dynamic extensions, and foreign modules.
- **Strictly Bounded Traversal**: U-cycle navigation (`on_enter` descent,
  terminal execution, `on_exit` ascent) with zero-allocation short-circuiting.
- **Pluggable Orthogonality**: Clean separation between routing, state
  storage (e.g. `statefs`), and isolation containers.

## Tiered Developer Experience (DX)

`modtree` provides a layered DX architecture:

- **Tier 1 (Attribute Macros)**: Declarative Rust structs and impl blocks
  annotated with `#[modtree::module]`, `#[modtree::library]`, and
  `#[modtree::command]`. Direct function identifiers are used for hooks
  (`enter = check_auth`), preserving IDE navigation and type safety.
- **Tier 2 (Fluent Builder API)**: Type-safe builders (`ModuleNode::named`,
  `LibraryNode::named`) with the typestate pattern enforcing that
  libraries only host terminal leaves.
- **Tier 3 (Core SMA Traversal)**: Zero-allocation dispatch engine
  operating on CPU stack frames and byte slices.

### Quick Start Example

```rust
use modtree::prelude::*;

// 1. Declare a flat library grouping terminal commands
#[derive(Clone, Default)]
pub struct UsersLibrary;

#[modtree::library(
    name = "users",
    context(MyCtx),
    intent(u64),
    outcome(bool),
    error(&'static str)
)]
impl UsersLibrary {
    #[modtree::command]
    pub fn ban(ctx: &mut MyCtx, target_id: u64) -> Result<bool, &'static str> {
        ctx.ban(target_id);
        Ok(true)
    }
}

// 2. Declare a structural module mounting child libraries and modules
#[modtree::module(
    name = "admin",
    context(MyCtx),
    intent(u64),
    outcome(bool),
    error(&'static str)
)]
pub struct AdminModule {
    pub users: UsersLibrary,
}

// 3. Dispatch with zero heap allocations on the hot path
let root_node = AdminModule { users: UsersLibrary }.into_tree_node();
let res = dispatch(&root_node, &["admin", "users", "ban"], &mut ctx, 42);
assert_eq!(res, Ok(true));
```

## Core Concepts

- **Node**: Fundamental unit of the hierarchy with an associated path
  component and execution contract.
- **Module**: Structural composite node containing child modules, commands,
  or nested namespaces.
- **Plugin**: Dynamic mount node with capability constraints, sandboxing
  boundaries, and runtime lifecycle hooks.
- **Library**: Terminal container group exposing a flat collection of
  executable leaves without further branching.
- **Leaf**: Terminal execution point representing a concrete command, action,
  or handler.

## Versioning & Governance

`modtree-rs` adheres strictly to [Semantic Versioning 2.0.0](https://semver.org/).
All breaking changes, deprecations, and feature enhancements are tracked
in [CHANGELOG.md](CHANGELOG.md).

- `0.1.x`: Initial domain taxonomy and SMA U-cycle execution engine.
- `0.2.x`: Procedural macro attributes and tiered DX builders.

## Architectural Documentation

For deep technical specifications, memory layouts, traversal lifecycle
contracts, and zero-allocation invariants, see [ARCHITECTURE.md](ARCHITECTURE.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
