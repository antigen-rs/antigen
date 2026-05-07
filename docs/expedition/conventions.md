# Antigen — Conventions

> Quick reference for naming, file layout, and structural conventions. Prevents
> bikeshedding by giving the team explicit defaults; deviations require explicit
> rationale rather than informal preference.

---

## Antigen names

**Format**: `kebab-case` slug, descriptive of the failure-class.

**Good**: `panicking-in-drop`, `polarity-inverted-class-meet`, `mutexguard-cross-await-point`, `silent-truncation-on-cast`.

**Bad**:
- `BadDropImpl` (PascalCase; doesn't read as a slug; too vague)
- `panic_drop` (snake_case; OK but inconsistent with other conventions; too short)
- `panicking_in_destructor` (snake_case; inconsistent)
- `dont-panic-in-drop` (imperative tone; reads as advice not classification)

**Rules**:
- Lowercase, kebab-case
- Verb-or-adjective form describing the FAILURE-CLASS, not advice on prevention
- 2-6 words
- Prefer specificity over abstraction (`hash-collision-iteration-order` over
  `unstable-iteration`)
- The name is the antigen TYPE name in PascalCase converted to kebab-case for the
  string (`PanickingInDrop` → `"panicking-in-drop"`)

---

## Antigen Rust type names

The actual Rust struct that gets passed to `#[presents]` and `#[immune]` follows
PascalCase, matching Rust conventions:

```rust
#[antigen(name = "panicking-in-drop", ...)]
pub struct PanickingInDrop;

#[presents(PanickingInDrop)]
impl Drop for MyType { ... }
```

**Rules**:
- PascalCase, no abbreviations
- Match the kebab-case name word-for-word
- Use `pub struct Name;` (zero-sized type)
- Place in the antigen module hierarchy (see "Module organization" below)

---

## Family names

Family is the parent class in the 8-class first-principles taxonomy or a project-
specific family.

**Standard families** (the 8 first-principles classes):
- `frame-translation`
- `forgotten-lesson`
- `implicit-coupling`
- `stale-context`
- `premature-abstraction`
- `incompatible-merger`
- `boundary-violation`
- `optionality-collapse`

**Project-specific families** are allowed but discouraged for stdlib antigens. Use
project-specific only when the failure pattern doesn't fit any of the 8 classes
AND has multiple instances within the project.

**Format**: kebab-case, same convention as antigen names.

---

## Witness function names

Witnesses are Rust functions or test/proptest blocks. Naming convention:

```rust
fn <antigen_name_in_snake>_<what_it_proves>_test() { ... }
fn <antigen_name_in_snake>_<what_it_proves>_proptest() { ... }
```

**Examples**:
- `panicking_in_drop_no_unwrap_test`
- `polarity_inverted_class_meet_max_polarity_proptest`
- `mutexguard_cross_await_drop_before_await_test`

**Rules**:
- Snake case (Rust convention for functions)
- Antigen name converted to snake case as prefix
- Subject of the witness as suffix (what it proves)
- Suffix of `_test` or `_proptest` makes the witness type immediately visible
- For tests inside `#[cfg(test)]` modules, the prefix is shortened (drop the
  antigen prefix when the module name already disambiguates)

---

## Module organization

For projects adopting antigen, the antigen-related code lives at:

```
your_project/
├── src/
│   ├── antigens.rs        ← project-specific antigen declarations
│   └── lib.rs
└── tests/
    ├── antigen_witnesses/ ← witness functions live here (per-antigen file)
    │   ├── panicking_in_drop_witness.rs
    │   ├── frame_translation_witness.rs
    │   └── ...
    └── (existing test layout)
```

**For larger projects**, antigens may live in their own crate:

```
your_workspace/
├── your-project-antigens/   ← workspace member crate for antigen declarations
│   └── src/lib.rs
├── your-project-core/
└── ...
```

**Antigen-stdlib (and other published antigen libraries) layout**:

```
antigen-stdlib/
├── src/
│   ├── lib.rs                                ← re-exports
│   └── families/
│       ├── frame_translation.rs              ← antigens for this family
│       ├── boundary_violation.rs
│       ├── implicit_coupling.rs
│       └── ...
└── README.md
```

The `families/<family>.rs` layout makes the family taxonomy structurally visible.

---

## Campsite slug conventions

When the JBD team is active, campsites for ADR drafts follow:

- `adr-NNN-<slug>` for ADR drafts (e.g., `adr-011-cross-crate-versioning`)
- `sweep-X<N>-<theme>` for sweep work (e.g., `sweep-a1-design-ratification`)
- `<role>-<topic>` for role-specific work (e.g., `naturalist-roam-fingerprint-grammar`)
- `bug-<id>-<short-description>` for bug-tracking campsites
- `prior-art-<topic>` for scout's prior-art surveys

Slugs are kebab-case, descriptive, short.

---

## ADR numbering

- ADRs are numbered sequentially, three digits with leading zeros (ADR-001, ADR-002,
  ..., ADR-099, ADR-100, ADR-1001 if needed but unlikely)
- Numbers are assigned at draft-creation time, not at ratification time
- A draft that gets superseded before ratification still occupies its number; the
  number is NOT recycled
- Foundational ADRs (1-8 in this project) are pre-team scaffolding ratifications;
  they don't follow the sweep ratification process. ADR-009 onwards do
- Reservations: skipping an ADR number requires explicit reservation in
  `decisions.md`'s index with a placeholder line ("ADR-N: RESERVED for future
  use; topic <X>")

---

## Witness type abbreviations

For the `witness = ...` field, use these conventions:

| Witness type | Format |
|--------------|--------|
| Test function | `witness = test_function_name` |
| Proptest block | `witness = proptest_function_name` |
| Phantom-type proof | `witness = MyPhantomProof::new()` |
| Clippy lint | `witness = clippy::lint_name` |
| Kani proof | `witness = kani::proof_function_name` |
| Prusti annotation | `witness = prusti::trusted_or_proven_path` |
| Cargo-mutants check | `witness = mutants::function_name_no_missed` |
| Custom witness | `witness = my_module::CustomWitnessType` |

The witness identifier is parsed by cargo-antigen scan to determine its type and
how to validate it.

---

## References field conventions

The `references = [...]` field is open-vocabulary per ADR-009. Use these patterns:

| Reference type | Format |
|----------------|--------|
| URL | `"https://github.com/.../issue/N"` (the full URL as a string) |
| Internal ADR/DEC | `"DEC-030 §1.1"` or `"ADR-007"` (identifier as string) |
| CVE | `"CVE-2025-XXXXX"` |
| RustSec advisory | `"RUSTSEC-2024-XXXX"` |
| RFC | `"RFC-XXXX"` |
| Issue tracker | `"<project>#NNN"` (e.g., `"rust-lang/rust#12345"`) |
| Free text | `"see related discussion in <document>"` (last resort) |

Be consistent within a project; don't mix URL and shortcut forms for the same
reference unless intentional.

---

## Comment conventions on antigen-presenting code

When code is `#[presents(antigen)]`, an inline comment is encouraged but not
required:

```rust
// Per #[presents(PolarityInvertedClassMeet)]: see DEC-030 §1.1.
// This enum's discriminants are strongest-first; meet must use max
// (verified by polarity_proptest below).
#[presents(PolarityInvertedClassMeet)]
pub enum CommutativityClass { ... }
```

The comment is informational; the immunity declaration is structural. Comments
help reviewers understand WHY the antigen applies; the structural enforcement is
what protects against drift.

---

## Documentation conventions for antigens

Each antigen's docstring should answer:

1. **What failure-class** does this antigen recognize? (1-2 sentences)
2. **What real-world instances** motivated this antigen? (1-3 examples with refs)
3. **What's the typical witness pattern** for proving immunity? (brief code example)
4. **What's the family** and why? (1 sentence)
5. **What's NOT covered** by this antigen? (negative scope, important for adopters
   to understand)

Example:

```rust
/// Class enums with strongest-first discriminants must use max for lattice meet.
///
/// **What this antigen recognizes**: a class enum where the discriminant ordering
/// is reverse to the lattice ordering. The lattice's "weakest" element is at the
/// largest discriminant; meet must return the discriminant-larger (lattice-weaker)
/// variant.
///
/// **Real-world instances**: tambear's GAP-BIT-EXACT-1 (DeterminismClass::meet
/// returning strongest instead of weakest); near-miss in DEC-030 v2 with
/// CommutativityClass.
///
/// **Typical witness**: a proptest verifying `meet(a, b) as u8 >= a as u8`
/// AND `meet(a, b) as u8 >= b as u8` for all (a, b) pairs.
///
/// **Family**: frame-translation. The same class name is interpreted with one
/// polarity in the lattice frame and another in the discriminant frame.
///
/// **NOT covered**: classes with weakest-first discriminants (where meet =
/// std::cmp::min IS correct). Antigen flags potential matches; the witness
/// distinguishes correct from incorrect polarity.
#[antigen(name = "polarity-inverted-class-meet", ...)]
pub struct PolarityInvertedClassMeet;
```

---

## When to deviate

Conventions exist to reduce decision fatigue, not to constrain. Deviate when:

- A specific antigen has a name that doesn't fit kebab-case naturally (e.g., a
  pattern named after a specific person/paper)
- A project's existing module organization conflicts with antigen-stdlib's defaults
- A witness type isn't covered by the standard abbreviations

Document the deviation in the antigen's docstring or the project's antigen-related
documentation. The convention's purpose is to be the default, not the rule.

---

## Last word

Conventions are substrate. Like ADRs, like the glossary, like the process — the
team operates inside them and may amend them via the same lifecycle that governs
ADRs (draft → review → ratify). When the team encounters a convention that no
longer serves, propose an amendment.

The conventions are starting context, not constraint.
