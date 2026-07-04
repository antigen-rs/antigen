//! DOGFOOD HONESTY GUARD — assert the CURRENT honest outcome of the antigen-on-
//! antigen dogfood, so a flip is NOTICED (captain's batch-2 routing, 2026-06-11).
//!
//! # The drift this closes (a NAME-asserted-property gap)
//!
//! `learn_dogfood_propose.rs::propose_promotes_the_felt_draft_only_through_b…` is
//! shaped `if let Ok(token) = promoted { assert spare-clean } /* Err arm asserts
//! NOTHING */`. Its NAME says "promotes," but it stays GREEN whether the dogfood
//! promotes OR routes-to-human — so a regression that made the gate route EVERYTHING
//! to human (the maximally-conservative failure) would not be noticed, and a future
//! fix that makes the payoff FIRE (a real self-immunizing promote) would also not be
//! noticed. The test cannot go red on the thing its name claims.
//!
//! # The empirically-settled truth this pins (captain "THESIS SETTLED", 2026-06-11)
//!
//! antigen's two real read-loop `#[dread]` twins anti-unify into a draft of ~21
//! ALL-SHARED conjuncts with NO `any_of`. A near-miss needs a clean item one
//! conjunct from binding (≈20-of-21 matching); read-loop-FREE fns match FEW of the
//! 21, so NO near-miss exists at ANY corpus size — and the only 2 files carrying the
//! read-loop shape are BOTH in the cluster. So `propose(real twins, clean)` =
//! **`Err(NotCorpusWitnessable)` — route-to-human, NOT promote.** The loop
//! demonstrates the PLUMBING (anti-unify → gate → legible outcome) but NOT the
//! PAYOFF (a promoted self-immunizing fingerprint). v0.5 ships HONEST: "antigen
//! anti-unifies a draft from its own marks + routes it to human ratification"; the
//! self-immunizing promote is CHARTERED (v0.6, needs abstract-recall).
//!
//! # Why this guard is a TRIPWIRE, not a permanent verdict
//!
//! It asserts the CURRENT honest outcome (route-to-human). If a future change makes
//! the payoff fire (the dogfood PROMOTES), THIS TEST GOES RED — a deliberate signal
//! to (1) celebrate the strange-loop closing AND (2) update the docs / CLAUDE.md /
//! briefing that currently say "routes to human ratification" (the captain's claim-
//! calibration). A red here is GOOD NEWS that demands a doc update — the comment
//! says so, so whoever trips it knows it is not a regression.

use antigen::learn::propose::{self, ProposeOutcome};
use antigen::learn::self_tolerance::ToleranceVerdict;

/// Re-acquire the `fn` item named `fn_name` from `file` on disk (antigen's own
/// marks ride the committed source — the same path-B re-acquisition the dogfood
/// test uses; duplicated here to keep this guard independent of that file).
fn reacquire_fn(file: &str, fn_name: &str) -> syn::Item {
    let src = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file}: {e}"));
    let parsed = syn::parse_file(&src).unwrap_or_else(|e| panic!("parse {file}: {e}"));
    parsed
        .items
        .into_iter()
        .find(|it| matches!(it, syn::Item::Fn(f) if f.sig.ident == fn_name))
        .unwrap_or_else(|| panic!("no `fn {fn_name}` in {file}"))
}

/// antigen's two genuinely-felt silent-skip twins — its OWN `#[dread]`-marked cluster.
fn felt_twins() -> Vec<syn::Item> {
    vec![
        reacquire_fn("src/scan/walk.rs", "scan_workspace_inner"),
        reacquire_fn("src/audit/immunity.rs", "collect_function_index"),
    ]
}

/// A clean directory-walk sibling that propagates its read error (`?`) instead of
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

/// THE TRIPWIRE — assert the dogfood's CURRENT honest outcome is route-to-human.
///
/// A RED here is GOOD NEWS: the payoff fired (a real self-immunizing promote). When
/// it does, do NOT "fix" this test to stay green — CELEBRATE, then update the docs /
/// CLAUDE.md / briefing that say "routes to human ratification" (the claim is then
/// stale), and re-home this guard to assert the new promote outcome.
#[test]
fn dogfood_currently_routes_to_human_not_promotes_the_payoff_is_chartered() {
    let twins = felt_twins();
    let clean = vec![clean_walk_sibling()];

    let outcome = propose::propose(&twins, &clean);

    assert_eq!(
        outcome,
        Err(ProposeOutcome::Rejected(
            ToleranceVerdict::NotCorpusWitnessable
        )),
        "DOGFOOD CLAIM-CALIBRATION (captain, THESIS SETTLED): antigen's real read-loop \
         twins anti-unify to a ~21-all-shared-conjunct draft with NO any_of, so NO \
         near-miss exists at any corpus size → the gate ROUTES TO HUMAN, it does not \
         promote. The PAYOFF (a self-immunizing promote) is CHARTERED (v0.6, needs \
         abstract-recall). \n\n\
         IF THIS WENT RED: the payoff FIRED — that is GOOD NEWS, not a regression. \
         Do NOT silence this test. Celebrate the strange-loop closing, then update the \
         docs / CLAUDE.md / briefing that say 'routes to human ratification' (now \
         stale) and re-home this guard to the new promote outcome. \n\n\
         got: {outcome:?}"
    );
}
