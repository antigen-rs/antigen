//! The **catalog-match service** (v0.4 E1) — the one callable spine the renders
//! ride (ADR-043 §E: one spine, four renders).
//!
//! E0 closed the zero-hits-cliff by injecting a bundled catalog into the scan's
//! synthesis pass. E1 is the *callable projection*: it turns a scan's
//! fingerprint-match presentations into the unified [`Finding`] wire-format,
//! carrying each match's authored class-provenance — the structured population
//! the editor-flycheck (render B), the agent-query MCP (render C), and the
//! session-prime batch (render D) all consume. The renders are serializers +
//! transports over **this one call**; they add no second match engine.
//!
//! # Claim-scope (ADR-043 Amendment 1 / ADR-044) — the syntactic/semantic line
//!
//! Every [`Finding`] this service emits is a [`FindingBody::FingerprintMatch`]: a
//! **syntactic FACT** — "this site's structure matches a known failure-class
//! fingerprint, at a calibrated tier" — and **never** a
//! [`FindingBody::DialVerdict`] (an *audited* verdict that the audit stage owns).
//! The two bodies are kept distinct on purpose: a fingerprint match that read as
//! an audited / defended / all-clear verdict would be antigen's own over-claim
//! class. The machine states what it matched; it does not ratify.
//!
//! The `class_provenance` is whatever the catalog **authored** for the class
//! (the honest tier — `Constructable`/`Encountered` for the flagships). The
//! service never invents or upgrades a provenance: a class the caller's
//! provenance map does not name is **not** projected (it is out of this catalog's
//! claim), never silently stamped with a default.

use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::finding::{
    DialTier, Finding, FindingBody, OriginStage, Presentation as FindingPresentation, Provenance,
    Severity, cluster_key_of,
};
use crate::scan::{MatchKind, ScanReport};

/// The emit-source string stamped on findings this service produces.
const CATALOG_MATCH_SOURCE: &str = "scan:catalog-match";

/// Project a scan's fingerprint-match presentations into the unified [`Finding`]
/// population, carrying each match's authored class-provenance (the E1 spine).
///
/// `provenance_by_class` maps an antigen class-name (the `Presentation`'s
/// `antigen_type`) to the provenance the catalog authored for it. Only
/// `FingerprintMatch` presentations whose class is in this map are projected:
///
/// - explicit-marker / generated presentations are out of scope (this is the
///   *match* render, not the declaration surface),
/// - a fingerprint match whose class is **not** in `provenance_by_class` is not
///   projected — it is outside this catalog's claim, so the service drops it
///   rather than stamp an invented provenance (the claim-scope honesty rule).
///
/// `source` is the emit-source string (e.g. `"scan:bundled-catalog"`); it lets a
/// consumer tell a bundled-catalog render apart from an in-tree catalog render
/// without changing the body.
#[must_use]
pub fn catalog_match_findings_with_source<S: BuildHasher>(
    report: &ScanReport,
    provenance_by_class: &HashMap<String, Provenance, S>,
    source: &str,
) -> Vec<Finding> {
    report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .filter_map(|p| {
            let class_provenance = *provenance_by_class.get(&p.antigen_type)?;
            Some((p, class_provenance))
        })
        .enumerate()
        .map(|(i, (p, class_provenance))| Finding {
            schema_version: crate::finding::FINDING_SCHEMA_VERSION,
            file: p.file.display().to_string(),
            line: p.line,
            structural_digest: p.structural_fingerprint.clone(),
            // A matched-item Finding keys on IDENTITY (structural_digest), not on
            // body-shape — shape_digest is the marked-unknown PROPOSE-slice's key,
            // empty here (two-field model, ADR-045 Amd-2).
            shape_digest: String::new(),
            cluster_key: cluster_key_of(&p.structural_fingerprint, &p.antigen_type),
            severity: severity_for(class_provenance),
            source: source.to_string(),
            class_provenance,
            // Tooling-side, scan-emitted, no user-macro burden — a fingerprint
            // match is a passive surfacing, not an authored active mark.
            presentation: FindingPresentation::Passive,
            // Index-based monotonic counter — no clock dependency in this pure
            // projection; a real wall-clock emit is a downstream concern.
            timestamp: i as u64,
            origin_stage: OriginStage::Scan,
            body: FindingBody::FingerprintMatch {
                class: p.antigen_type.clone(),
                // Shape-only scan floor: a fingerprint match has not been
                // graduated by a site-specific audit, so it rides the suspected
                // (non-loud) tier.
                tier: DialTier::Suspected,
            },
        })
        .collect()
}

/// As [`catalog_match_findings_with_source`] with the default
/// `"scan:catalog-match"` source.
#[must_use]
pub fn catalog_match_findings<S: BuildHasher>(
    report: &ScanReport,
    provenance_by_class: &HashMap<String, Provenance, S>,
) -> Vec<Finding> {
    catalog_match_findings_with_source(report, provenance_by_class, CATALOG_MATCH_SOURCE)
}

/// Map a class-provenance to a routing severity (cytokine routing, charter): the
/// verified-core tiers are `High`; the unverified tiers are `Medium`.
const fn severity_for(provenance: Provenance) -> Severity {
    match provenance {
        Provenance::Constructable | Provenance::Encountered => Severity::High,
        Provenance::Heuristic | Provenance::Imagined => Severity::Medium,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::scan::{ItemTarget, Presentation};

    fn fp_presentation(class: &str, file: &str, line: usize, digest: &str) -> Presentation {
        Presentation {
            antigen_type: class.to_string(),
            file: PathBuf::from(file),
            line,
            item_kind: "fn".to_string(),
            item_target: ItemTarget::Fn(format!("f{line}")),
            match_kind: MatchKind::FingerprintMatch,
            canonical_path: None,
            inherited_from: None,
            structural_fingerprint: digest.to_string(),
            requires_predicate: None,
            proof: None,
        }
    }

    #[test]
    fn projects_only_classes_in_the_provenance_map() {
        let mut report = ScanReport::default();
        report
            .presentations
            .push(fp_presentation("known-class", "a.rs", 1, "d1"));
        report
            .presentations
            .push(fp_presentation("unknown-class", "b.rs", 2, "d2"));

        let mut prov = HashMap::new();
        prov.insert("known-class".to_string(), Provenance::Constructable);

        let findings = catalog_match_findings(&report, &prov);
        // The unknown class (not in the catalog's claim) is dropped, never
        // stamped with an invented provenance.
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].class_provenance, Provenance::Constructable);
        assert!(matches!(
            &findings[0].body,
            FindingBody::FingerprintMatch { class, .. } if class == "known-class"
        ));
    }

    #[test]
    fn never_emits_an_audited_verdict_body() {
        let mut report = ScanReport::default();
        report
            .presentations
            .push(fp_presentation("c", "a.rs", 1, "d"));
        let mut prov = HashMap::new();
        prov.insert("c".to_string(), Provenance::Encountered);

        let findings = catalog_match_findings(&report, &prov);
        for f in &findings {
            assert!(!matches!(f.body, FindingBody::DialVerdict { .. }));
            assert_eq!(f.origin_stage, OriginStage::Scan);
        }
    }

    #[test]
    fn explicit_marker_presentations_are_not_projected() {
        // The match render is the FingerprintMatch surface only — an explicit
        // #[presents] marker is a declaration, not a catalog match.
        let mut report = ScanReport::default();
        let mut explicit = fp_presentation("c", "a.rs", 1, "d");
        explicit.match_kind = MatchKind::ExplicitMarker;
        report.presentations.push(explicit);
        let mut prov = HashMap::new();
        prov.insert("c".to_string(), Provenance::Constructable);

        let findings = catalog_match_findings(&report, &prov);
        assert!(
            findings.is_empty(),
            "explicit markers are not catalog matches"
        );
    }

    #[test]
    fn cluster_key_is_class_at_digest() {
        let mut report = ScanReport::default();
        report
            .presentations
            .push(fp_presentation("panic-in-drop", "a.rs", 9, "fnv:abc"));
        let mut prov = HashMap::new();
        prov.insert("panic-in-drop".to_string(), Provenance::Constructable);

        let findings = catalog_match_findings(&report, &prov);
        assert_eq!(findings[0].cluster_key, "panic-in-drop@fnv:abc");
    }
}
