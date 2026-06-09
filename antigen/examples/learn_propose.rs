//! The learning core — PROPOSE a fingerprint from a cluster, promote it through
//! the self-tolerance gate.
//!
//! When you have several sites that feel like the same unnamed footgun, the
//! learning core turns that cluster into a candidate fingerprint instead of
//! leaving you to hand-author one. Two pieces do the work:
//!
//! - **PROPOSE** ([`antigen::learn::propose::anti_unify`]) anti-unifies a cluster
//!   of structurally-similar items into a draft fingerprint. It generalizes *to
//!   disjunction*: the signals every member shares become required conjuncts; the
//!   signals that distinguish members become an `any_of`. The naive
//!   generalization — drop the differing leaves — would collapse the cluster to
//!   its bare skeleton ("any `Drop` impl") and match clean code too.
//! - **The self-tolerance gate** ([`antigen::learn::self_tolerance::promote_if_safe`])
//!   is the selector. A draft is promotable only if it spares every item in a
//!   *clean corpus* — the known-good siblings the draft must not flag. A draft
//!   that binds clean code is rejected; promoting it would flag that clean code.
//!
//! The two co-ship: [`antigen::learn::propose::propose`] is the only path to a
//! promotable fingerprint, and it routes every draft through the gate. This
//! example walks one cluster through both halves and shows the draft binding the
//! defects while sparing the clean sibling.
//!
//! Run:
//!
//! ```sh
//! cargo run --example learn_propose --package antigen
//! ```
//!
//! ## What PROPOSE proves, and what it doesn't
//!
//! A promoted draft is proven to bind every member it was generalized from and to
//! spare every item in the clean corpus you supplied. That is a syntactic fact,
//! decidable against those items. It is **not** a claim that the draft names a
//! real failure-class, that it generalizes correctly to all code, or that the
//! cluster is a true failure-family. A draft is a suggestion to ratify, never an
//! asserted class — a human or an incident promotes it to a named antigen. The
//! gate is corpus-bounded: a richer clean corpus is a stronger gate, not a total
//! guarantee that the draft spares all clean code everywhere.

use antigen::learn::{propose, self_tolerance};
use antigen_fingerprint::Constraint;

/// A toy cluster: two `Drop` impls that both reach a panic in teardown — one via
/// `.unwrap()`, one via `.expect()` — plus a clean `Drop` sibling that does
/// neither. The two defects share the cleanup call `take`; they differ in how
/// they panic. The clean sibling shares the skeleton (`impl Drop`, `take`) but
/// reaches no panic source.
const DROP_FAMILY: &str = r#"
    pub struct GuardA;
    impl Drop for GuardA {
        fn drop(&mut self) { let _ = flush(self.h).take().unwrap(); }
    }

    pub struct GuardB;
    impl Drop for GuardB {
        fn drop(&mut self) { let _ = flush(self.h).take().expect("must flush"); }
    }

    pub struct CleanGuard;
    impl Drop for CleanGuard {
        fn drop(&mut self) { let _ = flush(self.h).take().ok(); }
    }
"#;

/// Pull the `impl Drop for <ty>` item out of the parsed family.
fn drop_impl_for(items: &[syn::Item], ty: &str) -> syn::Item {
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
            let syn::Type::Path(p) = &*i.self_ty else {
                return false;
            };
            is_drop && p.path.segments.last().is_some_and(|s| s.ident == ty)
        })
        .expect("impl Drop for the named type is present")
        .clone()
}

/// Describe a constraint in plain terms, for the printed walkthrough.
fn describe(c: &Constraint) -> String {
    match c {
        Constraint::Item(kind) => format!("item = {kind:?}"),
        Constraint::ImplOfTrait(t) => format!("impl_of_trait(\"{t}\")"),
        Constraint::BodyCalls(n) => format!("body_calls(\"{n}\")"),
        Constraint::BodyContainsMacro(n) => format!("body_contains_macro(\"{n}\")"),
        Constraint::AnyOf(arms) => {
            let inner: Vec<String> = arms.iter().map(describe).collect();
            format!("any_of([{}])", inner.join(", "))
        },
        other => format!("{other:?}"),
    }
}

fn main() {
    let family = syn::parse_file(DROP_FAMILY).expect("the toy family parses");
    let cluster = vec![
        drop_impl_for(&family.items, "GuardA"),
        drop_impl_for(&family.items, "GuardB"),
    ];
    let clean_sibling = drop_impl_for(&family.items, "CleanGuard");

    println!("== PROPOSE: anti-unify a cluster of two panic-in-Drop sites ==\n");
    println!("cluster:");
    println!("  GuardA::drop — flush(..).take().unwrap()");
    println!("  GuardB::drop — flush(..).take().expect(\"must flush\")");
    println!("clean sibling (must be spared):");
    println!("  CleanGuard::drop — flush(..).take().ok()\n");

    // --- Step 1: the raw draft (a hypothesis; NOT yet gate-checked) -----------
    let draft = propose::anti_unify(&cluster).expect("the cluster shares a skeleton to generalize");

    println!("drafted fingerprint (the anti-unification):");
    for c in &draft.constraints {
        println!("  {}", describe(c));
    }
    println!();
    println!("  the signals both members share (`flush`, `take`) are required conjuncts.");
    println!("  the differing panic sources `unwrap`/`expect` become an any_of —");
    println!("  the load-bearing wall that carries the defect signal without");
    println!("  collapsing to the bare `impl Drop` skeleton clean code also has.\n");

    // --- Step 2: the draft already spares the clean sibling -------------------
    let binds_a = draft.matches(&cluster[0]);
    let binds_b = draft.matches(&cluster[1]);
    let binds_clean = draft.matches(&clean_sibling);
    println!("does the raw draft bind each member?");
    println!("  GuardA     -> {binds_a}");
    println!("  GuardB     -> {binds_b}");
    println!("  CleanGuard -> {binds_clean}   (the any_of arm is no-match on .ok())\n");

    // --- Step 3: promote through the self-tolerance gate ----------------------
    // The clean corpus is the cluster's known-good sibling. The gate refuses an
    // empty corpus, so we supply the real clean sibling to check against.
    let clean_corpus = vec![clean_sibling.clone()];
    let promoted = propose::propose(&cluster, &clean_corpus);

    if let Some(fp) = &promoted {
        println!("== self-tolerance gate: PROMOTED ==\n");
        println!("the draft spared the clean corpus, so the gate moved it through.");
        println!("a promoted draft is guaranteed to spare every clean-corpus item:");
        println!("  binds GuardA     -> {}", fp.matches(&cluster[0]));
        println!("  binds GuardB     -> {}", fp.matches(&cluster[1]));
        println!("  binds CleanGuard -> {}", fp.matches(&clean_corpus[0]));
    } else {
        // Unreachable for this spare-clean cluster, but handled honestly: a None
        // means the gate refused (the draft bound a clean item, or the corpus was
        // empty). The caller must treat None as "no safe draft" — never fall back
        // to the raw anti_unify output (that bypasses the gate).
        println!("== self-tolerance gate: REJECTED ==\n");
        println!("the gate refused to promote (the draft bound a clean item).");
    }

    // --- Step 4: the gate is what catches the naive over-generalization -------
    // Show the rejection directly: the bare `impl Drop` skeleton (what the naive
    // drop-the-leaves generalization would produce) binds CleanGuard, so the gate
    // rejects it. This is why the generator must never promote without the gate.
    println!("\n== why the gate is not optional ==\n");
    let naive =
        antigen_fingerprint::Fingerprint::parse(r#"all_of([item = impl, impl_of_trait("Drop")])"#)
            .expect("the naive draft parses");
    println!("the naive generalization (drop the differing leaves) is just:");
    println!("  all_of([item = impl, impl_of_trait(\"Drop\")])");
    println!(
        "  binds CleanGuard -> {}   (it flags clean code)",
        naive.matches(&clean_sibling)
    );
    let naive_promoted = self_tolerance::promote_if_safe(naive, &clean_corpus);
    println!(
        "  promote_if_safe(naive) -> {:?}   (the gate rejects it)",
        naive_promoted.map(|_| "Some(..)")
    );
}
