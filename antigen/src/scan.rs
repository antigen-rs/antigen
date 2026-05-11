//! Source-AST scanner for antigen declarations.
//!
//! Walks Rust source files in a workspace, extracts `#[antigen]`, `#[presents]`,
//! `#[immune]`, and `#[descended_from]` attribute invocations, and produces a
//! [`ScanReport`] suitable for further analysis or rendering.
//!
//! This module is the engine behind `cargo antigen scan`. It's also exposed for
//! custom integrations (e.g., a project's own CI harness, IDE plugins, or
//! programmatic audit tooling).
//!
//! ## Status (v0.1.0-rc.1)
//!
//! Initial implementation. Discovers attribute invocations, matches presentations
//! against immunities at the same item level, synthesizes fingerprint matches
//! against unmarked code (W6a), and collects `#[descended_from]` lineage edges
//! with cycle detection (A3 D1+D2). Future versions will add:
//!
//! - `#[descended_from]` propagation (synthesizing inherited presentations on
//!   descendants) — lineage edges + cycle/depth guards already land in
//!   [`ScanReport::lineage_edges`] (A3 D1+D2); the propagation step depends on
//!   the ADR-005 sub-clause F ruling on inherited-witness re-verification
//! - Cross-crate antigen declaration discovery (A3 D3)
//! - Witness validation (delegating to clippy/kani/proptest as appropriate)
//! - Performance optimizations (incremental scan, parallel file walks)
//!
//! ## Known v1 limitations
//!
//! 1. **Witness validation is presence-only at scan time** — the scan records
//!    the witness identifier verbatim. Validity classification (`Test`,
//!    `Proptest`, `PhantomType`, `Function`, `External`) and tier mapping
//!    (`Reachability`, `Execution`, `FormalProof`) are the [`crate::audit`]
//!    module's job (shipped in W7 per ADR-001 Amendment 1 + ADR-013).
//!
//! W3 (sweep A2) replaced the prior 20-line proximity heuristic in
//! [`ScanReport::unaddressed_presentations`] with structural item-identity
//! matching via [`ItemTarget`] + [`ItemTarget::addresses`]. See those types.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use syn::parse::Parse;
use syn::visit::Visit;
use walkdir::WalkDir;

// ============================================================================
// Scan-side attribute argument parsers
//
// These mirror the proc-macro-side parsers in antigen-macros but live in a
// regular (non-proc-macro) crate so they can be used at scan time. Both must
// produce identical results for the same input — the canonical representation
// is `syn::LitStr::value()`, which correctly unescapes string literal content.
//
// The proc-macro crate cannot be re-exported as a library (proc-macro = true
// crates export only their macro entry points), so we duplicate the parsing
// logic here. Any change to the attribute grammar must be reflected in both.
// ============================================================================

/// Scan-time parse of `#[antigen(name = "...", fingerprint = "...", ...)]`.
struct ScanAntigenArgs {
    name: String,
    fingerprint: Option<String>,
    family: Option<String>,
    summary: Option<String>,
}

impl Parse for ScanAntigenArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Ident, LitStr, Token};
        let mut name: Option<String> = None;
        let mut fingerprint: Option<String> = None;
        let mut family: Option<String> = None;
        let mut summary: Option<String> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name = Some(lit.value());
                }
                "fingerprint" => {
                    let lit: LitStr = input.parse()?;
                    fingerprint = Some(lit.value());
                }
                "family" => {
                    let lit: LitStr = input.parse()?;
                    family = Some(lit.value());
                }
                "summary" => {
                    let lit: LitStr = input.parse()?;
                    summary = Some(lit.value());
                }
                "references" => {
                    // Consume the array without storing (not used in scan output yet).
                    let _arr: syn::ExprArray = input.parse()?;
                }
                _ => {
                    // Unknown field: consume the value expression and continue.
                    let _: syn::Expr = input.parse()?;
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            name: name.unwrap_or_default(),
            fingerprint,
            family,
            summary,
        })
    }
}

/// Scan-time parse of `#[immune(AntigenType, witness = expr, ...)]`.
struct ScanImmuneArgs {
    antigen_type: String,
    witness: String,
}

impl Parse for ScanImmuneArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Ident, Path, Token};
        // First token is the antigen path.
        let antigen_path: Path = input.parse()?;
        let antigen_type = antigen_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let mut witness = String::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let val: syn::Expr = input.parse()?;
            if key == "witness" {
                // Render the witness expression as its token string — this is the
                // identifier or path the user wrote, e.g. `my_test_fn` or
                // `clippy::no_panic_in_drop`. We use `quote::ToTokens` to get
                // a canonical rendering without depending on string heuristics.
                use quote::ToTokens;
                witness = val.to_token_stream().to_string();
            }
            // rationale and other fields: consume silently.
        }

        Ok(Self {
            antigen_type,
            witness,
        })
    }
}

/// Scan-time parse of `#[antigen_tolerance(antigen, rationale = "...",
/// until = "...", see = [...])]`. Per ADR-011.
struct ScanToleranceArgs {
    antigen_type: String,
    rationale: String,
    until: Option<String>,
    see: Vec<String>,
}

impl Parse for ScanToleranceArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, Lit, LitStr, Path, Token};
        let antigen_path: Path = input.parse()?;
        let antigen_type = antigen_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let mut rationale = String::new();
        let mut until: Option<String> = None;
        let mut see: Vec<String> = Vec::new();

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale = lit.value();
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until = Some(lit.value());
                }
                "see" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = elem
                        {
                            see.push(s.value());
                        }
                    }
                }
                _ => {
                    // Unknown field: consume silently (forward-compat per
                    // ADR-009 adoption-gradient tolerance).
                    let _: Expr = input.parse()?;
                }
            }
        }

        Ok(Self {
            antigen_type,
            rationale,
            until,
            see,
        })
    }
}

/// A single antigen declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigenDeclaration {
    /// The kebab-case antigen name from `#[antigen(name = "...")]`.
    pub name: String,
    /// The Rust type name the attribute is applied to (e.g., `PanickingInDrop`).
    pub type_name: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number of the antigen attribute.
    pub line: usize,
    /// Optional family classification (e.g., "boundary-violation").
    pub family: Option<String>,
    /// Optional human-readable summary.
    pub summary: Option<String>,
    /// Optional fingerprint string in the [`antigen_fingerprint`] grammar
    /// (ADR-010, W6a). Parsed at scan time during the synthesis pass to
    /// emit synthetic [`Presentation`] records for unmarked items that
    /// match the structural pattern. `None` for antigens declared without
    /// a fingerprint (Layer 1 minimum-viable form, ADR-009).
    pub fingerprint: Option<String>,
    /// Canonical declaration site of this antigen, in the
    /// `"<crate-name>@<version>"` form (e.g., `"serde@1.0.193"`).
    ///
    /// ADR-017 (canonical declaration site identity). `None` for
    /// intra-workspace declarations — the default for the workspace-only
    /// scan path. Set by the cargo-metadata-driven `--include-deps`
    /// pipeline after scanning a dependency crate root. The full identity
    /// tuple at the cross-crate boundary is `(type_name, canonical_path)`.
    #[serde(default)]
    pub canonical_path: Option<String>,
}

/// Identity of the Rust item that an antigen-related attribute is applied to.
///
/// W3 (sweep A2): replaces the old proximity heuristic in
/// `unaddressed_presentations` with structural matching. `Presentation` and
/// `Immunity` carry an `item_target` that names the *item they live on*; two
/// declarations address each other if and only if their item targets are
/// equal (and they're in the same file and reference the same antigen).
///
/// The variants mirror the visitor entry points:
/// - `Struct`, `Enum`, `Trait`: top-level type declarations
/// - `Fn`: a free function
/// - `Impl`: an `impl ... for ...` or inherent `impl ...` block
/// - `ImplFn`: a method inside an impl block (with its enclosing impl
///   target captured so two methods named `drop` on different types
///   don't collide)
/// - `Unknown`: visitor fallback for shapes we don't yet model (e.g.,
///   free constants); kept rather than asserted so scans never panic on
///   third-party code with novel item shapes.
///
/// `trait_path` on `Impl`/`ImplFn` is the trait being implemented (e.g.,
/// `Drop` from `impl Drop for X`); `None` for inherent impls. The path is
/// captured as a string after canonical rendering — full-path equality is
/// W3's invariant, but A3 cross-crate matching may need richer
/// representation later.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemTarget {
    /// A top-level struct declaration. Holds the struct identifier.
    Struct(String),
    /// A top-level enum declaration. Holds the enum identifier.
    Enum(String),
    /// A top-level trait declaration. Holds the trait identifier.
    Trait(String),
    /// A free function. Holds the function identifier.
    Fn(String),
    /// A type alias declaration (`type Foo = ...;`). Holds the alias
    /// identifier. ATK-W3-005: without this, type aliases fall back to
    /// `Unknown`, and two unrelated Unknown items collide on equality.
    TypeAlias(String),
    /// An `impl ... for ...` or inherent `impl ...` block. The trait
    /// portion is `None` for inherent impls and `Some(rendered_path)`
    /// otherwise. Methods inside the impl carry an `ImplFn` target that
    /// references the same `target_type` and `trait_path`.
    Impl {
        /// The trait being implemented, rendered to its canonical token
        /// string. `None` for inherent impls (no trait).
        trait_path: Option<String>,
        /// The implementing type, rendered to its canonical token string.
        target_type: String,
    },
    /// A method inside an impl block. `target_type` and `trait_path`
    /// mirror the enclosing `Impl` target so that two methods with the
    /// same name on different types do not collide for matching purposes.
    ImplFn {
        /// Trait of the enclosing impl, if any.
        trait_path: Option<String>,
        /// Type of the enclosing impl.
        target_type: String,
        /// The method name.
        fn_name: String,
    },
    /// A method declared inside a `trait` definition. Pairs with
    /// `ImplFn { trait_path: Some(trait_name), fn_name, .. }` — the
    /// presents-on-trait-method + immune-on-impl-method pattern is one
    /// of the W3 README's adversarial cases. Holds the trait name and
    /// method name; matching bridges `TraitFn` ↔ `ImplFn` explicitly.
    TraitFn {
        /// The enclosing trait identifier.
        trait_name: String,
        /// The method name.
        fn_name: String,
    },
    /// Visitor fallback for shapes we don't yet model (e.g., free
    /// constants, modules with attribute-bearing macro-expansion).
    /// Kept rather than asserted so scans never panic on third-party
    /// code with novel item shapes. Carries the source line so that two
    /// Unknown items at different positions are not falsely equal —
    /// ATK-W3-005 caught the previous unit-variant form colliding on
    /// equality across unrelated items. The line is a best-effort
    /// discriminator; perfect identity for unhandled shapes requires
    /// per-shape visitor methods (deferred).
    Unknown {
        /// Best-effort line number; used as a tie-breaker for equality.
        line: usize,
    },
}

impl ItemTarget {
    /// Best-effort short name for diagnostic output. Not used for matching.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Struct(n) | Self::Enum(n) | Self::Trait(n) | Self::Fn(n) | Self::TypeAlias(n) => {
                n.clone()
            }
            Self::Impl {
                trait_path: Some(t),
                target_type,
            } => format!("impl {t} for {target_type}"),
            Self::Impl {
                trait_path: None,
                target_type,
            } => format!("impl {target_type}"),
            Self::ImplFn {
                trait_path: Some(t),
                target_type,
                fn_name,
            } => format!("<{target_type} as {t}>::{fn_name}"),
            Self::ImplFn {
                trait_path: None,
                target_type,
                fn_name,
            } => format!("{target_type}::{fn_name}"),
            Self::TraitFn {
                trait_name,
                fn_name,
            } => format!("trait {trait_name}::{fn_name}"),
            Self::Unknown { line } => format!("<unknown at line {line}>"),
        }
    }

    /// Whether this item-target addresses another for the purposes of the
    /// presents+immune match. The relation is reflexive and symmetric.
    ///
    /// W3 (sweep A2) — the "addresses" relation is wider than strict
    /// equality, per the A2 README's matching rules:
    ///
    /// - Same kind, same name (Struct/Enum/Trait/Fn/TypeAlias) → match.
    /// - Two `Impl` blocks for the same base type (regardless of trait
    ///   being implemented) → match. Generics are normalised away so
    ///   `Container<T>` and `Container<i32>` share a base type.
    /// - Two `ImplFn` items on the same base type with the same method
    ///   name → match (regardless of whether the impls implement the
    ///   same trait).
    /// - `TraitFn(T, f)` ↔ `ImplFn { trait_path: Some(T), fn_name: f, .. }`
    ///   → match. Handles the README's case (a): presents on a trait
    ///   method, immune on the impl method.
    /// - `Unknown` never matches anything — never a false negative on
    ///   unclassified items (per ATK-W3-005's premise).
    /// - Heterogeneous variants don't match.
    ///
    /// The relaxation is intentional: false positives in the matcher
    /// surface as unaddressed presentations the user can investigate;
    /// false negatives silently green-light a vulnerability. Err on the
    /// side of matching legitimate presents+immune pairings.
    #[must_use]
    #[allow(
        clippy::match_same_arms,
        reason = "the explicit `Unknown` arm is the load-bearing invariant — \
                  Unknown items must NEVER match anything, including each other. \
                  Keeping it explicit (even though it duplicates the `_` wildcard's \
                  body) makes the invariant readable and refactor-safe."
    )]
    pub fn addresses(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown { .. }, _) | (_, Self::Unknown { .. }) => false,
            (Self::Struct(a), Self::Struct(b))
            | (Self::Enum(a), Self::Enum(b))
            | (Self::Trait(a), Self::Trait(b))
            | (Self::Fn(a), Self::Fn(b))
            | (Self::TypeAlias(a), Self::TypeAlias(b)) => a == b,
            (
                Self::Impl {
                    target_type: t1, ..
                },
                Self::Impl {
                    target_type: t2, ..
                },
            ) => normalize_type_name(t1) == normalize_type_name(t2),
            (
                Self::ImplFn {
                    target_type: t1,
                    fn_name: f1,
                    ..
                },
                Self::ImplFn {
                    target_type: t2,
                    fn_name: f2,
                    ..
                },
            ) => normalize_type_name(t1) == normalize_type_name(t2) && f1 == f2,
            (
                Self::TraitFn {
                    trait_name,
                    fn_name: tf,
                },
                Self::ImplFn {
                    trait_path: Some(t),
                    fn_name: imf,
                    ..
                },
            )
            | (
                Self::ImplFn {
                    trait_path: Some(t),
                    fn_name: imf,
                    ..
                },
                Self::TraitFn {
                    trait_name,
                    fn_name: tf,
                },
            ) => trait_name == t && tf == imf,
            (
                Self::TraitFn {
                    trait_name: t1,
                    fn_name: f1,
                },
                Self::TraitFn {
                    trait_name: t2,
                    fn_name: f2,
                },
            ) => t1 == t2 && f1 == f2,
            _ => false,
        }
    }
}

/// Strip generic parameters from a `quote::ToTokens`-rendered type name.
/// `"Container < T >"` → `"Container"`. Used for impl-block matching so
/// that `impl<T> Container<T>` and `impl Container<i32>` share an
/// addressable identity at the type level.
fn normalize_type_name(rendered: &str) -> String {
    let s = rendered.trim();
    s.find('<')
        .map_or_else(|| s.to_string(), |idx| s[..idx].trim().to_string())
}

/// How a [`Presentation`] was discovered.
///
/// Per ADR-001 Amendment 1 Change 2 (the 5-state matrix): explicit
/// `#[presents]` markers and synthetic fingerprint matches share the
/// `Presentation` shape but differ in provenance. Audit and CLI output
/// distinguish the two — passive (synthetic) matches are the structural
/// surface ADR-010's recognition-not-yet-marked half exposes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    /// `#[presents(X)]` was written on this item.
    #[default]
    ExplicitMarker,
    /// The item was not marked but matches an antigen's fingerprint per
    /// ADR-010. Surfaced by the synthesis pass after explicit collection.
    FingerprintMatch,
}

/// Provenance entry: the identity of one ancestor antigen whose
/// presentations propagated to a descendant via `#[descended_from]`.
///
/// ADR-018 (propagation semantics). Each [`Presentation`] inherited via
/// the lineage walk carries one [`ProvenanceEntry`] per transitive
/// ancestor it inherited from. The entry fully identifies the ancestor
/// via the same `(antigen_type, canonical_path)` tuple that
/// [`unaddressed_presentations`](ScanReport::unaddressed_presentations)
/// uses for antigen identity.
///
/// `Ord` is derived so a `BTreeSet<ProvenanceEntry>` can be used
/// internally during propagation for O(log n) set-union; the serialised
/// form is `Vec<ProvenanceEntry>` for JSON schema stability.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    /// Antigen type name at the ancestor declaration site.
    pub antigen_type: String,
    /// Crate identity (`"<crate-name>@<version>"`) where the ancestor
    /// antigen was originally declared. `None` if the ancestor is
    /// intra-workspace.
    pub canonical_path: Option<String>,
}

/// A `#[presents(antigen_type)]` declaration or synthetic fingerprint match
/// discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    /// The antigen type referenced (last path segment, e.g., `PanickingInDrop`).
    pub antigen_type: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated (impl, fn, struct, etc.).
    pub item_kind: String,
    /// Item identity for structural matching against `Immunity`. W3 (sweep A2).
    pub item_target: ItemTarget,
    /// How this presentation was discovered: explicit marker vs fingerprint
    /// match. W6a (sweep A2). Defaults to `ExplicitMarker` for backwards
    /// compatibility with serialized reports from before W6a.
    #[serde(default)]
    pub match_kind: MatchKind,
    /// Canonical declaration site of the *antigen* referenced by this
    /// presentation (not the presentation's own location). ADR-017.
    /// `None` for intra-workspace antigens; `Some("<crate>@<version>")`
    /// for cross-crate antigens (set by the `--include-deps` driver
    /// after scanning the dependency crate root).
    #[serde(default)]
    pub canonical_path: Option<String>,
    /// Provenance chain of ancestor antigens this presentation was
    /// inherited from. ADR-018 (propagation semantics).
    ///
    /// - `None` = direct presentation (explicit marker or fingerprint match).
    /// - `Some(chain)` = synthesized via the propagation walk; the chain
    ///   names every transitive ancestor antigen whose presentation
    ///   propagated here (set-union across diamond paths). Empty `Vec`
    ///   inside `Some` is forbidden — normalised to `None` at construction.
    ///
    /// Audit emits a warn-level diagnostic for presentations with
    /// `inherited_from = Some(_)` that lack a re-attested immunity or
    /// tolerance on the descendant site (state 7 of the 7-state matrix).
    #[serde(default)]
    pub inherited_from: Option<Vec<ProvenanceEntry>>,
}

/// An `#[immune(antigen_type, witness = ...)]` declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Immunity {
    /// The antigen type referenced.
    pub antigen_type: String,
    /// The witness expression as a string (validated lazily).
    pub witness: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural matching against `Presentation`. W3 (sweep A2).
    pub item_target: ItemTarget,
    /// Canonical declaration site of the *antigen* referenced by this
    /// immunity claim (not where the immunity is declared). ADR-017.
    /// `None` for intra-workspace antigens.
    #[serde(default)]
    pub canonical_path: Option<String>,
}

/// An `#[antigen_tolerance(antigen, rationale = "...", until = "...", see = [...])]`
/// declaration discovered in source. Per ADR-011.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    /// The antigen type referenced (last path segment).
    pub antigen_type: String,
    /// The rationale string from the macro args (required, non-empty per
    /// ADR-011).
    pub rationale: String,
    /// Optional expiry tag (e.g., `"v1.0"`); `None` for forever-tolerance.
    pub until: Option<String>,
    /// Optional open-vocabulary references list (mirrors ADR-009's `references`
    /// field shape).
    pub see: Vec<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural matching against fingerprint matches.
    pub item_target: ItemTarget,
    /// Canonical declaration site of the *antigen* this tolerance
    /// addresses. ADR-017. `None` for intra-workspace antigens.
    #[serde(default)]
    pub canonical_path: Option<String>,
}

/// A file that failed to parse during a scan, with the associated error.
///
/// Serializes as `{"file": "...", "error": "..."}` — named fields, consistent
/// with every other collection in [`ScanReport`]. (`Vec<(PathBuf, String)>`
/// would serialize as positional JSON arrays, breaking JSON consumers.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseFailure {
    /// Path to the file that failed.
    pub file: PathBuf,
    /// Human-readable parse error.
    pub error: String,
}

/// A `#[descended_from(parent)]` lineage edge discovered during scan.
///
/// A3 (sweep) — every `#[descended_from]` site contributes one edge with
/// `child` = the bearing antigen type's name and `parent` = the last segment
/// of the path supplied as the attribute argument. Edges are collected during
/// the visitor pass and consumed afterwards by:
///
/// - cycle detection (ATK-A3-002 — required safety guard before propagation)
/// - the propagation walk (ADR-013 — child inherits parent's presentations)
/// - [`ScanReport::orphaned_lineage_edges`] (ATK-A3-003 — semantic warning
///   parallel to [`ScanReport::orphaned_tolerances`] for declarations whose
///   parent is no longer present in the scan)
///
/// `#[descended_from]` is meaningful only on antigen-type declarations
/// (unit `struct` and class-shaped `enum`). The visitor surfaces other
/// placements as `parse_failures`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    /// Bare type name of the antigen bearing `#[descended_from]` (the child).
    pub child: String,
    /// Last path segment of the `#[descended_from]` argument (the parent
    /// antigen type), stored as the bare type name. Cross-crate identity
    /// at the parent endpoint lives in [`Self::parent_canonical_path`].
    pub parent: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number of the `#[descended_from]` attribute.
    pub line: usize,
    /// Canonical declaration site of the *parent* antigen (the
    /// `#[descended_from]` argument), `"<crate-name>@<version>"`.
    /// ADR-017. `None` for intra-workspace ancestors.
    ///
    /// Two `parent_canonical_path` fields make cross-crate lineage edges
    /// first-class: an intra-workspace child can declare descent from a
    /// cross-crate parent, or vice-versa. The full lineage edge identity
    /// is `(child, parent, child_canonical_path, parent_canonical_path)`.
    #[serde(default)]
    pub parent_canonical_path: Option<String>,
    /// Canonical declaration site of the *child* antigen (the bearer of
    /// `#[descended_from]`). ADR-017. `None` for intra-workspace.
    #[serde(default)]
    pub child_canonical_path: Option<String>,
}

/// Aggregate result of scanning a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanReport {
    /// All discovered antigen declarations.
    pub antigens: Vec<AntigenDeclaration>,
    /// All discovered `#[presents]` sites + synthetic fingerprint matches.
    /// Distinguish the two via [`Presentation::match_kind`].
    pub presentations: Vec<Presentation>,
    /// All discovered `#[immune]` sites.
    pub immunities: Vec<Immunity>,
    /// All discovered `#[antigen_tolerance]` sites. W6a (sweep A2).
    #[serde(default)]
    pub tolerances: Vec<Toleration>,
    /// All discovered `#[descended_from]` edges. A3.
    ///
    /// `#[serde(default)]` so reports serialized before A3 deserialize
    /// cleanly with an empty edge list (additive change, not breaking).
    #[serde(default)]
    pub lineage_edges: Vec<LineageEdge>,
    /// Files scanned successfully.
    pub files_scanned: usize,
    /// Files that failed to parse.
    pub parse_failures: Vec<ParseFailure>,
}

/// A presentation that has no matching immunity declaration on the same item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaddressedPresentation {
    /// The presentation itself.
    pub presentation: Presentation,
    /// True if the antigen referenced is found in the scan report.
    pub antigen_known: bool,
}

impl ScanReport {
    /// Find presentations that lack a corresponding immunity declaration.
    ///
    /// W3 (sweep A2) — structural item-identity matching. A `Presentation`
    /// and an `Immunity` "address each other" when:
    ///
    /// - they reference the same `antigen_type`, AND
    /// - they're in the same source file, AND
    /// - their `item_target` values are equal (i.e., they're applied to
    ///   the same Rust item).
    ///
    /// This replaces the pre-W3 20-line proximity heuristic, which produced
    /// false positives in multi-impl files (immunity on `impl X` matched
    /// presentation on `impl Y` if their attributes happened to be within
    /// 20 lines) and false negatives when long doc-comments separated paired
    /// declarations on the same item.
    ///
    /// Cross-file matching remains out of scope here — different items can
    /// share names across modules, and the structural identity of an
    /// "item" extends to its containing module path. That's A3 territory
    /// (cross-crate scan + `#[descended_from]` propagation).
    #[must_use]
    pub fn unaddressed_presentations(&self) -> Vec<UnaddressedPresentation> {
        let known_antigens: std::collections::HashSet<&str> =
            self.antigens.iter().map(|a| a.type_name.as_str()).collect();

        let mut result = Vec::new();
        for p in &self.presentations {
            let has_matching_immunity = self.immunities.iter().any(|i| {
                i.antigen_type == p.antigen_type
                    && i.file == p.file
                    && i.item_target.addresses(&p.item_target)
            });
            // W6a: tolerance acknowledges a presentation per ADR-011
            // §Mechanics. A site with `#[antigen_tolerance(X, ...)]` for
            // the same antigen on the same item is reported under
            // "tolerated", not "unaddressed".
            let has_matching_tolerance = self.tolerances.iter().any(|t| {
                t.antigen_type == p.antigen_type
                    && t.file == p.file
                    && t.item_target.addresses(&p.item_target)
            });
            if !has_matching_immunity && !has_matching_tolerance {
                result.push(UnaddressedPresentation {
                    presentation: p.clone(),
                    antigen_known: known_antigens.contains(p.antigen_type.as_str()),
                });
            }
        }
        result
    }

    /// Tolerances whose named antigen is no longer declared in the scanned
    /// workspace. Per ADR-011 §Mechanics + ATK-A2-009 (the stale-tolerance
    /// orphan check, naturalist's biology cognate "peripheral suppression
    /// continuing after the antigen it suppressed is no longer present").
    ///
    /// Cross-crate antigens are deferred to A3 — for v0.1, an "orphan" is a
    /// tolerance whose antigen `type_name` doesn't appear in any
    /// `AntigenDeclaration` in the same scan. Consumers using cross-crate
    /// antigens may produce false positives here; that's the recognized
    /// v0.1 limitation.
    #[must_use]
    pub fn orphaned_tolerances(&self) -> Vec<&Toleration> {
        let known: std::collections::HashSet<&str> =
            self.antigens.iter().map(|a| a.type_name.as_str()).collect();
        self.tolerances
            .iter()
            .filter(|t| !known.contains(t.antigen_type.as_str()))
            .collect()
    }

    /// Lineage edges whose parent antigen is not present in the scan.
    ///
    /// A3 / ATK-A3-003 — parallel to [`ScanReport::orphaned_tolerances`].
    ///
    /// A `#[descended_from(Parent)]` declaration whose `Parent` is no
    /// longer declared in the scanned workspace (rename, removal, or
    /// — for v0.1 — a parent that lives in a not-yet-scanned crate) is
    /// a *semantic warning*, not a structural error: the scan completed
    /// correctly, but the declaration references something that isn't
    /// there. Surfaced via this query method rather than emitted into
    /// `parse_failures` so callers (CLI, audit tooling, IDE plugins)
    /// choose the severity, the same channel discipline used for
    /// orphaned tolerances.
    ///
    /// Cross-crate antigens are deferred to A3+ — for v0.1, an "orphan"
    /// is a lineage edge whose `parent` doesn't appear as a `type_name`
    /// in any [`AntigenDeclaration`] in the same scan. Consumers using
    /// cross-crate antigens may produce false positives here; that's
    /// the recognized v0.1 limitation.
    ///
    /// See also [`ScanReport::dangling_child_lineage_edges`] for the dual case
    /// (child missing rather than parent missing).
    #[must_use]
    pub fn orphaned_lineage_edges(&self) -> Vec<&LineageEdge> {
        let known: std::collections::HashSet<&str> =
            self.antigens.iter().map(|a| a.type_name.as_str()).collect();
        self.lineage_edges
            .iter()
            .filter(|e| !known.contains(e.parent.as_str()))
            .collect()
    }

    /// Lineage edges whose CHILD has no [`AntigenDeclaration`] in the scan.
    ///
    /// BUG-A3-002 fix (adversarial 2026-05-09). The dual of
    /// [`ScanReport::orphaned_lineage_edges`] — `orphaned` checks the
    /// parent endpoint, `dangling` checks the child endpoint.
    ///
    /// A struct or enum bearing `#[descended_from(Parent)]` *without* its
    /// own `#[antigen]` declaration is structurally incoherent: it claims
    /// to inherit into the antigen system without being a participant
    /// itself. The propagation walk (D1.5) cannot meaningfully attach
    /// inherited presentations to a non-antigen child — the descendant
    /// has no record in [`ScanReport::antigens`] for inheritance to flow
    /// into.
    ///
    /// Surfaced as a *semantic warning*, not a `parse_failure` — the
    /// declaration is structurally well-formed; only the relationship
    /// to the antigen registry is missing. Caller (CLI, audit tooling)
    /// chooses severity, mirroring the `orphaned_tolerances` /
    /// `orphaned_lineage_edges` channel discipline.
    ///
    /// The propagation walk skips edges flagged by this query the same
    /// way it skips edges flagged by `orphaned_lineage_edges`.
    #[must_use]
    pub fn dangling_child_lineage_edges(&self) -> Vec<&LineageEdge> {
        let known: std::collections::HashSet<&str> =
            self.antigens.iter().map(|a| a.type_name.as_str()).collect();
        self.lineage_edges
            .iter()
            .filter(|e| !known.contains(e.child.as_str()))
            .collect()
    }

    /// Total count of antigen-related declarations found.
    #[must_use]
    pub fn total_declarations(&self) -> usize {
        self.antigens.len()
            + self.presentations.len()
            + self.immunities.len()
            + self.tolerances.len()
    }
}

/// Hard depth limit for `#[descended_from]` lineage chains.
///
/// ADR-005 Amendment 3 (crash-resistance) — bounds pathological-linear
/// chains that exceed reasonable inheritance depth. Default 64; longer chains
/// surface as `parse_failures` rather than letting the propagation walk
/// recurse without bound. The limit is a sibling guard to cycle detection;
/// both are required entry conditions before propagation.
///
/// The constant is internal for v0.1; per the scope-lock document, it will
/// become configurable via `[package.metadata.antigen]` in a follow-up.
pub(crate) const MAX_LINEAGE_DEPTH: usize = 64;

/// Deduplicate lineage edges by the ADR-018 four-tuple key and emit one
/// [`ParseFailure`] per collapsed duplicate group. BUG-A3-001 fix +
/// ADR-018 §"Edge-level dedup".
///
/// The dedup key is `(child, parent, child_canonical_path,
/// parent_canonical_path)`. Same-name edges at different
/// `canonical_path` values are structurally distinct and NOT duplicates
/// (a workspace depending on `foo@1.0.0::P` and `foo@2.0.0::P`
/// legitimately has both edges).
///
/// Two `#[descended_from(B)]` attributes on the same struct `A` produce
/// two identical `LineageEdge` entries. Without this pre-pass the DFS
/// in [`detect_lineage_failures`] would silently swallow the second one
/// (black-skip path), so duplicates would never reach the user. Per
/// ADR-004 implicit-to-explicit elevation, dedup surfaces collapsed
/// duplicates as explicit diagnostics on the `parse_failures` channel.
///
/// Returns the deduped edge `Vec` and the failure list. Both
/// [`detect_lineage_failures`] (cycle/depth detection) AND the
/// propagation walk (D1.5 commit 4) consume the deduped output —
/// dedup is structurally upstream of both per ADR-018 §"Implementation
/// order in `scan_workspace`".
fn dedupe_lineage_edges(edges: &[LineageEdge]) -> (Vec<LineageEdge>, Vec<ParseFailure>) {
    use std::collections::{HashMap, HashSet};

    // Four-tuple key: (child, parent, child_canonical_path, parent_canonical_path).
    // Borrow the inner string values; the lifetime of the returned
    // Vec<LineageEdge> is independent (we clone on insert).
    type DedupKey<'a> = (&'a str, &'a str, Option<&'a str>, Option<&'a str>);
    fn key_of(edge: &LineageEdge) -> DedupKey<'_> {
        (
            edge.child.as_str(),
            edge.parent.as_str(),
            edge.child_canonical_path.as_deref(),
            edge.parent_canonical_path.as_deref(),
        )
    }

    let mut counts: HashMap<DedupKey<'_>, usize> = HashMap::new();
    for edge in edges {
        *counts.entry(key_of(edge)).or_insert(0) += 1;
    }

    // Walk edges in source order: emit the first occurrence per key into
    // the deduped slice, flag duplicates as parse_failures (one per
    // duplicate group, anchored at the first occurrence).
    let mut emitted: HashSet<DedupKey<'_>> = HashSet::new();
    let mut deduped: Vec<LineageEdge> = Vec::with_capacity(edges.len());
    let mut failures: Vec<ParseFailure> = Vec::new();
    for edge in edges {
        let key = key_of(edge);
        let count = counts.get(&key).copied().unwrap_or(0);
        if emitted.insert(key) {
            deduped.push(edge.clone());
            if count > 1 {
                failures.push(ParseFailure {
                    file: edge.file.clone(),
                    error: format!(
                        "duplicate #[descended_from({})] declarations on `{}` \
                         (first at line {}); structural lies surface as \
                         diagnostics rather than being silently collapsed \
                         (ADR-004 implicit-to-explicit elevation)",
                        edge.parent, edge.child, edge.line
                    ),
                });
            }
        }
    }
    (deduped, failures)
}

/// Detect circular and over-deep `#[descended_from]` chains.
///
/// ATK-A3-002. Iterative DFS with white/gray/black coloring on the lineage
/// graph (`child → parent` edges). Stack frames carry `(node, child_index)`
/// so the algorithm is iterative — no recursion → no stack-overflow risk on
/// pathological inputs.
///
/// Coloring discipline:
/// - **white** (absent from `color`): not yet visited.
/// - **gray** (`= 1`): on the current DFS path. Re-encountering a gray node
///   closes a cycle.
/// - **black** (`= 2`): fully processed. Re-encountering a black node is a
///   shortcut — its subtree was already proven cycle-free in this scan.
///
/// Returns one [`ParseFailure`] per discovered cycle (cycle anchored at the
/// first edge that closed it) and one per chain that exceeded `max_depth`.
/// The chain text is preserved in the `error` string — the structured-enum
/// representation of `ParseFailure` is an open question (see scope-lock §5
/// and aristotle's pending Phase 1-8 ruling).
fn detect_lineage_failures(edges: &[LineageEdge], max_depth: usize) -> Vec<ParseFailure> {
    use std::collections::HashMap;

    // BUG-A3-001 + ADR-018 §"Edge-level dedup": this function ASSUMES edges
    // are already deduped (caller invariant). `scan_workspace` runs
    // `dedupe_lineage_edges()` before calling here; unit-test callers that
    // pass raw edges with duplicates may observe silent black-skip on the
    // dup pair — that's by design at this layer. The dedup contract is
    // tested separately against `dedupe_lineage_edges` directly.
    let mut failures: Vec<ParseFailure> = Vec::new();

    // Build adjacency: child → list of (parent, edge-index). The edge-index
    // lets us recover the source location (file + line) of the closing edge
    // when a cycle is reported, which matters for human-readable diagnostics.
    let mut adjacency: HashMap<&str, Vec<(&str, usize)>> = HashMap::new();
    for (idx, edge) in edges.iter().enumerate() {
        adjacency
            .entry(edge.child.as_str())
            .or_default()
            .push((edge.parent.as_str(), idx));
    }

    let mut color: HashMap<&str, u8> = HashMap::new();
    // Seen-cycle set keyed by the canonicalised cycle (smallest rotation of
    // the node sequence) so we don't report the same loop multiple times
    // when entered from different start nodes.
    let mut reported_cycles: std::collections::HashSet<Vec<String>> =
        std::collections::HashSet::new();

    // For deterministic output (tests, diff stability) iterate roots in the
    // order edges were discovered rather than HashMap iteration order.
    let mut roots_in_order: Vec<&str> = Vec::new();
    let mut seen_roots: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for edge in edges {
        let c = edge.child.as_str();
        if seen_roots.insert(c) {
            roots_in_order.push(c);
        }
    }

    for &root in &roots_in_order {
        if color.contains_key(root) {
            continue;
        }
        // Stack frame: (node, next-child-index, file-of-edge-into-node).
        // The path vector is maintained alongside so cycles can render the
        // full chain text on closure. file-of-edge is `None` for the root.
        let mut stack: Vec<(&str, usize)> = Vec::new();
        let mut path: Vec<&str> = Vec::new();

        color.insert(root, 1);
        stack.push((root, 0));
        path.push(root);

        while let Some(&mut (node, ref mut idx)) = stack.last_mut() {
            // Hard depth guard — per ADR-005 Amendment 3 sibling to cycle
            // detection. Path length includes the current node, so a chain
            // a -> b -> c at this frame has path.len() == 3.
            if path.len() > max_depth {
                // Anchor the diagnostic at the edge that pushed us over —
                // the most recent edge in the path.
                let leaf = *path.last().unwrap_or(&node);
                let anchor = adjacency
                    .get(leaf)
                    .and_then(|v| v.first())
                    .and_then(|(_, edge_idx)| edges.get(*edge_idx))
                    .map_or_else(PathBuf::new, |e| e.file.clone());
                failures.push(ParseFailure {
                    file: anchor,
                    error: format!(
                        "#[descended_from] chain exceeds maximum depth ({max_depth}) at \
                         `{leaf}`; chain: {}",
                        path.join(" -> ")
                    ),
                });
                // Mark the leaf black and pop so the rest of the graph is
                // still examined for other failures.
                color.insert(node, 2);
                stack.pop();
                path.pop();
                continue;
            }

            let children = adjacency.get(node).map_or(&[][..], Vec::as_slice);
            if *idx >= children.len() {
                // All children processed — paint black and unwind one level.
                color.insert(node, 2);
                stack.pop();
                path.pop();
                continue;
            }

            let (child, edge_idx) = children[*idx];
            *idx += 1;

            match color.get(child).copied().unwrap_or(0) {
                0 => {
                    // White — descend into it.
                    color.insert(child, 1);
                    path.push(child);
                    stack.push((child, 0));
                }
                1 => {
                    // Gray — closing a cycle. Capture the chain from the
                    // first occurrence of `child` in `path` to the current
                    // node, then back to `child`.
                    let cycle_start = path.iter().position(|n| *n == child).unwrap_or(0);
                    let bare_refs: Vec<&str> = path[cycle_start..].to_vec();
                    let mut cycle_chain: Vec<String> =
                        bare_refs.iter().map(|s| (*s).to_string()).collect();
                    cycle_chain.push(child.to_string());

                    // Canonicalise (smallest rotation of the bare cycle,
                    // excluding the duplicated tail) for dedup.
                    let canonical = canonicalise_cycle(&bare_refs);
                    if reported_cycles.insert(canonical) {
                        let edge = edges.get(edge_idx);
                        let file = edge.map_or_else(PathBuf::new, |e| e.file.clone());
                        let line = edge.map_or(0, |e| e.line);
                        failures.push(ParseFailure {
                            file,
                            error: format!(
                                "#[descended_from] forms a cycle (closing edge at line \
                                 {line}): {}",
                                cycle_chain.join(" -> ")
                            ),
                        });
                    }
                    // Don't descend into the gray child — that would loop.
                    // Continue with the next child of `node`.
                }
                _ => {
                    // Black — already proven cycle-free in this scan; skip.
                }
            }
        }
    }

    failures
}

/// Canonicalise a cycle as the lexicographically smallest rotation of its
/// node sequence, so cycles entered from different start nodes deduplicate.
///
/// Input is the bare cycle `[a, b, c]` (without the repeated tail node) —
/// `[a, b, c]` and `[b, c, a]` are the same cycle and produce the same
/// canonical form `[a, b, c]` here.
fn canonicalise_cycle(bare: &[&str]) -> Vec<String> {
    if bare.is_empty() {
        return Vec::new();
    }
    let n = bare.len();
    let mut best_start = 0;
    for start in 1..n {
        // Compare rotation starting at `start` vs current best.
        for i in 0..n {
            let a = bare[(start + i) % n];
            let b = bare[(best_start + i) % n];
            if a < b {
                best_start = start;
                break;
            } else if a > b {
                break;
            }
        }
    }
    (0..n)
        .map(|i| bare[(best_start + i) % n].to_string())
        .collect()
}

/// Scan a directory tree, reading every `.rs` file and extracting antigen
/// declarations.
///
/// `excluded_dirs` is a list of directory names (not full paths) to skip during
/// the walk; the default is `["target", ".git", "node_modules"]` if `None` is
/// passed.
///
/// # Errors
///
/// Currently never returns `Err` — IO errors during the walk (unreadable
/// files, permission denied, etc.) are silently skipped, and parse errors
/// are recorded in `ScanReport::parse_failures` rather than aborting the
/// scan. The `std::io::Result` return type reserves space for future
/// failure modes (e.g., a `--strict` mode that fails the walk on the first
/// unreadable file, or an out-of-memory cap on `parsed_files` cache size).
/// Callers should treat any `Err` as a hard scan failure and surface the
/// error to the user.
pub fn scan_workspace(root: &Path, excluded_dirs: Option<&[&str]>) -> std::io::Result<ScanReport> {
    let default_exclusions = ["target", ".git", "node_modules"];
    let exclusions = excluded_dirs.unwrap_or(&default_exclusions);

    let mut report = ScanReport::default();

    // Cache parsed files between pass 1 (collect explicit declarations) and
    // pass 2 (synthesize fingerprint matches) to avoid re-parsing every .rs.
    let mut parsed_files: Vec<(PathBuf, syn::File)> = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !exclusions.iter().any(|x| *x == name)
            } else {
                true
            }
        })
    {
        let Ok(entry) = entry else { continue };

        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };

        match syn::parse_file(&content) {
            Ok(file) => {
                let file_path = entry.path().to_path_buf();
                let mut visitor = ScanVisitor {
                    file_path: file_path.clone(),
                    report: &mut report,
                    impl_stack: Vec::new(),
                    trait_stack: Vec::new(),
                };
                visitor.visit_file(&file);
                report.files_scanned += 1;
                // Cache for the synthesis pass — avoids re-reading + re-parsing.
                parsed_files.push((file_path, file));
            }
            Err(e) => {
                report.parse_failures.push(ParseFailure {
                    file: entry.path().to_path_buf(),
                    error: e.to_string(),
                });
            }
        }
    }

    // ---- Lineage safety pass ----
    //
    // ATK-A3-002 — `#[descended_from]` chains require two hard entry guards
    // (ADR-005 Amendment 3 crash-resistance, both required) before any
    // propagation walk reads the edge graph:
    //
    //   1. Cycle detection — a `child → parent → ... → child` chain would
    //      cause a propagation walker to recurse indefinitely. Every cycle
    //      surfaces as one `ParseFailure` with the full chain text so the
    //      user sees which edges form the loop.
    //
    //   2. Depth limit (default 64) — bounds pathological-linear chains
    //      that aren't cyclic but blow the stack. Reports the offending
    //      child + observed depth.
    //
    // Both are emitted into `parse_failures` because they prevent correct
    // scan completion (channel taxonomy: structural error, not semantic
    // warning — the latter is `orphaned_lineage_edges()`).
    //
    // ADR-018 §"Implementation order in scan_workspace": edge-level dedup
    // (BUG-A3-001) MUST run before cycle detection AND propagation walk.
    // The deduped edge set feeds both downstream consumers; the duplicate
    // diagnostic accumulates into parse_failures alongside cycle/depth
    // failures.
    let (deduped_edges, dedup_failures) = dedupe_lineage_edges(&report.lineage_edges);
    report.lineage_edges = deduped_edges;
    report.parse_failures.extend(dedup_failures);
    let lineage_failures = detect_lineage_failures(&report.lineage_edges, MAX_LINEAGE_DEPTH);
    report.parse_failures.extend(lineage_failures);

    // ---- Fingerprint synthesis pass ----
    //
    // After explicit-collection, walk every file again and emit synthetic
    // `Presentation { match_kind: FingerprintMatch }` records for items that
    // match a declared antigen's fingerprint but weren't explicitly annotated.
    //
    // Only antigens with a parseable fingerprint participate. Parse failures
    // are appended to `report.parse_failures` as non-fatal diagnostics —
    // a malformed fingerprint never silently suppresses all matching.
    //
    // Deduplication: an item that already has an explicit `#[presents(X)]`
    // gets no synthetic match for antigen X — `addresses()` is the bridge.

    // Build the set of parseable fingerprints once, before the file re-walk.
    // Collect parse failures separately to avoid aliasing `report` inside the
    // iterator (immutable borrow on `report.antigens` + mutable push on
    // `report.parse_failures` would conflict at borrow-check time).
    let mut fp_parse_failures: Vec<ParseFailure> = Vec::new();
    let fingerprints: Vec<(String, antigen_fingerprint::Fingerprint)> = report
        .antigens
        .iter()
        .filter_map(|ag| {
            let raw = ag.fingerprint.as_deref()?;
            match antigen_fingerprint::Fingerprint::parse(raw) {
                Ok(fp) => Some((ag.type_name.clone(), fp)),
                Err(e) => {
                    fp_parse_failures.push(ParseFailure {
                        file: ag.file.clone(),
                        error: format!(
                            "antigen `{}`: fingerprint failed to re-parse during synthesis: {e}",
                            ag.type_name
                        ),
                    });
                    None
                }
            }
        })
        .collect();
    report.parse_failures.extend(fp_parse_failures);

    if !fingerprints.is_empty() {
        synthesis_pass(&parsed_files, &fingerprints, &mut report);
    }

    Ok(report)
}

// ============================================================================
// Cross-crate enumeration (A3 D3)
//
// Per the A3 scope-lock and navigator's 2026-05-09 ruling: cross-crate scope
// in v0.1 is enumeration + per-crate scanning, NOT merged cross-crate matching.
// The `addresses()` relation stays file-scoped; module-path-qualified
// `ItemTarget` is an ADR-class decision (ATK-A3-005) deferred until aristotle
// rules + an ADR sentence drafts.
//
// Empirical substrate findings (pre-flight P1/P2/P5, 2026-05-09):
//   P1: `cargo metadata --format-version 1` returns `manifest_path` already
//       resolved per-package — no need to construct paths from cargo home +
//       index hash + crate-version suffix. Path-deps, workspace-internal,
//       and registry deps share the same shape.
//   P2: `~/.cargo/registry/src/index.crates.io-<hash>/<crate>-<version>/`
//       hosts multiple co-existing versions of the same crate. The
//       `cargo metadata`-driven approach avoids the multi-version problem
//       entirely because cargo dedupes by version per package.
//   P5: zero `#[antigen(...)]` instances in the wild across the registry
//       (sample: this workspace's 96 reg deps + tambear's 227 reg deps).
//       The collision question is hypothetical until antigen-stdlib lands;
//       Approach 2 vs 3-revised ruling can absorb after D3 ships.
//
// Sub-clause F (ADR-005): cross-crate antigen declarations are trusted
// inputs; the trust anchor is cargo's own checksum verification chain.
// The trust-model ADR sentence is in flight with aristotle.
// ============================================================================

/// How a [`DepCrateRoot`] was sourced — the `cargo metadata` `source` field
/// classified into the buckets the scan tooling cares about.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrateOrigin {
    /// `source: null` — workspace-internal package or path-dep (cross- or
    /// in-workspace). Source already lives at `manifest_path`'s parent.
    PathOrWorkspace,
    /// `source: "registry+..."` — a crates.io or alt-registry dependency
    /// downloaded into `~/.cargo/registry/src/<index>/<crate>-<version>/`.
    Registry,
    /// `source: "git+..."` — a git dependency cloned into
    /// `~/.cargo/git/checkouts/`. Captures `manifest_path` directly without
    /// path-construction.
    Git,
    /// Anything else cargo returns we don't classify yet (sparse registries,
    /// alternative registry indices, future cargo source kinds). The raw
    /// source string is preserved so consumers can decide to scan it or not.
    Other(String),
}

impl CrateOrigin {
    fn from_source(source: Option<&str>) -> Self {
        match source {
            None => Self::PathOrWorkspace,
            Some(s) if s.starts_with("registry+") => Self::Registry,
            Some(s) if s.starts_with("git+") => Self::Git,
            Some(s) => Self::Other(s.to_string()),
        }
    }
}

/// A single dependency crate's enumerated source root.
///
/// Returned by [`enumerate_dep_crate_roots`]. The `crate_root` directory is
/// the parent of the package's `Cargo.toml`; passing it to [`scan_workspace`]
/// scans the crate's full source tree. The `package_name` and `version`
/// pair uniquely identifies the dep across the workspace's resolved graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepCrateRoot {
    /// Cargo package name (e.g., `"serde"`, `"antigen-fingerprint"`).
    pub package_name: String,
    /// Cargo package version (e.g., `"1.0.219"`).
    pub version: String,
    /// Directory containing the package's `Cargo.toml` — i.e., the crate
    /// root suitable for [`scan_workspace`].
    pub crate_root: PathBuf,
    /// Where this crate came from. See [`CrateOrigin`].
    pub origin: CrateOrigin,
}

/// Enumerate dependency crates resolved by cargo for the workspace at
/// `workspace_root`.
///
/// Runs `cargo metadata --format-version 1 --manifest-path <workspace>/Cargo.toml`
/// in a subprocess, parses the JSON, and returns one [`DepCrateRoot`] per
/// non-workspace-member package. Workspace-internal members are excluded:
/// when [`scan_workspace`] is called on the workspace root, it already
/// covers them.
///
/// `include_path_workspace` controls whether `CrateOrigin::PathOrWorkspace`
/// dependencies (cross-workspace path-deps) are returned. The default for
/// CLI consumers is `false` — these path-deps usually live alongside the
/// workspace and are scanned independently. Set `true` to opt in.
///
/// # Errors
///
/// Returns an `io::Error` if:
/// - the `cargo` binary cannot be invoked (`PATH` or executable issue),
/// - `cargo metadata` exits non-zero (manifest parse error, lock-file out
///   of date, network failure on first resolve, etc.),
/// - the JSON output cannot be parsed.
///
/// In all error cases, the error message preserves the underlying cause
/// (cargo's stderr or the JSON parse error) for diagnostic surfacing.
///
/// # Sub-clause F note (ADR-005 / ADR-017 trust delegation)
///
/// Cross-crate antigen declarations are trusted inputs — the trust anchor
/// is cargo's own checksum verification of registry sources + git revision
/// pinning. The ADR-017 (draft) trust delegation model requires two
/// preconditions before extending trust to a registry path:
///
/// 1. The path is reachable from `cargo metadata`'s resolution graph as
///    a transitive dependency of the consumer workspace.
/// 2. The path's parent directory matches the registry's expected layout
///    (`<index>/<crate>-<version>/`).
///
/// **Both preconditions are satisfied by construction here**: this function
/// is the only public mechanism for enumerating cross-crate scan targets,
/// and every path it returns is sourced from `cargo metadata`'s output.
/// Cargo verifies registry layout itself before populating that output;
/// we inherit cargo's verification rather than re-implementing it.
///
/// **Discipline for future contributors**: do NOT add a non-cargo-metadata
/// path discovery mechanism (e.g., recursive walking of
/// `~/.cargo/registry/src/`) without explicitly adding the layout-matching
/// and reachability checks. Such a path would extend trust outside cargo's
/// resolution chain. Adversarial ATK-A3-007 (in
/// `antigen/tests/atk_a3_fractal_preview.rs`) is the green-test for that
/// scenario.
pub fn enumerate_dep_crate_roots(
    workspace_root: &Path,
    include_path_workspace: bool,
) -> std::io::Result<Vec<DepCrateRoot>> {
    use std::process::Command;

    let manifest_path = workspace_root.join("Cargo.toml");
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!(
                    "failed to invoke `cargo metadata` at `{}`: {e} \
                     (is cargo on PATH?)",
                    manifest_path.display()
                ),
            )
        })?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "`cargo metadata` exited with status {} for manifest `{}`: {}",
            output.status,
            manifest_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        std::io::Error::other(format!("failed to parse `cargo metadata` JSON output: {e}"))
    })?;

    // Identify workspace-member package IDs so we can exclude them — running
    // scan_workspace on the workspace root already covers these.
    let workspace_members: std::collections::HashSet<String> = metadata
        .get("workspace_members")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let packages = metadata
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            std::io::Error::other(
                "`cargo metadata` output missing `packages` array — unexpected schema",
            )
        })?;

    let mut roots: Vec<DepCrateRoot> = Vec::new();
    for pkg in packages {
        let id = pkg.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        if workspace_members.contains(id) {
            continue;
        }

        let package_name = pkg
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let version = pkg
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let source = pkg.get("source").and_then(|v| v.as_str());
        let manifest_str = pkg.get("manifest_path").and_then(|v| v.as_str());

        let Some(manifest_str) = manifest_str else {
            // No manifest_path — defensive guard. Skip rather than panic;
            // future cargo schemas may surface unexpected shapes.
            continue;
        };
        let manifest = PathBuf::from(manifest_str);
        let Some(crate_root) = manifest.parent().map(Path::to_path_buf) else {
            continue;
        };

        let origin = CrateOrigin::from_source(source);

        // Path-or-workspace deps: `source` is null. Some are workspace
        // members (already excluded above by id); the rest are path-deps to
        // sibling workspaces (e.g., tambear's path-dep to `R:\antigen`).
        // Skip by default — those workspaces are normally scanned on their
        // own — but allow opt-in for full transitive coverage.
        if matches!(origin, CrateOrigin::PathOrWorkspace) && !include_path_workspace {
            continue;
        }

        roots.push(DepCrateRoot {
            package_name,
            version,
            crate_root,
            origin,
        });
    }

    Ok(roots)
}

/// Emit synthetic `FingerprintMatch` presentations for items that match a
/// declared antigen fingerprint but weren't explicitly annotated.
///
/// Called from [`scan_workspace`] after the explicit-collection walk. Uses the
/// cached `(path, syn::File)` pairs from pass 1 — no re-reading or re-parsing.
/// Only top-level items are checked (`syn::File::items`); descent into `impl`
/// methods and `trait` methods is deferred to W6b/A3.
fn synthesis_pass(
    parsed_files: &[(PathBuf, syn::File)],
    fingerprints: &[(String, antigen_fingerprint::Fingerprint)],
    report: &mut ScanReport,
) {
    for (file_path, parsed) in parsed_files {
        for syn_item in &parsed.items {
            let Some((kind_str, item_target)) = item_kind_and_target(syn_item) else {
                continue;
            };

            // Node-kind dispatch: skip fingerprints whose top-level item
            // constraint can't match this item's kind — cheap O(1) filter
            // per ADR-010 Amendment 3 Performance Invariant 4.
            let item_kind_for_dispatch = match syn_item {
                syn::Item::Struct(_) => Some(antigen_fingerprint::ItemKind::Struct),
                syn::Item::Enum(_) => Some(antigen_fingerprint::ItemKind::Enum),
                syn::Item::Trait(_) => Some(antigen_fingerprint::ItemKind::Trait),
                syn::Item::Fn(_) => Some(antigen_fingerprint::ItemKind::Fn),
                syn::Item::Impl(_) => Some(antigen_fingerprint::ItemKind::Impl),
                syn::Item::Type(_) => Some(antigen_fingerprint::ItemKind::Type),
                syn::Item::Mod(_) => Some(antigen_fingerprint::ItemKind::Mod),
                _ => None,
            };

            for (antigen_type, fp) in fingerprints {
                // Node-kind dispatch: if the fingerprint pins a required kind,
                // skip evaluation when this item's kind doesn't match.
                if let Some(required_kind) = fp.node_kind() {
                    if item_kind_for_dispatch != Some(required_kind) {
                        continue;
                    }
                }

                if !fp.matches(syn_item) {
                    continue;
                }

                // Deduplication: skip if an explicit #[presents] already covers
                // this (antigen_type, file, item) triple, OR if a tolerance
                // acknowledges the match — tolerated sites belong in the
                // "tolerated" state, not "fingerprint match" (5-state matrix,
                // ADR-001 Amendment 1 Change 2).
                let already_covered = report.presentations.iter().any(|p| {
                    p.match_kind == MatchKind::ExplicitMarker
                        && p.antigen_type == *antigen_type
                        && p.file == *file_path
                        && p.item_target.addresses(&item_target)
                }) || report.tolerances.iter().any(|t| {
                    t.antigen_type == *antigen_type
                        && t.file == *file_path
                        && t.item_target.addresses(&item_target)
                });
                if already_covered {
                    continue;
                }

                // Compute line from the item's first attribute or item span.
                let line = item_line(syn_item);

                report.presentations.push(Presentation {
                    antigen_type: antigen_type.clone(),
                    file: file_path.clone(),
                    line,
                    item_kind: kind_str.to_string(),
                    item_target: item_target.clone(),
                    match_kind: MatchKind::FingerprintMatch,
                    canonical_path: None,
                    inherited_from: None,
                });
            }
        }
    }
}

/// Build a `(kind_str, ItemTarget)` pair from a top-level `syn::Item`.
/// Returns `None` for item kinds we don't model (macros, extern crates, etc.).
fn item_kind_and_target(item: &syn::Item) -> Option<(&'static str, ItemTarget)> {
    match item {
        syn::Item::Struct(s) => Some(("struct", ItemTarget::Struct(s.ident.to_string()))),
        syn::Item::Enum(e) => Some(("enum", ItemTarget::Enum(e.ident.to_string()))),
        syn::Item::Trait(t) => Some(("trait", ItemTarget::Trait(t.ident.to_string()))),
        syn::Item::Fn(f) => Some(("fn", ItemTarget::Fn(f.sig.ident.to_string()))),
        syn::Item::Type(t) => Some(("type", ItemTarget::TypeAlias(t.ident.to_string()))),
        syn::Item::Impl(i) => {
            let trait_path = i.trait_.as_ref().map(|(_, path, _)| render_path(path));
            let target_type = render_type(&i.self_ty);
            Some((
                "impl",
                ItemTarget::Impl {
                    trait_path,
                    target_type,
                },
            ))
        }
        // `mod` items are not yet modeled in `ItemTarget`; skip for synthesis.
        _ => None,
    }
}

/// Best-effort line number for a top-level `syn::Item` (line of its first
/// attribute if any, else the item's own span start).
fn item_line(item: &syn::Item) -> usize {
    use syn::spanned::Spanned;
    item.span().start().line
}

/// AST visitor that extracts antigen-related attributes.
struct ScanVisitor<'a> {
    file_path: PathBuf,
    report: &'a mut ScanReport,
    /// Context stack for nested items. The current top of stack is the
    /// enclosing-impl context for any `visit_impl_item_fn` call — so that
    /// a method's `ItemTarget::ImplFn` knows which impl block it lives in.
    /// W3 (sweep A2): structural item-identity tracking.
    impl_stack: Vec<(Option<String>, String)>,
    /// Context stack for nested traits — analogous to `impl_stack`, but
    /// for `visit_trait_item_fn` so trait methods carry the enclosing
    /// trait identifier in `ItemTarget::TraitFn`.
    trait_stack: Vec<String>,
}

impl ScanVisitor<'_> {
    /// Compute 1-indexed line number for a span by counting newlines in source up
    /// to the span's start.
    ///
    /// Resolve the source line of a specific `#[attr]` invocation via
    /// `syn::spanned::Spanned::span().start().line`. Each per-instance call
    /// reports the line of *that* invocation rather than the first match in
    /// the file (the pre-fix heuristic that broke ATK-A2-002 for multi-
    /// instance scenarios).
    ///
    /// Falls back to `0` only if the span info is unavailable (which on
    /// stable rustc with `proc-macro2`'s default features is rare; a 0
    /// return means "we don't know," which is honest).
    fn line_of_attr(attr: &syn::Attribute) -> usize {
        use syn::spanned::Spanned;
        attr.span().start().line
    }

    fn extract_antigen(&mut self, item: &syn::ItemStruct, attr: &syn::Attribute) {
        let type_name = item.ident.to_string();
        let line = Self::line_of_attr(attr);

        if let syn::Meta::List(list) = &attr.meta {
            match syn::parse2::<ScanAntigenArgs>(list.tokens.clone()) {
                Ok(args) => {
                    self.report.antigens.push(AntigenDeclaration {
                        name: args.name,
                        type_name,
                        file: self.file_path.clone(),
                        line,
                        family: args.family,
                        summary: args.summary,
                        fingerprint: args.fingerprint,
                        canonical_path: None,
                    });
                }
                Err(_) => {
                    // Malformed attribute: record with empty name so scan output
                    // surfaces the file for investigation rather than silently skipping.
                    self.report.antigens.push(AntigenDeclaration {
                        name: String::new(),
                        type_name,
                        file: self.file_path.clone(),
                        line,
                        family: None,
                        summary: None,
                        fingerprint: None,
                        canonical_path: None,
                    });
                }
            }
        }
    }

    fn extract_presents(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        let antigen_type = if let syn::Meta::List(list) = &attr.meta {
            // Parse the body as a `syn::Path` rather than splitting the
            // `quote::ToTokens` rendering on `::`. The string form contains
            // whitespace artifacts (`" my_crate :: Foo "`) that the prior
            // tactical-fix code recovered from with a `.trim()` — but the
            // structural fix is to never produce the string in the first
            // place. ATK-A2-001's pre-W3 hotfix landed in commit b9440b2;
            // this is the W3 structural form. Path's last segment is the
            // bare type name regardless of qualifier.
            match syn::parse2::<syn::Path>(list.tokens.clone()) {
                Ok(path) => path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default(),
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[presents] attribute: {e}"),
                    });
                    return;
                }
            }
        } else {
            return;
        };
        let line = Self::line_of_attr(attr);
        self.report.presentations.push(Presentation {
            antigen_type,
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
            match_kind: MatchKind::ExplicitMarker,
            canonical_path: None,
            inherited_from: None,
        });
    }

    fn extract_immune(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        if let syn::Meta::List(list) = &attr.meta {
            // Scan records the witness expression verbatim; validity
            // classification (Test, Proptest, PhantomType, Function, External)
            // and behavioral verification (cargo test invocation) are the
            // audit module's responsibility. ADR-005 sub-clause F: the
            // trust boundary at "immunity claim" is checked by audit, not
            // by scan — scan provides the substrate, audit decides validity.
            let (antigen_type, witness) = match syn::parse2::<ScanImmuneArgs>(list.tokens.clone()) {
                Ok(args) => (args.antigen_type, args.witness),
                Err(e) => {
                    // Malformed #[immune] args: record a parse failure rather
                    // than silently inserting a ghost immunity record with empty
                    // antigen_type and witness. A ghost record would pass
                    // WitnessStatus::Missing detection only if the empty-string
                    // check fires, and would produce a misleading "0 unaddressed
                    // presentations" result. ADR-005: every trust boundary requires
                    // a validation check; malformed immunity claims are not claims.
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[immune] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.immunities.push(Immunity {
                antigen_type,
                witness,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
                canonical_path: None,
            });
        }
    }

    fn extract_tolerance(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanToleranceArgs>(list.tokens.clone()) {
                Ok(args) => args,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[antigen_tolerance] attribute: {e}"),
                    });
                    return;
                }
            };
            // Per ADR-011 §Mechanics §1: rationale required + non-empty.
            // Scan side enforces the same boundary the macro enforces — a
            // tolerance without rationale is silent suppression.
            if args.rationale.is_empty() {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: "#[antigen_tolerance] requires non-empty rationale".to_string(),
                });
                return;
            }
            let line = Self::line_of_attr(attr);
            self.report.tolerances.push(Toleration {
                antigen_type: args.antigen_type,
                rationale: args.rationale,
                until: args.until,
                see: args.see,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
                canonical_path: None,
            });
        }
    }

    fn extract_descended_from(&mut self, attr: &syn::Attribute, item_target: &ItemTarget) {
        // ADR-013: `#[descended_from]` is meaningful only on antigen-type
        // declarations (unit `struct` and class-shaped `enum`). Other
        // placements — impl blocks, free functions, traits, methods —
        // surface as parse_failures so the user sees what got dropped
        // rather than the visitor silently no-op'ing them.
        let child = match item_target {
            ItemTarget::Struct(name) | ItemTarget::Enum(name) => name.clone(),
            other => {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!(
                        "#[descended_from] on `{}` is not a type declaration; \
                         this attribute is meaningful only on `struct` and `enum` \
                         antigen declarations",
                        other.label()
                    ),
                });
                return;
            }
        };

        let syn::Meta::List(list) = &attr.meta else {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "malformed #[descended_from] attribute: expected `(parent)`".to_string(),
            });
            return;
        };

        // Body is a single positional `syn::Path`, mirroring
        // `extract_presents`. Last segment becomes the bare parent type
        // name — module-path qualification is an A3+ ADR-class question
        // (ATK-A3-005), so for now we keep names bare.
        let parent = match syn::parse2::<syn::Path>(list.tokens.clone()) {
            Ok(path) => path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            Err(e) => {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!("malformed #[descended_from] attribute: {e}"),
                });
                return;
            }
        };

        if parent.is_empty() {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "#[descended_from] requires a parent path argument".to_string(),
            });
            return;
        }

        let line = Self::line_of_attr(attr);
        self.report.lineage_edges.push(LineageEdge {
            child,
            parent,
            file: self.file_path.clone(),
            line,
            parent_canonical_path: None,
            child_canonical_path: None,
        });
    }

    fn check_attrs(&mut self, attrs: &[syn::Attribute], item_kind: &str, item_target: &ItemTarget) {
        for attr in attrs {
            if attr_is(attr, "presents") {
                self.extract_presents(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "immune") {
                self.extract_immune(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "antigen_tolerance") {
                self.extract_tolerance(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "descended_from") {
                self.extract_descended_from(attr, item_target);
            }
        }
    }
}

/// Render a `syn::Type` to its canonical token-stream string. Used to
/// extract a string identifier for `impl Trait for Type` blocks. The
/// rendering normalizes whitespace via `quote::ToTokens`. For W3 we only
/// need a stable string for equality matching — A3 cross-crate work will
/// likely want a richer canonical form (e.g., resolved module paths).
fn render_type(ty: &syn::Type) -> String {
    use quote::ToTokens;
    ty.to_token_stream().to_string()
}

/// Whether an attribute's path matches a given antigen attribute name.
///
/// `syn::Path::is_ident("X")` only returns true for single-segment paths.
/// Path-qualified attribute forms — `#[antigen::immune(...)]`,
/// `#[crate::presents(...)]`, `#[my::module::antigen(...)]` — produce
/// multi-segment paths that `is_ident` rejects, causing the scan to
/// silently drop them. The fix: an attribute's path matches `name`
/// either when it's the bare ident, OR when its *last segment* is the
/// ident.
///
/// This is the path-segment-aware analog of `is_ident` and is the only
/// matcher used inside `ScanVisitor`. Using last-segment equality is
/// cheap and the same heuristic Rust itself uses to find the macro
/// being invoked — name resolution happens elsewhere.
fn attr_is(attr: &syn::Attribute, name: &str) -> bool {
    let path = attr.path();
    path.is_ident(name) || path.segments.last().is_some_and(|s| s.ident == name)
}

/// Render a `syn::Path` similarly. Used for the trait portion of
/// `impl Trait for Type` so that `Drop` and `core::ops::Drop` produce
/// distinct strings (which is correct — they're different items in
/// Rust's name resolution, even when they alias).
fn render_path(path: &syn::Path) -> String {
    use quote::ToTokens;
    path.to_token_stream().to_string()
}

impl<'ast> Visit<'ast> for ScanVisitor<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        for attr in &item.attrs {
            if attr_is(attr, "antigen") {
                self.extract_antigen(item, attr);
            }
        }
        let target = ItemTarget::Struct(item.ident.to_string());
        self.check_attrs(&item.attrs, "struct", &target);
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let trait_path = item.trait_.as_ref().map(|(_, path, _)| render_path(path));
        let target_type = render_type(&item.self_ty);
        let target = ItemTarget::Impl {
            trait_path: trait_path.clone(),
            target_type: target_type.clone(),
        };
        self.check_attrs(&item.attrs, "impl", &target);
        // Push impl context so visit_impl_item_fn can build ImplFn targets.
        self.impl_stack.push((trait_path, target_type));
        syn::visit::visit_item_impl(self, item);
        self.impl_stack.pop();
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let target = ItemTarget::Fn(item.sig.ident.to_string());
        self.check_attrs(&item.attrs, "fn", &target);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let target = self.impl_stack.last().map_or_else(
            || ItemTarget::Fn(item.sig.ident.to_string()),
            |(trait_path, target_type)| ItemTarget::ImplFn {
                trait_path: trait_path.clone(),
                target_type: target_type.clone(),
                fn_name: item.sig.ident.to_string(),
            },
        );
        self.check_attrs(&item.attrs, "impl_fn", &target);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let target = ItemTarget::Trait(item.ident.to_string());
        self.check_attrs(&item.attrs, "trait", &target);
        // Push trait context so visit_trait_item_fn produces TraitFn targets
        // identifying the enclosing trait.
        self.trait_stack.push(item.ident.to_string());
        syn::visit::visit_item_trait(self, item);
        self.trait_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let target = self.trait_stack.last().map_or_else(
            || ItemTarget::Fn(item.sig.ident.to_string()),
            |trait_name| ItemTarget::TraitFn {
                trait_name: trait_name.clone(),
                fn_name: item.sig.ident.to_string(),
            },
        );
        self.check_attrs(&item.attrs, "trait_fn", &target);
        syn::visit::visit_trait_item_fn(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        // Type aliases (`type Foo = ...;`) carry attributes too. ATK-W3-005:
        // without this handler, attributes on type aliases would fall back
        // to ItemTarget::Unknown, and two unrelated Unknown items collide
        // on equality. Tracking the alias name keeps each alias as its own
        // distinct match target.
        let target = ItemTarget::TypeAlias(item.ident.to_string());
        self.check_attrs(&item.attrs, "type_alias", &target);
        syn::visit::visit_item_type(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        for attr in &item.attrs {
            if attr_is(attr, "antigen") {
                // ATK-A2-007: silently dropping #[antigen] on enums eats the
                // class-enum pattern (the frame-translation antigen's primary
                // use case). Surface the situation as a parse_failure so the
                // user sees it, rather than the previous `let _ = attr` no-op.
                // The macro itself still rejects non-unit structs at compile
                // time; this scan-side diagnostic catches enum cases that
                // wouldn't reach the macro (e.g., a user investigating "why
                // doesn't my class enum scan as an antigen?").
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!(
                        "#[antigen] on enum `{}` is not supported in v0.1; \
                         antigen declarations must be unit structs (e.g., \
                         `pub struct {};`). Enum-shaped failure-classes are \
                         tracked by ADR-010 Amendment 1's class-enum operator \
                         in a future grammar version.",
                        item.ident, item.ident
                    ),
                });
            }
        }
        let target = ItemTarget::Enum(item.ident.to_string());
        self.check_attrs(&item.attrs, "enum", &target);
        syn::visit::visit_item_enum(self, item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scan_report_has_no_unaddressed() {
        let report = ScanReport::default();
        assert!(report.unaddressed_presentations().is_empty());
    }

    #[test]
    fn antigen_args_parses_name_and_fingerprint() {
        let tokens: proc_macro2::TokenStream = r#"
            name = "frame-translation",
            fingerprint = "item: enum, has_method(\"meet\", \"(Self, Self) -> Self\")"
        "#
        .parse()
        .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "frame-translation");
        // The fingerprint must be correctly unescaped — not contain raw backslash-quote.
        let fp = args.fingerprint.unwrap();
        assert!(
            fp.contains("has_method(\"meet\""),
            "fingerprint should contain unescaped double-quotes, got: {fp:?}"
        );
        assert!(
            !fp.contains(r#"\""#),
            "fingerprint must not contain raw backslash-quote escape sequences, got: {fp:?}"
        );
    }

    #[test]
    fn antigen_args_parses_optional_fields() {
        let tokens: proc_macro2::TokenStream =
            r#"name = "panicking-in-drop", fingerprint = "impl Drop", family = "boundary-violation", summary = "Drop impl can panic""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "panicking-in-drop");
        assert_eq!(args.family.as_deref(), Some("boundary-violation"));
        assert_eq!(args.summary.as_deref(), Some("Drop impl can panic"));
    }

    #[test]
    fn immune_args_parses_antigen_type_and_witness() {
        let tokens: proc_macro2::TokenStream = r"PanickingInDrop, witness = no_panic_in_drop_test"
            .parse()
            .unwrap();
        let args = syn::parse2::<ScanImmuneArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type, "PanickingInDrop");
        assert_eq!(args.witness, "no_panic_in_drop_test");
    }

    #[test]
    fn immune_args_parses_path_witness() {
        let tokens: proc_macro2::TokenStream =
            r"FrameTranslation, witness = clippy :: no_panic_in_drop"
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanImmuneArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type, "FrameTranslation");
        // witness is the token-stream rendering of the expression
        assert!(args.witness.contains("no_panic_in_drop"));
    }

    // ========================================================================
    // Cross-parser equivalence fixtures
    //
    // These fixtures must stay in sync with `antigen-macros::parse::tests::
    // ANTIGEN_PARSER_FIXTURES`. The invariant: for any input the proc-macro
    // parser accepts as valid, the scan parser must produce equivalent semantic
    // content for the four overlapping fields (name, fingerprint, family,
    // summary). The two parsers live in different crates by structural
    // necessity (proc-macro = true crates can't be linked as libraries) and
    // their drift was the substance of ATK-001-2 in pre-team scaffolding.
    //
    // When adding a fixture here, add the matching one in antigen-macros's
    // parse.rs ANTIGEN_PARSER_FIXTURES table. Field-additions to the antigen
    // attribute grammar must add fixtures in BOTH crates to lock the
    // equivalence.
    // ========================================================================

    // (input, expected_name, expected_fingerprint, expected_family, expected_summary)
    type ScanFixture = (
        &'static str,
        &'static str,
        &'static str,
        Option<&'static str>,
        Option<&'static str>,
    );

    const SCAN_PARSER_FIXTURES: &[ScanFixture] = &[
        (
            r#"name = "panicking-in-drop", fingerprint = "impl Drop with panic""#,
            "panicking-in-drop",
            "impl Drop with panic",
            None,
            None,
        ),
        (
            r#"name = "frame-translation", fingerprint = "class enum + meet", family = "semantic-drift", summary = "Polarity inverts at the frame boundary""#,
            "frame-translation",
            "class enum + meet",
            Some("semantic-drift"),
            Some("Polarity inverts at the frame boundary"),
        ),
        (
            r#"name = "x", fingerprint = "item: enum, has_method(\"meet\", \"(Self, Self) -> Self\")""#,
            "x",
            r#"item: enum, has_method("meet", "(Self, Self) -> Self")"#,
            None,
            None,
        ),
        (
            r#"summary = "S", family = "F", fingerprint = "FP", name = "n""#,
            "n",
            "FP",
            Some("F"),
            Some("S"),
        ),
        (
            r#"name = "x", fingerprint = "y", references = ["GAP-1", "DEC-2"]"#,
            "x",
            "y",
            None,
            None,
        ),
        (
            "name = \"multi-line\",\n\tfingerprint = \"shape\",\n\tfamily = \"family\"",
            "multi-line",
            "shape",
            Some("family"),
            None,
        ),
    ];

    #[test]
    fn scan_parser_accepts_all_macro_fixtures() {
        for (input, exp_name, exp_fp, exp_family, exp_summary) in SCAN_PARSER_FIXTURES {
            let tokens: proc_macro2::TokenStream = input
                .parse()
                .unwrap_or_else(|e| panic!("fixture failed to tokenize: {input:?}: {e}"));
            let args = syn::parse2::<ScanAntigenArgs>(tokens)
                .unwrap_or_else(|e| panic!("scan parser rejected fixture {input:?}: {e}"));
            assert_eq!(&args.name, exp_name, "name mismatch for fixture: {input:?}");
            assert_eq!(
                args.fingerprint.as_deref(),
                Some(*exp_fp),
                "fingerprint mismatch for fixture: {input:?}"
            );
            assert_eq!(
                args.family.as_deref(),
                *exp_family,
                "family mismatch for fixture: {input:?}"
            );
            assert_eq!(
                args.summary.as_deref(),
                *exp_summary,
                "summary mismatch for fixture: {input:?}"
            );
        }
    }

    #[test]
    fn scan_parser_tolerates_unknown_fields() {
        // Asymmetry from macro side: scan tolerates unknown fields silently
        // (forward-compat for fields added later that this scan binary doesn't
        // know yet). Macro side rejects them strictly.
        let tokens: proc_macro2::TokenStream =
            r#"name = "x", fingerprint = "y", future_field = "irrelevant""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "x");
        assert_eq!(args.fingerprint.as_deref(), Some("y"));
    }

    #[test]
    fn scan_parser_tolerates_missing_required_fields() {
        // Asymmetry from macro side: scan defaults to empty rather than
        // erroring. Malformed declarations get aggregated into parse_failures
        // upstream rather than blocking the scan.
        let tokens: proc_macro2::TokenStream = r#"name = "only-name""#.parse().unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "only-name");
        assert_eq!(args.fingerprint, None);
    }

    // ========================================================================
    // Property tests (W1) — proptest invariants over the scan-side parser.
    //
    // These are the scan-side half of the cross-parser equivalence story.
    // Their invariants mirror the macro-side proptests in
    // `antigen-macros/src/parse.rs::parser_props`. Both sides share the same
    // input strategies (literal-copied; if you change one, change the other
    // in the same commit) — this is the by-construction substitute for
    // running both parsers in one binary, which the proc-macro crate
    // separation forbids.
    //
    // Cross-parser invariants asserted (macro-side P1-P8 mirror):
    //
    //   I1 (P1 mirror) — round-trip on intersection: any input the macro
    //        side accepts, the scan side accepts and produces equivalent
    //        semantic content for name/fingerprint/family/summary.
    //
    //   I2 (P2 mirror) — order-invariance: shuffling fields doesn't change
    //        the parsed result on the scan side.
    //
    //   I3 (asymmetry) — scan tolerates what macro rejects:
    //        - unknown fields: macro errors, scan silently consumes
    //        - missing required: macro errors, scan defaults to empty
    //        These asymmetries are intentional (forward-compat + non-blocking
    //        scan progress on partial workspaces) and are documented as
    //        properties so they don't accidentally regress.
    //
    // The macro-side parser reports were tested under `parser_props` in the
    // sister crate. Together, the two test modules form the W1 floor that
    // ADR-001 Amendment 1 C5 (drift-detection-at-scan-time) makes
    // load-bearing.
    // ========================================================================

    mod parser_props {
        use super::super::*;
        use proc_macro2::TokenStream;
        use proptest::prelude::*;

        /// Rust strict + reserved keywords that cannot appear as path
        /// segments. Kept in sync with `antigen-macros/src/parse.rs::
        /// parser_props::RUST_KEYWORDS` (literal-shared by convention; if
        /// you change one, change the other in the same commit).
        const RUST_KEYWORDS: &[&str] = &[
            "as", "async", "await", "box", "break", "const", "continue", "crate", "do", "dyn",
            "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
            "macro", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
            "struct", "super", "trait", "true", "type", "union", "unsafe", "use", "where", "while",
            "yield", "abstract", "become", "final", "override", "priv", "try",
        ];

        // --- Strategies (kept literally identical to macro side; see
        //     antigen-macros/src/parse.rs::parser_props). ---

        fn valid_kebab() -> impl Strategy<Value = String> {
            proptest::collection::vec(
                (
                    proptest::char::range('a', 'z'),
                    proptest::collection::vec(
                        prop_oneof![
                            proptest::char::range('a', 'z'),
                            proptest::char::range('0', '9'),
                        ],
                        0..8usize,
                    ),
                )
                    .prop_map(|(first, rest)| {
                        let mut s = String::with_capacity(rest.len() + 1);
                        s.push(first);
                        for c in rest {
                            s.push(c);
                        }
                        s
                    }),
                1..5usize,
            )
            .prop_map(|segments| segments.join("-"))
        }

        fn valid_text(max_len: usize) -> impl Strategy<Value = String> {
            proptest::collection::vec(
                prop_oneof![
                    proptest::char::range(' ', '~').prop_filter("excluded chars", |c| {
                        *c != '\\' && *c != '"' && *c != '\0'
                    }),
                ],
                1..=max_len,
            )
            .prop_map(|chars| chars.into_iter().collect())
        }

        fn lit(s: &str) -> String {
            format!("{s:?}")
        }

        fn render_antigen_body(
            name: &str,
            fingerprint: &str,
            family: Option<&str>,
            summary: Option<&str>,
        ) -> String {
            let mut parts = vec![
                format!("name = {}", lit(name)),
                format!("fingerprint = {}", lit(fingerprint)),
            ];
            if let Some(f) = family {
                parts.push(format!("family = {}", lit(f)));
            }
            if let Some(s) = summary {
                parts.push(format!("summary = {}", lit(s)));
            }
            parts.join(", ")
        }

        proptest! {
            // I1 — equivalence-on-intersection: any input the macro side
            // accepts (i.e., any input render_antigen_body produces from
            // valid_kebab + valid_text), the scan side accepts and produces
            // matching semantic content. The macro-side counterpart asserts
            // its own acceptance + identical extraction; together they lock
            // cross-parser drift.
            #[test]
            fn scan_parser_round_trip_on_macro_inputs(
                name in valid_kebab(),
                fingerprint in valid_text(64),
                family in proptest::option::of(valid_text(32)),
                summary in proptest::option::of(valid_text(64)),
            ) {
                let body = render_antigen_body(&name, &fingerprint, family.as_deref(), summary.as_deref());
                let tokens: TokenStream = body.parse().expect("body must tokenize");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan must accept macro-acceptable input");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
                prop_assert_eq!(args.family.as_deref(), family.as_deref());
                prop_assert_eq!(args.summary.as_deref(), summary.as_deref());
            }

            // I2 — order-invariance: scan side, like macro side, must not
            // depend on field order.
            #[test]
            fn scan_parser_order_invariant(
                name in valid_kebab(),
                fingerprint in valid_text(48),
                family in valid_text(24),
                summary in valid_text(48),
            ) {
                let canonical = format!(
                    "name = {}, fingerprint = {}, family = {}, summary = {}",
                    lit(&name), lit(&fingerprint), lit(&family), lit(&summary),
                );
                let reversed = format!(
                    "summary = {}, family = {}, fingerprint = {}, name = {}",
                    lit(&summary), lit(&family), lit(&fingerprint), lit(&name),
                );
                let a: ScanAntigenArgs = syn::parse2(canonical.parse::<TokenStream>().unwrap()).unwrap();
                let b: ScanAntigenArgs = syn::parse2(reversed.parse::<TokenStream>().unwrap()).unwrap();
                prop_assert_eq!(&a.name, &b.name);
                prop_assert_eq!(&a.fingerprint, &b.fingerprint);
                prop_assert_eq!(&a.family, &b.family);
                prop_assert_eq!(&a.summary, &b.summary);
            }

            // I3a (asymmetry) — scan tolerates unknown fields. For ANY
            // valid base input plus an arbitrary unknown field, scan still
            // parses and recovers name/fingerprint correctly.
            #[test]
            fn scan_parser_tolerates_arbitrary_unknown_field(
                name in valid_kebab(),
                fingerprint in valid_text(32),
                unknown in "[a-z][a-z_]{2,12}".prop_filter(
                    "must not collide with known fields or Rust keywords",
                    |s| {
                        !matches!(s.as_str(), "name" | "fingerprint" | "family" | "summary" | "references")
                            && !RUST_KEYWORDS.contains(&s.as_str())
                    },
                ),
                unknown_val in valid_text(16),
            ) {
                let body = format!(
                    "name = {}, fingerprint = {}, {} = {}",
                    lit(&name), lit(&fingerprint), unknown, lit(&unknown_val),
                );
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan tolerates unknown fields");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
            }

            // I3b (asymmetry) — scan tolerates missing required fields:
            // an input with only `name` (or only `fingerprint`) parses,
            // with the other field as `None` / empty. Macro side errors.
            #[test]
            fn scan_parser_tolerates_missing_fingerprint(
                name in valid_kebab(),
            ) {
                let body = format!("name = {}", lit(&name));
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan tolerates missing fingerprint");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint, None);
            }

            // I4 — references field is consumed silently (not stored in
            // the scan output today, but must not error). Macro side
            // stores into Vec<String>.
            #[test]
            fn scan_parser_consumes_references_array(
                name in valid_kebab(),
                fingerprint in valid_text(32),
                refs in proptest::collection::vec(valid_text(24), 0..6usize),
            ) {
                let refs_lit: Vec<String> = refs.iter().map(|s| lit(s)).collect();
                let body = format!(
                    "name = {}, fingerprint = {}, references = [{}]",
                    lit(&name), lit(&fingerprint), refs_lit.join(", "),
                );
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan parses references");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
            }

            // I5 — ScanImmuneArgs: any (path, witness-path) parses and
            // exposes the last segment as antigen_type. Mirrors the
            // macro-side P7 property.
            //
            // Identifier strategies skip Rust keywords: even though our
            // regex generates legal-looking strings like "as" or "fn",
            // syn rejects them as path segments. Filtering the keyword
            // set out is the by-construction property — "valid path
            // segments parse" — rather than the false stronger one
            // "any [a-z][a-z_0-9]* parses".
            #[test]
            fn scan_immune_extracts_last_path_segment(
                antigen in "[A-Z][A-Za-z0-9]{0,16}",
                witness_segments in proptest::collection::vec(
                    "[a-z][a-z_0-9]{0,8}".prop_filter(
                        "must not be a Rust keyword",
                        |s| !RUST_KEYWORDS.contains(&s.as_str()),
                    ),
                    1..4usize,
                ),
            ) {
                let witness = witness_segments.join("::");
                let body = format!("{antigen}, witness = {witness}");
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanImmuneArgs>(tokens).expect("body parses");
                prop_assert_eq!(args.antigen_type.as_str(), antigen.as_str());
                // witness_segments.last() is the trailing identifier; the
                // scan parser canonicalises whitespace via quote::ToTokens,
                // so the rendered witness contains all segments separated
                // by " :: ".
                let last = witness_segments.last().unwrap();
                prop_assert!(args.witness.contains(last.as_str()),
                    "rendered witness {:?} should contain trailing segment {:?}", args.witness, last);
            }

            // I6 — ScanImmuneArgs: a qualified-path antigen extracts only
            // the last segment as the antigen_type (the matching surface
            // antigen scan/audit reasons against). This is a regression
            // anchor for ATK-A2-001 — the path-split corruption that the
            // adversarial pass surfaced.
            #[test]
            fn scan_immune_qualified_antigen_path_extracts_last_segment(
                module_segs in proptest::collection::vec(
                    "[a-z][a-z_0-9]{0,6}".prop_filter(
                        "must not be a Rust keyword",
                        |s| !RUST_KEYWORDS.contains(&s.as_str()),
                    ),
                    1..3usize,
                ),
                antigen in "[A-Z][A-Za-z0-9]{0,12}",
                witness in "[a-z][a-z_0-9]{0,12}".prop_filter(
                    "must not be a Rust keyword",
                    |s| !RUST_KEYWORDS.contains(&s.as_str()),
                ),
            ) {
                let qualified = format!("{}::{}", module_segs.join("::"), antigen);
                let body = format!("{qualified}, witness = {witness}");
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanImmuneArgs>(tokens).expect("body parses");
                prop_assert_eq!(args.antigen_type.as_str(), antigen.as_str(),
                    "qualified antigen path {:?} must yield bare last-segment antigen_type", qualified);
            }
        }
    }

    // ========================================================================
    // A3: lineage edge cycle detection (ATK-A3-002)
    // ========================================================================

    fn edge(child: &str, parent: &str) -> LineageEdge {
        LineageEdge {
            child: child.to_string(),
            parent: parent.to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            parent_canonical_path: None,
            child_canonical_path: None,
        }
    }

    #[test]
    fn lineage_no_edges_no_failures() {
        let failures = detect_lineage_failures(&[], MAX_LINEAGE_DEPTH);
        assert!(failures.is_empty());
    }

    #[test]
    fn lineage_acyclic_chain_no_failures() {
        // C -> B -> A (deepest first declared)
        let edges = vec![edge("C", "B"), edge("B", "A")];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert!(
            failures.is_empty(),
            "acyclic chain must produce no failures, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_self_loop_detected() {
        // A -> A
        let edges = vec![edge("A", "A")];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            1,
            "self-loop must report exactly one failure"
        );
        assert!(
            failures[0].error.contains("cycle"),
            "self-loop error must mention cycle, got: {}",
            failures[0].error
        );
        assert!(
            failures[0].error.contains("A -> A"),
            "self-loop error must contain chain `A -> A`, got: {}",
            failures[0].error
        );
    }

    #[test]
    fn lineage_two_node_cycle_detected() {
        // A -> B -> A
        let edges = vec![edge("A", "B"), edge("B", "A")];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            1,
            "2-cycle must report one failure, got: {failures:?}"
        );
        let err = &failures[0].error;
        assert!(err.contains("cycle"), "must mention cycle, got: {err}");
        assert!(
            err.contains("A -> B -> A") || err.contains("B -> A -> B"),
            "must contain full cycle chain, got: {err}"
        );
    }

    #[test]
    fn lineage_three_node_cycle_detected() {
        // A -> B -> C -> A
        let edges = vec![edge("A", "B"), edge("B", "C"), edge("C", "A")];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            1,
            "3-cycle must report exactly one failure, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_cycle_dedup_across_entry_points() {
        // A -> B -> C -> A (same cycle reachable from B and from C)
        // Adding extra non-cyclic edges should still produce one failure.
        let edges = vec![
            edge("A", "B"),
            edge("B", "C"),
            edge("C", "A"),
            edge("D", "B"), // D enters the cycle through B
            edge("E", "C"), // E enters the cycle through C
        ];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            1,
            "same cycle entered from multiple roots must dedup, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_two_disjoint_cycles_both_reported() {
        // A -> B -> A (cycle 1) and X -> Y -> X (cycle 2)
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("X", "Y"),
            edge("Y", "X"),
        ];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            2,
            "two disjoint cycles must produce two failures, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_diamond_no_cycle() {
        // A descended_from B, A descended_from C, B descended_from D, C descended_from D
        // (a DAG diamond — no cycle even though D is reached via two paths)
        let edges = vec![
            edge("A", "B"),
            edge("A", "C"),
            edge("B", "D"),
            edge("C", "D"),
        ];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert!(
            failures.is_empty(),
            "DAG diamond must not be reported as cycle, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_depth_limit_fires_on_long_linear_chain() {
        // 10-node linear chain with a depth limit of 5 fires depth-exceeded.
        let edges: Vec<LineageEdge> = (0..10)
            .map(|i| edge(&format!("N{i}"), &format!("N{}", i + 1)))
            .collect();
        let failures = detect_lineage_failures(&edges, 5);
        assert!(
            failures.iter().any(|f| f.error.contains("maximum depth")),
            "depth limit must fire on long linear chain, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_canonicalise_cycle_basic() {
        // Three rotations of the same cycle produce the same canonical form.
        let a = canonicalise_cycle(&["A", "B", "C"]);
        let b = canonicalise_cycle(&["B", "C", "A"]);
        let c = canonicalise_cycle(&["C", "A", "B"]);
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn lineage_canonicalise_cycle_distinguishes_distinct() {
        // Different cycles canonicalise to different forms.
        let a = canonicalise_cycle(&["A", "B"]);
        let b = canonicalise_cycle(&["A", "C"]);
        assert_ne!(a, b);
    }

    // ========================================================================
    // A3: orphaned lineage edges query (ATK-A3-003)
    // ========================================================================

    fn antigen_decl(type_name: &str) -> AntigenDeclaration {
        AntigenDeclaration {
            name: type_name.to_lowercase(),
            type_name: type_name.to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            family: None,
            summary: None,
            fingerprint: None,
            canonical_path: None,
        }
    }

    #[test]
    fn orphaned_lineage_edges_empty_report_returns_empty() {
        let report = ScanReport::default();
        assert!(report.orphaned_lineage_edges().is_empty());
    }

    #[test]
    fn orphaned_lineage_edges_known_parent_not_orphan() {
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("Parent"));
        report.antigens.push(antigen_decl("Child"));
        report.lineage_edges.push(edge("Child", "Parent"));
        assert!(report.orphaned_lineage_edges().is_empty());
    }

    #[test]
    fn orphaned_lineage_edges_unknown_parent_is_orphan() {
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("Child"));
        report.lineage_edges.push(edge("Child", "MissingParent"));
        let orphans = report.orphaned_lineage_edges();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].parent, "MissingParent");
    }

    // ========================================================================
    // ATK-A3: adversarial edge cases.
    // Two are FAILING bug contracts (atk_a3_dup, atk_a3_orphan_child).
    // Two are PASSING positive-controls verifying dedup correctness.
    // ========================================================================

    #[test]
    fn atk_a3_dup_duplicate_lineage_edge_is_diagnosed_not_silent() {
        // ATK-A3-DUP / BUG-A3-001: two `#[descended_from(B)]` on the same
        // struct A produce two identical lineage edges. Without the dedup
        // pass, the DFS in `detect_lineage_failures` silently swallows the
        // duplicate (black-skip path). Per ADR-004 (implicit-to-explicit
        // elevation), the dedup pass surfaces collapsed duplicates as
        // explicit parse_failures.
        //
        // ADR-018 §"Implementation order in scan_workspace" ratifies that
        // edge-level dedup runs as a separate pass before cycle detection
        // AND the propagation walk. This test exercises the dedup
        // function directly; the integration is verified in
        // `scan_workspace` end-to-end via the BUG-A3-001 fixture (see
        // atk_a3_fractal_preview.rs).
        let edges = vec![
            edge("A", "B"),
            edge("A", "B"), // exact duplicate (same canonical_path = None)
        ];
        let (deduped, failures) = dedupe_lineage_edges(&edges);
        assert_eq!(
            deduped.len(),
            1,
            "duplicate edges must collapse to one in the deduped output"
        );
        assert!(
            !failures.is_empty(),
            "duplicate lineage edge (A->B twice) must produce at least one \
             diagnostic; got: {failures:?}"
        );
    }

    #[test]
    fn dedupe_distinguishes_edges_by_canonical_path() {
        // ADR-018 §"Edge-level dedup": dedup key is (child, parent,
        // child_canonical_path, parent_canonical_path). Same-name edges
        // with different canonical_paths are NOT duplicates (a workspace
        // depending on `foo@1.0.0::P` and `foo@2.0.0::P` legitimately has
        // both edges pointing at different identities).
        let edge_v1 = LineageEdge {
            child: "Child".to_string(),
            parent: "Parent".to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            parent_canonical_path: Some("foo@1.0.0".to_string()),
            child_canonical_path: None,
        };
        let edge_v2 = LineageEdge {
            child: "Child".to_string(),
            parent: "Parent".to_string(),
            file: PathBuf::from("test.rs"),
            line: 2,
            parent_canonical_path: Some("foo@2.0.0".to_string()),
            child_canonical_path: None,
        };
        let (deduped, failures) = dedupe_lineage_edges(&[edge_v1, edge_v2]);
        assert_eq!(
            deduped.len(),
            2,
            "edges differing in parent_canonical_path are distinct identities, \
             not duplicates"
        );
        assert!(
            failures.is_empty(),
            "no dedup failure should fire for cross-version edges; got: {failures:?}"
        );
    }

    #[test]
    fn atk_a3_shared_two_cycles_sharing_a_node_both_reported() {
        // ATK-A3-SHARED: A->B->A forms cycle 1; A->C->A forms cycle 2.
        // Node A participates in both. The canonicalise_cycle dedup must NOT
        // suppress cycle 2 because it shares node A with cycle 1 — the cycles
        // are structurally distinct ({A,B} vs {A,C}).
        //
        // This is a positive-control test verifying the dedup logic does not
        // over-suppress. Expected: 2 failures.
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("A", "C"),
            edge("C", "A"),
        ];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            2,
            "two distinct cycles sharing node A must both be reported; \
             got: {failures:?}"
        );
        let texts: Vec<&str> = failures.iter().map(|f| f.error.as_str()).collect();
        assert!(
            texts.iter().any(|t| t.contains('A') && t.contains('B')),
            "one failure must name the A-B cycle; texts: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains('A') && t.contains('C')),
            "one failure must name the A-C cycle; texts: {texts:?}"
        );
    }

    #[test]
    fn atk_a3_combined_cycle_and_depth_exceeded_both_reported() {
        // ATK-A3-COMBINED: a graph with BOTH a cycle (X->Y->X) and a long chain
        // (N0->...->N5) exceeding a small depth limit. Both failure types must
        // appear — the pass must not short-circuit after the first failure type.
        //
        // Contract: at least one "cycle" failure AND at least one "maximum depth"
        // failure must be present.
        let depth = 3_usize;
        let mut edges = vec![edge("X", "Y"), edge("Y", "X")];
        for i in 0..=(depth + 2) {
            edges.push(edge(&format!("N{i}"), &format!("N{}", i + 1)));
        }
        let failures = detect_lineage_failures(&edges, depth);
        assert!(
            failures.iter().any(|f| f.error.contains("cycle")),
            "cycle X->Y->X must be detected even when long chain is also present; \
             all failures: {failures:?}"
        );
        assert!(
            failures.iter().any(|f| f.error.contains("maximum depth")),
            "depth limit must fire on long chain even when cycle is also present; \
             all failures: {failures:?}"
        );
    }

    #[test]
    fn atk_a3_orphan_child_without_antigen_declaration_is_surfaced() {
        // ATK-A3-ORPHAN-CHILD (adversarial BUG-A3-002): lineage edge where the
        // CHILD has no corresponding `#[antigen]` declaration. The user wrote
        // `#[descended_from(Parent)]` on a struct that is NOT itself an antigen.
        //
        // `orphaned_lineage_edges()` only checks if the PARENT is in the
        // known-antigens set; this is its dual case. A child-without-antigen
        // is structurally incoherent: it claims inheritance into the antigen
        // system without being a participant.
        //
        // Contract: this must be surfaced via SOME query channel —
        // `dangling_child_lineage_edges()` (the chosen channel),
        // `orphaned_lineage_edges()`, or `parse_failures`. Pathmaker chose
        // a separate `dangling_child_lineage_edges()` method (parallel to
        // `orphaned_lineage_edges`) because the channel separation
        // is structurally cleaner per ADR-006 (recognition-not-design).
        //
        // The ADR-018 propagation walk (D1.5) skips edges flagged by either
        // query method — both produce the same effect on inheritance
        // resolution.
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("Parent"));
        // OrphanChild is NOT in antigens — only in lineage_edges.
        report.lineage_edges.push(edge("OrphanChild", "Parent"));

        let orphans = report.orphaned_lineage_edges();
        let dangling = report.dangling_child_lineage_edges();
        assert!(
            !orphans.is_empty() || !dangling.is_empty() || !report.parse_failures.is_empty(),
            "lineage edge whose child has no #[antigen] declaration must be \
             surfaced via orphaned_lineage_edges, dangling_child_lineage_edges, or \
             parse_failures; got orphans: {orphans:?}, dangling: {dangling:?}"
        );

        // Specific assertion: pathmaker chose dangling_child_lineage_edges as the
        // channel. The orphan-channel must NOT also surface this case
        // (parent IS in antigens, so it's not a parent-orphan).
        assert!(
            orphans.is_empty(),
            "child-missing case must NOT appear in orphaned_lineage_edges \
             (that channel is for parent-missing); got: {orphans:?}"
        );
        assert_eq!(
            dangling.len(),
            1,
            "child-missing must appear in dangling_child_lineage_edges, exactly one"
        );
        assert_eq!(dangling[0].child, "OrphanChild");
        assert_eq!(dangling[0].parent, "Parent");
    }
}
