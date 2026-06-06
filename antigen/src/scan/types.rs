//! Scan data model — the types `scan_workspace` produces and `audit` consumes.
//!
//! Extracted from the former monolithic `scan.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). This module holds every `pub` scan type
//! (`ScanReport` + the per-declaration records + the `*Kind` enums + the
//! coverage/partition types), their query methods (`ScanReport`'s impl), and the
//! shared matching rule (`canonical_paths_match` / `defense_addresses` / the
//! `addresses_for_*` helpers — the single source of truth for "does this defense
//! cover this site", routed through by both the scan query API and the audit
//! cross-checks). Moving types before the passes (extraction step 5) lets every
//! later pass module compile against a stable `types`.
//!
//! API-invisible: re-exported from the `scan` module root via `pub use`, so
//! `antigen::scan::ScanReport` etc. resolve byte-for-byte as before.

use std::path::PathBuf;

use antigen_macros::{antigen_tolerance, presents};
use serde::{Deserialize, Serialize};

/// A single antigen declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[presents(VecCardinalityMasqueradingAsSet)]
#[antigen_tolerance(
    VecCardinalityMasqueradingAsSet,
    rationale = "Accepted: `category` is a Vec modeling a set (each AntigenCategory meaningful at \
                 most once), so it structurally presents the masquerade shape. The duplicate-injection \
                 risk is DEFENDED UPSTREAM at the declaration boundary — AntigenArgs::validate() in \
                 antigen-macros rejects duplicate category variants at parse-time (fixed 30e10e6, pinned \
                 by antigen_parser_duplicate_category_in_array_is_rejected). It cannot be marked \
                 #[immune] here because the defense + witness live in the proc-macro crate, which the \
                 scanned struct's crate cannot reference (dependency-cycle + proc-macro-self-application \
                 constraints — see MarkerStructDeadCodeInBinary). So this scanned representation tolerates \
                 the shape; the real guard is one layer up at macro-validate.",
    until = "v0.3"
)]
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
    /// Category variants from `category = AntigenCategory::X` (ADR-028).
    ///
    /// Empty vec means absent (v0.1 backward-compat; audit tool emits
    /// `antigen-category-defaulted-implicit-functional` migration hint).
    /// Single-element = pure substrate-alignment or functional-correctness.
    /// Two elements = hybrid antigen requiring both witness types.
    #[serde(default)]
    pub category: Vec<crate::category::AntigenCategory>,
    /// Authored provenance claim from `provenance = Provenance::X` (ADR-039 §C),
    /// stored as the variant string (e.g. `"Heuristic"`) for forward-compat —
    /// same posture as `category`. `None` ⇒ the audit defaults it to `Imagined`
    /// (the lowest tier; an unlabeled antigen is the weakest claim) and may emit a
    /// provenance-defaulted hint. This is the AUTHORED claim the audit
    /// tier-VERIFIER checks; it sets the floor the dial-derived confidence tier
    /// may graduate from (the confidence tier itself is never authored).
    #[serde(default)]
    pub provenance: Option<String>,
    /// Authored presentation axis from `presentation = Presentation::X`
    /// (ADR-039 §C), stored as the variant string (`"Passive"` / `"Active"`).
    /// `None` ⇒ defaults to `Passive` (the passive-by-default-for-low-provenance
    /// rule — an imagined antigen costs nothing until someone encounters one).
    #[serde(default)]
    pub presentation: Option<String>,
}

impl AntigenDeclaration {
    /// Resolve the authored `provenance` claim to a typed
    /// [`Provenance`](crate::finding::Provenance) (ADR-039 §C). An absent or
    /// unknown-variant claim resolves to the honest FLOOR
    /// [`Provenance::DEFAULT`](crate::finding::Provenance::DEFAULT) (`Imagined`) —
    /// the mandatory-with-default invariant: a permissive catalog is trustworthy
    /// only because the label is always present, and the default can never
    /// over-claim (it is the lowest tier). The parse-time macro rejects unknown
    /// variants, so an unknown here is only reachable from a
    /// hand-written/forward-incompatible scan record; it too resolves to the floor.
    #[must_use]
    pub fn resolved_provenance(&self) -> crate::finding::Provenance {
        self.provenance
            .as_deref()
            .and_then(crate::finding::Provenance::from_variant_str)
            .unwrap_or(crate::finding::Provenance::DEFAULT)
    }

    /// Resolve the authored `presentation` axis to a typed
    /// [`Presentation`](crate::finding::Presentation) (ADR-039 §A). Absent ⇒
    /// [`Presentation::DEFAULT`](crate::finding::Presentation::DEFAULT) (`Passive`,
    /// the passive-by-default-for-low-provenance rule).
    #[must_use]
    pub fn resolved_presentation(&self) -> crate::finding::Presentation {
        self.presentation
            .as_deref()
            .and_then(crate::finding::Presentation::from_variant_str)
            .unwrap_or(crate::finding::Presentation::DEFAULT)
    }

    /// Whether the author explicitly supplied a `provenance` claim (vs relying on
    /// the `Imagined` default). The audit layer uses this to emit the
    /// provenance-defaulted-implicit hint (the category-defaulted-implicit
    /// precedent) — surfaced as a migration nudge, never a gate.
    #[must_use]
    pub fn provenance_is_explicit(&self) -> bool {
        self.provenance
            .as_deref()
            .is_some_and(|s| crate::finding::Provenance::from_variant_str(s).is_some())
    }
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
    /// An enum variant carrying its own attribute (e.g. `#[presents]` on the
    /// `External` variant of `enum RequestKind`). Holds the enclosing enum
    /// identifier and the variant identifier so two variants of the same name
    /// on different enums do not collide. ATK-A2-ENUM-VARIANT: without a
    /// `visit_variant` override the scanner silently ignored variant-level
    /// attributes — a presentation invisible to failure-class memory.
    EnumVariant {
        /// The enclosing enum identifier.
        enum_name: String,
        /// The variant identifier.
        variant_name: String,
    },
    /// An associated `const` inside an `impl` block (e.g.
    /// `#[presents]` on `impl Parser { const MAX_INPUT_BYTES … }`). Mirrors
    /// [`Self::ImplFn`]: carries the enclosing impl's trait (if any) + type +
    /// the const name. ATK-A2-IMPL-CONST: without a `visit_impl_item_const`
    /// override the scanner silently ignored impl-const attributes — the same
    /// blind-spot class as [`Self::EnumVariant`].
    ImplConst {
        /// Trait of the enclosing impl, if any.
        trait_path: Option<String>,
        /// Type of the enclosing impl.
        target_type: String,
        /// The associated-const name.
        const_name: String,
    },
    /// A free-standing (top-level or module-level) `const` item carrying its own
    /// attribute (e.g. `#[presents] const MAX_REQUEST_SIZE: usize = …`). Holds
    /// the const identifier. ATK-A2-TOPLEVEL-CONST: same scanner blind-spot
    /// class as [`Self::EnumVariant`] / [`Self::ImplConst`] — a missing
    /// `visit_item_const` override let the attribute pass unscanned.
    Const(String),
    /// A free-standing `static` item carrying its own attribute (e.g.
    /// `#[presents] static GLOBAL_LIMIT: usize = …`). Distinct from
    /// [`Self::Const`] so a `static` and a `const` of the same name do not
    /// collide. Closed preemptively alongside the const cases (ADR-007:
    /// the same scanner blind-spot class — a missing `visit_item_static`
    /// override would otherwise let the attribute pass unscanned).
    Static(String),
    /// A C-like `union` item carrying its own attribute (e.g.
    /// `#[presents] union Layout { … }`). Closed alongside const/static
    /// as the same scanner blind-spot class applies — a missing
    /// `visit_item_union` override let the attribute pass unscanned.
    Union(String),
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
            Self::Struct(n)
            | Self::Enum(n)
            | Self::Trait(n)
            | Self::Fn(n)
            | Self::TypeAlias(n)
            | Self::Const(n)
            | Self::Static(n)
            | Self::Union(n) => n.clone(),
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
            Self::EnumVariant {
                enum_name,
                variant_name,
            } => format!("{enum_name}::{variant_name}"),
            Self::ImplConst {
                trait_path: Some(t),
                target_type,
                const_name,
            } => format!("<{target_type} as {t}>::{const_name}"),
            Self::ImplConst {
                trait_path: None,
                target_type,
                const_name,
            } => format!("{target_type}::{const_name}"),
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
            | (Self::TypeAlias(a), Self::TypeAlias(b))
            | (Self::Const(a), Self::Const(b))
            | (Self::Static(a), Self::Static(b))
            | (Self::Union(a), Self::Union(b)) => a == b,
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
            (
                Self::EnumVariant {
                    enum_name: e1,
                    variant_name: v1,
                },
                Self::EnumVariant {
                    enum_name: e2,
                    variant_name: v2,
                },
            ) => e1 == e2 && v1 == v2,
            (
                Self::ImplConst {
                    target_type: t1,
                    const_name: c1,
                    ..
                },
                Self::ImplConst {
                    target_type: t2,
                    const_name: c2,
                    ..
                },
            ) => normalize_type_name(t1) == normalize_type_name(t2) && c1 == c2,
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
    /// FNV-1a structural digest of the presented item at scan time.
    /// Populated for `FingerprintMatch` presentations; empty string for
    /// `ExplicitMarker` presentations and inherited presentations where the
    /// ancestor was an explicit marker. Allows adopters to pass this value
    /// directly to `attest scaffold --fingerprint` without needing an
    /// `#[immune]` marker first (DX finding 6).
    #[serde(default)]
    pub structural_fingerprint: String,
    /// Substrate-witness predicate JSON folded onto the presents-site via
    /// `#[presents(X, requires = <predicate>)]` (ADR-029 R5 — the substrate-tier
    /// migration target for `#[immune(requires=...)]`). `Some` only when the
    /// presents-site carries site-attached substrate evidence. The audit
    /// evaluates this against `.attest/` sidecars to grade the immune-state
    /// verdict.
    ///
    /// `#[serde(default)]` so pre-ADR-029 reports deserialize cleanly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_predicate: Option<String>,
    /// Phantom-type proof expression folded onto the presents-site via
    /// `#[presents(X, proof = <expr>)]` (ADR-029 R5 — the phantom-tier migration
    /// target for `#[immune(witness = <phantom>)]`), rendered as its token
    /// string (e.g. `NonPanickingProof :: < T > :: verified`). The audit
    /// recognizes the phantom shape structurally and grades `FormalProof`.
    ///
    /// `#[serde(default)]` so pre-ADR-029 reports deserialize cleanly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,
}

/// An `#[immune(antigen_type, witness = ...)]` declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Immunity {
    /// The antigen type referenced.
    pub antigen_type: String,
    /// The witness expression as a string (validated lazily).
    /// Empty string when `requires_predicate` is set (substrate-witness path).
    pub witness: String,
    /// Substrate-witness predicate JSON, present when the immunity was
    /// declared with `requires = <predicate>` (ADR-019 §P3b). The JSON
    /// matches `serde_json::to_string(&antigen_attestation::Predicate)`.
    /// Mutually exclusive with a non-empty `witness`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_predicate: Option<String>,
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
    /// Structural digest of the defended item's source, computed via
    /// [`antigen_fingerprint::structural_digest`]. This is the value an
    /// adopter signs against (`signed_against_fingerprint`); audit recomputes
    /// it to detect drift for `against = "current"` / `fresh_within_days`
    /// (ADR-019). Distinct from the antigen *pattern* fingerprint — this is a
    /// per-item content hash of the immune site. Empty only on the legacy
    /// deserialization path (pre-this-field reports); always populated by scan.
    #[serde(default)]
    pub structural_fingerprint: String,
}

/// A `#[defended_by(antigen_type)]` code-tier witness registration discovered
/// in source (ADR-029).
///
/// Where [`Immunity`] (the deprecated `#[immune]`) bundled the immunity-claim
/// (a verdict) with the witness pointer at the *defended site*, a `Defense`
/// carries only the registration: "this test/proptest function declares it
/// defends failure-class X." The verdict — whether the witness actually defends
/// a `#[presents(X)]` site, and at what tier — is computed by
/// `cargo antigen audit` cross-referencing the registration to the sites it
/// covers. Immunity is observed, not declared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defense {
    /// The antigen type the witness declares it defends (last path segment).
    pub antigen_type: String,
    /// Source file path of the witness function.
    pub file: PathBuf,
    /// Line number of the `#[defended_by]` attribute.
    pub line: usize,
    /// Item kind that was annotated (typically `fn`).
    pub item_kind: String,
    /// Item identity of the witness function. For a `#[defended_by]` site this
    /// is the *witness*, not the defended site — the cross-reference to defended
    /// sites is computed at audit time.
    pub item_target: ItemTarget,
    /// Canonical declaration site of the *antigen* this witness defends (ADR-017),
    /// `Some("<crate>@<version>")` for a cross-crate antigen, `None` for an
    /// intra-workspace one (set by the `--include-deps` driver post-scan, like
    /// [`Immunity::canonical_path`]). Cross-reference matching uses the
    /// `(antigen_type, canonical_path)` tuple so a `#[defended_by(Foo)]` in one
    /// crate does NOT silently satisfy a same-bare-name `#[presents(Foo)]` from a
    /// DIFFERENT crate (ATK-ADR029-21 / ATK-G2-22 cross-crate overclaim). A
    /// `None` `canonical_path` matches any (backward-compat: a defense that hasn't
    /// been canonical-stamped behaves as before).
    #[serde(default)]
    pub canonical_path: Option<String>,
}

/// An `#[antigen_generates(antigen_type, rationale = "...")]` declaration
/// discovered on a macro DEFINITION (ADR-014).
///
/// The macro author declares "my macro emits code presenting `antigen_type`."
/// The connection key is [`Self::macro_name`] — the identifier used at the
/// macro INVOCATION site:
/// - a `#[proc_macro_derive(Name)]` registers `Name` (matches `#[derive(Name)]`),
/// - a `#[proc_macro_attribute]` registers the annotated fn's name (matches
///   `#[that_name]`),
/// - a `macro_rules! name` registers `name` (matches `name!(...)`).
///
/// `cargo antigen scan`'s generates-synthesis pass builds a `macro_name →
/// [antigen_type]` index from these declarations, then walks every macro
/// invocation in the workspace and emits a synthetic [`Presentation`] at the
/// invocation site for each matching generator. Per ADR-014 §A3 this is
/// same-workspace only; cross-crate macro-output recognition (§A4) is deferred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratesDeclaration {
    /// The antigen type the macro's expansion presents (last path segment).
    pub antigen_type: String,
    /// The macro author's justification (required, non-empty per ADR-014).
    pub rationale: String,
    /// The macro identifier this declaration registers as a generator — the
    /// name used at invocation sites. See the type doc for resolution rules.
    pub macro_name: String,
    /// Source file path of the macro definition.
    pub file: PathBuf,
    /// Line number of the `#[antigen_generates]` attribute.
    pub line: usize,
    /// Canonical declaration site of the *antigen* (ADR-017); `None` for
    /// intra-workspace (set by the `--include-deps` driver post-scan, like
    /// [`Defense::canonical_path`]).
    #[serde(default)]
    pub canonical_path: Option<String>,
}

/// A marked-unknown declaration (`#[aura]` / `#[dread]` / `#[red_flag]`).
///
/// A felt-but-unnamed danger marked at a site (ADR-041) — off the dial's
/// classification axis (at ⊥), on the magnitude × existence-certainty plane. The
/// author supplied only the **required** `trigger`; the plane corner is fixed by
/// which marker macro was used.
///
/// This is the scan-time half of ADR-039's `Finding` (it emits as a
/// `FindingBody::MarkedUnknown`). It surfaces at the dial's non-gating floor —
/// never gates, never nags; an untouched marker is a *mild* substrate-smell
/// (`#[red_flag]` auto-escalates).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkedUnknown {
    /// Which marker: `"aura"` / `"dread"` / `"red-flag"`.
    pub marker: String,
    /// The magnitude corner (`"aura"` / `"dread"` — `"smell"` is absorbed),
    /// fixed by the marker macro. Maps to `finding::Magnitude`.
    pub magnitude: String,
    /// The existence-certainty corner (`"unsure"` / `"sure"`), fixed by the
    /// marker macro. A FIRST-CLASS field (ADR-041) — NOT folded into the dial
    /// tier, so the maturation engine can cluster high-certainty `red_flag`s
    /// apart from low-certainty `aura`s. Maps to `finding::ExistenceCertainty`.
    pub existence_certainty: String,
    /// The required felt-trigger ("what did you see?"), non-empty (guard 3).
    pub trigger: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number of the marker attribute.
    pub line: usize,
}

impl MarkedUnknown {
    /// Convert a scanned marked-unknown into the unified
    /// [`Finding`](crate::finding::Finding) schema
    /// (ADR-039 §C, the scan-time half) — the marker wave's emit (ADR-041
    /// §Emit-seam). A marked-unknown is an authored mark at a site the author
    /// *encountered*, so it carries `class_provenance = Encountered` +
    /// `presentation = Active` (the author chose to mark their own site, ADR-041);
    /// `origin_stage = Scan`. Severity follows the magnitude × existence-certainty
    /// plane: a `red_flag` (Sure) auto-escalates to High; otherwise Dread → High,
    /// Aura/Smell → Medium/Low.
    #[must_use]
    pub fn to_finding(&self, timestamp: u64) -> crate::finding::Finding {
        use crate::finding::{
            ExistenceCertainty, FINDING_SCHEMA_VERSION, Finding, FindingBody, Magnitude,
            OriginStage, Presentation, Provenance, Severity, cluster_key_of,
        };

        let magnitude = Magnitude::from_variant_str(&self.magnitude).unwrap_or(Magnitude::Aura);
        let existence_certainty = ExistenceCertainty::from_variant_str(&self.existence_certainty)
            .unwrap_or(ExistenceCertainty::Unsure);

        // The red_flag corner (Sure) auto-escalates on first match (ADR-041) → High;
        // otherwise severity tracks magnitude (Dread → High, Aura → Medium, Smell → Low).
        let severity = match (existence_certainty, magnitude) {
            (ExistenceCertainty::Sure, _) | (_, Magnitude::Dread) => Severity::High,
            (_, Magnitude::Aura) => Severity::Medium,
            (_, Magnitude::Smell) => Severity::Low,
        };

        // The cluster-key derives from (structural_digest, class); for a
        // marked-unknown the "class" is the marker name (the maturation engine
        // clusters related marks of the same marker-kind). No structural digest at
        // the marker site (it is an authored mark, not a matched item).
        let cluster_key = cluster_key_of("", &self.marker);

        Finding {
            schema_version: FINDING_SCHEMA_VERSION,
            file: self.file.to_string_lossy().into_owned(),
            line: self.line,
            structural_digest: String::new(),
            cluster_key,
            severity,
            source: format!("scan:marked-unknown:{}", self.marker),
            // Authored mark at an encountered site (ADR-041): encountered + active.
            class_provenance: Provenance::Encountered,
            presentation: Presentation::Active,
            timestamp,
            origin_stage: OriginStage::Scan,
            body: FindingBody::MarkedUnknown {
                magnitude,
                existence_certainty,
                trigger: self.trigger.clone(),
            },
        }
    }
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
    /// Substrate-witness sidecar predicate JSON, present when the tolerance
    /// was declared with `requires = <predicate>` (ADR-019 tolerance tier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_predicate: Option<String>,
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
    /// Structural digest of the tolerated item's source — the
    /// `signed_against_fingerprint` value for substrate-witness tolerance
    /// sidecars (ADR-019 tolerance tier). Mirrors [`Immunity::structural_fingerprint`].
    #[serde(default)]
    pub structural_fingerprint: String,
}

// ============================================================================
// Deferred-Defense Family output types (ADR-023)
// ============================================================================

/// Which of the four deferred-defense postures was declared.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeferredDefenseKind {
    /// `#[anergy]` — deferred-but-muted; until required.
    Anergy,
    /// `#[immunosuppress]` — surgical silencing with duration cap.
    Immunosuppress,
    /// `#[poxparty]` — intentional controlled exposure; cfg-gated.
    Poxparty,
    /// `#[orient]` — see-also context; lightest-weight.
    Orient,
}

/// A deferred-defense declaration discovered in source (ADR-023).
///
/// Covers all four primitives: `#[anergy]`, `#[immunosuppress]`,
/// `#[poxparty]`, `#[orient]`. The `kind` field distinguishes them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredDefense {
    /// Which deferred-defense posture was declared.
    pub kind: DeferredDefenseKind,
    /// Antigen type referenced, if a positional argument was provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub antigen_type: Option<String>,
    /// Primary text field: `rationale` (immunosuppress), `reason` (anergy),
    /// `exercise_type` (poxparty), or empty string (orient).
    /// For anergy: `reason`; for immunosuppress: `rationale`;
    /// for poxparty: `exercise_type`; for orient: empty string.
    pub text: String,
    /// Expiry date string (`until` field), if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    /// Optional co-stimulation hint (anergy only; advisory).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_co_stimulation: Option<String>,
    /// Optional signer identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_by: Option<String>,
    /// See-also references (orient; also poxparty name field stored here).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub see: Vec<String>,
    /// `#[immunosuppress(since = "YYYY-MM-DD")]` — the suppression start date,
    /// as a typed field (was previously stuffed into `see[]` as a `"since:DATE"`
    /// string tag, which the audit could never parse — the
    /// `ImmunosuppressDurationCapExceeded`-unreachable root cause). The audit
    /// computes elapsed days from this to enforce the duration cap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// `#[immunosuppress(duration_cap = N)]` — the maximum allowed suppression
    /// duration in days, as a typed field (was a `"duration_cap:Nd"` `see[]`
    /// string tag). When `since + duration_cap` is in the past, the audit emits
    /// `ImmunosuppressDurationCapExceeded` — the cap was unenforceable at audit
    /// time while this lived only as an unparsed string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_cap: Option<u64>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated (fn, impl, struct, etc.).
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
}

// ============================================================================
// Convergent-Evidence Family output types (ADR-024)
// ============================================================================

/// Which of the seven convergent-evidence primitives was declared.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConvergentEvidenceKind {
    /// `#[diagnostic(modalities = [...], min_independent = N)]`.
    Diagnostic,
    /// `#[clonal(witness = ..., iterations = N, seed = SeedKind::...)]`.
    Clonal,
    /// `#[igg(witnesses = [...], historical_span = N, min_reattestations = N)]`.
    Igg,
    /// `#[crossreactive(fingerprints = [...])]`.
    Crossreactive,
    /// `#[polyclonal]` marker.
    Polyclonal,
    /// `#[monoclonal]` marker.
    Monoclonal,
    /// `#[adcc]` marker.
    Adcc,
}

/// A convergent-evidence declaration discovered in source (ADR-024).
///
/// Covers all seven primitives. The `kind` field distinguishes them; the
/// rest of the fields are loosely-typed string captures shared across
/// kinds for forward-compat with the adoption gradient (per ADR-009).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergentEvidence {
    /// Which convergent-evidence primitive was declared.
    pub kind: ConvergentEvidenceKind,
    /// `#[diagnostic]` modality classes — the final segment of each
    /// `WitnessClass::*` path, e.g., `"StaticAnalysis"`. Empty for
    /// other kinds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modality_classes: Vec<String>,
    /// `#[diagnostic]` `min_independent` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_independent: Option<u64>,
    /// `#[clonal]` `witness` identifier (token string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness: Option<String>,
    /// `#[clonal]` `iterations` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u64>,
    /// `#[clonal]` `seed` final ident (e.g., `"Random"`, `"Fixed"`).
    /// `Fixed` here is itself a bug-signal — the proc-macro rejects it
    /// at parse time, but a scan over older source can still surface it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_kind: Option<String>,
    /// `#[igg]` historical span in days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical_span: Option<u64>,
    /// `#[igg]` minimum re-attestations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_reattestations: Option<u64>,
    /// `#[igg]` witness identifier strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witnesses: Vec<String>,
    /// `#[crossreactive]` fingerprint strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fingerprints: Vec<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated (fn, impl, struct, etc.).
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
}

/// Which recurrent-emergence primitive was declared (ADR-024 §Family 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecurrentKind {
    /// `#[itch]` — below-threshold noticing (cognitive-organizational).
    Itch,
    /// `#[recurrence_anchor]` — cross-substrate recurrence formally anchored
    /// (clinical-medicine).
    RecurrenceAnchor,
    /// `#[crystallize]` — itch-cluster promotion to named failure-class
    /// (cognitive-organizational).
    Crystallize,
    /// `#[chronic]` — low-level persistent NON-cross-substrate signal
    /// (immunology-proper).
    Chronic,
    /// `#[saturate]` — accumulating saturation evidence
    /// (cognitive-organizational).
    Saturate,
    /// `#[strand]` — thread of related noticing (cognitive-organizational).
    Strand,
}

/// A recurrent-emergence declaration discovered in source (ADR-024 §Family 2).
///
/// Covers all six primitives. The `kind` field distinguishes them; the rest
/// are loosely-typed optional captures shared across kinds for forward-compat
/// with the adoption gradient (per ADR-009), mirroring [`ConvergentEvidence`].
/// All members are antigen-category `SubstrateAlignment` per ADR-028.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrentDeclaration {
    /// Which recurrent primitive was declared.
    pub kind: RecurrentKind,
    /// `name` slug — `#[itch]`, `#[crystallize]`, `#[strand]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Antigen-type path final segment — `#[recurrence_anchor]`,
    /// `#[chronic]`, optional on `#[itch]`/`#[crystallize]`/`#[saturate]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub antigen_type: Option<String>,
    /// `description` / `summary` text — the human-facing noticing/rationale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// `#[recurrence_anchor]` instance count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instances: Option<u64>,
    /// `since` date-or-version — `#[recurrence_anchor]`, `#[chronic]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// `#[recurrence_anchor]` rationale text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// `#[crystallize]` `from_itches` ident strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub from_itches: Vec<String>,
    /// `#[strand]` `anchored_by` ident strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anchored_by: Vec<String>,
    /// `#[chronic]` `managed_by` role/team.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed_by: Option<String>,
    /// `#[saturate]` `contributing_to` slug.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributing_to: Option<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
}

/// Which mucosal-boundary primitive was declared (ADR-027 + Amendment 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MucosalKindTag {
    /// `#[mucosal]` — active boundary defense.
    Mucosal,
    /// `#[mucosal_delegate]` — defense delegated to a named handler.
    MucosalDelegate,
    /// `#[mucosal_tolerant]` — boundary intentionally permitted.
    MucosalTolerant,
}

/// A mucosal-boundary declaration discovered in source (ADR-027 + Amendment 1).
///
/// Covers all three primitives. The `tag` distinguishes them; the rest are
/// loosely-typed optional captures shared across kinds (forward-compat per
/// ADR-009), mirroring [`RecurrentDeclaration`]. `boundary_kind` holds the
/// final segment of the `MucosalKind::X` path (`"UserInput"` etc.).
/// All members are antigen-category `SubstrateAlignment` per ADR-028.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MucosalDeclaration {
    /// Which primitive was declared.
    pub tag: MucosalKindTag,
    /// `MucosalKind::X` final segment — the boundary kind (`kind` on
    /// `#[mucosal]`/`#[mucosal_tolerant]`, `boundary` on `#[mucosal_delegate]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary_kind: Option<String>,
    /// `rationale` text (all three primitives).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// `#[mucosal_delegate]` `handled_by` path rendered to its final segment
    /// (the handler function name). Audit-time kind-matching (Change 5)
    /// resolves this against the workspace function index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by: Option<String>,
    /// `#[mucosal_tolerant]` `accepts` description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepts: Option<String>,
    /// `#[mucosal_tolerant]` `reviewed_by`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    /// `#[mucosal_tolerant]` `until` review-deadline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
}

/// Which prescriptive work-orchestration primitive was declared (ADR-033).
///
/// The eight clinical-named macros. The audit maps each to its structural
/// SHAPE (S1 role-workflow / S2 elimination / S3 ordering / S4 frame-only) via
/// [`PrescriptiveKind::shape`] — four shapes, eight names (ADR-033 §Decision 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrescriptiveKind {
    /// `#[panel]` — a battery of work-needs (S1 Role-workflow).
    Panel,
    /// `#[rx]` — a prescribed treatment (S1 Role-workflow).
    Rx,
    /// `#[refer]` — a referral to an external owner (S1 Role-workflow).
    Refer,
    /// `#[biopsy]` — a deep-investigation request (S1 Role-workflow).
    Biopsy,
    /// `#[ddx]` — a differential diagnosis: alternatives to rule out (S2 Elimination).
    Ddx,
    /// `#[triage]` — a re-validatable priority ordering (S3 Ordering).
    Triage,
    /// `#[culture]` — a time-boxed test/observation (S4 Frame-only).
    Culture,
    /// `#[quarantine]` — an isolated region under a time-boxed hold (S4 Frame-only).
    Quarantine,
}

/// The four structural shapes the eight prescriptive names route to (ADR-033
/// §Decision 1).
///
/// Antigen ships four shape-parsers, not nine bespoke ones; the clinical names
/// are adopter-facing vocabulary distributed across the shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkShape {
    /// S1 — ordered who-steps + optional frame + a need-set (panel/rx/refer/biopsy).
    RoleWorkflow,
    /// S2 — a set of independently-closeable alternatives (ddx).
    Elimination,
    /// S3 — a re-validatable priority total-order (triage).
    Ordering,
    /// S4 — a temporal window with a satisfaction/expiry (culture/quarantine).
    FrameOnly,
}

impl PrescriptiveKind {
    /// The structural shape this clinical name routes to (ADR-033 §Decision 1).
    #[must_use]
    pub const fn shape(self) -> WorkShape {
        match self {
            Self::Panel | Self::Rx | Self::Refer | Self::Biopsy => WorkShape::RoleWorkflow,
            Self::Ddx => WorkShape::Elimination,
            Self::Triage => WorkShape::Ordering,
            Self::Culture | Self::Quarantine => WorkShape::FrameOnly,
        }
    }
}

/// A prescriptive work-orchestration declaration discovered in source (ADR-033).
///
/// Covers all eight primitives. The `kind` field distinguishes them (and maps to
/// a [`WorkShape`] via [`PrescriptiveKind::shape`]); the rest are loosely-typed
/// optional captures shared across kinds for forward-compat (ADR-009), mirroring
/// [`RecurrentDeclaration`]. Scan is recall-tuned (ADR-010): every field is
/// optional here; per-kind required-field validation lives at the audit layer
/// (and at parse-time in the macros). All members are antigen-category
/// `SubstrateAlignment`+`FunctionalCorrectness` per ADR-024/ADR-028.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrescriptiveDeclaration {
    /// Which prescriptive primitive was declared.
    pub kind: PrescriptiveKind,
    /// `needs` (panel) / `rule_out` (ddx) / `priority_order` (triage) — the
    /// shape's required list. Held as one field because exactly one of the three
    /// is meaningful per `kind`; the audit reads it through the kind's shape.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// `who`-refs that fill the work (panel/rx `filled_by`; refer `to`; biopsy
    /// `deep_investigation_by`; ddx `investigator`; triage `triaged_by`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filled_by: Vec<String>,
    /// `who`-refs that review the work (panel/rx `reviewed_by`; ddx `reviewer`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviewed_by: Vec<String>,
    /// `who`-ref that ordered the work (panel `ordered_by`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordered_by: Option<String>,
    /// The intrinsic temporal frame, if any (panel/rx `due`; refer `response_due`;
    /// triage `re_triage_due`; culture `runs_until`; quarantine `until`). ISO-8601.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    /// The primary free-text content of the need (rx `treatment`; biopsy
    /// `request_text`; ddx `symptom`; culture `test_kind`; quarantine `reason`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_text: Option<String>,
    /// A secondary opaque label (rx `diagnosis`; biopsy `location`; quarantine
    /// `scope`) — a v0.3 opaque label, not resolved (VOID-4b).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
    /// Structural digest of the annotated item AS SCANNED, computed by
    /// [`antigen_fingerprint::structural_digest`]. The audit pins who-step
    /// satisfaction to this fingerprint (NFA-21): an attestation that signed
    /// against an older fingerprint is stale and does NOT count toward
    /// fulfillment — the same freshness discipline immunity witnesses use
    /// (mirrors [`Immunity::structural_fingerprint`]).
    ///
    /// `#[serde(default)]` so reports serialized before this field deserialize
    /// cleanly with an empty fingerprint (the audit falls back to the sidecar's
    /// stored value when empty, the same legacy path as the immunity audit).
    #[serde(default)]
    pub structural_fingerprint: String,
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
    /// All discovered deferred-defense declarations: `#[anergy]`,
    /// `#[immunosuppress]`, `#[poxparty]`, `#[orient]`. ADR-023.
    ///
    /// `#[serde(default)]` so pre-v0.2 reports deserialize cleanly.
    #[serde(default)]
    pub deferred_defenses: Vec<DeferredDefense>,
    /// All discovered convergent-evidence declarations: `#[diagnostic]`,
    /// `#[clonal]`, `#[igg]`, `#[crossreactive]`, `#[polyclonal]`,
    /// `#[monoclonal]`, `#[adcc]`. ADR-024.
    ///
    /// `#[serde(default)]` so pre-v0.2 reports deserialize cleanly.
    #[serde(default)]
    pub convergent_evidences: Vec<ConvergentEvidence>,
    /// All discovered recurrent-emergence declarations: `#[itch]`,
    /// `#[recurrence_anchor]`, `#[crystallize]`, `#[chronic]`,
    /// `#[saturate]`, `#[strand]`. ADR-024 §Family 2.
    ///
    /// `#[serde(default)]` so pre-recurrent reports deserialize cleanly.
    #[serde(default)]
    pub recurrent_declarations: Vec<RecurrentDeclaration>,
    /// All discovered mucosal-boundary declarations: `#[mucosal]`,
    /// `#[mucosal_delegate]`, `#[mucosal_tolerant]`. ADR-027 + Amendment 1.
    ///
    /// `#[serde(default)]` so pre-mucosal reports deserialize cleanly.
    #[serde(default)]
    pub mucosal_declarations: Vec<MucosalDeclaration>,
    /// All discovered prescriptive work-orchestration declarations: `#[panel]`,
    /// `#[rx]`, `#[refer]`, `#[biopsy]`, `#[ddx]`, `#[triage]`, `#[culture]`,
    /// `#[quarantine]`. ADR-033 (extends ADR-024). The audit projects each to a
    /// four-valued `WorkVerdict` (the board).
    ///
    /// `#[serde(default)]` so pre-prescriptive reports deserialize cleanly.
    #[serde(default)]
    pub prescriptive_declarations: Vec<PrescriptiveDeclaration>,
    /// All discovered `#[defended_by(X)]` code-tier witness registrations
    /// (ADR-029). Each records that a test/proptest function declares it
    /// defends a failure-class; `cargo antigen audit` cross-references these
    /// to the `#[presents(X)]` sites they cover to compute the immune-state
    /// verdict. Immunity is observed, not declared.
    ///
    /// `#[serde(default)]` so pre-ADR-029 reports deserialize cleanly.
    #[serde(default)]
    pub defenses: Vec<Defense>,
    /// All discovered `#[antigen_generates(X, ...)]` declarations on macro
    /// definitions (ADR-014). The generates-synthesis pass connects these to
    /// macro invocation sites and emits synthetic presentations.
    ///
    /// `#[serde(default)]` so pre-ADR-014 reports deserialize cleanly.
    #[serde(default)]
    pub generates_declarations: Vec<GeneratesDeclaration>,
    /// All discovered marked-unknown markers (`#[aura]` / `#[dread]` /
    /// `#[red_flag]`, ADR-041) — the felt-but-unnamed dangers. These are the
    /// scan-time half of ADR-039's `Finding`; they surface at the dial's
    /// non-gating floor (never gate, never nag). `#[serde(default)]` so
    /// pre-ADR-041 reports deserialize cleanly.
    #[serde(default)]
    pub marked_unknowns: Vec<MarkedUnknown>,
    /// Files scanned successfully.
    pub files_scanned: usize,
    /// Files that failed to parse.
    pub parse_failures: Vec<ParseFailure>,
    /// Member-aware scan coverage (v0.3): which workspace member crates were
    /// enumerated vs actually scanned. `None` for a flat
    /// [`scan_workspace`](crate::scan::scan_workspace) scan (which has no member concept) — preserves
    /// byte-identical JSON for flat-scan consumers via
    /// `skip_serializing_if`. `Some` only from
    /// [`scan_workspace_multi_crate`](crate::scan::scan_workspace_multi_crate).
    ///
    /// This is the substrate for **ignorance detection** (regulatory tier): a
    /// member that exists in the workspace but was NOT scanned is a region
    /// where `#[presents]` sites go *unseen* — ignored, not defended. The
    /// coverage record makes that frontier explicit so a downstream audit can
    /// surface it. The audit/verdict layer is ADR scope; this field is the
    /// floor it stands on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_coverage: Option<ScanCoverage>,
}

/// Member-aware scan coverage: the workspace member set vs the set actually
/// scanned. Produced by [`scan_workspace_multi_crate`](crate::scan::scan_workspace_multi_crate).
///
/// The complement (`enumerated_members` − `scanned_members`) is the
/// **ignorance frontier**: members whose `#[presents]` sites the scan never
/// reached. In the current `--workspace` happy path the two sets are equal
/// (every enumerated member is scanned), so the frontier is empty — but
/// recording both makes any future partial-coverage mode (a member filter, a
/// member whose scan errored) surface its unscanned members explicitly rather
/// than silently dropping them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanCoverage {
    /// Every workspace member `cargo metadata` reported, as ADR-017 canonical
    /// paths (`<name>@<version>`). Sorted for determinism.
    pub enumerated_members: Vec<String>,
    /// The members actually scanned (canonical paths). A member is here iff its
    /// per-member scan ran. Sorted for determinism.
    pub scanned_members: Vec<String>,
}

impl ScanCoverage {
    /// Members that were enumerated but NOT scanned — the ignorance frontier.
    /// Their `#[presents]` sites (if any) were never seen by this scan.
    ///
    /// The frontier is a **set**: each unscanned member appears at most once,
    /// even if `enumerated_members` contains a duplicate (degenerate input — a
    /// valid Cargo workspace cannot have two members sharing a `name@version`,
    /// but the data type carries no construction guard). De-duplicating here
    /// means a downstream ignorance audit reads "is this member unseen?" once
    /// per member, not once per accidental repeat (ATK-COV-2 decision,
    /// pathmaker 2026-06-01). Order follows first appearance in
    /// `enumerated_members` for determinism.
    #[must_use]
    pub fn unscanned_members(&self) -> Vec<&str> {
        let scanned: std::collections::HashSet<&str> =
            self.scanned_members.iter().map(String::as_str).collect();
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        self.enumerated_members
            .iter()
            .map(String::as_str)
            .filter(|m| !scanned.contains(m) && seen.insert(m))
            .collect()
    }

    /// True iff every enumerated member was scanned (the ignorance frontier is
    /// empty). The happy path for a full `--workspace` scan.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.unscanned_members().is_empty()
    }
}

/// A presentation that has no matching immunity declaration on the same item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaddressedPresentation {
    /// The presentation itself.
    pub presentation: Presentation,
    /// True if the antigen referenced is found in the scan report.
    pub antigen_known: bool,
}

/// Unaddressed presentations split by confidence tier.
///
/// `explicit` contains sites where a developer wrote `#[presents(X)]` —
/// high-specificity declared intent. `inferred` contains sites where no marker
/// was written but the item matched an antigen's fingerprint pattern — broad
/// structural signal that requires human triage before acting.
///
/// The CLI gates `--strict` on `explicit` only; `inferred` is informational.
/// Library callers building custom CI should apply the same distinction:
/// gate on `explicit`, triage `inferred`.
#[derive(Debug, Clone, Default)]
pub struct PartitionedPresentations {
    /// Sites marked with `#[presents(X)]` — declared, CI-gateable.
    pub explicit: Vec<UnaddressedPresentation>,
    /// Sites matching a fingerprint pattern — inferred, triage-first.
    pub inferred: Vec<UnaddressedPresentation>,
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
        // ADR-017 §addresses() semantics: known-antigen lookup uses the
        // canonical_path-aware tuple `(type_name, canonical_path)`.
        let known_antigens: std::collections::HashSet<(&str, Option<&str>)> = self
            .antigens
            .iter()
            .map(|a| (a.type_name.as_str(), a.canonical_path.as_deref()))
            .collect();

        let mut result = Vec::new();
        for p in &self.presentations {
            let has_matching_immunity =
                self.immunities.iter().any(|i| addresses_for_immunity(i, p));
            // W6a: tolerance acknowledges a presentation per ADR-011
            // §Mechanics. A site with `#[antigen_tolerance(X, ...)]` for
            // the same antigen on the same item is reported under
            // "tolerated", not "unaddressed".
            let has_matching_tolerance = self
                .tolerances
                .iter()
                .any(|t| addresses_for_tolerance(t, p));
            // ADR-029: a `#[defended_by(X)]` code-tier witness addresses a
            // `#[presents(X)]` site at the CLASS level (the witness declares it
            // defends X; it covers every X presents-site — same matching the
            // audit verdict uses). Without this, `unaddressed_presentations()`
            // and `audit().presentation_verdicts` DIVERGE: the verdict says
            // "defended" while this surface says "unaddressed" — the exact
            // `ParallelStateTrackersDiverge` shape. The two notions of
            // "addressed" must agree.
            let has_matching_defense = self.defenses.iter().any(|d| defense_addresses(d, p));
            if !has_matching_immunity && !has_matching_tolerance && !has_matching_defense {
                result.push(UnaddressedPresentation {
                    presentation: p.clone(),
                    antigen_known: known_antigens
                        .contains(&(p.antigen_type.as_str(), p.canonical_path.as_deref())),
                });
            }
        }
        result
    }

    /// Unaddressed presentations split by confidence tier.
    ///
    /// Equivalent to calling [`unaddressed_presentations`](Self::unaddressed_presentations)
    /// and partitioning by [`MatchKind`], but in a single pass and with doc
    /// guidance on what each bucket means for CI gates.
    ///
    /// See [`PartitionedPresentations`] for the distinction between
    /// `explicit` (CI-gateable) and `inferred` (human-triage).
    #[must_use]
    pub fn partitioned_presentations(&self) -> PartitionedPresentations {
        let mut out = PartitionedPresentations::default();
        for up in self.unaddressed_presentations() {
            match up.presentation.match_kind {
                MatchKind::ExplicitMarker => out.explicit.push(up),
                MatchKind::FingerprintMatch => out.inferred.push(up),
            }
        }
        out
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
        // ADR-017 + ADR-018 §Enforcement: orphan checks compare
        // `(type_name, canonical_path)` tuples, NOT bare names.
        // Two crates each declaring `Foo` would have the same `type_name`
        // but distinct `canonical_path` values; a tolerance for
        // `foo@1.0.0::Foo` is orphaned when only `foo@2.0.0::Foo` is in
        // scope, even though "Foo" appears in `self.antigens`.
        let known: std::collections::HashSet<(&str, Option<&str>)> = self
            .antigens
            .iter()
            .map(|a| (a.type_name.as_str(), a.canonical_path.as_deref()))
            .collect();
        self.tolerances
            .iter()
            .filter(|t| !known.contains(&(t.antigen_type.as_str(), t.canonical_path.as_deref())))
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
        // ADR-017 + ADR-018 §Enforcement: orphan check compares
        // `(type_name, canonical_path)` tuples. An edge with
        // `parent_canonical_path: Some("foo@1.0.0")` is satisfied ONLY by
        // an AntigenDeclaration with matching `type_name` AND matching
        // `canonical_path`. Bare-name equality alone allows cross-crate
        // name collision to silently mask orphans (ATK-A3-006).
        let known: std::collections::HashSet<(&str, Option<&str>)> = self
            .antigens
            .iter()
            .map(|a| (a.type_name.as_str(), a.canonical_path.as_deref()))
            .collect();
        self.lineage_edges
            .iter()
            .filter(|e| !known.contains(&(e.parent.as_str(), e.parent_canonical_path.as_deref())))
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
        // ADR-017 + ADR-018 §Enforcement: canonical_path-aware
        // comparison. Symmetric to `orphaned_lineage_edges` — the child
        // endpoint check uses the same tuple key.
        let known: std::collections::HashSet<(&str, Option<&str>)> = self
            .antigens
            .iter()
            .map(|a| (a.type_name.as_str(), a.canonical_path.as_deref()))
            .collect();
        self.lineage_edges
            .iter()
            .filter(|e| !known.contains(&(e.child.as_str(), e.child_canonical_path.as_deref())))
            .collect()
    }

    /// Stamp `canonical_path` (and `parent_canonical_path` /
    /// `child_canonical_path` on lineage edges) on every record in this
    /// report that does not already have one.
    ///
    /// ADR-017 (Option A — caller stamps post-scan). Called by the
    /// cargo-metadata-driven `--include-deps` driver after running
    /// [`scan_workspace`](crate::scan::scan_workspace) on a dependency crate root: the driver knows
    /// the dependency's canonical path (`"<crate-name>@<version>"`), but
    /// the directory scanner doesn't, so the driver stamps the canonical
    /// path on every record post-scan.
    ///
    /// **Idempotent + non-overwriting**: records whose `canonical_path`
    /// (or relevant lineage-edge endpoint) is already `Some(_)` are
    /// left unchanged. This protects records that were stamped during
    /// an earlier (e.g., nested) scan from being silently re-keyed.
    ///
    /// `crate_id` is expected to be in the ADR-017 format
    /// `"<crate-name>@<version>"` (e.g., `"serde@1.0.193"`); the method
    /// does not validate the format — that's the driver's responsibility.
    pub fn stamp_canonical_path(&mut self, crate_id: &str) {
        for a in &mut self.antigens {
            if a.canonical_path.is_none() {
                a.canonical_path = Some(crate_id.to_string());
            }
        }
        for p in &mut self.presentations {
            if p.canonical_path.is_none() {
                p.canonical_path = Some(crate_id.to_string());
            }
        }
        for i in &mut self.immunities {
            if i.canonical_path.is_none() {
                i.canonical_path = Some(crate_id.to_string());
            }
        }
        for t in &mut self.tolerances {
            if t.canonical_path.is_none() {
                t.canonical_path = Some(crate_id.to_string());
            }
        }
        for d in &mut self.defenses {
            // ADR-029 defenses are stamped like immunities so cross-crate scans
            // carry the canonical_path the (antigen_type, canonical_path) match
            // needs to avoid the bare-name overclaim (ATK-ADR029-21/ATK-G2-22).
            if d.canonical_path.is_none() {
                d.canonical_path = Some(crate_id.to_string());
            }
        }
        for g in &mut self.generates_declarations {
            // ADR-014 generates-declarations are stamped like defenses so the
            // antigen identity carries its declaring crate for cross-crate
            // macro-output recognition (§A4; the v0.3 synthesis is same-workspace).
            if g.canonical_path.is_none() {
                g.canonical_path = Some(crate_id.to_string());
            }
        }
        for e in &mut self.lineage_edges {
            // Both endpoints are stamped to the same crate_id when missing —
            // they're both intra-crate by construction at this point
            // (cross-crate edges land later when D1.5's propagation walk
            // discovers them). Each endpoint is independently None-guarded.
            if e.parent_canonical_path.is_none() {
                e.parent_canonical_path = Some(crate_id.to_string());
            }
            if e.child_canonical_path.is_none() {
                e.child_canonical_path = Some(crate_id.to_string());
            }
        }
    }

    /// Total count of antigen-related declarations found.
    #[must_use]
    pub const fn total_declarations(&self) -> usize {
        self.antigens.len()
            + self.presentations.len()
            + self.immunities.len()
            + self.tolerances.len()
    }
}

/// Whether two records share the ADR-017 "same locus" identity.
///
/// Implements the combined locus check from ADR-017
/// `§addresses()` semantics (decisions.md lines 3637-3645):
///
/// - intra-workspace (both `canonical_path` are `None`): same source file
/// - cross-crate (both `Some`): same `canonical_path`
/// - mixed (one `Some`, one `None`): NOT a match — different scan modalities
pub fn locus_matches(
    a_path: &std::path::Path,
    a_canonical: Option<&str>,
    b_path: &std::path::Path,
    b_canonical: Option<&str>,
) -> bool {
    match (a_canonical, b_canonical) {
        (None, None) => a_path == b_path,
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// Does this `Immunity` address this `Presentation`?
///
/// ADR-017 `§addresses()` — combined check of identity (`antigen_type` +
/// `canonical_path`) + item (`ItemTarget::addresses`) + locus.
fn addresses_for_immunity(i: &Immunity, p: &Presentation) -> bool {
    i.antigen_type == p.antigen_type
        && canonical_paths_match(i.canonical_path.as_deref(), p.canonical_path.as_deref())
        && i.item_target.addresses(&p.item_target)
        && locus_matches(
            i.file.as_path(),
            i.canonical_path.as_deref(),
            p.file.as_path(),
            p.canonical_path.as_deref(),
        )
}

/// Does this `Toleration` address this `Presentation`?
fn addresses_for_tolerance(t: &Toleration, p: &Presentation) -> bool {
    t.antigen_type == p.antigen_type
        && canonical_paths_match(t.canonical_path.as_deref(), p.canonical_path.as_deref())
        && t.item_target.addresses(&p.item_target)
        && locus_matches(
            t.file.as_path(),
            t.canonical_path.as_deref(),
            p.file.as_path(),
            p.canonical_path.as_deref(),
        )
}

/// Does this `Defense` (`#[defended_by(X)]`) address this `Presentation`?
/// (ADR-029.)
///
/// Unlike immunity/tolerance matching, a defense is **class-level** — a witness
/// for failure-class X defends every `#[presents(X)]` site, not one co-located
/// item — so this does NOT compare `item_target` (the witness is elsewhere).
///
/// It IS canonical-path-aware to prevent the cross-crate bare-name overclaim
/// (ATK-ADR029-21 / ATK-G2-22 / ATK-ADR029-23): a `#[defended_by(Foo)]` in
/// crate A must not silently satisfy a `#[presents(Foo)]` from crate B. The
/// match is plain equality on `canonical_path` — None matches None only
/// (intra-workspace, both unstamped), Some(x) matches Some(x) only (cross-crate
/// stamped, same path). The previous "None wildcards against any" rule
/// (ATK-ADR029-23) let an unstamped primary-workspace defense silently address
/// a stamped dep presentation, hiding the dep's undefended vulnerability in
/// `--include-deps` scans; `stamp_canonical_path` runs all-or-nothing per scan
/// so `(None defense, Some presentation)` is always a cross-boundary case that
/// must not match. The single source of truth for "does this defense cover this
/// site" — all three call sites (`unaddressed_presentations`, the verdict
/// computation, and the G2 cross-check) route through here so the matching
/// rule cannot drift.
#[must_use]
pub fn defense_addresses(d: &Defense, p: &Presentation) -> bool {
    // canonical_path equality via the shared helper (None == None for intra-workspace,
    // Some(x) == Some(x) for cross-crate stamped; None ≠ Some always).
    // See `canonical_paths_match` for the design rationale (ATK-ADR029-23 +
    // forward/shared-canonical-path-addresses-helper ruling).
    d.antigen_type == p.antigen_type
        && canonical_paths_match(d.canonical_path.as_deref(), p.canonical_path.as_deref())
}

/// Strict canonical-path equality check: does `item_canonical_path` match
/// `decl_canonical_path` under the None-means-intra-workspace rule?
///
/// **Semantics**: `None == None` (both intra-workspace, both unstamped) and
/// `Some(x) == Some(x)` (both dep-stamped, same crate path). `None ≠ Some`
/// always — an intra-workspace item cannot address a stamped dep declaration
/// (ATK-ADR029-23 + forward/shared-canonical-path-addresses-helper ruling).
///
/// This is the single source of truth for the canonical-path dimension of
/// any "does item X address antigen Y" check. All call sites (defense loop,
/// immunity loop, tolerance loop, G2 cross-check) must route through this
/// function so the matching rule cannot drift independently.
#[must_use]
pub fn canonical_paths_match(
    item_canonical_path: Option<&str>,
    decl_canonical_path: Option<&str>,
) -> bool {
    item_canonical_path == decl_canonical_path
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
pub const MAX_LINEAGE_DEPTH: usize = 64;
