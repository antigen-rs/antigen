//! Public types for the Mucosal Boundary Family (ADR-027 + Amendment 1).
//!
//! The mucosal-boundary taxonomy models trust boundaries with **selective
//! permeability** — the data/control-flow surfaces where a system meets the
//! outside world. Per ADR-027 the biological cognate grounds the
//! TIER-CLAIM (mucosal surfaces are a distinct immune tier) + four
//! functional disciplines, NOT per-variant tissue mapping (which is
//! software-engineering scope-selection by data-flow type, not anatomy).
//!
//! This module hosts the **public** [`MucosalKind`] enum that adopters
//! supply to the three proc-macros:
//!
//! - `#[mucosal(kind, rationale)]` — declares a boundary is actively
//!   defended at this site
//! - `#[mucosal_delegate(boundary, handled_by, rationale)]` — declares the
//!   boundary discipline is delegated to a named handler that must itself
//!   carry a matching `#[mucosal(kind = ...)]`
//! - `#[mucosal_tolerant(kind, rationale, accepts, reviewed_by?, until?)]` —
//!   declares the boundary is INTENTIONALLY permitted (active tolerance, not
//!   absence of defense)
//!
//! ## Three response states (ADR-027 Amendment 1 Change 6)
//!
//! Per naturalist's biology-prediction: mucosal sites have THREE response
//! states, not two — active defense (`#[mucosal]`), active tolerance
//! (`#[mucosal_tolerant]`), and undecided (absence of any declaration). This
//! parallels ADR-016's `#[immune]` / `#[antigen_tolerance]` / undeclared
//! triad, but at the BOUNDARY tier rather than the failure-class tier.
//! Active tolerance is not absence of response — it is antigen-specific
//! Treg-mediated suppression with its own cellular machinery (oral
//! tolerance, fetal-maternal interface, commensal-microbiome tolerance).
//!
//! ## Sealed-set discipline
//!
//! `MucosalKind` is a sealed 13-variant set per ADR-001 Amendment 1 C6.
//! The inclusion criterion (ADR-027 Amendment 1 Change 1): a variant
//! belongs iff (a) it names a kind of data/control flow crossing a trust
//! boundary at runtime; (b) the data-flow type is meaningfully distinct in
//! sanitization vocabulary (no isomorphism with an existing variant); (c)
//! the boundary surfaces in a way `#[mucosal]` can attach to. Process
//! events (PR creation, CI hook firing) are NOT data-flow types — they
//! occur AROUND the boundary, not AT it (deferred to a v0.3+ sibling axis).

use serde::{Deserialize, Serialize};

// ============================================================================
// MucosalKind
// ============================================================================

/// Type of data/control flow crossing a trust boundary (ADR-027 +
/// Amendment 1).
///
/// Sealed 13-variant set. `PrBoundary` and `Import` were removed in
/// Amendment 1 (the former is a process event, not a data-flow; the latter
/// was ambiguous/redundant — in-language `use` statements do not cross
/// trust boundaries in Rust's model, and dependency intake is covered by
/// `DependencyImport`).
///
/// The axis is **type-of-data-crossing-boundary**, organized by DATA-FLOW
/// TYPE — explicitly NOT by anatomical location (per ADR-027 §Biology
/// grounding NON-NEGOTIABLE; biology grounds the tier-claim + 4 functional
/// disciplines, not per-variant tissue mapping).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MucosalKind {
    /// Inbound API request (HTTP, gRPC) carrying caller-supplied data.
    ApiRequest,
    /// Outbound API response — data leaving the trust boundary.
    ApiResponse,
    /// Model Context Protocol invocation — tool/resource call crossing
    /// into the agent's trust surface.
    McpInvocation,
    /// External hyperlink target — data flowing to/from a linked resource.
    ExternalLink,
    /// Embedded iframe — a nested document context crossing the boundary.
    Iframe,
    /// Database query — caller-influenced query construction crossing into
    /// the data layer (injection surface).
    DatabaseQuery,
    /// Cross-service call within a distributed system trust boundary.
    CrossService,
    /// Subprocess launch — argument construction crossing into an external
    /// executable (command-injection surface).
    SubprocessLaunch,
    /// Dependency intake — 3rd-party crate code crossing into the workspace
    /// (per ADR-025 supply-chain family). The cargo-dep-intake boundary,
    /// NOT in-language `use` statements.
    DependencyImport,
    /// User-supplied input at any boundary (forms, params, uploads).
    UserInput,
    /// Filesystem path construction — user-influenced path crossing into
    /// the filesystem (path-traversal surface).
    FilesystemPath,
    /// Environment variable read — env-supplied data crossing the boundary
    /// (env-injection surface).
    EnvironmentVariable,
    /// Shell argument construction — user-influenced data crossing into a
    /// shell invocation (shell-injection surface).
    ShellArgument,
}

impl MucosalKind {
    /// Kebab-case string form for CLI rendering, audit-hint detail strings,
    /// and serde round-trip.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApiRequest => "api-request",
            Self::ApiResponse => "api-response",
            Self::McpInvocation => "mcp-invocation",
            Self::ExternalLink => "external-link",
            Self::Iframe => "iframe",
            Self::DatabaseQuery => "database-query",
            Self::CrossService => "cross-service",
            Self::SubprocessLaunch => "subprocess-launch",
            Self::DependencyImport => "dependency-import",
            Self::UserInput => "user-input",
            Self::FilesystemPath => "filesystem-path",
            Self::EnvironmentVariable => "environment-variable",
            Self::ShellArgument => "shell-argument",
        }
    }

    /// Parse from a kind string. Accepts kebab-case (canonical serde form),
    /// `snake_case`, `PascalCase`, and the `MucosalKind::X` path form for
    /// parser ergonomics. Returns `None` for unknown variants.
    #[must_use]
    pub fn parse_kind(s: &str) -> Option<Self> {
        // Strip an optional `MucosalKind::` path prefix.
        let bare = s.strip_prefix("MucosalKind::").unwrap_or(s);
        match bare {
            "api-request" | "api_request" | "ApiRequest" => Some(Self::ApiRequest),
            "api-response" | "api_response" | "ApiResponse" => Some(Self::ApiResponse),
            "mcp-invocation" | "mcp_invocation" | "McpInvocation" => Some(Self::McpInvocation),
            "external-link" | "external_link" | "ExternalLink" => Some(Self::ExternalLink),
            "iframe" | "Iframe" => Some(Self::Iframe),
            "database-query" | "database_query" | "DatabaseQuery" => Some(Self::DatabaseQuery),
            "cross-service" | "cross_service" | "CrossService" => Some(Self::CrossService),
            "subprocess-launch" | "subprocess_launch" | "SubprocessLaunch" => {
                Some(Self::SubprocessLaunch)
            },
            "dependency-import" | "dependency_import" | "DependencyImport" => {
                Some(Self::DependencyImport)
            },
            "user-input" | "user_input" | "UserInput" => Some(Self::UserInput),
            "filesystem-path" | "filesystem_path" | "FilesystemPath" => Some(Self::FilesystemPath),
            "environment-variable" | "environment_variable" | "EnvironmentVariable" => {
                Some(Self::EnvironmentVariable)
            },
            "shell-argument" | "shell_argument" | "ShellArgument" => Some(Self::ShellArgument),
            _ => None,
        }
    }

    /// All 13 sealed-set variants, for exhaustive CLI listing + tests.
    #[must_use]
    pub const fn all() -> [Self; 13] {
        [
            Self::ApiRequest,
            Self::ApiResponse,
            Self::McpInvocation,
            Self::ExternalLink,
            Self::Iframe,
            Self::DatabaseQuery,
            Self::CrossService,
            Self::SubprocessLaunch,
            Self::DependencyImport,
            Self::UserInput,
            Self::FilesystemPath,
            Self::EnvironmentVariable,
            Self::ShellArgument,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mucosal_kind_str_roundtrip_all_13() {
        let all = MucosalKind::all();
        assert_eq!(all.len(), 13, "sealed set must be exactly 13 variants");
        for variant in all {
            let s = variant.as_str();
            let back = MucosalKind::parse_kind(s).expect("kebab roundtrip");
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn mucosal_kind_parses_all_forms() {
        assert_eq!(
            MucosalKind::parse_kind("MucosalKind::UserInput"),
            Some(MucosalKind::UserInput)
        );
        assert_eq!(
            MucosalKind::parse_kind("user_input"),
            Some(MucosalKind::UserInput)
        );
        assert_eq!(
            MucosalKind::parse_kind("UserInput"),
            Some(MucosalKind::UserInput)
        );
        assert_eq!(
            MucosalKind::parse_kind("user-input"),
            Some(MucosalKind::UserInput)
        );
        assert_eq!(MucosalKind::parse_kind("unknown"), None);
    }

    #[test]
    fn mucosal_kind_rejects_removed_variants() {
        // PrBoundary + Import were removed in Amendment 1.
        assert_eq!(MucosalKind::parse_kind("PrBoundary"), None);
        assert_eq!(MucosalKind::parse_kind("pr-boundary"), None);
        assert_eq!(MucosalKind::parse_kind("Import"), None);
        assert_eq!(MucosalKind::parse_kind("import"), None);
    }

    #[test]
    fn mucosal_kind_serde_kebab_case() {
        let s = serde_json::to_string(&MucosalKind::DatabaseQuery).unwrap();
        assert_eq!(s, "\"database-query\"");
        let v: MucosalKind = serde_json::from_str("\"shell-argument\"").unwrap();
        assert_eq!(v, MucosalKind::ShellArgument);
    }
}
