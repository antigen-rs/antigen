//! Read-engine integration tests (ADR-068) — the four point-wise queries over a 5-node cyclic graph,
//! plus the semiring closure engines they read from.
//!
//! The graph (a C–D–E cycle behind a resolved A→B→C spine):
//!
//! ```text
//!   A --Resolved--> B --Resolved--> C --Syntactic--> D --Syntactic--> E
//!                                   ^                                  |
//!                                   +--------------Syntactic-----------+
//! ```

use std::collections::HashSet;

use antigen_stroma::base::facts::{EdgeFact, EdgeFacts, EdgeKind};
use antigen_stroma::db::StromaDb;
use antigen_stroma::engine::{
    Blast, Conductance, Detection, ReachabilityRelation, condense, run_closure, run_condensed,
};
use antigen_stroma::node::StromaNodeId;
use antigen_stroma::node::cfg::CfgSet;
use antigen_stroma::node::digest::IdentityDigest;
use antigen_stroma::node::id::FqPath;
use antigen_stroma::read::query::{blast_from, field_at, provenance_of, reachable_from};
use antigen_stroma::read::{DetectionGrade, ResolutionTier, SnapshotHandle};

fn node(name: &str) -> StromaNodeId {
    StromaNodeId {
        fq_path: FqPath {
            path: name.to_string(),
            tier: ResolutionTier::Resolved,
        },
        identity_digest: IdentityDigest::of_tokens(name.as_bytes()),
        cfg_set: CfgSet::default(),
    }
}

fn edge(src: &str, dst: &str, tier: ResolutionTier) -> EdgeFact {
    EdgeFact {
        src: node(src),
        dst: node(dst),
        kind: EdgeKind::Call,
        tier,
    }
}

fn graph_edges() -> Vec<EdgeFact> {
    use ResolutionTier::{Resolved, Syntactic};
    vec![
        edge("A", "B", Resolved),
        edge("B", "C", Resolved),
        edge("C", "D", Syntactic),
        edge("D", "E", Syntactic),
        edge("E", "C", Syntactic), // closes the C–D–E cycle
    ]
}

fn build_db() -> StromaDb {
    let mut db = StromaDb::default();
    let edge_facts = EdgeFacts::new(&db, graph_edges());
    db.set_edge_facts(edge_facts);
    db
}

fn set_of(ids: &[StromaNodeId]) -> HashSet<String> {
    ids.iter().map(|id| id.fq_path.path.clone()).collect()
}

fn names(items: &[&str]) -> HashSet<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn reachable_from_d_is_the_backward_closure_at_syntactic_tier() {
    let db = build_db();
    let snap = SnapshotHandle::open(&db);
    let ans = reachable_from(&snap, &node("D"));

    // Everything with a path INTO D: A→B→C→D, plus the cycle E→C→D. Excludes D itself.
    let value = ans
        .read_at_least(DetectionGrade::Dread)
        .expect("a dread floor is always readable");
    assert_eq!(set_of(value), names(&["A", "B", "C", "E"]));

    // A path crosses the C→D syntactic edge, so the answer is tier-capped at Syntactic / Dread.
    assert_eq!(ans.tier(), ResolutionTier::Syntactic);
    assert_eq!(ans.grade(), DetectionGrade::Dread);
    assert!(
        ans.read_at_least(DetectionGrade::Presents).is_none(),
        "a syntactic-sourced answer must REFUSE a presents-grade read (tier-honesty, R6)"
    );
}

#[test]
fn provenance_of_b_earns_resolved_grade() {
    let db = build_db();
    let snap = SnapshotHandle::open(&db);
    let ans = provenance_of(&snap, &node("B"));

    // B's best forward evidence is the Resolved B→C edge → the max-tier lattice-JOIN is Resolved.
    assert_eq!(ans.tier(), ResolutionTier::Resolved);
    assert!(
        ans.read_at_least(DetectionGrade::Presents).is_some(),
        "a resolved-sourced answer must serve a presents-grade read"
    );
}

#[test]
fn blast_from_a_terminates_on_the_cycle_and_covers_downstream() {
    let db = build_db();
    let snap = SnapshotHandle::open(&db);
    // If SCC-condensation were not applied, the counting blast over the C–D–E cycle would diverge;
    // this call returning at all is the termination assertion (R3/R4).
    let ans = blast_from(&snap, &node("A"));

    let value = ans
        .read_at_least(DetectionGrade::Dread)
        .expect("a dread floor is always readable");
    assert_eq!(set_of(value), names(&["B", "C", "D", "E"]));
}

#[test]
fn field_at_a_is_the_forward_closure() {
    let db = build_db();
    let snap = SnapshotHandle::open(&db);
    let ans = field_at(&snap, &node("A"));
    let value = ans
        .read_at_least(DetectionGrade::Dread)
        .expect("a dread floor is always readable");
    assert_eq!(set_of(value), names(&["B", "C", "D", "E"]));
}

#[test]
fn a_syntactic_back_edge_does_not_demote_a_resolved_field() {
    use ResolutionTier::{Resolved, Syntactic};
    // A --Resolved--> B, with a B --Syntactic--> B self-loop. The self-loop is a cycle-closing artifact,
    // NOT a reaching path — so field_at(A) = {B} must stay Resolved / Presents, not be demoted to
    // Syntactic / Dread. Regression: the tier fold once ran before the visited-check, so a lower-tier
    // back-edge silently dragged the answer tier below its real reaching path (R6 tier-honesty).
    let mut db = StromaDb::default();
    let facts = EdgeFacts::new(
        &db,
        vec![edge("A", "B", Resolved), edge("B", "B", Syntactic)],
    );
    db.set_edge_facts(facts);
    let snap = SnapshotHandle::open(&db);
    let ans = field_at(&snap, &node("A"));

    assert_eq!(
        set_of(
            ans.read_at_least(DetectionGrade::Dread)
                .expect("floor readable")
        ),
        names(&["B"]),
    );
    assert_eq!(
        ans.tier(),
        ResolutionTier::Resolved,
        "the Resolved reaching edge sets the tier"
    );
    assert!(
        ans.read_at_least(DetectionGrade::Presents).is_some(),
        "a Resolved-only reaching path serves a presents read despite the Syntactic self-loop",
    );
}

#[test]
fn queries_on_an_unpopulated_base_are_empty_at_the_floor() {
    let db = StromaDb::default(); // no edge relation published
    let snap = SnapshotHandle::open(&db);
    let ans = reachable_from(&snap, &node("A"));
    assert!(
        ans.read_at_least(DetectionGrade::Dread)
            .expect("floor readable")
            .is_empty()
    );
    assert_eq!(ans.tier(), ResolutionTier::Syntactic);
}

// --- the semiring closure engines the queries read from ------------------------------------------

#[test]
fn detection_closure_is_boolean_reachability() {
    let rel = ReachabilityRelation::from_edges(&graph_edges());
    let matrix = run_closure::<Detection>(&rel);
    let a = rel.index_of(&node("A")).unwrap();
    let e = rel.index_of(&node("E")).unwrap();
    assert!(matrix[&(a, e)], "A reaches E");
    assert!(!matrix[&(e, a)], "E does not reach A (A is a source)");
    assert!(matrix[&(a, a)], "reflexive: A reaches A");
}

#[test]
fn conductance_closure_is_shortest_hop_count() {
    let rel = ReachabilityRelation::from_edges(&graph_edges());
    let matrix = run_closure::<Conductance>(&rel);
    let a = rel.index_of(&node("A")).unwrap();
    let b = rel.index_of(&node("B")).unwrap();
    let c = rel.index_of(&node("C")).unwrap();
    assert_eq!(matrix[&(a, b)], 1, "A→B is one hop");
    assert_eq!(matrix[&(a, c)], 2, "A→B→C is two hops");
}

#[test]
fn blast_counting_runs_only_on_a_condensed_dag() {
    use ResolutionTier::Resolved;
    // A diamond DAG: A→B, A→C, B→D, C→D → two distinct paths A⇝D (the non-idempotent count). Built
    // through the real pipeline — relation → condense() → proof → run_condensed. The proof is the ONLY
    // door and carries its DAG, so a raw cyclic graph cannot be smuggled into the counting closure.
    let rel = ReachabilityRelation::from_edges(&[
        edge("A", "B", Resolved),
        edge("A", "C", Resolved),
        edge("B", "D", Resolved),
        edge("C", "D", Resolved),
    ]);
    let cond = condense(&rel);
    let counts = run_condensed::<Blast>(cond.proof::<Blast>());
    let sa = cond.scc_of[rel.index_of(&node("A")).unwrap()];
    let sb = cond.scc_of[rel.index_of(&node("B")).unwrap()];
    let sd = cond.scc_of[rel.index_of(&node("D")).unwrap()];
    assert_eq!(counts[&(sa, sd)], 2, "two distinct DAG paths from A to D");
    assert_eq!(counts[&(sa, sb)], 1, "one path from A to B");
}

#[test]
fn condensation_collapses_the_cycle_into_one_super_node() {
    let rel = ReachabilityRelation::from_edges(&graph_edges());
    let cond = condense(&rel);
    let c = rel.index_of(&node("C")).unwrap();
    let d = rel.index_of(&node("D")).unwrap();
    let e = rel.index_of(&node("E")).unwrap();
    let a = rel.index_of(&node("A")).unwrap();
    assert_eq!(cond.scc_of[c], cond.scc_of[d], "C and D share an SCC");
    assert_eq!(cond.scc_of[d], cond.scc_of[e], "D and E share an SCC");
    assert_ne!(cond.scc_of[a], cond.scc_of[c], "A is its own SCC");
}
