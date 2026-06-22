//! FEASIBILITY-WITNESS — GATE-G is SATISFIABLE, and the dogfood loop closes
//! end-to-end on antigen's OWN code (v0.5 build-order unit 2, the witness that the
//! safety spine isn't a gate nothing can pass).
//!
//! The safety spine (ADR-047 GATE-G + ADR-048 `PromotedDraft` + ADR-056 C-guard)
//! makes promotion HARD on purpose — a draft must carry a discriminating signal
//! (the `(A)`-binary check), be near-miss-witnessed by the corpus (non-vacuity), AND spare every
//! clean item. A gate that nothing can pass is as useless as no gate. This witness
//! proves the gate is SATISFIABLE, in two honest halves:
//!
//! - **(3a) satisfiable-in-principle** — a constructed isomorphic defect cluster +
//!   a near-miss clean sibling PROMOTES (`Ok(PromotedDraft)`). The gate CAN mint a
//!   token; the spine is not a brick wall.
//! - **(3b) the dogfood loop closes on antigen's OWN real `#[dread]` marks** — the
//!   two genuinely-felt silent-skip twins (`scan_workspace_inner` +
//!   `collect_function_index`) anti-unify to a real draft, and the gate returns a
//!   LEGIBLE 3-valued verdict end-to-end. On the *current* thin dogfood corpus that
//!   verdict is **`NotCorpusWitnessable` (route-to-human)** — and that is the gate
//!   being HONEST, not failing: two genuine twins share everything (no discriminating
//!   `any_of`), and the lone clean sibling is structurally far (not one-constraint-
//!   away), so the gate refuses to certify a generalization it cannot witness. The
//!   *plumbing* (anti-unify → gate → legible outcome) works on real dogfood; the
//!   *promote payoff* on real marks is a frontier that needs the afferent fix (more
//!   source-distinct clustered marks, or a richer near-miss-providing corpus) — the
//!   3a-now / 3b-frontier distinction, ground-truthed by RUN (a reasoned "it should
//!   promote" is a claim, only a run settles a verdict).
//!
//! Together: the gate is satisfiable (3a), the loop is real on antigen-on-antigen
//! (3b), and the gate is sound — it never autoimmune-promotes; when it cannot
//! witness a generalization it routes-to-human rather than guess.

use std::path::Path;

use antigen::learn::propose::{self, ProposeOutcome};
use antigen::learn::self_tolerance::ToleranceVerdict;
use antigen_fingerprint::Constraint;

// ===========================================================================
// (3a) SATISFIABLE-IN-PRINCIPLE — the gate CAN mint a token.
// ===========================================================================

fn items(src: &str) -> Vec<syn::Item> {
    syn::parse_file(src).expect("parses").items
}

fn impl_for(items: &[syn::Item], ty: &str) -> syn::Item {
    items
        .iter()
        .find(|it| match it {
            syn::Item::Impl(i) => match &*i.self_ty {
                syn::Type::Path(p) => p.path.segments.last().is_some_and(|s| s.ident == ty),
                _ => false,
            },
            _ => false,
        })
        .unwrap_or_else(|| panic!("impl for {ty} not found"))
        .clone()
}

/// A constructed isomorphic defect family with a DISCRIMINATING difference (one
/// panics via `.unwrap()`, the other via `.expect()`) + a clean sibling that is
/// one-constraint-away (it shares the skeleton, calls neither panic shape) — the
/// near-miss the gate needs. This is the canonical satisfying instance.
const ISOMORPHIC_FAMILY: &str = r#"
    impl Drop for GuardA { fn drop(&mut self) { let _ = flush().take().unwrap(); } }
    impl Drop for GuardB { fn drop(&mut self) { let _ = flush().take().expect("m"); } }
    impl Drop for CleanGuard { fn drop(&mut self) { let _ = flush().take().ok(); } }
"#;

#[test]
fn gate_g_is_satisfiable_a_real_cluster_promotes() {
    let fam = items(ISOMORPHIC_FAMILY);
    let cluster = vec![impl_for(&fam, "GuardA"), impl_for(&fam, "GuardB")];
    let clean_corpus = vec![impl_for(&fam, "CleanGuard")];

    let promoted = propose::propose(&cluster, &clean_corpus).expect(
        "GATE-G MUST be satisfiable: a discriminating cluster + near-miss clean sibling promotes",
    );

    // The minted token wraps a draft that binds the cluster and spares clean.
    let fp = promoted.fingerprint();
    for (i, m) in cluster.iter().enumerate() {
        assert!(
            fp.matches(m),
            "the promoted draft must bind cluster member {i}"
        );
    }
    assert!(
        !fp.matches(&clean_corpus[0]),
        "the promoted draft must spare the clean sibling (it came through B)"
    );
    // The draft carries the discriminating any_of (the reason the gate could
    // witness a near-miss on the clean sibling — it's one any_of-arm from binding).
    assert!(
        fp.constraints
            .iter()
            .any(|c| matches!(c, Constraint::AnyOf(_))),
        "the satisfying draft carries a discriminating any_of: {:?}",
        fp.constraints
    );
}

// ===========================================================================
// (3b) THE DOGFOOD LOOP CLOSES ON ANTIGEN'S OWN #[dread] MARKS — end-to-end,
//      with an HONEST route-to-human verdict on the current thin corpus.
// ===========================================================================

/// Re-acquire a free `fn` by name from antigen's own on-disk source (the dogfood
/// corpus rides the committed source — the marks ride file+line+digest, PROPOSE
/// re-parses at propose-time).
fn reacquire_fn(file: &str, fn_name: &str) -> syn::Item {
    let src = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file}: {e}"));
    let parsed = syn::parse_file(&src).unwrap_or_else(|e| panic!("parse {file}: {e}"));
    parsed
        .items
        .into_iter()
        .find(|it| matches!(it, syn::Item::Fn(f) if f.sig.ident == fn_name))
        .unwrap_or_else(|| panic!("no `fn {fn_name}` in {file}"))
}

/// Antigen's OWN two genuinely-felt silent-skip twins (the production `#[dread]`
/// cluster): `scan_workspace_inner` (in `scan/walk.rs`) + `collect_function_index`
/// (in `audit/immunity.rs`). Same `WalkDir` + read-or-continue + parse-or-skip shape.
fn felt_twins() -> Vec<syn::Item> {
    vec![
        reacquire_fn("src/scan/walk.rs", "scan_workspace_inner"),
        reacquire_fn("src/audit/immunity.rs", "collect_function_index"),
    ]
}

/// A clean directory-walk sibling that PROPAGATES its read error (`?`) instead of
/// swallowing it — the anti-correlated safe case (parsed-as-text, never compiled).
fn clean_walk_sibling() -> syn::Item {
    syn::parse_str(
        "fn strict_walk(root: &Path) -> std::io::Result<Vec<String>> { \
             let mut out = Vec::new(); \
             for entry in WalkDir::new(root) { \
                 let entry = entry?; \
                 let content = std::fs::read_to_string(entry.path())?; \
                 out.push(content); \
             } \
             Ok(out) \
         }",
    )
    .expect("clean sibling parses")
}

#[test]
fn the_dogfood_twins_cluster_into_a_real_multi_conjunct_draft() {
    // The PLUMBING half: antigen's own felt twins anti-unify into a real draft that
    // binds BOTH twins and carries ≥2 conjuncts (the shared silent-skip body
    // signals) — a real fingerprint born from antigen's own self-doubt, not theater.
    let twins = felt_twins();
    let draft = propose::anti_unify(&twins).expect("the felt twins share a skeleton to anti-unify");
    assert!(
        draft.constraints.len() >= 2,
        "the twins draft must carry ≥2 conjuncts (the GATE-G near-miss floor); got {:?}",
        draft.constraints
    );
    for (i, twin) in twins.iter().enumerate() {
        assert!(draft.matches(twin), "the draft must bind felt twin {i}");
    }
    // The two are GENUINE twins (same error-discipline) → no discriminating any_of.
    // This is WHY the gate routes-to-human below (see the next test): a draft with no
    // discriminating split, witnessed only by a structurally-distant clean sibling,
    // cannot show a near-miss.
    assert!(
        !draft
            .constraints
            .iter()
            .any(|c| matches!(c, Constraint::AnyOf(_))),
        "two genuine twins share everything → no discriminating any_of (the route-to-human cause)"
    );
}

#[test]
fn the_dogfood_loop_closes_end_to_end_with_an_honest_route_to_human() {
    // THE THESIS PLUMBING, end-to-end on antigen's OWN dread-marked code:
    // anti-unify → GATE-G → a LEGIBLE 3-valued verdict. On the current thin corpus
    // (one structurally-distant clean sibling, twins with no discriminating split)
    // the honest verdict is NotCorpusWitnessable (route-to-human) — the gate refusing
    // to certify a generalization it cannot witness, NOT an autoimmune promote and
    // NOT a silent failure. This is the gate being SOUND on real dogfood.
    let twins = felt_twins();
    let clean = vec![clean_walk_sibling()];

    let outcome = propose::propose(&twins, &clean);
    assert_eq!(
        outcome,
        Err(ProposeOutcome::Rejected(
            ToleranceVerdict::NotCorpusWitnessable
        )),
        "on the current thin dogfood corpus the gate routes the genuine-twins draft to \
         a human (no near-miss: the twins have no discriminating split and the lone \
         clean sibling is structurally far, not one-constraint-away). This is the gate \
         being HONEST — it never autoimmune-promotes and never fakes a generalization \
         verdict it cannot make. (The promote-payoff on real marks is the afferent \
         frontier: more source-distinct clustered marks, or a near-miss-providing \
         corpus — the 3a-now/3b-frontier distinction, ground-truthed by RUN.)"
    );
    // The ONE invariant that must hold whatever the verdict: the gate NEVER promotes
    // a draft that binds the clean sibling (autoimmunity). Here it didn't promote at
    // all — but assert the safety floor explicitly for the regression guard.
    if let Ok(token) = &outcome {
        assert!(
            !token.fingerprint().matches(&clean[0]),
            "a promoted dogfood draft must SPARE the clean sibling (B guaranteed it)"
        );
    }
}

#[test]
fn the_dogfood_marks_are_real_and_surfaced_by_antigens_own_scan() {
    // The corpus is not theater: antigen's own scan surfaces ≥2 #[dread] marks (the
    // felt PROPOSE corpus is non-empty). The feasibility witness rests on a real
    // marked-unknown population, not an empty one.
    let report = antigen::scan::scan_workspace(Path::new("src"), None).expect("scan src");
    let dread_marks = report
        .marked_unknowns
        .iter()
        .filter(|m| m.marker == "dread")
        .count();
    assert!(
        dread_marks >= 2,
        "antigen's own src must carry ≥2 #[dread] marks (the felt feasibility corpus); found {dread_marks}"
    );
}
