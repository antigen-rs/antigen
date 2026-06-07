//! E1 — the catalog-match SPINE (ADR-043 §E: one spine, four renders). The
//! integration/consistency gates over the SHIPPED `catalog_match_findings` (the
//! one call B/C/D ride).
//!
//! The unit tests in `scan::catalog_match` already cover the claim-scope core
//! (projects-only-classes-in-map, never-audited-verdict, explicit-markers-skipped,
//! cluster-key shape). THESE gates pin the properties a unit test over hand-built
//! presentations cannot: the spine over a REAL bundled-catalog scan, and the
//! cross-surface consistency between the spine and the E0
//! `bundled_catalog_findings` projection (both ride the same matches — a divergence
//! is antigen's own `ParallelStateTrackersDiverge` class in its own pipeline).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use antigen::finding::{FindingBody, Provenance};
use antigen::scan::{
    bundled_catalog_findings, catalog_match_findings, scan_workspace_bundled_catalog,
};
use antigen::stdlib::catalog::stdlib_catalog_entries;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

const CONSUMER: &str = "e0_consumer_crate_zero_decls";

/// The bundled catalog's `class → provenance` map, the way a render builds it to
/// drive the spine.
fn catalog_provenance_map() -> HashMap<String, Provenance> {
    stdlib_catalog_entries()
        .into_iter()
        .map(|e| (e.name, e.provenance))
        .collect()
}

// ===========================================================================
// (1) THE SPINE over a real bundled-catalog scan: every projected finding is a
//     FingerprintMatch (never a DialVerdict), carrying a verified-core provenance.
// ===========================================================================

#[test]
fn spine_projects_real_scan_matches_as_claim_scoped_findings() {
    let scan = scan_workspace_bundled_catalog(&fixture(CONSUMER), None, true)
        .expect("bundled-catalog scan completes");
    let prov = catalog_provenance_map();

    let findings = catalog_match_findings(&scan, &prov);
    assert!(
        !findings.is_empty(),
        "the spine must project ≥1 finding for the consumer fixture's footguns"
    );
    for f in &findings {
        assert!(
            matches!(f.body, FindingBody::FingerprintMatch { .. }),
            "every spine finding is a FingerprintMatch (claim-scope) — never a \
             DialVerdict. got {:?}",
            f.body
        );
        assert!(
            matches!(
                f.class_provenance,
                Provenance::Constructable | Provenance::Encountered
            ),
            "spine finding provenance must be verified-core; got {:?}",
            f.class_provenance
        );
    }
}

// ===========================================================================
// (2) CROSS-SURFACE CONSISTENCY: the E0 `bundled_catalog_findings` convenience
//     and the E1 spine `catalog_match_findings` ride the SAME matches with the
//     SAME catalog provenance — so they must agree on the set of (class, file,
//     line) findings. A divergence is a ParallelStateTrackersDiverge in antigen's
//     own pipeline (two projections of one match set drifting).
// ===========================================================================

#[test]
fn spine_and_bundled_convenience_agree_on_the_finding_set() {
    let scan = scan_workspace_bundled_catalog(&fixture(CONSUMER), None, true)
        .expect("bundled-catalog scan completes");

    let via_convenience = bundled_catalog_findings(&scan);
    let via_spine = catalog_match_findings(&scan, &catalog_provenance_map());

    let key = |f: &antigen::finding::Finding| -> (String, String, usize, String) {
        let class = match &f.body {
            FindingBody::FingerprintMatch { class, .. } => class.clone(),
            other => panic!("non-match body in a bundled projection: {other:?}"),
        };
        (
            class,
            f.file.clone(),
            f.line,
            format!("{:?}", f.class_provenance),
        )
    };

    let mut a: Vec<_> = via_convenience.iter().map(key).collect();
    let mut b: Vec<_> = via_spine.iter().map(key).collect();
    a.sort();
    b.sort();

    assert_eq!(
        a, b,
        "the E0 convenience projection and the E1 spine must agree on the \
         (class, file, line, provenance) finding set — they project the SAME \
         matches with the SAME catalog provenance. A divergence is a \
         parallel-state drift between two views of one match set.\n  convenience: {a:?}\n  spine:       {b:?}"
    );
}

// ===========================================================================
// (3) THE CLAIM-SCOPE SKIP IS SAFE-NOT-SILENT-DEFAULT: a class absent from the
//     provenance map is NOT projected (correct — out of claim), and is NEVER
//     stamped with a default Imagined provenance (which would be an over-claim:
//     surfacing a finding antigen can't honestly label). Verified by passing an
//     EMPTY map: zero findings, not a pile of Imagined-stamped ones.
// ===========================================================================

#[test]
fn a_class_absent_from_the_map_is_dropped_never_default_stamped() {
    let scan = scan_workspace_bundled_catalog(&fixture(CONSUMER), None, true)
        .expect("bundled-catalog scan completes");

    // Sanity: with the real map there ARE findings.
    assert!(
        !catalog_match_findings(&scan, &catalog_provenance_map()).is_empty(),
        "precondition: the real map yields findings"
    );

    // With an EMPTY map, every match's class is absent → ZERO findings. The spine
    // must NOT invent a default Imagined provenance to surface them anyway.
    let empty: HashMap<String, Provenance> = HashMap::new();
    let findings = catalog_match_findings(&scan, &empty);
    assert!(
        findings.is_empty(),
        "a fingerprint match whose class is absent from the provenance map must be \
         DROPPED (out of claim), NOT surfaced with a default/invented provenance — \
         that would be an over-claim (a finding antigen cannot honestly label). \
         got {} findings: {findings:?}",
        findings.len()
    );
}

// ===========================================================================
// (4) CROSS-RENDER CONSISTENCY: Render B (flycheck) and Render D (session-prime)
//     both consume the SAME E1 `Finding` population. An editor (B) and a fresh
//     agent (D) must see the SAME set of (class, file, line) sites — a divergence
//     means the two surfaces disagree about what the scan found (the editor flags
//     a site the agent's digest omits, or vice versa). Both filter to
//     FingerprintMatch; they must agree on the set.
// ===========================================================================

#[test]
fn render_b_and_render_d_surface_the_same_site_set() {
    use antigen::render::flycheck::findings_to_cargo_jsonl;
    use antigen::render::session_prime::session_prime_top_n;

    let scan = scan_workspace_bundled_catalog(&fixture(CONSUMER), None, true)
        .expect("bundled-catalog scan completes");
    let findings = catalog_match_findings(&scan, &catalog_provenance_map());
    assert!(
        !findings.is_empty(),
        "precondition: the spine yields findings"
    );

    // Render B: parse the flycheck JSONL, pull (class-from-code, file, line).
    let jsonl = findings_to_cargo_jsonl(&findings).expect("flycheck serializes");
    let mut b_sites: Vec<(String, String, u64)> = Vec::new();
    for json_line in jsonl.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(json_line).expect("valid json line");
        // The code is "antigen::<class>"; strip the prefix to compare with D's class.
        let code = v["message"]["code"]["code"]
            .as_str()
            .expect("code present")
            .strip_prefix("antigen::")
            .expect("antigen-prefixed code")
            .to_string();
        let primary_span = &v["message"]["spans"][0];
        let file = primary_span["file_name"]
            .as_str()
            .expect("file")
            .to_string();
        let line_no = primary_span["line_start"].as_u64().expect("line");
        b_sites.push((code, file, line_no));
    }
    b_sites.sort();

    // Render D: take ALL clusters (n = total) and flatten to (class, file, line).
    let digest = session_prime_top_n(&findings, usize::MAX);
    let mut d_sites: Vec<(String, String, u64)> = Vec::new();
    for cluster in &digest.top_clusters {
        for site in &cluster.sites {
            d_sites.push((cluster.class.clone(), site.file.clone(), site.line as u64));
        }
    }
    d_sites.sort();

    assert_eq!(
        b_sites, d_sites,
        "Render B (flycheck) and Render D (session-prime) must surface the SAME \
         (class, file, line) site set from the SAME E1 findings — an editor and a \
         fresh agent looking at one scan must not disagree about what was found.\n  \
         B: {b_sites:?}\n  D: {d_sites:?}"
    );
}
