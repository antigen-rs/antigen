//! The ONE typed `Finding` / event schema (ADR-039 §C — SOLE OWNER).
//!
//! This is the unified, queryable typed-event record that both the scan stage
//! and the audit stage emit into, **merged at the audit stage** (the last stage
//! that sees both halves). It is the external platform contract the three
//! downstream learning-loop organs (the affinity-maturation engine,
//! antigen-as-platform, the cytokine-signaling-network — all charter) subscribe
//! to. Per the captain's "emit, don't display" lock: the dial verdict and the
//! `#[dread]` / `#[aura]` markers MUST be queryable as structured output, not
//! merely rendered.
//!
//! **Schema ownership boundary (ADR-039 §C / ADR-036 §The two seams converge).**
//! ADR-039 is the SOLE OWNER of this schema (the record + its full field-list).
//! ADR-036 (the decomposition) owns only the orchestration + the *merge-locus*
//! (where the unified population is assembled — [`crate::pipeline`]) and cites
//! this module for the schema; it defines no `Finding` type of its own. Exactly
//! ONE schema definition (here) + ONE merge-locus (the pipeline coordinator) —
//! building a second `Finding` from a second field-list would be antigen's own
//! `ParallelStateTrackersDiverge` class at the schema level, which this boundary
//! forecloses.
//!
//! **Forward-compat contract (ADR-039 §C, adversarial finding #12).** The record
//! is the EXTERNAL contract; it is versioned via
//! [`Finding`](crate::finding::Finding)'s `schema_version` and **every future
//! field is additive + optional** (`#[serde(default)]`); external
//! consumers branch on `schema_version`. `ScanReport` already runs this exact
//! additive-optional discipline (the pre-v0.2-deserialize-cleanly pattern); the
//! contract inherits it. (ADR-041 already grew the schema once — it added
//! `existence_certainty` — proving it *will* grow.) Recognition, not design.
//!
//! The current shape was spiked to a compiling, running, serializing struct at
//! `roles/pathmaker/spikes/seam-spike`; this is the real-crate landing of that
//! shape under ADR-036/039/041.

use antigen_macros::dread;
use serde::{Deserialize, Serialize};

/// The current `Finding` schema version (ADR-039 §C). Bump on a breaking field
/// change; additive `#[serde(default)]` fields do NOT bump it. External consumers
/// branch on this.
pub const FINDING_SCHEMA_VERSION: u32 = 1;

/// Which pipeline stage emitted a [`Finding`] — the locked `origin-stage` field
/// (ADR-039 §C). Both stages emit into ONE schema; this records which half.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum OriginStage {
    /// Emitted at scan time (a `#[dread]` / `#[aura]` marked-unknown declaration).
    Scan,
    /// Emitted at audit time (a confidence-dial verdict on a classified site).
    Audit,
}

/// The confidence-dial tier — the calibration reading (ADR-039 decision A/B).
///
/// Distinct from [`Provenance`] (how solid the *class* is) and from the
/// instance-confidence posterior (how sure *this site* is). This is "how loud":
/// the innate/`Suspected` floor (shape-only, antigen's lysozyme) vs the
/// declared/`Named` loud tier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum DialTier {
    /// Innate / suspected — shape-only, soft, the non-gating lysozyme floor.
    Suspected,
    /// Named / confident — declared, graduated, loud.
    Named,
}

/// The PROVENANCE ladder (ADR-039 §C) — how we know this failure-CLASS exists.
///
/// A **mandatory** typed enum on every [`Finding`] (never an `Option`, never a
/// free string): the permissive admission (ADR-039 decision A) is trustworthy
/// *only* because this label is always present + honest. The honest tier IS the
/// Goodhart protection — a manufactured fiction can only ever hold
/// `Heuristic`/`Imagined`; it can never *be labeled* `Constructable`/`Encountered`
/// without a real demonstration, so it stays passive + visibly unproven.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Provenance {
    // --- VERIFIED CORE (Goodhart-safe; a fiction can never claim these) ---
    /// Seen in real code — the highest provenance.
    Encountered,
    /// A minimal case can be *built that verifiably exhibits* the failure
    /// (constructed-and-verified; wild-found NOT required). The V(D)J insight:
    /// instantiable, not merely narratable.
    Constructable,
    // --- UNVERIFIED tiers (passive-by-default, honestly labeled) ---
    /// A scannable tell that *correlates* with the failure without a
    /// verifiable-constructable demo (clippy-lint-style; correlational, NOT
    /// causal). A manufactured heuristic can ONLY ever claim this tier.
    Heuristic,
    /// Articulated from shape/reasoning — a class with a tell but no constructable
    /// demo yet. The lowest tier; a standing request to construct the demo that
    /// would graduate it into the verified core.
    Imagined,
}

impl Provenance {
    /// The default for an antigen that does NOT author a `provenance` (ADR-039
    /// §A/§C): `Imagined`, the LOWEST tier. An unlabeled antigen is the weakest
    /// claim — it can never over-claim by omission (defaulting to the floor, not
    /// the ceiling, is the honest-labeling invariant at the default).
    pub const DEFAULT: Self = Self::Imagined;

    /// Parse from the scanner's stored variant string (the `#[antigen]` attribute
    /// last-segment, e.g. `"Heuristic"` / `"Provenance::Heuristic"` last-seg).
    /// Returns `None` for an unknown variant.
    #[must_use]
    pub fn from_variant_str(s: &str) -> Option<Self> {
        match s {
            "Encountered" => Some(Self::Encountered),
            "Constructable" => Some(Self::Constructable),
            "Heuristic" => Some(Self::Heuristic),
            "Imagined" => Some(Self::Imagined),
            _ => None,
        }
    }
}

/// PASSIVE (tooling-side) vs ACTIVE (user-macro) presentation.
///
/// The first-class presentation axis (ADR-039 decision A): an imagined/
/// low-provenance antigen defaults `Passive` (tooling/scan-side, no user-macro
/// burden); `Active` is the user-facing macro chosen by an encounterer. This is
/// what makes permissive admission free — the field of imagined-but-never-
/// triggered antigens sits passive, costing nothing, until someone meets one.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Presentation {
    /// Tooling/scan-side; the default for imagined/low-provenance classes.
    Passive,
    /// User-macro; chosen by whoever encounters the thing (higher provenance can
    /// warrant it).
    Active,
}

impl Presentation {
    /// The default for an antigen that does NOT author a `presentation`
    /// (ADR-039 §A): `Passive` — tooling/scan-side, no user-macro burden. The
    /// vast field of imagined-but-never-triggered antigens sits passive, costing
    /// nothing until someone encounters one.
    pub const DEFAULT: Self = Self::Passive;

    /// Parse from the scanner's stored variant string.
    #[must_use]
    pub fn from_variant_str(s: &str) -> Option<Self> {
        match s {
            "Passive" => Some(Self::Passive),
            "Active" => Some(Self::Active),
            _ => None,
        }
    }
}

/// Universal severity (ADR-039 §C, adversarial finding #12) — on EVERY [`Finding`].
///
/// The cytokine-signaling organ (charter) routes by severity-as-priority, so a
/// named dial-verdict Finding must also carry a severity or it cannot be routed —
/// it is NOT a marked-unknown-plane-only field. The dread/aura plane (ADR-041)
/// adds `existence_certainty` *on top of* this universal severity; it does not
/// own it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Low priority (a lint-shaped smell, a passive heuristic).
    Low,
    /// Medium priority.
    Medium,
    /// High priority (a constructable/encountered named class, a `#[dread]`).
    High,
}

/// The marked-unknown MAGNITUDE axis (ADR-041) — `smell → aura → dread`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Magnitude {
    /// The lightest mark — a faint smell.
    Smell,
    /// A stronger, more-localized mark — an aura.
    Aura,
    /// The strongest authored mark — dread.
    Dread,
}

impl Magnitude {
    /// Parse from the scanner's stored kebab variant string (`"smell"` / `"aura"`
    /// / `"dread"` — the marker macro's fixed-corner value). `None` if unknown.
    #[must_use]
    pub fn from_variant_str(s: &str) -> Option<Self> {
        match s {
            "smell" => Some(Self::Smell),
            "aura" => Some(Self::Aura),
            "dread" => Some(Self::Dread),
            _ => None,
        }
    }
}

/// The marked-unknown EXISTENCE-CERTAINTY axis (ADR-041).
///
/// Orthogonal to the classification certainty the dial tier carries. A
/// first-class field (NOT folded into the dial tier) so the maturation engine
/// can cluster on it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ExistenceCertainty {
    /// Might-be-something (the dread corner: alarmed but unsure).
    Unsure,
    /// Sure-but-unnameable (the sentinel corner: act now, can't yet name it).
    Sure,
}

impl ExistenceCertainty {
    /// Parse from the scanner's stored kebab variant string (`"unsure"` /
    /// `"sure"` — the marker macro's fixed-corner value). `None` if unknown.
    #[must_use]
    pub fn from_variant_str(s: &str) -> Option<Self> {
        match s {
            "unsure" => Some(Self::Unsure),
            "sure" => Some(Self::Sure),
            _ => None,
        }
    }
}

/// What a [`Finding`] is ABOUT — a classified failure-class verdict (audit-time)
/// OR a marked-unknown declaration (scan-time). The schema carries BOTH halves so
/// the two stages emit into one population.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FindingBody {
    /// Scan-time: a marked-unknown declaration (`#[dread]` / `#[aura]`).
    MarkedUnknown {
        /// The magnitude axis (ADR-041): `smell → aura → dread`.
        magnitude: Magnitude,
        /// The orthogonal existence-certainty axis (ADR-041).
        existence_certainty: ExistenceCertainty,
        /// REQUIRED — not `Option` (ADR-041 guard 3): a triggerless dread is
        /// graffiti. The author states what they are afraid of.
        trigger: String,
    },
    /// Audit-time: a confidence-dial verdict on a classified site.
    DialVerdict {
        /// The failure-class this verdict is about.
        class: String,
        /// The dial tier reading (suspected / named).
        tier: DialTier,
    },
    /// Scan-time: a **structural fingerprint match** against a declared
    /// antigen's (or the bundled catalog's) fingerprint (ADR-043 §E / v0.4 E0).
    ///
    /// This is the honest claim-scope of a bundled-catalog or synthesis match: a
    /// **syntactic FACT** — "this site's structure matches a known failure-class
    /// fingerprint, at a calibrated tier" — and **NOT** an audited
    /// [`DialVerdict`](FindingBody::DialVerdict). A fingerprint match never
    /// asserts a defense was audited or that the site is all-clear; promoting a
    /// match to an audited verdict requires the audit stage (which sees the
    /// site's `#[defended_by]` / witness half). Keeping these two bodies distinct
    /// is the ADR-044 syntactic/semantic boundary made structural: the machine
    /// states what it matched, never that it ratified.
    FingerprintMatch {
        /// The failure-class (antigen name) whose fingerprint matched.
        class: String,
        /// The calibration tier of the match (`Suspected` for the shape-only
        /// floor; `Named` only when the matched fingerprint is precise enough to
        /// graduate). Bundled-catalog matches ride the catalog's authored tier.
        tier: DialTier,
    },
}

/// THE ONE typed `Finding` / event record (ADR-039 §C).
///
/// Both stages emit into this; the audit stage merges. This is the wire-format
/// the three charter learning-organs subscribe to — the queryable, structured
/// output the "emit, don't display" lock requires (not a rendered string).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    /// The schema version (ADR-039 §C). The EXTERNAL contract is versioned; every
    /// future field is additive + optional (`#[serde(default)]`) and external
    /// consumers branch on this. Defaults to the current version on deserialize of
    /// any older record that predates the field.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Site file (the `site` field is `file` + `line`).
    pub file: String,
    /// Site line.
    pub line: usize,
    /// The item's FNV-1a structural fingerprint. **Scan already computes this and
    /// discards it before emit** — carrying it is negative-to-zero cost and lets
    /// the affinity-maturation engine (charter) cluster + diff by STRUCTURE rather
    /// than re-parsing every clustered site from `file` + `line`. May be empty
    /// when the emitter has no digest for the site.
    #[serde(default)]
    pub structural_digest: String,
    /// The **name-insensitive shape digest** of the item (ADR-045 Amd-2 —
    /// the two-field ruling). Distinct from `structural_digest` (the
    /// name+code-sensitive IDENTITY hash diff-native DETECT keys on): two items
    /// with identical bodies and different names share a `shape_digest` but NOT a
    /// `structural_digest`. The marked-unknown PROPOSE-slice clusters on shape, so
    /// this is the digest its `cluster_key` is derived from. Empty for emitters
    /// that have no shape digest (e.g. a matched-item Finding, where identity is
    /// the relevant key). Additive-optional (`#[serde(default)]`) — forward-compat.
    #[serde(default)]
    pub shape_digest: String,
    /// The cluster key — **specified** (ADR-039 §C), NOT an opaque label:
    /// `derived-from(<the clustering digest>, class)`. Makes "cluster by shared
    /// structure" a field-lookup, not a re-parse. Build it with [`cluster_key_of`].
    /// For a marked-unknown the clustering digest is the `shape_digest` (cluster by
    /// body shape); for a matched item it is the `structural_digest`.
    pub cluster_key: String,
    /// Universal severity (every Finding, for cytokine routing). Distinct from the
    /// dread plane's `existence_certainty`.
    pub severity: Severity,
    /// Emit-source provenance string (who/what stage produced it). Distinct from
    /// the CLASS-provenance ladder below.
    pub source: String,
    /// MANDATORY class-provenance (ADR-039 §C) — never `Option`: a permissive
    /// catalog is trustworthy only because this is always honest.
    pub class_provenance: Provenance,
    /// PASSIVE (tooling-side, default for imagined) vs ACTIVE (user-macro).
    pub presentation: Presentation,
    /// Timestamp (first-seen vs re-seen feeds the ADR-042 triage-state NEW signal).
    pub timestamp: u64,
    /// Which stage emitted it.
    pub origin_stage: OriginStage,
    /// The body — the marked-unknown half OR the dial-verdict half.
    pub body: FindingBody,
}

/// Backward-compat default for [`Finding::schema_version`] on records serialized
/// before the field existed.
const fn default_schema_version() -> u32 {
    FINDING_SCHEMA_VERSION
}

/// Derive a [`Finding::cluster_key`] from `(structural_digest, class)`.
///
/// The specified (not opaque) derivation (ADR-039 §C) — the single place the
/// cluster-key shape is defined, so "cluster by shared structure" stays a
/// field-lookup across every emitter.
// Dogfood mark (v0.4 keystone): the cluster key is a string concatenation of
// two free-form fields with a `@` delimiter that is NOT escaped out of either
// operand. If `class` ever contained `@`, or a digest were empty, two distinct
// `(class, digest)` pairs could format to the same key — a silent cluster-merge.
// This is not hypothetical: the `dread@` degenerate-key over-merge (an empty
// shape_digest collapsing every dread mark into one bucket) is exactly this shape.
// A stringly-typed identity that wants to be a delimiter-safe struct.
#[dread(
    trigger = "cluster_key_of builds an identity as `format!(\"{class}@{digest}\")` — a \
               raw concat with an unescaped `@`; an `@` in a class name or an empty \
               digest can collide two distinct (class,digest) pairs onto one key, \
               silently merging unrelated clusters (the `dread@` over-merge was this)."
)]
#[must_use]
pub fn cluster_key_of(structural_digest: &str, class: &str) -> String {
    format!("{class}@{structural_digest}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_serializes_kebab_cased_and_round_trips() {
        let f = Finding {
            schema_version: FINDING_SCHEMA_VERSION,
            file: "lib.rs".into(),
            line: 42,
            structural_digest: "fnv:drop-guard-before-flush".into(),
            shape_digest: "fnv:shape-drop-guard".into(),
            cluster_key: cluster_key_of("fnv:shape-drop-guard", "dread"),
            severity: Severity::High,
            source: "scan:declaration".into(),
            class_provenance: Provenance::Encountered,
            presentation: Presentation::Active,
            timestamp: 500,
            origin_stage: OriginStage::Scan,
            body: FindingBody::MarkedUnknown {
                magnitude: Magnitude::Dread,
                existence_certainty: ExistenceCertainty::Unsure,
                trigger: "the teardown drops the guard before the flush".into(),
            },
        };
        let json = serde_json::to_string(&f).expect("Finding serializes");
        // Enums render kebab-cased + the body is internally-tagged on `kind`.
        assert!(json.contains("\"origin_stage\":\"scan\""));
        assert!(json.contains("\"class_provenance\":\"encountered\""));
        assert!(json.contains("\"presentation\":\"active\""));
        assert!(json.contains("\"kind\":\"marked-unknown\""));
        let back: Finding = serde_json::from_str(&json).expect("Finding round-trips");
        assert_eq!(back, f);
    }

    #[test]
    fn schema_version_defaults_on_older_record() {
        // A record serialized before `schema_version` existed must deserialize
        // cleanly (the forward-compat additive-optional contract, ADR-039 §C).
        let older = r#"{
            "file": "a.rs", "line": 1, "cluster_key": "C@d", "severity": "low",
            "source": "scan:imagined-repertoire", "class_provenance": "imagined",
            "presentation": "passive", "timestamp": 0, "origin_stage": "scan",
            "body": {"kind": "dial-verdict", "class": "C", "tier": "suspected"}
        }"#;
        let f: Finding = serde_json::from_str(older).expect("older record deserializes");
        assert_eq!(f.schema_version, FINDING_SCHEMA_VERSION);
        assert_eq!(f.structural_digest, ""); // additive-optional default
        // An imagined class defaults passive — the permissive-admission rule, seen.
        assert_eq!(f.class_provenance, Provenance::Imagined);
        assert_eq!(f.presentation, Presentation::Passive);
    }

    #[test]
    fn provenance_default_is_the_floor_not_the_ceiling() {
        // The honest-labeling invariant at the default: an unlabeled antigen
        // resolves to the LOWEST tier (Imagined), never the verified core. This is
        // why omission can never over-claim (ADR-039 §A/§C).
        assert_eq!(Provenance::DEFAULT, Provenance::Imagined);
        assert_ne!(Provenance::DEFAULT, Provenance::Encountered);
        assert_ne!(Provenance::DEFAULT, Provenance::Constructable);
        assert_eq!(Presentation::DEFAULT, Presentation::Passive);
    }

    #[test]
    fn provenance_from_variant_str_round_trips_and_rejects_unknown() {
        for (s, want) in [
            ("Encountered", Provenance::Encountered),
            ("Constructable", Provenance::Constructable),
            ("Heuristic", Provenance::Heuristic),
            ("Imagined", Provenance::Imagined),
        ] {
            assert_eq!(Provenance::from_variant_str(s), Some(want));
        }
        assert_eq!(Provenance::from_variant_str("Bogus"), None);
        // kebab (the serde form) is NOT the scanner variant form — rejected.
        assert_eq!(Provenance::from_variant_str("heuristic"), None);
        assert_eq!(
            Presentation::from_variant_str("Passive"),
            Some(Presentation::Passive)
        );
        assert_eq!(
            Presentation::from_variant_str("Active"),
            Some(Presentation::Active)
        );
        assert_eq!(Presentation::from_variant_str("Bogus"), None);
    }

    #[test]
    fn magnitude_and_existence_certainty_from_variant_str() {
        // The marker macro stores the fixed-corner kebab values; the scan-half emit
        // re-parses them onto the Finding.
        assert_eq!(Magnitude::from_variant_str("smell"), Some(Magnitude::Smell));
        assert_eq!(Magnitude::from_variant_str("aura"), Some(Magnitude::Aura));
        assert_eq!(Magnitude::from_variant_str("dread"), Some(Magnitude::Dread));
        assert_eq!(Magnitude::from_variant_str("bogus"), None);
        assert_eq!(
            ExistenceCertainty::from_variant_str("unsure"),
            Some(ExistenceCertainty::Unsure)
        );
        assert_eq!(
            ExistenceCertainty::from_variant_str("sure"),
            Some(ExistenceCertainty::Sure)
        );
        assert_eq!(ExistenceCertainty::from_variant_str("bogus"), None);
    }
}
