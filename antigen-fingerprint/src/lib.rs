//! Fingerprint grammar parser + matcher for antigen failure-class declarations.
//!
//! This crate is the canonical implementation of the structural-pattern DSL
//! that appears inside `#[antigen(fingerprint = r#"..."#)]`. Both
//! `antigen-macros` (compile-time validation) and the `antigen` library
//! (scan-time matching) consume it as a workspace dependency.
//!
//! Most users interact with the fingerprint grammar through the
//! [`#[antigen]`](https://docs.rs/antigen-macros) macro rather than this
//! crate's API directly. The user-facing reference is
//! [`docs/fingerprint-grammar.md`](https://github.com/antigen-rs/antigen/blob/main/docs/fingerprint-grammar.md);
//! this module-level documentation is the implementer's-view summary.
//!
//! See ADR-010 + Amendments 1-4 for the design substrate.
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
//! has_method("meet", "(self, Self) -> Self"),
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
//! - `has_method("<name>", "<signature>")` — signature canonicalized at load
//!   time via `proc_macro2` round-trip. User-natural Rust syntax works:
//!   `"(&mut self, T) -> U"`, `"(& mut self, T) -> U"`, and
//!   `"(&  mut  self, T)  ->  U"` all canonicalize to the same form and
//!   match the same signatures. (Pre-A3.5 the engine required the spaced
//!   form `"(& self, ...)"` — that warning is now obsolete.)
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
//!
//! ## Two distinct "fingerprint" concepts (do not conflate)
//!
//! - **[`Fingerprint`]** (this module's headline) — a failure-class *pattern*
//!   that recognizes a shape across many items.
//! - **[`digest::structural_digest`]** — a single item's *content hash*, used
//!   as the `signed_against_fingerprint` / `current_fingerprint` value in the
//!   substrate-witness sign/audit cycle (ADR-019). It changes when the item's
//!   code drifts. See [`digest`] for the stability contract.

use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use syn::parse::{Parse, ParseStream};

pub mod digest;
mod glob;
mod matcher;
mod parser;

pub use digest::{structural_digest, HasAttributes};
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
    /// Signatures are canonicalized at fingerprint-load time via
    /// `proc_macro2`'s tokenizer, so user-natural Rust syntax works:
    /// `"(&mut self, T) -> U"` and `"(& mut self, T) -> U"` and
    /// `"(&  mut  self,  T)  ->  U"` all canonicalize to the same form and
    /// match the same set of signatures. ATK-W6a-013 / ATK-W6a-013b
    /// document the contract.
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
    /// `const NAME: T = ...;` — free-standing const item.
    Const,
    /// `static NAME: T = ...;` — free-standing static item.
    Static,
    /// `union Name { ... }` — C-like union for unsafe memory reinterpretation.
    Union,
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
            "const" => Self::Const,
            "static" => Self::Static,
            "union" => Self::Union,
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
            Self::Const => "const",
            Self::Static => "static",
            Self::Union => "union",
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

/// Canonicalize a `has_method` signature pattern through `proc_macro2`'s
/// tokenizer so user-written `&self` / `&mut self` matches the spacing
/// that `proc_macro2` produces when rendering the actual signature.
///
/// **The bug this fixes**: the matcher renders `syn::Signature` via
/// `to_token_stream().to_string()`, which produces `"& self"` and
/// `"& mut self"` (space between `&` and the next token). When a user
/// writes the natural-Rust shape `"(&mut self)"`, plain whitespace
/// normalization leaves it as `"(&mut self)"` — and the string compare
/// against the matcher's `"(& mut self)"` never matches. Silent failure,
/// zero matches. (Tambear's `PanickingInDrop` was bitten by this — fixed at
/// tambear commit 7d9664a; A3.5 onboarding sweep surfaced it as a
/// production footgun worth fixing in the engine.)
///
/// **The fix**: round-trip the user-provided string through
/// `proc_macro2::TokenStream::from_str(_).to_string()`. `proc_macro2` inserts
/// canonical spacing around `&`, `<`, `>`, etc. — the same spacing the
/// matcher produces — so the round-trip is idempotent and canonicalizing.
/// `"(&mut self)"` → `"(& mut self)"`; `"(& mut self)"` → `"(& mut self)"`.
///
/// **Strict failure on un-tokenizable input** (Amendment 5 OQ1 STRICT
/// ratification): when `proc_macro2` cannot tokenize the input (unbalanced
/// parens, unterminated string, raw backtick, etc.), this function returns
/// `None`. Callers MUST surface the failure:
/// - The parser path (`parse_has_method`) maps `None` to a `syn::Error`
///   with the signature literal's span so the user sees a fingerprint
///   parse error pointing at the malformed signature string.
/// - The matcher path (serde-deserialized `MethodPattern` with
///   `normalized_signature: None`) returns `false` from `has_matching_method`
///   on `None` — a malformed pattern cannot produce a canonical form that
///   matches any real signature, so "never matches" is the structurally
///   honest answer at the match layer.
///
/// The pre-Amendment-5 lenient fallback (silently fall back to plain
/// `normalize_ws` on the raw input) produced asymmetric normalization
/// against the strict-tokenized match-site path — exactly the spacing bug
/// this canonicalization exists to eliminate. Strict failure is the only
/// shape consistent with the symmetric-canonicalization invariant.
#[must_use]
pub(crate) fn normalize_signature_canonical(sig: &str) -> Option<String> {
    use std::str::FromStr;
    let stream = proc_macro2::TokenStream::from_str(sig).ok()?;
    Some(normalize_ws(&stream.to_string()))
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
