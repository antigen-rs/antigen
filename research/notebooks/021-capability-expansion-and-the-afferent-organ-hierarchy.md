# 021 — Ratified Design: The Capability Expansion Law & The Afferent Organ Hierarchy

*The human-side convergence on the two v0.7 draft ADRs (067-amendment, 071),
2026-07-07. Inputs: the deconstruction council ([020](020-adr-067-071-deconstruction-council.md)),
the drafts ([018](018-ADR-067-amendment-DRAFT-the-capability-expansion-law.md),
[019](019-ADR-071-DRAFT-organ-hood-and-the-loop-as-diagnostic-taxonomy.md)), and
a long co-design pass with Tekgy. This notebook is the **authoritative statement
of what we decided** — the source of truth the redrafts of 018/019 and the eventual
`decisions.md` ratification draw from. Several rulings **overrule the council and
both drafts**; those are called out.*

---

## A. ADR-067 amendment — The Capability Expansion Law (final)

### A1. No adoption trigger. Refactor is unconditional; user-impact is what's governed.
**(Overrules both drafts + the council.)** There is **no "window while adoption is
negligible."** The law is: **do what the codebase needs — no tech debt, even at the
cost of user debt** — because user debt is always *handled*, never dumped. Every
refactor discharges its user-impact one of three ways:
1. keep the user-facing API the same (invisible), **or**
2. ship it as a **documented breaking change** version, **or**
3. provide an **old→new migration tool**.

This deletes the "outcome-in-disguise" trigger both council lenses flagged. The
three valves subsume every council worry: published-crate-can't-unpublish →
supersede with a new version; on-disk format break → migration tool;
internal/path-dep consumers (tambear) → migration tool. It's principled semver +
migration-tooling as a standing, non-expiring discipline.

### A2. Refactor via EXPAND-AND-CONTRACT (parallel-change).
The new (stroma-based) implementation lives **alongside** the old, in separate
functions — **both work, old marked `#[deprecated]`.** No in-place breakage.
- **ABSORB** re-points every **internal** caller onto the new path; the old path
  remains only for external back-compat.
- Within a thematic era, everything is **additive + deprecated-but-functional** →
  the whole `0.x.y` line stays cargo-compatible, continuous auto-delivery, zero
  user-facing breaks.
- **If a test breaks:** in expand mode the old code is untouched, so old tests
  *should* pass. A break is a real signal (you touched shared infrastructure) —
  resolve the root cause, don't paper it. New behavior earns *new* tests (the
  born-red witness lives there). *(Tests serve reality.)*

### A3. Breaks are BATCHED at the era boundary (CONTRACT).
At a minor bump (`0.8.0`), a **contract pass removes** all of that era's deprecated
old code and breaks the links. This is **clean because the era's absorbs already
wired everything internal onto the new** — internal removal is a no-op; only
external users who ignored a whole era of deprecation warnings feel it, and they
get an announced, batched, era-boundary break + migration tool. This is *stronger*
than "break anytime with a note": breaks are predictable and the minor bump earns
real meaning.

### A4. Every capability = THREE full expeditions: BUILD → ABSORB → PROVE.
Not a mini-phase — three complete JBD cycles over the **same charter set**, with
charters **re-homed to the next expedition and un-sealed** between passes (or one
expedition re-voyaged 3×). Each expedition-end **tags + ships a release**
(`0.7.0`, `0.7.1`, `0.7.2` for one organ's build/absorb/prove), all additive — with
the **`prove`-tagged release being the one to rely on.**

### A5. ABSORB-done = exhaustive-AND-proven (no assumptions).
Not "the born-red test passes." ABSORB is done when:
- **regression clean** via existing tests — *except* where we broke behavior *on
  purpose*, in which case the old test was asserting the wrong thing → fix it to the
  new intent + ship docs + migration tool (A1);
- the **full JBD cycle** has been run on the new component —
  *dream → research → deconstruct → converge → test → build → survey → document*
  everything it enables. There is **no hard end**; the soft signal is *"we've done
  exhaustive-enough work that finding more needs user feedback or another dream
  wave";*
- **proven total coverage** — the entire codebase, the whole doc suite, all
  help-line output, doc comments, line comments, parameter names, everything —
  *checked* to confirm nothing was missed. **No assumptions. Proven.** (Antigen's
  own dogfood ethos turned inward: you don't *claim* coverage, you scan for it.)

---

## B. ADR-071 — Organ-hood & "the loop is edges, not nodes" (final)

### B1. Single decision: THE LOOP IS EDGES, NOT NODES.
**(Collapses the draft's three decisions into one; resolves the ROUTE blocker.)**
The ADR-037 control loop is the **connective flow-structure** — edges: how signal
moves, what feeds what, the coordinates a capability sits at. **Organs are the
nodes** — the built things, individuated by *build cost + blast-if-absent*, never
by which stage they occupy. Everything derives:
- don't cut organs along the loop (old D2) = the principle for organs;
- the control-plane items are *indexed-by*, not *constituted-by*, stages (old D3);
- the split-test = the individuation rule's contrapositive at finer grain;
- known-completeness = enumerate every coordinate (even empty ones).

**Shared-coordinate arbitration** (the old ROUTE blocker): a node sitting where two
tracks meet is **ONE build unit** (a node) at a coordinate both consume; sharing a
coordinate is not being individuated by it. The **organ track owns the external
interface contract; the control-plane hooks on top** (consumer, not co-owner).

### B2. Organ-hood admits LATENT organs (the afferent hierarchy).
**(The big model correction.)** Organs form an **afferent hierarchy**, exactly like
biology: tier-0 organs sense **raw stroma signal** (cochlea→sound,
vestibular→motion); **latent organs sense from *other organs' outputs*** — they
"sense from the senses." A higher integrator fuses lower outputs to compute what
neither knows alone (sound + head-motion → "is that pitch-shift real or am I
turning?"); a higher-still controller acts on the integrated estimate (balance,
righting responses).

So the organ-hood law reads: **an organ's afferent input may be a raw stroma signal
OR another organ's written-back output — both are just "reading a fact."** A latent
organ is still a **node**; its inputs are **edges from other nodes** (keeps B1
clean). This generalizes the council's F9 finding: the four control-theory
self-regulation organs (observability / controllability / delay / stability) are
*latent organs aimed inward* — sensing antigen's own operational outputs. One tree,
spanning outward (higher user-code patterns) and inward (self-regulation).

### B3. Beneficiary-absorb = distinct beneficiary-SET (fixes greenfield exclusion).
Conjunct (b) means **"has a distinct set of consumers,"** not "has an existing
consumer to re-point" — the latter wrongly excludes greenfield organs
(self/non-self re-points nothing). "Who consumes it" is *individuation*; the ABSORB
migration is a *separate lifecycle duty* (A2). Individuation-law and split-test are
kept distinct (belongs-iff belongs only to the split-test / refinement rule).

### B4. Re-warrant on build-space ≠ fault-space.
"The loop doesn't individuate organs" is grounded on the **non-injective map
between build-space (cost/blast) and fault-space (the loop's failure partition)**,
NOT on `decisions.md:9808` (which is about the user-code disturbance genus — demote
it to a supporting analogy).

### B5. The capstone is the recursive LATENT-ORGAN TREE — not a join.
**(Overrules both drafts.)** Base organs pre-wire during their own absorbs, so
there's no leftover "join." The capstone is the **afferent tree above tier-0**:
integrator organs + controller organs, each **sensing from organs**, recursively,
plus the orchestration and the meta-tooling (macros, syntax, CLI, integrations) the
full set makes possible. It is **open-ended / multi-wave**, climbing tiers until the
**fixpoint** — combining outputs yields nothing new.

### B6. Signal-algebra = the substrate the latent tree stands on.
For a latent organ to sense from another organ, that organ's **output must be
written back into the stroma as a first-class fact/edge.** The "dimensional climb"
is successive **read → derive → write-back** rounds; dimensionality = tree depth.
So **signal-algebra is load-bearing, not nice-to-have** — it gets its **own
build/absorb/prove expedition, landing (built + absorbed) before the latent tree.**

### B7. Honest scope + known-completeness.
- The organ map is a **design decomposition, not a parallel build decomposition,
  until the stroma closure ships** (the closure gates the tier-0 organs — confirmed
  real, non-stale, per the scout: `query.rs:25/35/47/61` + `scip.rs:36`).
- "Known-complete" softens to **complete w.r.t. ADR-037's *open* six-stage frame**;
  name all four F9 control-theory latent organs as structurally-guaranteed
  (Ashby), on_hold for the latent era.

---

## C. Semver as organizational clarity — the era model

Version numbers are **legible as project-phase** (co-native design applied to
versioning). The era boundary does **double duty** — technical *contract* (remove
the era's deprecations) **and** thematic cut (new phase) — and they **coincide by
design**, keeping it cargo-honest (the theme cut *is* a real break).

- **`0.7.x` era = the whole immune organism** — stroma (done) + tier-0 base organs
  (raw-signal sensing: self/non-self, effector tiers, suppression, sense, hooks,
  feedback, teeth) **+ signal-algebra + the latent-organ tree** (integrators +
  controllers that sense from the base organs, incl. the four F9 self-regulation
  organs). All **one additive climb**: each capability's build/absorb/prove voyages
  ship `0.7.x` releases (`0.7.10, 0.7.11 …`) to the **fixpoint** (combining organ
  outputs yields nothing new). The latent tree is *still organ-building* — higher-
  order sensing, same theme — so it stays `0.7`, **not** a new era. *(Corrects an
  earlier over-split that put the latent tree in `0.8`; the latent organs "sense
  from the senses" but are the same theme, and `0.8` was already claimed.)*
- **`0.8` = open / TBD** — a *future* boundary: contract `0.7`'s accumulated
  pre-stroma deprecations **+** whatever the next theme turns out to be. Not
  defined now.
- **Platform / value / multi-tenant** (universal-stroma, the fleet, view-language,
  governance) = **distant horizon**, un-pinned from any specific number. *(It had
  squatted on `0.8`; it does not — it's further out.)*

**Semver mechanics settled:** patch/minor are **integers, not digits** —
`0.7.10 > 0.7.9`, no rollover, nothing forces `0.8`. The whole organism climb stays
`0.7.x` as long as it's additive (which it is — expand + deprecate); the minor steps
to `0.8` **only** at a deliberate *future* contract, theme TBD.

---

## D. What this changes downstream

1. **Redraft 018 (ADR-067)** into A1–A5 (the law: unconditional refactor +
   expand/contract + era-batched breaks + build/absorb/prove + exhaustive-proven
   absorb-done).
2. **Redraft 019 (ADR-071)** into B1–B7 (loop-is-edges single decision + latent
   organs + capstone-as-latent-tree + signal-algebra substrate).
3. **Reshape the v07 island map**: each organ → build/absorb/prove triple; add the
   **signal-algebra** expedition (closes 0.7); the **capstone** node → the
   **0.8 latent-organ tree** (open-ended) + a **contract expedition** at the era
   boundary; F9's four self-regulation organs named on_hold in the latent era.
4. **Formal ratification** into `decisions.md` via the process ceremony — this
   notebook is the convergence input, not the ratified text.

---

## E. Certification amendments (2026-07-07, author-distinct panel)

The ratification ceremony's author-distinct panel (aristotle + adversarial certify; the
drafter excluded — the no-self-witness invariant) returned one word-fix and two BLOCKs; all
resolved, co-captain-blessed, and carried canonically in the ratified `decisions.md` entries
(ADR-067 Amendment 1 + ADR-071). Recorded here (append-only, supersede-not-erase) so the
design substrate matches the ratified text:

- **Aristotle word-fix — A5 "proven" → "witnessed-complete."** "Proven total coverage"
  over-reaches antigen's own three-valued / self-distrust discipline (a scan is a
  *witness-to-a-tier*, not a QED). ABSORB-done's coverage is **witnessed-complete (scanned,
  tier-tagged, not assumed)** — admit no *unscanned* assumption, never claim completeness the
  tool cannot prove.
- **Adversarial BLOCK-1 — the behavioral-contract gap (Hyrum's Law).** A1's valve (a) "keep
  the API the same" covered only the *typed* surface; observable behavior (log/error formats,
  ordering, defaults, span names) can break while (a) "fires." Fix: valve (a) = the
  **observable** contract is unchanged; a behavioral change ships via valve (b) with a
  **behavior-change note** even when the typed API is untouched; plus an honest **Hyrum's-Law
  residual** (undocumented reliance can't be fully enumerated — the note mitigates, doesn't
  guarantee).
- **Adversarial BLOCK-2 — the afferent hierarchy admits cycles (the fixpoint was
  ungrounded).** Write-back-as-facts let an organ feed its own upstream (A→B→A), so the
  "tree"/fixpoint claim needed acyclicity. Fix — the **acyclicity invariant (B2′):**
  signal-algebra enforces an organ writes facts at a **strictly higher tier than any it
  reads** (afferent = read-lower / write-higher, up the DAG); **feedback/control is efferent —
  a separate channel, not afferent input**; the fixpoint is grounded on the DAG's finite
  tiers, and oscillation-risk lives in the efferent loop (the F9 stability-margin organ's
  beat), never the afferent tree.
- **Advisories (folded in):** the shared-coordinate node individuates by the **organ track's
  blast** (self-regulation is a consumer view, not a second individuation); greenfield organs
  (idle beneficiary-set at birth) individuate by **afferent-source / build-cost** via the
  OR-witness; the **write-back-as-facts step carries ADR-005 sub-clause F** (a new trust
  boundary — provenance tiers + parity surveillance validate the written fact before a higher
  organ extends trust to it).
