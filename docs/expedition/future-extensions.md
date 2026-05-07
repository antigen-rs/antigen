# Antigen — Future Extensions

> Captured shape for the project's expansion arc beyond v0.1's basic macros + scan.
> Each extension is staged: lower-friction additions land first; harder/riskier
> additions wait for the substrate to mature. The team owns prioritization but
> this document gives them the full landscape.

---

## The expansion arc

The project's value compounds in this order:

```
Phase 1: declared markers + scan (THE CURRENT v0.0.1 SHIP)
  ↓
Phase 2: witness validation (audit subcommand)
  ↓
Phase 3: fingerprint-driven hunting (find unmarked vulnerable code)
  ↓
Phase 4: vaccinate (bulk apply immunity across structural families)
  ↓
Phase 5: rust-analyzer LSP integration (real-time inline diagnostics)
  ↓
Phase 6: cargo antigen fix (auto-apply rewrites, like cargo fix)
  ↓
Phase 7: build-time integration (#[deny(antigen_unaddressed)])
  ↓
Phase 8: cross-tool delegation (clippy/kani/prusti/proptest witness adapters)
  ↓
Phase 9: cross-crate antigen versioning (semver discipline for the library)
```

Each phase delivers value independently. Stalling at any phase is acceptable; the
substrate continues serving its consumers.

The most important thing for the team to understand: **the project is "complete"
even at Phase 2**. Phase 2's audit + working witness validation provides real
value to early adopters. Phase 3+ extensions are amplification, not necessity.

---

## Phase 2: Witness validation

**Status**: scoped for sweep A2/A3 (in progress as of 2026-05-07).

The audit subcommand walks every `#[immune(X, witness = Y)]` declaration and
verifies:

1. The witness identifier `Y` resolves to a real Rust item in the workspace
2. The item is a test/proptest function (or a delegated reference to a known
   tool — clippy lint, kani proof, etc.)
3. (Optionally) Run the test/proptest and verify it passes

Output: per-immunity validation pass/fail. Reports broken witnesses (named but
missing) and witnesses that exist but fail.

**What's implementable in Phase 2 without depending on later phases**:
- Identifier resolution (parse the workspace, find item with the matching name)
- Test-function detection (look for `#[test]` attribute, `proptest!` macro)
- Optional test invocation (run `cargo test --test <name>` for the witness)

**What's deferred to Phase 8**: delegation to external tools. Witnesses that
reference `clippy::lint_name` or `kani::proof_name` need adapters that
understand each tool's invocation pattern. Phase 2 audits report these as
"external witness; manual validation required" until Phase 8 lands the adapters.

---

## Phase 3: Fingerprint-driven hunting

**Status**: scoped for sweep A4-A5; depends on ADR-010 v1 grammar plus extensions.

Scan today only finds DECLARED `#[presents]` attributes. Phase 3 inverts: scan
walks unmarked code looking for sites that MATCH a known antigen's fingerprint
but DON'T have any antigen markers. These are *latent* presentations — the
structural pattern is present but the developer hasn't acknowledged the
vulnerability.

This is where antigen earns its name. The immune system patrols the body; it
doesn't wait to be told what's wrong. Phase 3's `cargo antigen scan --hunt`
operates the same way:

```sh
$ cargo antigen scan --hunt

Hunting for unmarked sites matching declared antigens...

⚠ Latent presentation found:
  antigen: polarity-inverted-class-meet
  family:  frame-translation
  site:    crates/yourproj/src/something.rs:45
  matched: enum FooClass with strongest-first discriminants and meet method

  No #[presents] or #[immune] declared on this item.

  Suggested action:
    a) Add #[presents(PolarityInvertedClassMeet)] (acknowledges vulnerability;
       requires #[immune] or #[antigen_tolerance] to clear scan)
    b) Add #[immune(PolarityInvertedClassMeet, witness = ...)] (proactive
       immunity if you've verified the meet polarity)
    c) Add #[antigen_tolerance(PolarityInvertedClassMeet, reason = "...")]
       (explicit opt-out with rationale)
```

This is the killer feature for adoption. Users get value from antigen-stdlib
WITHOUT writing any antigens themselves — they just import the library, run
hunt, and see what fires.

**Implementation considerations**:
- The fingerprint grammar (ADR-010) v1 is too coarse for accurate hunting
  alone; expect false positives in early Phase 3
- Per-antigen tolerance for false-positive rates: stdlib antigens that fire >5%
  on real codebases are over-broad and need refinement
- Hunt mode is OPT-IN initially (`--hunt` flag), then opt-out by default once
  precision is high enough for most use cases

---

## Phase 4: Vaccinate (bulk apply immunity)

**Status**: scoped for sweep A5; design described in `case-study-determinism-class.md`.

`cargo antigen vaccinate <antigen> <pattern>` applies known immunity across a
structural family in one operation:

```sh
$ cargo antigen vaccinate PolarityInvertedClassMeet 'enum *Class with meet method'
```

Mechanics:
1. Use the structural pattern to find target sites
2. For each target, check existing immunity status
3. For sites without immunity, scaffold a witness function and add `#[presents]`
   + `#[immune]` markers
4. Display a summary; require confirmation before applying

This is essentially a refactoring tool that operates at the antigen-graph
level. It mirrors `cargo fix` for clippy lints but for antigen markers.

---

## Phase 5: Rust-analyzer LSP integration

**Status**: future; depends on stable v0.x antigen API + community interest.

A `rust-analyzer` plugin (or similar IDE integration) that surfaces antigen
state inline during editing:

- **Hover**: show antigen declarations with full metadata (family, summary,
  references, fingerprint) when hovering over a `#[antigen]` or `#[presents]`
  invocation
- **Inline diagnostics**: red squiggle under unaddressed presentations (like
  clippy lints today). Click to see the antigen and suggested actions.
- **Quick-fix actions**: "add #[immune] with stub witness", "add
  #[antigen_tolerance]", "navigate to antigen declaration"
- **Auto-completion**: when typing `#[presents(`, list all antigens in scope
  (including those imported from antigen-stdlib)

This is the ergonomics phase that makes antigen feel native to the IDE
experience. The 60-second-declaration target (per ADR-008 named-observer
ergonomics) becomes 5 seconds with good IDE integration.

**Implementation surface**: rust-analyzer has a stable LSP extension API.
A separate `antigen-rust-analyzer` crate (workspace member or independent)
provides the language-server extensions.

---

## Phase 6: cargo antigen fix

**Status**: future; analogous to `cargo fix` for clippy lints.

`cargo antigen fix` automatically applies recommended changes for unaddressed
presentations:

```sh
$ cargo antigen fix --conservative

Found 7 unaddressed presentations:

  Applying conservative fixes (only when no risk of breakage):
    - 3 sites: added #[antigen_tolerance(...)] with TODO marker (manual review needed)
    - 2 sites: added #[immune(X, witness = ...)] with auto-generated stub
    - 2 sites: skipped (require manual decision; printed for review)
```

Modes:
- `--conservative` — only apply fixes that can't break anything; remaining
  sites flagged for human review
- `--aggressive` — apply auto-generated stubs for all sites; user must verify
  each manually
- `--dry-run` — show what would be applied without modifying files

Mirrors `cargo fix` behavior. Useful in CI for "auto-pr the fixes for review."

---

## Phase 7: Build-time integration (#[deny])

**Status**: future; depends on cargo + lint-system extensibility.

Antigen markers become first-class lints that can be denied in `Cargo.toml`:

```toml
[package.metadata.antigen]
adr_registry = "docs/decisions.md"
required = ["FrameTranslation", "BoundaryViolation"]
deny_unaddressed = true   # build fails if any presentation lacks immunity
```

Implementation:
- `cargo antigen scan --strict` already exits non-zero on unaddressed
  presentations (Phase 1); CI integration uses this
- Phase 7 adds compile-time enforcement: a build script that runs
  `cargo antigen scan --strict` before `cargo build` proceeds, OR an
  attribute macro that emits compile errors directly
- The `deny_unaddressed` flag in Cargo.toml is the user-facing knob

---

## Phase 8: Cross-tool delegation

**Status**: future; tools-author engagement required (per Phase 4 of adoption pathway).

The witness mechanism delegates to external tools. Phase 8 builds adapters:

| Witness type | Adapter |
|--------------|---------|
| `witness = test_function` | Read `#[test]` attribute, optionally run via cargo test |
| `witness = proptest!` | Detect proptest! macro invocation, optionally run |
| `witness = clippy::lint_name` | Verify lint exists; run clippy on the immunity site |
| `witness = kani::proof_function` | Detect `#[kani::proof]`, optionally invoke kani |
| `witness = prusti::trusted` | Detect `#[prusti::trusted]` and `#[prusti::proven]` |
| `witness = verus::proof` | Detect `proof fn`, optionally invoke verus |
| `witness = mutants::no_missed` | Run cargo-mutants on the immunity site, verify all caught |
| `witness = phantom_proof::TypeName` | Detect type construction, verify it compiles |

Each adapter is a small focused implementation. Land them as adopters request
them. The witness-pluralism design ensures that adding new adapters doesn't
break existing antigens.

---

## Phase 9: Cross-crate antigen versioning

**Status**: future; ADR draft pending.

When `antigen-stdlib` ships v1.0 with antigen `X` and v1.1 refines `X`'s
fingerprint, what happens to consumers of v1.0 who declared immunity?

Options:
- **Strict pinning**: consumers must explicitly upgrade; old fingerprints
  remain enforceable
- **Opt-in upgrade**: cargo antigen audit suggests upgrade path; consumer
  decides
- **Auto-upgrade with warnings**: consumers get the new fingerprint but old
  immunity claims are re-validated; broken claims surface

The semver discipline for antigen libraries is novel territory. A future ADR
ratifies the policy. Coordinated with `cargo-semver-checks` (one of the seed
antigens leverages this tool).

---

## Other extensions not in the main arc

### Antigen libraries beyond antigen-stdlib

Domain-specific antigen crates: `tokio-antigens` (async-Rust failure-classes),
`bevy-antigens` (game-engine patterns), `tambear-antigens` (math-library
patterns). The antigen-rs github org may host community-maintained ones; others
live independently.

### Observability integrations

Antigen audit results feeding into project observability:
- Prometheus metrics: count of unaddressed presentations over time
- Dashboards: coverage trend per family; high-risk antigens with low coverage
- Alerts: regression in immunity coverage on main branch

### Documentation generation

`cargo antigen docs --output html` generates a static site documenting:
- All antigens declared in the workspace
- Coverage statistics
- Cross-references between antigens, ADRs, and presentations
- Links to underlying tools (clippy lints, kani proofs, etc.)

This is what would let an enterprise team produce an audit-trail report for
compliance purposes.

### Multi-language support (v3+)

The fingerprint grammar (ADR-010) v1 is Rust-syntax-shaped via syn. ADR-010's
deferred questions include tree-sitter integration for cross-language support.
v3+ might extend antigen to TypeScript, Python, Go — wherever the failure-class
memory pattern is valuable.

This is far future. Rust-only is the right scope for v0.x and probably v1.x.

---

## How to use this document

The team's first sweep (Sweep A1) doesn't need to address most of this. Phase
2's audit work IS in scope for sweep A2-A3. Phase 3+ work surfaces in later
sweeps as the substrate matures.

When a contributor proposes a feature, the team can map it to a phase here:
- "Is this Phase N work?" — if yes, follow the sweep priorities
- "Does this jump phases?" — usually no; pre-requisites matter
- "Is this in the document?" — if not, propose adding it

The phases are not constraint; they're a default ordering for amplifying
value. The team can re-sequence based on adoption needs.

---

## References

- [`docs/decisions.md`](../decisions.md) — ratified ADRs (especially ADR-010
  fingerprint grammar)
- [`docs/expedition/case-study-determinism-class.md`](case-study-determinism-class.md) — illustrates Phases 2-4 in worked example
- [`docs/expedition/risk-register.md`](risk-register.md) — what could kill each phase
- [`docs/expedition/first-sweep-plan.md`](first-sweep-plan.md) — Sweep A1's
  current scope
- [`docs/vision-pitch.md`](../vision-pitch.md) — the public-facing adoption pathway
