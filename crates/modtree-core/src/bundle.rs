//! Bundle deployment specifications and lifecycle contracts.
//!
//! A [`Bundle`] is not a tree node; it is a deployment specification / container
//! image (analogous to a Docker image or package manifest) that encapsulates
//! modules, libraries, commands, and capability policies, and installs them
//! into a root tree.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt;

#[cfg(feature = "alloc")]
use crate::tree::{ModuleNode, TreeNode};

/// Metadata descriptor identifying and describing an installable bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleDescriptor {
    /// Canonical identifier and package name of the bundle.
    pub name: &'static str,
    /// Semantic version of the bundle (e.g. "1.0.0").
    pub version: &'static str,
    /// Human-readable summary of the bundle's capabilities.
    pub description: &'static str,
    /// Author or organizational owner.
    pub author: &'static str,
}

impl BundleDescriptor {
    /// Creates a new bundle descriptor with mandatory name and version.
    #[inline]
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Self {
            name,
            version,
            description: "",
            author: "",
        }
    }

    /// Attaches an informative description to this descriptor.
    #[inline]
    pub const fn with_description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    /// Attaches author or organization metadata to this descriptor.
    #[inline]
    pub const fn with_author(mut self, author: &'static str) -> Self {
        self.author = author;
        self
    }
}

/// Errors that can occur during bundle lifecycle operations (install, uninstall).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    /// Conflict: a node with the same name already exists at the target path.
    MountConflict {
        /// Target path segment or full path where collision occurred.
        target: &'static str,
        /// Concrete architectural explanation of the conflict.
        reason: &'static str,
    },

    /// The provided mount path is malformed or empty.
    InvalidMountPath(&'static str),

    /// A required prerequisite bundle or capability is missing.
    MissingDependency {
        /// The bundle that requested the dependency.
        bundle: &'static str,
        /// The missing requirement or dependency name.
        dependency: &'static str,
    },

    /// General bundle installation failure.
    InstallationFailed(&'static str),

    /// General bundle uninstallation failure.
    UninstallationFailed(&'static str),
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MountConflict { target, reason } => {
                write!(f, "mount conflict at '{target}': {reason}")
            }
            Self::InvalidMountPath(path) => write!(f, "invalid mount path: {path}"),
            Self::MissingDependency { bundle, dependency } => {
                write!(
                    f,
                    "bundle '{bundle}' requires missing dependency '{dependency}'"
                )
            }
            Self::InstallationFailed(msg) => write!(f, "bundle installation failed: {msg}"),
            Self::UninstallationFailed(msg) => write!(f, "bundle uninstallation failed: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BundleError {}

/// Contract for declarative components packaged as an installable bundle.
#[cfg(feature = "alloc")]
pub trait Bundle<Ctx, Intent, Outcome, E> {
    /// Metadata descriptor for this bundle.
    fn descriptor(&self) -> &BundleDescriptor;

    /// Installs this bundle's subtrees and components into the target root module.
    fn install(&self, root: &mut ModuleNode<Ctx, Intent, Outcome, E>) -> Result<(), BundleError>;

    /// Optional uninstallation logic to detach components and clean up resources.
    #[inline]
    fn uninstall(
        &self,
        _root: &mut ModuleNode<Ctx, Intent, Outcome, E>,
    ) -> Result<(), BundleError> {
        Ok(())
    }
}

/// Single mount action representing a node attached at a designated hierarchy path.
#[cfg(feature = "alloc")]
pub struct BundleMount<Ctx, Intent, Outcome, E> {
    /// Target path segments (e.g. `&["admin", "security"]`).
    pub path: Vec<&'static str>,
    /// The node subtree to mount.
    pub node: TreeNode<Ctx, Intent, Outcome, E>,
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> Clone for BundleMount<Ctx, Intent, Outcome, E> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            node: self.node.clone(),
        }
    }
}

/// A statically declared or built bundle containing predefined mounts.
#[cfg(feature = "alloc")]
pub struct StaticBundle<Ctx, Intent, Outcome, E> {
    /// Descriptor metadata.
    pub descriptor: BundleDescriptor,
    /// Registered component mounts.
    pub mounts: Vec<BundleMount<Ctx, Intent, Outcome, E>>,
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> Clone for StaticBundle<Ctx, Intent, Outcome, E> {
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor.clone(),
            mounts: self.mounts.clone(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> Bundle<Ctx, Intent, Outcome, E>
    for StaticBundle<Ctx, Intent, Outcome, E>
{
    #[inline]
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn install(&self, root: &mut ModuleNode<Ctx, Intent, Outcome, E>) -> Result<(), BundleError> {
        for mount in &self.mounts {
            if mount.path.is_empty() {
                root.mount(mount.node.clone());
            } else {
                root.mount_into(&mount.path, mount.node.clone())?;
            }
        }
        Ok(())
    }
}

/// Fluent builder for constructing [`StaticBundle`] instances declaratively.
#[cfg(feature = "alloc")]
pub struct BundleBuilder<Ctx, Intent, Outcome, E> {
    descriptor: BundleDescriptor,
    mounts: Vec<BundleMount<Ctx, Intent, Outcome, E>>,
}

#[cfg(feature = "alloc")]
impl<Ctx, Intent, Outcome, E> BundleBuilder<Ctx, Intent, Outcome, E> {
    /// Initiates a new bundle builder with name and version.
    #[inline]
    pub fn new(name: &'static str, version: &'static str) -> Self {
        Self {
            descriptor: BundleDescriptor::new(name, version),
            mounts: Vec::new(),
        }
    }

    /// Attaches an informative description to the bundle.
    #[inline]
    pub fn description(mut self, description: &'static str) -> Self {
        self.descriptor.description = description;
        self
    }

    /// Attaches author information to the bundle.
    #[inline]
    pub fn author(mut self, author: &'static str) -> Self {
        self.descriptor.author = author;
        self
    }

    /// Mounts a node directly into the root level of the bundle.
    #[inline]
    pub fn mount<T>(mut self, node: T) -> Self
    where
        T: crate::tree::IntoTreeNode<Ctx, Intent, Outcome, E>,
    {
        self.mounts.push(BundleMount {
            path: Vec::new(),
            node: node.into_tree_node(),
        });
        self
    }

    /// Mounts a node into a target parent module path within the bundle.
    #[inline]
    pub fn mount_into<T>(mut self, parent_path: &[&'static str], node: T) -> Self
    where
        T: crate::tree::IntoTreeNode<Ctx, Intent, Outcome, E>,
    {
        self.mounts.push(BundleMount {
            path: parent_path.to_vec(),
            node: node.into_tree_node(),
        });
        self
    }

    /// Finalizes construction of the [`StaticBundle`].
    #[inline]
    pub fn build(self) -> StaticBundle<Ctx, Intent, Outcome, E> {
        StaticBundle {
            descriptor: self.descriptor,
            mounts: self.mounts,
        }
    }
}
