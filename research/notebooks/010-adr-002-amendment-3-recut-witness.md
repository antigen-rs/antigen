# Lab Notebook 010: ADR-002 Amendment 3 Re-cut — Observer Witness Record

> *Renumbered 005→010 on landing into the 0.6.1-self-non-self research canon (005 is taken by
> the ADR-066 antibodies notebook). Originally authored in the pathmaker-core worktree.*

**Date**: 2026-06-23
**Authors**: camp-voyage-observer (independent witness)
**Branch**: main (worktree: antigen-pathmaker-core-worktree)
**Status**: Complete
**Depends on**: notebooks 001-004 (antigen project baseline)

---

## Context & Motivation

A prior witness (aristotle) flagged that Amendment 3's first draft re-privileged a default
under a swagger framing while claiming "purely additive" — the classic overclaim. The re-cut
relocates the center of gravity from opinion to safety property and honestly names the
supersede. This witness record is DISTINCT from the aristotle's additive/redundant
deconstruction: I am checking consistency, gate-checkability, code-truth on two real surfaces,
and honest-scope. I am not rubber-stamping. I ran all four checks against actual texts and
code before forming a verdict.

Sources read (all verified at path, all non-empty):
- `R:/antigen/jbd/expeditions/v0.6.1-self-non-self/adr-002-amendment-3-sovereign-by-default-draft.md`
- `R:/antigen-061-self-non-self/docs/decisions.md` — ADR-002 (~L379), Amendment 2 (~L459),
  ADR-025 (~L6578)
- `R:/antigen-061-self-non-self/antigen-attestation/src/predicate.rs` — the supply-chain
  leaf implementations (`DepPinned`, `DepAttested`, `MaintainerUnchanged`,
  `ContentHashMatches`, `SandboxClean`)
- `R:/antigen-061-self-non-self/antigen/src/supply_chain/mod.rs` — module structure
- `R:/antigen-061-self-non-self/antigen/src/supply_chain/manifest.rs` — hand-rolled TOML
  scanner (ADR-002 Amd2 compete cite)
- `R:/antigen-061-self-non-self/cargo-antigen/src/main.rs` — `ureq` network call at
  `fetch_cratesio_cksum()` (L1504), `--live` flag, feature-gate evidence
- `R:/antigen-061-self-non-self/cargo-antigen/Cargo.toml` — dependency declarations

---

## What the Re-cut IS

The re-cut is a safety-property-grounded supersede of Amendment 2's "neither default
privileged" posture. It introduces a default — roll-our-own — justified not by preference
but by the non-cascade guarantee: antigen cannot be a cascade vector for the supply-chain
attacks it exists to detect. The key structural moves:

1. Clause 1: exempts the universal toolchain (clippy, cargo, rust-analyzer) because these are
   NOT in antigen's dependency graph — composing them adds zero cascade surface.
2. Clause 3: allows compose-by-exception for things too dangerous to hand-roll (TLS, crypto)
   or too large to out-safe (serde, syn) — gated + protected.
3. Clause 5: makes the exception decision empirical (scaffold both paths, measure, choose) —
   not a gut call or training prior.
4. Clause 6: the guarantee is "enforced, not promised" — antigen runs its own supply-chain
   family over its own dep tree as a blocking release gate.

The biology disclaimer explicitly separates Class-1 outcome (owning-as-prevention, skin-
barrier/granuloma) from software-engineering invention (empirical mechanic, cost-asymmetry
argument).

---

## Check 1: Consistency

### Does it cleanly handle being an honest supersede of Amendment 2's "neither privileged"?

YES, with one minor precision gap.

The re-cut's §What this SUPERSEDES section states the supersede plainly: "antigen now has a
default — roll-our-own — because it is a safety default (like memory-safety), and a safety
property is permitted to set one." It explicitly frames the compose-exception as the analog of
an `unsafe` block. This is honest and structurally clear.

Amendment 2's per-surface distinguishing-test machinery (the four-item substrate-citation
requirement, the compose-when/compete-when lists) is NOT discarded — the re-cut says it is
"preserved and re-pointed at the compose exception rather than the compete one." This holds:
the re-cut's clause 5 (the empirical-decision mechanic) is the re-grounded form of Amendment
2's four-item citation, now required for COMPOSE decisions (exceptions) rather than compete
decisions (the old default was compose). The directionality flipped; the discipline stayed.

One precision gap: Amendment 2's "compete-when" list (cohesion, vocabulary conflict,
in-lockstep evolution) is NOT explicitly re-grounded in the re-cut — it is implicitly subsumed
into clause 2 (own everything that is antigen's capability). The re-cut could name this
explicitly to make the inheritance traceable. It doesn't currently. This is a minor clarity
issue, not a logical hole.

### Does it conflict with ADR-025?

NO conflict — the relationship is STRENGTHENED. ADR-025 made a "compete" decision for the
supply-chain defense family using Amendment 2's four-item substrate-citation (adjacent tools:
cargo-vet/cargo-deny/cargo-audit; cohesion reason: unified vocabulary vs fragmented
translation; measurable adopter differential; alternative path preserved). The re-cut does not
touch that decision — ADR-025's compete verdict was about building antigen's own supply-chain
ANTIGENS, not about antigen's own dep graph. The re-cut is about the LATTER. No conflict.

### Does it correctly describe antigen's own supply-chain leaves?

This is the most important consistency check, and the answer is: PARTIALLY, with a material
accuracy issue.

The re-cut (clause 3) says the supply-chain family leaves — `ContentHashMatches`,
`MaintainerUnchanged`, `DepAttested`, `SandboxClean` — are run over antigen's own dep tree as
"active dynamic protection." The predicate.rs code tells a more precise story:

- `DepPinned`: checks Cargo.toml exact-pin specifiers. STATIC, not dynamic.
- `DepAttested`: checks `.attest/supply-chain/dep-attest/<crate>@<version>.json` sidecars.
  STATIC substrate-read.
- `MaintainerUnchanged`: checks `.attest/supply-chain/maintainer/<crate>.json` snapshots.
  The code explicitly notes "v0.2 cannot live-query crates.io; emits
  `crates-io-metadata-query-failed` for any version-mismatch case." STATIC in v0.2.
- `ContentHashMatches`: checks `Cargo.lock` checksum against a recorded sidecar. STATIC. The
  `--live` flag in cargo-antigen does make a network call, but it is opt-in, gracefully
  degrades on network failure, and is NOT the default evaluation path.
- `SandboxClean`: the code comment is explicit: "v0.4+ feature: v0.2 returns
  ToolingNotYetAvailable." NOT YET IMPLEMENTED.

The re-cut calling these "active dynamic protection" overstates v0.2 reality. In v0.2, the
protection is STATIC ATTESTATION CHECKING (sidecar presence, Cargo.lock checksum comparison)
with a deferred roadmap for live registry queries (v0.3+) and sandbox execution (v0.4+).
"Active" is defensible in the sense that these are active checks at audit-time; "dynamic" is
not accurate for v0.2. This is not a fatal flaw in the ADR's decision, but the descriptive
claim needs qualification. The phrase should be "attestation-based protection (static in v0.2;
live-query in v0.3+; sandbox in v0.4+)" or similar.

---

## Check 2: Is the New Gate Checkable?

The re-cut's checkable gate: "is it a thing the user invokes (compose, zero cascade) vs a
crate that ships inside our graph (own/protect)?"

VERDICT: YES, substantially more checkable than "cohesion-critical surface."

The prior gate ("cohesion-critical") was undefined — the prior witness correctly flagged it.
The new gate has a binary decision axis: is the dep in antigen's `Cargo.toml`/`Cargo.lock`?

- If yes: it is "inside the graph" → ownership default applies; compose requires exception +
  evidence (clause 3 + 5).
- If no (CLI toolchain that the user invokes): clause 1 exempts it.

The gate IS operationally checkable by `cargo tree` or direct manifest inspection. The
distinction is crisp: `ureq` appears in `cargo-antigen/Cargo.toml` → it is inside the graph
→ the ownership default applies; its presence requires the clause-3 justification. `clippy`
does NOT appear in any `Cargo.toml` → it is outside the graph → clause 1 applies, zero
cascade.

One edge the gate leaves fuzzy: CLI binary deps in `cargo-antigen` that ship as part of the
antigen toolchain but are only invoked by the user (e.g., `gix`, `clap`). These are in the
graph but are infrastructure rather than antigen's "capability." The re-cut's clause 2 ("own
everything that is antigen's capability") and clause 3 (compose for "too big to out-safe")
handle this via the empirical mechanic — but the checkable gate's binary axis (in-graph /
not-in-graph) is correct as stated. The refinement is in clause 3's exception logic, which
the gate correctly delegates to.

---

## Check 3: Code-True Assessment — Two Real Surfaces

### Surface A: `ureq` in `cargo-antigen` (live crates.io checksum fetch)

Code facts established from reading `cargo-antigen/Cargo.toml` and `src/main.rs`:

1. `ureq = "2"` appears in `[dependencies]` WITHOUT a feature gate. It is an unconditional
   dep — all `cargo-antigen` users get it in their compiled binary.
2. The network call (`fetch_cratesio_cksum` at main.rs:1504) is RUNTIME-gated behind `--live`
   (a CLI flag), not a Cargo `[features]` gate. The binary is always compiled with ureq; the
   network call only fires when `--live` is passed.
3. The re-cut (clause 3) says "feature-gate the breach so it only opens when the capability
   is requested (e.g. the live crates.io-checksum verify in `cargo-antigen` — antigen's *only*
   network call — gated, because TLS is the security-too-hard carve-out)."

ACCURACY ISSUE: The re-cut says the breach is "feature-gated." The code shows it is
RUNTIME-FLAG-GATED, not Cargo-feature-gated. There is a meaningful difference:
- A Cargo `[features]` gate means users who don't request the feature get a binary with zero
  ureq in the compiled output.
- A `--live` runtime flag means ureq is compiled in for EVERYONE; only the HTTP call is
  conditioned on the flag.

This is not nothing. A supply-chain attack on ureq would affect all cargo-antigen users even
if they never pass `--live`. The re-cut's "feature-gated" claim is therefore inaccurate
against the current code. The correct description is "runtime-flag-gated."

Whether this rises to a DECISION-LEVEL problem or an IMPLEMENTATION MISMATCH is the question.
The re-cut is a DRAFT ADR, not ratified code. If the intent is a true Cargo feature gate
(`cargo-antigen/Cargo.toml` `[features]` with `ureq` behind `live-verify = ["dep:ureq"]`),
then the code must move to match the ADR. If the intent is that runtime-gating suffices, the
ADR's language must be corrected from "feature-gate" to "runtime-gate." Either way, the
current state has an ADR-vs-code mismatch that must be resolved before ratification.

ADR-025's own carve-out for ureq: The re-cut correctly identifies ureq as the TLS/security-
too-hard carve-out (clause 3b: "rolling-it-ourselves is the more dangerous move"). This
reasoning is sound — hand-rolling TLS would be genuinely less safe. The clause-3 exception is
well-founded; only the "feature-gate" label is imprecise.

### Surface B: `serde` / `syn` (compose-with-protection vs reinvent)

Code facts: `serde` and `syn` appear in `cargo-antigen/Cargo.toml` as workspace deps. They
are in-graph.

The re-cut (clause 3a) says compose for "the thing is too big to out-safe by hand (serde,
syn, proc-macro2)." This correctly describes the decision for these crates:
- `serde`: The world's most widely audited Rust serialization framework. Hand-rolling a safe
  replacement is not achievable — the attack surface of a hand-rolled version would be larger
  than the attack surface of serde's known vulnerabilities. Clause 3a applies correctly.
- `syn`: Proc-macro parsing infrastructure. Same reasoning. Too large, too central, too
  audited to out-safe.

The re-cut says these should be used with "active non-cascade protection (antigen's own
ADR-025 family)." Per the code-truth check above, in v0.2 this means attestation-based static
checks (not live-query or sandbox). For `serde` and `syn` specifically: antigen's own
`Cargo.toml` pins them with `= X.Y.Z` exact specifiers (consistent with the dep-pinned leaf),
and ADR-025 defines the `ContentHashMatches` leaf as the load-bearing defense for
content-replacement attacks. Whether antigen has ACTUALLY run these leaves over its own dep
tree (per clause 6's "blocking release gate" claim) is a v0.6.1 pre-tag-audit question that
this witness cannot verify without the audit artifacts — but the framework for doing so is
code-true and sound.

---

## Check 4: Honest-Scope

### Biology disclaimer

The re-cut's biology grounding section states: "Owning-as-prevention and the
skin-barrier/granuloma framing are Class-1 (biology-predicted at outcome level: a maturing
immune-system-as-tool will own the defenses it must guarantee and wall off what it cannot).
The empirical-decision mechanic (clause 5) and the cost-asymmetry argument are
software-engineering invention (honest silence; biology does not ground them)."

VERDICT: HOLDS. This is the correct ADR-003 dual-axis posture. The biology grounds the
structural necessity of the default (you own the defenses you must guarantee — this IS
predicted by immune biology), but is silent on the empirical scaffolding mechanic or the
cost-asymmetry calculation. The re-cut names the split explicitly and does not overclaim.

### Clause 6 — "enforced, not promised"

Clause 6: "The guarantee is enforced, not promised: antigen runs its own supply-chain family
over its own dependency tree as a blocking release gate (the v0.6.1 pre-tag-audit is its first
instance)."

The framework for enforcement is code-true — the supply-chain leaves exist, the evaluate.rs
machinery exists, the CLI subfamily exists. The question is whether the blocking gate is
ACTUALLY set up as a CI/pre-tag check for the antigen repository itself. This witness cannot
verify the CI configuration from the sources read (would require reading `.github/workflows/`).
The ADR is a DRAFT being staged for ratification; clause 6's claim should be verified against
the CI config before ratification, not after. This is a pre-ratification condition, not a
reason to block the draft now.

---

## Summary Findings

| Check | Verdict | Evidence |
|-------|---------|----------|
| Consistency with Amendment 2 supersede | CLEAN with minor gap | Per-surface machinery inherited but compete-when list implicitly subsumed |
| Consistency with ADR-025 | NO CONFLICT | Orthogonal scopes confirmed |
| "Active dynamic protection" description of v0.2 leaves | OVERSTATED | predicate.rs + supply_chain/mod.rs: v0.2 is static attestation; live = v0.3+; sandbox = v0.4+ |
| New gate checkable? | YES — substantially improved | Binary in-graph/not-in-graph axis is operationally verifiable |
| ureq code-truth | ADR-vs-CODE MISMATCH on "feature-gate" | Cargo.toml: unconditional dep; runtime `--live` flag, not Cargo feature gate |
| serde/syn code-truth | CORRECT | Clause 3a carve-out is sound; protection framework is code-true |
| Biology disclaimer | HOLDS | Explicit Class-1 / software-invention split; no overclaim |
| Clause 6 enforced gate | PLAUSIBLE but unverified | CI config not read; pre-ratification condition |

---

## Verdict

**WITNESSED-WITH-CAVEATS**

The re-cut is a genuine improvement over the first draft. The core decision (sovereignty-by-
default as a safety property, compose-as-unsafe-block exception, empirical mechanic for
exceptions) is sound and correctly grounded. The gate is checkable. The biology is honest. The
supersede of Amendment 2 is clean.

Two caveats that must be resolved before ratification:

**Caveat 1 (material — requires ADR text correction OR code change):** The re-cut says
`ureq` is "feature-gated." It is not. It is an unconditional `Cargo.toml` dependency; the
network call is runtime-flag-gated via `--live`. This either (a) requires correcting the ADR
to say "runtime-gated" instead of "feature-gated," or (b) requires adding an actual Cargo
`[features]` gate in `cargo-antigen/Cargo.toml` (`ureq` behind `live-verify = ["dep:ureq"]`)
and updating the code accordingly. The decision which path to take is the navigator's call,
but the mismatch must be resolved.

**Caveat 2 (minor — descriptive accuracy):** The re-cut calls the supply-chain family leaves
"active dynamic protection." In v0.2 they are static attestation checks. The language should
be qualified with the version-phase reality (static v0.2, live-query v0.3+, sandbox v0.4+)
to preserve the project's honest-scope discipline.

Neither caveat touches the core safety-property framing, which this witness regards as well-
reasoned and a genuine advance over the prior draft. The distinction between "inside the dep
graph" vs "invoked from user's toolchain" IS the checkable gate the prior draft lacked.

**Signature**: camp-voyage-observer, independent witness. Read all source texts and code
before forming this verdict. Did not author the re-cut. Did not read the aristotle's
deconstruction before running my checks.

**Files attested**:
- `R:/antigen/jbd/expeditions/v0.6.1-self-non-self/adr-002-amendment-3-sovereign-by-default-draft.md`
- `R:/antigen-061-self-non-self/docs/decisions.md` (ADR-002, Amd2, ADR-025)
- `R:/antigen-061-self-non-self/antigen-attestation/src/predicate.rs`
- `R:/antigen-061-self-non-self/antigen/src/supply_chain/mod.rs`
- `R:/antigen-061-self-non-self/antigen/src/supply_chain/manifest.rs`
- `R:/antigen-061-self-non-self/cargo-antigen/src/main.rs`
- `R:/antigen-061-self-non-self/cargo-antigen/Cargo.toml`
