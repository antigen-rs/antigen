//! C ══ B — the AUTOIMMUNITY SAFETY GATE. The single safety-tangle on the v0.4
//! chart (ADR-045: the one safety-tangle; captain: highest-stakes test).
//!
//! Campsites: `keystone/self-tolerance-negative-selection-anchor`,
//! `dream/affinity-maturation-engine`, `dream/self-tolerance-negative-selection-engine`.
//!
//! THE TANGLE (proven run-as-code by the pathmaker's spike, re-proven here):
//!   - C (PROPOSE) anti-unifies a cluster of structurally-similar defective sites
//!     into a DRAFT fingerprint.
//!   - NAIVE-LGG (drop the differing leaves) OVER-GENERALIZES: it drops
//!     `body_calls(unwrap)` / `body_calls(expect)` and collapses to "any Drop
//!     impl" — which MATCHES A CLEAN DROP SIBLING. Flagging clean code is the
//!     AUTOIMMUNITY. The ungoverned generator's own output IS the false positive.
//!   - ANTI-UNIFY-TO-DISJUNCTION (`any_of([body_calls(unwrap), body_calls(expect)])`)
//!     binds the family AND spares the clean sibling.
//!   - B (the self-tolerance / spare-clean gate) is the selector that REJECTS a
//!     draft matching a clean sibling. **C must NEVER promote a draft without B
//!     green.** Even the disjunction draft must pass B (a disjunction over a
//!     cluster that happens to share a clean-sibling-binding leaf would still
//!     over-bind — so B is required regardless of how PROPOSE generalizes).
//!
//! This gate pins the SAFETY PROPERTY against the SHIPPED public
//! `Fingerprint::matches` + the SHIPPED B-gate
//! (`antigen::learn::self_tolerance::spare_clean`). Three layers:
//!
//! - (A) the property is real — the naive draft binds the clean sibling, the
//!   disjunction draft spares it (so the gate has something to reject/accept);
//! - (B) the B-gate contract — `spare_clean` rejects the naive draft, accepts
//!   the disjunction draft;
//! - (C) THE HARD CASE (ADR-045's warning) — even a DISJUNCTION draft that
//!   happens to include a clean-sibling-binding leaf is autoimmune, and B must
//!   STILL reject it. This is why B is required regardless of how PROPOSE
//!   generalizes: anti-unify-to-disjunction REDUCES but does not ELIMINATE the
//!   autoimmunity; only the corpus-checked gate eliminates it.

use std::path::{Path, PathBuf};

use antigen_fingerprint::Fingerprint;

fn fixture_items(name: &str) -> Vec<syn::Item> {
    let p: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
        .join("lib.rs");
    let src = std::fs::read_to_string(p).expect("fixture lib.rs readable");
    syn::parse_file(&src).expect("fixture parses").items
}

/// Find the `impl Drop for <type_name>` item in the fixture.
fn drop_impl_for(items: &[syn::Item], type_name: &str) -> syn::Item {
    items
        .iter()
        .find(|it| {
            let syn::Item::Impl(i) = it else { return false };
            let Some((_, trait_path, _)) = &i.trait_ else {
                return false;
            };
            let is_drop = trait_path
                .segments
                .last()
                .is_some_and(|s| s.ident == "Drop");
            let syn::Type::Path(tp) = &*i.self_ty else {
                return false;
            };
            let on_type = tp
                .path
                .segments
                .last()
                .is_some_and(|s| s.ident == type_name);
            is_drop && on_type
        })
        .unwrap_or_else(|| panic!("no `impl Drop for {type_name}` in fixture"))
        .clone()
}

/// The NAIVE-LGG draft: drops the differing `body_calls` leaves → "any Drop
/// impl". This is the autoimmune over-generalization PROPOSE must NOT promote.
fn naive_lgg_draft() -> Fingerprint {
    Fingerprint::parse(r#"all_of([item = impl, impl_of_trait("Drop")])"#)
        .expect("naive draft parses")
}

/// The ANTI-UNIFY-TO-DISJUNCTION draft: generalizes the differing same-leaf
/// payloads to `any_of([body_calls(unwrap), body_calls(expect)])`. Binds the
/// family, spares the clean sibling.
fn disjunction_draft() -> Fingerprint {
    Fingerprint::parse(
        r#"all_of([item = impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), body_calls("expect")])])"#,
    )
    .expect("disjunction draft parses")
}

// ===========================================================================
// (A1) THE AUTOIMMUNITY IS REAL: the naive draft MATCHES THE CLEAN SIBLING.
//      If this ever stops being true, the safety tangle's premise changed —
//      re-derive before relaxing B.
// ===========================================================================

#[test]
fn naive_lgg_draft_is_autoimmune_it_binds_the_clean_sibling() {
    let items = fixture_items("autoimmunity_drop_family");
    let guard_a = drop_impl_for(&items, "GuardA"); // .unwrap()
    let guard_b = drop_impl_for(&items, "GuardB"); // .expect()
    let clean = drop_impl_for(&items, "CleanGuard"); // .ok() — no panic source

    let naive = naive_lgg_draft();

    // It binds the real defects (good) …
    assert!(
        naive.matches(&guard_a) && naive.matches(&guard_b),
        "the naive draft must bind the defective family (both GuardA and GuardB)"
    );
    // … AND it binds the CLEAN sibling (the autoimmunity — the whole problem).
    assert!(
        naive.matches(&clean),
        "the NAIVE-LGG draft MUST match the CLEAN sibling — this is the autoimmune \
         false positive (it over-generalized to 'any Drop impl'). If this is false, \
         the autoimmunity premise changed; B's necessity must be re-derived."
    );
}

// ===========================================================================
// (A2) THE SAFE DRAFT WORKS: the disjunction draft binds the family AND SPARES
//      the clean sibling. This is the target PROPOSE must produce.
// ===========================================================================

#[test]
fn disjunction_draft_binds_the_family_and_spares_the_clean_sibling() {
    let items = fixture_items("autoimmunity_drop_family");
    let guard_a = drop_impl_for(&items, "GuardA");
    let guard_b = drop_impl_for(&items, "GuardB");
    let clean = drop_impl_for(&items, "CleanGuard");

    let disj = disjunction_draft();

    assert!(
        disj.matches(&guard_a) && disj.matches(&guard_b),
        "the disjunction draft must BIND the defective family (GuardA via unwrap, \
         GuardB via expect)"
    );
    assert!(
        !disj.matches(&clean),
        "the disjunction draft must SPARE the clean sibling (.ok(), no \
         unwrap/expect) — this is what anti-unify-to-`any_of` buys over naive-LGG. \
         If it matches, the draft is still autoimmune."
    );
}

// ===========================================================================
// (B) THE B-GATE CONTRACT (the safety gate itself). `spare_clean(draft,
//     clean_corpus)` must REJECT a draft that matches ANY clean-corpus item, and
//     ACCEPT one that spares them all. C must never PROMOTE a draft B rejects.
//
//     LANDED (pathmaker): the gate shipped as
//     `antigen::learn::self_tolerance::spare_clean`. This gate is now wired to the
//     real implementation — GREEN means the shipped gate enforces the contract.
// ===========================================================================

/// The shipped B-gate: `true` iff `draft` is SAFE to promote (spares every
/// clean-corpus item). The B half of the C ══ B co-ship; C must never promote a
/// draft this rejects.
fn b_gate_spares_clean(draft: &Fingerprint, clean_corpus: &[syn::Item]) -> bool {
    antigen::learn::self_tolerance::spare_clean(draft, clean_corpus)
}

#[test]
fn b_gate_rejects_the_autoimmune_draft_and_accepts_the_safe_one() {
    let items = fixture_items("autoimmunity_drop_family");
    let clean_corpus: Vec<syn::Item> = vec![drop_impl_for(&items, "CleanGuard")];

    // (1) B MUST REJECT the naive (autoimmune) draft — it binds the clean sibling.
    assert!(
        !b_gate_spares_clean(&naive_lgg_draft(), &clean_corpus),
        "B MUST REJECT the naive-LGG draft: it matches the clean sibling, so \
         promoting it would flag clean code (autoimmunity). A gate that accepts \
         this draft has failed its one safety job."
    );

    // (2) B MUST ACCEPT the disjunction draft — it spares the clean sibling.
    assert!(
        b_gate_spares_clean(&disjunction_draft(), &clean_corpus),
        "B must ACCEPT the disjunction draft: it spares every clean-corpus sibling \
         while binding the family. This is the promotable draft."
    );
}

// ===========================================================================
// (C) THE HARD CASE — a DISJUNCTION draft that STILL over-binds (ADR-045's
//     warning made concrete). anti-unify-to-`any_of` is not a free pass: if the
//     PROPOSE step folds a leaf that the CLEAN sibling also satisfies into the
//     disjunction, the draft binds the clean sibling DESPITE being a disjunction.
//     B must STILL reject it. This is the test the B unit-tests do NOT do (they
//     only check the obvious naive vs clean disjunction) — and it's the whole
//     reason C ══ B is non-negotiable: a smarter generalizer is not a substitute
//     for the corpus-checked gate.
// ===========================================================================

#[test]
fn b_gate_rejects_a_disjunction_that_still_binds_a_clean_sibling() {
    let items = fixture_items("autoimmunity_drop_family");
    let clean_corpus: Vec<syn::Item> = vec![drop_impl_for(&items, "CleanGuard")]; // uses .ok()
    let guard_a = drop_impl_for(&items, "GuardA"); // uses .unwrap()

    // A disjunction draft where ONE arm — body_calls("ok") — matches the CLEAN
    // sibling (CleanGuard's drop body calls `.ok()`). A careless PROPOSE that
    // anti-unified the cluster's leaves INCLUDING a leaf the clean sibling shares
    // produces exactly this. It is a disjunction, but it is autoimmune.
    let autoimmune_disjunction = Fingerprint::parse(
        r#"all_of([item = impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), body_calls("ok")])])"#,
    )
    .expect("parses");

    // Premise: it DOES bind a real defect (GuardA via the unwrap arm) — so a
    // naive "does it match the family?" check would green-light it …
    assert!(
        autoimmune_disjunction.matches(&guard_a),
        "premise: the autoimmune disjunction still binds the defective family"
    );
    // … AND it binds the CLEAN sibling (via the .ok() arm) — the autoimmunity a
    // disjunction does NOT prevent.
    assert!(
        autoimmune_disjunction.matches(&clean_corpus[0]),
        "premise: this disjunction ALSO binds the clean sibling (the .ok() arm) — \
         a disjunction is not automatically safe"
    );

    // THE GATE: B MUST reject it despite it being a disjunction.
    assert!(
        !b_gate_spares_clean(&autoimmune_disjunction, &clean_corpus),
        "B MUST REJECT a disjunction draft that binds a clean sibling — being a \
         disjunction is NOT sufficient for safety. If B accepts this, a smarter \
         PROPOSE generalizer that folds a clean-binding leaf into the disjunction \
         would ship autoimmunity. This is the case the B unit-tests miss and the \
         reason C ══ B (corpus-checked gate) is non-negotiable."
    );
    assert!(
        antigen::learn::self_tolerance::promote_if_safe(autoimmune_disjunction, &clean_corpus)
            .is_none(),
        "promote_if_safe must structurally refuse to promote the autoimmune disjunction"
    );
}

// ===========================================================================
// (D) THE FALSIFICATION GATE for C (PROPOSE) — the keystone's falsifiability
//     proof (briefing §1 STREAM-LEARN: "produce ONE real draft fingerprint on
//     antigen's own marks that binds the cluster and spares clean — this is what
//     makes charter-learning-core falsifiable-not-faith").
//
//     The end-to-end contract C must satisfy: given a CLUSTER of structurally
//     similar DEFECTIVE marked sites, C produces a DRAFT that (1) BINDS every
//     cluster member, AND (2) PASSES B (spares the clean corpus). C must
//     anti-unify TO DISJUNCTION (the naive drop-leaves LGG over-generalizes to
//     match clean code — proven in test (A1)). C promotes ONLY through B.
//
//     RED-by-design, NON-BLOCKING (`#[ignore]`): C (PROPOSE) is unbuilt. The stub
//     names the contract; the pathmaker points `propose_draft` at the real
//     generator (e.g. `antigen::learn::propose::anti_unify(cluster)`) and drops
//     the `#[ignore]`. Discover with `-- --ignored`.
// ===========================================================================

/// THE C (PROPOSE) generator — LANDED as `antigen::learn::propose::anti_unify`.
/// Anti-unifies a cluster of structurally-similar defective items into a draft
/// fingerprint. Returns `None` if the cluster has no shared skeleton.
fn propose_draft(cluster: &[syn::Item]) -> Fingerprint {
    antigen::learn::propose::anti_unify(cluster)
        .expect("the defective cluster has a shared skeleton to anti-unify")
}

#[test]
fn propose_produces_a_draft_that_binds_the_cluster_and_passes_b() {
    let items = fixture_items("autoimmunity_drop_family");
    // The DEFECTIVE cluster C anti-unifies (GuardA via .unwrap(), GuardB via .expect()).
    let cluster = vec![
        drop_impl_for(&items, "GuardA"),
        drop_impl_for(&items, "GuardB"),
    ];
    // The CLEAN corpus B checks the draft against.
    let clean_corpus = vec![drop_impl_for(&items, "CleanGuard")];

    let draft = propose_draft(&cluster);

    // (1) THE DRAFT BINDS THE CLUSTER — a draft that doesn't match the sites it
    //     was generalized from is useless (over-specialized or wrong).
    for (i, member) in cluster.iter().enumerate() {
        assert!(
            draft.matches(member),
            "the PROPOSE draft must BIND every cluster member it was anti-unified \
             from; it missed member {i}"
        );
    }

    // (2) THE DRAFT PASSES B — it spares the clean corpus (anti-unify-to-disjunction
    //     achieved, not the autoimmune naive collapse). This is the falsifiable
    //     proof: a real draft on real marks that binds the bad and spares the good.
    assert!(
        b_gate_spares_clean(&draft, &clean_corpus),
        "the PROPOSE draft must PASS B (spare the clean sibling) — if it binds the \
         clean code, PROPOSE produced the autoimmune naive draft and must NOT be \
         promoted. C anti-unifies to disjunction precisely to spare the clean."
    );

    // (3) AND it promotes through B (the type-level co-ship enforcement).
    assert!(
        antigen::learn::self_tolerance::promote_if_safe(draft, &clean_corpus).is_some(),
        "a binds-cluster + spares-clean draft must promote through B"
    );
}

// ===========================================================================
// (E) THE NO-BYPASS GATE (the captain's "one thing that must not pass"). C's
//     END-TO-END promote path must route through B — a draft that binds a clean
//     sibling MUST NOT be promotable, no matter how C is structured internally.
//     This is a BLACK-BOX test of C's promote API: given the MIXED-PANIC-SHAPE
//     trap (the EASY generalization "any Drop impl" is autoimmune; the CORRECT
//     one is a disjunction that spares clean), C's promote path must yield EITHER
//     a draft that binds the cluster AND spares clean (the disjunction), OR None
//     (prune). The ONE outcome that must NOT happen: a promoted draft that binds
//     the clean sibling — that means C took the easy autoimmune path and B did
//     not stop it. A safe disjunction draft DOES exist here, so a strong C
//     promotes it; a weaker C prunes; neither is a failure. Only bypass is.
//
//     RED-by-design, NON-BLOCKING (`#[ignore]`): C's promote path is unbuilt. The
//     stub names the SYNC-2 contract; the pathmaker points `propose_and_promote`
//     at C's real end-to-end promote entrypoint (the one that internally consults
//     B) and drops the `#[ignore]`.
// ===========================================================================

/// C's END-TO-END PROMOTE path (SYNC-2) — LANDED as `antigen::learn::propose::propose`.
/// Anti-unifies the cluster, then promotes ONLY through B's spare-clean gate.
/// `Some(draft)` is structurally guaranteed to spare the clean corpus; `None`
/// when no safe draft exists (prune). This is the surface that must never bypass B.
fn propose_and_promote(cluster: &[syn::Item], clean_corpus: &[syn::Item]) -> Option<Fingerprint> {
    antigen::learn::propose::propose(cluster, clean_corpus)
}

#[test]
fn c_promote_path_never_promotes_a_draft_that_binds_clean() {
    let items = fixture_items("autoimmunity_forced_prune");
    // The mixed-panic cluster: DefectiveOne via .unwrap(), DefectiveTwo via panic!.
    let cluster = vec![
        drop_impl_for(&items, "DefectiveOne"),
        drop_impl_for(&items, "DefectiveTwo"),
    ];
    let clean_corpus = vec![drop_impl_for(&items, "CleanSibling")];

    // SANITY (run-as-code premise): the EASY generalization "any Drop impl" DOES
    // bind the clean sibling — so a C that takes the easy path ships autoimmunity.
    let easy_autoimmune = naive_lgg_draft(); // all_of([item=impl, impl_of_trait("Drop")])
    assert!(
        easy_autoimmune.matches(&clean_corpus[0]),
        "premise: the easy 'any Drop impl' generalization binds the clean sibling — \
         taking it is the autoimmune trap C must avoid"
    );

    // THE GATE: whatever C's promote path yields, it must NEVER be a draft that
    // binds the clean sibling. A safe disjunction EXISTS (any_of([body_calls(unwrap),
    // body_contains_macro(panic)]) binds both defectives + spares clean), so a
    // strong C promotes that; a weaker C prunes (None). Both are safe. ONLY a
    // promoted draft that binds clean is the bypass-B failure.
    let promoted = propose_and_promote(&cluster, &clean_corpus);
    if let Some(draft) = &promoted {
        assert!(
            b_gate_spares_clean(draft, &clean_corpus),
            "C PROMOTED A DRAFT THAT BINDS THE CLEAN SIBLING — this is the \
             bypass-B failure: C produced and promoted an autoimmune draft without \
             B's gate rejecting it. THE ONE THING THAT MUST NOT PASS. draft = {draft:?}"
        );
        for (i, m) in cluster.iter().enumerate() {
            assert!(
                draft.matches(m),
                "a promoted draft must still bind its cluster member {i}"
            );
        }
    }
    // promoted == None (prune) OR a spares-clean Some: both safe. A binds-clean
    // Some is the only failure, asserted above.
}

// ===========================================================================
// (F) THE DECISIVE NO-BYPASS PROOF — `anti_unify`'s OWN output is autoimmune, and
//     `propose` MUST prune it. This is the strongest run-as-code refutation of the
//     bypass: a HOMOGENEOUS impl-Drop cluster (one C definitely anti-unifies)
//     whose disjunction `any_of([unwrap, expect])` BINDS the clean sibling
//     (because the clean Drop also calls `.expect()` on a safe value). So the raw
//     anti_unify draft is genuinely autoimmune — and `propose` (which routes
//     through B) must return None, proving B catches anti_unify's own autoimmune
//     output. If `propose` returned Some here, B was bypassed.
//
//     Verified live against the shipped C: raw draft binds clean=true, propose
//     returns None (pruned). This is the captain's one-thing-that-must-not-pass,
//     proven NOT to pass.
// ===========================================================================

#[test]
fn propose_prunes_when_anti_unifys_own_disjunction_binds_clean() {
    // Three impl-Drops, parsed bare so the cluster is HOMOGENEOUS (all Impl kind):
    // DefA panics via .unwrap(), DefB via .expect(); the CLEAN sibling ALSO calls
    // .expect() (on a safe value, no panic intent) — so the any_of([unwrap, expect])
    // disjunction anti_unify builds binds the clean sibling via the expect arm.
    let src = r#"
        impl Drop for DefA { fn drop(&mut self) { let _ = a().unwrap(); } }
        impl Drop for DefB { fn drop(&mut self) { let _ = b().expect("x"); } }
        impl Drop for Clean { fn drop(&mut self) { let _ = c().expect("fine"); } }
    "#;
    let impls: Vec<syn::Item> = syn::parse_file(src).expect("parses").items;
    let def_a = impls[0].clone();
    let def_b = impls[1].clone();
    let clean = impls[2].clone();
    let cluster = vec![def_a.clone(), def_b.clone()];
    let clean_corpus = vec![clean.clone()];

    // PREMISE: anti_unify produces a draft, and that raw draft is AUTOIMMUNE — it
    // binds the clean sibling (via the shared .expect() arm of the disjunction).
    let raw = antigen::learn::propose::anti_unify(&cluster)
        .expect("a homogeneous impl-Drop cluster anti-unifies to a draft");
    assert!(
        raw.matches(&def_a) && raw.matches(&def_b),
        "premise: the raw draft binds both defectives"
    );
    assert!(
        raw.matches(&clean),
        "premise: the raw anti_unify draft BINDS the clean sibling (the disjunction's \
         .expect() arm) — anti_unify's own output is autoimmune here. If this is \
         false, the test no longer exercises the bypass and must be re-armed."
    );

    // THE DECISIVE GATE: propose (which routes anti_unify's draft through B) MUST
    // NOT promote this autoimmune draft. It returns None — B caught anti_unify's
    // own over-binding output. A Some here = B was bypassed = the captain's
    // one-thing-that-must-not-pass.
    let promoted = propose_and_promote(&cluster, &clean_corpus);
    assert!(
        promoted.is_none(),
        "BYPASS-B FAILURE: anti_unify produced an autoimmune draft (binds clean), \
         and propose PROMOTED it instead of pruning — B was bypassed. propose MUST \
         return None here. got: {promoted:?}"
    );
}

// ===========================================================================
// (G) THE VACUOUS-CORPUS REFUSAL (the scout's "autoimmunity-with-a-green-check",
//     RATIFIED CLOSED by the captain's gate-G ruling). B's spare-clean PREDICATE
//     is vacuously true against an EMPTY clean corpus (nothing to reject) — so a
//     promote against an empty corpus would be autoimmunity WITH a green check: B
//     verified NOTHING. The guarantee "C never promotes without B green" is only
//     meaningful if B actually checked against real clean code.
//
//     RULING (captain, supersedes the prior "documented hazard" pin): the GATE
//     `promote_if_safe` REFUSES an empty corpus — "cannot certify safety against
//     nothing." The refusal is STRUCTURAL (at the gate, not per-caller), so every
//     promote path inherits the conservative-safe default. `propose(cluster, &[])`
//     therefore returns None. This test was written by the adversarial pinning the
//     pre-ruling behavior with an explicit flip-instruction ("if this is now None,
//     propose was hardened — update this gate to assert the prune"); the ruling
//     landed, so it now asserts the REFUSAL.
// ===========================================================================

#[test]
fn propose_refuses_an_empty_corpus_cannot_certify_against_nothing() {
    // An autoimmune cluster: both members are impl-Drop with .unwrap() — the
    // generalization would bind any unwrap-in-Drop, including clean ones.
    let src = r"
        impl Drop for DefA { fn drop(&mut self) { let _ = a().unwrap(); } }
        impl Drop for DefB { fn drop(&mut self) { let _ = b().unwrap(); } }
    ";
    let cluster: Vec<syn::Item> = syn::parse_file(src).expect("parses").items;

    // RATIFIED behavior (captain's gate-G ruling): with an EMPTY clean corpus, the
    // gate cannot certify safety against nothing → REFUSE. propose returns None.
    let promoted = propose_and_promote(&cluster, &[]);
    assert!(
        promoted.is_none(),
        "GATE-G RULING: propose MUST REFUSE an empty clean corpus (cannot certify \
         safety against nothing — a vacuous spare-clean is autoimmunity-with-a-green- \
         check). It must return None, never promote against emptiness. got: {promoted:?}"
    );

    // Proof the gate is REAL when fed a corpus the draft binds: feed propose a
    // clean sibling that IS one of the cluster's own shapes (so the draft binds
    // it). B then rejects → prune. This shows the refusal generalizes: B works the
    // moment it has something real to check against, and refuses when it does not.
    let with_self = propose_and_promote(&cluster, std::slice::from_ref(&cluster[0]));
    assert!(
        with_self.is_none(),
        "with a non-empty corpus containing an item the draft binds, propose PRUNES \
         (B works when fed a corpus). got: {with_self:?}"
    );
}
