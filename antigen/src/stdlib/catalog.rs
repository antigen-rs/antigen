//! The **bundled stdlib catalog** (v0.4 E0) — the compile-in projection of
//! antigen's flagship stdlib fingerprints.
//!
//! # Why this exists
//!
//! Each flagship stdlib antigen ships as a `fingerprint = r#"…"#` string literal
//! inside an `#[antigen(…)]` attribute (e.g. [`PanicInDrop`](crate::stdlib::drop_panic::PanicInDrop)).
//! There is no runtime accessor for those fingerprints, and a *consumer* crate
//! that depends on the published `antigen` crate has **no antigen source on
//! disk** — so re-parsing source to recover them yields nothing. The bundled
//! catalog is **compiled in** by [`build.rs`](https://github.com/antigen-rs/antigen)
//! (it parses antigen's own `src/stdlib/*.rs` at build time and emits a
//! `STDLIB_CATALOG` const into `OUT_DIR`), so the catalog travels inside the
//! `antigen` rlib to every consumer.
//!
//! This closes the **zero-hits-cliff** (ADR-043, v0.4): a crate with *zero*
//! antigen declarations would otherwise produce an empty `fingerprints` set,
//! `synthesis_pass` would never run, and the scan would report a false
//! all-clear. The bundled catalog supplies a non-empty default repertoire so a
//! fresh consumer crate gets real findings on its first scan.
//!
//! # Claim-scope (ADR-043 Amendment 1 / ADR-044)
//!
//! **What this proves:** a bundled-catalog scan reports *fingerprint matches* —
//! sites whose structure matches a flagship stdlib failure-class. Each carries a
//! [`Provenance`](crate::finding::Provenance) the catalog authored
//! (Constructable/Encountered for the flagships).
//!
//! **What this does NOT prove:** it does **not** audit a defense. A bundled
//! match is a "this site presents a known failure-class" signal, never an
//! "audited / defended" verdict (auditing a bundled use-case needs synthetic
//! declarations so sub-clause-F resolves — a deferred use-case). The render
//! must never let a bundled match read as an audited all-clear.
//!
//! **Who ratifies:** the human/incident reviewer decides whether a flagged site
//! is a real defect or an accepted presentation; the catalog only *surfaces* it.

use std::collections::HashMap;

use antigen_fingerprint::Fingerprint;

use crate::finding::{Finding, Provenance};
use crate::scan::ScanReport;

// The generated `STDLIB_CATALOG: &[(&str, &str, &str)]` — (name,
// fingerprint-string, provenance-string) — emitted by `build.rs` into OUT_DIR.
// `include!` lands it as a private const in this module.
include!(concat!(env!("OUT_DIR"), "/stdlib_catalog.rs"));

/// One bundled catalog entry, with its fingerprint already parsed and its
/// authored provenance resolved.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    /// The antigen's `name` (e.g. `"panic-in-drop"`).
    pub name: String,
    /// The parsed structural fingerprint.
    pub fingerprint: Fingerprint,
    /// The class provenance the stdlib declaration authored (the honest tier —
    /// the flagships are `Constructable`). An unauthored provenance resolves to
    /// [`Provenance::DEFAULT`] (`Imagined`, the floor), never an over-claim.
    pub provenance: Provenance,
}

/// The bundled stdlib catalog as `(name, parsed Fingerprint)` pairs, in the
/// shape `synthesis_pass` consumes for its `fingerprints` argument.
///
/// This is the public E0 accessor: a consumer crate (or `cargo antigen scan
/// --bundled-catalog`) calls this to get antigen's flagship repertoire without
/// antigen's source present.
///
/// # Panics
///
/// Never panics in practice: every bundled fingerprint string was parsed by the
/// same grammar at build time (it is a shipped stdlib fingerprint). A malformed
/// entry here would be an antigen bug, not adopter input; the
/// [`stdlib_catalog_checked`] variant surfaces a parse error instead if a caller
/// prefers to handle it.
#[must_use]
pub fn stdlib_catalog() -> Vec<(String, Fingerprint)> {
    stdlib_catalog_entries()
        .into_iter()
        .map(|e| (e.name, e.fingerprint))
        .collect()
}

/// The bundled stdlib catalog as rich [`CatalogEntry`] values.
///
/// Each entry carries name + parsed fingerprint + authored provenance. The
/// provenance is what the bundled-catalog scan attaches to each emitted match
/// for claim-scope honesty (ADR-044).
///
/// # Panics
///
/// Panics if a bundled fingerprint string fails to parse — an antigen bug, since
/// every entry is a shipped stdlib fingerprint. Use [`stdlib_catalog_checked`]
/// for a non-panicking variant.
#[must_use]
pub fn stdlib_catalog_entries() -> Vec<CatalogEntry> {
    stdlib_catalog_checked().unwrap_or_else(|e| {
        panic!("antigen bug: a bundled stdlib fingerprint failed to parse: {e}")
    })
}

/// The bundled stdlib catalog, surfacing a parse error rather than panicking.
///
/// # Errors
///
/// Returns the first [`syn::Error`] if any bundled fingerprint string fails to
/// re-parse (an antigen-internal invariant violation — every shipped stdlib
/// fingerprint parses).
pub fn stdlib_catalog_checked() -> syn::Result<Vec<CatalogEntry>> {
    STDLIB_CATALOG
        .iter()
        .map(|(name, fp_str, prov_str)| {
            let fingerprint = Fingerprint::parse(fp_str)?;
            // An unrecognized provenance string is impossible from build.rs (it
            // emits the variant ident verbatim), but default to the honest floor
            // rather than over-claim if it ever happens.
            let provenance = Provenance::from_variant_str(prov_str).unwrap_or(Provenance::DEFAULT);
            Ok(CatalogEntry {
                name: (*name).to_string(),
                fingerprint,
                provenance,
            })
        })
        .collect()
}

/// The number of entries in the bundled catalog (a `const`-friendly accessor for
/// tests / callers that only need the count without parsing).
#[must_use]
pub const fn stdlib_catalog_len() -> usize {
    STDLIB_CATALOG.len()
}

/// A `name → authored Provenance` map over the bundled catalog, for projecting a
/// bundled-catalog scan's fingerprint matches into claim-scoped [`Finding`]s.
fn catalog_provenance_map() -> HashMap<String, Provenance> {
    stdlib_catalog_entries()
        .into_iter()
        .map(|e| (e.name, e.provenance))
        .collect()
}

/// Project a **bundled-catalog scan**'s fingerprint matches into [`Finding`]s,
/// carrying each match's authored class-provenance (v0.4 E0 — the claim-scoped
/// render input).
///
/// Thin specialization of the E1 catalog-match spine
/// ([`catalog_match_findings`](crate::scan::catalog_match_findings)) for the
/// bundled stdlib catalog: it supplies the bundled catalog's authored
/// `name → Provenance` map and the `"scan:bundled-catalog"` emit-source. The
/// spine owns the claim-scope discipline — every emitted finding is a
/// scan-fact fingerprint-match body, never an audited verdict, and only catalog
/// classes are projected (ADR-043 Amendment 1 / ADR-044). See that function for
/// the full claim-scope contract.
#[must_use]
pub fn bundled_catalog_findings(report: &ScanReport) -> Vec<Finding> {
    // Delegate to the E1 catalog-match spine, supplying the bundled catalog's
    // authored provenance map + the bundled emit-source. The spine owns the
    // claim-scope discipline (FingerprintMatch body, never DialVerdict; only
    // catalog classes projected); this wrapper just names the bundled catalog.
    crate::scan::catalog_match_findings_with_source(
        report,
        &catalog_provenance_map(),
        "scan:bundled-catalog",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{FindingBody, OriginStage};
    use crate::scan::MatchKind;

    #[test]
    fn bundled_catalog_is_non_empty() {
        // The whole point of E0: a non-empty default repertoire so a zero-
        // declaration consumer crate does not hit the zero-hits-cliff.
        assert!(
            stdlib_catalog_len() > 0,
            "the bundled stdlib catalog must ship at least one flagship fingerprint"
        );
    }

    #[test]
    fn every_bundled_fingerprint_parses() {
        // Build-time-derived; re-parsing here is the runtime half of the
        // single-source guarantee. A failure means build.rs emitted a malformed
        // fingerprint string — an antigen bug, caught here.
        let entries = stdlib_catalog_checked().expect("every bundled fingerprint parses");
        assert_eq!(entries.len(), stdlib_catalog_len());
    }

    #[test]
    fn flagship_provenance_is_in_the_honest_verified_core() {
        // Claim-scope (ADR-044): the bundled flagships carry a verified-core
        // provenance (Constructable/Encountered) — never a manufactured
        // Imagined/Heuristic. This is what lets the bundled-catalog render claim
        // a real match without over-claiming.
        let entries = stdlib_catalog_entries();
        for e in &entries {
            assert!(
                matches!(
                    e.provenance,
                    Provenance::Constructable | Provenance::Encountered
                ),
                "bundled flagship `{}` must carry a verified-core provenance, got {:?}",
                e.name,
                e.provenance
            );
        }
    }

    #[test]
    fn known_flagship_present() {
        // panic-in-drop is the canonical flagship; assert it's bundled so a
        // regression in build.rs extraction is caught.
        let names: Vec<String> = stdlib_catalog_entries()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert!(
            names.iter().any(|n| n == "panic-in-drop"),
            "expected the `panic-in-drop` flagship in the bundled catalog; got {names:?}"
        );
    }

    // ---- E0 acceptance (baton §2 E0 spec) ----

    fn e0_fixture() -> std::path::PathBuf {
        std::path::Path::new("tests")
            .join("fixtures")
            .join("e0_bundled_catalog_consumer")
    }

    #[test]
    fn zero_declaration_crate_without_bundled_catalog_is_a_false_all_clear() {
        // The zero-hits-cliff, demonstrated: a zero-declaration crate scanned the
        // PLAIN way produces no fingerprint matches (empty `fingerprints` →
        // synthesis_pass never runs). This is the correctness bug E0 closes.
        let report = crate::scan::scan_workspace(&e0_fixture(), None).expect("scan succeeds");
        assert!(
            report.antigens.is_empty(),
            "fixture is a zero-declaration consumer crate"
        );
        let fp_matches = report
            .presentations
            .iter()
            .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
            .count();
        assert_eq!(
            fp_matches, 0,
            "without the bundled catalog, a zero-declaration crate gets zero matches (the false all-clear)"
        );
    }

    #[test]
    fn bundled_catalog_scan_finds_real_failure_classes() {
        // E0 spec: a fresh zero-declaration crate scanned --bundled-catalog
        // produces >=1 real finding from the stdlib catalog.
        let report = crate::scan::scan_workspace_bundled_catalog(&e0_fixture(), None, true)
            .expect("bundled-catalog scan succeeds");
        let findings = bundled_catalog_findings(&report);
        assert!(
            !findings.is_empty(),
            "the bundled catalog must surface >=1 real finding on the consumer fixture; got none"
        );
        // The fixture's UnwindBomb is a panic-in-drop site — assert it's caught.
        assert!(
            findings.iter().any(|f| {
                matches!(&f.body, FindingBody::FingerprintMatch { class, .. } if class == "panic-in-drop")
            }),
            "expected a panic-in-drop match on the fixture's UnwindBomb; got {:?}",
            findings.iter().map(|f| &f.body).collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_bundled_finding_carries_a_verified_core_provenance() {
        // E0 spec / claim-scope (ADR-043 Amd-1 + ADR-044): every emitted finding
        // carries class_provenance in {Constructable, Encountered} — the honest
        // verified core, never a manufactured tier.
        let report = crate::scan::scan_workspace_bundled_catalog(&e0_fixture(), None, true)
            .expect("bundled-catalog scan succeeds");
        let findings = bundled_catalog_findings(&report);
        assert!(!findings.is_empty(), "precondition: >=1 finding");
        for f in &findings {
            assert!(
                matches!(
                    f.class_provenance,
                    Provenance::Constructable | Provenance::Encountered
                ),
                "every bundled finding must carry a verified-core provenance; {f:?} did not"
            );
        }
    }

    #[test]
    fn no_bundled_finding_claims_an_audited_defense_verdict() {
        // E0 spec / the syntactic-semantic line (ADR-044): a bundled-catalog match
        // is a SCAN-FACT (FingerprintMatch), never an audited DialVerdict. A
        // FingerprintMatch body asserts "structure matches a known class", not
        // "audited / defended / all-clear" — the over-claim antigen forecloses.
        let report = crate::scan::scan_workspace_bundled_catalog(&e0_fixture(), None, true)
            .expect("bundled-catalog scan succeeds");
        let findings = bundled_catalog_findings(&report);
        assert!(!findings.is_empty(), "precondition: >=1 finding");
        for f in &findings {
            assert!(
                !matches!(f.body, FindingBody::DialVerdict { .. }),
                "a bundled-catalog match must NEVER masquerade as an audited DialVerdict; {f:?}"
            );
            assert_eq!(
                f.origin_stage,
                OriginStage::Scan,
                "a bundled match is scan-emitted, not audit-emitted"
            );
        }
    }

    #[test]
    fn bundled_catalog_spares_the_clean_sibling() {
        // The fixture's CleanGuard is a Drop impl with NO panic source — the
        // negative-selection case. The bundled panic-in-drop fingerprint must NOT
        // bind it (no false positive on the safe sibling).
        let report = crate::scan::scan_workspace_bundled_catalog(&e0_fixture(), None, true)
            .expect("bundled-catalog scan succeeds");
        let findings = bundled_catalog_findings(&report);
        // Every panic-in-drop match must be on UnwindBomb, never CleanGuard.
        for f in &findings {
            if matches!(&f.body, FindingBody::FingerprintMatch { class, .. } if class == "panic-in-drop")
            {
                assert!(
                    !f.file.contains("CleanGuard"),
                    "panic-in-drop must spare the clean sibling"
                );
            }
        }
        // Stronger: there is exactly one Drop impl that reaches a panic source.
        let drop_matches = findings
            .iter()
            .filter(|f| matches!(&f.body, FindingBody::FingerprintMatch { class, .. } if class == "panic-in-drop"))
            .count();
        assert_eq!(
            drop_matches, 1,
            "exactly one panic-in-drop site (UnwindBomb) — the clean sibling is spared"
        );
    }
}
