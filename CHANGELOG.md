<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-11-01

### Added

- `modtree-macros`: Procedural macro crate providing high-DX attribute macros.
- `#[modtree::module]`: Attribute macro for structs creating `ModuleNode` trees.
- `#[modtree::library]`: Attribute macro for structs and `impl` blocks creating
  flat `LibraryNode` trees with terminal leaves.
- `#[modtree::command]`: Attribute marking associated functions as executable
  leaves.
- Granular attributes: `enter`, `exit`, `name`, `context`, `intent`, `outcome`,
  `error` supporting direct function identifiers without string quotes.
- `IntoTreeNode` and `AsTreeNode` conversion traits for modular composition.
- Fluent builder methods: `ModuleNode::named`, `LibraryNode::named`,
  `LeafNode::named`, `PluginNode::named`, and `ModuleNode::with_mounted`.
- Comprehensive test coverage for attribute-driven macro tree expansion.

### Changed

- Migrated root package into a Cargo workspace containing `crates/modtree-macros`.
- Enabled `macros` feature by default in the root `modtree` crate.

## [0.1.0] - 2026-11-01

### Added

- Initial release of `modtree`.
- Domain taxonomy: `NodeKind` (`Module`, `Library`, `Plugin`, `Leaf`).
- Zero-cost traversal flow control: `FlowControl` (`Proceed`, `ShortCircuit`,
  `Halt`).
- Traversal contracts: `NodeHook`, `EnterHook`, `ExitHook`, `LeafHandler`.
- Hierarchical tree nodes: `TreeNode`, `ModuleNode`, `LibraryNode`, `PluginNode`,
  `LeafNode`.
- Zero-allocation U-cycle traversal dispatcher: `router::dispatch`.
- Zero-allocation path tokenizer: `PathSegments`.
- Domain error handling: `RoutingError`.
- Criterion micro-benchmark suite measuring 4-tier traversal latency.
