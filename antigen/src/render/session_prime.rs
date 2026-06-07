//! Render D — **session-prime** batch digest (v0.4 ADR-043 §E).
//!
//! Serves the agent that *doesn't know to ask*: a fresh or compacted coding
//! agent needs the failure-class picture of a codebase up front, ranked, without
//! issuing a query per fragment. This render takes the unified
//! [`Finding`] population (the catalog-match spine's
//! output) and produces a compact, **structured** digest — co-native (a
//! serializable struct, not a pre-rendered string) so an agent consumes it
//! natively and a human render can be layered on top.
//!
//! The shape: group findings by [`cluster_key`](crate::finding::cluster_key_of)
//! (shared structure), rank clusters by **severity then blast-radius** (how many
//! sites share the cluster — a wide cluster is a systemic smell), and take the
//! top-N. Each cluster carries its class, provenance, severity, the site count,
//! and a bounded sample of sites for context.
//!
//! # Claim-scope (ADR-044)
//!
//! This render is a re-presentation; it asserts nothing the spine did not. Every
//! cluster's `class_provenance` is carried straight through from the findings;
//! the digest never upgrades a tier, never names a new class, and never claims a
//! defense was audited. It ranks and summarizes — it does not ratify.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::finding::{DialTier, Finding, FindingBody, Provenance, Severity};

/// One site inside a cluster (file + line + the dial tier of the match).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeSite {
    /// Source file of the matched site.
    pub file: String,
    /// Source line of the matched site.
    pub line: usize,
    /// The dial tier of *this site's* match (`Suspected` = shape-only scan floor;
    /// `Named` = graduated). Carried per-site so a `Suspected` match is never
    /// laundered into a bare high-severity alarm by the cluster's rank — the
    /// per-site honesty the claim-scope (ADR-044) requires.
    pub tier: DialTier,
}

/// One ranked cluster in the session-prime digest — a failure-class with the
/// sites that share its structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeCluster {
    /// The cluster key (`class@structural_digest`) — shared-structure identity.
    pub cluster_key: String,
    /// The failure-class (antigen name) the cluster is about.
    pub class: String,
    /// The class-provenance carried straight through from the findings (the
    /// honest tier; never upgraded by this render).
    pub class_provenance: Provenance,
    /// The cluster's severity (the max severity across its sites).
    pub severity: Severity,
    /// **Blast radius** — how many sites share this cluster. A wide cluster is a
    /// systemic smell; it is the secondary rank key after severity.
    pub blast_radius: usize,
    /// A bounded sample of the cluster's sites (capped at
    /// [`MAX_SITES_PER_CLUSTER`] for digest compactness; `blast_radius` is the
    /// true total).
    pub sites: Vec<PrimeSite>,
}

/// The session-prime digest — the top-N ranked clusters plus the totals a fresh
/// agent needs to calibrate (how much was surfaced, how much the digest shows).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPrime {
    /// Total number of fingerprint-match findings the scan surfaced (before
    /// clustering / top-N truncation) — the agent's "how big is the picture".
    pub total_findings: usize,
    /// Total number of distinct clusters (before top-N truncation).
    pub total_clusters: usize,
    /// The top-N ranked clusters (severity desc, then blast-radius desc, then
    /// `cluster_key` for a stable tie-break).
    pub top_clusters: Vec<PrimeCluster>,
}

/// The default cap on clusters shown in the digest (the "top-N").
pub const DEFAULT_TOP_N: usize = 10;

/// The cap on sample sites shown per cluster (the full count is `blast_radius`).
pub const MAX_SITES_PER_CLUSTER: usize = 5;

/// Build a [`SessionPrime`] digest from a [`Finding`] population, showing the top
/// [`DEFAULT_TOP_N`] clusters.
#[must_use]
pub fn session_prime(findings: &[Finding]) -> SessionPrime {
    session_prime_top_n(findings, DEFAULT_TOP_N)
}

/// Build a [`SessionPrime`] digest showing the top `n` clusters.
///
/// Only `FingerprintMatch` findings participate (the catalog-match render
/// surface); marked-unknown / dial-verdict findings, if present in the
/// population, are out of this render's scope and ignored.
#[must_use]
pub fn session_prime_top_n(findings: &[Finding], n: usize) -> SessionPrime {
    // Group the fingerprint-match findings by cluster_key. BTreeMap for a stable
    // iteration order (the cluster_key tie-break below relies on determinism).
    let mut by_cluster: BTreeMap<String, Vec<&Finding>> = BTreeMap::new();
    for f in findings {
        if matches!(f.body, FindingBody::FingerprintMatch { .. }) {
            by_cluster.entry(f.cluster_key.clone()).or_default().push(f);
        }
    }

    let total_findings: usize = by_cluster.values().map(Vec::len).sum();
    let total_clusters = by_cluster.len();

    let mut clusters: Vec<PrimeCluster> = by_cluster
        .into_iter()
        .map(|(cluster_key, group)| build_cluster(cluster_key, &group))
        .collect();

    // Rank: severity desc, then blast-radius desc, then cluster_key asc (stable
    // tie-break). `severity_rank` makes High > Medium > Low.
    clusters.sort_by(|a, b| {
        severity_rank(b.severity)
            .cmp(&severity_rank(a.severity))
            .then(b.blast_radius.cmp(&a.blast_radius))
            .then(a.cluster_key.cmp(&b.cluster_key))
    });
    clusters.truncate(n);

    SessionPrime {
        total_findings,
        total_clusters,
        top_clusters: clusters,
    }
}

/// Build one [`PrimeCluster`] from a cluster's findings. The cluster severity is
/// the **max** across its sites (a single High site makes the cluster High); the
/// provenance is the strongest (most-verified) tier present. Both are carried
/// from the findings — never invented (claim-scope).
fn build_cluster(cluster_key: String, group: &[&Finding]) -> PrimeCluster {
    let blast_radius = group.len();
    let severity = group
        .iter()
        .map(|f| f.severity)
        .max_by_key(|s| severity_rank(*s))
        .unwrap_or(Severity::Low);
    let class_provenance = group
        .iter()
        .map(|f| f.class_provenance)
        .min_by_key(|p| provenance_rank(*p))
        .unwrap_or(Provenance::DEFAULT);
    // The class name is the same across a cluster (cluster_key = class@digest);
    // read it from the first finding's body, falling back to `source` parsing
    // never needed because the body always carries it for a FingerprintMatch.
    let class = group
        .first()
        .and_then(|f| match &f.body {
            FindingBody::FingerprintMatch { class, .. } => Some(class.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let mut sites: Vec<PrimeSite> = group
        .iter()
        .map(|f| PrimeSite {
            file: f.file.clone(),
            line: f.line,
            // The per-site dial tier from the body's tier (FingerprintMatch or
            // DialVerdict); a marked-unknown (not expected in this grouping) falls
            // back to the floor.
            tier: match &f.body {
                FindingBody::FingerprintMatch { tier, .. }
                | FindingBody::DialVerdict { tier, .. } => *tier,
                FindingBody::MarkedUnknown { .. } => DialTier::Suspected,
            },
        })
        .collect();
    // Deterministic, then bounded.
    sites.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    sites.truncate(MAX_SITES_PER_CLUSTER);

    PrimeCluster {
        cluster_key,
        class,
        class_provenance,
        severity,
        blast_radius,
        sites,
    }
}

/// Severity ordering for ranking: `High`(2) > `Medium`(1) > `Low`(0).
const fn severity_rank(s: Severity) -> u8 {
    match s {
        Severity::Low => 0,
        Severity::Medium => 1,
        Severity::High => 2,
    }
}

/// Provenance ordering for "strongest tier": the verified core ranks below the
/// unverified tiers so `min_by_key` selects the most-verified label present.
const fn provenance_rank(p: Provenance) -> u8 {
    match p {
        Provenance::Encountered => 0,
        Provenance::Constructable => 1,
        Provenance::Heuristic => 2,
        Provenance::Imagined => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{OriginStage, Presentation, cluster_key_of};

    fn match_finding(class: &str, file: &str, line: usize, sev: Severity) -> Finding {
        let digest = format!("d-{class}");
        Finding {
            schema_version: crate::finding::FINDING_SCHEMA_VERSION,
            file: file.to_string(),
            line,
            structural_digest: digest.clone(),
            shape_digest: String::new(),
            cluster_key: cluster_key_of(&digest, class),
            severity: sev,
            source: "scan:catalog-match".to_string(),
            class_provenance: Provenance::Constructable,
            presentation: Presentation::Passive,
            timestamp: line as u64,
            origin_stage: OriginStage::Scan,
            body: FindingBody::FingerprintMatch {
                class: class.to_string(),
                tier: DialTier::Suspected,
            },
        }
    }

    #[test]
    fn groups_by_cluster_and_counts_blast_radius() {
        let findings = vec![
            match_finding("panic-in-drop", "a.rs", 1, Severity::High),
            match_finding("panic-in-drop", "b.rs", 2, Severity::High),
            match_finding("unbounded-deser", "c.rs", 3, Severity::High),
        ];
        let prime = session_prime(&findings);
        assert_eq!(prime.total_findings, 3);
        assert_eq!(prime.total_clusters, 2);
        let drop_cluster = prime
            .top_clusters
            .iter()
            .find(|c| c.class == "panic-in-drop")
            .expect("drop cluster present");
        assert_eq!(drop_cluster.blast_radius, 2, "two panic-in-drop sites");
    }

    #[test]
    fn ranks_by_severity_then_blast_radius() {
        // A wide Medium cluster must rank below a narrow High cluster (severity
        // dominates), and a wider High ranks above a narrower High.
        let findings = vec![
            match_finding("low-but-wide", "a.rs", 1, Severity::Medium),
            match_finding("low-but-wide", "a.rs", 2, Severity::Medium),
            match_finding("low-but-wide", "a.rs", 3, Severity::Medium),
            match_finding("high-narrow", "b.rs", 1, Severity::High),
            match_finding("high-wide", "c.rs", 1, Severity::High),
            match_finding("high-wide", "c.rs", 2, Severity::High),
        ];
        let prime = session_prime(&findings);
        let order: Vec<&str> = prime
            .top_clusters
            .iter()
            .map(|c| c.class.as_str())
            .collect();
        assert_eq!(order, vec!["high-wide", "high-narrow", "low-but-wide"]);
    }

    #[test]
    fn top_n_truncates_but_totals_report_the_full_picture() {
        let findings: Vec<Finding> = (0..20)
            .map(|i| match_finding(&format!("class-{i}"), "a.rs", i, Severity::High))
            .collect();
        let prime = session_prime_top_n(&findings, 3);
        assert_eq!(prime.top_clusters.len(), 3, "top-3 only");
        assert_eq!(prime.total_clusters, 20, "but totals report all 20");
        assert_eq!(prime.total_findings, 20);
    }

    #[test]
    fn carries_provenance_through_never_upgrades() {
        // claim-scope: the digest reports the finding's provenance verbatim.
        let findings = vec![match_finding("c", "a.rs", 1, Severity::High)];
        let prime = session_prime(&findings);
        assert_eq!(
            prime.top_clusters[0].class_provenance,
            Provenance::Constructable
        );
    }

    #[test]
    fn each_site_carries_its_dial_tier_per_site_honesty() {
        // Per-site claim-scope (ADR-044): a Suspected match must carry its
        // Suspected tier through to the site, so a shape-only match is never
        // laundered into a bare high-severity alarm by the cluster rank.
        let findings = vec![match_finding("c", "a.rs", 1, Severity::High)];
        let prime = session_prime(&findings);
        let site = &prime.top_clusters[0].sites[0];
        assert_eq!(
            site.tier,
            DialTier::Suspected,
            "the scan-floor match's Suspected tier must reach the site"
        );
    }

    #[test]
    fn sites_sample_is_bounded_but_blast_radius_is_exact() {
        let findings: Vec<Finding> = (0..12)
            .map(|i| match_finding("wide", "a.rs", i, Severity::High))
            .collect();
        let prime = session_prime(&findings);
        let cluster = &prime.top_clusters[0];
        assert_eq!(cluster.blast_radius, 12, "exact total");
        assert_eq!(
            cluster.sites.len(),
            MAX_SITES_PER_CLUSTER,
            "sample is bounded"
        );
    }

    #[test]
    fn empty_population_is_an_empty_digest() {
        let prime = session_prime(&[]);
        assert_eq!(prime.total_findings, 0);
        assert_eq!(prime.total_clusters, 0);
        assert!(prime.top_clusters.is_empty());
    }

    #[test]
    fn serializes_co_natively() {
        let findings = vec![match_finding("c", "a.rs", 1, Severity::High)];
        let prime = session_prime(&findings);
        let json = serde_json::to_string(&prime).expect("SessionPrime serializes");
        assert!(json.contains("\"total_findings\":1"));
        let back: SessionPrime = serde_json::from_str(&json).expect("round-trips");
        assert_eq!(back, prime);
    }
}
