# Sweep A3.5 — Onboarding

> **Status**: Scope-locked (ratified 2026-05-11 by team-lead + Tekgy).
> **Owner**: team-lead (coordination) + navigator (substrate-currency).
> **Predecessor**: Sweep A3 (cross-crate scan + descended_from + multi-component framing).
> **Successors**: A4 (composition rules + body-level fingerprint grammar), v0.1.0-rc.1 tag.

---

## Posture

Tekgy's directive (2026-05-11): **onboarding is required before any tag or rc, not optional.**
Best-in-class, not merely sufficient. The architecture is good; the tooling works (235 tests
passing at A3 closure); the substrate is rich. What ships at v0.1.0-rc.1 is the welcoming
first encounter of all of it.

---

## Theme

Produce the canonical user-facing substrate that ships at v0.1.0-rc.1. Every capability
the project has built gets a path of decreasing friction that a new adopter browsing
crates.io can follow to a working antigen declaration + passing scan + passing audit.

---

## Scope

**In scope** (13 deliverables):

1. **README revision** — working quickstart, real output shown inline, multi-component-immunity
   framing woven in
2. **Crate-level doc-comments** — review + improvement across all four crates; renders cleanly
   with `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
3. **Tutorial** — `docs/tutorial.md`; narrative walkthrough of first 10-15 minutes for a Rust
   developer who has never seen antigen
4. **`docs/fingerprint-grammar.md`** — six operators + composition semantics with worked examples
5. **`docs/witness-tiers.md`** — each tier described with code example + expected audit output
6. **`docs/output-formats.md`** — scan/audit human-readable + JSON schema; every field enumerated
7. **`docs/macros.md`** — full reference for each of the five macros; attribute syntax + examples
8. **Examples directory expansion** — fix or document cross-reactivity in basic.rs; add:
   descended_from inheritance, antigen_tolerance, phantom-type witness examples
9. **CLI hide** — `new` and `vaccinate` hidden from `--help` until they ship (done: ce75896)
10. **`docs/roadmap.md`** — concise user-facing what's-coming with substrate-grounded confidence
    intervals
11. **Cargo.toml metadata audit** — descriptions, keywords, categories for crates.io discovery
12. **CHANGELOG.md accuracy verification** — every v0.1.0-rc.1 claim checked against actual
    substrate
13. **Code verification pass** — final review of every user-facing doc against current code state

**Out of scope** (explicitly deferred):

- Migration guides (no breaking changes to migrate from)
- Full cookbook beyond usage-patterns.md seed
- IDE integration documentation (A6)
- Cross-crate / antigen-stdlib user-facing docs (A4-A5)
- Internationalization (post-v1.0)
- Detailed comparison-against-other-tools (vision-pitch.md carries the floor)
- Marketing material / blog posts

---

## Scope-lock amendments (ratified)

Five amendments from aristotle Phase 1-8, ratified 2026-05-11:

1. **Phase 5 convergence-check discipline** — Phase 5 final review uses convergence-check
   methodology: cross-check outputs from multiple roles rather than independent parallel reads.
   Navigator coordinates the convergence step.

2. **Phase 1 dependency-mapping sub-task** — navigator runs an explicit dependency-mapping
   pass on all Phase 2/3/4 deliverables before Phase 2 begins. Produces: which deliverables
   depend on what substrate; which parallel, which sequential.

3. **Criterion #3 mechanism** — tutorial verification = team-member-fresh-to-tutorial-content
   (pathmaker reads scout's tutorial cold, per Criterion #3). Imperfection explicitly named:
   no team member is "new to antigen" in the same way a crates.io user is.
   New-Claude-instance flagged as a future-amendment-eligible mechanism (can follow tutorial
   from cold start without prior session context).

4. **Gap item #5 reframed** — `DemoBrokenWitness` is declared in
   `antigen/examples/broken_witness.rs` (not `basic.rs`). Its structural fingerprint fires
   matches at 7 sites in `antigen/examples/basic.rs` (lines 26, 53, 59, 69, 75, 89, 101).
   Proposal's attribution was declaration-location vs match-fire-location conflation —
   corrected. Cross-reactivity verified real (adversarial, 2026-05-11). Deliverable: describe
   what `cargo antigen scan` produces when run against both files, showing that a fingerprint
   declared in one file generates matches in another. Disposition decision (fix vs explicitly
   teach) lands during examples-directory work; both valid.

5. **V1 canonicalization timing** — Option B (coordinate inline during weaving). V1 stays in
   `docs/expedition/multi-component-immunity.md` during this sweep. The weaving deliverable
   (README + scope.md + vision-pitch.md) cites the expedition path with an explicit note:
   "framing currently maturing in expedition/; expected to canonicalize as
   `docs/multi-component-immunity.md` after team Phase 1-8 fully ratifies + outstanding
   refinements (C1-C7 cognates, engineered-substrate-exceeds-biology family, etc.) settle."
   Premature canonicalization would freeze substrate still actively producing refinements.

---

## Verification criteria

Substrate-grounded check per deliverable (per A1 verification-protocol discipline):

1. **README quickstart**: `cargo install --path cargo-antigen && cargo antigen scan` from a
   freshly-created crate following the README steps produces the documented output.
2. **Crate-level doc-comments**: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
   produces clean output with no missing-doc warnings on public surface.
3. **Tutorial**: team-member-fresh-to-tutorial-content (pathmaker reads scout's tutorial cold)
   reaches working antigen declaration + passing scan + passing audit in under 15 minutes.
4. **fingerprint-grammar.md**: every operator listed has an example that, when run through the
   fingerprint engine, matches the documented set of inputs.
5. **witness-tiers.md**: each tier has code example + expected audit output; examples actually
   produce the expected tier when audited.
6. **output-formats.md**: every field in scan/audit output enumerated with type + meaning;
   JSON schema validated against actual JSON produced by the tools.
7. **macros.md**: every public macro documented with full attribute syntax + example; examples
   compile.
8. **Examples directory**: every example compiles; every example runs without unexpected
   errors; every example's claimed behavior verified.
9. **CLI hide**: `cargo antigen --help` does not list `new` or `vaccinate`. **Verified**: done
   at ce75896.
10. **roadmap.md**: covers v0.2.0 (W6b body-level fingerprint), A4-A6 anticipated work, with
    substrate-grounded confidence intervals.
11. **Cargo.toml metadata**: `cargo publish --dry-run` succeeds for all four crates;
    description, keywords, categories appropriate for crates.io.
12. **CHANGELOG**: every v0.1.0-rc.1 claim verified against git log (or current main pre-tag).
13. **Code verification pass**: final review checks each user-facing doc against current code
    state; substrate-currency at documentation tier.

---

## Role distribution

**Pathmaker**:
- Crate-level doc-comments review + improvement
- Verify every code-related claim in user-facing docs matches actual behavior
- basic.rs cross-reactivity disposition (fix OR explicitly-teach)
- Examples directory expansion (descended_from, antigen_tolerance, phantom-type)
- Tutorial verification (reads scout's tutorial cold — Criterion #3 mechanism)

**Scout**:
- Tutorial (`docs/tutorial.md`)
- `docs/fingerprint-grammar.md`
- `docs/usage-patterns.md` continuation (already seeded at 2026-05-11)
- Initial code-verification pass on tutorial + reference docs

**Aristotle**:
- Phase 1-8 complete (this document is the output)
- Final review pass (Phase 5) for completeness/coherence before tag
- Canon-tier docs review against verification criteria

**Adversarial**:
- `docs/troubleshooting.md` or equivalent (errors users will see + remediation)
- Gap-check pass: "what could a new user encounter that breaks the promised behavior?"
- Verify CHANGELOG accuracy against substrate
- Examples adversarial discipline (no theatrical examples)

**Naturalist**:
- C1-C7 cognate refinements at cadence (idle-as-invitation)
- Biology framing contribution in tutorial where helpful (not forced)

**Team-lead**:
- README revision (with multi-component-immunity framing)
- `docs/scope.md` + `docs/vision-pitch.md` weaving
- `docs/roadmap.md`
- `docs/witness-tiers.md` and `docs/output-formats.md` (or delegate to pathmaker)
- `docs/macros.md` (or delegate to pathmaker)
- Coordination + final review + sweep close ratification

**Navigator**:
- Phase 1 dependency-mapping sub-task (amendment #2)
- Sweep coordination and substrate-currency at documentation tier
- `deferred-substrate.md` maintenance throughout sweep
- Phase 5 convergence-check coordination (amendment #1)
- Final substrate-grounded check that every verification criterion is met before sweep-close signal

---

## Sequencing

**Phase 1: Scope-lock + foundation** (COMPLETE)
- Aristotle Phase 1-8 — DONE (campsite `20260511-pre-release-onboarding-sweep-phase-1-8.md`)
- CLI hide (`new`/`vaccinate`) — DONE (ce75896)
- Scout: usage-patterns.md + where-to-look-for-antigens.md — DONE
- Adversarial: gap item #5 cross-reactivity verification — DONE (outcome a: 7 sites)
- Navigator: dependency-mapping pass — PENDING (starts after scope-lock formalization)
- Team-lead: README rough draft — IN PROGRESS

**Phase 2: Core docs in parallel** (2-4 sessions)
- Pathmaker: crate-level doc-comments + verify-against-code
- Scout: tutorial + fingerprint-grammar.md
- Team-lead: README refinement + scope.md/vision-pitch.md weaving
- Adversarial: troubleshooting.md + gap-check + examples adversarial discipline
- Examples directory expansion (pathmaker + adversarial pair)

**Phase 3: Reference docs in parallel** (2-3 sessions)
- Team-lead or pathmaker: witness-tiers.md, output-formats.md, macros.md
- Scout: review tutorial against fingerprint-grammar.md for coherence
- Naturalist: cognate-refinements continue at idle cadence; integrate where helpful

**Phase 4: Roadmap + metadata** (1 session)
- Team-lead: `docs/roadmap.md`
- Adversarial + pathmaker: Cargo.toml metadata audit + CHANGELOG verification

**Phase 5: Final review pass + sweep close** (1-2 sessions)
- Aristotle: canon-tier review against verification criteria
- Adversarial: gap-check pass
- Navigator: convergence-check + substrate-grounded verification of each criterion
- Team-lead: ratify sweep close
- Tag v0.1.0-rc.1

**Tekgy's meta-framing**: no rush; mix, weave, sequence as fits natural cadence. Phase 2
doesn't have to march in lockstep; flexibility is the discipline.

**Estimated total**: 7-12 sessions of focused work.

---

## Post-sweep deferred item

When `new` / `vaccinate` eventually ship (A5), they need to surface in `--help` correctly.
Registered in deferred-substrate.md as: "restore `new`/`vaccinate` to CLI help when A5 ships them."

---

## Risks

1. **Scope creep**: best-in-class can drift to "comprehensive forever." Verification criteria
   are the floor. Resist additions beyond what's named until rc.1 ships.
2. **Doc-drift during sweep**: code evolves while docs are being written. Verification pass at
   end + substrate-currency discipline throughout.
3. **Theatrical examples**: examples that compile but don't demonstrate the failure-class they're
   for (ATK-A3-011 class). Adversarial discipline on examples is the check.
4. **Tutorial requires undocumented prerequisites**: assume reader has Rust experience but not
   antigen experience. Criterion #3 mechanism is the check.
5. **Roadmap promising too much**: A4-A6 work is real but plans evolve. Substrate-grounded
   confidence intervals; no firm dates beyond what's committed.
6. **CLI hide reverting after rc.1**: tracked in deferred-substrate.md.

---

## Connection to existing substrate

- **Multi-component immunity V1** (`docs/expedition/multi-component-immunity.md`): weaving
  target for README + scope.md + vision-pitch.md (V1 stays in expedition/ during this sweep
  per amendment #5)
- **Scout's usage-patterns.md** (`docs/usage-patterns.md`): seeded 2026-05-11; continuation
  in this sweep
- **Scout's where-to-look-for-antigens.md** (`docs/where-to-look-for-antigens.md`): shipped
  2026-05-11; closes gap #9
- **Aristotle's Q7 process.md sub-section** (encounters discipline): may inform user-facing
  docs when encounters reference surfaces
- **Naturalist's cognate refinements**: feed tutorial biology-cognate framing where helpful
- **deferred-substrate.md**: tracks active sweep work + post-sweep follow-up items throughout

---

## Predecessor context

**Source proposal**: `docs/expedition/pre-release-onboarding-sweep-proposal.md` (team-lead,
2026-05-11; 339 lines; 11 deliverable categories).

**Aristotle Phase 1-8** at
`campsites/antigen-A3/20260509163016-20260509080000-launch/aristotle/20260511-pre-release-onboarding-sweep-phase-1-8.md`:
8 findings, 5 scope-lock amendments (incorporated above).

---

*Ratified: 2026-05-11.*
*Owner: team-lead (sweep lead) + navigator (substrate-currency).*
