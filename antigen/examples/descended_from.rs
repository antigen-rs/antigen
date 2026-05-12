//! Inheritance example: `#[descended_from]` propagates antigen presentations
//! along a lineage chain. ADR-013 / ADR-018.
//!
//! When a child antigen declares `#[descended_from(Parent)]`, the propagation
//! walk in `cargo antigen scan` synthesizes inherited Presentation records on
//! the child for every presentation declared on the parent. Each inherited
//! Presentation carries a `ProvenanceEntry` chain naming the ancestor whose
//! presentations propagated. The audit then surfaces unaddressed inherited
//! presentations as state-7 warnings (ADR-018 §"7-state interaction matrix").
//!
//! Run:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples \
//!     --format json | grep -A 4 inherited_from
//! ```
//!
//! Or:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
//! ```
//!
//! Expected: the scan synthesizes an inherited Presentation for
//! `MemoryUnsafetyClass` on `UseAfterFreeClass`. The audit emits a warn-level
//! state-7 diagnostic because `UseAfterFreeClass` hasn't re-attested the
//! ancestor's witness.

#![allow(dead_code)]

use antigen::{antigen, descended_from, presents};

#[antigen(
    name = "memory-unsafety-class",
    family = "boundary-violation",
    fingerprint = r#"name = matches("MemoryUnsafetyClass")"#,
    summary = "The parent class: any unsoundness around raw-pointer access."
)]
pub struct MemoryUnsafetyClass;

/// Child antigen: a specific kind of memory unsafety.
///
/// By declaring descent from `MemoryUnsafetyClass`, the child inherits the
/// parent's presentations at scan time — but does NOT inherit the parent's
/// immunity claims. ADR-005 sub-clause F: inherited claims do not
/// propagate without re-attestation.
#[antigen(
    name = "use-after-free-class",
    family = "boundary-violation",
    fingerprint = r#"name = matches("UseAfterFreeClass")"#,
    summary = "A specific sub-class of memory unsafety: dangling-pointer dereference."
)]
#[descended_from(MemoryUnsafetyClass)]
pub struct UseAfterFreeClass;

/// A demonstration site bearing `#[presents(MemoryUnsafetyClass)]`.
///
/// The propagation walk takes this presentation and synthesizes an
/// inherited counterpart on `UseAfterFreeClass`'s declaration site. The
/// inherited Presentation has `inherited_from = Some([{antigen_type:
/// "MemoryUnsafetyClass", canonical_path: None}])`.
pub struct VulnerableMemoryHandle {
    ptr: *const u8,
}

#[presents(MemoryUnsafetyClass)]
impl VulnerableMemoryHandle {
    /// Returns a raw pointer that callers might dereference after the
    /// underlying allocation is freed.
    #[must_use]
    pub const fn raw(&self) -> *const u8 {
        self.ptr
    }
}

fn main() {
    println!("antigen descended_from example: ancestor presentations propagate as");
    println!("inherited Presentations on the descendant — but immunity claims do NOT.");
    println!();
    println!("Run `cargo run --bin cargo-antigen -- antigen scan --root antigen/examples`");
    println!("to see the inherited Presentation on `UseAfterFreeClass` synthesized");
    println!("from the explicit #[presents(MemoryUnsafetyClass)] above.");
}
