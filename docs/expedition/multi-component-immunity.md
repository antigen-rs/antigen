# Multi-Component Structural Immunity

> **Status**: Deep-dive draft, V0 (2026-05-11). Authored by team-lead
> (Claude Opus 4.7) following extended conversation with Tekgy preserved at
> `multi-component-immunity-conversation.md`. Comprehensive best-read with
> explicit seams for biology-cognate refinement (naturalist), additional
> components (scout), attack-surface analysis (adversarial), and Phase 1-8
> deconstruction (aristotle).
>
> **Where this lives in the lifecycle**: expedition substrate. Not project-
> tier yet. Promoted to `docs/multi-component-immunity.md` after team Phase
> 1-8 + ratification per `process.md`.
>
> **Status of enumeration**: PROVISIONAL. Six components currently named;
> enumeration is explicitly open per recognition-not-design (ADR-006).
> Future-readers and team-members are invited to find more components,
> refine cognates, surface relationships, and identify what we haven't
> named.
>
> **Relationship to existing project framings**:
> - Extends "antigen catches failure-class memory" without replacing it
>   (that framing remains valid as the floor concept).
> - Extends "antigen is a tool" without replacing it (the tool component
>   is one of several).
> - Extends "antigen composes with existing ecosystem tools" (ADR-002)
>   without replacing it (composition is the architectural property; the
>   component fabric is the architectural shape).

---

## Part I: The vocabulary as spine

Antigen is centrally a **vocabulary**. The macros, the cargo subcommand, the
scan/audit logic, the cross-crate enumeration, the ADR-grounded discipline,
the biology cognate — all of these are components hanging off a shared
vocabulary. The vocabulary is what makes the components composable; the
vocabulary is what makes the project co-native with both human and LLM
collaborators; the vocabulary is what gives the project its coherent
identity across instantiations.

**Why vocabulary first?**

A team can adopt the vocabulary without adopting any specific tooling. A
single declaration of `#[antigen(name = "X", summary = "...")]` in a Rust
file is antigen-the-vocabulary in operation — even with `cargo antigen scan`
never run. The team has *named a failure class*; they have *given it
structural memory*; they have made it *legible to future-readers including
LLM collaborators*. The tool would extend this; the discipline would extend
this; the ecosystem would extend this. But the floor is the vocabulary
itself.

This is structurally different from how most software-engineering tools are
described. Most tools are *primary*; the vocabulary they enable is
*secondary*. Antigen inverts this: the vocabulary is primary; the tool is
one component among several that operates on the vocabulary.

**What this means for the architecture:**

- The vocabulary is the *interface contract* between components. Component 1
  (dev-judgment) and Component 2 (passive scan) both operate on the same
  vocabulary. Component 3 (test-integration) reads witnesses from the
  vocabulary. Component 4 (knowledge-ecosystem) attaches references to the
  vocabulary's primitives. Etc.
- New components compose with existing ones *through the vocabulary*. A
  hypothetical future component (e.g., real-time CI feedback, or per-PR
  antigen surface diff, or cross-organization antigen sharing) attaches to
  the same vocabulary; doesn't require restructuring of existing components.
- The vocabulary is the *thing the project ships*, not the tool. The
  `antigen` and `antigen-macros` crates ship the vocabulary; `cargo-antigen`
  ships one tool component; future crates may ship other components.

**The biology cognate**:

In biology, "the immune system" is not a single mechanism but a *protocol* —
a set of shared molecular signals (MHC presentation, cytokine cascades,
antibody isotypes) that wildly different cell types use to coordinate. The
protocol is what makes innate and adaptive immunity composable; the
protocol is what makes B cells, T cells, NK cells, dendritic cells, and
macrophages able to cooperate despite their wildly different mechanisms.

Antigen-the-vocabulary is the protocol. The components are the cell types.
[NATURALIST: refine this cognate — is "protocol" the right word for what
MHC + cytokine + antibody-isotype shared signaling actually IS in biology?
Is there a sharper term?]

---

## Part II: The six components (provisional enumeration)

### Component 1: Dev-in-the-loop immunity

**What it does**

The developer writes antigen declarations into their Rust code based on
their judgment of what failure classes exist, where vulnerabilities sit,
what's protected and how, and how the failure classes inherit. This is
*production of immunity through human cognition* — the team knows
something is dangerous, names it, and makes it structural.

**The five vocabulary primitives:**

- `#[antigen(name = "...", summary = "...", references = [...])]` —
  declares a failure class exists, with summary describing what it is and
  references pointing to its lived context (CVEs, RFCs, ADRs, post-mortems).
- `#[presents(antigen_name)]` — marks a code site as vulnerable to a
  declared antigen. Acknowledges the failure-class memory at the point of
  exposure.
- `#[immune(antigen_name, witness = ...)]` — claims a code site is
  protected against a declared antigen, with the witness identifying *what
  protects it* (a test, a proptest, a formal proof, a phantom-type pattern,
  a lint).
- `#[descended_from(parent)]` — declares this antigen inherits structure
  from a parent antigen; presentations and witnesses propagate.
- `#[antigen_tolerance(antigen_name, rationale = "...", until = ...)]` —
  team-acknowledged tolerance of a known antigen instance; declares "we
  see this, we accept it for now, here's why."

**The discipline side**

Dev-in-the-loop is *primarily* a discipline component. The developer's
judgment is the production mechanism. The discipline trains the team to:
- Notice failure-classes in their codebase before they bite
- Name failure-classes in shared vocabulary (recognition-not-design)
- Ground each declaration in real instances (ADR-006 threshold)
- Provide rationale for tolerances (ADR-005 Amendment 2)
- Maintain `descended_from` lineage as code evolves

**The tooling side**

The proc-macros (`antigen-macros` crate) provide the syntactic surface.
The macros don't *produce* the declarations; they *parse* them, *validate*
them, and *make them structurally present* to other components. The
tooling is in service of the discipline, not in replacement.

**Floor / ceiling**

- **Floor**: a single `#[antigen(...)]` declaration in a single file. The
  team has named one failure class. That alone provides structural memory
  that didn't exist before.
- **Ceiling**: comprehensive antigen taxonomy with rich rationale, full
  `descended_from` lineages, version-pinned references, every vulnerable
  site marked `#[presents]`, every protected site marked `#[immune]`,
  every accepted-known-risk marked `#[antigen_tolerance]`.

**Biology cognate**

Closest to **deliberate vaccination + informed prior exposure**. The team
*chooses* to develop immunity to specific failure classes based on
informed judgment. This is humoral adaptive immunity at the deliberate
end — the body (team) develops specific antibodies (antigens) based on
exposure (lived experience) and prior knowledge (references).

[NATURALIST: refine. Is this closer to vaccination, to memory B-cell
maintenance, to T-helper coordination of B-cell maturation? Is there a
biology cognate for the *judgment* aspect — the cell deciding what
warrants memory? Or is that the wrong question because biology doesn't
have judgment?]

**Value-prop**

The team's collective judgment about danger becomes *durable structural
memory* in the codebase. New team members inherit the failure-class
awareness by reading the antigen declarations. LLM collaborators read
the same declarations the same way humans do. Tribal knowledge becomes
explicit and survives team turnover.

**Failure modes / attack surface**

- **Mis-named antigen**: team declares an antigen for a failure-class
  that's actually a different shape. Subsequent presentations are
  miscategorized; immunity claims defend against wrong things.
- **Speculative antigen**: team declares an antigen without grounding in
  real instances. ADR-006 violation. The declaration becomes noise.
- **Stale rationale**: `#[antigen_tolerance(... rationale = "X")]` where
  X no longer applies. The tolerance becomes a "trust me" comment.
- **Lineage drift**: `descended_from` chain that no longer reflects the
  actual inheritance structure of failure-classes.

[ADVERSARIAL: deeper attack-surface analysis. What about *malicious*
antigen declarations — a contributor declares an antigen that's actually
the inverse of the failure class, suppressing real bugs by mis-presenting?
What about declaration injection through proc-macro generation
(ADR-014 territory)?]

**Connection to other components**

- Component 2 (passive scan) *recognizes* the declarations Component 1
  produces; without recognition, the declarations are inert.
- Component 3 (test-integration) *consumes* the `witness = Y` field
  Component 1 sets.
- Component 4 (knowledge-ecosystem) *follows* the `references = [...]`
  Component 1 attaches.
- Component 5 (version/lineage) *traverses* the `descended_from`
  edges Component 1 declares.
- Component 6 (cross-crate) *propagates* Component 1's declarations
  across crate boundaries.

Component 1 is the *production source* — most other components consume
what it produces. Floor-mode antigen (vocabulary-only) is essentially
Component 1 alone, with the other components mostly inactive.

**Substrate locations**

- Macros: `antigen-macros/` crate
- Parser: `antigen-macros/src/parse.rs`
- Vocabulary lock: `docs/glossary.md`
- Discipline references: ADR-001, ADR-004, ADR-005 Amendment 2, ADR-011,
  ADR-014

---

### Component 2: Passive scan/lint/tool immunity

**What it does**

Automated walks of the codebase find antigens, find presentations,
verify witnesses, detect lineage cycles, surface unaddressed presentations,
detect fingerprint matches in unmarked code. *Recognition through
structural analysis* — the tool does what the team would have to do by
hand, at scale, on every build.

**Concrete operations:**

- `cargo antigen scan` — walks workspace, collects all antigens,
  presentations, immunities, tolerances, lineage edges; produces
  `ScanReport` with structured data.
- `cargo antigen audit` — verifies witness validity at each immunity
  site; reports witness tier honestly (FormalProof / ExecutionVerified /
  Reachability / ExternalUnvalidated / Missing per ADR-005 Amendment 3);
  surfaces unaddressed presentations.
- Fingerprint engine — matches structural patterns (item-level operators
  via syn; body-level operators via ast-grep subprocess per ADR-015)
  against unmarked code to surface *latent* presentations the team hasn't
  yet marked.
- Cycle detection — guards `#[descended_from]` from infinite loops
  (ATK-A3-002).
- Diamond inheritance dedup — handles multi-parent lineage correctly
  (ADR-018).

**The discipline side**

Minimal. The tool runs; the team reads the output. Some discipline
*around* the tool: read the audit report, address unaddressed presentations,
respond to surfaced fingerprint matches. But the tool's value lands without
team participation.

**The tooling side**

This is *primarily* a tooling component. The cargo subcommand, the scan
walker, the audit logic, the fingerprint engine, the cycle detector — all
shipped code that operates on the vocabulary Component 1 produces.

**Floor / ceiling**

- **Floor**: `cargo antigen scan` once, look at the report. Even without
  any team action, the team now sees structural memory laid bare.
- **Ceiling**: `cargo antigen scan` in CI, audit-gated PRs, fingerprint
  matches surfaced inline, scan-failure rejects on commit, audit-report
  diffs visible per-PR.

**Biology cognate**

Closest to **innate immunity** — generic pattern-recognition that operates
automatically without specific prior exposure. The body's pattern-recognition
receptors (PRRs) detect generic molecular patterns (PAMPs) without needing
to have seen the specific pathogen before. Similarly, the fingerprint
engine detects structural patterns in code without needing specific prior
team-declaration of those patterns.

[NATURALIST: refine. Is innate immunity the right cognate, or is this closer
to complement system (constitutive surveillance)? Or to the cellular
machinery (proteasome, MHC class I presentation) that constitutively
processes any cellular protein for surface display? The latter would map
to "the scanner walks the whole codebase by default" — a constitutive
surveillance mechanism rather than a triggered one.]

**Value-prop**

Structural memory gets *operationalized* without team effort. The tool
catches what the team would have to manually check on every build. The
audit report becomes a structural truth about the codebase that's always
current.

**Failure modes / attack surface**

- **Tool says clean when substrate is broken**: scan or audit fails to
  surface a real failure-class instance. Sub-clause F violation.
- **Fingerprint false-positive autoimmunity**: fingerprint matches against
  non-vulnerable code, generating noise that drowns real signal.
- **Audit tier over-claiming**: audit reports FormalProof tier when
  underlying verification is only ExecutionVerified (ADR-005 Amendment 3
  is the discipline against this).
- **Cycle detection false-pass**: pathological input bypasses cycle guard
  (ATK-A3-002 is the contract).
- **Crash-resistance violation**: scan/audit crashes on legitimate-but-
  pathological input (ADR-005 Amendment 3 mechanics §3).

[ADVERSARIAL: deeper analysis. Trust boundary is at `enumerate_dep_crate_roots`
post-ADR-017 — what attacks bypass it? What about poisoned `cargo metadata`
output? What about race conditions between scan and source mutation?]

**Connection to other components**

- Consumes Component 1's declarations as input.
- Provides input to Component 3 (test-integration reads witness fields
  from scan output) and Component 4 (knowledge-ecosystem follows
  references from scan output).
- Operates on Component 5's lineage graph (cycle detection, propagation
  walk).
- Operates on Component 6's cross-crate substrate (cargo metadata
  walk; canonical_path resolution).

**Substrate locations**

- Scanner: `antigen/src/scan.rs`
- Auditor: `antigen/src/audit.rs`
- Cargo subcommand: `cargo-antigen/`
- Fingerprint engine: per ADR-015, evaluator-trait abstraction across
  syn-based item operators (W6a) and ast-grep-subprocess body operators
  (W6b; deferred to v0.2)
- Discipline references: ADR-001, ADR-002, ADR-005, ADR-010, ADR-015,
  ADR-017, ADR-018

---

### Component 3: Test-integration immunity

**What it does**

Witnesses link to actual tests. Test history becomes immune history. The
audit's verification of immunity claims grounds in real behavioral
confirmation. *Verification through behavioral observation* — the
structural memory connects to the runtime confirmation that the protection
actually works.

**The witness vocabulary:**

- `witness = test::function_name` — links to a `#[test]` function whose
  passing demonstrates the immunity.
- `witness = proptest::function_name` — links to a property-based test
  whose passing demonstrates the immunity across a generated input space.
- `witness = clippy::lint_name` — delegates to a clippy lint whose absence
  of warnings demonstrates the immunity (compose-don't-compete per
  ADR-002).
- `witness = kani::function_name` / `prusti::...` / `verus::...` /
  `creusot::...` — delegates to formal-verification tools.
- Phantom-type witnesses (ADR-013) — the witness IS the type structure
  itself, not a separate verification artifact.

**WitnessTier gradient** (per ADR-005 Amendment 3 + ADR-007):

- **FormalProof** — mathematically verified by a formal-method tool
- **ExecutionVerified** — passing runtime verification (test/proptest)
- **Reachability** — function is in scope but not yet executed during
  scan (covers static visibility without runtime check)
- **ExternalUnvalidated** — witness type accepted but tool integration
  not yet verified
- **Missing** — no witness present or witness fails to resolve

**The discipline side**

The team must *write the tests* (or the proofs, or the proptests). The
team must keep witnesses *aligned* with the antigens they defend against —
when an antigen evolves, the witness may need to evolve too. The team must
*honestly report tier* (audit-tier-honesty per ADR-005 Amendment 3).

**The tooling side**

The audit detects external tool prefixes, resolves witnesses, validates
their existence and applicability, reports tier honestly. The tier
gradient is enforced by audit logic; the audit doesn't *create* the
verification — it *recognizes* what verification actually happened.

**Floor / ceiling**

- **Floor**: `#[immune(X, witness = test::tx_test)]` where `tx_test` is a
  passing `#[test]`. Verification at ExecutionVerified tier; structural
  memory connects to behavioral confirmation.
- **Ceiling**: every immunity claim across the codebase has a tier-
  appropriate witness; FormalProof tier where possible; coverage of all
  declared antigens; cross-version witness validity tracked across
  `#[descended_from]` boundaries (Component 5 territory); witness identity
  is presentation-keyed (per ADR-018).

**Biology cognate**

Closest to **memory B-cell binding confirmation**. A memory B-cell holds
an antibody specificity (the witness), and the actual binding event
(test execution) confirms the specificity is operationally correct against
the antigen-as-encountered. Without the binding event, the antibody is
*declared* but not *verified*. The binding event is what produces the
calibrated confidence.

[NATURALIST: refine. Memory B-cells specifically? Or is this closer to
the affinity-maturation cycle (B-cell hypermutation followed by selection
against antigen)? Or is the binding confirmation closer to T-cell receptor
recognition? What's the right resolution at biology's side for "behavioral
confirmation that the structural memory matches reality"?]

**Value-prop**

Immunity claims become *verifiable*. The audit can answer "is this site
actually protected?" with grounded tier information, not just declared
intent. Tier-honesty (per ADR-005 Amendment 3) prevents the audit from
producing false confidence.

**Failure modes / attack surface**

- **Witness drift**: witness was valid when written, no longer applies as
  antigen or function evolved. The audit may continue reporting "well-
  formed" when the verification is stale.
- **Tier over-claiming**: declaring FormalProof when underlying tool only
  provides ExecutionVerified.
- **Witness-as-placeholder**: writing `witness = test::todo_test` where
  todo_test always passes. The audit reports well-formed but the
  verification is theatrical.
- **External-unvalidated proliferation**: writing `witness = kani::...`
  without kani actually being run in CI. Tier should be ExternalUnvalidated;
  if reported as FormalProof, audit-tier-honesty is violated.

[ADVERSARIAL: deeper analysis. What about tests that pass but don't
actually exercise the antigen's failure mode (test pass-but-not-meaningful)?
This is the "test technically passes but no longer means what it should"
class — possibly its own attack surface. ATK-A2-003/004/005/011/012
covered this.]

**Connection to other components**

- Consumes Component 1's witness declarations.
- Operates within Component 2's audit pipeline.
- Witness *identity* is presentation-keyed (per ADR-018 Open Question 4 →
  Tekgy ratification) which connects to Component 5's lineage propagation.
- Cross-crate witnesses (Component 6) require canonical_path resolution.

**Substrate locations**

- Audit logic: `antigen/src/audit.rs`
- Witness recognition: `audit.rs::detect_external_tool`
- WitnessTier definition: per ADR-005 Amendment 3, ADR-013
- Discipline references: ADR-002, ADR-005, ADR-007, ADR-013, ADR-018

---

### Component 4: Knowledge-ecosystem immunity

**What it does**

References attached to antigen declarations point to the lived context
where the failure-class was learned, discussed, decided about, fixed,
documented. PR threads, post-mortem blog posts, git issues, manual pages,
ADR/DEC files, internal tutorials, CVEs, RFCs, papers. *Contextual memory
linking lived history* — the structural memory in code connects to the
distributed knowledge across the team's communication ecosystem.

**Mechanism:**

- `#[antigen(..., references = [...])]` — open-vocabulary list of pointers
  per ADR-009 (Layer 2). Examples:
  ```rust
  #[antigen(
      name = "PanickingInDrop",
      summary = "Drop implementation that may panic, violating drop safety",
      references = [
          "rfc:RFC-1857",
          "url:https://blog.rust-lang.org/drop-panic-postmortem.html",
          "pr:owner/repo#1234",
          "issue:owner/repo#567",
          "adr:project/internal-ADR-042",
      ],
  )]
  ```
- Rationale fields on tolerance and immunity primitives — narrative
  justification that lives next to the structural declaration.
- Eventual: bidirectional links — the *referenced* artifact (PR, blog post)
  can carry a back-link to the antigen declaration, making the connection
  legible from both sides.

**The discipline side**

Team writes references when declaring antigens. Team keeps references
*current* — when a referenced artifact moves, archives, or becomes
superseded, the reference is updated. Team treats antigen declarations as
*the connection point* between code and lived history; new failures get
linked to existing artifacts rather than re-discussed in isolation.

**The tooling side**

Currently minimal in v0.1. The `references` field exists in the
vocabulary; the scan collects it; the audit doesn't yet *validate* the
references resolve or *follow* them. Future tooling could:
- Validate URL references resolve (no dead links)
- Cross-link with git issue trackers
- Cross-link with Slack archives (if team chooses to expose them)
- Cross-link with ADR/DEC directories
- Surface "antigens that share references" — failure-class clusters that
  the same lived context informs

**Floor / ceiling**

- **Floor**: any antigen declaration with at least one reference. The
  failure class has a connection-point to lived context.
- **Ceiling**: comprehensive bidirectional references; every antigen
  links to its lived context; lived context links back to its antigen
  declarations; cross-team antigens share references; references span
  CVEs/RFCs/papers/internal-tickets/post-mortems/PRs.

**Biology cognate**

This one doesn't have a clean single cognate at my current resolution.
Several candidates:

- **Cytokine signaling network** — the immune system's *messaging fabric*
  that coordinates between cells. Chemokines, interleukins, interferons.
  This produces *contextual coordination* — cells respond differently
  based on signals from other cells about what's been seen elsewhere. The
  knowledge-ecosystem framing maps loosely: antigen declarations carry
  signals (references) from the broader knowledge environment.
- **Complement system** — a cascade of plasma proteins that amplify
  immune signals. Generic but context-amplifying.
- **Lymphatic system architecture** — the *anatomy* of where immune
  signals travel. Which lymph nodes drain which tissues. The
  knowledge-ecosystem analog: the *architecture* of where lived knowledge
  lives across the team's tools.
- **Antigen presentation context (MHC class II)** — antigens are
  presented IN CONTEXT (with co-stimulatory signals); presentation
  without context is anergy. The reference list provides *context for
  presentation*.

[NATURALIST: critical refinement wanted. This component's cognate is the
loosest in the current enumeration. What's the right biology cognate for
"distributed contextual memory across heterogeneous knowledge substrates"?
Is there one? Or is this where biology goes silent (per the
metaphor-as-instrument discipline)? Boundary-silence here would be data —
biology may not have a sharp analog because human-knowledge-ecosystems
don't have direct biological precedent.]

**Value-prop**

Antigen declarations stop being isolated structural facts and become
*nodes in a knowledge graph* that connects to the team's full
communication and decision substrate. Future developers (including LLM
collaborators) can follow references to understand *why* an antigen exists,
*what* the lived context produced it, *who* engaged with it. The
"lived history of the code in brief, while maintaining the most rich
context" property (Tekgy's framing) lands at this component specifically.

**Failure modes / attack surface**

- **Stale references** — links to artifacts that no longer exist, moved,
  or now describe different content.
- **Fake references** — references that point to plausible-looking but
  fabricated artifacts.
- **Poisoned external references** — a CVE or RFC that's been
  recategorized, redacted, or superseded.
- **Reference noise** — too many references per antigen; signal drowns.
- **Cross-reference loops** — antigen A references blog post that
  references antigen A.

[ADVERSARIAL: deeper analysis wanted. Knowledge-ecosystem is where most
of the *trust* surface lives because the references are inherently external.
What's the threat model for a malicious contributor who attaches misleading
references to legitimate antigens? For a compromised external knowledge
source? For an LLM collaborator that hallucinates references?]

**Connection to other components**

- Consumes Component 1's `references = [...]` declarations.
- Component 2's scan collects references; audit doesn't yet validate them
  (potential future tooling extension).
- Component 5's version/lineage interacts with references — a reference
  may be version-specific (a CVE for a specific version of a dependency).
- Component 6's cross-crate may carry references across crate boundaries.

**Substrate locations**

- Vocabulary: `#[antigen(..., references = [...])]` per ADR-009 Layer 2
- Discipline references: ADR-001, ADR-004, ADR-005 Amendment 2, ADR-009
- Future tooling extensions: deferred substrate / encounters-tier

---

### Component 5: Cross-version / lineage immunity

**What it does**

`#[descended_from]` chains track how failure-classes inherit, evolve, and
specialize across antigen declarations. Temporal recognition surface
(ADR-016) tracks *when* immunity was established, *what version* was
verified, *how* immunity has evolved. Version-boundary-as-feature (ADR-017)
treats version transitions as recognition opportunities. *Evolutionary
memory across change.*

**Mechanism:**

- `#[descended_from(parent_antigen)]` — child antigen inherits structural
  shape from parent; presentations propagate; witnesses propagate (subject
  to re-validation per ADR-005 Decision item 2); tolerances propagate.
- Diamond inheritance dedup (ADR-018) — multi-parent lineage handled
  correctly.
- `verified_at` (ADR-016 temporal field) — when was immunity last
  affirmed?
- `canonical_path` in form `name@version` (ADR-017) — version-aware
  identity at the cross-crate boundary.
- Version-boundary-orphans-as-feature — when a dependency's antigen
  declaration moves between versions, the orphan-lineage-edge isn't a bug;
  it's an explicit signal that a recognition surface has changed.

**The discipline side**

Team maintains `descended_from` chains as failure-class taxonomy evolves.
Team re-validates inherited witnesses at descendants (sub-clause F at the
inheritance trust-boundary). Team treats version transitions as
recognition opportunities, not just as compatibility-shim work.

**The tooling side**

Scan collects lineage edges. Cycle detection guards against infinite
loops. Synthesis pass propagates presentations across descended_from
(with diamond dedup). Audit validates inherited witnesses still apply.
`orphaned_lineage_edges()` query method surfaces declarations whose
parent no longer exists.

**Floor / ceiling**

- **Floor**: a single `#[descended_from(Parent)]` declaration. Failure-
  class taxonomy starts.
- **Ceiling**: rich inheritance trees, version-aware identity throughout,
  temporal recognition surface tracking when verification last happened,
  proper handling of version-boundary transitions, cross-version
  re-validation, isotype-switching equivalent at the witness-evolution
  level.

**Biology cognate**

Multiple cognates converge here:

- **Antibody class-switching** (isotype switching, IgM → IgG → IgA → IgE) —
  the same B-cell lineage produces different antibody forms specialized for
  different roles while preserving recognition specificity. Cognate to
  `descended_from` with refinement: the child antigen has the parent's
  recognition shape but specialized application.
- **B-cell hypermutation** (somatic hypermutation in affinity maturation) —
  small variations in the antibody's CDR regions produce slightly different
  specificities that compete for antigen binding. Cognate to fingerprint
  refinement across `descended_from` lineages.
- **Memory B-cell vs. plasma B-cell differentiation** — same lineage,
  different roles. Memory B-cell retains the recognition for future
  encounters; plasma cell produces antibody for current response. Cognate
  to antigen-as-memory vs. presentation-as-instance.

**Specifically for version-boundary-as-feature** (the recently surfaced
naturalist correction from drift/shift):

This is **antigenic drift / antigenic shift** in immunology. When a
pathogen mutates enough to escape memory B-cell recognition, the immune
system encounters it as new. Antigenic shift is *bigger* than drift —
genome reassortment producing categorically new recognition surface. The
cognate for version-boundary in antigen-the-project is most cleanly
antigenic drift (small version updates) or antigenic shift (major version
boundary). The orphan-lineage-edge IS the body recognizing a previously-
known antigen as no-longer-matching-memory because the drift has been
large enough.

[NATURALIST: extend. The drift/shift framing was your prior correction —
how deep does the cognate go? Are there other version-boundary phenomena
in immunology beyond drift/shift? What about the body's *retraining*
process when shift occurs — does it predict something we should be
building in Component 5?]

**Value-prop**

Failure-class memory becomes *durable across change*. When code evolves,
when dependencies version, when failure-classes specialize — the lineage
structure preserves the connection between past and present
recognition. Re-validation discipline (sub-clause F at the inheritance
boundary) prevents inherited witnesses from going stale silently.

**Failure modes / attack surface**

- **Lineage cycle** — A descended from B descended from A; non-termination
  attack (ATK-A3-002, guarded by cycle detection).
- **Stale lineage** — `descended_from(Parent)` where Parent no longer
  exists or has been semantically replaced.
- **Witness staleness across descent** — inherited witness valid at parent
  but no longer at descendant; ADR-005 Decision item 2 requires
  re-validation.
- **Diamond inheritance unintended-dedup** — two paths through diamond
  that *should* produce distinct propagations (per ADR-018 ProvenanceEntry
  with canonical_path) get collapsed; provenance destroyed.
- **Version-boundary autoimmunity** — a version transition incorrectly
  flagged as still-matching, causing immunity claims from old version to
  be (wrongly) trusted against new version's antigen surface.

[ADVERSARIAL: ATK-A3-006 (orphan edge canonical_path false resolution)
and ATK-A3-010 (drift-vs-waning audit message category error) cover some
of this surface. Are there attack-surface gaps for the diamond
ProvenanceEntry mechanism specifically? For cross-version witness
re-validation specifically?]

**Connection to other components**

- Operates on Component 1's `#[descended_from]` declarations.
- Component 2's scan walks lineage; cycle detection guards.
- Component 3's witness re-validation operates across lineage descents.
- Component 4's references may be version-specific.
- Component 6's cross-crate lineage edges carry canonical_path with
  version (`name@version`).

**Substrate locations**

- Lineage edge type: `antigen/src/scan.rs::LineageEdge`
- Propagation walk: `antigen/src/scan.rs` synthesis pass
- Diamond dedup + ProvenanceEntry: per ADR-018
- canonical_path name@version format: per ADR-017
- Discipline references: ADR-001, ADR-005, ADR-007, ADR-008, ADR-016,
  ADR-017, ADR-018

---

### Component 6: Cross-crate / ecosystem immunity

**What it does**

Antigen declarations propagate across crate boundaries. Cross-crate scan
via `.cargo/registry` source-walking reads antigens from dependencies.
`antigen-stdlib` (post-A5) provides shared failure-class memory across the
Rust ecosystem. Canonical-path identity (name@version per ADR-017)
distinguishes same-named antigens across crates. Trust delegation to
cargo's checksum chain (ADR-017) handles the supply-chain trust boundary.
*Population-level immunity.*

**Mechanism:**

- `cargo antigen scan` walks workspace + dependencies via cargo metadata
  resolution.
- `enumerate_dep_crate_roots()` (per ADR-017) is the ONLY trust-delegated
  path-discovery mechanism; alternative path-discovery bypasses trust.
- canonical_path on declarations carries `crate@version::Type` for
  cross-crate identity disambiguation.
- ProvenanceEntry (ADR-018 Option C) on `inherited_from` field preserves
  cross-crate provenance through propagation.
- Future antigen-stdlib (post-A5) ships standardized failure-class memory
  for ecosystem-wide patterns.

**The discipline side**

Team uses dependencies' antigen declarations rather than re-declaring
locally. Team contributes to antigen-stdlib when their per-project
antigens accumulate to ecosystem-relevant patterns. Team treats
cross-crate trust boundaries with proper validation discipline.

**The tooling side**

cargo metadata walking, registry path resolution, canonical_path stamping,
cross-crate cycle detection, cross-crate diamond dedup, cross-crate
witness resolution (re-exports = ATK-A3-001 territory; A4+).

**Floor / ceiling**

- **Floor**: `cargo antigen scan --include-deps` reads any cross-crate
  antigens that happen to be in dependencies.
- **Ceiling**: antigen-stdlib widely adopted; ecosystem-shared failure-
  class memory; cross-ecosystem trust boundaries (e.g., scoped antigens
  for cryptographic safety classes, async-soundness classes,
  drop-safety classes); per-organization antigen registries; antigen
  declarations in CVE databases.

**Biology cognate**

Multiple cognates:

- **Herd immunity** — population-level protection where enough individuals
  are immune that the pathogen can't propagate. antigen-stdlib's
  ecosystem-wide adoption produces analogous protection: enough crates
  have failure-class memory that ecosystem-level failures get caught
  before they propagate widely.
- **MHC polymorphism** — different individuals in a population present
  different MHC alleles, producing population-level resilience to novel
  pathogens (some individuals will recognize what others can't). Cognate
  to ecosystem-level antigen diversity: different crates may carry
  different per-project antigens; the union catches more than any
  individual.
- **Microbiome / commensal organisms** — the body's tolerance of beneficial
  organisms while maintaining hostility to pathogens. Cognate to
  `antigen_tolerance` at the ecosystem level: known-acceptable patterns
  shared across the community.

[NATURALIST: deepen. Population-level immunology has rich vocabulary —
herd immunity thresholds, ring vaccination, sterilizing vs non-sterilizing
immunity. Are any of these architecturally relevant to antigen-the-project's
cross-crate component? What's the cognate for an *exclusion* — a
known-bad pattern that the ecosystem agrees to refuse?]

**Value-prop**

Failure-class memory becomes *shared infrastructure*. New crates inherit
the ecosystem's accumulated failure-class memory without re-discovering
each pattern. Per-organization antigens scale up to industry-wide pattern
sharing. Cross-team / cross-organization coordination becomes possible
through a shared vocabulary.

**Failure modes / attack surface**

- **Trust-boundary bypass** — alternative path-discovery mechanism
  (e.g., directly passing fake registry path to scan_workspace) bypasses
  enumeration trust delegation (ATK-A3-007 covers this).
- **Cross-crate name collision** — same-named antigens in different
  crates without canonical_path disambiguation produce silent
  false-suppression (ATK-A3-005, solved by Approach 3-revised /
  ADR-017).
- **Re-export false NotFound** — antigens re-exported across crates may
  not resolve through bare-name lookup (ATK-A3-001, A4+ territory).
- **Registry tampering** — files planted in `.cargo/registry/src/` after
  cargo fetch; cargo's checksum chain doesn't cover post-fetch
  manipulation (ATK-A3-007 covers detection at enumeration layer).
- **Supply-chain antigen poisoning** — a malicious crate ships antigen
  declarations that defend against the wrong things, suppressing real
  vulnerabilities in dependents.

[ADVERSARIAL: deeper analysis. Supply-chain attack surface is the
biggest unstudied area. What's the threat model for a malicious crate
that ships *plausible* antigens with *wrong* witnesses? What about
version-pinning attacks (a malicious version of an otherwise-trusted
crate)?]

**Connection to other components**

- Component 1's declarations cross crate boundaries.
- Component 2's scan walks across dependencies; audit reports cross-crate
  status.
- Component 3's witnesses may be cross-crate (a dependency's test
  function); witness resolution handles canonical_path.
- Component 4's references may point to other crates' artifacts.
- Component 5's lineage may span crates; canonical_path identity is
  load-bearing here.

**Substrate locations**

- Cross-crate scan: `antigen/src/scan.rs::enumerate_dep_crate_roots`
- canonical_path: `antigen/src/scan.rs` types
- antigen-stdlib: deferred to post-A5
- Discipline references: ADR-001 Amendment 1 (C7 cross-crate commitment),
  ADR-002, ADR-005, ADR-017, ADR-018

---

## Part III: Composition patterns

### Cross-component flow

Most teams will deploy *some* components and not others. The composition
is *plural at its core* — not a single "antigen deployment" but a fabric
of immune-system components the team selects from.

Some likely composition patterns:

**Minimum viable (linter-mode)**: Component 2 alone. Team installs
`cargo-antigen`, runs scan, gets passive structural memory of what's
declared in their dependencies. Floor value, zero buy-in.

**Pragmatic dev (linter + manual antigens)**: Components 1 + 2. Team
writes some `#[antigen]` declarations for their own failure-classes;
scan operationalizes them. Most common adoption shape.

**Pragmatic dev + tested immunity**: Components 1 + 2 + 3. Team adds
witnesses to their `#[immune]` claims; audit reports verification tier;
team treats audit-tier-honesty as discipline.

**Bridged-knowledge organization**: Components 1 + 2 + 3 + 4. Team
attaches references to lived context; antigens become knowledge graph
nodes; cross-team coordination through shared vocabulary.

**Lineage-aware long-lived codebase**: Components 1 + 2 + 3 + 4 + 5.
Team manages failure-class taxonomy via descended_from; version-boundary
transitions handled with proper re-validation; temporal recognition
surface tracks immunity history.

**Ecosystem participant**: All six components. Team contributes to
antigen-stdlib, uses cross-crate scan, propagates per-project antigens
to ecosystem-relevant patterns.

**Floor for each component** is independent of all other components. A
team can deploy Component 5 (lineage) without Component 4 (knowledge-
ecosystem). A team can deploy Component 4 (knowledge-ecosystem) without
Component 3 (test-integration). The composition is genuinely orthogonal,
not hierarchical.

### Cross-component dependencies (real, but minimal)

Components are *almost* independent. A few real dependencies:

- **Component 5 + Component 6** are tightly coupled through
  canonical_path identity. Cross-crate lineage edges require both
  components active.
- **Component 3's witness re-validation across descent** uses Component
  5's lineage. A team using Component 3 without Component 5 would have
  flat immunity claims with no propagation.
- **Component 2's audit-tier-honesty** depends on Component 3's witness
  types being declared. Without witnesses, audit has nothing to evaluate.

These dependencies are real but small. The architecture is *primarily*
compositional, with a few cases where one component's full value lands
only with another component active.

### Extend-not-replace at the component level

Each component *extends* a baseline practice without *replacing* it.

- Component 1 extends developer judgment without replacing it (the macros
  give judgment a vocabulary, not a substitute).
- Component 2 extends manual code review without replacing it (the
  scanner catches what review can't, doesn't substitute for review).
- Component 3 extends testing without replacing it (witnesses link to
  tests; they don't replace what tests do).
- Component 4 extends knowledge management without replacing it (the
  references point to existing artifacts; they don't replace the
  artifacts).
- Component 5 extends version-management without replacing it (lineage
  tracks failure-class evolution alongside the existing semver / changelog
  practice).
- Component 6 extends ecosystem coordination without replacing it
  (antigen-stdlib augments existing crates.io / docs.rs / RFCs
  ecosystem).

The architectural property *is* the extension at each level. Antigen
doesn't compete with any practice; it extends each practice with a
structural-memory layer specific to that practice's failure modes.

---

## Part IV: The architectural properties

### Heterogeneous recursion

The compositional property recurses through all scales. The mechanisms
at each scale do not.

- At the project-vs-ecosystem scale, antigen extends Rust without
  replacing it; the project-tier mechanism is different from the
  ecosystem-tier mechanism, but the compositional property is the same.
- At the antigen-vs-practice scale, antigen extends testing / docs /
  ADRs / etc. without replacing them; the mechanisms differ wildly, but
  the compositional property is the same.
- At the tool-vs-discipline scale (within antigen), tooling extends
  discipline without replacing it; mechanisms differ, property is the
  same.
- At the component-vs-component scale (within antigen tooling and
  discipline), components extend each other without replacing each other;
  mechanisms differ, property is the same.
- At the framing-vs-framing scale (in communication), the three
  manuscript framings extend each other without replacing each other;
  mechanisms differ (different audiences, different abstraction levels),
  property is the same.

The property: *extend-not-replace through composition under a shared
vocabulary protocol.*

The mechanisms: *wildly different at each scale — different cells,
different signals, different time-courses, different anatomical
locations (in biology) / different file types, different team practices,
different tools, different abstraction levels (in software).*

This is genuinely how biological immune systems compose. Helper T cells,
NK cells, antibodies, MHC, complement, macrophages — different mechanisms,
shared protocol (MHC presentation + cytokine signaling), unified
architectural class (recognition-with-memory-and-inheritance).

### Structural-tier vs. maintenance-tier

Tests, documentation, ADRs, sprint planning, knowledge wikis, Slack —
these are all **maintenance-tier** practices. Their currency depends on
ongoing team effort. As soon as code evolves past them, they're stale
until the team updates them.

Antigen operates at **structural-tier**. Its currency is enforced by the
same machinery that enforces type-checking. When fingerprints fail to
match, the antigen surface *notices* — not because someone updated a doc,
but because the structural memory and the structural reality diverged
and the compiler/scanner sees it.

This is the architectural property Tekgy named:

> "It's the structural failure-class memory stuff but also it's a WAY to
> keep low context, high value, always accurate lifecycle/evolution/
> intent/design/architecture/connection/tracing/etc."

The *always accurate* property comes from antigen being structurally
present in the code, with currency enforced by the compilation/scan
machinery. The other components extend this property in different
directions:

- Component 1's declarations stay current because they're in the code
  alongside what they describe.
- Component 2's scan surfaces drift immediately when the structural
  memory and reality diverge.
- Component 3's witness-tier reporting catches when verification claims
  exceed actual verification work.
- Component 4's references *can* go stale (this is where maintenance-tier
  re-enters), but the antigen they're attached to stays structurally
  current.
- Component 5's lineage cycles get caught structurally; orphan-lineage
  edges get surfaced structurally.
- Component 6's canonical_path identity catches cross-crate
  collisions structurally.

The components vary in how much they extend the structural-tier property
into adjacent practices. Component 4 (references) is the most
maintenance-tier-prone; Components 1, 2, 5, 6 are most structural-tier.

### Co-native with human and LLM collaborators

The vocabulary is designed to be readable by both kinds of collaborators
without translation. No specialized syntax that requires expert
interpretation; no jargon-encrusted convention; no implicit knowledge
required to parse declarations.

This is structural, not stylistic:

- The biology metaphor is universal lived experience for humans (everyone
  has an immune system; everyone has had infections and recoveries).
- The biology metaphor is unambiguous structural cognate for LLMs (clean
  semantic structure; no project-specific jargon; consistent across the
  vocabulary).
- The macro syntax follows existing Rust attribute conventions; humans
  read it like other Rust attributes; LLMs read it like other Rust
  attributes.
- The audit report is structured data (`ScanReport`); humans read the
  human-readable version; LLMs consume the JSON. Both surfaces convey the
  same information.

The co-native property is what makes encounters-the-discipline work in a
mixed-collaboration team. Future versions of any team-member (human or
LLM) can inherit failure-class memory by reading what's already in the
code. The vocabulary is the bridge.

---

## Part V: Open enumeration — what we haven't named

The 6 components are provisional. The team should expect to find more as
the project matures. Some candidates worth watching:

### Candidate component: Real-time / CI feedback immunity

The current passive scan (Component 2) operates at build-time. A potential
component is *real-time* recognition during PR review: surfacing antigen-
surface diffs per PR; flagging when a PR touches a vulnerable site marked
`#[presents]`; surfacing fingerprint matches against unmarked changes.

This is *operationally distinct* from passive scan because the audience
is different (the PR reviewer in-the-moment), the latency requirement is
different (sub-second vs build-time), and the integration surface is
different (GitHub/GitLab/etc. webhooks vs cargo subcommand).

If this is a 7th component, it would map to *neutrophil response* in
biology — the rapid, sub-acute response to acute injury / pattern
detection.

[SCOUT: investigate. Is this its own component or is it Component 2
operating at a different latency tier?]

### Candidate component: Cross-team / organizational immunity

Beyond cross-crate (Component 6), there's an *organizational* tier where
teams within an organization share antigens not via crates.io but via
internal registries, internal ADR conventions, internal knowledge bases.
This is *between* Component 6 and the per-team Component 1 — neither
fully ecosystem nor fully per-project.

[SCOUT: investigate. Are teams adopting antigen across organizational
boundaries doing something distinct from the cross-crate cargo mechanism?]

### Candidate component: Adversarial / red-team immunity

The adversarial discipline that finds antigen attack surfaces (currently
tracked through ATK-* contracts) is itself a form of immunity production
— but at a different level than the dev-judgment Component 1. The
adversarial component *attacks* the antigen surface to surface weakness,
which is closer to *T-cell memory testing* in biology (the body's
deliberate maintenance of immune readiness through occasional re-exposure).

[ADVERSARIAL: investigate. Is your role's discipline (Phase 8 forced-
rejection, ATK contract authoring) its own immune-system component, or
is it discipline that operates *within* Component 1 (dev-judgment)?]

### Candidate component: Educational / onboarding immunity

The discipline by which new team members (human or LLM) learn the
team's failure-class taxonomy through reading antigen declarations is
itself a form of immunity propagation. Cognate to *adaptive immune
education* in the thymus — T-cell maturation, negative selection,
acquisition of repertoire. The educational component is how new
collaborators *acquire* the team's accumulated antigen surface.

[NATURALIST: investigate. Is onboarding a distinct component or is it
the natural propagation property of Components 1 + 4 working together?]

### Other candidates worth watching

- **Decay / sunset immunity**: antigens that should be retired because
  their underlying failure-class no longer applies (the language evolved,
  the dependency changed, the platform deprecated the API).
- **Cross-language immunity**: antigen-the-vocabulary extending beyond
  Rust to other languages with similar memory-safety / type-system
  primitives.
- **Cross-organism immunity**: antigen-the-vocabulary extending beyond
  software to other engineering disciplines (hardware design, control
  systems, financial systems, biology research workflows).

These are speculative. They surface here as substrate for the team to
notice if/when they recur in real instances.

---

## Part VI: What this changes

### Adoption framing

The "adoption flywheel" reframes around component selection rather than
engagement-intensity. Not "how engaged is this team with antigen?" but
"which immune-system components has this team composed?"

Marketing / vision-pitch implications:

- Lead with the vocabulary (the spine) — not with the tool.
- Show the floor concept ("antigen catches failure-class memory") first.
- Reveal the components progressively for audiences that want to compose.
- Don't insist on full-fabric adoption; the floor-mode is genuinely
  valuable on its own.

### Manuscript framing

Three coexisting framings at three abstraction levels (per Tekgy's
"both can work" ratification):

- **Tool paper / v0.1.0 release**: "antigen catches failure-class memory"
- **Foundational paper / v0.2.0+**: "antigen composes multiple kinds of
  structural immunity in your codebase"
- **Paradigm-shift paper / post-A6**: "antigen is a vocabulary with a
  fabric of immune-system components; structural failure-class memory is
  a new tier of software-engineering practice"

The manuscript trajectory is *layered*, not sequenced. Each layer has its
audience; each layer's substrate accumulates at its own rate.

### scope.md / vision-pitch.md updates

Both documents currently describe antigen primarily through the
"failure-class memory" framing. They should be extended (not replaced)
with the multi-component framing. Suggested cuts:

- `scope.md` vision section: add a "components fabric" subsection after
  the four-window convergence framing.
- `scope.md` adoption section: reframe adoption-ergonomics in terms of
  component selection rather than engagement.
- `vision-pitch.md`: keep the failure-class memory framing as the first
  paragraph. Add a multi-component paragraph after it. Add a "where you
  start, and where you can grow" component-fabric paragraph.

These edits land *after* this deep-dive ratifies; not in the same commit
as this draft.

### Project trajectory implications

If the multi-component framing holds through team Phase 1-8, several
follow-on substrate updates make sense:

- `glossary.md`: add "component" as a vocabulary term, with each of the 6
  components defined.
- `README.md`: extend the project description with the components fabric
  framing.
- Future sweep planning: A4 / A5 / A6 sweeps can be planned around
  component-tier capabilities (e.g., A5 = antigen-stdlib = Component 6
  ecosystem-tier work; future sweep for CI/real-time = candidate
  Component 7).
- Encounters-tier substrate (when ratified): each component is a candidate
  *encounter site* — instances of the component's operation that future-
  encounters should recognize.

---

## Open questions for team Phase 1-8

Q1. **Enumeration coherence**: are these 6 components really *one
abstraction at different scales*, or are some of them at categorically
different abstraction levels than others? Specifically: is Component 4
(knowledge-ecosystem) at the same abstraction level as Component 2
(passive scan), or is Component 4 a *higher* tier that operates *on*
Components 1-3-5-6?

Q2. **Biology-cognate sharpness**: Component 4 (knowledge-ecosystem) has
the loosest cognate. Is there a sharper biology analog, or is this where
biology goes silent (boundary-silence-as-evidence per naturalist's
discipline)?

Q3. **Component dependencies**: the doc says "almost independent" but
lists three real dependencies (5+6 via canonical_path; 3 propagation via
5; 2 audit-tier-honesty via 3). Are there more? Should some of these be
classified differently — e.g., Component 5 as a meta-component that
operates within Components 1-3?

Q4. **Real-time / CI feedback**: is this a 7th component or is it
Component 2 at a different latency tier?

Q5. **Adversarial discipline**: is the team's adversarial role's work
(Phase 8 forced-rejection, ATK contract authoring) its own component
or is it discipline operating within Component 1?

Q6. **Educational / onboarding**: is this a component or is it a
property of the vocabulary itself (the co-native readability)?

Q7. **Trade-offs of the multi-component framing**: does this framing
make the project harder to explain to first-time audiences? Does the
"floor / ceiling" framing serve better for first-contact, with the
multi-component framing held for audiences who need more depth?

Q8. **Stability of the enumeration**: should the team commit to "six
components in v1, with a clear extension mechanism" or stay genuinely
open with the enumeration provisional? Recognition-not-design suggests
staying open; communication clarity suggests at least naming what's
stable vs. provisional.

Q9. **Cross-language extension**: does the multi-component framing
break when antigen's vocabulary extends beyond Rust? Are some
components Rust-specific (e.g., proc-macro-based Component 1)?

---

## What this document is NOT

To prevent scope creep:

- **Not a ratified framing**. This is a deep-dive draft for team
  Phase 1-8. The framing here may be substantially modified or refined.
- **Not a replacement for existing framings**. Per the conversation
  substrate, the failure-class-memory framing remains valid; this
  extends it for different audiences.
- **Not an implementation roadmap**. Components are an architectural
  framing, not a development plan. The team's current sweep plan
  (A3 → A4 → A5) continues per existing process.
- **Not authoritative on biology**. The biology cognates are my best
  read; naturalist's refinements supersede where they conflict.
- **Not exhaustive on attack surface**. The adversarial seams are
  flagged for adversarial-role expansion; what's here is my best
  read, not a comprehensive threat model.

---

## Acknowledgments

This deep-dive draft processes the substrate of the 2026-05-11
conversation between Tekgy (Christopher Averill) and team-lead (Claude
Opus 4.7), preserved verbatim at
`multi-component-immunity-conversation.md`. The framing emerged in
dialogue; this document is one step of processing it into project
substrate.

The team's parallel work during this same period — antigen-A3
implementation completing through commit 937fa0d, 235 tests passing —
provides the real-substrate grounding that this framing describes.
Multi-component immunity isn't speculative architecture; it's the shape
of what's actually been built and is being built.

The recursion continues. There is no fixed point. We may find more
components as we keep recursing. The enumeration is open.

*V0 authored 2026-05-11 by team-lead for antigen-A3 substrate. Open for
team Phase 1-8 + biology-cognate refinement + attack-surface analysis +
additional-component discovery. Subject to revision; not yet project-tier
substrate.*
