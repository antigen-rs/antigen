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
- [discipline-witnesses-naturalist-framing-call-2026-05-19.md](captures/discipline-witnesses-naturalist-framing-call-2026-05-19.md) — team-naturalist framing call on framing-A vs framing-B (Phase 1 pivot per navigator note 18:24:20). Call: framing-B correct; biology distinguishes recognition-role (memory-B-cell BCR) from evidence-role (plasma-cell secreted antibody / cellular activation history). Web-verified memory-B-cell vs plasma-cell biology. **Correction appended 2026-05-19** after observer NB003 caught conflation of biology-positioning with information-architecture: biology only mandates role-distinction, NOT file-layout (sidecar-only vs hybrid both satisfy). NEW load-bearing biology finding cross-checked against observer NB004: biology REQUIRES a WHY-of-attestation field (somatic hypermutation lineage analog) — observer's `reasoning: Option<String>` on `SignerBasis::Fresh` is biology-aligned. Architecture decision belongs to pathmaker/adversarial/aristotle, not naturalist
- [discipline-witnesses-adversarial-v3-pass-2026-05-19.md](captures/discipline-witnesses-adversarial-v3-pass-2026-05-19.md) — v3 post-folding adversarial pass; 6 attack surfaces; 13 named attacks; 9 land (HIGH: chain-cap no-floor T2R-C, descended_from predicate contract undefined FA5, fingerprint-scheme migration policy undefined FA2; MEDIUM: kind-mismatch audit states missing TOL-A/B, immunity-tolerance contradiction hint missing T4-A, rationale minimum dropped T2R-B, enforcement-mechanism scope descriptions T5-A/B/C, Windows subprocess isolation network gap T5-C; LOW: rotating signers documentation T2R-A, compile-time coexistence check TOL-C, migration burden TOL-D, compound_evidence kind-list T4-B); 4 new frontier attacks NFA-1 through NFA-4
- [discipline-witnesses-naturalist-f3-scope-biology-2026-05-19.md](captures/discipline-witnesses-naturalist-f3-scope-biology-2026-05-19.md) — team-naturalist on aristotle F3 scope question. Biology validates `site/file/module/crate/workspace` scope axis (cellular/sub-tissue/tissue-organ/organ-system/systemic) AND extends aristotle's analysis: scope is also a property of CLAIM KIND, not just substrate-location. Coarser scopes can encode coordination structures across finer scopes (secondary lymphoid organs — lymph nodes, spleen — as architectural substrate that shapes how cellular immunity operates). Three predictions: (P1) workspace-scope antigens can have quantified constraints over site-scope populations, not just aggregations; (P2) coarser-scope substrate may include architecture of finer-scope substrate; (P3) coarser-scope sidecars should be at existing coordination-points (CODEOWNERS, workspace Cargo.toml) not arbitrary higher-level files. v0.1 site+file scope only confirmed biology-aligned; v0.2+ workspace-scope needs claim-kind distinction encoded. Scan multi-scope question flagged for scout/pathmaker
- [discipline-witnesses-naturalist-notary-arc-verification-2026-05-19.md](captures/discipline-witnesses-naturalist-notary-arc-verification-2026-05-19.md) — team-naturalist verification of scout S4's notary-institution 800-year arc. Scout's claim survives (late 12th-13th C Bologna formalization; guild-regulated profession; papal/imperial licensing). NEW finding scout missed: medieval Italian city courts treated notarial documents as near-self-authenticating proof — exact rhyme for antigen's tier-honesty AUDIT-TIME SAVINGS at known confidence level. Sharpened ADR-019 prediction: civic notary (place-bounded) ↔ git-trust+CODEOWNERS (workspace-bounded audit-time-savings); notary public (papal/imperial license) ↔ OIDC+Rekor (cross-org audit-time-savings). The escalation path isn't "stronger signing" but "stronger AUDIENCE for whom audit-time-savings hold." One scout claim (specific false-attestation sanctions) not directly verified but structural argument holds without it
- [discipline-witnesses-naturalist-f8-evidence-kind-biology-2026-05-19.md](captures/discipline-witnesses-naturalist-f8-evidence-kind-biology-2026-05-19.md) — team-naturalist on aristotle F8 EvidenceKind biology validation. R-N1 mapped to two-category immunology (innate vs adaptive) with Behavioral "sits between." This pass identifies the missing third biological category: **trained immunity** (epigenetic memory in innate cells, established since ~2010s). Clean three-way mapping: TypeSystemProof ↔ germline-encoded innate; Behavioral ↔ trained-immunity / epigenetic memory; SubstrateState ↔ adaptive (B/T-cell substrate). Per-tier ceilings v3 names reflect biology mechanism (germline immutable → FormalProof; epigenetic needs triggering → Execution; adaptive needs currency → Execution). Predictions for v0.2+: (P1) Behavioral evidence should have training-window/freshness; (P2) failing tests should DOWN-train evidence (bidirectional); (P3) evidence-decay-rate as per-EvidenceKind property. v0.1 enum unchanged — biology validates the three values
- [discipline-witnesses-outsider-naive-pass-2026-05-19.md](captures/discipline-witnesses-outsider-naive-pass-2026-05-19.md) — first outsider pass on the discipline-witnesses thread; 19 findings (OUT-1...OUT-19) approaching v3 as a fresh user with zero prior context. Grouped: vocabulary/framing dust (OUT-1..OUT-11: "discipline" overloaded across three senses; substrate-witness vocabulary chain with three internal names; `.attest/` unexplained; three-axis output ceremonial in v0.1; biology metaphor dual-job; 4-point bright-line missing threat model; `evidence_provenance` self-reported; tolerance-immunity schema isomorphism as human footgun; ADR-019 8-citation prerequisite mass; missing new-user walkthrough; `signers(against="any")` undocumented default); substrate-vs-promise gaps (OUT-12..OUT-17: `antigen-attestation` crate orphaned; duplicate `WitnessTier` enums across crates; `EvidenceKind` built in attestation but not threaded into `audit.rs`; CLI subcommands not built; phase ordering broken at report-time; **HEADLINE OUT-17: v3 macro examples won't compile against current parser — `scope`/`discipline_doc`/`evidence_provenance` rejected by `#[antigen]`; `requires` rejected by `#[immune]` and `#[antigen_tolerance]`**); jurisdictional citation (OUT-18: ADR-005 Am 3 ratified TWO axes (`WitnessTier`+`audit_hint`); v3 cites it as authority for THREE — `EvidenceKind` needs its own authorization in ADR-019 or a fresh ADR-005 Am 4); team-composition meta (OUT-19: substrate built by 5 critique-from-inside roles in 12 passes with 0 captures from outsider/pathmaker/observer/executor before declaring "ready for full-team launch" — readiness signal biased toward critique-density). OUT-17 triggered tasks #22-25
- [discipline-witnesses-naturalist-self-recognition-biology-2026-05-19.md](captures/discipline-witnesses-naturalist-self-recognition-biology-2026-05-19.md) — team-naturalist focused note for navigator on scout S5 (`attestation-void-discipline-claim` as antigen-on-antigen instance). Tight biology rhyme: vaccine-attestation-without-titer-verification IS same failure mode (self-reported vaccination accepted without antibody titer) with same cure structure (require substrate verification). Strengthens scout S5 for antigen-applied-to-antigen.md instance 8. Biology goes SILENT at cellular layer (no cell catalogs what it hasn't recognized) — informative silence predicts `AttestationVoidDisciplineClaim` naturally lives at coarser scope (workspace/crate) per F3 scope-biology, not site-scope. Falsifiable prediction for Phase 6+ stdlib seed proposal
- [discipline-witnesses-naturalist-immune-arc-encounters-to-witnesses-2026-05-19.md](captures/discipline-witnesses-naturalist-immune-arc-encounters-to-witnesses-2026-05-19.md) — team-naturalist on scout's structural-rhyme finding (encounters-proposal innate-cognate + discipline-witnesses adaptive-cognate = two phases of one biological immune arc, currently unconnected in substrate). Substrate-verified scout's claim is correct (encounters-proposal.md L96-100 cites PRR/PAMP innate; v3 cites adaptive co-stimulation/vaccination/memory). Small biology correction to scout's step 2: innate cells don't become specific; the bridge is ANTIGEN PRESENTATION via dendritic cells, which maps to the three-instance threshold for declaring `#[antigen]`. Trained-immunity (per F8 capture) IS the intermediate stage scout's step 2 was approximating. SHARPENED framing-B: unit-of-analysis is the FULL ARC (innate first-encounter → trained-immunity intermediate → antigen-presentation bridge → clonal-selection specific recognition → adaptive memory cells + plasma cell secretion). Ready-to-fold biology-grounding text for ADR-019 names this end-to-end. Antigen-on-antigen instance for scout's S5 gets sharper form: the arc has completed in the substrate when the project self-recognized ADR-011's tolerance gap and developed the discipline-witnesses cure

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
