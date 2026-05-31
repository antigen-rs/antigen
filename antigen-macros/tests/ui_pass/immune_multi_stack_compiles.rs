//! Pass-fixture (findings/immune-multi-stack-const-collision): stacked `#[immune]`
//! attributes must compile cleanly in EVERY position the macro can appear —
//! including on a method inside an `impl` block, which is the position that
//! actually triggered the original bug.
//!
//! Root cause (empirically confirmed): the migration nudge emits a `const` item.
//! With an anonymous `const _: () = {…}`, two stacked `#[immune]` on a method
//! inside an `impl` block produce two associated `const _` items, which Rust
//! rejects with BOTH "`const` items in this context need a name" AND
//! "duplicate definitions with name `_`" (E0592). At module scope the anonymous
//! consts stack fine — so a module-only regression guard would be a false green.
//!
//! Fix: emit a NAMED const with a per-emission-unique name. A named const is
//! legal in module, impl, and fn-body positions, and the unique name prevents
//! the duplicate-definition collision even when the SAME antigen is stacked twice.
//!
//! This fixture is the regression guard: if the emitted const ever reverts to
//! anonymous (or stops being uniquely named per emission), the impl-method cases
//! below fail to compile and the pass-harness goes red.

use antigen_macros::immune;

struct ParseAntigen;
struct AuditAntigen;
struct StackedTwiceAntigen;

fn witness_a() {}
fn witness_b() {}

struct Defended;

impl Defended {
    // TWO different antigens stacked on a method inside an impl block — the
    // associated-const position that triggered the original collision.
    #[immune(crate::ParseAntigen, witness = witness_a)]
    #[immune(
        crate::AuditAntigen,
        requires = ratified_doc(path = "docs/discipline.md", min_version = "1.0")
    )]
    #[allow(deprecated)]
    fn doubly_immune_method(&self) {}

    // The SAME antigen stacked twice (a code-tier witness AND a substrate-tier
    // requires=) on one method — only the per-emission counter (not the antigen
    // path) makes these two const names distinct.
    #[immune(crate::StackedTwiceAntigen, witness = witness_b)]
    #[immune(
        crate::StackedTwiceAntigen,
        requires = signers(required = ["alice"])
    )]
    #[allow(deprecated)]
    fn same_antigen_twice(&self) {}
}

// Also stack at module scope (the position that always worked) to keep coverage
// of the non-impl path.
#[immune(crate::ParseAntigen, witness = witness_a)]
#[immune(crate::AuditAntigen, witness = witness_b)]
#[allow(deprecated)]
fn module_scope_doubly_immune() {}

fn main() {}
