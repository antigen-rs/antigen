//! Render D — session-prime batch digest (ADR-043 §E render D). Refutation gates
//! over the SHIPPED `session_prime` / `session_prime_top_n`.
//!
//! Render D is a RANK + TRUNCATE render — the two places a digest silently lies:
//!
//! - truncation that drops clusters/sites without telling the agent the digest
//!   is partial (a false "this is the whole picture");
//! - a rank that inverts severity vs blast-radius (burying a `High` cluster under
//!   a noisy `Medium` one).
//!
//! And, like every render, it must preserve the spine's claim-scope (a
//! fingerprint match never laundered into an audited verdict / invented severity).

use antigen::finding::{
    DialTier, FINDING_SCHEMA_VERSION, Finding, FindingBody, OriginStage, Presentation, Provenance,
    Severity, cluster_key_of,
};
use antigen::render::session_prime::{
    DEFAULT_TOP_N, MAX_SITES_PER_CLUSTER, session_prime, session_prime_top_n,
};

/// Build a synthetic `FingerprintMatch` finding for `class` at `file:line` with a
/// given severity + provenance. The `cluster_key` is `class@<digest>` so all
/// findings sharing `(class, digest)` cluster together.
fn fp_finding(
    class: &str,
    digest: &str,
    file: &str,
    line: usize,
    severity: Severity,
    provenance: Provenance,
) -> Finding {
    Finding {
        schema_version: FINDING_SCHEMA_VERSION,
        file: file.to_string(),
        line,
        structural_digest: digest.to_string(),
        shape_digest: digest.to_string(),
        cluster_key: cluster_key_of(digest, class),
        severity,
        source: "scan:test".to_string(),
        class_provenance: provenance,
        presentation: Presentation::Passive,
        timestamp: line as u64,
        origin_stage: OriginStage::Scan,
        body: FindingBody::FingerprintMatch {
            class: class.to_string(),
            tier: DialTier::Suspected,
        },
    }
}

// ===========================================================================
// (1) RANK: severity is PRIMARY, blast-radius SECONDARY. A High-severity cluster
//     with ONE site must outrank a Medium-severity cluster with MANY sites.
// ===========================================================================

#[test]
fn high_severity_single_site_outranks_medium_severity_many_sites() {
    let mut findings = vec![fp_finding(
        "rare-but-severe",
        "d1",
        "a.rs",
        1,
        Severity::High,
        Provenance::Constructable,
    )];
    // A noisy Medium cluster with 8 sites.
    for i in 0..8 {
        findings.push(fp_finding(
            "common-but-mild",
            "d2",
            "b.rs",
            10 + i,
            Severity::Medium,
            Provenance::Constructable,
        ));
    }

    let digest = session_prime(&findings);
    assert_eq!(digest.top_clusters.len(), 2, "two distinct clusters");
    assert_eq!(
        digest.top_clusters[0].class,
        "rare-but-severe",
        "the HIGH-severity cluster must rank first even with ONE site vs the \
         Medium cluster's EIGHT — severity is the primary key, not blast-radius. \
         got order: {:?}",
        digest
            .top_clusters
            .iter()
            .map(|c| &c.class)
            .collect::<Vec<_>>()
    );
    assert_eq!(digest.top_clusters[0].blast_radius, 1);
    assert_eq!(digest.top_clusters[1].blast_radius, 8);
}

// ===========================================================================
// (2) TRUNCATION HONESTY — blast_radius reports the FULL site count even when the
//     sample sites are capped at MAX_SITES_PER_CLUSTER. The agent must not read a
//     capped sample as the full blast radius.
// ===========================================================================

#[test]
fn blast_radius_is_the_full_count_even_when_sample_sites_are_capped() {
    let n_sites = MAX_SITES_PER_CLUSTER + 7;
    let findings: Vec<Finding> = (0..n_sites)
        .map(|i| {
            fp_finding(
                "widespread",
                "d",
                "f.rs",
                i + 1,
                Severity::High,
                Provenance::Constructable,
            )
        })
        .collect();

    let digest = session_prime(&findings);
    let c = &digest.top_clusters[0];
    assert_eq!(
        c.blast_radius, n_sites,
        "blast_radius must be the FULL site count ({n_sites}), not the capped \
         sample — a capped sample read as the blast radius understates the spread"
    );
    assert_eq!(
        c.sites.len(),
        MAX_SITES_PER_CLUSTER,
        "sample sites are capped at MAX_SITES_PER_CLUSTER for compactness"
    );
    assert_eq!(
        digest.total_findings, n_sites,
        "total_findings counts every match pre-truncation"
    );
}

// ===========================================================================
// (3) TOP-N TRUNCATION HONESTY — total_clusters reports the count BEFORE the
//     top-N cut, so an agent seeing N shown clusters knows whether more exist.
// ===========================================================================

#[test]
fn top_n_truncation_reports_the_full_cluster_total() {
    // Make DEFAULT_TOP_N + 3 distinct clusters.
    let extra = 3;
    let total = DEFAULT_TOP_N + extra;
    let findings: Vec<Finding> = (0..total)
        .map(|i| {
            fp_finding(
                &format!("class{i}"),
                &format!("digest{i}"),
                "f.rs",
                i + 1,
                Severity::High,
                Provenance::Constructable,
            )
        })
        .collect();

    let digest = session_prime(&findings);
    assert_eq!(
        digest.top_clusters.len(),
        DEFAULT_TOP_N,
        "the digest shows only the top-N clusters"
    );
    assert_eq!(
        digest.total_clusters, total,
        "but total_clusters reports the FULL count ({total}) before the top-N cut — \
         else the agent reads a truncated digest as the whole picture (a false \
         all-surfaced). got total_clusters={}",
        digest.total_clusters
    );
}

// ===========================================================================
// (4) CLAIM-SCOPE PRESERVED — the cluster's severity/provenance are CARRIED from
//     the findings, never invented. A cluster of two findings (High + Medium)
//     takes the MAX severity (High); the provenance is the strongest tier present.
//     The render ranks + summarizes; it does not ratify or upgrade.
// ===========================================================================

#[test]
fn cluster_severity_and_provenance_are_carried_not_invented() {
    let findings = vec![
        fp_finding("c", "d", "a.rs", 1, Severity::Medium, Provenance::Heuristic),
        fp_finding(
            "c",
            "d",
            "a.rs",
            2,
            Severity::High,
            Provenance::Constructable,
        ),
    ];
    let digest = session_prime(&findings);
    assert_eq!(
        digest.top_clusters.len(),
        1,
        "one cluster (same class@digest)"
    );
    let c = &digest.top_clusters[0];
    assert_eq!(
        c.severity,
        Severity::High,
        "cluster severity is the MAX across sites (a single High site makes the \
         cluster High) — carried, not invented"
    );
    assert_eq!(
        c.class_provenance,
        Provenance::Constructable,
        "cluster provenance is the STRONGEST (most-verified) tier present \
         (Constructable > Heuristic) — never upgraded past what a site actually \
         carries"
    );
}

// ===========================================================================
// (5) DEGENERATE — an empty population yields an empty, honest digest (no panic,
//     no phantom cluster). And session_prime_top_n(_, 0) shows zero clusters but
//     still reports the real totals.
// ===========================================================================

#[test]
fn empty_and_zero_n_are_honest_not_panicking() {
    let empty = session_prime(&[]);
    assert_eq!(empty.total_findings, 0);
    assert_eq!(empty.total_clusters, 0);
    assert!(empty.top_clusters.is_empty());

    let findings = vec![fp_finding(
        "c",
        "d",
        "a.rs",
        1,
        Severity::High,
        Provenance::Constructable,
    )];
    let zero = session_prime_top_n(&findings, 0);
    assert!(zero.top_clusters.is_empty(), "top-0 shows no clusters");
    assert_eq!(
        zero.total_clusters, 1,
        "but the real total is still reported — top-0 is a view, not a denial that \
         findings exist"
    );
}
