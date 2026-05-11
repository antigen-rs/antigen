# Pre-Release Onboarding Sweep — Proposal

**Status**: Substrate proposal (2026-05-11). Awaiting aristotle Phase 1-8
scope-lock + team ratification of formal sweep name + work distribution.

**Authored**: team-lead, in conversation with Tekgy, after concrete-review
pass on user-facing artifacts surfaced doc-gaps that warrant sweep-tier
work before any tag.

**Core posture**: Tekgy's framing — "treat the onboarding as required
before any tag or rc." Onboarding is a structural commitment, not a
preference. Best-in-class, not merely sufficient. We ship welcoming
first-encounter, not "we'll fix docs in patch releases."

---

## Why this sweep exists

The architecture is good. The tooling works (235 tests passing across
A3 closure, scan/audit are real, witness-tier gradient honest). The
substrate is rich. What's missing is the user-facing surface that lets a
new adopter browsing crates.io install antigen, follow a path of decreasing
friction, and successfully use every capability that's been built.

The concrete-review pass surfaced these gaps:

1. **No "your first 10 minutes" tutorial.** README jumps to quick-start
   code without narrative walkthrough.
2. **Fingerprint grammar undocumented in any user-findable place.** Six
   operators + composition exist; their semantics live in source and ADRs,
   not in user-facing reference.
3. **Output formats undocumented.** scan/audit produce structured output;
   the format isn't shown in user-facing material; JSON schema isn't
   described.
4. **Crate-level doc-comments unverified.** Pathmaker's commits include
   doc-comments but no comprehensive review-against-cargo-doc has been
   done.
5. **basic.rs cross-reactivity** — `DemoBrokenWitness` fires fingerprint
   matches on `PanickingInDrop`-shaped code in the same file. Either fix
   the example or explicitly teach the cross-reactivity as a feature.
6. **CLI shows `new` and `vaccinate` as "design phase."** First-time users
   see "not yet implemented" — friction at the first encounter.
7. **No public roadmap.** Users can read about future sweeps in
   `sweeps/` but no concise "here's what's coming, here's when" surface
   exists for adopters deciding whether antigen will fit their needs over
   time.
8. **`references` field underdemonstrated.** Open-vocabulary cross-
   references are the bridge to knowledge-ecosystem (Component 4 of
   multi-component immunity); users need examples showing what shapes work.
9. **Where antigens live in a project** undocumented. Single-file in
   examples; `src/antigens.rs` in tambear; no recommended convention.
10. **Troubleshooting absent.** No "what does this error mean / what do I
    do" reference.

Per Tekgy's framing, these are not blockers we ship around. They are
prerequisites to tag.

---

## Scope

**In scope**:

1. README revision (working quickstart, real output shown inline,
   multi-component-immunity framing weaving)
2. Crate-level doc-comments review and improvement (cargo doc renders
   cleanly across all four crates: `antigen`, `antigen-macros`,
   `antigen-fingerprint`, `cargo-antigen`)
3. Tutorial: `docs/tutorial.md` (or equivalent location aristotle's
   judgment) — narrative walkthrough of first 10-15 minutes
4. Reference docs:
   - `docs/fingerprint-grammar.md` — six operators, composition semantics
   - `docs/witness-tiers.md` — what each tier means, when each applies
   - `docs/output-formats.md` — scan/audit output (human-readable +
     JSON schema)
   - `docs/macros.md` — full reference for each of the five macros
5. Examples directory expansion:
   - Fix or document basic.rs cross-reactivity
   - Add: descended_from inheritance example
   - Add: antigen_tolerance example
   - Add: phantom-type witness example
   - Consider: simple real-world failure-class example
6. CLI: hide `new` and `vaccinate` from help until they ship
7. Roadmap: `docs/roadmap.md` — concise user-facing what's-coming surface
   (separate from internal sweep planning in `sweeps/`)
8. Cargo.toml metadata audit (descriptions, keywords, categories)
9. CHANGELOG.md accuracy verification against actual v0.1.0-rc.1 substrate
10. usage-patterns.md continuation (scout already seeded)
11. Code verification pass — every claim in docs checked against actual
    code; every example compiles; every command works; every output shown
    matches reality

**Out of scope** (deferred, named explicitly):

- Migration guides (premature; no breaking changes to migrate from)
- Full cookbook beyond usage-patterns.md seed (will grow organically)
- IDE integration documentation (A6 territory)
- Cross-crate / antigen-stdlib user-facing docs (A4-A5 territory)
- Internationalization (post-v1.0)
- Comparison-against-other-tools detailed treatment (vision-pitch.md
  carries the floor; deeper comparison is post-v0.1)
- Marketing material / blog posts (separate from canonical docs)

**Sweep boundary clarification**:

This sweep produces the canonical user-facing substrate that ships at
v0.1.0-rc.1. It does NOT produce ongoing marketing, ecosystem evangelism,
or post-adoption support material. Those are separate work-streams that
come after tag.

---

## Verification criteria — what "done" looks like

For each deliverable, the verification criterion names a substrate-grounded
check (per A1 verification-protocol discipline):

1. **README quickstart**: `cargo install --path cargo-antigen && cd
   /tmp/test && cargo new test-antigen && cd test-antigen && (apply README
   quickstart steps) && cargo antigen scan` produces the documented output
   from a clean shell.
2. **Crate-level doc-comments**: `RUSTDOCFLAGS="-D warnings" cargo doc
   --workspace --no-deps --document-private-items=false` produces clean
   docs.rs-equivalent output with no missing-doc warnings on public
   surface.
3. **Tutorial**: a person who has never seen antigen can follow the
   tutorial start-to-finish and reach a working antigen declaration +
   passing scan + passing audit in under 15 minutes. (Empirically verified
   by team-member-not-yet-deep-in-substrate following the tutorial fresh.)
4. **fingerprint-grammar.md**: every operator listed in the doc is
   demonstrated with an example that, when run through the fingerprint
   engine, matches the documented set of inputs. Composition operators
   demonstrated similarly.
5. **witness-tiers.md**: each tier described with code example + expected
   audit output; code examples actually produce the expected tier when
   audited.
6. **output-formats.md**: every field in scan/audit output enumerated
   with type + meaning; JSON schema validated against actual JSON
   produced by the tools.
7. **macros.md**: every public macro documented with full attribute
   syntax + example + what each attribute does; examples compile.
8. **Examples directory**: every example compiles; every example runs
   without unexpected errors; every example's claimed behavior verified.
9. **CLI hide**: `cargo antigen --help` does not list `new` or
   `vaccinate` in v0.1.0-rc.1.
10. **roadmap.md**: covers v0.2.0 (W6b body-level fingerprint),
    A4-A6 anticipated work, with substrate-grounded confidence intervals
    on each ("planned for v0.2 / planned for A4 / aspirational A6+").
11. **Cargo.toml metadata**: `cargo publish --dry-run` succeeds for all
    four crates; description, keywords, categories appropriate for
    crates.io discovery.
12. **CHANGELOG**: every claim of "shipped in v0.1.0-rc.1" verified
    against `git log v0.1.0-rc.1 --oneline` (once tag lands) or against
    current main if pre-tag.
13. **Code verification pass**: a final review checks each user-facing
    doc against current code state; substrate-currency at the
    documentation tier.

---

## Proposed role distribution

(Aristotle Phase 1-8 may revise; this is the starting proposal.)

**Pathmaker** (implementation-domain owner):
- Crate-level doc-comments review + improvement across all four crates
- Verify every code-related claim in user-facing docs matches actual
  behavior
- basic.rs disposition (fix OR explicitly-teach the cross-reactivity)
- Hide CLI subcommands `new` and `vaccinate` (simple clap modification)
- Examples directory expansion (descended_from, antigen_tolerance,
  phantom-type)

**Scout** (exploration / pattern-mapping):
- Tutorial draft (`docs/tutorial.md`)
- `fingerprint-grammar.md` reference
- `usage-patterns.md` continuation (already seeded)
- Initial code-verification pass on tutorial + reference docs

**Aristotle** (rigor / governance):
- Phase 1-8 the sweep scope-lock (this proposal)
- Phase 1-8 canon-tier docs before tag (README, tutorial, reference docs)
- Final review pass for completeness/coherence before tag

**Adversarial** (failure-mode / gap-detection):
- Examples that safely demonstrate failure-modes (broken_witness.rs is
  the template)
- Gap-check pass: "what could a new user encounter that breaks the
  promised behavior?" — file as encounter-candidates
- Verify CHANGELOG accuracy against actual substrate
- `troubleshooting.md` or equivalent (errors users will see + what to
  do about them)

**Naturalist** (biology cognate / story):
- Continue C1-C7 cognate refinements at cadence (still pending from
  earlier work)
- Contribute biology framing in tutorial where it surfaces (without
  forcing — the tutorial's primary audience is Rust developers, not
  biology students)

**Me (team-lead)**:
- README revision (with multi-component-immunity framing extended in,
  per yesterday's substrate)
- `scope.md` + `vision-pitch.md` weaving (multi-component framing
  extension)
- `docs/roadmap.md` (concise user-facing what's-coming)
- `docs/witness-tiers.md` (overlaps with my scope.md work; can absorb)
- `docs/output-formats.md` (or delegate to pathmaker)
- `docs/macros.md` (or delegate to pathmaker)
- Coordination + final review pass + ratification gating

**Navigator**:
- Sweep coordination (work distribution, substrate-currency at
  documentation tier, deferred-substrate.md maintenance)
- Final substrate-grounded check that every deliverable's verification
  criterion is met before signaling sweep-close

---

## Sequencing

**Phase 1: Scope-lock + foundation** (1-2 sessions)
- Aristotle Phase 1-8 this proposal; team ratifies sweep scope
- Pathmaker: hide `new`/`vaccinate` in CLI (simple, unblocks tutorial work)
- Team-lead: README rough draft
- Scout: usage-patterns.md continued

**Phase 2: Core docs in parallel** (2-4 sessions)
- Pathmaker: crate-level doc-comments + verify-against-code
- Scout: tutorial + fingerprint-grammar.md
- Team-lead: README refinement + scope.md/vision-pitch.md weaving
- Adversarial: troubleshooting.md + gap-check
- Examples directory expansion (pathmaker + adversarial pair)

**Phase 3: Reference docs in parallel** (2-3 sessions)
- Team-lead or pathmaker: witness-tiers.md, output-formats.md, macros.md
- Scout: review tutorial against fingerprint-grammar.md for coherence
- Naturalist: cognate-refinements continue at cadence; integrate where
  helpful

**Phase 4: Roadmap + metadata** (1 session)
- Team-lead: `docs/roadmap.md` (with input from scout on A4+ substrate)
- Adversarial + pathmaker: Cargo.toml metadata audit
- Adversarial + pathmaker: CHANGELOG verification

**Phase 5: Final review pass + sweep close** (1-2 sessions)
- Aristotle: canon-tier review of all docs against verification criteria
- Adversarial: gap-check pass; surface any missing user-experience
  cases
- Navigator: substrate-grounded check that each deliverable's criterion
  is met
- Team-lead: ratify sweep close
- Tag v0.1.0-rc.1

**Total estimate**: 7-12 sessions of focused work depending on depth
and how parallel the phases run. Genuine quality-tier work, not rush-job.

---

## Sweep name + location

This proposal lives at `docs/expedition/` pending formalization. When
aristotle Phase 1-8s and the team ratifies, the sweep should formalize at
`sweeps/<name>/scope-lock.md` per existing convention.

**Naming candidates**:

- **A3.5** — preserves sweep-number sequence; signals "between A3 and A4"
- **OR-1** — "Onboarding-Readiness 1"; signals release-readiness focus
- **Pre-RC** — "Pre-Release Candidate"; descriptive of timing
- **Onboarding** — descriptive; could be A3.5 internally with
  "Onboarding" as friendly name

Aristotle's call. The team should pick what fits the existing sweep-naming
discipline; `A3.5` has nice continuity with A3 which it follows; `OR-1`
opens a numbering pattern for future release-readiness sweeps.

---

## Risks / what could go wrong

1. **Scope creep**: best-in-class can drift to "comprehensive forever";
   the verification criteria are the floor. Resist additions beyond
   what's named in-scope until v0.1.0-rc.1 ships.
2. **Doc-drift during sweep**: code evolves while docs are being
   written. Verification pass at end + substrate-currency discipline
   throughout.
3. **Theatrical examples**: examples that compile but don't actually
   demonstrate the failure-class they're for (ATK-A3-011 class).
   Adversarial discipline on examples.
4. **Tutorial that requires undocumented prerequisites**: assume reader
   has Rust experience but NOT antigen experience. Tutorial verification
   criterion (someone-fresh-can-follow-it) is the check.
5. **Roadmap promising too much**: A4-A6 work is real but plans evolve.
   Roadmap should use substrate-grounded confidence intervals; no firm
   dates beyond what's actually committed.
6. **CLI hide reverting after rc.1**: when `new` / `vaccinate` ship,
   they need to surface in `--help` correctly. Track as deferred-
   substrate entry.

---

## Connection to existing substrate

This sweep operationalizes substrate we've already produced:

- **Multi-component immunity V1**: README + scope.md + vision-pitch.md
  weaving is where the multi-component framing lands user-facing.
- **Scout's usage-patterns.md seed**: continuation in this sweep.
- **Aristotle's Q7 process.md sub-section** (encounters discipline): may
  inform how user-facing docs reference encounters when relevant.
- **Naturalist's cognate refinements**: feed tutorial biology-cognate
  framing where helpful.
- **Adversarial's ATK contracts**: inform troubleshooting + examples-of-
  failure-modes.
- **Aristotle's ADR-018 prose-drift catches**: same discipline applied
  to user-facing docs.
- **deferred-substrate.md**: tracks active sweep work + post-sweep
  follow-up items.

This is not a separate stream; it's the integration point where
everything we've built becomes user-facing.

---

## Acknowledgment

Authored 2026-05-11 by team-lead after Tekgy's directive that onboarding
is required-before-any-tag, not optional. The directive came from
recognizing that the architecture is good, the tooling works, and
shipping with welcoming first-encounter is consistent with project
values rather than "ship and patch."

Subject to revision via aristotle Phase 1-8. Open for team refinement
of scope, sequencing, role distribution, and naming.

*The architecture is real; the tooling works; the substrate is rich.
What ships at v0.1.0-rc.1 is the welcoming first encounter of all of
it.*
