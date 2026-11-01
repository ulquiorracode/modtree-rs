<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Bundle` deployment specification and package system in `crates/modtree-core`:
  - `Bundle` trait (`descriptor`, `install`, `uninstall`).
  - `BundleDescriptor` containing package metadata (`name`, `version`,
    `description`, `author`).
  - `BundleError` for lifecycle failures (`MountConflict`, `InvalidMountPath`,
    `MissingDependency`, `InstallationFailed`).
  - `StaticBundle` and `BundleBuilder` fluent builder for programmatic
    bundle construction.
  - Path-routing and automated tree synthesis: `ModuleNode::mount_into` and
    `mount_at` with fast-fail conflict checking.
- Procedural macro attribute `#[modtree::bundle]` in `crates/modtree-macros`:
  - Declarative bundle definitions on structs.
  - Declarative field mount targets via `#[modtree::mount("sub/path")]`.
  - Package metadata attributes: `#[modtree::version]`,
    `#[modtree::description]`, `#[modtree::author]`.
- End-to-end integration tests for macro-based and builder-based bundle
  installation in `tests/bundle_integration.rs`.

## [0.2.0] - 2026-11-01

### Added

- `crates/modtree-core`: Extracted core `#![no_std]` zero-allocation routing
  engine into a dedicated crate with optional `alloc` and `std` features.
- `crates/modtree-macros`: Procedural macro crate providing high-DX attribute macros.
- `#[modtree::module]`: Attribute macro for structs creating `ModuleNode` trees.
- `#[modtree::library]`: Attribute macro for structs and `impl` blocks creating
  flat `LibraryNode` trees containing terminal commands.
- `#[modtree::command]`: Attribute marking associated functions as executable
  commands.
- Granular attributes: `enter`, `exit`, `name`, `context`, `intent`, `outcome`,
  `error` supporting direct function identifiers without string quotes.
- `IntoTreeNode` and `AsTreeNode` conversion traits for modular composition.
- Fluent builder methods: `ModuleNode::named`, `LibraryNode::named`,
  `CommandNode::named`, `PluginNode::named`, and `ModuleNode::with_mounted`.
- Unified SMA foundation: Adopted `stitch-core` as canonical dependency for
  `FlowControl`, eliminating duplicate definitions and standardizing U-cycle
  contracts across the ecosystem.
- End-to-end integration tests for macro tree expansion in `tests/macro_integration.rs`.

### Changed

- **Breaking**: Renamed `Leaf` nomenclature to `Command` across the entire codebase:
  - `NodeKind::Leaf` $\to$ `NodeKind::Command`.
  - `LeafNode` $\to$ `CommandNode`.
  - `LeafHandler` $\to$ `CommandHandler`.
  - `TreeNode::Leaf` $\to$ `TreeNode::Command`.
  - `LibraryNode::leaves` $\to$ `commands`, `add_leaf` $\to$ `add_command`,
    and `with_leaf` $\to$ `with_command`.
  - `RoutingError::UnexpectedTerminalLeaf` $\to$ `RoutingError::UnexpectedTerminalCommand`.
  - `RoutingError::ExpectedTerminalLeaf` $\to$ `RoutingError::ExpectedTerminalCommand`.
- Restructured workspace layout into canonical multi-crate workspace:
  - Root `modtree` crate as an ergonomic facade.
  - `crates/modtree-core` as `#![no_std]` core.
  - `crates/modtree-macros` as proc-macro crate.
- Enabled `macros` and `std` features by default in the root `modtree` facade.

## [0.1.0] - 2026-11-01

### Added

- Initial release of `modtree`.
- Domain taxonomy: `NodeKind` (`Module`, `Library`, `Plugin`, `Command`).
- Zero-cost traversal flow control: `FlowControl` (`Proceed`, `ShortCircuit`,
  `Halt`).
- Traversal contracts: `NodeHook`, `EnterHook`, `ExitHook`, `CommandHandler`.
- Hierarchical tree nodes: `TreeNode`, `ModuleNode`, `LibraryNode`, `PluginNode`,
  `CommandNode`.
- Zero-allocation U-cycle traversal dispatcher: `router::dispatch`.
- Zero-allocation path tokenizer: `PathSegments`.
- Domain error handling: `RoutingError`.
- Criterion micro-benchmark suite measuring 4-tier traversal latency.
