//! E0 — the bundled-catalog → `Finding` PROJECTION gate (the un-built half).
//!
//! ADR-043 Amd-1 (match-render only, claim-scoped); ADR-044 (claim-scope
//! honesty).
//!
//! The scan-side gates (`e0_bundled_catalog_scan.rs`, all GREEN) prove the
//! catalog injects into the synthesis pass and the matches resolve to
//! verified-core provenance. THIS gate is the claim-scope LINE: the E0 spec says
//! "every emitted FINDING carries `class_provenance`" — so the bundled-catalog
//! matches must be projected into a `Finding` population for the render, and:
//!   - each MUST be a `FindingBody::FingerprintMatch` (a syntactic FACT — match
//!     render only, ADR-043 Amd-1),
//!   - NONE may be a `FindingBody::DialVerdict` (an AUDITED verdict — that crosses
//!     the ADR-044 syntactic/semantic line; a bundled match reading as an audited
//!     all-clear/defended verdict is antigen's OWN over-claim class, the deepest
//!     E0 break the checks guard against),
//!   - each carries the catalog member's verified-core provenance.
//!
//! ## RED-by-design, but NON-BLOCKING (`#[ignore]`)
//!
//! As of the HEAD this was authored against there is NO public `Presentation →
//! Finding` projection for catalog matches: `FindingBody::FingerprintMatch`
//! exists but nothing constructs it, and the pipeline `findings` population is
//! marked-unknowns-only (pipeline.rs:133-157). Calling the (not-yet-existent)
//! projection directly would make this whole test binary fail to COMPILE, which
//! hard-stops `cargo test -p antigen` and bricks the iteration loop.
//! So this gate is `#[ignore]`d with the contract spelled out inline. When
//! the projection lands, delete the `panic!` stub + the
//! `#[ignore]`, call the real projection (name is free to pick — the BEHAVIOR
//! asserted below is the gate), and it flips GREEN.
//!
//! Discover it with: `cargo test -p antigen --test e0_bundled_catalog_entrypoint
//! -- --ignored`.

use std::path::{Path, PathBuf};

use antigen::finding::{Finding, FindingBody, Provenance};
use antigen::scan::{MatchKind, bundled_catalog_findings, scan_workspace_bundled_catalog};
use antigen::stdlib::catalog::stdlib_catalog_entries;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// The projection under test. THE CONTRACT: a public function that turns a
/// bundled-catalog `ScanReport`'s fingerprint-match presentations into the
/// unified `Finding` population (each a `FindingBody::FingerprintMatch` carrying
/// the catalog member's authored provenance). Replace this stub's body with the
/// real call once it ships (e.g. `antigen::scan::bundled_catalog_findings(scan)`
/// or whatever it ends up named) and remove the `#[ignore]` on the test.
fn project_bundled_findings(scan: &antigen::scan::ScanReport) -> Vec<Finding> {
    // LANDED: the projection shipped as
    // `antigen::scan::bundled_catalog_findings` (re-exported from
    // `stdlib::catalog`, which owns the catalog provenance). It turns the
    // bundled-catalog scan's `FingerprintMatch` presentations into
    // `FindingBody::FingerprintMatch` findings stamped with the catalog member's
    // authored provenance — never a `DialVerdict`.
    bundled_catalog_findings(scan)
}

#[test]
fn bundled_catalog_matches_project_to_findings_and_never_an_audited_verdict() {
    let scan = scan_workspace_bundled_catalog(&fixture("e0_consumer_crate_zero_decls"), None, true)
        .expect("bundled-catalog scan completes");

    let findings = project_bundled_findings(&scan);

    assert!(
        !findings.is_empty(),
        "the bundled-catalog scan must project its matches into a Finding \
         population (the render's input). Empty = the projection surfaced no \
         findings for a crate that DOES present known failure-classes."
    );

    // NO SILENT DROP: every FingerprintMatch presentation whose class IS a bundled
    // catalog member must yield exactly one finding. A PARTIAL name-drift between
    // synthesis_pass's `antigen_type` and the catalog name would pass the
    // non-empty check above while silently dropping real findings (the
    // zero-hits-cliff re-introduced for the drifted members). This pins the count.
    let catalog_names: std::collections::HashSet<String> = stdlib_catalog_entries()
        .into_iter()
        .map(|e| e.name)
        .collect();
    let catalog_match_presentations = scan
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .filter(|p| catalog_names.contains(&p.antigen_type))
        .count();
    assert_eq!(
        findings.len(),
        catalog_match_presentations,
        "the projection must emit ONE finding per bundled-catalog FingerprintMatch \
         (no silent drops): {} catalog-member matches, {} findings. A mismatch means \
         a name-drift between synthesis_pass's antigen_type and the catalog name \
         silently launders some matches into a false all-clear.",
        catalog_match_presentations,
        findings.len()
    );

    for f in &findings {
        // (a) the claim-scope line: match-render body, NEVER the audited-verdict body.
        match &f.body {
            FindingBody::FingerprintMatch { .. } => { /* honest claim-scope */ },
            FindingBody::DialVerdict { .. } => panic!(
                "a bundled-catalog match was rendered as a DialVerdict (an AUDITED \
                 verdict) at {}:{} — this crosses the ADR-044 syntactic/semantic \
                 line. A bundled match is a syntactic FACT, never an audited defense \
                 verdict. finding = {f:?}",
                f.file, f.line
            ),
            other @ FindingBody::MarkedUnknown { .. } => panic!(
                "a bundled-catalog match produced an unexpected FindingBody {other:?} \
                 at {}:{}; expected FingerprintMatch (match-render only). A \
                 marked-unknown body for a fingerprint MATCH is a mis-projection.",
                f.file, f.line
            ),
        }
        // (b) verified-core provenance re-asserted on the projected Finding.
        assert!(
            matches!(
                f.class_provenance,
                Provenance::Constructable | Provenance::Encountered
            ),
            "projected bundled finding carries provenance {:?}; must be verified-core \
             for E0 claim-scope. finding = {f:?}",
            f.class_provenance
        );
    }
}

/// Claim-scope tier honesty: an UNAUDITED bundled-catalog match must ride the
/// `Suspected` (shape-only scan floor) dial tier — NOT `Named`. The catalog
/// member may itself be a "named"-tier class in the stdlib, but the MATCH at a
/// consumer site has not been graduated by a site-specific audit, so the
/// finding's tier is the floor. A `Named` tier here would read as "this site is
/// a confirmed named defect" — an over-claim across the audit line. (The class's
/// real severity rides `Finding::severity`; the per-site confidence rides
/// `tier`. Keeping them distinct is what lets severity=High coexist honestly with
/// an unaudited match.)
#[test]
fn unaudited_bundled_match_rides_the_suspected_floor_not_named() {
    use antigen::finding::DialTier;
    let scan = scan_workspace_bundled_catalog(&fixture("e0_consumer_crate_zero_decls"), None, true)
        .expect("bundled-catalog scan completes");
    let findings = bundled_catalog_findings(&scan);
    assert!(
        !findings.is_empty(),
        "precondition: there must be findings to check"
    );
    for f in &findings {
        let FindingBody::FingerprintMatch { class, tier } = &f.body else {
            panic!(
                "a bundled finding must be a FingerprintMatch body; got {:?}",
                f.body
            );
        };
        assert_eq!(
            *tier,
            DialTier::Suspected,
            "the unaudited bundled match for `{class}` must ride the Suspected \
             shape-only floor — a `Named` tier would over-claim the site as a \
             confirmed named defect across the audit line. finding = {f:?}"
        );
    }
}
