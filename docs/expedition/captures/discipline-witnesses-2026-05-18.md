# Capture — Discipline Witnesses Design Conversation

> **Date**: 2026-05-18
> **Participants**: Tekgy + Claude (main-thread, in `R:\antigen\`)
> **Origin**: tambear (`R:\tambear\`) hit a doc-witness gap during v0.1-rc
> adoption attempt
> **Status**: append-only capture, lightly cleaned for formatting only

> **What this is**: high-fidelity record of the design conversation that
> produced the substrate-witness predicate family + Ratification schema +
> `cargo antigen attest` CLI shape. Includes the dead ends and the
> user-driven moves so next-session pickup can see how the shape evolved
> rather than re-derive it. The synthesized draft is at
> [drafts/discipline-witnesses-v1.md](../drafts/discipline-witnesses-v1.md).

---

## Origin — tambear-Claude surfaces the gap

Tekgy opened by sharing raw output from Claude-in-tambear:

> "A witness type for doc-attestation. Current witness types: proptest /
> clippy / kani / prusti / verus / phantom-type / test fn. None lets you
> say 'this code claims immunity from the antigen because an
> agent/human reviewed it against the linked discipline doc and attested
> compliance.'"
>
> Proposed macro shape from tambear-Claude:
> ```rust
> #[immune(MyDiscipline, witness = doc_attested(
>     doc = "R:/tambear/docs/architecture/signed-zero-discipline.md#sinh",
>     attested_by = "math-researcher",
>     at = "2026-05-18",
>     rationale = "Verified sinh uses expm1(|x|) with sign-copy-back per §3.2",
> ))]
> ```
>
> Tambear-Claude's proposed action: surface to navigator as "doc-witness
> is THE next antigen feature ask; tambear has standing to author the
> ADR proposal."

Tekgy explicit that this was raw output for routing through antigen's
discipline, not a directive: "we'll run it through our antigen
discipline, decisions, planning, practices, etc."

## Turn 1 — Claude substrate-check pushback

Substrate-checked before routing. Three load-bearing pieces fell out:

**1. `rationale` and `references` already exist** — carrying part of what
tambear-Claude named as a gap:

- `#[antigen(..., references = ["docs/.../signed-zero.md#sinh"])]`
  already attaches doc links (parse.rs:111)
- `#[immune(..., witness = ..., rationale = "...")]` already carries
  narrative justification (ADR-001 Am 1 Change 7, hardened into
  rationale-as-required-field by ADR-005 Am 2)
- Roadmap line 157 explicitly names "Multi-maintainer attestation for
  stdlib antigens — threshold signatures, signed declarations" as
  post-A5 territory

So the gap wasn't "no attestation primitive" — it was "no *structured*
attestation with first-class `attested_by` / `verified_at` /
`against_doc` fields, surfaceable to audit for stale-review detection."
More specific, more recognition-grounded.

**2. Putting doc-attestation in the witness enum at peer level with
`kani::` / `proptest::` / `phantom-type` would be a tier-honesty
violation waiting to happen.** ADR-005 Am 3 (audit-tier-honesty)
requires the audit to report actual verification strength performed,
never stronger. A doc-attestation reports "an agent asserted compliance
against doc X at date Y." The audit can verify the *fields* of that
assertion (presence, freshness, doc-link reachability) — it cannot
verify the *truth* of the attestation. Sits structurally *below*
`Reachability`.

Two recognition-grounded shapes that respect tier-honesty:
- **Sub-tier under witness**: extend `WitnessTier` with an `Attestation`
  tier below `Reachability`
- **Orthogonal attribute**: `#[attested(by, at, against)]` lives
  *alongside* `#[immune]`, not inside its witness enum

**3. The interim suggestion (phantom-type-as-stand-in for attestation)
is the tier-honesty violation ADR-005 Am 3 exists to prevent.** Faking
`witness = phantom_type` for an agent assertion would report
`FormalProof` tier for an attestation. Don't do that, even temporarily.

**4. Tambear is smoke-test, not design authority** (memory note
`feedback_tambear_is_smoke_test_not_design_input.md`). Tambear surfaced
the gap by hitting it. Antigen team designs the response. "Tambear-as-
originating-team should author the ADR" inverts the standing constraint.
Contribution is the report, not authorship.

Proposed sending to navigator: "tambear hit a smoke-test gap; structured
attestation doesn't compose into current witness enum without violating
tier-honesty; two candidate recognitions (sub-tier vs orthogonal);
recommend team Phase 1-3 deconstruction pass to decide
absorption-vs-addition before ADR text."

## Turn 2 — Tekgy: design-conversation mode, no team-spawning

> "exactly right. I was just giving raw output from you-in-tambear here
> so we can run it through our antigen discipline... there's SOMETHING
> that we could work on shipping, that may look nothing at all like what
> tambear is imagining. but the 'this is a discipline problem and we need
> a discipline antigen/immune system component + way to
> witness/resolve/whatever it' is real. we aspire to discipline issues
> as well as other issues. but what that looks like on either the
> declaration or the witness or other end is totally up to us. nothing
> needed to send back to tambear until we have a solution."

Reframe: not "where does doc-witness go in the witness enum" but **"what
does *verification* even mean for a discipline failure-class, and at what
abstraction layer should antigen recognize the answer?"** The witness-
enum shape was tambear-Claude's framing; can be dropped entirely.

Four candidate shapes sketched, ascending abstraction layer:

1. **Attestation tier under `WitnessTier`** — `FormalProof > Execution >
   Reachability > Attestation > None`. Smallest change. Creates laundering
   vector: once `witness = doc_attested(...)` exists in enum, gets used
   for things that should have had mechanical witnesses; "Attestation"
   tier becomes ambient cover for "we didn't really verify."
2. **Orthogonal `#[attested(by, at, against, expires)]` attribute** —
   lives *next to* `#[immune]`. Cleaner separation. Adds new attribute
   layer for a problem that may not need one.
3. **Declaration-side `#[antigen(..., kind = "discipline")]`** — antigen
   declares its verification model at declaration time. Audit treats
   discipline-antigens with different posture: NO immunity claim
   available; surfaces *attestation substrate* per presented site. Audit
   doesn't pretend to verify discipline; produces high-fidelity surface
   for reviewer.
4. **Refuse to encode immunity for discipline-antigens** — discipline
   failure-classes ARE expressible as antigens but `#[immune]` is
   structurally unavailable. Most rigorous about tier-honesty; reduces
   expressiveness.

Claude's lean: (3) is the right abstraction layer; (4) might be how (3)
operationally lands.

Open questions named:
- Does `kind = "discipline"` open slope to `kind = "performance"`,
  `kind = "security"`, etc.?
- For (3)/(4): what's the new audit state's exact shape?
- Roadmap.md:157's "multi-maintainer attestation" — same primitive at
  ecosystem scale?
- Naturalist check: does biology have a referent for "antigen the immune
  system *can't mechanically respond to, only document its presence
  and let cognition decide*"? Genuine question.

## Turn 3 — Tekgy: lifecycle + abstraction question

> "it would be good to think through if there's a way to mark it as
> handled or something, like if we pre-emptively mark a function with a
> discipline antigen that sort of injects the context of the linked
> doc/whatever to it, and scans/audits make sure it surfaces. but how
> can we later have it no longer show up in scans/audits, or how can
> someone mark that they have confirmed that the thing is handled, or
> etc. or can we force discipline to ALSO have signatures that are
> matchable. if it can't be represented in some useful way then we have
> 3/4 as a last ditch effort, but what CAN we do to abstract signatures
> for DISCIPLINES (coding disciplines, like we can't do this until that,
> or this function must use X method not Y method due to whatever, or
> must have X team member (imagine math-researcher here for example)
> ratify this, or can't consider this function as done regardless of
> tests until linked doc is ratified and team ratifies function, or
> whatever). like what CAN we do that is shows up as issue, then shows up
> as resolved/handled or whatever"

This was the reframe that broke the prior framing open. Claude's response:

**The reframe**: witness types currently only check *code-side*
substrate (function bodies, test execution, type-system proofs, lint
configuration). The discipline gap isn't "no Attestation tier" — it's
**witnesses don't yet check *non-code* substrate**: docs, ratification
records, approval chains, signed trailers, version pins. Once they can,
"discipline antigens" stop being a special category and become
*ordinary antigens whose witnesses point at substrate other than .rs
files*.

This preempts the laundering trap. No "Attestation tier below
Reachability" needed. Need a **substrate-witness family** at the existing
tiers, just over different artifacts.

Walking Tekgy's examples:

- **"Must use `expm1(|x|)` not `exp(x) - 1`"** — already in scope
  today; AST-greppable fingerprint + clippy-or-custom-lint witness.
- **"Function must have math-researcher ratify before considered done"**
  — substrate-witness; audit reads git log / forge metadata /
  `RATIFICATIONS.md` / signed-off-by trailers. Honestly mechanical.
- **"Linked doc must be ratified, and function ratified against it"** —
  compound substrate-witness; audit checks (a) doc exists, (b)
  ratification version meets min, (c) function/anchor named in
  ratifications, (d) not expired.
- **"Can't do X until Y"** — substrate-witness over issue-tracker / gate
  file / git tag.

Macro shape proposed:

```rust
// Witnesses over code-side substrate (existing)
#[immune(A, witness = my_test_fn)]
#[immune(A, witness = clippy::lint)]
#[immune(A, witness = Proof::<T>::verified)]

// Witnesses over non-code substrate (the gap)
#[immune(A, witness = ratified_doc(path = "...", min_version = "v1.0", anchor = "#sinh"))]
#[immune(A, witness = role_attestation(role = "math-researcher", expires_after_days = 180))]
#[immune(A, witness = signed_trailer(key = "Discipline-Verified-By", role = "..."))]
#[immune(A, witness = compound([ratified_doc(...), role_attestation(...)]))]
```

These are mechanical. Audit either resolves them honestly or doesn't.

**Lifecycle proposed**:
- **Issue state**: discipline antigen presented; immunity claim absent
  OR refers to doc/approval/signature that doesn't yet exist or is
  expired. Audit reports outstanding.
- **Resolution path**: author produces real substrate; updates witness
  predicate. Audit re-resolves: substrate exists, signatures match,
  fresh. State: handled.
- **Re-issue triggers**: code fingerprint changes; doc version bumps past
  pin; ratification expires; approving role leaves CODEOWNERS.

**"Handled" is earned by producing real substrate the audit can verify,
never by self-assertion.**

**Discipline-fingerprint clarification**:
- AST-greppable disciplines ("uses Y where X required") — already
  supported
- Non-AST-greppable disciplines ("must have been reviewed against
  canonical doc") — fingerprint is broader/structural; substrate-witness
  is what carries discipline-specific verification

Proposed adding optional `discipline_doc` field on antigen declaration
that *binds* canonical doc to the antigen.

**Biology rhyme** (first appearance): **T-cell + B-cell co-stimulation**.
B-cell receptor binding to antigen is signal 1; CD40-CD40L co-stim from
helper T-cell is signal 2. Without both signals, B-cell **anergizes**
rather than activates. Single-signal exposure produces *tolerance, not
immunity*. Predictive rhyme for compound witnesses: one leaf resolves
and another doesn't shouldn't behave like "partial immunity" — should
structurally fail.

## Turn 4 — Tekgy: composability + library-shareability

> "these would also be user-composable + global abstract library-able
> I think. so for tambear it could be they declare one that says this
> isn't valid until each team member has stamped their work in the
> linked document, or added their name to the approved list in the
> macro, or whatever. until everyone has signed, we don't treat as
> working even if the tests all pass, or whatever ; against oracles
> x y and z (etc) and until those named-in-antigen-requirement files
> exist and are marked complete or whatever, the nag keeps coming up
> in scan/audit because the oracles aren't done, etc"

Composability move. Sharpened the substrate-witness shape into a
**predicate language**.

**Three composability tiers proposed**:

**Tier 1 — built-in leaves + combinators**. Antigen ships fixed set:
- `ratified_doc(path, min_version, anchor?)`
- `all_signers(file, required)`
- `signed_trailer(key, role?, count?)`
- `oracle_complete(file)`
- Combinators: `all_of`, `any_of`, `not`
- Freshness: `within_days(N)`

Tier-honest (audit literally reads files), composes everything tambear's
example needs, no plugin machinery.

**Tier 2 — project-local named compositions**:

```rust
witness_alias! {
    tambear_team_ratified(doc, anchor) = all_of([
        ratified_doc(path = doc, min_version = "1.0", anchor = anchor),
        all_signers(file = ".tambear/team.md", required = ["alice", "bob", "carol"]),
        fresh_within_days(180),
    ]);
}
```

Macro-expands at parse time. No audit changes, no plugin machinery.

**Tier 3 — shareable witness provider crates**.
`antigen-witnesses-research-rigor` ships `peer_reviewed_and_replicated`.
`antigen-witnesses-corporate-discipline` ships `codeowners_approved +
slo_signed`. Crates the project depends on; audit walks dep graph.
Rhymes with roadmap.md:151 "cross-organization antigen registries."

ADR-007 anti-YAGNI: structurally-guaranteed-need; design extension
point now, ship only Tier 1 + 2.

**"Nag stays" property**: structurally automatic. If
`all_of([oracle_complete(x), oracle_complete(y), oracle_complete(z)])`
and only x is complete, audit reports failure with details. Until all
three oracles complete, the nag is structural. No laundering surface.

**Biology rhyme deepened**: co-stim is the right mental model.
**Costimulation deficiency** without B7-CD28 produces anergy. The
metaphor predicts missing-one-signal-in-compound-witness should not
"partially work"; should structurally fail. Audit reporting
nothing-while-incomplete is co-stim-style.

**Affinity-maturation** as freshness rhyme: antibodies undergo
iterative refinement; old antibodies that haven't been re-selected
against current antigen drift out of repertoire. Freshness-as-required
is co-evolution discipline.

## Turn 5 — Tekgy: structural representation, CLI tooling

> "I think we can also define a front-matter that would be required and
> it can just read it in any file format? or else we could also require
> that the handling substrate even if from a linked MD or git issue page
> or whatever must be in the correct repo as sidecar file or something
> IN RUST with a particular type/formatting whether yaml frontmatter or
> json objects with particular shape or whatever - we can just require
> that the team do stuff in certain ways, we can ship cli tools that can
> create and populate the right structure in the right place for the
> user to reduce friction and avoid drift away from templates whatever"

Claude's response named the shift: **strict-sidecar move closes the
tier-honesty leak.** Earlier shape had audit doing fuzzy interpretation
of markdown bodies. Strict sidecar files = audit reads typed data
against known schema. Same posture as Cargo.toml. Tier-honest by
construction.

**Three sidecar options weighed**:
- **Option A**: Rust const sidecars (`docs/signed-zero.attestation.rs`).
  Compile-time validation. Heavy. Doesn't compose with human readers.
- **Option B**: YAML/JSON frontmatter in the doc itself.
  Human-readable + machine-readable. More parsing surface; YAML
  footguns.
- **Option C**: Pure JSON sidecars adjacent to the doc
  (`signed-zero.attest.json`). Cleanest. Two files per doc. "Drift"
  between doc content and attestation is a *feature* — they're
  different claims with different lifetimes.
- **Option D**: All-of-above with format adapters. Defer.

**Lean: Option C with format-adapter ambition deferred.** Why JSON:
- Single canonical format keeps audit simple in v0.1
- Schema locked via serde-derived Rust type
- Cross-language friendly
- Diffable + mergeable
- Easy CLI scaffolding

**CLI sketch**:
```sh
cargo antigen attest new --antigen X --doc docs/X.md
cargo antigen attest sign --file X.attest.json --as math-researcher
cargo antigen attest check --file X.attest.json
cargo antigen attest list  # workspace-wide; stale + pending
```

**Tier mapping**:
- `Reachability` = sidecar file exists at path
- `Execution` = sidecar exists + parses + content predicate passes
- `None` = sidecar missing, malformed, or predicate fails

Cleaner than prior shape; audit is structured-data validation against
typed schema, not pattern-matching markdown.

**Biology rhyme (third appearance)**: **MHC presentation**. Antigens get
presented in a *structured frame* (MHC class I/II grooves) that the
immune system can recognize. Frame is constrained, not arbitrary.
Substrate that arrives in random format isn't recognizable. Predictive
rhyme for "structured sidecar instead of free-form markdown." Biology
already uses presentation-format-discipline.

Three coupled ADRs proposed:
1. Substrate-witness predicate family
2. `Ratification` schema + JSON sidecar convention
3. `cargo antigen attest` subcommand family

Each small; together load-bearing. ADR-007 argues shipping all three —
without (3), template-drift adoption failure; without (2), tier-honesty
drift; without (1), (2)/(3) have no use.

## Turn 6 — Tekgy: opinionated + flexible

> "like we can be opinionated even while allowing flexibility for people
> to integrate into their workflows"

Claude's response named the posture: **already antigen-the-project's
posture**. ADR-002 (compose-don't-compete) is opinionated-with-
flexibility by design. Not adding new posture; applying existing one
to discipline-witnesses.

**Seam falls cleanly**:

**Closed (the posture)**:
- Schema is Rust type; no new top-level fields
- Combinator language is closed set; no Turing tarpit
- Tier-honesty is mandatory
- Sidecar substrate-currency is JSON-against-schema
- CLI scaffolds canonical shape

**Open (the integration surface)**:
- `extensions: BTreeMap` slot on schema
- Witness provider crates contribute new leaves (Tier 3 deferred)
- Format adapter trait for TOML/frontmatter integration (deferred)
- Optional crypto signing
- CODEOWNERS interop opt-in
- Sidecar location configurable

**Principle**: *closed at load-bearing parts, open at integration
edges*. Closed where openness would leak tier-honesty. Open where
closedness would block adoption.

**Biology rhyme (fourth)**: immune system itself is exactly this.
Hard rules: no MHC presentation = no recognition; no co-stim = anergy;
self-reactive = clonally deleted. Massive flexibility: ~10^11 unique
TCRs, ~10^13 unique antibodies via somatic recombination + affinity
maturation. Opinionated about *posture*; flexible about *content*. That's
the rhyme.

Named that conversation substrate was getting dense and would evaporate
into scrollback. Asked if write expedition doc now or sit with it.

## Turn 7 — Tekgy: CODE-LOCALITY reframe

> "I'm sort of thinking the opposite end, that while the antigen or
> immune system component for this type of issue is in the code, and it
> may point to a doc or webpage or postmortem or pr or git issue or
> local doc or whatever, I think we want the requirements and the
> fulfillment next to the code not next to the doc, we won't always be
> able to put things next to the doc but this is a developer-side
> concern, so we should get the stamps and approvals and ratifications
> and discipline and confirmations and whatever next to the actual code.
> so some-function.rs gets some-function-d1.json and some-function-d5.json
> and whatever per discipline antigen in the .rs file and until the json
> meets the requirement specified by the macro in the .rs, then we fail
> audits"

**This was the user-driven reframe that broke open the biology rhyme at
the right layer.** Claude had been thinking doc-locality
(central-registry pattern). Tekgy pushed to code-locality
(germinal-center pattern).

Claude's response named what shifted:
> "Code-locality is the more biological choice. I had the MHC rhyme
> right but applied it to the wrong layer. In immunology, antigens are
> presented AT THE CELL where they're processed — MHC presentation is
> per-cell, recognition memory lives in the lymphocyte, somatic
> hypermutation happens in germinal centers adjacent to the cells doing
> the recognition. There is no 'central doc registry of who's been
> vaccinated.' Distributed substrate, locally validated, per-presentation.
> The doc-side fulfillment I was sketching was central-registry-shaped.
> Yours is germinal-center-shaped. Yours is correct."

**Structural property unlocked by code-locality**: fingerprint-pinned
attestation. The sidecar pins to code fingerprint at sign-time, *per
signer*:

```json
"signers": [
  {
    "name": "alice",
    "signed_against_fingerprint": "sha256:abc123..."
  }
]
```

When function body changes, antigen's existing fingerprint machinery
fires. Sidecar reports: "alice signed against abc123; current is def456;
signature is stale; re-attest required." **Automatic re-attestation
discipline at the signer level, not just the doc level.**

This was the Claude-driven realization on top of the user-driven
code-locality move. Affinity-maturation rhyme strengthens: antibodies
selected against specific antigen variants; when antigen mutates, old
antibodies may no longer bind; germinal center refines.

**PR-review workflow alignment** as adoption win:
- Function changes → reviewer sees function diff
- Adjacent sidecar in same PR → reviewer sees `signed_against_fingerprint`
  drifted
- Sidecar diff shows alice's signature stale
- Reviewer requests re-sign in same PR, same diff thread

Substrate flows with code through review. No jumping to external
systems.

## Turn 8 — Tekgy: layered persistence ask

> "i def feel we need to park things on disk somewhere, overall stuff,
> raw text dumps, summaries, plans, initial draft designs we might
> reconsider later, various layers of persistence to pick up from"

Claude proposed subdirectory layers within expedition/: captures/,
drafts/, summaries/, plans/, with INDEX.md. Existing flat files stay.

Layer discipline:
- **captures**: append-only; high-fidelity; date-stamped
- **drafts**: version-bumped (v1 stays when v2 lands)
- **summaries**: mutable; re-written in place
- **plans**: mutable until commitment

## Turn 9 — Tekgy: per-antigen-per-file granularity

> "I think it's per antigen per file, you? like if we specify 50
> different antigens in a file all discipline or mixed discipline and
> other types, any that have a requires that gets a sidecar each gets
> their own sidecar file to check against? maybe we do some-function.rs
> and some-function.ag/antigen-{id}* type thing to keep things tidier
> with an antigen subfolder for multiple antigen sidecars?"

Claude reversed prior preference (per-file-with-items-inside) and
agreed with subfolder shape. Pushed slightly on naming: `.attest/`
over `.ag/` because:
- Parallels CLI subcommand
- Names what's IN folder, not what folder is FOR
- `.ag/` overclaims (mechanical antigens don't get sidecars)

**Three-layer granularity locked**:
- Source file ↔ `.attest/` folder
- Antigen ↔ `<AntigenName>.json` sidecar in folder
- Item (fn/struct/impl) ↔ entry in sidecar's `items[]` array

## Turn 10 — Tekgy: `.attest/` confirmed

> "i like attest, i was just thinking out loud in draft, not suggesting
> actual naming with the .ag"

Shape locked on naming. Wrote INDEX.md, drafts/discipline-witnesses-v1.md,
and initial version of this capture.

## Turn 11 — Tekgy: generated code is niche, defer with hooks

> "i suspect generated code is more of a niche issue that may need to be
> handled later with tools that have hooks or something to let the code
> generator plug into our machinery to author whatever it recognizes, or
> else human/llm agents will need to parse generated code to hook into
> antigen or something?"

Claude sketched three paths:

- **A. Code-generator hooks**: generators expose "pending attestation"
  API emitting sidecars with placeholder signers
  (`scaffolded-by-generator-awaiting-author-sign`). Complex; needs
  plugin coordination.
- **B. Post-hoc agent parsing**: human/LLM reads generated code,
  authors sidecars. Brittle — regeneration breaks linkage.
- **C. Input-level discipline**: antigen + sidecar applies at the
  generator's *input layer* (schema, IDL, build.rs). Author signs at
  input level; generator faithfully emits output. Antigen scan walks
  both input + output via cargo's normal build artifacts.

Concrete example sketched: tonic generates gRPC service trait impls
from `.proto` files. Discipline antigen about gRPC error handling
belongs next to the `.proto` (or the build.rs that invokes tonic),
not next to the generated `service.rs` in OUT_DIR. The author
exercises discipline at the `.proto` level; generated code is
mechanical output.

Claude's lean: C is the right answer for most cases. A is fallback for
generators that don't expose authorable input (rare). B is brittle and
should be avoided.

## Turn 12 — Tekgy: confirm C + "accept the tradeoff" escape valve

> "you're absolutely right C. and if gen doesn't expose authorable input
> I suggest they would need to make it do so, or else they just skip
> .attests on generated code and accept the tradeoff"

This was the cleaner closing move. Tekgy compressed the position to:
input-level discipline OR accept-the-tradeoff. No generator-plugin hook
subsystem needed.

**Compose-don't-compete in action**: discipline applies at the right
layer (input where judgment is exercised), or the project accepts the
gap. No new machinery built to paper over the gap. ADR-002 posture
extended cleanly.

**Confirmed v0.1 position**:
- Generated `.rs` files (OUT_DIR, proc-macro expansion) are out of scope
  for `.attest/` sidecars
- Generated code can still *present* antigens via emitted macros; audit
  scans them
- Discipline antigens for generator output belong at input layer
  (`.proto`, IDL, build.rs, macro invocation site)
- Two escape options if input not authorable:
  (a) request upstream to expose authorable input
  (b) skip `.attest/` coverage on generated code; accept the tradeoff
- No generator-plugin hooks planned absent specific adopter use cases

Draft updated to confirm this position in "What this is NOT" section.

## Turn 13 — Tekgy: "accept the tradeoff" IS the natural default

> "in some ways I think accept the tradeoff on generated code is going
> to be the default for people in that circumstance anyway, the code is
> generated, they already trust the generator. if someone's writing a
> new generator they can build antigen attests into it. no biggie. if
> the generator exposes authorables, then also no biggie. but otherwise,
> they're using the generator because they trust the code output, so...
> less likely they'll care to adopt something in that specific code
> output"

Reframe of the v0.1 position: "accept the tradeoff" isn't a regrettable
gap; it's the *correct* alignment with how generators are actually used.
Trust was extended at adoption time, at the coarser boundary of "this
generator." Per-output attestation would be redundant re-checking of
something already attested at the right layer.

**Three cases sharpened**:
- **(a) Existing trusted generator without authorable input** — accept
  tradeoff; this is the expected default. Adopters extended trust at
  adoption time; output is *what they're using the generator for*.
- **(b) Generator could expose authorable input but doesn't yet** —
  upstream contribution / vendoring; once authorable, input-level
  discipline applies.
- **(c) New generator being written** — build antigen attestation in
  from the start.

**Biology rhyme**: **clonal selection at thymic education**. Immune
system establishes trust in self-antigens during T-cell development;
TCRs that pass negative selection aren't re-checked against every
self-antigen presentation. Trust boundary lives at the developmental
layer, not per-encounter. Generator adoption is the thymic-education
equivalent — adoption-time selection extends trust to the generator's
self-antigens (its output); per-output re-attestation would be like
running thymic selection on every cell interaction.

**ADR-005 sub-clause F connection**: trust boundaries live where trust
was extended, not at every downstream presentation. Generator adoption
IS the trust boundary; the discipline attests at that layer (or doesn't,
because adoption already extended trust there).

Draft updated to reflect "accept tradeoff is the expected default"
framing in "What this is NOT" section, with three cases and the
thymic-education rhyme.

---

## Dead ends preserved (so next-session-Claude doesn't re-derive them)

1. **"Add an Attestation tier below Reachability in `WitnessTier`"** —
   creates laundering vector; eroding tier-honesty into ambient cover
   for unverified claims. Don't.

2. **"Put doc-attestation in the witness enum at peer level"** — tier-
   honesty violation; doc-attestation sits structurally *below*
   Reachability (audit can't verify the truth of the assertion, only
   verify that the assertion exists in correct format).

3. **"Use phantom-type witness as stand-in for agent assertion"** —
   reports FormalProof tier for an attestation. The exact failure
   mode ADR-005 Am 3 was named to catch.

4. **"Doc-side sidecar location"** (`signed-zero.md` +
   `signed-zero.attest.json` next to each other) — central-registry
   pattern. Wrong abstraction layer. Per-cell antigen processing
   predicts code-locality.

5. **"One sidecar per file with all antigens inside as nested keys"** —
   sidecars become huge; merge conflicts when two PRs touch different
   antigens; signing one antigen rewrites whole file. Per-antigen-
   per-file with `.attest/` subfolder is better.

6. **"`kind = 'discipline'` on antigen declaration as the load-bearing
   distinction"** — would create slippery slope to `kind = 'performance'`,
   `kind = 'security'`, etc. The substrate-witness reframe means
   discipline-antigens aren't a special category; they're ordinary
   antigens whose witness predicates evaluate against non-`.rs`
   substrate. `discipline_doc` field as optional default is the
   minimum-viable version of this.

7. **"Tambear authors the ADR"** — inverts standing constraint; tambear
   is smoke-test, not design authority. Memory note:
   `feedback_tambear_is_smoke_test_not_design_input.md`.

---

## User-driven moves vs Claude-driven moves

Honest accounting:

**User-driven** (the reframes that broke things open):
- Reframe as design conversation, not navigator-routing decision
- Push on lifecycle/handled-state question that forced the substrate-
  witness reframe
- Composability + library-shareability push
- Strict structural representation move (frontmatter / sidecars / Rust
  types — proposed range of options that led to JSON-sidecar lean)
- Opinionated-with-flexibility posture naming
- **Code-locality reframe** (the load-bearing one; broke central-registry
  pattern Claude was reaching for)
- Per-antigen-per-file granularity with subfolder shape
- Layered persistence ask
- **"Accept the tradeoff" escape valve** for generators without
  authorable input — prevented Claude from over-engineering toward a
  generator-plugin hook subsystem

**Claude-driven** (sharpenings on top of user's moves):
- Substrate-check pushback on tier-honesty implications
- Four-shape candidate sketch
- "Witnesses currently only check code-side substrate" reframe
- Predicate language with combinators
- Schema-as-Rust-type detail
- Fingerprint-pinned signatures (after user's code-locality move)
- `.attest/` over `.ag/` naming nudge
- Biology rhymes (MHC, co-stim, affinity maturation, germinal centers,
  passive immunity break-point)
- Three-coupled-ADR family proposal
- Layer-discipline naming for the persistence ask
- A/B/C decomposition for generated-code paths (user picked C and
  added escape valve)

The shape that emerged is genuinely co-authored. Neither party could
have reached this shape alone in this number of turns.

---

## Biology rhymes that earned their keep

1. **MHC presentation → typed sidecar substrate-currency** — structured
   frames make recognition tractable; free-form is unrecognizable
2. **T-cell + B-cell co-stimulation → compound witnesses require all
   signals** — single-signal exposure produces anergy, not immunity;
   compound witness with one leaf failing must structurally fail
3. **Affinity maturation → fingerprint-pinned signatures** — antibodies
   re-selected against current antigen variant; old signatures drift to
   stale
4. **Per-cell antigen processing → code-locality** — distributed
   substrate, locally validated, per-presentation; germinal-center
   pattern
5. **Productive break-point: passive immunity / record-trust** — biology's
   passive immunity still works through molecular machinery; substrate-
   witnesses are more like "immune system trusts the medical record";
   biology can't read records. This is where metaphor stops predicting
   and we're inventing.

---

## What this capture is for

- **Next-session pickup**: avoid re-deriving the dead ends; see the
  user-driven reframes that need preserving
- **Team deconstruction**: aristotle/naturalist/adversarial can see
  the reasoning chain, not just the conclusion
- **Adversarial review**: the dead ends are honest about what was
  considered and rejected
- **Future post-mortem**: if the shape doesn't survive deconstruction,
  this capture shows where the reasoning went wrong vs which moves
  were load-bearing
