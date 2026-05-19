# Discipline-Witnesses — Team Launch Brief

> **Purpose**: focused team-launch brief for the discipline-witnesses
> development thread. Companion to [team-briefing.md](team-briefing.md)
> (general antigen project context). Read team-briefing.md first for
> project orient; this file scopes you into the active discipline-
> witnesses substrate.

> **Audience**: the JBD team spawning into discipline-witnesses
> development work. Substrate state as of 2026-05-19. This thread is at
> "ready for full-team development launch" — substrate is dense, T1-T8
> open questions are precise, FA-1...FA-6 adversarial frontier is named.

---

## What you're launching into

The substrate-witness predicate family — a structural extension of
antigen's witness vocabulary to cover any substrate the audit can read
(JSON sidecars, git log, doc frontmatter, completion markers), not just
Rust source. Plugs ADR-011 tolerance-vibes-grade gap as the v0.1-rc
sweetener. ONE-ADR position (ADR-019) with named amendments through v0.4+.

This work has been through:
- Original design conversation (13 turns, 2026-05-18)
- Single-instance refinement (3 self-passes: adversarial, naturalist,
  aristotle-approximation)
- Background-agent team-passes (aristotle Phase 1-8 real; academic-
  research 14-system landscape; scout F2-pattern-reach + cross-domain;
  adversarial 10-attack stress)
- Rolling consolidation into v3 (current canonical draft)

What's open now:
- T1-T8 substantive open questions
- FA-1...FA-6 adversarial frontier attacks
- Implementation work (the actual code; current state is substrate only)
- ADR-019 drafting against [`docs/process.md`](../process.md) lifecycle
- Coordination with tambear (first-user) adoption planning

---

## Read in this order

1. **[`drafts/discipline-witnesses-v3.md`](drafts/discipline-witnesses-v3.md)** —
   rolling current canonical shape. Read this first; everything else
   makes more sense after.
2. **[`INDEX.md`](INDEX.md)** — full substrate trail; pick captures
   relevant to your role from here.
3. **Your role's relevant captures**:
   - **aristotle**: [`captures/discipline-witnesses-aristotle-team-pass-2026-05-18.md`](captures/discipline-witnesses-aristotle-team-pass-2026-05-18.md) — prior Phase 1-8 with F1-F9; attack T3, T4, T5, T7, T8 fresh from v3
   - **adversarial**: [`captures/discipline-witnesses-adversarial-team-pass-2026-05-19.md`](captures/discipline-witnesses-adversarial-team-pass-2026-05-19.md) — 10-attack prior pass; FA-1...FA-6 frontier for you to attack; v3 has absorbed T1-R...T9-R refinements which means you're attacking the post-refinement frontier
   - **naturalist**: [`captures/discipline-witnesses-naturalist-self-pass-2026-05-18.md`](captures/discipline-witnesses-naturalist-self-pass-2026-05-18.md) — 7 refinements (R-N1...R-N7); framing-A vs framing-B is the open call; F3 scope biology + F8 evidence-kind biology are fresh territory
   - **scout**: [`captures/discipline-witnesses-scout-pass-2026-05-19.md`](captures/discipline-witnesses-scout-pass-2026-05-19.md) — prior scout pass; long-arc design-preserves (fingerprint-ratification, lineage-validation) are unscouted; scan-side asymmetry, lifetime-on-claims are open
   - **pathmaker**: read v3 + INDEX + sample 2-3 prior captures; the ADR-019 draft is yours to author when team is ready; the code implementing v3's shape is also yours
   - **observer**: peer-review what gets written; lab-notebook discipline for the team's own decisions
   - **executor**: dependency-graph the v0.2+ amendments named in v3's ADR-019 citation map; what's the critical path from v3 → v0.1-rc → v0.2 → v0.4+?
   - **outsider**: ask why for every assumption in v3; the substrate-witness reframe in particular has lots of "everyone knows" inheritance from prior captures that you should challenge fresh
4. **[`origin.md`](../origin.md)** — project WHY (read at some point if you haven't)
5. **[`decisions.md`](../decisions.md)** — grep for ADR-005 Amendment 3 (audit-tier-honesty — load-bearing for v3); ADR-002 (compose-don't-compete); ADR-006 (recognition-not-design); ADR-007 (anti-YAGNI); ADR-011 (tolerance — the gap v3 plugs)

---

## The journey ahead

Phases (executor will refine; this is the rough shape):

### Phase 1 — Team-attack v3 at frontier
- Each role attacks v3 from their angle (see role-specific captures above)
- T1-T8 open questions get team-resolutions
- FA-1...FA-6 frontier attacks land or are absorbed
- Naturalist makes the framing-A vs framing-B call
- Captures preserve the team's attacks; v3 updates in place with what survives

### Phase 2 — Draft ADR-019
- Pathmaker authors ADR-019 against [`process.md`](../process.md) lifecycle
- ADR-019 cites: ADR-002, ADR-004, ADR-005 + Amendment 3, ADR-006, ADR-007,
  ADR-008 (pending T6 substrate-grep), ADR-011 (RESOLVED — vibes-grade gap closed)
- Naturalist's framing call lands in ADR text
- Observer peer-reviews against lifecycle checklist
- Aristotle Phase 1-8 verdict before ratification
- Adversarial fixture set (ATK-019-*) added to test corpus

### Phase 3 — Implementation (v0.1-rc)
- New crate: `antigen-attestation` (separate per v2 R4)
- Schema (Ratification, ItemRatification, Signer, SignerBasis,
  RatificationKind, EvidenceKind, scope field)
- Predicate evaluator (closed combinator grammar; 5 leaf primitives)
- CLI: `cargo antigen attest scaffold/sign/delta/oracle/check/list/move/migrate/gc`
- CLI: `cargo antigen tolerate scaffold/sign/check/list`
- Audit integration: read .attest/ sidecars; emit three-axis output
  (WitnessTier × AuditHint × EvidenceKind + signature_strength)
- Anti-laundering safeguards in `attest delta` (chain-depth cap +
  cumulative-fingerprint tracking + required-rationale)
- Closed-set tool 4-point bright-line rule enforced at leaf-design review
- Unification drift guardrail: in-code comments + adversarial precision test
- Tests (atk_a3_*): unification guardrail, evidence-kind axis, tolerance
  audit hint, delta-chain caps, all v0.1 leaf primitives, hint
  emissions, scope field

### Phase 4 — Tambear adoption (concurrent with Phase 3 closing)
- Tambear declares first real discipline-antigen using the substrate-
  witness mechanism (sinh/cosh signed-zero discipline was the originating
  motivation; ship that)
- Adoption-log captures what tambear's developers actually hit
- Findings feed back into v0.2 amendment planning

### Phase 5 — v0.1-rc release
- Process: per [`docs/process.md`](../process.md) release lifecycle
- Crates: `antigen`, `cargo-antigen`, `antigen-attestation` (new),
  `antigen-fingerprint` (existing), `antigen-macros` (existing)
- Release notes name discipline-witnesses as new primitive +
  tolerance-ratification as ADR-011 gap closure

### Phase 6+ — v0.2 amendments
- CODEOWNERS interop (T2)
- TUF k-of-n threshold signatures
- Witness-provider-crate ADR with enforcement-mechanism specification
- Cargo-vet imports pattern for cross-crate
- Lifetime on discipline claims
- `--prioritized` flag

(v0.3+: SARIF; v0.4+: DSSE + Sigstore; v0.5+: T7 fingerprint-scheme
evolution; eventually: fingerprint-ratification sidecars +
lineage-validation sidecars)

---

## Coordination via campsite logbook

This team has a campsite (spawned alongside this brief). Campsite
logbook is the team's coordination substrate — narrate findings, route
between roles, mark when each phase advances. Navigator owns the logbook;
all roles contribute.

The substrate trail (captures/) is parallel to the campsite. Captures
are append-only high-fidelity records; campsite is the conversation /
coordination layer. Both matter.

---

## Standing constraints (carried from team-briefing.md + project-specific)

All [team-briefing.md standing constraints](team-briefing.md#standing-constraints)
apply. Specifically load-bearing for this thread:

- **Substrate over memory** — `git log`, read files, don't trust your
  context-summary of what's been done. v3 is the substrate; captures are
  the substrate; the convention shift (in-place vs version-bump) is
  documented in INDEX.
- **Co-design with tekgy** — architectural decisions are conversations,
  not orders. T1-T8 are decisions to make WITH tekgy when they're load-
  bearing for the user, not unilaterally.
- **Recognition over design** (ADR-006) — every new leaf primitive must
  recognize existing substrate; v3's leaves all do (markdown, git,
  JSON, file existence). Future leaves must too.
- **Compose, don't compete** (ADR-002) — substrate-witnesses are the
  extension-of-witness-vocabulary, not a replacement for code-witnesses.
  Don't reinvent what cargo-vet, in-toto, SLSA, Sigstore already
  provide — compose.
- **Tier-honesty** (ADR-005 Am 3) — the ratchet is the discipline;
  reports lower bound; promotions require evidence; downgrades automatic.
- **Biology metaphor is load-bearing, not decorative** — naturalist
  owns this; if a rhyme is imprecise, refine it (don't abandon).
- **Code-locality (germinal-center pattern)** — sidecars live with code,
  not with docs. This was a load-bearing reframe in turn 7 of original
  capture; don't drift.

---

## What you have agency to decide

- All T1-T8 open questions (with tekgy consultation when load-bearing)
- All FA-1...FA-6 frontier attacks (mitigations or absorb-into-design)
- Implementation details below the architectural ADR layer
- Test strategy for the v0.1 surface
- Phase ordering and parallelization
- When to involve tekgy and when to handle internally

## What you don't decide unilaterally

- ADR-019 ratification (per process.md lifecycle — multi-role consensus
  required)
- Release timing (tekgy owns)
- Crate name / repo name changes (tekgy owns)
- Tambear adoption commitments on tambear's side (cross-project; tekgy
  coordinates)
- Anything that breaks ADR-001's 8-class taxonomy or other foundational
  ADRs without going through the amendment process

---

## When stuck

- Read v3 again (it's where everything consolidates)
- Read the relevant capture (especially aristotle Phase 1-8 which
  goes deepest)
- Ask the campsite — navigator routes
- Ask tekgy directly when something feels load-bearing for them
- Use the [glossary](../glossary.md) when vocabulary feels ambiguous
- Use the [risk register](risk-register.md) when surfacing a new risk
- Use the [tambear adoption log](tambear-adoption-log.md) when something
  intersects with tambear-side reality

---

## The vibe

This is creative, collaborative, exploratory development work. The
substrate is rich; the journey is well-mapped enough to start but
flexible enough that surprises welcome. Adversarial pre-pass has
already attacked the obvious surfaces; aristotle has done Phase 1-8 on
load-bearing principles; naturalist has grounded the biology; scout
has wandered; academic-researcher has mapped the landscape. You're
launching into work that's been DE-RISKED in advance, not work that's
been pre-decided. Bring your own attacks, your own surprises, your own
direction.

Journey before destination.
