use std::fmt;

/// Which flavour of `augment` a diagnostic came from. The two differ in
/// how their target is written (RFC 7950 §7.17), so the distinction
/// matters when reporting a bad target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AugmentKind {
    /// A top-level `augment` statement.
    Augment,
    /// An `augment` substatement of a `uses`.
    UsesAugment,
}

impl fmt::Display for AugmentKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AugmentKind::Augment => write!(f, "augment"),
            AugmentKind::UsesAugment => write!(f, "uses augment"),
        }
    }
}

/// A problem found while building an [`Entry`](crate::Entry) tree.
///
/// These are warnings rather than errors: `to_entry` still returns a
/// tree, with the offending augment skipped or its duplicate child
/// removed. They are collected on the [`YangStore`](crate::YangStore)
/// (see [`YangStore::diagnostics`](crate::YangStore::diagnostics)) so
/// the caller decides whether to log them, fail a build, or ignore
/// them — previously they were written straight to stderr, which left
/// a library deciding how an application reports its problems.
///
/// Each variant names the module whose augment is at fault, so a
/// diagnostic is actionable without re-deriving where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Diagnostic {
    /// A top-level `augment` target must be an absolute
    /// schema-node-identifier (leading `/`).
    AugmentTargetNotAbsolute { module: String, target: String },

    /// A `uses`-substatement `augment` target must be a descendant
    /// schema-node-identifier (no leading `/`).
    AugmentTargetNotDescendant { module: String, target: String },

    /// The augment target did not resolve to any node in the tree.
    /// `missing` is the first path segment that failed to match.
    AugmentTargetNotFound {
        kind: AugmentKind,
        module: String,
        target: String,
        missing: String,
    },

    /// The target resolved to a leaf or leaf-list. Data nodes and
    /// actions may only be added to a container, list, choice, case,
    /// input, output or notification.
    AugmentIntoLeaf {
        module: String,
        target: String,
        leaf: String,
    },

    /// The augment introduced a child whose name already existed in the
    /// target, so it was dropped. An augment must not shadow a node
    /// already present.
    AugmentDuplicateNode {
        module: String,
        target: String,
        name: String,
    },
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Diagnostic::AugmentTargetNotAbsolute { module, target } => write!(
                f,
                "{module}: augment target \"{target}\" must use the absolute form (leading '/')"
            ),
            Diagnostic::AugmentTargetNotDescendant { module, target } => write!(
                f,
                "{module}: uses augment target \"{target}\" must use the descendant form \
                 (no leading '/')"
            ),
            Diagnostic::AugmentTargetNotFound {
                kind,
                module,
                target,
                missing,
            } => write!(
                f,
                "{module}: {kind} target \"{target}\" not found \
                 (no node matching \"{missing}\")"
            ),
            Diagnostic::AugmentIntoLeaf {
                module,
                target,
                leaf,
            } => write!(
                f,
                "{module}: augment cannot add nodes to leaf target \"{leaf}\" ({target})"
            ),
            Diagnostic::AugmentDuplicateNode {
                module,
                target,
                name,
            } => write!(
                f,
                "{module}: augment node \"{name}\" already exists in target \"{target}\"; \
                 not added"
            ),
        }
    }
}
