# Staging draft — ADR-066 (Antibodies: the Adaptive Self; antigen as a Lens)

> **Staging/ceremony notebook, not the canonical record.** Status **Proposed**. Lands in `docs/decisions.md`
> (after ADR-065) only after a final ceremony pass on this revision. Drafted by the design-pair (Tekgy +
> Claude), 2026-06-24, from a real dogfood pain (tambear's `#[defended_by]` limits).
>
> Two registers: a **ratified spine** (Decision — durable invariant + process) and a **current map**
> (drift-allowed orientation — the *why*, never decreed). The split is
> [[feedback_adrs_encode_invariant_and_process_not_outcome]] applied to the ADR's own structure.

## Revision log

**v1 → v2 (post-7-lens-council).** Council confirmed the bones (per-site binding, emission-fidelity) are
bedrock; rebuilt the foundation: proof is a *culture* not a byte-property; rigor is *borrowed from proven
biology-math*; no "DONE PROTECTING" threshold (show the biology); the standing is a field, not an invented
scalar. All 7 v1 BLOCKERs folded.

**v2 → v3 (post-7-lens *verification* council).** A second, differently-composed council (detail-auditor ·
biology-faithfulness · mal-intent · practitioner · pedant · observer · expansionist) verified v2: **6/7 v1
BLOCKERs closed at the root, all 5 supersessions clean.** It also corrected three real things v2 got wrong,
and they reshape the spine:

1. **The diffusion claim was an overclaim** (biology-faithfulness, applying v2's own guard): pure diffusion
   demands a conserved quantity, a metric, and a flux boundary — none of which the stroma supplies — so "we
   inherit the proven diffusion math" *fails the guard v2 itself states*. **Fix (converging with the
   expansionist):** drop pure-diffusion; the faithful maths are **reaction-diffusion** (the antibody is the
   *sink* — unifying field §4 and mitigation §7; the steady-state *is* residual loudness, "an equilibrium, not
   a bar"), **quasispecies / replicator-mutator** (antigen's own canonization loop is germinal-center
   selection — the process it models most faithfully and hadn't claimed math for), **Hill kinetics** (the
   combine-semantics; the clonal/igg/polyclonal qualifiers are its cooperativity coefficient — closing the
   ADR-024 seam), and **percolation / R₀** (graph-level herd / blast-radius). §4 now ratifies the *discipline +
   named candidates*; the full derivation is a **math-research follow-on**.
2. **antigen is a *lens*, not a gate** (practitioner + design-pair): "never green" is not a defect to fix — it
   is the spine working. *Immune activity is health; an all-green readout would mean no immune system, not a
   protected one.* The fix the practitioner's finding actually demanded is **legibility, not greenness**.
3. **The output model** (design-pair): one complete persisted biology (the stroma map, every item in detail,
   regenerated each run, nothing hidden) + many projections (summary → `-v…-vvvv` → the full opt-in report).
   Progressive disclosure reconciles *hide-nothing* with *don't-drown*.

Plus mal-intent's substrate-invariants (ship-strict-default, graph-integrity, downstream-suppression-surfacing,
targeting-map opsec) and the auditor/pedant polish (the §-ref fixes, the Finding-§2 mis-diagnosis, two real
biology errors, glossary `observed/asserted`, the `kind` alphabet).

---

## ADR-066 — Antibodies: the Adaptive Self; antigen as a Lens (Observability over Declaration)

**Status**: **Ratified 2026-06-24** (v0.6.1). Drafted through two author-distinct ceremony councils (a 7-lens
design pass + a 7-lens verification pass) and a sealing notary; landed in `docs/decisions.md`. The §4
math-development is a named research follow-on (the `math-feeder-066` voyage).

**Supersedes / extends**: ADR-029 (supersedes class-level "one witness credits all sites" matching + the
test-only witness scope; **preserves + generalizes** "immunity is *observed*, never *declared*"). ADR-029
**Amendment 1** (substrate-gap precedence — a failing `requires=` is not masked by a witness) is honored at §3.
**Names ADR-024 as a source** for the evidence-qualifiers (incomplete reconciliation — see Open seams). Mirrors
**ADR-009 Amendment 1** (verify-only/external-substrate antigens) onto the defense side. Generalizes **ADR-002
Amendment 3** (herd-immunized substrate = the non-self-good quadrant cell; the stroma-builder is a sovereign
own-capability — its clause-3 empirical rebuttal lives in *its* follow-on ADR). **Honors ADR-041** (`dread`/`aura`
stay on the marked-unknown plane, OFF the classification axis; this ADR only *adds* proof-of-emission — note
ADR-041 is itself "locked, not yet ratified").

**Related**: the no-self-witness invariant, co-native + honest-labeling disciplines, the decidability ceiling
([[feedback_biology_summons_math_and_enforced_math_guides_the_build]]), the structural digest / stroma,
*Journey Before Destination*.

---

### Finding

Dogfooding surfaced four seams in `#[defended_by]`, one root:

1. It attaches only to a test/fn (`antigen-macros/src/lib.rs:347`) — asserting, falsely, that every silent
   failure is test-coverable.
2. **Its macro *name and attachment* are inverted:** `#[defended_by(X)]` reads "a *test* is defended-by a
   *threat* X" and attaches to the test, not to the defending act. (The *argument* naming the antigen is
   correct — a paratope is named for the epitope it binds; the defect is the name + where it hangs, not the
   argument.)
3. Matching is class-level (`immunity.rs:253-262`, self-documented as an open gap) — N×`#[presents(X)]` +
   1 witness ⇒ all N read defended. A real soundness hole.
4. A defense is asserted, never observed (the macro is a pure doc-marker) — the audit cannot tell a
   demonstrated defense from a claimed one, nor express a partial one.

Root: **antigen has a rich catalog of non-self and no first-class repertoire of self.** An immune system is
*defined* by adaptive memory of the protective. The missing positive-self is structurally demanded by the
biology.

The deeper question the councils forced — *on what authority do we credit a defense?* Not on the bytes being
"proof" (you can manufacture proof; many silent fail-classes are unprovable). Authority is twofold: **rigor**
borrowed from *proven biology-math* where the analog is faithful, and **trustworthiness** from a *culture of
proof-over-trust* — observability over declaration. The honesty is the rigor.

---

### The thesis: a lens, not a gate; show the biology; the metaphor does the work

**antigen is a sensing organ for a codebase's self-knowledge — a lens, never a compile-blocker.** It does not
**declare** a verdict ("defended", "done", "safe"); it **shows the current biology** — the immune state at each
site — and makes it legible to *every* mind on the team, human and AI alike, in ways docs and Slack
structurally cannot. **Immune activity is health.** A codebase showing *no* immune signal isn't protected — it
has no immune system. "Never green" is therefore not a defect; it is the organ being alive. The job is to make
the living readout **legible** (below), and to **teach** that antigen is *signal, not noise*.

The immune metaphor (antigen's only public vocabulary) carries **four jobs at once**, which is why it is
infrastructure, not decoration: **rigor** (proven biology-math, §4); **transparency** (show the biology, never
hide, never declare done); **risk-communication** (biology names carry their own caution — `immunosuppress`
*feels* risky to anyone); **responsibility** (the mask-in-a-pandemic model, §Responsibility).

"Proof over trust" is a **cultural commitment**, not a byte-property. Trust remains; the culture (observability,
no-self-witness, openness, strict defaults, re-derivable evidence) makes manufactured proof *discoverable* and
trust *earned*. **"Proof" has two non-overclaiming senses:** proof of **detection** (observable evidence the
pattern is present) and proof of **mitigation-capability** (transparent evidence a *recommended* procedure
*does or can* mitigate — many alternatives, any may suffice, combinations may be partial; sufficiency is the
user's judgment on shown evidence, never our decree). Honest scope for the median real defense: most are
*asserted* (a semantic fix), so antigen is best read as **a disciplined, per-site defense *registry* with a
re-verified minority — not a proof *engine*.**

---

### Decision

**The invariant: antigen shows the current biology and never declares a protection-verdict, and never blocks.**
A defense is a named, per-site, observable (or honestly-labeled asserted) *reaction*, whose effect is shown as
a change in the site's immune state. Rigor is borrowed from proven biology-math where the analog is faithful;
trustworthiness is the proof-over-trust culture. Everything below is method.

1. **The antibody is a first-class definition — a *recommended mitigation*, not a guaranteed cure.**
   `#[antibody(binds = X, kind = …, references = […])]`. **`binds` always names the ANTIGEN** (the epitope) —
   *enforced at compile time* (the path must resolve to a declared antigen, never a defender shape; this also
   kills the inversion footgun). **The `kind` alphabet** (closed, extensible by ADR): `Test` · `Structural`
   (a re-checkable shape) · `TypeProof` · `SubstratePredicate` · `Asserted*` (the asserted vocabulary —
   `AssertedFixedInPR`/`…Commit`/`…Refactor`/`…PackageUpdate`/`…InternalDoc`/`…BySignedPerson`). An antigen has
   *many* possible antibodies; any may suffice; combinations may be partial. `#[defended_by(X)]` is retained as
   a **deprecated alias** — honestly: *it retains class-level (hint-grade) crediting; instance-binding is what
   upgrades a site's shown state* (a migration *codemod* + a transition window before the new self-antigen
   fires are the adoptable path — not "we broke your sites politely").

2. **A binding addresses one site, over the sovereign stroma.** The unit is **one reaction to one stroma-node**
   (multiple antibodies may react to one node — §7 — so 1:1 is binding↔reaction, not binding↔site). Identity is
   the **stroma-node** in antigen's **own sovereign stroma-builder** (ADR-002 Amd3): **fully-qualified,
   collision-free** (closing the cross-file `validate`/`validate` collision — `scan/diff.rs:117` last-write-wins
   — that bare item-name leaves open); **node lifecycle** (deletion/rename/split re-home or re-open bindings);
   and it **owns the digest + threat model** (doc-comment-only edits do not re-open; semantic-equivalence under
   different tokens is out of scope by the ceiling). **The binding carries the structural predicate it matched,
   so re-confirmation is *mechanical/automatic* where the predicate still holds — re-verify, don't re-ask**
   (this kills the re-confirmation treadmill). **Graph-integrity is a faithfulness precondition** for the field
   math (§4): the field is only as honest as the stroma it runs over, so the stroma-builder follow-on ADR MUST
   carry graph-integrity (who may add/remove edges; append-only vs mutable) — a poisoned graph yields
   false-quiet with mathematical credibility. Honest strength: **1:1 by-construction for single-present nodes;
   conservative-reopen for multi-present** (the positional `id=` disambiguator means editing one present can
   re-open a sibling — fails *safe* (LOUD), named `AntibodyRebindsAcrossLocalDisambiguator`). Class-level
   matching is demoted to a non-closing hint. **An out-of-source binding manifest** (a sidecar keyed on
   stroma-node id) is offered for teams that don't want stacked annotations on hot code. The full builder is a
   **required sovereign dependency with its own follow-on ADR** — not built here.

3. **Observability over declaration — the proof/trust line.** A reaction is **observed** (the audit
   independently **re-checks it at this site**) or **asserted** (a claim it cannot re-check). The discriminator
   is one decidable question — *"can the audit re-check this, here?"* — a property of the verifier, not the
   world. **The re-check scope of a `Structural` antibody must be declared** (immediate-body vs effective-
   behavior): a body-only check is evadable by one-hop delegation (move the danger into a helper), so the kind
   states its depth and the readout shows it. Per **ADR-029 Amendment 1**, a failed `requires=` substrate-gap
   takes precedence over any binding.

4. **The standing is a *field* governed by structure-faithful math — the discipline is ratified; the exact
   math is research.** A site's immune state is a field over the stroma (the manifold), not a binary verdict
   and not an invented scalar — that structural direction is bedrock. **What v2 got wrong, and v3 fixes:** we
   do *not* "inherit proven continuous-diffusion math" — applying our own guard, pure diffusion demands a
   conserved quantity, a metric, and a flux boundary the stroma lacks, so that borrow is *unfaithful* and
   falsified. The ratified commitment is the **discipline**: *the relationships are governed by the best
   structure-faithful math; the guard is **applied** (the required vs supplied vs absent terms are enumerated
   per candidate); the lossiness is named; a discrete code-graph summons discrete/graph math, not a borrowed
   continuous PDE.* The **named candidates** (to be developed in the math-research follow-on, each
   guard-checked before enforcement):
   - **reaction-diffusion** — the **antibody is the sink**, which *unifies* this field with mitigation (§7);
     **residual loudness is the steady-state** given current sources+sinks ("an equilibrium, not a bar" — and
     the principled meaning of "quiescent under policy P", §5);
   - **quasispecies / replicator-mutator** — antigen's own canonization loop (no-self-witness · maturation-
     through-exposure · variant-research) *is* germinal-center selection; summons a fitness function, a
     competition term, a variant operator, and a computable complexity ceiling (Eigen's error-threshold);
   - **Hill kinetics** — the combine-semantics (resolving superpose-vs-saturate); the clonal/igg/polyclonal
     qualifiers are its cooperativity coefficient — **closing the ADR-024 seam**;
   - **percolation / R₀** — the *graph-level* herd / blast-radius signal (critical-immunization fraction).
   (`clonal`→branching-process is **downgraded to metaphor** — antigen's matching is deterministic, not
   stochastic — flagged per the guard.) **The convergent missing primitive** three of these maths independently
   demand: a first-class **membrane / compartment-boundary** (host↔federation permeability — what crosses is a
   re-verifiable fingerprint; what doesn't is host-local trust/instance/suppression). **The scalars measure
   smoke** — the math is rigorous, the *observables fed into it* are proxies; we say so. **`dread`/`aura` are
   NOT points on this field** — they live on ADR-041's marked-unknown plane (⊥); this ADR adds only
   proof-of-emission (§8).

5. **No protection-threshold in the truth — thresholds are policy, and we ship strict.** antigen never emits
   "defended/done/sufficient"; truth = the field, **shown**. *Partial* protection is **residual loudness
   shown**, not "below a bar". Any gate is the **user's policy layer** on top — and **that antigen ships a
   *non-permissive* default is itself a ratified invariant** (the *category* "strict, not a fig-leaf"; the
   *value* drifts), so "show the biology" can never become a responsibility-disclaimer via a permissive shipped
   policy. We ship **named presets** (strict/standard/advisory) so no team cargo-cults a raw number. The reader
   can predict the **loudness they will see** and the **bucket**, never an exact internal number — said plainly.

6. **Immunosuppress = a visible, felt, individually-signed reaction-by-reaction act — never a silent global
   knob.** Per-(site, antigen); *N* acceptances are *N* signed, individually-expiring lines (friction
   proportional to scope), and the **aggregate is a first-class LOUD status line**. A global threshold-softener
   is **forbidden** (v1's silent flip). The name carries the risk. Guards: cap consecutive renewals;
   no-self-witness on renewal (a *different* `signed_by` after K); asserter-trust config is **non-self**
   (no one sets their own; unknown-asserter default = 0). **A flood of cheap asserted reactions must not drive
   a node quiet:** asserted reactions contribute *less* to the shown reduction than observed, with a floor for
   asserted-only sites, and asserted-count > N triggers the aggregate-loud line without an explicit suppress.

7. **Partial mitigation is shown as residual loudness; sufficiency is the user reading the biology.** No
   machine "sufficient". A reaction that *claims coverage it lacks* is the born-red self-antigen
   `AntibodyOverclaimsBindingScope` — caught where catchable (the audit re-checks an *observed* reaction's
   actual reach; an *asserted* claim is labeled asserted and shown, never trusted). Coverage is **shown, not
   unioned-to-a-verdict.**

8. **Proof of *emission-fidelity* at every tier — not of universal detection.** Every fingerprint (antigen or
   antibody, incl. `dread`/`aura`) carries a **positive specimen**, a **non-vacuous negative control** (defined:
   a *minimal structural perturbation* of the positive — a near-miss — never an unrelated specimen; verified by
   the author-distinct reviewer at canonization, since this teeth-check is cultural, not a mechanically-
   unfakeable byte-check), and **scoped circumstances**. Constructable at every tier; **decidability caps how
   high a verdict climbs, never whether the behavior is provable.** Proven dreads/auras are equal citizens.

9. **The data model carries, first-class** (anti-YAGNI): an **evidence-package** (`reproducer_or_method`
   (test-id | command | structural-predicate | prose-method) · `research_narrative` · `public_scan_findings?` ·
   `disclosure_trail?` — field-set ratified, schema drifts), a **citable identity** (content-addressed on the
   structural digest), and **queryable taxonomy lineage** (CWE/CAPEC/ATT&CK/CVE — structured, so
   corpus-completeness is auditable).

10. **Public-health duty.** **A published weakness fingerprint ships with its antibody** (diagnosis-plus-cure —
    the only part of disclosure that is durable spine). The disclosure *workflow* is process. **Two operator
    opsec invariants** the mal-intent lens forces into the spine (the externality duty, not the entity's): the
    **scan-results inventory is a targeting map** — it MUST NOT be centralized beyond the disclosure window and
    MUST be access-scoped; and **a published artifact's own suppressions surface in downstream consumers'
    reports as residual loudness attributed to the upstream dependency** ("upstream X suppressed this; it is
    your residual" — the immunosuppressed-carrier-is-still-infectious mechanism).

11. **One complete biology, many projections — the lens's output.** antigen **always computes and persists the
    complete biology**: the full stroma map, every item in detail, regenerated each run — *nothing hidden*. The
    **surface is a legible, prioritized summary** (e.g. *"presents 37 of these, 15 of those, 18 dreads — only 3
    with no antibody present; here is your action-menu: confirm · suppress · assert-if-configured ·
    automated-only"*) with **stable pointers** (the sovereign stroma-node ids, §2) into the full report. A
    standard **`-v … -vvvv`** dial pulls more inline. Depth is **progressive / opt-in** — this is the
    reconciliation of *hide-nothing* (the complete map is always there) with *don't-drown* (the summary is
    clean). The persisted map is the **distributed-cognition substrate** the whole team references; its
    **per-run delta is itself signal** (a codebase's immune *history*). The specific verbosity levels and report
    format drift; the *completeness + progressive-disclosure + stable-pointers* are the invariant.

**New self-antigens** (born-red constructable): `ClassLevelDefenseCreditsUnexercisedSite` (Finding §3) and
`AntibodyOverclaimsBindingScope` (§7).

---

### Responsibility — the mask-in-a-pandemic model (two levels)

antigen is *an immune system*; it does not police. **The individual entity is sovereign over itself** (its
policy thresholds, suppressions) — we **inform**, default-strict, make-risk-felt, and **show**; we do not force.
**The public-health operator (antigen / the federation) owns the *externalities*** — disclosure, herd
stewardship, the targeting-map opsec (§10): *the health authority must never publish a hit-list of the
unprotected to those who would exploit it.* A vulnerable published crate is someone else's exposure (an
unmasked carrier) — which is why §10's downstream-attribution invariant exists.

---

### Biology grounding

**Class-1 (biology-predicted; where rigor is borrowed — each guard-checked in the math-research follow-on):**
the antibody↔antigen binding relation; **the field maths of §4** (reaction-diffusion / quasispecies-selection /
Hill-kinetics / percolation) — proven *in their native domains*, to be verified faithful term-by-term before
enforcement; **clonal/IgG/polyclonal as *distinct evidence mechanisms*** — affinity (IgG), repetition (clonal),
breadth (polyclonal); **immune memory as a learned, matured repertoire** (the positive-self the biology
demands); **structural recognizer / missing-self** (a structural antibody = the antigen-shape is *gone* —
*not* antibody-mediated; NK missing-self has no paratope, so it is named a recognizer, not an antibody); **the
danger model** (Matzinger — threat is danger, not origin); **no homeostatic "done".** **The guard:** enforce a
process's math *only where the analog is genuinely faithful*; a math-demanded-but-absent term (a conserved
quantity, a metric, a flux boundary) is the **falsifier** — back off (pure-diffusion → reaction-diffusion is
this guard *working*). **"Proven" caveat:** in biology "proven" means *well-modeled and empirically supported*,
not theorem-proven — we lean on the math's *formal* rigor in its native domain, and on the *guard* for the
transfer. **Software-engineering / security-research invention (honest silence):** the observed/asserted
discriminator, the policy/truth split + ship-strict invariant, the data model, responsible disclosure, the
output/projection model.

---

### The current map (orientation — drift-allowed, NOT decreed)

**antigen is the self-knowledge organ of a codebase, across distributed (human + AI) cognition** — a lens that
makes the codebase legible to every kind of mind, the thing docs/Slack can't be. **Adoption is for those who
want the lens**, and **education is load-bearing** (teaching "antigen is signal, not noise; here's why we never
say 'good to go'" is a first-class onboarding artifact, with a real cost: a narrower initial audience — a
deliberate positioning choice, not an accident).

**The danger-model quadrant** (self/non-self × good/bad; threat = danger not origin): self-good = healthy code +
own bindings; self-bad = own presents; **non-self-good = herd-immunized deps / imported repertoires (ADR-002
Amd3)**; non-self-bad = supply-chain pathogens. **Two stacked selves** (antigen-the-organ vs the host entity);
the quadrant is host-relative; dogfood is the reflexive case. **Entity-sovereign, ecosystem-aware** — antigen
spans prophylactic / preventative / specific-immunity (per-entity repertoire) / herd-immunity (shared
substrate) / public-health (federation, disclosure) / policy (user gates).

**The public library — three layers** (decidability governing potency / portability / shareability):
re-verifiable antigen fingerprints; re-verifiable antibody fingerprints (potent in a foreign host *because*
re-verified there — the repertoire self-selects for the decidable frontier); and the asserted-antibody
*vocabulary* (the kind + its checkable/claimed profile travel; instance + trust are host-local).

**A curated arXiv-for-immunity — with a trust-root caveat.** Never auto-publish; team-reviewed in batches;
canonized only with no-self-witness. A contribution is a **reproducible before→after case record** (self-
verifies + documents + upgrades provenance). DOI-like identity; submitters credited; community pen-tests
(adversarial peer review at scale). **Design against the centralized-canonization trust-root** (multiple
independent canonizers; content-addressed identity that needs no registrar's blessing; re-verification-or-it-
doesn't-count *inside* the federation; **Sybil-resistance is a precondition** for the pen-test/citation
maturation to mean anything). Governing filter: ***a fingerprint that requires the federation's trust to be
believed does not belong in the federation.***

**The MITRE proof-corpus.** Fingerprint CWE/CAPEC/ATT&CK — trust-corpora → proof-corpora, **stratified by
provable-vs-reasoned** (the honesty MITRE's uniform-trust lacks). A measurable coverage frontier;
**bidirectional** (consumer + proof-backed contributor). The **ICR** is a separate, linkable platform-format
(it cites the antigen-DOIs + MITRE mappings).

**Every fingerprint is a public good independent of antigen-adoption** (a white paper that protects even
partially helps a maintainer who never runs the binary). **Triple life** — code (recognizer) · white paper
(the research-account / proof-of-emission, sized to its ceiling — *the journey is part of the proof*) · citable
identity — no translation layer (co-native). Public artifacts badge the **scoped** claim ("specimen-verified at
tier T, coverage C"), never bare "proven".

**The math-research follow-on.** §4's named candidates are developed in a dedicated math-research voyage (read
the original papers; verify faithfulness term-by-term; apply the guard; name the lossiness) — first targets:
the quasispecies fitness-function build-spec and the percolation critical-fraction computation. Its output feeds
a successor ADR; the membrane/compartment-boundary primitive is its convergent design target.

---

### Process not outcome (durable / drifting)

- **Invariant (durable):** show the biology, never declare a verdict, never block; a defense is a named
  per-site observable-or-honestly-asserted reaction; observability over declaration; rigor borrowed *only where
  the analog is faithful, the guard applied*; thresholds are policy and **antigen ships a non-permissive
  default**; a published weakness ships with its antibody + the two operator-opsec duties; every fingerprint
  carries proof-of-emission + evidence-package + citable identity + taxonomy lineage; **the complete biology is
  always computed and persisted, surfaced with progressive disclosure and stable pointers**; the entity is
  sovereign, the operator owns externalities.
- **Process (durable):** the standing is a structure-faithful field; immunosuppress is per-site/visible/
  friction-proportional/aggregate-loud; canonization is team-gated + no-self-witness; partial = residual
  loudness shown; re-confirmation is mechanical where the predicate holds.
- **Outcome (must drift — NOT decreed):** *which* structure-faithful math (and its constants); the policy
  threshold *values* and preset definitions; verbosity levels + report format; the federation platform / DOI
  registrar / venue / ICR shape; the repertoire's own/compose mix.

---

### What this ADR does NOT do

- Does **not** build the stroma-builder, the federation/platform/registrar, the MITRE corpus, **or develop the
  §4 math** (named dependencies / research follow-on — this ADR builds the *one primitive*: a first-class,
  observable, per-site defense, and ratifies the *disciplines*).
- Does **not** gate/block — antigen is a lens, never a compile-blocker.
- Does **not** remove `#[defended_by]` (deprecated alias + codemod).
- Does **not** claim any artifact *is* proof, or that trust is eliminated; many fail-classes are unprovable and
  we say so.
- Does **not** decree which math, nor any threshold value; truth shows the field, thresholds are policy.
- Does **not** police the entity's own risk choices.

---

### Open seams (surfaced honestly)

- **The §4 math is named, not developed** — the faithfulness verification (guard applied term-by-term) and the
  membrane primitive are the math-research follow-on's work; until then §4 ratifies the *discipline + candidates*,
  not finished maths.
- **ADR-024 reconciliation is incomplete** (`clonal` fixed-seed enforcement, `igg` nominal-only limit not yet
  carried through — Hill-cooperativity is the likely closer, pending the math voyage).
- **`Structural` re-check depth** (immediate-body vs effective-behavior) must be pinned in implementation.
- **The worked example + glossary below are part of the ADR** (an ADR whose thesis is "the example IS the
  documentation" must contain one).

---

### Glossary

- **antigen** — a named failure-class; a "bad shape."
- **presents** — a site exhibits an antigen's structural shape.
- **dread / aura** — *declared* marked-unknowns (felt-but-unnameable), on ADR-041's ⊥ plane, OFF the
  classification axis.
- **antibody** — a *recommended mitigation* for an antigen (many may exist; any may suffice).
- **binding (reaction)** — one antibody's reaction to one specific site.
- **observed / asserted** — the proof/trust line: *observed* = the audit can re-check it at this site;
  *asserted* = an honestly-labeled claim it cannot re-check.
- **stroma-node** — a site's stable structural identity (fully-qualified item-path + structural digest); the
  manifold the immune-signal field lives over. *(The codebase-map "stroma" made first-class; reconcile the name
  against the existing `antigen/examples/oracle_lifecycle.rs` usage during the build.)*
- **loudness** — the shown immune-signal value at a site (the field's value; "concentration" only when speaking
  the math; "standing" = the field-value-at-a-site).

### Worked example *(proposed syntax — not yet implemented)*

```rust
// ANTIGEN — a named bad shape (in the repertoire).
#[antigen(name = "panicking-in-drop",
          fingerprint = "impl Drop with unwrap/expect/panic in body")]
pub struct PanickingInDrop;
```
```rust
// PRESENTS — this site exhibits the shape. (id= only when a node has >1 same-class present.)
#[presents(PanickingInDrop, id = "buf-flush")]
impl Drop for Buf { fn drop(&mut self) { self.flush().unwrap(); } }
```
```rust
// ANTIBODY — a recommended mitigation bound to THIS site. Observed: the audit re-checks the shape here.
// `binds` NAMES THE ANTIGEN (compile-checked); `kind = Structural` is from the §1 alphabet.
#[antibody(binds = PanickingInDrop, kind = Structural, references = ["PR#12345"])]
impl Drop for Buf { fn drop(&mut self) { let _ = self.flush(); } }   // the after-state
```

What antigen **shows** (never "defended"): before — full loudness at `buf-flush` (an undefended `presents`).
After — the structural antibody re-verifies at the site, the signal **drops to its steady-state**; any residual
(e.g. the now-swallowed error) shows as *residual loudness*, attributed to *this* site. The summary reads
*"…buf-flush: now has 1 observed antibody, residual loudness low…"* with a pointer into the full map. No "done"
is declared; the current biology is shown; the user's *policy* (not antigen) decides whether the residual gates
their CI.
