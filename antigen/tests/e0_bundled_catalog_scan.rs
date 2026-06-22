//! E0 — the bundled-stdlib-catalog scan mode. THE ACCEPTANCE GATE (the
//! behavioral half, testable against the PUBLIC API today).
//!
//! ADR-043 Amd-1 (match-render only, claim-scoped); ADR-044 (claim-scope
//! honesty).
//!
//! THE E0 SPEC (briefing §2 E0): a fresh ZERO-DECLARATION crate scanned with the
//! bundled catalog produces ≥1 real finding from the stdlib catalog; every
//! emitted finding carries `class_provenance ∈ {Constructable, Encountered}`; NO
//! finding claims an audited defense verdict.
//!
//! This file pins the parts of that gate that are checkable against the SHIPPED
//! public API:
//!   1. The zero-hits-cliff bug is REAL (a zero-declaration scan today reports a
//!      false all-clear: empty `fingerprints` → `synthesis_pass` never runs).
//!   2. The bundled catalog, applied to the SAME consumer fixture, DOES match
//!      its real footguns (≥1 hit) — the value E0 delivers.
//!   3. Every catalog entry that produces a match carries a verified-core
//!      provenance (Constructable/Encountered) — the claim-scope tier honesty.
//!
//! The ENTRYPOINT-level gate (a public `scan + bundled-catalog` call that emits
//! the matches into the `Finding` population with provenance stamped + NO
//! audited-defense verdict) lives in `e0_bundled_catalog_entrypoint.rs`; it is
//! RED until the public catalog-match entrypoint ships.

use std::path::{Path, PathBuf};

use antigen::finding::Provenance;
use antigen::scan::{MatchKind, scan_workspace, scan_workspace_bundled_catalog};
use antigen::stdlib::catalog::{stdlib_catalog, stdlib_catalog_entries};
use antigen_fingerprint::Fingerprint;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Parse the consumer fixture's `lib.rs` into top-level `syn::Item`s, the way
/// the scan walk feeds `synthesis_pass`. Lets a test apply catalog fingerprints
/// at the public `Fingerprint::matches` level without reaching the `pub(crate)`
/// synthesis pass.
fn fixture_items(name: &str) -> Vec<syn::Item> {
    let src =
        std::fs::read_to_string(fixture(name).join("lib.rs")).expect("fixture lib.rs readable");
    syn::parse_file(&src).expect("fixture parses").items
}

/// Count, for a given catalog of `(name, Fingerprint)`, how many (entry, item)
/// pairs match — the raw match count the synthesis pass would surface.
fn catalog_match_count(catalog: &[(String, Fingerprint)], items: &[syn::Item]) -> usize {
    let mut hits = 0;
    for (_name, fp) in catalog {
        for item in items {
            // Mirror synthesis_pass's node-kind pre-filter then the full match.
            if let Some(required) = fp.node_kind() {
                let item_kind = item_kind_of(item);
                if item_kind != Some(required) {
                    continue;
                }
            }
            if fp.matches(item) {
                hits += 1;
            }
        }
    }
    hits
}

const fn item_kind_of(item: &syn::Item) -> Option<antigen_fingerprint::ItemKind> {
    use antigen_fingerprint::ItemKind;
    Some(match item {
        syn::Item::Struct(_) => ItemKind::Struct,
        syn::Item::Enum(_) => ItemKind::Enum,
        syn::Item::Trait(_) => ItemKind::Trait,
        syn::Item::Fn(_) => ItemKind::Fn,
        syn::Item::Impl(_) => ItemKind::Impl,
        syn::Item::Type(_) => ItemKind::Type,
        syn::Item::Mod(_) => ItemKind::Mod,
        syn::Item::Const(_) => ItemKind::Const,
        syn::Item::Static(_) => ItemKind::Static,
        syn::Item::Union(_) => ItemKind::Union,
        _ => return None,
    })
}

// ===========================================================================
// (1) The zero-hits-cliff bug is REAL. A zero-declaration consumer crate scan
//     reports a false all-clear. (GREEN today — documents the bug E0 closes.)
// ===========================================================================

#[test]
fn zero_declaration_consumer_scan_finds_nothing_without_the_catalog() {
    // The consumer fixture has REAL footguns (get_unchecked, panic-in-Drop) but
    // ZERO antigen declarations. Today's scan builds its fingerprint set ONLY
    // from in-tree #[antigen] decls, so the set is empty and synthesis_pass never
    // runs → a false all-clear.
    let scan = scan_workspace(&fixture("e0_consumer_crate_zero_decls"), None).expect("scan");
    assert_eq!(
        scan.antigens.len(),
        0,
        "the consumer fixture must declare ZERO antigens (it stands in for an \
         install-and-scan adopter); got {}",
        scan.antigens.len()
    );
    assert_eq!(
        scan.presentations.len(),
        0,
        "WITHOUT the bundled catalog the scan surfaces NOTHING — the zero-hits-cliff \
         false all-clear. This is the bug E0 closes. presentations = {:?}",
        scan.presentations
    );
}

// ===========================================================================
// (2) The bundled catalog, applied to the SAME fixture, matches its footguns.
//     (RED until E0 — the public catalog is now built, so this should flip GREEN
//     once the catalog ships; the ENTRYPOINT wiring is the separate gate.)
// ===========================================================================

#[test]
fn bundled_catalog_matches_the_consumer_fixtures_real_footguns() {
    let catalog = stdlib_catalog();
    assert!(
        !catalog.is_empty(),
        "the bundled catalog must be non-empty (E0's whole point)"
    );

    let items = fixture_items("e0_consumer_crate_zero_decls");
    let hits = catalog_match_count(&catalog, &items);

    assert!(
        hits >= 1,
        "the bundled catalog must produce AT LEAST ONE match against the consumer \
         fixture's real footguns (get_unchecked → panic-on-index; panic-in-Drop → \
         drop-and-panic). Zero hits here means the catalog does not actually close \
         the zero-hits-cliff for this crate. hits = {hits}"
    );
}

// ===========================================================================
// (3) Every catalog entry carries a verified-core provenance. This is the
//     claim-scope tier-honesty precondition: a bundled match can only ever read
//     as Constructable/Encountered, never a manufactured Imagined/Heuristic.
//     (The build.rs filters to 8 flagship modules to guarantee this;
//     this gate FALSIFIES that guarantee against the SHIPPED catalog, so a future
//     module add that drags in an unlabeled member trips here.)
// ===========================================================================

#[test]
fn every_bundled_catalog_entry_is_verified_core_provenance() {
    use antigen::finding::Provenance;
    let entries = stdlib_catalog_entries();
    assert!(!entries.is_empty(), "catalog must be non-empty");
    for e in &entries {
        assert!(
            matches!(
                e.provenance,
                Provenance::Constructable | Provenance::Encountered
            ),
            "bundled catalog entry `{}` carries provenance {:?} — the E0 claim-scope \
             requires every bundled finding to read in the verified core \
             {{Constructable, Encountered}}. An Imagined/Heuristic member bundled \
             into the default repertoire would over-claim on a fresh scan.",
            e.name,
            e.provenance
        );
    }
}

// ===========================================================================
// (4) THE ENTRYPOINT closes the cliff: a zero-declaration consumer scan WITH the
//     bundled catalog injected surfaces ≥1 FingerprintMatch presentation. This
//     exercises the REAL `scan_workspace_bundled_catalog` wiring (not a
//     hand-rolled match) — the consumer-crate path, no antigen source on disk.
// ===========================================================================

#[test]
fn bundled_catalog_entrypoint_surfaces_matches_on_a_zero_declaration_crate() {
    let scan = scan_workspace_bundled_catalog(&fixture("e0_consumer_crate_zero_decls"), None, true)
        .expect("bundled-catalog scan completes");

    assert_eq!(
        scan.antigens.len(),
        0,
        "the consumer fixture declares ZERO antigens; got {}",
        scan.antigens.len()
    );

    let fp_matches: Vec<_> = scan
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .collect();

    assert!(
        !fp_matches.is_empty(),
        "the bundled-catalog entrypoint must surface ≥1 FingerprintMatch on a \
         zero-declaration consumer crate (closing the zero-hits-cliff). Got {} \
         fingerprint-match presentations; all presentations: {:?}",
        fp_matches.len(),
        scan.presentations
    );
}

// ===========================================================================
// (5) CLAIM-SCOPE PROVENANCE at the entrypoint: every surfaced catalog match
//     resolves (by name) to a catalog member whose provenance is verified-core
//     (Constructable/Encountered). A match whose `antigen_type` is NOT in the
//     catalog, or that resolves to Imagined/Heuristic, is a wiring bug or an
//     over-claim — the render would have nothing honest (or something dishonest)
//     to stamp.
// ===========================================================================

#[test]
fn every_entrypoint_match_resolves_to_verified_core_provenance() {
    let scan = scan_workspace_bundled_catalog(&fixture("e0_consumer_crate_zero_decls"), None, true)
        .expect("bundled-catalog scan completes");

    let prov: std::collections::HashMap<String, Provenance> = stdlib_catalog_entries()
        .into_iter()
        .map(|e| (e.name, e.provenance))
        .collect();

    let fp_matches: Vec<_> = scan
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .collect();
    assert!(
        !fp_matches.is_empty(),
        "precondition: there must be ≥1 fingerprint match to check provenance on"
    );

    for p in fp_matches {
        let resolved = prov.get(&p.antigen_type).copied().unwrap_or_else(|| {
            panic!(
                "bundled match for `{}` at {}:{} resolves to NO catalog member — the \
                 claim-scoped render cannot stamp an honest provenance. Known members: {:?}",
                p.antigen_type,
                p.file.display(),
                p.line,
                prov.keys().collect::<Vec<_>>()
            )
        });
        assert!(
            matches!(
                resolved,
                Provenance::Constructable | Provenance::Encountered
            ),
            "entrypoint match `{}` resolves to provenance {resolved:?}; E0 claim-scope \
             requires verified-core {{Constructable, Encountered}}",
            p.antigen_type
        );
    }
}

// ===========================================================================
// (6) THE PARTIAL-ADOPTER AUTO-DETECT TRAP (a design-stress finding pinned as a
//     deliberate spec decision). `scan_workspace_bundled_catalog`'s auto-detect
//     mode injects the catalog ONLY when zero in-tree antigens exist. So a crate
//     with ONE local declaration loses ALL flagship catalog coverage under
//     auto-detect — even a real get_unchecked footgun is silently missed.
//
//     The CLI `--bundled-catalog` flag drives auto_detect=true (main.rs), so a
//     user who EXPLICITLY asks for the catalog but has one local antigen gets no
//     flagship coverage. This test pins BOTH modes so the gap is a CONSCIOUS
//     choice: if `--bundled-catalog` is decided to mean "always
//     inject" (the explicit-request reading), the auto-detect assertion below
//     trips and forces the change deliberately rather than letting a silent miss
//     ship.
// ===========================================================================

fn partial_adopter_get_unchecked_matches(auto_detect: bool) -> usize {
    let scan =
        scan_workspace_bundled_catalog(&fixture("e0_partial_adopter_one_decl"), None, auto_detect)
            .expect("scan completes");
    assert_eq!(
        scan.antigens.len(),
        1,
        "the partial-adopter fixture declares exactly one local antigen"
    );
    scan.presentations
        .iter()
        .filter(|p| {
            p.match_kind == MatchKind::FingerprintMatch
                && p.antigen_type == "get-unchecked-without-proof"
        })
        .count()
}

#[test]
fn partial_adopter_auto_detect_suppresses_the_catalog_a_known_silent_miss() {
    // AUTO-DETECT: one in-tree antigen ⇒ catalog NOT injected ⇒ the flagship
    // get_unchecked footgun is MISSED. This is the trap. If this flips to >0, the
    // auto-detect semantics changed (likely deliberately — verify
    // `--bundled-catalog` was decided to mean always-inject) and this fence updates.
    let auto = partial_adopter_get_unchecked_matches(true);
    assert_eq!(
        auto, 0,
        "auto-detect currently SUPPRESSES the catalog for a partial adopter (one \
         local antigen) — the flagship get_unchecked footgun is silently missed. \
         If this is now >0, the auto-detect rule changed; confirm it was a \
         deliberate spec decision (explicit --bundled-catalog ⇒ always inject)."
    );
}

#[test]
fn partial_adopter_always_mode_catches_the_flagship_footgun() {
    // ALWAYS: the catalog augments the in-tree repertoire ⇒ the flagship footgun
    // IS caught. The safety net exists; the only question is which mode the CLI
    // flag should drive.
    let always = partial_adopter_get_unchecked_matches(false);
    assert_eq!(
        always, 1,
        "Always-mode must augment a partial adopter's in-tree antigens with the \
         bundled catalog so the flagship get_unchecked footgun is caught; got {always}"
    );
}
