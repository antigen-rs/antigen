//! DOC-TRANSCRIPT — the learning-core teach-arc, written AS a test so the docs can
//! never show output the binary doesn't produce.
//!
//! # Why this test-class exists (the gap it closes)
//!
//! `antigen/examples/learn_propose.rs` is a beautiful *demonstration* — but it
//! `println!`s without asserting, and CI's `cargo test --workspace --all-targets`
//! only BUILDS examples (it never runs an example's `main`). So the example's
//! teaching output is **unverified**: a regression that changed a verdict would
//! compile clean and the example would silently teach a lie. This test-class closes
//! that — it exercises the SAME library teach-arc the example + the docs show and
//! **asserts every claimed output**. One artifact per claim = the test = the example
//! = the doc: when the teach-wave inherits these, every line was gate-verified.
//!
//! Each test below is a copy-paste-ready transcript for the docs-wright: the code is
//! the example, the asserted values are the doc's "you'll see ..." lines. If a doc
//! claims a verdict, that verdict is asserted here against the real gate.

use antigen::finding::Provenance;
use antigen::learn::propose::{ProposeOutcome, propose};
use antigen::learn::self_tolerance::{ToleranceVerdict, promote_if_safe};
use antigen_fingerprint::Fingerprint;

/// The canonical panic-in-Drop family the docs teach with: two defects
/// (`unwrap`/`expect`) sharing a `take()` cleanup, and a clean `.ok()` sibling the
/// generalization must spare. Parsed-as-text (never compiled) — the same shape the
/// scanner reads.
const fn drop_family_src() -> &'static str {
    r#"
        pub struct GuardA;
        impl Drop for GuardA { fn drop(&mut self) { let _ = flush(self.h).take().unwrap(); } }
        pub struct GuardB;
        impl Drop for GuardB { fn drop(&mut self) { let _ = flush(self.h).take().expect("must"); } }
        pub struct CleanGuard;
        impl Drop for CleanGuard { fn drop(&mut self) { let _ = flush(self.h).take().ok(); } }
    "#
}

fn drop_impl_for(items: &[syn::Item], ty: &str) -> syn::Item {
    items
        .iter()
        .find(|it| {
            let syn::Item::Impl(i) = it else { return false };
            let Some((_, tp, _)) = &i.trait_ else {
                return false;
            };
            let is_drop = tp.segments.last().is_some_and(|s| s.ident == "Drop");
            let syn::Type::Path(p) = &*i.self_ty else {
                return false;
            };
            is_drop && p.path.segments.last().is_some_and(|s| s.ident == ty)
        })
        .expect("found")
        .clone()
}

/// TRANSCRIPT 1 — the happy path: a cluster of two defects promotes through the
/// gate, minting a `PromotedDraft` that binds both defects and spares the clean
/// sibling. (The docs' "PROMOTED" block.)
#[test]
fn transcript_cluster_promotes_and_spares_clean() {
    let items = syn::parse_file(drop_family_src()).unwrap().items;
    let cluster = vec![
        drop_impl_for(&items, "GuardA"),
        drop_impl_for(&items, "GuardB"),
    ];
    let clean_corpus = vec![drop_impl_for(&items, "CleanGuard")];

    // propose = anti_unify + promote-through-B in one call (the only path to a token).
    let token = propose(&cluster, &clean_corpus)
        .expect("a spare-clean near-miss-witnessed cluster promotes");
    let fp = token.fingerprint();

    // DOC LINE: "the promoted draft binds both defects" —
    assert!(fp.matches(&cluster[0]), "binds GuardA");
    assert!(fp.matches(&cluster[1]), "binds GuardB");
    // DOC LINE: "...and spares the clean sibling (it came through B)" —
    assert!(!fp.matches(&clean_corpus[0]), "spares CleanGuard");
    // DOC LINE: "score tier -> Imagined (the gate-assigned floor; ADR-050 routing
    // wires the real tier later)" — pin the CURRENT honest floor so a future
    // ADR-050 change is a VISIBLE, intentional diff (a build-seam tripwire).
    assert_eq!(token.tier(), Provenance::DEFAULT);
    assert_eq!(Provenance::DEFAULT, Provenance::Imagined);
}

/// TRANSCRIPT 2 — why the gate is not optional: the naive generalization (drop the
/// differing leaves) is bare-structural, binds clean code, and the gate REFUSES it.
/// (The docs' "why the gate is not optional" block — the autoimmunity B prevents.)
#[test]
fn transcript_naive_generalization_is_refused_by_the_gate() {
    let items = syn::parse_file(drop_family_src()).unwrap().items;
    let clean = vec![drop_impl_for(&items, "CleanGuard")];

    // The naive draft the docs warn against — "any Drop impl", no discriminating signal.
    let naive = Fingerprint::parse(r#"all_of([item = impl, impl_of_trait("Drop")])"#).unwrap();

    // DOC LINE: "binds CleanGuard -> true (it flags clean code)" —
    assert!(
        naive.matches(&clean[0]),
        "the naive draft binds the clean sibling"
    );
    // DOC LINE: "promote_if_safe(naive) -> Err(BindsCleanItem { clean_index: None })
    // (the gate rejects it: bare-structural over-general fails the (A)-binary check)" —
    assert_eq!(
        promote_if_safe(naive, &clean),
        Err(ToleranceVerdict::BindsCleanItem { clean_index: None }),
    );
}

/// TRANSCRIPT 3 — the legible non-promotion reasons: a degenerate cluster (sharing
/// only its structural shape) is refused AT THE GENERATOR with a generator-
/// appropriate diagnostic, never a bare `None`. (The docs' "every non-promotion is
/// legible" line — ADR-048/056.)
#[test]
fn transcript_degenerate_cluster_is_refused_legibly_at_the_generator() {
    // Two Drop impls whose bodies share NO call/macro signal → the anti-unifier
    // collapses to bare-structural; C refuses it as Degenerate (not handed to B).
    let items = syn::parse_file(
        r"
            impl Drop for A { fn drop(&mut self) { self.a = 1; } }
            impl Drop for B { fn drop(&mut self) { self.b = 2; } }
        ",
    )
    .unwrap()
    .items;
    let cluster = vec![drop_impl_for(&items, "A"), drop_impl_for(&items, "B")];
    let clean = syn::parse_file("impl Drop for Clean { fn drop(&mut self) { log(); } }")
        .unwrap()
        .items;

    // DOC LINE: "propose(degenerate_cluster) -> Err(Degenerate) (these sites share
    // only their structural shape — not a real failure-family)" —
    assert_eq!(propose(&cluster, &clean), Err(ProposeOutcome::Degenerate));
}
