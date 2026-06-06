//! ADR-014 `#[antigen_generates]` — macro-output recognition (the fifth core
//! macro). Integration coverage for the generates-synthesis pass.
//!
//! `cargo antigen scan` walks the source AST: it sees a `#[derive(Foo)]`
//! invocation but NOT the code the `Foo` derive emits. A macro author declares
//! `#[antigen_generates(X, rationale=...)]` on their macro DEFINITION; the scan
//! connects that declaration to every macro INVOCATION and emits a synthetic
//! presentation at the invocation site. Same-workspace only (ADR-014 §A3).
//!
//! These tests build a hermetic single-crate workspace in a tempdir holding
//! BOTH the (fake) macro definition (carrying `#[antigen_generates]`) and the
//! invocation site, since the v0.3 synthesis is same-workspace. The scanner is
//! a syntactic walker, so the "macro" need not actually expand — the attribute
//! is read as source text.

use std::io::Write;

use antigen::scan::{MatchKind, scan_workspace};

/// Write a single `.rs` file into a fresh tempdir and scan it.
fn scan_src(src: &str) -> antigen::scan::ScanReport {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("lib.rs");
    let mut f = std::fs::File::create(&path).expect("create lib.rs");
    f.write_all(src.as_bytes()).expect("write lib.rs");
    drop(f);
    scan_workspace(dir.path(), None).expect("scan")
}

const DERIVE_GENERATOR_AND_INVOCATION: &str = r#"
// --- macro DEFINITION side (carries the generates declaration) ---
#[antigen_generates(
    PanickingInDrop,
    rationale = "This derive emits a Drop impl that may panic if the inner type's \
                 destructor panics; users must verify inner types are panic-safe.",
)]
#[proc_macro_derive(PanicDrop)]
pub fn panic_drop_derive(input: TokenStream) -> TokenStream { input }

// --- INVOCATION side ---
#[derive(PanicDrop)]
struct ConsumerType {
    inner: String,
}
"#;

#[test]
fn generates_declaration_is_discovered_on_macro_definition() {
    let report = scan_src(DERIVE_GENERATOR_AND_INVOCATION);
    assert_eq!(
        report.generates_declarations.len(),
        1,
        "the #[antigen_generates] declaration on the derive fn must be discovered"
    );
    let g = &report.generates_declarations[0];
    assert_eq!(g.antigen_type, "PanickingInDrop");
    assert_eq!(
        g.macro_name, "PanicDrop",
        "the registered macro name must be the proc_macro_derive name (used at #[derive] sites), \
         not the fn name"
    );
    assert!(!g.rationale.is_empty(), "rationale must be captured");
}

#[test]
fn derive_invocation_gets_synthetic_generated_presentation() {
    let report = scan_src(DERIVE_GENERATOR_AND_INVOCATION);

    // A synthetic presentation for PanickingInDrop must appear at the
    // #[derive(PanicDrop)] invocation site, attributed to the consumer struct.
    let generated: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.antigen_type == "PanickingInDrop" && p.item_kind == "generated_PanicDrop")
        .collect();

    assert_eq!(
        generated.len(),
        1,
        "exactly one generated presentation must be synthesized at the #[derive(PanicDrop)] site; \
         got: {:?}",
        report
            .presentations
            .iter()
            .map(|p| (&p.antigen_type, &p.item_kind))
            .collect::<Vec<_>>()
    );
    let p = generated[0];
    assert_eq!(
        p.match_kind,
        MatchKind::ExplicitMarker,
        "a generated presentation is author-declared, not a fingerprint guess"
    );
    // Attributed to the consumer struct so an immunity on it would address it.
    assert!(
        matches!(&p.item_target, antigen::scan::ItemTarget::Struct(s) if s == "ConsumerType"),
        "the generated presentation must be attributed to the #[derive]'d item (ConsumerType); \
         got {:?}",
        p.item_target
    );
}

#[test]
fn unrelated_derive_does_not_synthesize() {
    // A #[derive(Clone)] with no matching generator must NOT produce a
    // generated presentation — only declared generators connect.
    let report = scan_src(
        r#"
#[antigen_generates(
    PanickingInDrop,
    rationale = "emits a Drop impl that may panic; verify inner types are panic-safe.",
)]
#[proc_macro_derive(PanicDrop)]
pub fn panic_drop_derive(input: TokenStream) -> TokenStream { input }

#[derive(Clone, Debug)]
struct Unrelated;
"#,
    );
    assert!(
        report
            .presentations
            .iter()
            .all(|p| !p.item_kind.starts_with("generated_")),
        "a #[derive(Clone, Debug)] with no matching generator must synthesize nothing; got: {:?}",
        report
            .presentations
            .iter()
            .filter(|p| p.item_kind.starts_with("generated_"))
            .map(|p| (&p.antigen_type, &p.item_kind))
            .collect::<Vec<_>>()
    );
}

#[test]
fn bang_macro_invocation_synthesizes() {
    // A macro_rules-style generator registered by fn/macro name, invoked as
    // name!(...). The generator declaration here is on a fn (fallback name
    // resolution = the fn's own name), and the invocation is name!(...).
    let report = scan_src(
        r#"
#[antigen_generates(
    UnpinnedDependency,
    rationale = "this macro expands to a dependency-include without a pinned version.",
)]
pub fn include_dep(input: TokenStream) -> TokenStream { input }

fn consumer() {
    include_dep!(some, args);
}
"#,
    );
    let generated = report
        .presentations
        .iter()
        .filter(|p| {
            p.antigen_type == "UnpinnedDependency" && p.item_kind == "generated_include_dep"
        })
        .count();
    assert_eq!(
        generated,
        1,
        "the include_dep!(...) bang invocation must synthesize one generated presentation; got: {:?}",
        report
            .presentations
            .iter()
            .map(|p| (&p.antigen_type, &p.item_kind))
            .collect::<Vec<_>>()
    );
}

#[test]
fn empty_rationale_is_a_parse_failure_not_a_silent_generator() {
    // ADR-014 §Sub-clause F: a generation claim without rationale is not a
    // claim. The scan-side validate must surface it on parse_failures and NOT
    // register the generator.
    let report = scan_src(
        r#"
#[antigen_generates(PanickingInDrop, rationale = "")]
#[proc_macro_derive(PanicDrop)]
pub fn panic_drop_derive(input: TokenStream) -> TokenStream { input }
"#,
    );
    assert!(
        report.generates_declarations.is_empty(),
        "an empty-rationale generates declaration must not register a generator"
    );
    assert!(
        report
            .parse_failures
            .iter()
            .any(|f| f.error.contains("antigen_generates") && f.error.contains("rationale")),
        "empty rationale must surface as a parse failure; got: {:?}",
        report.parse_failures
    );
}

#[test]
fn stacked_generates_declarations_register_all() {
    // ADR-014: a macro can stack multiple #[antigen_generates]. Both must
    // register, and a single invocation must synthesize both presentations.
    let report = scan_src(
        r#"
#[antigen_generates(PanickingInDrop, rationale = "the emitted Drop impl may panic.")]
#[antigen_generates(UnpinnedDependency, rationale = "the expansion includes an unpinned dep.")]
#[proc_macro_derive(MultiGen)]
pub fn multi_gen(input: TokenStream) -> TokenStream { input }

#[derive(MultiGen)]
struct C;
"#,
    );
    assert_eq!(
        report.generates_declarations.len(),
        2,
        "both stacked #[antigen_generates] declarations must register"
    );
    let path = "generated_MultiGen";
    let kinds: Vec<&str> = report
        .presentations
        .iter()
        .filter(|p| p.item_kind == path)
        .map(|p| p.antigen_type.as_str())
        .collect();
    assert!(
        kinds.contains(&"PanickingInDrop") && kinds.contains(&"UnpinnedDependency"),
        "the single #[derive(MultiGen)] invocation must synthesize BOTH generated presentations; \
         got: {kinds:?}"
    );
}

#[test]
fn workspace_root_with_no_generators_is_unaffected() {
    // Regression: a workspace with zero #[antigen_generates] declarations must
    // produce no generated presentations and behave exactly as before.
    let report = scan_src(
        r#"
#[derive(Clone, Debug)]
struct Plain;

fn uses_a_macro() {
    println!("hello");
    vec![1, 2, 3];
}
"#,
    );
    assert!(report.generates_declarations.is_empty());
    assert!(
        report
            .presentations
            .iter()
            .all(|p| !p.item_kind.starts_with("generated_")),
        "no generators ⇒ no generated presentations"
    );
}
