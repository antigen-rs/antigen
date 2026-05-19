# Expedition INDEX

> Navigation map for `docs/expedition/`. Names what each layer is for, the
> discipline that distinguishes the layers, and what's currently in each.

`docs/expedition/` is the project's substrate-staging area — design substrate
that isn't yet ratified into ADRs but is too load-bearing to live only in
conversation. Existing flat files (`design-intent.md`, `failure-class-instances.md`,
etc.) stay where they are; they're cross-linked from CLAUDE.md and `decisions.md`
and the link churn isn't worth migration.

The four-layer convention below is **forward-only** — applied to new work, not
imposed retroactively.

---

## Layer convention

### `captures/` — raw conversation captures
High-fidelity, low-synthesis. The actual back-and-forth from a design
conversation, lightly cleaned for formatting but not rewritten for clarity.

**Discipline**: append-only. Once written, edit only for typos/formatting.
The point of a capture is fidelity-to-origin; rewriting it to look smarter
than the conversation actually was defeats the purpose. If clarity is
needed, that's what summaries are for.

**Naming**: `<topic>-<YYYY-MM-DD>.md` (e.g.,
`captures/discipline-witnesses-2026-05-18.md`).

**Use when**: a design conversation produced substrate dense enough that
losing the back-and-forth would be expensive. Especially when the
conversation includes dead ends — the dead ends prevent next-session
re-derivation of the same wrong shapes.

### `drafts/` — proto-designs, pre-deconstruction
Synthesized but explicitly pre-ratification. Proto-ADRs, design sketches,
shape proposals. The team will deconstruct these; expect them to change.

**Discipline**: version-bumped at significant pivot points (cross-session,
major cognitive shift, team-launch readiness); updated in place for
in-session refinement absorption (agent-returns, small extensions).
v1 and v2 were cross-session evolution / significant pivots; v3 onward
is rolling-current and absorbs in-session work in place. Captures
preserve the full substrate trail regardless — version-bumping is about
which draft is "current canonical shape," not about preserving history.

**Naming**: `<topic>-v<N>.md` (e.g., `drafts/discipline-witnesses-v1.md`).

**Use when**: the shape from a capture is dense enough to be engagable
but not yet team-deconstructed. The draft makes the proposal explicit
enough that aristotle / naturalist / adversarial have something concrete
to chew on.

### `summaries/` — distilled output
Lossy compression of longer substrate. Re-written in place as the shape
settles.

**Discipline**: mutable. Updated to reflect current understanding of the
captured/drafted substrate. Useful weeks-months later when details have
faded and you want to remember the shape without re-reading the full
capture.

**Naming**: `<topic>.md` (single canonical, no version stamp).

**Use when**: a capture or draft has grown long enough that engaging with
it has high cost. The summary is the on-ramp.

### `plans/` — work-stream sequencing
Time- and action-oriented. "Here's how we'd sequence this if the team
agrees the design is right." Distinct from drafts (which are about
*shape*) and decisions (which are about *commitment*).

**Discipline**: mutable until commitment. Once a plan crosses into
ratified commitment, it gets promoted to an ADR or a permanent plan doc
in `docs/`, not kept in expedition/.

**Naming**: `<topic>-<purpose>.md` (e.g.,
`plans/discipline-witnesses-rollout.md`).

**Use when**: the shape is settling and the question shifts from "is this
right?" to "how do we ship this?". Plans can be written before
deconstruction (as a hypothesis about sequencing) or after.

---

## Current entries

### captures/
- [discipline-witnesses-2026-05-18.md](captures/discipline-witnesses-2026-05-18.md) — tambear-smoke-test → substrate-witness predicate family → typed sidecars → code-locality → per-antigen-per-file `.attest/` shape
- [discipline-witnesses-adversarial-self-attack-2026-05-18.md](captures/discipline-witnesses-adversarial-self-attack-2026-05-18.md) — single-instance adversarial self-attack on v2; 8 attacks + responses + 9 refinements (R-A1 through R-A9) for v3 folding
- [discipline-witnesses-naturalist-self-pass-2026-05-18.md](captures/discipline-witnesses-naturalist-self-pass-2026-05-18.md) — single-instance naturalist self-pass; biology rhymes validated; 7 refinements (R-N1 through R-N7); R-N1 reframed B-cell-vs-T-cell tier-cap argument to evidence-kind ceiling; framing-B (expanded unit-of-analysis) gained substrate support
- [discipline-witnesses-aristotle-self-pass-2026-05-18.md](captures/discipline-witnesses-aristotle-self-pass-2026-05-18.md) — single-instance approximation of Phase 1-8 on 4 load-bearing principles; 4 refinements (R-Ar1 through R-Ar4); R-Ar2 surfaced unification insight (substrate-witness over JSON sidecar AND cross-crate-witness over dep source as "substrate over substrate-other-than-this-code")
- [discipline-witnesses-aristotle-team-pass-2026-05-18.md](captures/discipline-witnesses-aristotle-team-pass-2026-05-18.md) — team-aristotle real Phase 1-8 on v2 + self-pass refinements; 8 F-findings + 1 frontier flag; F7 NEW (witness-provider-crate trust boundary unspecified, needs sub-clause F at leaf level); F8 NEW (EvidenceKind as first-class axis parallel to WitnessTier × AuditHint); ratify/replace mapping to self-pass; 8 priority-ordered v3 changes
- [discipline-witnesses-academic-research-2026-05-18.md](captures/discipline-witnesses-academic-research-2026-05-18.md) — 14-system landscape map (in-toto, SLSA, Sigstore, TUF, DSSE, PASETO, cargo-deny, cargo-vet, OPA/Rego, Salsa, CODEOWNERS, Renovate, npm-audit/GHA, GUAC); cargo-vet is closest analog; 3 genuinely novel to antigen (code-site-locality at item granularity, tier-honesty as first-class verification-output property, substrate-witness-as-extension-not-new-category); 9-item absorb list, 6-item don't-absorb list
- [discipline-witnesses-scout-pass-2026-05-19.md](captures/discipline-witnesses-scout-pass-2026-05-19.md) — F2 absorption-pattern generalizes (tolerance-ratification as v0.1-rc addition; lineage-validation + fingerprint-ratification long-arc); F3 4 new implicit-uniform dimensions (evidence_provenance, severity-class, lifetime, presentation-density); F8 EvidenceKind reach beyond witnesses; cross-domain (notary institutions 800-yr arc; signaling theory; annotation-fatigue ergonomics; distributed consensus); structural rhyme — discipline-witnesses are Component 1.5 (attestation-mediated-judgment) in multi-component-immunity taxonomy; scan/audit asymmetry flag
- [discipline-witnesses-adversarial-team-pass-2026-05-19.md](captures/discipline-witnesses-adversarial-team-pass-2026-05-19.md) — 10 attacks on v2 + aristotle frontier; 6 land + refinements T1-R through T9-R (delta-chain anti-laundering; leaf-provider enforcement mechanism required for v0.2+ ADR; closed-set tool 4-point bright-line; hint name overclaim fixed; unification drift guardrail via code-comment + adversarial test; reviewer-not-committer workflow documented); 6 frontier-attacks (FA-1...FA-6) named for next pass; 4 attacks absorbed without design damage

### drafts/
- [discipline-witnesses-v1.md](drafts/discipline-witnesses-v1.md) — substrate-witness predicate family, Ratification schema, code-side `.attest/` sidecars, `cargo antigen attest` CLI, opinionated-with-flexibility posture
- [discipline-witnesses-v2.md](drafts/discipline-witnesses-v2.md) — single-instance critical pass on v1: strengthened load-bearing arguments, sorted open questions (team-needs vs my-take-embedded), sharpened tier-honesty mapping for substrate-witnesses (Execution reachable when predicate passes + currency holds; FormalProof unreachable), positioned ONE ADR over three, reframed biology break-point with both clean-break and expanded-unit-of-analysis options for naturalist call
- [discipline-witnesses-v3.md](drafts/discipline-witnesses-v3.md) — **rolling current canonical draft** (updates in place per evolved convention). Folds all 7 interim captures. NEW v0.1-rc structural additions: **tolerance-ratification via isomorphic sidecar schema** (scout S1 — plugs ADR-011 tier-honesty gap); EvidenceKind axis (F8); scope field (F3); evidence_provenance field (scout S2); Signer.basis with anti-laundering safeguards (F5 + adversarial T2-R: chain-depth cap + cumulative-fingerprint tracking + required-rationale); signature_strength field; hint rename `discipline-predicate-passed-substrate-current` (T6-R). NEW CLI: `attest delta` (cargo-vet); `tolerate scaffold/sign/check/list` parallel family. Structural commitments: ratchet-asymmetry; bounded audit-of-audit recursion; witness-provider-crate trust boundary on critical path WITH ENFORCEMENT-MECHANISM scope (T1-R: WASM/no_std/subprocess, not docs-only); discipline-vs-machinery unification asymmetry enforced via code-comment + adversarial precision test (T5-R); closed-set tool 4-point bright-line rule (T4-R: excludes cargo build/run, curl external); reviewer-not-committer workflow documented (T9-R: signed_trailer v0.1; crypto-signing v0.4+; NO --on-behalf-of). v0.2+: TUF threshold-signatures, CODEOWNERS required_role, leaf-provider ADR with enforcement mechanism, lifetime on claims, --prioritized for annotation-fatigue. v0.4+: DSSE envelope + Sigstore identity-bound signatures (notary 800-yr arc). ADR-019 citation map. ONE-ADR position reinforced. T1-T8 open questions for team. Component 1.5 framing for multi-component-immunity. Scan/audit asymmetry flagged.

### summaries/
*(none yet)*

### plans/
*(none yet)*

---

## Existing flat files (kept where they are)

The pre-INDEX expedition/ contents:

- **Origin / intent**: `origin.md`, `design-intent.md`, `api-shape.md`,
  `revolutionary-and-not.md`, `vision-pitch.md`
- **Active substrate** (long-lived design dumps): `deferred-substrate.md`,
  `academic-context.md`, `failure-class-instances.md`,
  `ecosystem-composition.md`, `cross-domain-architectural-map.md`,
  `structural-memory.md`
- **Scope locks / sweep plans**: `a3-scope-lock.md`, `first-sweep-plan.md`,
  `pre-release-onboarding-sweep-proposal.md`, `encounters-proposal.md`
- **Handoffs / session records**: `HANDOFF.md`,
  `SESSION-HANDOFF-2026-05-09.md`
- **Logs**: `tambear-adoption-log.md`, `inheritance-from-tambear.md`
- **Team substrate**: `team-briefing.md`, `conventions.md`,
  `risk-register.md`
- **Domain-specific design substrate**: `multi-component-immunity.md`,
  `multi-component-immunity-conversation.md`,
  `antigen-applied-to-antigen.md`, `case-study-determinism-class.md`,
  `stdlib-seed-antigens.md`, `future-extensions.md`

These will gradually be cross-linked from this INDEX as they're touched,
but no migration is planned.

---

## Picking up from substrate

When a new session starts and needs to engage with prior design work:

1. **Check this INDEX** for the layered structure
2. **Read summaries first** if they exist (cheap on-ramp)
3. **Read drafts** for the current proposed shape
4. **Read captures only when needed** — for the original reasoning chain,
   the dead ends, the user-driven moves vs Claude-driven moves
5. **Don't trust pre-compaction memory** — the substrate on disk is
   authoritative; conversation summaries describe past state, not
   current state
