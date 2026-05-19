# Tambear adoption log

> Living document tracking every antigen-related thing tambear does as it
> adopts, uses, refines, or removes antigen functionality. Each entry
> documents: why it was tried, what was done, whether it worked, what got
> removed if it didn't help, and what problem each thing solved.
>
> This log serves both projects:
> - **Tambear**: a record of integration decisions for context when later
>   sessions revisit them.
> - **Antigen team**: real-world adoption feedback to inform the project's
>   priorities.
>
> Entries are append-only by convention; the most recent appears at the bottom
> of each section. Markdown headings name what changed; body explains why,
> what, and how it played out.

---

## Conventions

Each entry has the shape:

```markdown
### [YYYY-MM-DD] Short descriptive title

**Why**: motivation for trying this — what problem were we trying to solve?

**What**: concrete change. File paths, code snippets, command invocations.

**Result**: did it work? What happened?

**Verdict**: keeping it / refining it / removed it because <reason> / not yet
sure (revisit by date).

**Lessons**: what the antigen team should know from this experience.
```

Keep entries brief but specific. The point is to make experience replayable
and for the antigen team to extract patterns across many projects' adoption
logs eventually.

---

## Entries

### [2026-05-07] First integration: path dep + antigens.rs module

**Why**: Validate that antigen-the-tool actually works against tambear's real
codebase, not just toy examples. Tekgy + Claude wanted to be the first user
during the pre-team scaffolding so we could:
- Surface adoption friction before the antigen JBD team launches
- Generate real performance data (how slow is scan on a real workspace?)
- Provide feedback on output ergonomics
- Demonstrate the inheritance-from-tambear.md "Phase 2 reciprocity" arc isn't
  hypothetical

**What**:

1. Added `antigen = { path = "../../../antigen/antigen" }` to
   `R:/tambear/crates/tambear/Cargo.toml`.
2. Created `R:/tambear/crates/tambear/src/antigens.rs` with two antigen
   declarations: `PolarityInvertedClassMeet` (the originator failure-class
   from GAP-BIT-EXACT-1) and `PanickingInDrop` (a stdlib-seed-catalog
   instance).
3. Registered the module via `pub mod antigens;` in `lib.rs`.
4. Ran `cargo antigen scan --root R:/tambear/crates` and `cargo antigen audit`.

**Result**:

- Tambear `cargo check` clean in 1.7s incremental.
- `cargo antigen scan` finds both declarations in 0.3s (217 files scanned).
- `cargo antigen audit` reports 0 immunity claims (because no `#[immune]`
  declarations exist yet — the antigens are declared but not yet protecting
  any code).
- No tambear class enum currently has a `meet` method, so the
  polarity-inverted-class-meet fingerprint correctly doesn't fire on any
  tambear code — confirming the antigen is for prospective protection.

**Verdict**: keeping it. The integration is minimal, reversible, and proves
the inheritance-arc is operational.

**Lessons**:

1. **Performance is not a barrier**: 217-file scans in 0.3s is well within CI
   budgets. The 11s first-run is dominated by `cargo-antigen` release-mode
   compile; subsequent scans are fast.
2. **5-step integration was straightforward**: Cargo.toml dep + module file +
   `pub mod` + `cargo check` + scan. No surprises.
3. **The "no setup" scan was already useful**: even before adding ANY
   declarations, running scan against tambear gave us "0 declarations across
   216 files in 0.3s" — establishing baseline performance and confirming
   the tool walks real-world code without errors.
4. **Output ergonomics asymmetry**: scan's brief summary works for clean
   workspaces; audit's diagnostic-rich output is more useful when something
   needs attention. The antigen team should consider making scan more
   diagnostic-rich for the "all clean" case.
5. **Honest no-bug finding is correct behavior**: tambear's class enums
   don't have meet methods, so the polarity-inverted antigen doesn't fire.
   This is the discipline working — the tooling doesn't generate false
   positives just because a class enum exists.

---

## Future entries

The following are placeholder slots for upcoming work. As the team takes
each on, the slots get replaced with real entry text.

### [2026-05-07 (later)] Third antigen + first real cleanup driven by antigen discipline

**Why**: ULP-CANON-1 spot check (item 2 from the post-DEC-029 roadmap)
discovered that despite the original 8 sign-magnitude re-implementation sites
being cleaned up in 2026-04-25, **two NEW sites had emerged in 2026-05-06
expedition work** (commits `f53bc1c` chains-E/F/G and `8c0555f`
sweep_29c_kappa_oracle). Each was a 7-line hand-rolled Dawson 2012 ULP
distance function, complete with the signed-zero sign-flip mechanics.

The pre-existing pattern detector at `tambear-substrate/src/pattern.rs` only
catches the inline single-expression form (`($A.to_bits() as i64).wrapping_sub(...)
.abs()`); it missed the multi-statement function form because those don't
match the same AST shape.

This is exactly the failure-class adversarial gardened about: "corrected
designs don't carry the failure that motivated them." Even with:
- The canonical `oracle_compare::ulp_distance_f64` existing
- The 8 prior sites cleaned up
- A pattern detector specifically for this pattern
- The lesson documented in roadmap-after-dec029.md

...two new sites STILL re-rolled the function from scratch. Different agents,
different contexts, different sweeps.

**What**:

1. Added a third antigen `UlpDistanceRolledByHand` to
   `crates/tambear/src/antigens.rs`:
   - Family: `forgotten-lesson` (the meta-pattern of lessons re-emerging)
   - Fingerprint: documented (multi-statement function form NOT covered by
     pattern.rs)
   - References: ULP-CANON-1 (original cleanup), `f53bc1c`, `8c0555f`,
     `tambear-substrate/src/pattern.rs`, Dawson 2012
2. Replaced the hand-rolled function in
   `crates/tambear/tests/dd_subnormal_sweep_oracle.rs` with a thin delegation
   to the canonical: `tambear::primitives::oracle_compare::ulp_distance_f64`
3. Same replacement in `crates/tambear/tests/sweep_29c_kappa_oracle.rs`
4. Verified both files' tests still pass (7 tests + 4 tests) — the canonical
   produces identical results for these test inputs

**Result**:

- `cargo test --workspace --lib` still passes 1397 tests
- `cargo antigen scan` against tambear finds 3 declarations (was 2)
- The 2 hand-rolled functions are gone; both files use the canonical
- ULP-CANON-1 is now actually complete (8 original sites + 2 newly-discovered
  sites all cleaned up)
- The `UlpDistanceRolledByHand` antigen now structurally documents the failure
  pattern, including the meta-finding that the pattern.rs detector has known
  blind spots (multi-statement function form)

**Verdict**: keeping it. This is the discipline working. The cleanup itself
wasn't dramatic (each replacement was 7 lines → 1 line), but the fact that
the failure-class re-emerged across sweeps is itself the antigen's
load-bearing case.

**Lessons**:

1. **The ULP-CANON-1 "complete" status from 2026-04-25 was actually
   incomplete-as-of-2026-05-07** because new sweeps reintroduced the pattern.
   This validates the antigen-stdlib argument that **failure-class memory
   needs to live in code, not in commit messages or roadmap items**. A
   commit-message-tracked cleanup gets reintroduced; a structural antigen
   doesn't (because `cargo antigen scan --hunt` would have flagged the new
   sites at PR time).
2. **Pattern detectors with blind spots create false confidence.** The
   `tambear-substrate/src/pattern.rs` pattern set covers inline forms but
   misses multi-statement function forms. The team thought ULP-CANON-1 was
   covered structurally; it was only partially covered. This is a Phase 3
   future-extensions.md item: fingerprint hunting needs to handle structural
   variants of the same pattern, not just specific expressions.
3. **The antigen-driven cleanup added net structural value beyond just the
   line replacement**: the antigen declaration now serves as documentation
   for any future contributor, with explicit references to all known sites
   AND the meta-finding about pattern.rs's blind spots. This is more durable
   than a commit message.
4. **Antigen team feedback**: the structural-fingerprint grammar needs to
   handle "function whose body matches pattern X" not just "expression
   matching pattern X." This is ADR-010's v1 grammar limitation; future
   versions should compose patterns over function bodies. (TODO(team) marker
   added to relevant scan code.)

[antigen team note: pattern-over-function-body is a real feature gap;
consider adding to ADR-010 amendment proposal for v2 grammar.]

### [2026-05-07 (later still)] PanickingInDrop and PolarityInvertedClassMeet: prospective protection only

**Why**: After the UlpDistanceRolledByHand cleanup, completed the audit pass
for the OTHER two declared antigens — looking at where in tambear they would
apply today.

**What**: 
- Grepped tambear for `impl Drop` blocks across all 7 workspace crates
- Confirmed earlier finding that tambear's class enums (DeterminismClass,
  FiniteClass, NyquistClass, PdeClass) don't have meet methods

**Result**:

- **Zero `impl Drop` blocks** anywhere in tambear's codebase. Tambear is
  pure-functional math; types aren't holding RAII resources, file handles,
  or external state that needs Drop cleanup. So PanickingInDrop has no current
  surface to fire on.
- Same finding for PolarityInvertedClassMeet (already documented earlier):
  no class enums currently have meet methods.

**Verdict**: keeping both antigens as declared, even though they don't fire
today. They serve **prospective protection**: when someone later introduces a
type that holds a Mutex/File/external handle (Drop impl) or a class enum
with strength-class lattice (meet method), they'll inherit the structural
memory of these failure-classes without having to remember the original bugs
or read tambear's history.

**Lessons**:

1. **Antigens that don't fire today are still doing work**: they make
   future-additions inherit the discipline. The 2 declared-but-not-firing
   antigens cost nothing at runtime, take ~50 lines of declaration each, and
   immunize all future contributors against the patterns. This is exactly
   what the inheritance-from-tambear.md "Future reciprocity" section
   anticipated: tambear becomes a teaching surface for failure-class memory
   that propagates beyond the current codebase.
2. **The full picture of A's findings**:
   - PolarityInvertedClassMeet: prospective (4 class enums exist; no meet
     methods today)
   - PanickingInDrop: prospective (0 impl Drop blocks today)
   - UlpDistanceRolledByHand: ACTIVE (2 sites found and cleaned up; future
     re-rolls would be flagged)
3. **The real-world adoption story is a mix**: 1 active antigen + 2
   prospective antigens. This is honest. Many production projects will look
   like this — most antigens prevent future bugs; a few catch current ones.
   The mix is fine; the substrate is doing its job whether the antigen fires
   today or in 2 years.

[antigen team note: when designing v0.1 docs, be explicit that "antigen
doesn't fire" is GOOD news for the consumer's codebase, not a sign the
antigen is useless. The educational/prospective value is real.]

### [2026-05-07 (later still 2)] First `#[immune]` declarations + witness functions

**Why**: Up to this point, tambear had 3 declared antigens but 0 immunity
claims. `cargo antigen audit` found nothing to validate because nothing was
asserting protection. To exercise the discipline end-to-end, we needed at
least one immunity claim with a real witness.

**What**:

1. Added `use antigen::immune; use tambear::antigens::UlpDistanceRolledByHand;`
   at file-scope in both `tests/dd_subnormal_sweep_oracle.rs` and
   `tests/sweep_29c_kappa_oracle.rs`.
2. Applied `#[immune(UlpDistanceRolledByHand, witness = ..., rationale = ...)]`
   to each file's `ulp_distance` wrapper function (the thin delegations to
   `oracle_compare::ulp_distance_f64` that replaced the original hand-rolled
   implementations).
3. Added `#[test]` witness functions in each file that exercise the
   across-zero killer assertion: `assert_eq!(ulp_distance(smallest_pos_denorm,
   smallest_neg_denorm), 2)` — the failure mode that hand-rolled
   re-implementations historically silently miss.

**Result**:

- `cargo test --workspace --lib`: 1397 tests pass (unchanged)
- `cargo test --test dd_subnormal_sweep_oracle`: 8 tests pass (was 7, +1 witness)
- `cargo test --test sweep_29c_kappa_oracle`: 5 tests pass (was 4, +1 witness)
- `cargo antigen scan --root R:/tambear/crates`: 5 antigen-related declarations
  (3 antigens + 2 immunity claims)
- `cargo antigen audit --root R:/tambear/crates`: 2 immunity claims, both
  structurally well-formed (witness identifiers resolved)

**Verdict**: keeping it. This is the discipline working end-to-end.

**Lessons**:

1. **Macro path matters for scan**: initially used `#[antigen::immune(...)]`
   (fully-qualified path) — scan didn't pick it up because the matcher uses
   `attr.path().is_ident("immune")` which requires a simple identifier. Fixed
   by adding `use antigen::immune;` at file-scope and using `#[immune(...)]`.
   This is a documentation gap: the antigen macros must be used as simple
   identifiers, not paths. Antigen team note: clarify in macro docs.
2. **Witness functions are best when adversarial**: the witness here doesn't
   just verify "the function returns something" — it specifically asserts
   the across-zero behavior that hand-rolled implementations historically
   fail. A weak witness (just `assert_eq!(ulp_distance(1.0, 1.0), 0)`) would
   pass even on broken re-rolls. The failing-as-passing pattern at the
   witness level: assert what the failure-class would NOT satisfy.
3. **Immunity declarations as documentation**: the rationale field on the
   `#[immune]` attribute documents WHY the immunity claim holds. Future
   readers (or re-roll attempters) see "this is a thin pass-through to
   canonical, and the witness verifies it" inline, without needing to consult
   external docs.

[antigen team note: macro-path scan limitation — `#[antigen::immune(...)]`
silently doesn't match. Either teach scan to handle path-form attributes,
or document the simple-identifier requirement prominently. Currently the
adoption-friction shows up as "audit reports 0 immunity even though
declarations exist."]

[antigen team note 2026-05-08, W7]: The two `#[immune(UlpDistanceRolledByHand)]`
declarations above — on `fn ulp_distance` in `dd_subnormal_sweep_oracle.rs`
and `sweep_29c_kappa_oracle.rs` — are the first real-world confirmation of
ATK-A2-005 (flat FunctionIndex ambiguity). Both files declare a function named
`ulp_distance`; pre-W7, the `or_insert_with` flat index would silently lose one
candidate, meaning one witness declaration would silently fail to resolve
correctly depending on filesystem walk order. W7's `WitnessStatus::Ambiguous`
fix (shipped in A2) means `cargo antigen audit` will correctly surface both
candidates as ambiguous when the user qualifies witnesses by bare name rather
than path. Lesson for tambear: if audit reports `WitnessStatus::Ambiguous` on
these witnesses, the fix is to qualify the witness path with a module prefix
(e.g., `witness = tests::ulp_wrapper_delegates_to_canonical_test`) so the name
is unambiguous across the workspace. Tracked in antigen as ATK-A2-005.]

### [2026-05-11] PanickingInDrop fingerprint silently failing — `&mut self` spacing bug

**Why**: During the antigen team's A3.5 onboarding sweep, scout was writing
`docs/fingerprint-grammar.md` and cross-checking all fingerprint examples
against the matcher source. The `render_inputs()` function in
`antigen-fingerprint/src/matcher.rs` uses `proc_macro2` tokenization to render
receiver arguments: `&self` renders as `"& self"`, `&mut self` renders as
`"& mut self"` (space between `&` and `self`/`mut` in both cases). This is
documented as ATK-W6a-013.

Tambear's `PanickingInDrop` declaration in `antigens.rs:88` used:
```
has_method("drop", "(&mut self)")
```
After whitespace normalization, this is `"(&mut self)"`. The matcher compares
against the rendered form `"(& mut self)"`. These are not equal — the
fingerprint was matching zero `impl Drop` blocks, silently.

**What**:

Fixed in antigen commit `7d9664a`:
```
has_method("drop", "(& mut self)")
```

**Result**:

- The fix is one character (space after `&`)
- There are zero `impl Drop` blocks in tambear today, so the behavioral
  impact was zero — the fingerprint was prospective protection only, and
  prospective protection that silently never fires is still no protection
- `cargo antigen scan --root R:/tambear/crates` now correctly evaluates the
  fingerprint against impl blocks (though it still finds zero matches, which
  is correct — tambear has no Drop impls)

**Verdict**: fixed. This category of silent mismatch is now documented in
`docs/fingerprint-grammar.md` under the "receiver spacing caveat" section,
with both `&self` → `"& self"` and `&mut self` → `"& mut self"` explicitly
called out.

**Lessons**:

1. **Silent fingerprint failures are the hardest class of fingerprint bug**:
   the scan runs, produces output, shows match counts — but the specific
   fingerprint you care about never fires. No error, no diagnostic, just
   structural protection that doesn't exist. This bug survived from the first
   tambear integration (2026-05-07) to the onboarding sweep (2026-05-11)
   because tambear has no `impl Drop` blocks to serve as a live check.
2. **Cross-checking docs against matcher source pays off**: the bug was found
   not by running the scan against code, but by reading the matcher's
   `render_inputs()` implementation while writing the grammar reference. The
   discipline of grounding docs in source rather than intuition is the check.
3. **Prospective antigens hide mismatch bugs longer**: an active antigen with
   real matches would have revealed this the first time a `has_method` check
   was expected to fire but didn't. Tambear's zero-Drop-impl state meant
   the mismatch was invisible.

[antigen team note 2026-05-11]: `docs/fingerprint-grammar.md` now documents
the receiver-spacing caveat explicitly. A future improvement would be for the
parser to normalize receiver token forms at parse time rather than requiring
users to know the `proc_macro2` rendering convention. Tracked as a known
paper-cut; no ADR yet.

### [2026-05-18] Doc-witness gap surfaced by discipline-shaped antibodies

**Why**: Tambear shipped three methodology patterns this day at the
*antibody-tier* of its methodology doc (`R:/winrapids/docs/expedition/session-methodology-patterns.md`):

- **Pattern 23** (five-type boundary taxonomy) — a *triage discipline* that
  routes special-function precision regressions through five named failure
  modes (Denominator-near-zero, Float-range overflow, Cancellation,
  Discontinuity, Combinatorial-explosion). Used by scientist when an oracle
  crossval reports a precision regression; the routing-language ("this is
  Type-1 — use Reformulation") becomes the audit trail.
- **Sub-pattern 5.10** (architecture-assumption antibody) — at anchor-construction
  time, *verify the specific function signature* of every dependency claimed
  in the anchor, not just that the named family exists. Example: an Airy
  anchor that claims "uses fractional-nu Bessel" must grep `bessel_i.rs` for
  `I_nu(f64)` not just confirm `bessel_i` exists. Used by math-researcher when
  writing anchor docs.
- **Sub-pattern 5.11** (anchor-stage value-check antibody) — at coefficient-anchor
  time, *verify each load-bearing constant through an end-to-end value test
  against an independent reference* (not just bit-pattern compile-tests against
  a transcribed table). Math-researcher's 343-line `tautological-antibody-scan.md`
  is the canonical instance: a retroactive audit of 7 shipped anchors against
  this discipline.

All three of these are checked *by humans against discipline docs at design/anchor
time*, not by code at test-execution time. They are real failure-class memory
(each was crystallized after operational evidence), but they cannot be expressed
in antigen's current witness shapes:

- `Test` / `Proptest` — these test *behavior*, not whether-the-discipline-was-applied
- `Function` — same as Test, with one indirection
- `PhantomType` — compile-time proof, but discipline application doesn't compile-fail
  if skipped; the antibody fires when a reviewer notices the omission, not when
  rustc rejects the code

Concretely: if we wanted to declare an antigen for "alternating-series-near-zero
premature termination" (the Type-1 boundary-taxonomy failure that produced the
2026-05-18 airy bug `b3fbb0c`), we could ship the antigen, but we couldn't
declare immunity on a new oscillatory-asymptotic recipe that *was vetted against
the discipline doc* but has no executable test exercising the failure mode at
the design-time check.

**What**:

Naturalist + navigator surfaced this gap during the 2026-05-18 antibody
crystallization arc:

1. Three antibody-tier patterns crystallized in one day, all operating at
   design/anchor/review time rather than test-execution time.
2. Math-researcher's tautological-antibody-scan (`R:/tambear/campsites/session-20260518/20260518123521-coordination/math-researcher/notebooks/` — specifically the retroactive 7-anchor audit) produced a doc-witness artifact: a written attestation that 7 specific recipes were checked against Sub-pattern 5.11 (anchor-stage value-check), with the check-and-result documented per recipe.
3. The team has no way to claim immunity declaratively in `#[immune(...)]` syntax that references this attestation doc.

Proposed shape (sketch — full proposal in adjacent FYI):

```rust
#[immune(
    AlternatingSeriesNearZeroPrematureTermination,
    witness = doc_attested(
        doc = "R:/tambear/campsites/.../tautological-antibody-scan.md",
        attested_by = "math-researcher",
        at = "2026-05-18",
        rationale = "audited under Sub-pattern 5.11 + Pattern 23 Type-1; \
                     dispatch shoulder verified Chebyshev-optimal at x=5; \
                     antibody battery includes envelope-max canary"
    ),
    rationale = "anchor doc + retroactive audit verify discipline application; \
                 no executable test exercises this at design-time because \
                 the failure-class is a design-discipline-omission, not a \
                 runtime behavior"
)]
fn ai_asym_positive(x: f64) -> f64 { /* ... */ }
```

Witness-tier mapping (per ADR-001 Amendment 1 Change 4): this would be a new
tier — call it **Attestation tier**, sitting parallel to Reachability /
Execution / Behavioral-alignment / Formal-proof. Audit verifies the doc path
exists + the attested_by/at fields parse + (optionally) the doc contains a
machine-readable section asserting the recipe is in-scope of the discipline.

**Result**:

Not yet implemented in antigen. The gap is named; the proposal is sketched.
Adjacent FYI doc at `R:/tambear/campsites/session-20260518/20260518123521-coordination/naturalist/insights/20260518-doc-witness-adr-proposal.md`
contains the full ADR-amendment draft (with mechanics, sweep-consequences,
enforcement, open-questions).

**Verdict**: not yet sure. The proposal is naturalist-authored; ratification
belongs to the antigen team. Three paths are visible:

1. **Add Attestation tier to ADR-001 Amendment 1 Change 4** and `WitnessKind::DocAttested`
   to ADR-013. Tambear's 5 antibody-tier patterns + math-researcher's anchor
   audit become declarable.
2. **Decline** — leave discipline-shaped antibodies outside antigen's scope; let
   them remain as methodology-doc patterns + commit-message citations.
   Tambear continues to operate without `#[immune]` claims on its methodology
   discipline.
3. **Extend `references` instead** — relax the gap by letting `references = [doc_path, ...]` carry the attestation, without a new witness type. Lower-friction
   but loses the *trust-boundary* discipline of attested_by/at (ADR-005 sub-clause F).

Naturalist's lean: Option 1, because tambear's antibody-tier patterns are
*structurally guaranteed to keep growing* (Sub-patterns 5.5/5.10/5.11 are
three instances in 4 days; the lane is open), and the team will keep needing
to express "this recipe was vetted against this discipline" declaratively.
ADR-007's anti-YAGNI says: if structure guarantees we'll need this, build it now.

**Lessons**:

1. **Discipline-shaped antibodies are a real witness shape**. Tambear's antibody
   tier of its methodology doc has 5+ patterns operating at this shape; the
   lane will only grow as the system matures. Antigen needs an expression for
   it OR an explicit decision that it's out-of-scope.

2. **The `references` field is load-bearing but not sufficient**. References
   point at the *origin* of the antigen's failure-class memory; they don't
   attest that *this specific recipe* was checked. Attestation is a different
   semantic surface — it carries the audit-trail discipline (who, when, against
   what doc, with what rationale).

3. **The witness-tier ladder is climbed by adding tiers above, not below**.
   Reachability → Execution → Behavioral-alignment → Formal-proof is a
   strictness ladder (each tier subsumes the one below). Attestation doesn't
   fit on this ladder cleanly — it's *orthogonal*, a different verification
   axis (human attestation vs mechanical verification). Either the ladder
   becomes a lattice, or Attestation becomes its own ladder.

4. **Self-application is the validation**. The math-researcher's anchor audit
   IS an instance of the proposed Attestation tier — it audits 7 specific
   anchors against Sub-pattern 5.11 and produces a doc. If antigen had
   `doc_attested(...)`, those 7 anchors would become immunity declarations
   referencing the audit doc. The audit doc IS the witness; we have the
   artifact but no syntax to claim immunity against it.

[antigen team note pending]: this adoption-log entry is naturalist-authored
2026-05-18 evening; the parallel FYI in the tambear naturalist insights folder
contains the full ADR-amendment draft. Co-design routing: this is the lane
where tambear's adoption *informs* antigen's grammar/design rather than just
*uses* it. The antigen team's call on Options 1/2/3 above shapes how tambear
proceeds with its methodology-tier substrate.

---

### [pending] Phase 1-8 deconstruction of `PolarityInvertedClassMeet`, `PanickingInDrop`, and `UlpDistanceRolledByHand`

(Aristotle thread, after JBD team launch. The antigens were declared without
formal Phase 1-8 review during pre-team scaffolding; same status as the
foundational ADRs.)

### [pending] First `#[presents]` markers in tambear

(Adding presentation markers on actual tambear code that exhibits known
failure-classes. Initially probably to demonstrate the workflow rather than
catch real bugs; later the markers become substrate for refactoring decisions.)

### [pending] First `#[immune]` declaration with witness

(Once a real witness pattern emerges — likely a proptest verifying a class
enum's invariant — declaring immunity. This is when audit becomes
operationally valuable.)

### [pending] Adding antigens for tambear-specific failure-classes

(Patterns that recur in tambear's sweeps may not be in `antigen-stdlib` yet;
they get declared in `crates/tambear/src/antigens.rs` first, then promoted to
`tambear-antigens` (a future crate) or contributed to `antigen-stdlib`.)

### [pending] CI integration with `--strict` flag

(Adding `cargo antigen scan --strict` to tambear's CI pipeline. Initially this
will be advisory-only; later, broken witnesses or unaddressed presentations
fail the build.)

### [pending] `#[descended_from]` propagation across tambear's recipe family

(Tambear has hundreds of recipes that share structural shapes. Once
descended_from propagation lands in antigen, tambear's recipes can declare
inheritance from canonical recipes, and immunity can propagate automatically.)

---

## Updates from antigen-team to this log

When the antigen JBD team takes actions on tambear's behalf (e.g., adapting
the scan output based on tambear's feedback, fixing bugs surfaced by tambear's
adoption), they should annotate entries here with cross-references to the
corresponding antigen-side commits or ADRs.

Example annotation format:

```
[antigen team note 2026-XX-XX]: addressed Lesson 4 (output ergonomics
asymmetry) in commit abc1234 by enriching scan output for the "all clean"
case. See ADR-NNN for the design rationale.
```

This builds the bidirectional feedback loop that makes the
"antigen-graduates-from-tambear" relationship work in practice.
