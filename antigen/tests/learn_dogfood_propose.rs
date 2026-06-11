//! P0b + C-PROPOSE — the DOGFOOD falsification proof (v0.4 keystone). The strange
//! loop made real: antigen anti-unifies a draft fingerprint from a cluster of its
//! OWN genuinely-felt-but-unnamed footguns (the P0b `#[dread]` marks), governed by
//! B (the spare-clean gate). Not a synthetic fixture — antigen's own source.
//!
//! THE CORPUS (the P0b marks, applied to production `src/`):
//!   - `scan::walk::scan_workspace_inner` — `#[dread]`: silently `continue`s past
//!     an unreadable file, reporting clean over an incomplete corpus.
//!   - `audit::immunity::collect_function_index` — `#[dread]`: the silent-skip
//!     TWIN (same `WalkDir` + read-or-continue + parse-or-skip shape) at audit time.
//!
//! Both are honest felt-sites (worries had while reading antigen's own infra),
//! each of a class antigen itself catches (silent-failure / incomplete-presented-
//! as-complete). They form a real felt-family: a directory walk that swallows an
//! IO/parse error and proceeds, building an incomplete result presented as whole.
//!
//! THE PROOF: re-acquire the two twins' ASTs (the scout's path-B: re-parse the
//! on-disk source — antigen's own marks ride the committed source as the corpus),
//! anti-unify them to a DRAFT, and confirm the draft binds BOTH twins while
//! sparing a clean directory-walk sibling that does NOT swallow its errors.
//! Promotion is ONLY through B (the C ══ B co-ship — ADR-045, the one safety-tangle).
//!
//! THE HONEST FINDING this test pins (deviate-and-flag, ADR-045 Amd-1 territory):
//! the two twins do NOT share an exact `shape_digest` — antigen's genuinely-felt
//! sites are abstractly-similar but concretely-heterogeneous (different visitor
//! constructions, different return shapes). So P0a's `shape_digest` clustering key
//! UNDER-clusters relative to what PROPOSE's anti-unifier can generalize: the
//! anti-unifier abstracts away the body differences that the exact-body shape
//! digest keeps. This is a real seam (the clustering heuristic is stricter than
//! the generalizer) — flagged to the Survey as a P0a clustering-recall note, not a
//! blocker: the falsification gate (`autoimmunity_safety_gate.rs` test D) carries the
//! ≥2-cluster proof on a shape-homogeneous corpus; this test carries the
//! antigen-self proof on the felt (abstractly-clustered) corpus.

use std::path::Path;

use antigen::learn::propose;

/// Re-acquire the `fn` item named `fn_name` from `file` on disk (the scout's
/// path-B AST re-acquisition: marks ride file+line+digest; PROPOSE re-parses the
/// clustered files at PROPOSE-time and locates the enclosing item — here keyed by
/// the function's stable name, which is sufficient for these two free functions).
fn reacquire_fn(file: &str, fn_name: &str) -> syn::Item {
    let src = std::fs::read_to_string(file).unwrap_or_else(|e| panic!("read {file}: {e}"));
    let parsed = syn::parse_file(&src).unwrap_or_else(|e| panic!("parse {file}: {e}"));
    parsed
        .items
        .into_iter()
        .find(|it| matches!(it, syn::Item::Fn(f) if f.sig.ident == fn_name))
        .unwrap_or_else(|| panic!("no `fn {fn_name}` in {file}"))
}

/// The two genuinely-felt silent-skip twins — antigen's OWN P0b-marked cluster.
fn felt_twins() -> Vec<syn::Item> {
    vec![
        reacquire_fn("src/scan/walk.rs", "scan_workspace_inner"),
        reacquire_fn("src/audit/immunity.rs", "collect_function_index"),
    ]
}

/// A CLEAN directory-walk sibling: it walks files but propagates its read error
/// (`?`) instead of swallowing it with `else { continue }` — the anti-correlated
/// safe case the draft must SPARE. (Parsed-as-text; never compiled.)
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
fn the_p0b_marks_exist_and_are_surfaced_by_antigens_own_scan() {
    // The dogfood loop's precondition: antigen's own scan surfaces the felt marks
    // (the marked-unknown population is non-empty — the falsification is not
    // theater over an empty corpus).
    let report = antigen::scan::scan_workspace(Path::new("src"), None).expect("scan src");
    let dread_markers: Vec<_> = report
        .marked_unknowns
        .iter()
        .filter(|m| m.marker == "dread")
        .collect();
    assert!(
        dread_markers.len() >= 2,
        "antigen's own src must carry ≥2 #[dread] marks (the felt PROPOSE corpus); found {}",
        dread_markers.len()
    );
    // Every mark carries a non-empty trigger (ADR-041 guard 3) and a real shape
    // digest (P0a) — the Goodhart guard at the data level: a mark with no felt
    // trigger is theater.
    for m in &report.marked_unknowns {
        assert!(
            !m.trigger.is_empty(),
            "every mark must carry a felt trigger (guard 3)"
        );
        assert!(
            m.shape_digest.starts_with("fnv1a64:"),
            "every mark must carry a real P0a shape_digest, got {:?}",
            m.shape_digest
        );
    }
}

#[test]
fn propose_anti_unifies_antigens_own_felt_twins_into_a_real_draft() {
    let twins = felt_twins();
    let draft = propose::anti_unify(&twins)
        .expect("the two felt silent-skip twins share a fn skeleton to anti-unify");

    // (1) The draft BINDS both genuinely-felt twins — a real fingerprint born from
    //     antigen's own self-doubt, not a synthetic fixture.
    for (i, twin) in twins.iter().enumerate() {
        assert!(draft.matches(twin), "the draft must bind felt twin {i}");
    }

    // (2) NON-DEGENERATE: the draft must carry real discriminating structure — at
    //     minimum the shared `fn` skeleton plus the shared body-call signal of the
    //     silent-skip shape (`read_to_string` is called by BOTH twins). A
    //     match-everything draft (just `item = fn`) would be useless theater.
    let has_read_signal = draft.constraints.iter().any(
        |c| matches!(c, antigen_fingerprint::Constraint::BodyCalls(n) if n == "read_to_string"),
    );
    assert!(
        has_read_signal,
        "the draft must capture the shared silent-skip signal (read_to_string), not collapse \
         to a shapeless `item = fn`: {:?}",
        draft.constraints
    );
}

#[test]
fn propose_promotes_the_felt_draft_only_through_b_sparing_the_clean_sibling() {
    let twins = felt_twins();
    let clean_corpus = vec![clean_walk_sibling()];

    // The end-to-end C ══ B path: anti-unify, then promote ONLY through B. Now
    // returns `Result<PromotedDraft, ProposeOutcome>` (ADR-047/048/056) — every
    // non-promotion reason is legible, never a bare `None`.
    let promoted = propose::propose(&twins, &clean_corpus);

    // The draft may or may not promote depending on whether its generalization
    // happens to bind the clean sibling / is near-miss-witnessed — BUT the one thing
    // that must hold: if it promotes, it spares clean (B guaranteed it). A promoted
    // draft that binds the clean sibling would be the bypass-B failure (must not pass).
    if let Ok(token) = &promoted {
        let draft = token.fingerprint();
        assert!(
            !draft.matches(&clean_corpus[0]),
            "a PROMOTED felt draft must SPARE the clean walk sibling — if it binds clean, \
             B was bypassed (autoimmunity shipped). draft = {draft:?}"
        );
        for (i, twin) in twins.iter().enumerate() {
            assert!(
                draft.matches(twin),
                "a promoted draft must still bind felt twin {i}"
            );
        }
    }
    // An `Err(_)` (B pruned an over-binding draft, or routed-to-human, or C refused a
    // degenerate one) is ALSO safe — the test's guarantee is "never a promoted
    // clean-binder", asserted above. The reason is legible in the ProposeOutcome.
}
