//! Adversarial contracts for the W6a fingerprint grammar (antigen-fingerprint crate).
//!
//! ## Contract taxonomy
//!
//! Active tests (no `#[ignore]`) assert current behavior — they fail if the
//! code regresses. `#[ignore]` tests are pre-implementation contracts: they
//! describe the DESIRED future behavior and are expected to panic or fail until
//! that behavior ships. Remove `#[ignore]` when the feature lands, verify the
//! test FAILS (confirming the contract), then fix to pass.
//!
//! ## W6a attack surface covered
//!
//! | ATK | Component | Description | Status |
//! |-----|-----------|-------------|--------|
//! | W6a-001 | parser | Over-broad fingerprint: `item = struct` matches all structs, no warning | IGNORED (pre-impl warning) |
//! | W6a-002 | parser | Circular fingerprint: structurally impossible in DSL tree | CLOSED (non-issue) |
//! | W6a-003 | parser | Rationale-stuffing: whitespace-only rationale bypasses required check | IGNORED (tolerance API not yet shipped) |
//! | W6a-004a | parser | `not` at top level rejected | ACTIVE (regression guard) |
//! | W6a-004b | parser | `not` under `any_of` rejected | ACTIVE (regression guard) |
//! | W6a-004c | parser | `not` inside `all_of` inside `any_of` — legal (positive sibling present) | ACTIVE (regression guard) |
//! | W6a-004d | parser | `all_of` with ONLY `not` children rejected | ACTIVE (regression guard) |
//! | W6a-005a | parser | Empty `all_of([])` rejected | ACTIVE (regression guard) |
//! | W6a-005b | parser | Empty `any_of([])` rejected | ACTIVE (regression guard) |
//! | W6a-005c | matcher | Empty top-level fingerprint string rejected | ACTIVE (regression guard) |
//! | W6a-006a | parser | Depth cap at exactly MAX_DEPTH: chain of MAX_DEPTH accepted | ACTIVE (boundary test) |
//! | W6a-006b | parser | Depth cap at MAX_DEPTH+1: chain of MAX_DEPTH+1 rejected | ACTIVE (boundary test) |
//! | W6a-007 | matcher | `attr_present("clippy :: panic")` with spaces: silent mismatch | ACTIVE (documents silent mismatch) |
//! | W6a-008 | matcher | `body_contains_macro` on `item = struct`: silent false, no diagnostic | ACTIVE (documents silent false) |
//! | W6a-009 | matcher | `variants = 0..=0` matches zero-variant enum | ACTIVE (edge-case guard) |
//! | W6a-010 | parser | Node count cap: 257 flat constraints rejected | ACTIVE (boundary test) |
//! | W6a-011 | matcher | `not` inside `not` inside `all_of`: double-negation matches correctly | ACTIVE (logic check) |
//! | W6a-012 | matcher | `any_of([item = struct, item = enum])` on impl item: both fail → false | ACTIVE (exhaustive kind check) |

use antigen_fingerprint::{Fingerprint, ItemKind, MAX_DEPTH, MAX_NODES};

fn parse(s: &str) -> syn::Result<Fingerprint> {
    Fingerprint::parse(s)
}

fn parse_ok(s: &str) -> Fingerprint {
    parse(s).unwrap_or_else(|e| panic!("expected parse to succeed: {e}"))
}

fn parse_err(s: &str) -> String {
    parse(s).unwrap_err().to_string()
}

fn item(src: &str) -> syn::Item {
    syn::parse_str::<syn::Item>(src).expect("test item parses")
}

// ============================================================================
// ATK-W6a-001 — Over-broad fingerprint: no warning when fingerprint matches
// everything in a kind bucket
// ============================================================================

/// ATK-W6a-001 — Status: IGNORED (pre-impl). When the over-broad-fingerprint
/// warning system ships, `Fingerprint::parse` (or a separate lint pass)
/// should return a warning or error when a fingerprint would trivially match
/// every item of a given kind in a codebase. `item = struct` alone matches
/// all structs — the user almost certainly wanted more constraints.
///
/// The parser currently accepts this without complaint. When the warning
/// system lands: remove `#[ignore]`, verify this test FAILS (the parse now
/// warns), then update the assertion to match the warning API.
#[test]
#[ignore = "pre-impl: over-broad fingerprint warning system not yet shipped"]
fn atk_w6a_001_over_broad_fingerprint_item_only_warns() {
    // `item = struct` alone would match every struct in a workspace.
    // A future warning pass should flag this as an autoimmunity risk.
    let result = parse("item = struct");
    // Once warning system exists, expect `Err` or a `warnings` field.
    assert!(result.is_err(), "ATK-W6a-001: `item = struct` alone must warn/reject as over-broad");
}

// ============================================================================
// ATK-W6a-002 — Circular fingerprint: CLOSED (non-issue)
// ============================================================================

// The fingerprint DSL produces a parsed tree at load time. Self-reference
// is structurally impossible: there is no `name_ref = <fingerprint_name>`
// operator and no deferred-evaluation mechanism. Circular fingerprints cannot
// be expressed in the grammar. This attack surface is closed by construction.
// No test needed — the impossibility is architectural.

// ============================================================================
// ATK-W6a-003 — Rationale-stuffing: whitespace-only rationale
// ============================================================================

/// ATK-W6a-003 — Status: IGNORED (pre-impl). The `#[antigen_tolerance]`
/// macro attribute (W6a tolerance API) accepts a `rationale = "..."` field.
/// A whitespace-only rationale `rationale = "   "` must be rejected with a
/// compile-time error — it satisfies the structural presence of the field
/// while providing zero information.
///
/// This contract cannot be tested until the tolerance attribute is implemented.
/// When the macro ships: remove `#[ignore]`, compile a `#[antigen_tolerance]`
/// with `rationale = "   "`, verify it produces a compile error.
#[test]
#[ignore = "pre-impl: #[antigen_tolerance] attribute not yet shipped"]
fn atk_w6a_003_whitespace_only_rationale_is_rejected() {
    // Pseudo-code for the compile-fail assertion once tolerance API ships:
    //
    //   #[antigen_tolerance(antigen = "frame-translation", rationale = "   ")]
    //   impl MyType { ... }
    //
    // Should produce: error: `rationale` must not be blank or whitespace-only
    panic!("pre-implementation contract — remove #[ignore] when #[antigen_tolerance] ships")
}

// ============================================================================
// ATK-W6a-004 — `not` placement rules
// ============================================================================

// 004a: `not` at top level must be rejected.
#[test]
fn atk_w6a_004a_not_at_top_level_rejected() {
    let err = parse_err("not(item = enum)");
    assert!(
        err.contains("not"),
        "ATK-W6a-004a: top-level `not` must be rejected with a diagnostic mentioning 'not'; got: {err}"
    );
}

// 004b: `not` directly under `any_of` must be rejected (De Morgan loophole).
#[test]
fn atk_w6a_004b_not_under_any_of_rejected() {
    let err = parse_err("any_of([not(item = enum), item = struct])");
    assert!(
        err.contains("not"),
        "ATK-W6a-004b: `not` under `any_of` must be rejected; got: {err}"
    );
}

// 004c: `not` inside `all_of` inside `any_of` — the `not` has a positive
// sibling inside `all_of`, so it is legal. The `any_of` wrapper must not
// contaminate the legality check.
#[test]
fn atk_w6a_004c_not_inside_all_of_inside_any_of_is_legal() {
    // any_of([
    //   all_of([item = struct, not(name = matches("Test*"))]),
    //   item = enum
    // ])
    //
    // The `not` is inside `all_of` with positive sibling `item = struct`.
    // This must parse and match correctly.
    let fp = parse_ok(
        r#"any_of([all_of([item = struct, not(name = matches("Test*"))]), item = enum])"#,
    );
    // Matches a non-Test struct.
    assert!(
        fp.matches(&item("struct ValidStruct;")),
        "ATK-W6a-004c: any_of([all_of([item=struct, not(name=Test*)]), item=enum]) must match ValidStruct"
    );
    // Does NOT match a Test-prefixed struct.
    assert!(
        !fp.matches(&item("struct TestFoo;")),
        "ATK-W6a-004c: any_of([all_of([item=struct, not(name=Test*)]), item=enum]) must NOT match TestFoo"
    );
    // Matches an enum (via second arm of any_of).
    assert!(
        fp.matches(&item("enum E { A, B }")),
        "ATK-W6a-004c: any_of([all_of([item=struct, not(...)]), item=enum]) must match enum"
    );
}

// 004d: `all_of` with only `not` children (no positive sibling) must be rejected.
#[test]
fn atk_w6a_004d_all_of_with_only_not_children_rejected() {
    let err = parse_err("all_of([not(item = enum), not(item = struct)])");
    assert!(
        err.contains("positive"),
        "ATK-W6a-004d: `all_of` with only `not` children must mention 'positive' in error; got: {err}"
    );
}

// ============================================================================
// ATK-W6a-005 — Empty composition lists
// ============================================================================

// 005a: `all_of([])` must be rejected.
#[test]
fn atk_w6a_005a_empty_all_of_rejected() {
    let err = parse_err("all_of([])");
    assert!(
        err.contains("at least one"),
        "ATK-W6a-005a: empty `all_of([])` must be rejected; got: {err}"
    );
}

// 005b: `any_of([])` must be rejected.
#[test]
fn atk_w6a_005b_empty_any_of_rejected() {
    let err = parse_err("any_of([])");
    assert!(
        err.contains("at least one"),
        "ATK-W6a-005b: empty `any_of([])` must be rejected; got: {err}"
    );
}

// 005c: An empty fingerprint string must be rejected.
#[test]
fn atk_w6a_005c_empty_fingerprint_string_rejected() {
    let err = parse_err("");
    assert!(
        err.contains("at least one"),
        "ATK-W6a-005c: empty fingerprint must be rejected; got: {err}"
    );
}

// ============================================================================
// ATK-W6a-006 — Depth cap boundary
// ============================================================================

/// 006a: A chain of exactly `MAX_DEPTH` `all_of` wrappers around a leaf must
/// be accepted. The leaf is at depth `MAX_DEPTH+1` inside `check_depth_and_count`;
/// however, the validator increments depth BEFORE checking, then checks
/// `depth > MAX_DEPTH`. A leaf at depth `MAX_DEPTH` passes (`MAX_DEPTH > MAX_DEPTH`
/// is false). Verify the exact boundary: chain of `MAX_DEPTH-1` `all_of`s
/// wrapping one leaf constraint — outermost `all_of` is at depth 1, each nesting
/// adds 1, leaf is at depth `MAX_DEPTH`.
#[test]
fn atk_w6a_006a_depth_cap_boundary_accepted() {
    // Build all_of([all_of([...all_of([item = enum])...])]) with MAX_DEPTH-1
    // all_of levels. The outermost is at depth=1, the leaf is at depth=MAX_DEPTH.
    // This should be accepted (depth <= MAX_DEPTH).
    let mut s = String::from("item = enum");
    for _ in 0..(MAX_DEPTH - 1) {
        s = format!("all_of([{s}])");
    }
    parse_ok(&s);
}

/// 006b: One level deeper than 006a must be rejected. The leaf is now at
/// depth `MAX_DEPTH+1`, which exceeds `MAX_DEPTH`.
#[test]
fn atk_w6a_006b_depth_cap_boundary_rejected() {
    // Add one more all_of level on top of 006a — leaf at depth MAX_DEPTH+1.
    let mut s = String::from("item = enum");
    for _ in 0..MAX_DEPTH {
        s = format!("all_of([{s}])");
    }
    let err = parse_err(&s);
    assert!(
        err.contains("depth"),
        "ATK-W6a-006b: fingerprint at depth MAX_DEPTH+1 must mention 'depth'; got: {err}"
    );
}

// ============================================================================
// ATK-W6a-007 — `attr_present` with spaces around `::` in path
// ============================================================================

/// ATK-W6a-007 — `attr_present("clippy :: panic")` silently mismatches.
///
/// The matcher's `attr_path_matches` renders the attribute path as segments
/// joined with `"::"` (no spaces). A user who writes spaces around `::` in the
/// `attr_present` argument will never match, silently. The fingerprint parses,
/// the matcher runs, and `false` is returned with no diagnostic.
///
/// This test documents the silent-mismatch behavior. It is a ACTIVE regression
/// guard: if the matcher is later fixed to normalize spaces in `attr_present`
/// paths, this test should be updated to assert `true` instead of `false`.
#[test]
fn atk_w6a_007_attr_present_spaces_around_colons_silently_mismatches() {
    // `#[derive(Debug)]` — `derive` is the path.
    let it = item("#[derive(Debug)] struct Foo;");
    // Correct form (no spaces) matches.
    let fp_correct = parse_ok(r#"attr_present("derive")"#);
    assert!(
        fp_correct.matches(&it),
        "ATK-W6a-007: `attr_present(\"derive\")` (correct) must match #[derive(Debug)] struct"
    );
    // Spaced form silently mismatches — the user wrote an incorrect path.
    // This is a documentation test: the matcher does no normalization.
    // If the path was "clippy :: panic" the matcher would receive that string
    // and compare it to the rendered "clippy::panic" — they are not equal.
    // We cannot put spaces in a path identifier for derive, but we can test
    // that a substring that includes extraneous spaces in the needle doesn't match.
    let fp_spaced = parse_ok(r#"attr_present("derive ")"#);
    assert!(
        !fp_spaced.matches(&it),
        "ATK-W6a-007: `attr_present(\"derive \")` with trailing space must NOT match (no normalization)"
    );
}

// ============================================================================
// ATK-W6a-008 — `body_contains_macro` on non-fn item returns false silently
// ============================================================================

/// ATK-W6a-008 — `body_contains_macro` applied to a struct returns `false`
/// without a diagnostic. A fingerprint like:
///
///   `item = struct, body_contains_macro("panic")`
///
/// can parse and match, but for a struct item `body_contains_macro` is always
/// false — structs have no body to walk. The fingerprint will never match any
/// struct regardless of what macros appear in its field types. The user gets
/// silent empty results with no guidance.
///
/// This is a documentation test. When a linting pass is added that warns on
/// impossible constraint combinations (`item=struct` + `body_contains_macro`),
/// update this test to assert the warning.
#[test]
fn atk_w6a_008_body_contains_macro_on_struct_always_false() {
    // A struct that (in real code) would typically appear with panic in context —
    // but structs have no function body, so body_contains_macro is always false.
    let fp = parse_ok(r#"item = struct, body_contains_macro("panic")"#);
    let it = item("struct DropBomb;");
    assert!(
        !fp.matches(&it),
        "ATK-W6a-008: body_contains_macro on item=struct must always be false (no body)"
    );
    // Confirm: an impl block with a body DOES match.
    let fp_impl = parse_ok(r#"item = impl, body_contains_macro("panic")"#);
    let impl_item = item("impl Drop for DropBomb { fn drop(&mut self) { panic!(\"bomb\"); } }");
    assert!(
        fp_impl.matches(&impl_item),
        "ATK-W6a-008: body_contains_macro on item=impl with panic body must match"
    );
}

// ============================================================================
// ATK-W6a-009 — `variants = 0..=0` matches a zero-variant enum
// ============================================================================

/// ATK-W6a-009 — Zero-variant enum edge case.
///
/// `variants = 0..=0` is legal at parse time and should match an enum with no
/// variants (an uninhabited type). This is a real pattern in Rust: `enum Never {}`
/// is used to represent uninhabited types. The matcher should handle this.
#[test]
fn atk_w6a_009_variants_zero_zero_matches_uninhabited_enum() {
    let fp = parse_ok("variants = 0..=0");
    // Zero-variant enum (uninhabited).
    assert!(
        fp.matches(&item("enum Never {}")),
        "ATK-W6a-009: `variants = 0..=0` must match zero-variant enum"
    );
    // Single-variant enum does not match.
    assert!(
        !fp.matches(&item("enum Unit { A }")),
        "ATK-W6a-009: `variants = 0..=0` must NOT match single-variant enum"
    );
    // Applying variants to a struct always returns false.
    assert!(
        !fp.matches(&item("struct S;")),
        "ATK-W6a-009: `variants = 0..=0` on struct must be false"
    );
}

// ============================================================================
// ATK-W6a-010 — Node count cap: 257 flat constraints rejected
// ============================================================================

/// ATK-W6a-010 — Total node count cap at `MAX_NODES` (256).
///
/// Building a fingerprint with `MAX_NODES+1` = 257 flat constraints must be
/// rejected. Each constraint counts as one node; the cap fires when `nodes`
/// exceeds `MAX_NODES` during `check_depth_and_count`.
#[test]
fn atk_w6a_010_node_count_cap_exceeded_rejected() {
    // 257 `item = struct` constraints, comma-separated.
    // Each one parses and adds to the top-level constraint list before validate()
    // fires. The 257th node pushes count to 257 > 256.
    let constraints: Vec<&str> = vec!["item = struct"; MAX_NODES + 1];
    let s = constraints.join(", ");
    let err = parse_err(&s);
    assert!(
        err.contains("node count") || err.contains("nodes"),
        "ATK-W6a-010: {MAX_NODES}+1 constraints must trigger node count error; got: {err}"
    );
}

/// ATK-W6a-010b — Exactly `MAX_NODES` (256) flat constraints accepted.
#[test]
fn atk_w6a_010b_node_count_cap_boundary_accepted() {
    let constraints: Vec<&str> = vec!["item = struct"; MAX_NODES];
    let s = constraints.join(", ");
    parse_ok(&s);
}

// ============================================================================
// ATK-W6a-011 — Double negation: `not(not(...))` inside `all_of`
// ============================================================================

/// ATK-W6a-011 — Double negation inside `all_of` should parse and evaluate
/// correctly. `not(not(item = struct))` is double-negation — it should behave
/// as `item = struct` semantically. The `check_not_placement` validator recurses
/// into `not` children with `in_legal_all_of = false`, so the inner `not` is
/// checked at the outer level (not inside an `all_of`). This means:
///
///   `all_of([item = struct, not(not(item = struct))])`
///
/// has an inner `not(item = struct)` that is the child of the outer `not`.
/// The outer `not` is inside `all_of` (legal). The inner `not` is recursed
/// with `in_legal_all_of = false` — so it fires as "not is only legal inside
/// `all_of`" and the parse FAILS.
///
/// This is the correct behavior per ADR-010 Amendment 3 OQ3 — `not(not(x))`
/// is a De Morgan identity that can be rewritten as `x`, and the grammar
/// does not need to support it. This test confirms the rejection.
#[test]
fn atk_w6a_011_double_negation_inside_all_of_is_rejected() {
    let err = parse_err("all_of([item = struct, not(not(item = struct))])");

    assert!(
        err.contains("not"),
        "ATK-W6a-011: `not(not(x))` must be rejected (inner not is outside all_of); got: {err}"
    );
}

// ============================================================================
// ATK-W6a-012 — `any_of` exhaustive kind mismatch
// ============================================================================

/// ATK-W6a-012 — `any_of([item = struct, item = enum])` applied to an `impl`
/// item returns `false`. Both arms fail; the result is `false` with no
/// diagnostic. This is correct behavior but easy to misuse: a user who writes
/// an antigen fingerprint for an `impl` block will get silent non-matches if
/// they list the wrong item kinds.
#[test]
fn atk_w6a_012_any_of_all_arms_fail_returns_false_silently() {
    let fp = parse_ok("any_of([item = struct, item = enum])");
    let impl_item = item("impl Foo { fn bar(&self) {} }");
    assert!(
        !fp.matches(&impl_item),
        "ATK-W6a-012: any_of([item=struct, item=enum]) must return false for impl items"
    );
    // Verify the node_kind dispatch optimization: node_kind() on any_of
    // returns None (no definitive kind constraint at top level).
    let fp2 = parse_ok("any_of([item = struct, item = enum])");
    assert_eq!(
        fp2.node_kind(),
        None,
        "ATK-W6a-012: node_kind() for any_of must be None (ambiguous kinds)"
    );
}

// ============================================================================
// ATK-W6a-013 — `has_method` signature whitespace normalization
// ============================================================================

/// ATK-W6a-013 — The `has_method` matcher uses `normalize_ws` to compare
/// signatures. `normalize_ws` collapses internal whitespace, so patterns with
/// extra spaces around tokens are equivalent to compact forms.
///
/// **Silent mismatch hazard**: `proc_macro2::ToTokens` for a `&self` receiver
/// renders as `"& self"` (space between `&` and `self`). A user writing
/// `"(&self, T) -> U"` in their fingerprint will never match because the
/// rendered form is `"(& self, T) -> U"`. After `normalize_ws`, both sides
/// lose extra spaces BUT the content differs: `"(&self"` vs `"(& self"` —
/// the `(` is attached to `&` in the user's pattern but `normalize_ws` only
/// collapses runs of whitespace, it doesn't insert spaces between `(` and `&`.
///
/// The correct pattern for a `&self` receiver is `"(& self, T) -> U"`.
/// This test documents both the normalization behavior AND the silent mismatch.
#[test]
fn atk_w6a_013_has_method_signature_whitespace_normalized() {
    let impl_src = "impl Lattice { fn meet(&self, other: Self) -> Self { unimplemented!() } }";

    // Correct form: `& self` with a space — matches the token render of `&self`.
    let fp_correct =
        parse_ok(r#"item = impl, has_method("meet", "(& self, Self) -> Self")"#);
    assert!(
        fp_correct.matches(&item(impl_src)),
        "ATK-W6a-013: has_method with '& self' (space after &) must match"
    );

    // normalize_ws collapses runs of whitespace within tokens but does NOT
    // remove punctuation-adjacent spaces. `"(&  self,  Self) -> Self"` (extra
    // internal spaces within the `& self` cluster) collapses correctly.
    let fp_extra_inner = parse_ok(r#"item = impl, has_method("meet", "(&  self,  Self)  ->  Self")"#);
    assert!(
        fp_extra_inner.matches(&item(impl_src)),
        "ATK-W6a-013: has_method with extra spaces INSIDE tokens collapses via normalize_ws"
    );

    // SILENT MISMATCH: `(&self, ...)` — no space after `&` — never matches.
    // The token stream renders `&self` as `& self`; `(&self` != `(& self` after
    // normalize_ws because normalize_ws doesn't insert the missing space.
    // ATK-W6a-013 documents this as a known usability hazard.
    let fp_wrong = parse_ok(r#"item = impl, has_method("meet", "(&self, Self) -> Self")"#);
    assert!(
        !fp_wrong.matches(&item(impl_src)),
        "ATK-W6a-013 (silent mismatch): '(&self, ...)' without space after '&' must NOT match \
         (ToTokens renders '& self' with space; normalize_ws cannot recover the missing space)"
    );
}

// ============================================================================
// ATK-W6a-014 — `node_kind` dispatch with nested `all_of`
// ============================================================================

/// ATK-W6a-014 — `Fingerprint::node_kind` should find an `item = <kind>`
/// constraint inside a nested `all_of` and return it for dispatch optimization.
/// Per the `node_kind_hint` implementation, it descends into `AllOf` children
/// but not `AnyOf` (too ambiguous).
#[test]
fn atk_w6a_014_node_kind_extracted_from_nested_all_of() {
    // item = struct is the first constraint inside an all_of — node_kind should
    // return Some(Struct).
    let fp = parse_ok(r#"all_of([item = struct, attr_present("repr")])"#);
    assert_eq!(
        fp.node_kind(),
        Some(ItemKind::Struct),
        "ATK-W6a-014: node_kind must return Some(Struct) from item=struct inside all_of"
    );
    // any_of wrapper — node_kind returns None (ambiguous).
    let fp2 = parse_ok("any_of([item = struct, item = enum])");
    assert_eq!(
        fp2.node_kind(),
        None,
        "ATK-W6a-014: node_kind must return None for any_of([struct, enum])"
    );
    // No item constraint at all — node_kind returns None.
    let fp3 = parse_ok(r#"name = matches("*Class")"#);
    assert_eq!(
        fp3.node_kind(),
        None,
        "ATK-W6a-014: node_kind must return None when no item= constraint present"
    );
}
