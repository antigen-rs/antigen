//! Coverage / reachability audit — the ignorance frontier as a per-site verdict.
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). A pure detector module: a fn of `&ScanReport`
//! returning its own report; no detector calls another (single-conductor
//! invariant, ADR-036). API-invisible: re-exported from the `audit` root via
//! `pub use`.
//!
//! (The detailed first-principles rationale for the three-valued ignorance
//! frontier — observe-don't-declare, the principled {enumerate → parse → match}
//! cardinality, the detectability limit — is preserved inline below.)

use serde::{Deserialize, Serialize};

use crate::scan::ScanReport;

// ============================================================================
// Coverage / reachability audit — the ignorance frontier as a per-site verdict
// ============================================================================
//
// "Did the scanner reach + evaluate this site?" is one structural question that
// surfaces in three tiers (regulatory IGNORANCE, prescriptive
// OutOfFrame well-posedness, the v0.2 dx-dogfood ScannerBoundaryFalseNegative).
// Immunological ignorance is the 4th canonical peripheral-tolerance mechanism
// (Khan & Ghazanfar 2018): a functional self-antigen that the immune system
// never *encounters*. Its software cognate is exact — a real `#[presents]` site
// that the scanner never reaches. Tolerance-by-non-encounter, distinct from
// anergy (seen + disabled), deletion (removed), and suppression (held back).
//
// A first-principles point is decisive and shapes this layer: ignorance
// is the ONE state that is purely *observed* and NEVER *declarable*. A
// `#[ignorance]` site-macro would be the observe-don't-declare contradiction — to
// write it you'd have reached the site, so it would not be ignorant. There is
// therefore no site-macro; there is the failure-CLASS (an `IgnoranceUnreachedSite`
// antigen) and the audit VERDICT below, which this
// module emits when it can determine a site should-have-been-reached-but-was-not.
//
// THE CARDINALITY IS PRINCIPLED, NOT ENUMERATED-BY-LUCK. A site can be lost at
// exactly three points in the scanner pipeline {enumerate → parse → match}, in
// order — so non-reach has exactly three causes, each with a *different remedy*.
// The verdict carries the cause (never a bare reached/not bool, which would
// collapse three-causes-with-three-remedies into one undifferentiated "unreached"
// and lose the remedy-routing — the cardinality-collapse this whole arc fights).
//
// DETECTABILITY LIMIT (formalized from the observe-don't-declare principle). Non-
// reach is only detectable *relative to a reference that points into the lost
// region* — a `#[descended_from]` target, a cross-need reference, an
// `addresses()` target that resolves into an unreached site (a dangling
// *resolvable* reference). Absolute ignorance — a site nothing references and no
// scan-root reaches — is undetectable in principle (you cannot find what nothing
// points at and nothing scans). That is a structural honesty-limit, not an
// implementation gap; biology agrees (tolerance-by-ignorance in an immune-
// privileged site is dangerous precisely because it is un-purgeable without
// presenting the antigen = reaching the site).
//
// DUAL PROJECTION. The Barrier-cause frontier and the scan-coverage VALUE (the
// titer-kind "member one") are the SAME substrate — [`crate::scan::ScanCoverage`]'s
// two member sets — read two ways: this module yields the per-site VERDICT
// (`UnreachedSite { cause: Barrier }`); the report-envelope/coverage surface
// yields the workspace-level VALUE (`coverage = |scanned| / |enumerated|`).

/// Why the scanner never reached + evaluated a site.
///
/// The three variants partition the scanner pipeline `{enumerate → parse →
/// match}` at its three pre-evaluation drop-stages, in order — the cardinality
/// is exactly three because there are exactly three places a site can be lost
/// *before* it is evaluated. Each cause routes a *different* remedy (see
/// [`UnreachedCause::remedy`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnreachedCause {
    /// Lost at the **enumerate** stage: the region was never in the scan
    /// frontier (a workspace member `cargo metadata` reported but that the scan
    /// never walked; a `cfg`-gated path not built). The immune-privileged-site
    /// cognate — the patrol never includes the region. Computed live from
    /// [`crate::scan::ScanCoverage::unscanned_members`]. Remedy: **coverage**
    /// (extend the patrol).
    Barrier,
    /// Lost at the **match** stage: the region *was* walked but the recognition
    /// heuristic did not fire (a non-standard input type, a fingerprint
    /// recall-gap). This is `ScannerBoundaryFalseNegative`. The
    /// below-activation-threshold cognate. Detecting it needs a resolvable
    /// reference pointing into a walked-but-unmatched site — multi-crate Layer-2
    /// reference-resolution (not yet wired; the variant exists, the detector
    /// composes when Layer-2 lands). Remedy: **sensitivity** (widen recall).
    SubThreshold,
    /// Lost at the **parse / shape** stage: the region is present and *would* be
    /// recognized, but the site is in a form the scanner cannot see yet (a
    /// macro-unexpanded body, a hidden impl-trait concrete type). The
    /// cryptic-epitope cognate; its remedy is the dendritic-cell
    /// antigen-processing analog — macro-expand-before-scan. Detecting it needs a
    /// resolvable reference pointing into an unparsed region — multi-crate
    /// Layer-2 (not yet wired; variant present, detector composes later).
    /// Remedy: **pre-processing** (expand/normalize before scanning).
    Cryptic,
}

impl UnreachedCause {
    /// The remedy class this cause routes to — rendered into the verdict so the
    /// audit tells the adopter *what to do*, not just *that* a site was unseen.
    /// Distinct per cause: collapsing them would lose the remedy-routing the
    /// three-cause cardinality exists to preserve.
    #[must_use]
    pub const fn remedy(self) -> &'static str {
        match self {
            Self::Barrier => {
                "coverage: extend the scan to include the unreached region \
                 (scan the member, build the cfg-gated path)"
            },
            Self::SubThreshold => {
                "sensitivity: the site was scanned but not recognized — \
                 widen the fingerprint or mark it explicitly with #[presents]"
            },
            Self::Cryptic => {
                "pre-processing: the site is in a form the scanner cannot see — \
                 macro-expand or normalize before scanning"
            },
        }
    }
}

/// One site the scanner should have evaluated but did not — a per-site
/// projection of the ignorance frontier. Emitted by [`audit_coverage`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnreachedSite {
    /// The unreached region's identity. For a [`UnreachedCause::Barrier`] this is
    /// the unscanned member's ADR-017 canonical path (`<name>@<version>`). For
    /// the reference-relative causes (when wired) it is the resolvable reference
    /// whose target fell into the lost region.
    pub region: String,
    /// Why the site was not reached — routes the remedy.
    pub cause: UnreachedCause,
    /// The remedy class for `cause` (see [`UnreachedCause::remedy`]), inlined so
    /// a JSON consumer gets the actionable text without re-deriving it.
    pub remedy: String,
}

/// Aggregate coverage / reachability audit report — the ignorance frontier as a
/// list of per-site verdicts plus a clean/unreached count split.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoverageAuditReport {
    /// Every site the scan should have evaluated but did not, with its cause.
    pub unreached_sites: Vec<UnreachedSite>,
    /// Whether the coverage question was *askable* at all: `true` iff the
    /// producing [`ScanReport`] carried a [`crate::scan::ScanCoverage`] record
    /// (a member-aware `--workspace` scan), `false` for a flat scan that has no
    /// member concept. This is the third value [`Self::is_complete`] cannot
    /// carry on its own (a type-discipline gap): a 2-valued
    /// `bool` over a 3-state domain collapses "complete because every member was
    /// scanned" and "complete because coverage was never applicable" to the same
    /// `true`. Read it via [`Self::coverage_was_applicable`].
    pub applicable: bool,
}

impl CoverageAuditReport {
    /// True when no unreached site was detected — the *detectable* ignorance
    /// frontier is empty. Tier-honest: this does NOT assert there is no
    /// *absolute* ignorance (a site nothing references and no scan-root reaches
    /// is undetectable in principle — the structural honesty-limit).
    ///
    /// **Two-valued over a three-state domain — read with
    /// [`Self::coverage_was_applicable`].** `is_complete() == true` arises from
    /// two structurally distinct situations a library consumer must be able to
    /// tell apart: (1) a member-aware scan ran and *nothing* was unreached
    /// (genuinely complete), and (3) a flat scan ran where the coverage question
    /// is *not applicable* (no member set to ask it against). Both yield `true`
    /// here; [`Self::coverage_was_applicable`] is the discriminator (`true` for
    /// case 1, `false` for case 3). Case (2) — a member-aware scan with some
    /// member unreached — is the only `is_complete() == false`.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.unreached_sites.is_empty()
    }

    /// Whether the coverage / reachability question was *applicable* to the scan
    /// that produced this report — the discriminator that makes the 3-state
    /// coverage domain readable from a 2-valued [`Self::is_complete`].
    ///
    /// `true` iff the producing scan was member-aware (a `--workspace`
    /// [`crate::scan::ScanCoverage`] record was present), so "every member
    /// reached" is a claim with content. `false` for a flat scan, where there is
    /// no member set and so no frontier to be complete *over* — `is_complete()`
    /// is then vacuously `true` and means only "nothing detectable was missed,"
    /// not "coverage was verified." Pairing the two methods lets a consumer
    /// distinguish a verified-complete audit from a not-applicable one without
    /// reaching back to the [`ScanReport`].
    #[must_use]
    pub const fn coverage_was_applicable(&self) -> bool {
        self.applicable
    }

    /// Count of unreached sites for a given cause — lets a consumer report
    /// per-remedy totals (how much coverage debt vs sensitivity debt vs
    /// pre-processing debt).
    #[must_use]
    pub fn count_by_cause(&self, cause: UnreachedCause) -> usize {
        self.unreached_sites
            .iter()
            .filter(|s| s.cause == cause)
            .count()
    }
}

/// Audit scanner coverage / reachability across a scan report — the ignorance
/// frontier rendered as per-site [`UnreachedSite`] verdicts.
///
/// **Barrier cause is live now**, computed from the merged report's
/// [`crate::scan::ScanCoverage`] (populated by a `--workspace` member-aware
/// scan): every enumerated-but-unscanned member is an unreached region whose
/// `#[presents]` sites went unseen. A flat scan has no `scan_coverage`
/// (`None`) — it has no member concept, so it cannot know what it missed; this
/// returns an empty report there (tier-honest: absence of a coverage record is
/// not a claim of completeness, it is the absence of the member-set needed to
/// even ask the question).
///
/// [`UnreachedCause::SubThreshold`] and [`UnreachedCause::Cryptic`] are present
/// in [`UnreachedCause`] (the cardinality is structurally guaranteed at three)
/// but their detectors are *gated on multi-crate Layer-2 reference-resolution* —
/// they fire only when a resolvable reference points into a walked-but-unmatched
/// (sub-threshold) or unparsed (cryptic) region. Until Layer-2 lands,
/// `audit_coverage` emits only Barrier verdicts; the other two arms compose in
/// without changing this surface (additive — a consumer already branches on
/// `cause`).
#[must_use]
pub fn audit_coverage(report: &ScanReport) -> CoverageAuditReport {
    let mut unreached_sites = Vec::new();

    // `applicable` records whether the coverage question was askable at all — a
    // member-aware scan carries a `ScanCoverage`, a flat scan does not. This is
    // the third value `is_complete()` cannot carry: it lets a consumer tell a
    // genuinely-complete member-aware audit from a not-applicable flat one
    // (both have an empty `unreached_sites`).
    let applicable = report.scan_coverage.is_some();

    if let Some(coverage) = report.scan_coverage.as_ref() {
        // Barrier cause: enumerated-but-unscanned members. The frontier is
        // already a set (unscanned_members dedups), so each member yields one
        // verdict. SubThreshold + Cryptic are not derivable from ScanCoverage
        // alone — they need reference-resolution (Layer-2), so they are not
        // emitted here yet.
        for member in coverage.unscanned_members() {
            unreached_sites.push(UnreachedSite {
                region: member.to_owned(),
                cause: UnreachedCause::Barrier,
                remedy: UnreachedCause::Barrier.remedy().to_owned(),
            });
        }
    }

    CoverageAuditReport {
        unreached_sites,
        applicable,
    }
}
