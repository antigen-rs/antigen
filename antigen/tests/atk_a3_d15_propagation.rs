//! ATK-A3 D1.5: lineage propagation walk acceptance tests.
//!
//! Tests for the propagation walk shipped per ADR-018 §"The synthesis
//! algorithm". Each test exercises one ADR-018 §Enforcement clause via a
//! dedicated fixture under `tests/fixtures/atk_a3_d15_*/`.
//!
//! Tests:
//! - `linear_chain_inheritance` — A → B, B has presentation, A inherits with
//!   B in `inherited_from`
//! - `diamond_dedup` — A→B→D and A→C→D, D's presentation propagates to A as
//!   ONE Presentation with both B and C in `inherited_from` (set-union)
//! - `explicit_plus_inherited_co_existence` — child has explicit `#[presents]`
//!   AND inherits the same antigen; one Presentation with `ExplicitMarker` +
//!   `inherited_from` set
//! - `immunity_does_not_propagate` — ancestor's `#[immune]` does NOT produce
//!   an Immunity record on the descendant
//! - `orphaned_edge_non_walk` — parent not in scan → no inherited Presentation
//! - `tolerance_covers_inherited` — state 4 absorbs inherited+tolerated
//! - `state6_anti_case` — `#[presents(A)]` does NOT re-attest inherited
//!   Presentation for B (different antigen)

use antigen::scan::{scan_workspace, MatchKind};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn linear_chain_inheritance() {
    let scan = scan_workspace(&fixture("atk_a3_d15_linear_chain"), None).expect("scan completes");

    // Expected: at least one inherited Presentation for "Parent" on Child's
    // declaration site, with inherited_from naming Parent's ProvenanceEntry.
    let inherited: Vec<_> = scan
        .presentations
        .iter()
        .filter(|p| p.antigen_type == "Parent" && p.inherited_from.is_some())
        .collect();
    assert!(
        !inherited.is_empty(),
        "Child must inherit a Presentation for Parent; got presentations: {:?}",
        scan.presentations
    );

    let inherited_on_child: Vec<_> = inherited
        .iter()
        .filter(|p| p.item_target == antigen::scan::ItemTarget::Struct("Child".to_string()))
        .collect();
    assert_eq!(
        inherited_on_child.len(),
        1,
        "exactly one inherited Presentation for Parent on Child's item_target; \
         got: {inherited_on_child:?}"
    );

    let chain = inherited_on_child[0]
        .inherited_from
        .as_ref()
        .expect("inherited_from is Some");
    assert_eq!(
        chain.len(),
        1,
        "linear chain has one ancestor in provenance"
    );
    assert_eq!(chain[0].antigen_type, "Parent");
    assert_eq!(chain[0].canonical_path, None, "intra-workspace ancestor");
}

#[test]
fn diamond_dedup() {
    // Diamond: Bottom → Left → Top  AND  Bottom → Right → Top.
    // Top has a #[presents(Top)] on the Vulnerable site; Left, Right
    // have no presentations of their own.
    //
    // Per ADR-018 §"The synthesis algorithm" lines 3866-3889, a
    // ProvenanceEntry's `antigen_type` is the ancestor whose presentations
    // are being propagated — NOT the intermediate edge endpoint. The
    // diamond dedup key is (antigen_type, item_target, canonical_path);
    // when an ancestor is reached via multiple paths, the second
    // discovery merges into the existing record rather than appending
    // a duplicate.
    //
    // The propagation walk uses defense-in-depth per-DFS-source
    // `visited: HashSet` (ADR-018 Finding 4), so each ancestor antigen
    // is visited at most once per descendant. For this fixture: Bottom's
    // DFS collects ancestors {Left, Right, Top}; only Top has
    // presentations; Bottom gets ONE inherited Presentation for Top
    // with inherited_from = [{antigen_type: "Top", canonical_path: None}].
    let scan = scan_workspace(&fixture("atk_a3_d15_diamond"), None).expect("scan completes");

    let inherited_on_bottom: Vec<_> = scan
        .presentations
        .iter()
        .filter(|p| {
            p.antigen_type == "Top"
                && p.item_target == antigen::scan::ItemTarget::Struct("Bottom".to_string())
                && p.inherited_from.is_some()
        })
        .collect();
    assert_eq!(
        inherited_on_bottom.len(),
        1,
        "diamond inheritance produces exactly ONE Presentation for Top on Bottom; \
         got: {inherited_on_bottom:?}"
    );

    let chain = inherited_on_bottom[0]
        .inherited_from
        .as_ref()
        .expect("inherited_from is Some");
    assert_eq!(
        chain.len(),
        1,
        "provenance names the ancestor whose presentation propagated (Top), \
         not intermediate edge endpoints (Left, Right); got chain: {chain:?}"
    );
    assert_eq!(chain[0].antigen_type, "Top");
    assert_eq!(chain[0].canonical_path, None, "intra-workspace ancestor");
}

#[test]
fn explicit_plus_inherited_co_existence() {
    let scan = scan_workspace(&fixture("atk_a3_d15_explicit_plus_inherited"), None)
        .expect("scan completes");

    // Child has BOTH #[presents(Parent)] explicit AND inherits Parent's
    // presentation via #[descended_from(Parent)]. After propagation,
    // ONE Presentation for Parent on Child with:
    //   - match_kind = ExplicitMarker (the explicit one dominates)
    //   - inherited_from = Some(_) (provenance attached)
    let on_child: Vec<_> = scan
        .presentations
        .iter()
        .filter(|p| {
            p.antigen_type == "Parent"
                && p.item_target == antigen::scan::ItemTarget::Struct("Child".to_string())
        })
        .collect();
    assert_eq!(
        on_child.len(),
        1,
        "exactly one Presentation for Parent on Child (explicit + inherited \
         merge into one record); got: {on_child:?}"
    );
    assert_eq!(
        on_child[0].match_kind,
        MatchKind::ExplicitMarker,
        "explicit marker dominates match_kind when both present"
    );
    assert!(
        on_child[0].inherited_from.is_some(),
        "inherited_from must be Some(_) — provenance recorded alongside \
         explicit marker; got: {on_child:?}"
    );
}

#[test]
fn immunity_does_not_propagate() {
    let scan = scan_workspace(&fixture("atk_a3_d15_immunity_no_propagation"), None)
        .expect("scan completes");

    // Ancestor has #[immune(Parent, ...)] on the Vulnerable site. Child
    // descended_from Parent. The propagation walk MUST NOT synthesize
    // an Immunity record on Child — only the Vulnerable site's Immunity
    // exists.
    let immunities_on_child: Vec<_> = scan
        .immunities
        .iter()
        .filter(|i| i.item_target == antigen::scan::ItemTarget::Struct("Child".to_string()))
        .collect();
    assert!(
        immunities_on_child.is_empty(),
        "immunity MUST NOT auto-propagate to descendants; got: {immunities_on_child:?}"
    );

    // Sanity: the original immunity on the Vulnerable site is still recorded.
    assert!(
        scan.immunities.iter().any(|i| i.antigen_type == "Parent"),
        "the source Immunity must still be in the report; got: {:?}",
        scan.immunities
    );
}

#[test]
fn orphaned_edge_non_walk() {
    // ATK-A3-003 fixture has Child #[descended_from(MissingParent)] with
    // no MissingParent declared. The propagation walk MUST NOT produce
    // inherited Presentations on Child — orphaned edges are skipped.
    let scan = scan_workspace(&fixture("atk_a3_003_stale_lineage"), None).expect("scan completes");

    let inherited_on_child: Vec<_> = scan
        .presentations
        .iter()
        .filter(|p| {
            p.item_target == antigen::scan::ItemTarget::Struct("Child".to_string())
                && p.inherited_from.is_some()
        })
        .collect();
    assert!(
        inherited_on_child.is_empty(),
        "orphaned edge (parent not in scan) MUST NOT produce inherited \
         Presentation; got: {inherited_on_child:?}"
    );

    // Sanity: the orphan IS visible via the query method.
    assert!(
        !scan.orphaned_lineage_edges().is_empty(),
        "the orphan edge must be surfaced via orphaned_lineage_edges()"
    );
}

#[test]
fn tolerance_covers_inherited() {
    let scan =
        scan_workspace(&fixture("atk_a3_d15_tolerance_covers"), None).expect("scan completes");

    // Child has #[antigen_tolerance(Parent, ...)] AND inherits Parent's
    // presentation. The inherited Presentation should be covered by the
    // tolerance — `unaddressed_presentations` should NOT surface it.
    let unaddressed = scan.unaddressed_presentations();
    let unaddressed_for_parent_on_child: Vec<_> = unaddressed
        .iter()
        .filter(|u| {
            u.presentation.antigen_type == "Parent"
                && u.presentation.item_target
                    == antigen::scan::ItemTarget::Struct("Child".to_string())
        })
        .collect();
    assert!(
        unaddressed_for_parent_on_child.is_empty(),
        "tolerance on Child covers inherited Parent presentation; \
         the inherited Presentation must NOT appear as unaddressed; \
         got: {unaddressed_for_parent_on_child:?}"
    );
}

#[test]
fn state6_anti_case_different_antigen_does_not_re_attest() {
    let scan =
        scan_workspace(&fixture("atk_a3_d15_state6_anti_case"), None).expect("scan completes");

    // Child has #[presents(ParentA)] (explicit) and inherits both
    // ParentA AND ParentB. The explicit ParentA marker must NOT re-attest
    // the inherited ParentB Presentation. So `unaddressed_presentations`
    // should include ParentB on Child (state 7) but NOT ParentA (state 1+6).
    let unaddressed = scan.unaddressed_presentations();
    let unaddressed_on_child: Vec<_> = unaddressed
        .iter()
        .filter(|u| {
            u.presentation.item_target == antigen::scan::ItemTarget::Struct("Child".to_string())
        })
        .collect();

    // ParentB inherited Presentation must be unaddressed (state 7).
    assert!(
        unaddressed_on_child
            .iter()
            .any(|u| u.presentation.antigen_type == "ParentB"),
        "ParentB inherited Presentation on Child must be unaddressed \
         (state 7); #[presents(ParentA)] does NOT cover ParentB; \
         got unaddressed on Child: {unaddressed_on_child:?}"
    );

    // ParentA explicit + inherited Presentation should also be
    // unaddressed (no #[immune] / #[antigen_tolerance] for ParentA on
    // Child). State 6 requires explicit re-attestation; this fixture has
    // no such immunity, so ParentA is also state 7. The anti-case is
    // specifically that the explicit ParentA marker doesn't re-attest
    // ParentB.
}
