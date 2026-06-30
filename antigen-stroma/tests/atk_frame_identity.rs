//! ATK-FRAME-IDENTITY — the qualified-path construction closes the bare-name defect.
//!
//! ## The claim this defends (ADR-070 §4.1/§4.2, the single highest-leverage invariant)
//! The frame node is KEYED by a CONSTRUCTED qualified-path, never by the bare name. The existing
//! `ItemTarget` stores bare names (`Struct(String)`, `Fn(String)`) and `diff.rs:101`'s
//! `item_digest_map` keys on the bare name with last-write-wins ACROSS files — `foo::bar` and
//! `baz::bar` collide. The frame MUST construct `crate::module::item` so the two are DISTINCT nodes.
//!
//! ## Born-red status (the repo idiom)
//! `#[ignore]` = pre-implementation contract. `syntactic_fq_path` is `todo!()` in the skeleton, so
//! the ATK body panics (RED) until the builder fills it. WHEN THE FILL LANDS: remove `#[ignore]`,
//! confirm the test was RED against the stub, confirm GREEN against the fill, leave it as a forever
//! regression guard. NEVER delete a passing ATK.
//!
//! ## Teeth (the negative control)
//! `nc_*` proves the test rejects ONLY the cross-module case: two genuinely-identical items in the
//! SAME module MUST collide (same identity). A test that distinguished them would be over-fit (it
//! would flag a real duplicate as distinct); a test that collided across modules would be the defect
//! itself. The NC pins the boundary.

use antigen_stroma::node::path::syntactic_fq_path;

// ATK-FRAME-IDENTITY (born-red): foo::bar and baz::bar MUST be distinct identities.
#[test]
#[ignore = "born-red until syntactic_fq_path is filled (frame epoch); de-ignore on fill"]
fn atk_frame_identity_cross_module_paths_are_distinct() {
    // Same item name `bar`, DIFFERENT module chains — the bare-name defect would collide these.
    let foo_bar = syntactic_fq_path("mycrate", &["foo".to_string()], "bar");
    let baz_bar = syntactic_fq_path("mycrate", &["baz".to_string()], "bar");

    // The proof the bare-name defect is closed: distinct module paths => distinct identities.
    assert_ne!(
        foo_bar, baz_bar,
        "ATK-FRAME-IDENTITY: foo::bar and baz::bar collided — the bare-name defect is NOT closed. \
         The frame must construct a qualified path (crate::module::item), not key on the bare name."
    );

    // And the constructed paths must actually carry the module qualification (not be the bare ident).
    assert!(
        foo_bar.path.contains("foo") && baz_bar.path.contains("baz"),
        "ATK-FRAME-IDENTITY: the constructed fq_path dropped the module chain — \
         a path that does not embed `foo`/`baz` cannot distinguish the two `bar`s."
    );
}

// NEGATIVE CONTROL (teeth): two IDENTICAL items in the SAME module MUST collide (same identity).
// A trivial impl that made every path unique (e.g. appended a counter) would PASS the ATK above but
// FAIL this — proving the ATK tests construction, not mere uniqueness.
#[test]
#[ignore = "born-red until syntactic_fq_path is filled (frame epoch); de-ignore on fill"]
fn nc_frame_identity_same_module_same_item_collide() {
    let a = syntactic_fq_path("mycrate", &["foo".to_string()], "bar");
    let b = syntactic_fq_path("mycrate", &["foo".to_string()], "bar");

    assert_eq!(
        a, b,
        "NC: two identical items in the same module produced DIFFERENT identities — \
         the construction is non-deterministic or over-disambiguating (it would flag real duplicates \
         as distinct nodes). The qualified-path must be a pure function of (crate, module_chain, item)."
    );
}

// NEGATIVE CONTROL (teeth, boundary): nested module chains must ALSO distinguish — `a::b::item` is
// distinct from `a::item`. Guards against a construction that only looks at the LAST module segment.
#[test]
#[ignore = "born-red until syntactic_fq_path is filled (frame epoch); de-ignore on fill"]
fn nc_frame_identity_nested_module_depth_is_load_bearing() {
    let shallow = syntactic_fq_path("mycrate", &["a".to_string()], "item");
    let deep = syntactic_fq_path("mycrate", &["a".to_string(), "b".to_string()], "item");

    assert_ne!(
        shallow, deep,
        "NC: a::item and a::b::item collided — the construction collapsed nesting depth. \
         The full module chain is load-bearing, not just the leaf segment."
    );
}
