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
//!   match the same signatures. (An earlier engine required the spaced
//!   form `"(& self, ...)"`; that requirement is gone.)
//! - `attr_present("<path>")` — outer attribute path matches (e.g.
//!   `repr`, `clippy::panic`)
//! - `doc_contains("<substring>")` — case-sensitive substring search in the
//!   item's doc comment(s)
//! - `body_contains_macro("<name>")` — function/method body contains a macro
//!   invocation of the named macro (last path segment match)
//! - `body_calls("<name>")` — function/method body contains a *call* to the
//!   named function or method: a free/path call (`foo()`, `std::process::exit()`
//!   — last path segment match) OR a method call (`.unwrap()`, `.expect()` —
//!   method-ident match). The call-shaped twin of `body_contains_macro` (ADR-040);
//!   closes the silent `.unwrap()`/`.expect()` gap a macro-only match misses.
//! - `is_async` / `is_unsafe` / `is_const` — value-less item-qualifier checks
//!   (ADR-040 G1): the item is an `async fn` / carries `unsafe` (`unsafe fn` or
//!   `unsafe impl`) / is a `const fn`. Partial-domain (like the body leaves):
//!   `Undefined` on item-classes with no locus for the qualifier (e.g. `is_async`
//!   on a `struct`), so `not(is_async)` stays sound inside `all_of`.
//! - `impl_of_trait("<name>")` — the item is an `impl <Trait> for <Type>` whose
//!   trait path's last segment equals `name` (ADR-040 G3). An inherent
//!   `impl Type {}` is `NoMatch`; a non-`impl` item is `Undefined` (partial
//!   domain). Reads one impl's own trait path — cross-item "does Type impl X
//!   anywhere" is a different question this leaf does not answer.
//! - `derives("<name>")` — `name` is in a `#[derive(...)]` list on the item
//!   (ADR-040 G1b; syntactic last-ident, no path resolution). Full-domain.
//! - `serde_arg("<name>")` — `name` is an argument in a `#[serde(...)]` attribute
//!   (ADR-040 G1b), e.g. `deny_unknown_fields`. Full-domain.
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
mod serialize;

pub use digest::{HasAttributes, ShapeNormalize, structural_digest, structural_shape_digest};
pub use glob::glob_match_ident;
pub use serialize::to_antigen_attr;

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

    /// `body_calls("<name>")` — the function/method body contains a *call* to
    /// the named function or method: a free/path call whose path's last segment
    /// equals `name` (`foo()`, `std::process::exit()`), OR a method call whose
    /// method identifier equals `name` (`.unwrap()`, `.expect()`). The
    /// call-shaped twin of [`Self::BodyContainsMacro`] (ADR-040 grammar
    /// increment); like its twin it is a partial-domain leaf — `Undefined` on an
    /// item-class with no body locus, so `not(body_calls(X))` stays sound inside
    /// `all_of`.
    BodyCalls(String),

    /// `is_async` / `is_unsafe` / `is_const` — an item-qualifier presence check
    /// (ADR-040 grammar increment, G1). A value-less leaf: it reads whether the
    /// item carries the named qualifier. Like the body leaves it is a
    /// **partial-domain** predicate — well-posed (`Match`/`NoMatch`) on the
    /// item-classes that *can* carry the qualifier (a `fn` is or isn't async;
    /// a `fn`/`impl` is or isn't unsafe), and `Undefined` on item-classes with no
    /// locus for it (a `struct` has no asyncness), so `not(is_async)` stays sound
    /// inside `all_of` (ADR-010 Amd6). See [`QualifierKind`].
    Qualifier(QualifierKind),

    /// `impl_of_trait("<name>")` — the item is an `impl <Trait> for <Type>` whose
    /// trait path's last segment equals `name` (ADR-040 grammar increment, G3).
    /// Reads ONE impl item's own trait path (syntactic last-segment, like the
    /// body leaves) — an inherent `impl Type {}` has no trait, so it is a definite
    /// `NoMatch`. A **partial-domain** leaf: `Undefined` on item-classes that are
    /// not `impl`s (a `struct` has no trait-impl locus), so `not(impl_of_trait(X))`
    /// stays sound inside `all_of`. Cross-item "does `Type` impl X *anywhere*" is
    /// a different (G4 / charter) question this leaf does NOT answer.
    ImplOfTrait(String),

    /// `derives("<name>")` — `name` appears in a `#[derive(...)]` list on the item
    /// (ADR-040 grammar increment, G1b). Syntactic **last-segment** membership: it
    /// reads the literal ident in the derive list (`Hash`, `Eq`, `Deserialize`),
    /// with NO path resolution — a user type also named `Hash` is indistinguishable
    /// at this tier (the honest false-positive the dial carries). Full-domain like
    /// [`Self::AttrPresent`]: a definite `Match`/`NoMatch` on every item (absent
    /// derive = `NoMatch`), so the anchored `not(derives(X))` form is the absence
    /// check (e.g. `derives("Hash")` + `not(derives("Eq"))`).
    Derives(String),

    /// `serde_arg("<name>")` — `name` appears as an argument in a `#[serde(...)]`
    /// attribute on the item (ADR-040 grammar increment, G1b). Reads the argument
    /// ident (`deny_unknown_fields`, `transparent`, `rename_all`), matching whether
    /// it is present regardless of any `= value`. Full-domain like
    /// [`Self::AttrPresent`]; the anchored `not(serde_arg("deny_unknown_fields"))`
    /// is the absence check the `DeserializeWithoutDenyUnknownFields` class needs.
    SerdeArg(String),

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

/// Item-qualifier kind for the value-less `is_async` / `is_unsafe` / `is_const`
/// leaves (ADR-040 grammar increment, G1).
///
/// Each names a syntactic qualifier an item may carry. The matcher reads the
/// corresponding `syn` field (`Signature.asyncness` / `unsafety` / `constness`
/// for `fn`s; `ItemImpl.unsafety` for `unsafe impl`). The *locus* — which
/// item-classes the question is well-posed on — is encoded in the matcher's
/// `qualifier_match`: a qualifier on an item-class with no place for it is
/// `Undefined`, never a vacuous `NoMatch` (ADR-010 Amd6).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QualifierKind {
    /// `is_async` — the item is an `async fn`. Locus: `fn` only.
    Async,
    /// `is_unsafe` — the item carries `unsafe` (an `unsafe fn` or an
    /// `unsafe impl`). Locus: `fn` and `impl`.
    Unsafe,
    /// `is_const` — the item is a `const fn`. Locus: `fn` only. (Distinct from
    /// the `item = const` *item-kind* check, which matches a `const NAME: T`
    /// item; this checks the `const` *qualifier* on a function.)
    Const,
}

impl QualifierKind {
    /// Parse from the bare keyword form (`is_async`, `is_unsafe`, `is_const`).
    fn from_ident(name: &str) -> Option<Self> {
        Some(match name {
            "is_async" => Self::Async,
            "is_unsafe" => Self::Unsafe,
            "is_const" => Self::Const,
            _ => return None,
        })
    }

    /// Render back to the keyword form (for error messages).
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Async => "is_async",
            Self::Unsafe => "is_unsafe",
            Self::Const => "is_const",
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
/// zero matches. (An adopter's `PanickingInDrop` was bitten by this — a
/// production footgun fixed in the engine.)
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
            },
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
