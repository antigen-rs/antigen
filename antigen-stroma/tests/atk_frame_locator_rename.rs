//! ATK-FRAME-LOCATOR-RENAME (gap-closing, property class) — the Locator's value-identity is keyed on
//! `fq_path` so a RENAME produces a NEW Locator (delete+create), while a BODY edit (which does not
//! touch the locator) leaves it UNCHANGED. ADR-070 §4.5 / adversarial A2.
//!
//! ## Why this ATK exists (closing a newly-undefended BINDING invariant)
//! §4.5 NAMES "rename = delete-old-Locator + create-new" as CORRECT (so a builder doesn't mistake it
//! for a bug) — but the §8 seed-table left it UNGUARDED. The come-apart is real: the Locator must NOT
//! be rename-proof (entity-continuity across a rename is the lifecycle layer's job, ONE layer up), and
//! it must NOT churn on a body edit (the locator carries no body/digest — that's the Node's job). A
//! Locator that folded the body in would re-mint on every edit (destroying the stable-key property);
//! one that ignored the path would collide a renamed item with its old self (a false continuity).
//!
//! ## Class: property/structural (NOT born-red — `Locator` is a concrete frozen type, no `todo!()`)
//! These run from day one. They guard the value-equality CONTRACT the salsa interning depends on.
//! Note on salsa semantics: with `#[salsa::interned]`, value-equality is guaranteed by interning —
//! equal (`fq_path`, `cfg_set`) inputs ⇒ equal `LocatorId` output (salsa deduplicates within a db).
//! The tests construct two independent `Locator`s per scenario; salsa's intern table enforces the
//! structural-equality → Id-equality contract we're verifying.

use antigen_stroma::db::StromaDb;
use antigen_stroma::node::cfg::CfgSet;
use antigen_stroma::node::locator::Locator;

fn loc(db: &StromaDb, fq_path: &str) -> Locator {
    Locator::new(db, fq_path.to_string(), CfgSet::default())
}

// ATK: a RENAME (fq_path changes) produces a DISTINCT Locator — salsa will see delete+create.
#[test]
fn atk_frame_locator_rename_changes_identity() {
    let db = StromaDb::default();
    let before = loc(&db, "mycrate::foo::handler");
    let after_rename = loc(&db, "mycrate::foo::process"); // same module, renamed item

    assert_ne!(
        before, after_rename,
        "ATK-FRAME-LOCATOR-RENAME: a renamed item kept the SAME Locator value. The interned key would \
         then NOT delete+create — it would silently treat the rename as a body edit, claiming a false \
         entity-continuity the base must NOT assert (that is the lifecycle layer's job, §4.5)."
    );
}

// ATK: a MOVE (module path changes, item name same) also produces a DISTINCT Locator.
#[test]
fn atk_frame_locator_move_changes_identity() {
    let db = StromaDb::default();
    let before = loc(&db, "mycrate::foo::handler");
    let after_move = loc(&db, "mycrate::bar::handler"); // moved to another module

    assert_ne!(
        before, after_move,
        "ATK-FRAME-LOCATOR-RENAME: a moved item (different module path) kept the same Locator — the \
         module qualification is not load-bearing in the locator value (the bare-name collision again)."
    );
}

// NEGATIVE CONTROL (teeth): the SAME path+cfg is the SAME Locator — value-equality is stable, so an
// edit that does NOT change the path (a BODY edit) leaves the locator UNCHANGED. A Locator that folded
// the body/digest in would FAIL here (re-minting on every edit, destroying the stable-key property).
#[test]
fn nc_frame_locator_same_path_is_same_locator() {
    let db = StromaDb::default();
    let a = loc(&db, "mycrate::foo::handler");
    let b = loc(&db, "mycrate::foo::handler"); // same item, post body-edit (locator carries no body)

    assert_eq!(
        a, b,
        "NC: two Locators for the same (fq_path, cfg) compared UNEQUAL — the locator is not a pure \
         function of (path, cfg). Salsa interning would mint a new Id on every body edit, destroying \
         the stable-key/in-place-mutation property the whole identity split depends on (§4.5)."
    );
}

// NEGATIVE CONTROL (teeth, boundary): cfg is part of the key — two items at the same path under
// different cfg are DISTINCT Locators (the cfg-collision handling, §4.8 / §4.5 cfg-aware identity).
#[test]
fn nc_frame_locator_cfg_is_part_of_key() {
    use antigen_stroma::node::cfg::CfgAtom;
    let db = StromaDb::default();
    let unix = Locator::new(
        &db,
        "mycrate::foo::handler".to_string(),
        CfgSet(vec![CfgAtom("unix".to_string())]),
    );
    let windows = Locator::new(
        &db,
        "mycrate::foo::handler".to_string(),
        CfgSet(vec![CfgAtom("windows".to_string())]),
    );
    assert_ne!(
        unix, windows,
        "NC: two items at the same path under DIFFERENT cfg compared equal — cfg is not part of the \
         locator key, so a cfg(unix)/cfg(windows) pair would collide (the cfg-collision the frame closes)."
    );
}
