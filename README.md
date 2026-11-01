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
