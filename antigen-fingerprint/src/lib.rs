//! Fingerprint grammar parser + matcher for antigen failure-class declarations.
//!
//! See ADR-010 + Amendments 1-4 for the design substrate. This crate is the
//! workspace-internal canonical implementation; both `antigen-macros` (for
//! compile-time validation) and `antigen` (for scan-time matching) consume it.
//!
//! ## Grammar (v1)
//!
//! Fingerprints are written as raw strings inside `#[antigen(fingerprint =
//! r#"..."#)]`. The DSL is comma-separated constraints, all AND'd at the top
//! level:
//!
//! ```text
//! item = enum,
//! name = matches("*Class"),
//! variants = 3..=8,
//! has_method("meet", "(Self, Self) -> Self"),
//! attr_present("repr(u8)"),
//! doc_contains("strength"),
//! all_of([
//!     attr_present("repr(u8)"),
//!     doc_contains("strength")
//! ]),
//! any_of([item = struct, item = enum]),
//! not(name = matches("Test*"))
//! ```
//!
//! ### Operators
//!
//! - `item = <kind>` — `struct`, `enum`, `trait`, `fn`, `impl`, `type`, `mod`
//! - `name = matches("<glob>")` — `*` (any) and `?` (one) are the only
//!   metachars; case-sensitive; whole-name match
//! - `variants = M..=N` — inclusive range over enum variant count
//! - `has_method("<name>", "<signature>")` — signature pre-parsed at load time;
//!   use `"(& self, T) -> U"` (space after `&`) — `proc_macro2` renders `&self`
//!   as `& self`, so `"(&self, ...)"` silently never matches
//! - `attr_present("<path>")` — outer attribute path matches (e.g.
//!   `repr`, `clippy::panic`)
//! - `doc_contains("<substring>")` — case-sensitive substring search in the
//!   item's doc comment(s)
//! - `body_contains_macro("<name>")` — function/method body contains a macro
//!   invocation of the named macro (last path segment match)
//! - `all_of([...])` — every child matches
//! - `any_of([...])` — at least one child matches
//! - `not(<constraint>)` — child does NOT match. Per ADR-010 Amendment 3
//!   OQ3, `not` is only legal inside `all_of`, and only as a sibling of at
//!   least one positive matcher (closes the De Morgan promiscuity loophole).
//!
//! ## Performance invariants (per ADR-010 Amendment 3)
//!
//! - Single-pass walks at the consumer site
//! - Pre-normalized signatures cached as `Option<String>` at parse time
//!   (`syn::Signature`-aware comparison is the v2 upgrade path)
//! - Depth cap 10 + total node count cap 256, both at parse time
//! - Node-kind dispatch: [`Fingerprint::node_kind`] returns the required
//!   item kind (or `None` if the fingerprint is shape-only) so consumers can
//!   skip evaluating fingerprints whose top-level `item:` constraint cannot
//!   match the current AST node

use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use syn::parse::{Parse, ParseStream};

mod glob;
mod matcher;
mod parser;

pub use glob::glob_match_ident;

/// Maximum fingerprint AST depth (per ADR-010 OQ4). Configurable via
/// `[package.metadata.antigen.fingerprint_max_depth]` in future sweeps.
pub const MAX_DEPTH: usize = 10;

/// Maximum total fingerprint AST node count (per ADR-010 OQ4). Configurable
/// via `[package.metadata.antigen.fingerprint_max_nodes]` in future sweeps.
pub const MAX_NODES: usize = 256;

// ============================================================================
// Public AST
// ============================================================================

/// A parsed fingerprint — an implicit AND over its constraints.
///
/// `Fingerprint::parse` enforces:
/// - depth ≤ [`MAX_DEPTH`] (10)
/// - total node count ≤ [`MAX_NODES`] (256)
/// - `not` only appears inside `all_of`
/// - signatures inside `has_method` are pre-parsed at load time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fingerprint {
    /// Top-level constraints, all AND'd together.
    pub constraints: Vec<Constraint>,
}

/// A single constraint in a fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Constraint {
    /// `item = <kind>` — the AST node must be of this kind.
    Item(ItemKind),

    /// `name = matches("<glob>")` — the item's identifier matches the glob.
    NameMatches(GlobPattern),

    /// `variants = M..=N` — enum variant count is in this inclusive range.
    Variants(VariantRange),

    /// `has_method("<name>", "<signature>")` — there exists an `impl` method
    /// with this name AND a signature whose shape matches `signature`.
    ///
    /// Signatures are whitespace-normalized before comparison, so extra spaces
    /// collapse. However, `proc_macro2` renders `&self` as `& self` (space
    /// between `&` and `self`), so the correct pattern form is
    /// `"(& self, T) -> U"`, not `"(&self, T) -> U"`. A missing space produces
    /// a silent mismatch with no diagnostic (ATK-W6a-013).
    HasMethod(MethodPattern),

    /// `attr_present("<path>")` — at least one outer attribute on the item
    /// has a path whose last segment equals `path` (or whose full path
    /// equals `path`).
    AttrPresent(String),

    /// `doc_contains("<substring>")` — the item's doc-comment text contains
    /// `substring` (case-sensitive).
    DocContains(String),

    /// `body_contains_macro("<name>")` — the function/method body contains
    /// a macro invocation whose path's last segment equals `name`.
    BodyContainsMacro(String),

    /// `all_of([...])` — every child constraint must match.
    AllOf(Vec<Self>),

    /// `any_of([...])` — at least one child constraint must match.
    AnyOf(Vec<Self>),

    /// `not(<constraint>)` — the child must NOT match. Per ADR-010 Amendment
    /// 3 OQ3, `not` is only legal inside `all_of`, alongside at least one
    /// positive matcher; the parser rejects bare-top-level `not` and
    /// `not` directly under `any_of`.
    Not(Box<Self>),
}

/// Item kind for the `item = <kind>` operator. Mirrors the `syn::Item`
/// variants we care about.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// `struct Foo;`, `struct Foo { ... }`, `struct Foo(...);`
    Struct,
    /// `enum E { ... }`
    Enum,
    /// `trait T { ... }`
    Trait,
    /// `fn f(...) { ... }` — free function only; methods inside `impl`
    /// blocks are matched via `item = impl` + `has_method`.
    Fn,
    /// `impl Trait for Type { ... }` or `impl Type { ... }`
    Impl,
    /// `type Alias = T;`
    Type,
    /// `mod m { ... }`
    Mod,
}

impl ItemKind {
    /// Parse from the bare keyword form (`struct`, `enum`, ...).
    fn from_ident(name: &str) -> Option<Self> {
        Some(match name {
            "struct" => Self::Struct,
            "enum" => Self::Enum,
            "trait" => Self::Trait,
            "fn" => Self::Fn,
            "impl" => Self::Impl,
            "type" => Self::Type,
            "mod" => Self::Mod,
            _ => return None,
        })
    }

    /// Render the kind back to its keyword form (for error messages).
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Fn => "fn",
            Self::Impl => "impl",
            Self::Type => "type",
            Self::Mod => "mod",
        }
    }
}

/// A glob pattern with `*` and `?` metachars.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobPattern(pub String);

impl GlobPattern {
    /// Match this pattern against a string. See [`glob_match_ident`].
    #[must_use]
    pub fn matches(&self, name: &str) -> bool {
        glob_match_ident(&self.0, name)
    }
}

/// Inclusive variant-count range for the `variants = M..=N` operator.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct VariantRange {
    /// Inclusive lower bound on the enum's variant count.
    pub min: usize,
    /// Inclusive upper bound on the enum's variant count.
    pub max: usize,
}

impl VariantRange {
    /// Whether `n` is inside `[min, max]`.
    #[must_use]
    pub const fn contains(&self, n: usize) -> bool {
        n >= self.min && n <= self.max
    }
}

/// Method pattern for the `has_method("<name>", "<signature>")` operator.
///
/// The signature is pre-parsed at fingerprint-load time (per ADR-010
/// Amendment 3 Performance Invariant 2 — naive per-match-site re-parse is a
/// 50× slowdown).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodPattern {
    /// Method name to match against `impl` items.
    pub name: String,
    /// User-supplied signature shape as written (e.g.,
    /// `"(& self, Self) -> Self"`). The canonical form for serde + equality.
    pub signature: String,
    /// Whitespace-normalized form of `signature`, computed at parse time
    /// per ADR-010 Amendment 3 Performance Invariant 2 (signatures inside
    /// `has_method` are pre-parsed at fingerprint-load time, NOT re-parsed
    /// per match site — the naive per-match-site re-normalize was a
    /// documented 50× slowdown). `None` when a `MethodPattern` is
    /// deserialized via serde without going through the parser; the matcher
    /// falls back to normalizing on first use in that case.
    #[serde(skip)]
    pub normalized_signature: Option<String>,
}

impl PartialEq for MethodPattern {
    fn eq(&self, other: &Self) -> bool {
        // Equality is on the user-visible shape (name + signature string).
        // `normalized_signature` is a derived performance cache; two
        // MethodPatterns with the same `signature` are equal regardless of
        // whether either has populated the cache.
        self.name == other.name && self.signature == other.signature
    }
}

impl Eq for MethodPattern {}

/// Whitespace-normalize a string: collapse runs of whitespace into single
/// spaces. Used by both the parser (to populate
/// [`MethodPattern::normalized_signature`] at load time) and the matcher
/// (to compare on the same canonical form). Per ADR-010 Amendment 3
/// Performance Invariant 2.
#[must_use]
pub(crate) fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ============================================================================
// Parsing entry points
// ============================================================================

impl Fingerprint {
    /// Parse a fingerprint from its DSL source. The caller-provided string is
    /// the value of `#[antigen(fingerprint = r#"...")]`'s raw-string body.
    ///
    /// # Errors
    ///
    /// Returns `syn::Error` when:
    /// - the input is malformed (unrecognized operator, bad punctuation, etc.)
    /// - the AST exceeds [`MAX_DEPTH`] or [`MAX_NODES`]
    /// - `not` appears outside `all_of`, or `not` appears in `all_of` without
    ///   any positive sibling matcher
    /// - `has_method`'s signature does not parse as a `syn::Signature`-shape
    pub fn parse(source: &str) -> syn::Result<Self> {
        use std::str::FromStr;
        let tokens = TokenStream::from_str(source)
            .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e.to_string()))?;
        syn::parse2::<Self>(tokens)
    }

    /// Returns the required top-level [`ItemKind`] if this fingerprint has an
    /// `item = <kind>` constraint at top level (or inside an `all_of` whose
    /// first constraint is `item = <kind>`).
    ///
    /// Used by the scan visitor for node-kind dispatch (per ADR-010
    /// Amendment 3 Performance Invariant 4): only fingerprints whose
    /// `node_kind` matches the current AST node are evaluated.
    ///
    /// `None` means the fingerprint applies to any node kind.
    #[must_use]
    pub fn node_kind(&self) -> Option<ItemKind> {
        for c in &self.constraints {
            if let Some(k) = c.node_kind_hint() {
                return Some(k);
            }
        }
        None
    }
}

impl Constraint {
    /// Best-effort inspection: if this constraint pins the AST node to one
    /// kind at top level, return it.
    fn node_kind_hint(&self) -> Option<ItemKind> {
        match self {
            Self::Item(k) => Some(*k),
            Self::AllOf(children) => {
                for c in children {
                    if let Some(k) = c.node_kind_hint() {
                        return Some(k);
                    }
                }
                None
            }
            // any_of / not / leaf-non-kind: no kind hint.
            _ => None,
        }
    }
}

// ============================================================================
// Convenience: implement Parse so callers can use syn::parse2 directly
// ============================================================================

impl Parse for Fingerprint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let constraints = parser::parse_top_level(input)?;
        let fp = Self { constraints };
        parser::validate(&fp)?;
        Ok(fp)
    }
}
