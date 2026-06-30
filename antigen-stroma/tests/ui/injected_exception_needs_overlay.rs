// ATK-FRAME-INJECT-FROM-OVERLAY (gap-closing, build-surfaced) — an `InjectedException` MUST be
// constructible ONLY from an overlay; "injected must derive from overlay" is unconstructible to
// violate (ADR-067 Open-seam-4 / write.rs §F).
//
// WHY THIS ATK EXISTS: the builder applied the C3 (law-in-types) pattern to write.rs — a THIRD
// type-state the §8 seed-table did not enumerate (the observer's F-005). A newly-enforced invariant
// with NO guard is one refactor away from silently regressing (someone adds `pub fn new()` or makes
// the field `pub`). This compile-state ATK locks the type-state: the ONLY door is `from_overlay`.
//
// THIS FIXTURE: construct an `InjectedException` by struct-literal (bypassing from_overlay). The
// private `_private: ()` field makes this a privacy error — it MUST NOT compile.

use antigen_stroma::write::InjectedException;

fn main() {
    // Forge an injected exception with NO overlay provenance. The private field forbids the literal.
    let _forged = InjectedException { _private: () }; // MUST NOT COMPILE — private field.
}
