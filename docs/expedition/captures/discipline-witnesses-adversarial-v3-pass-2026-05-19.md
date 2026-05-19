# Capture — Adversarial V3 Pass on Post-Folding Discipline-Witnesses

> **Date**: 2026-05-19
> **Author**: team-adversarial (Sonnet 4.6, adversarial role)
> **Relation to prior captures**: attacks v3 (post-folding) specifically.
> The prior adversarial team-pass (`discipline-witnesses-adversarial-team-pass-2026-05-19.md`)
> attacked v2 + aristotle frontier and produced T1-R through T9-R refinements, all absorbed
> into v3. This pass attacks the v3 draft directly at the six surfaces named in the
> team-launch brief as the post-folding frontier: FA-2/T7 (fingerprint-scheme
> evolution), FA-5/T8 (descended_from predicate weakening), T4 (compound evidence +
> tolerance sidecar coexistence), T5 (leaf-contract enforcement mechanism bypasses),
> delta-chain anti-laundering re-attack (T2-R absorbed), and tolerance-ratification
> kind-discriminator abuse.
> **Status**: append-only capture

---

## Posture

Same posture as prior pass: no benefit-of-the-doubt. "By construction" is suspect.
"Obviously" is a red flag. v3 absorbed 9 refinements from the prior pass — this is
a continuation, attacking the post-refinement surface. Every attack below is a named
specific failure mode. Where the design holds, I say why. Where it doesn't, I name
what specifically.

I do not re-attack surfaces already settled by prior passes unless I have new
angles. The six surfaces below are genuinely post-v3 frontier.

---

## Attack T2R-A — Delta-chain anti-laundering: rotating signers bypass per-signer chain cap

**The v3 position (T2-R absorbed)**: chain-depth cap (default 3) per signer,
cumulative_root_fingerprint references the LAST Fresh basis for THIS signer.
Cap is enforced per `Signer.basis` entry.

**Attack**: the chain-depth cap is per-signer. A predicate requiring
`signers(required = ["alice"])` (or any single-signer requirement from a pool)
allows rotating through new signers when alice's chain-depth cap is hit.

Scenario: alice signs deltas 1, 2, 3 (chain_depth = 3 — at cap). Alice cannot
carry-forward without a Fresh re-attestation. The malicious substantive change
happens. Bob has never signed this sidecar. Bob adds a Fresh signer entry
(chain_depth = 0) for the CURRENT (post-change) fingerprint.

Bob's `cumulative_root_fingerprint` is his own Fresh fingerprint — i.e., the
current state. The audit sees: Bob's Fresh signature against current fingerprint.
Predicate passes. Execution tier. The cumulative drift from Alice's original
full-review is INVISIBLE in Bob's entry — Bob's `cumulative_root_fingerprint`
references only Bob's starting point, not Alice's.

**Does the predicate prevent this?**

Only if the predicate requires BOTH alice AND bob (via `all_of` or explicit
`required = ["alice", "bob"]`). For any `any_of` or single-signer-from-a-pool
predicate, rotating signers bypasses the cap.

**Is this wrong by design?**

Partially. Bob's Fresh attestation IS tier-honest from Bob's perspective — Bob is
claiming "I reviewed the current code." The problem is Bob was introduced BECAUSE
alice's cap was hit (to avoid alice doing a Fresh re-review of the cumulative
change). This is a social-engineering + structural bypass: deliberate collusion
between alice and bob to avoid Fresh re-attestation of the cumulative drift.

**Verdict**: LANDS. The safeguards prevent automatic/silent single-signer laundering.
They do NOT prevent deliberate rotation through new signers specifically to reset
the cumulative-drift visibility. This is a social-engineering vector that the
structural safeguards cannot prevent — only predicate design (requiring specific
named signers) and team culture can.

**Refinement T2R-A-R**: The v0.1 documentation for `attest delta` should warn
explicitly: "chain-depth caps apply per-signer; predicates that allow any-signer-
from-a-pool can be satisfied by fresh signers who haven't accumulated delta history;
predicates with named required signers are more resistant to rotation attacks."
Additionally: predicates with `any_of([signers(required = ["alice"]), signers(required = ["bob"])])` create
exactly this bypass opportunity; the `attest lint` command (T7-R) should warn when
a `fresh_within_days` + `any_of` combination creates a rotation bypass surface.

**Severity for v0.1**: LOW (v0.1 is git-trust anyway; social engineering is the
baseline attack surface). MEDIUM for teams who believe chain-depth cap prevents
cumulative drift in all cases.

---

## Attack T2R-B — Rationale schema regression: "non-empty" is not "meaningful"

**The v3 position**: "Required non-empty rationale: schema-enforced. Empty or
whitespace-only rationale is schema-rejected at sidecar-parse time."

**Attack**: the prior T2-R refinement proposal specified a minimum character count
("e.g., 20 chars minimum — prevents `--rationale 'ok'`"). v3's rolling absorption
simplified this to just "non-empty." The minimum was dropped in folding.

A rationale of `"ok"` (2 chars), `"fine"` (4 chars), `"reviewed"` (8 chars), or
`"changes are safe"` (16 chars) all satisfy the non-empty check. These are
semantically meaningless as rationale for a compliance-preserving carry-forward.
The intent of the required-rationale was to create an explicit record for future
audit — a rubber-stamp rationale defeats that intent.

**Is this a design defect or a human-process problem?**

It's a schema underspecification. The prior capture named the minimum; v3 dropped
it without a stated reason. The minimum is recoverable without architectural change
(it's a schema field constraint).

**Verdict**: LANDS as a regression from the prior capture's T2-R specification.
The schema should enforce a minimum length (20 chars or similar) to prevent
rubber-stamp rationales. This is a small fix.

**Refinement T2R-B-R**: Schema should enforce `rationale.len() >= 20` (or
workspace-configurable minimum, e.g., `attest.delta_rationale_min_chars = 20`).
CLI `attest delta` should reject rationale shorter than this minimum. The lint
command should check rationale quality (heuristic: if rationale is the same on
consecutive deltas, warn "rationale appears unchanged from prior delta; ensure
each carry-forward is independently assessed").

**Severity**: MEDIUM. The rationale minimum was explicitly in the prior capture.
Its absence in v3 is a folding regression.

---

## Attack T2R-C — Workspace-config chain cap has no floor: set to 999

**The v3 position**: "Hard cap: `chain_depth <= 3` (configurable in workspace
TOML; default 3)."

**Attack**: the workspace TOML can set `attest.delta_chain_max_depth = 999`.
Anyone with commit access to the workspace TOML can do this. The "hard cap" becomes
effectively infinite. This is identical to the `fresh_within_days(36500)` bypass
identified in Attack T7 of the prior pass — now applied to chain-depth.

There is no specified workspace-config FLOOR (minimum value below which workspaces
cannot go — i.e., workspaces can only TIGHTEN the cap, not loosen it beyond some
hard minimum).

**Verdict**: LANDS. The configurable chain-depth cap can be defeated by editing
the workspace TOML. The "hard cap" language in v3 is misleading — it implies a
non-configurable bound, but the bound IS configurable.

**Refinement T2R-C-R**:
1. Remove "hard cap" language; use "workspace-configured cap (default 3)".
2. Specify a HARD FLOOR that is not configurable: `chain_depth <= 10` (or similar)
   regardless of workspace config. Workspace can tighten (e.g., set max to 1 or 2)
   but cannot loosen beyond the hardcoded floor (e.g., cannot set max to 100).
3. The CLI should refuse to apply workspace configs that exceed the hardcoded floor,
   with an explicit error: "delta_chain_max_depth = N exceeds the maximum allowed
   value of 10; antigen enforces this as a non-configurable safety boundary."

**Severity**: HIGH. The "hard cap" framing is misleading. Teams relying on chain-
depth enforcement may not realize the default is overridable without floor.

---

## Attack TOL-A — Tolerance kind-discriminator: Immunity sidecar at Tolerance site

**The v3 position**: `RatificationKind { Immunity, Tolerance }` discriminator
on sidecar schema. Tolerance state mapping table covers: no sidecar, predicate
fails, predicate passes, but NOT the case where the sidecar EXISTS but with the
WRONG kind.

**Attack**: site has `#[antigen_tolerance(SignedZeroDiscipline, sidecar = true)]`.
The `.attest/SignedZeroDiscipline.json` file exists but has `kind = Immunity`.
(This happens if: the team had an Immunity attestation, switched to tolerance,
but didn't regenerate the sidecar; or if a sidecar was copied from another site.)

The audit reads the sidecar. The sidecar is schema-valid. The predicate evaluates.
v3's tolerance state mapping does not specify what happens on kind-mismatch. The
audit either:
(a) Silently succeeds — reports `tolerance-predicate-passed-substrate-current`
    (WRONG: the sidecar was for Immunity, not Tolerance)
(b) Silently fails — reports `tolerance-predicate-failed` or similar
    (WRONG: the predicate may have passed; the failure is in the kind, not the predicate)
(c) Reports `tolerance-sidecar-schema-invalid` — ALSO WRONG: the sidecar is
    schema-valid; the kind mismatch is a semantic error, not a schema error

None of (a), (b), (c) is the right answer. The right answer is a new audit state.

**Verdict**: LANDS. The kind-mismatch case is a missing audit state in BOTH the
immunity and tolerance state mapping tables.

**Refinement TOL-A-R**:
Add two new audit states to the state mapping tables:
- For immunity sites: "Sidecar exists, kind = Tolerance (wrong kind)" →
  `WitnessTier::None`, hint `discipline-sidecar-kind-mismatch-expected-immunity-got-tolerance`
- For tolerance sites: "Sidecar exists, kind = Immunity (wrong kind)" →
  `WitnessTier::None`, hint `tolerance-sidecar-kind-mismatch-expected-tolerance-got-immunity`

These hints should be explicit so CI gates can catch misconfigured sidecars
during the `scaffold → sign` workflow transition.

**Severity**: MEDIUM. The kind-mismatch case is a real workflow error scenario
(switching from immune to tolerating). Without explicit audit state for this case,
the behavior is undefined.

---

## Attack TOL-B — Tolerance kind-discriminator: Tolerance sidecar at Immunity site

Symmetric to TOL-A, direction reversed: `#[immune(X, requires = ...)]` at a
site, sidecar has `kind = Tolerance`. Same unspecified audit behavior. Same
verdict. Covered by TOL-A-R's second new state.

**Verdict**: LANDS (same root cause, same fix).

---

## Attack TOL-C — Simultaneous Immunity + Tolerance macros on same function for same antigen

**The v3 position**: `#[immune(X, requires = ...)]` and
`#[antigen_tolerance(X, sidecar = true, requires = ...)]` can both exist in the
Rust source as macro attributes. v3 doesn't specify compile-time validation that
prevents both from decorating the same function for the same antigen.

**Attack**: a developer writes:

```rust
#[immune(SignedZeroDiscipline, requires = signers(required = ["alice"]))]
#[antigen_tolerance(SignedZeroDiscipline, sidecar = true, requires = signers(required = ["alice"]))]
pub fn sinh(x: f64) -> f64 { ... }
```

This is logically contradictory — a site cannot simultaneously be immune to
(compliant with) and tolerating (non-compliant with) the same discipline. But
the macros don't know about each other at expansion time (macro expansion is
typically independent).

The audit would then find: (a) `#[immune]` declaration + corresponding sidecar
+ immunity predicate passes → Execution tier immunity; (b) `#[antigen_tolerance]`
declaration + corresponding sidecar + tolerance predicate passes → Execution
tier tolerance. Both report for the same antigen at the same site.

The audit output contains logically contradictory information: site is both
immune and tolerating. There is no specified hint for this case.

**Is the single-file-per-antigen convention a guard here?**

NOT fully. The immunity sidecar and the tolerance sidecar would both try to write
to `.attest/SignedZeroDiscipline.json`. They'd collide on disk. But: the sidecar's
`kind` field disambiguates — whichever was written LAST wins. The audit would then
see one sidecar (one kind) and potentially mismatched with one of the macros.
This reduces to TOL-A or TOL-B (kind-mismatch case).

However: if the schema is extended to allow both Immunity and Tolerance sidecars
in the SAME JSON file (not currently proposed but a natural evolution for
coexistent claims), the collision is resolved and the contradiction becomes
explicitly representable. This is a long-arc concern.

**Verdict**: PARTIAL LAND. The single-file convention prevents full disk-level
coexistence. But: (a) the compile-time validation gap is real — macros don't
reject the combination; (b) the resulting on-disk state (one sidecar,
potentially mismatched with the losing macro) reduces to the TOL-A/B kind-mismatch
case, which IS a gap. The fix for TOL-A/B resolves the worst outcome. The
compile-time validation would be a quality-of-life improvement (clear error
instead of mysterious behavior).

**Refinement TOL-C-R**: The proc-macro for `#[antigen_tolerance]` should, at
macro expansion time, check for coexistent `#[immune]` attributes on the same
function for the same antigen and emit a `compile_error!`. This requires
attribute-aware macro logic (reading other attributes on the item) which is
possible with `syn`. This prevents the contradiction at compile time before
it reaches the sidecar layer.

**Severity**: LOW for v0.1 (the file collision naturally resolves to a detectable
kind-mismatch). MEDIUM for long-arc (if schema evolves to coexistent kinds).

---

## Attack TOL-D — Migration burden for existing #[antigen_tolerance] sites

**The v3 position**: `#[antigen_tolerance(X)]` without `sidecar = true` → None tier,
`tolerance-vibes-grade` hint. This is correct and intentional.

**Attack**: all EXISTING antigen codebases that use `#[antigen_tolerance(X)]`
(without sidecar = true, because the parameter didn't exist before v3) would
see a flood of `tolerance-vibes-grade` hints in CI when they upgrade to the
v0.1-rc antigen version that ships this. The migration path to Execution tier
requires adding `sidecar = true` AND creating the sidecar via `cargo antigen
tolerate scaffold` AND signing via `cargo antigen tolerate sign`. This is a
per-site migration burden that scales with the number of existing tolerance
annotations.

**Is this wrong?** No — making the vibes-grade gap VISIBLE is the feature.
But the adoption path needs explicit documentation: teams upgrading to v0.1-rc
should expect CI to flood with `tolerance-vibes-grade` hints for all existing
tolerances. Without documentation, this looks like a regression.

**Verdict**: PARTIAL LAND (documentation/migration-guidance gap, not design flaw).
The behavior is correct. The migration burden is unstated.

**Refinement TOL-D-R**: v0.1-rc release notes must include an explicit section:
"Breaking change in tolerance reporting: `#[antigen_tolerance(X)]` without
`sidecar = true` now reports `tolerance-vibes-grade` (None tier). This was
always a tier-honesty gap; v0.1-rc makes it visible. To migrate, add
`sidecar = true` to each tolerance annotation and run `cargo antigen tolerate
scaffold` + `cargo antigen tolerate sign`. Teams who prefer to continue with
vibes-grade tolerance can suppress the `tolerance-vibes-grade` hint in their
CI configuration."

---

## Attack T4-A — Compound evidence: Immunity + Tolerance logical contradiction in audit output

**The v3 position**: T4 is an open question — "compound evidence (behavioral
test + substrate signatures on same antigen-site) overclaim surface." The prior
T3-R addressed compound_evidence for multiple witnesses of ONE claim type.

**Attack**: site has BOTH `#[immune(SignedZeroDiscipline)]` AND
`#[antigen_tolerance(SignedZeroDiscipline, sidecar = true)]`. Both sidecars pass.
The audit emits entries for both.

A CI gate configured to "require no tolerance hints" would reject this site even
though it also has immunity — which is backward (immunity is STRONGER than
tolerance; the site should be passing the "no tolerance" gate because it has
immunity). A CI gate configured to "require Execution immunity" would pass — and
might not notice the tolerance claim that contradicts the immunity.

The combination is LOGICALLY INCOHERENT: the site cannot simultaneously be
immune (compliant) and tolerating (non-compliant). The audit reports both without
a diagnostic.

**This is NOT addressed by T3-R's compound_evidence boolean**: T3-R flags multiple
witnesses of the SAME claim type. This is compound evidence across CLAIM TYPES
(immunity AND tolerance) — a stronger incoherence that T3-R's flag doesn't capture.

**Verdict**: LANDS. The audit needs an explicit incoherence hint for the
immunity-AND-tolerance-coexistence case. This hint should be emitted regardless
of which sidecar kind the single on-disk file has.

**Refinement T4-A-R**: Add a new audit state:
"Site has both `#[immune(X)]` and `#[antigen_tolerance(X)]` macros declared" →
emit hint `discipline-immunity-tolerance-contradiction` at None tier (overrides
both individual reports). The contradiction is the operative state; reporting
individual tiers in addition to the contradiction hint is confusing.

The proc-macro compile-time check (TOL-C-R) would catch this earlier, but the
audit must also handle the case where macro + sidecar are out of sync.

**Severity**: MEDIUM. Teams using both `#[immune]` and `#[antigen_tolerance]`
on the same antigen at the same site will see confusing audit output.

---

## Attack T4-B — compound_evidence boolean: no actionable consumer guidance

**The v3 position (T3-R absorbed)**: when multiple witnesses report different
EvidenceKind for the same site, emit `compound_evidence = true` with note
"multiple evidence kinds present; independence not audited."

**Attack**: what should a consumer DO with `compound_evidence = true`?

Option 1: Ignore it (no behavioral change — the additive-confidence illusion persists)
Option 2: Treat any compound evidence as suspicious (over-rejection — independent
verification from multiple angles is flagged as suspicious)
Option 3: Write custom CI logic per EvidenceKind combination — but this requires
the consumer to know which kinds are present, not just that compound is true.

The `compound_evidence = true` boolean without a `compound_evidence_kinds: Vec<EvidenceKind>`
field forces consumers to re-derive which kinds were combined from the individual
witness entries. This is available in the full audit output but requires parsing
the entries rather than reading a single field.

**Verdict**: PARTIAL LAND. `compound_evidence = true` is underspecified as a
consumer signal. The boolean should be accompanied by the list of EvidenceKind
values that are combined, enabling consumers to write targeted CI logic.

**Refinement T4-B-R**: Upgrade `compound_evidence: bool` to
`compound_evidence: Option<Vec<EvidenceKind>>` (None when single evidence kind;
Some(kinds) when multiple). Consumers can then write:
```
compound_evidence.as_ref().map(|k| k.contains(&Behavioral) && k.contains(&SubstrateState))
```
This enables targeted consumer logic without requiring full entry parsing.

**Severity**: LOW. The boolean is better than nothing; the kind-list is
incrementally better.

---

## Attack T5-A — WASM sandbox: logic inversion is valid WASM

**The v3 position**: WASM sandboxing is "the most robust enforcement" for leaf
providers and "catches ALL four malicious behaviors" from the prior pass.

**Attack**: WASM sandbox enforces RESOURCE constraints (network, env-var, memory,
time). It does NOT enforce LOGIC correctness. A leaf that is deterministic,
terminating, side-effect-free, and bounded — but always returns `true` regardless
of input — satisfies all WASM sandbox constraints while being completely useless
(or malicious).

A "logic-inverted" leaf — returns `true` when the discipline is NOT followed,
`false` when it IS followed — is fully valid WASM. The sandbox has no way to
verify the relationship between the leaf's input (the substrate) and its output
(the tier claim).

The prior pass (T6 core tier-honesty framing) established: "every tier verifies
structure-of-evidence, not truth; truth is developer responsibility." This limit
applies recursively at the leaf level — the sandbox verifies structure-of-resource-
usage, not truth-of-evaluation.

**Does this mean WASM is useless?**

No. WASM is robust for the four specific malicious behaviors from the prior pass
(network exfiltration, env-var side channels, DoS hangs, panic-targeting). Logic
inversion is a fundamentally different attack — it requires the leaf AUTHOR to
deliberately invert the logic, which is detectable via code review of the leaf.

**Verdict**: PARTIAL LAND. WASM is correctly described as robust for resource
constraints. The prior pass's claim that it "catches ALL four malicious behaviors"
is still true (those four were specific resource-constraint violations). But WASM
does NOT prevent logic-level attacks. The ADR-019 description of WASM as the
recommended option must clarify: WASM prevents resource-constraint violations but
not logic-level manipulation; code review of leaf provider source is still required
for tier-honest leaf evaluation.

**Refinement T5-A-R**: The v0.2+ leaf-provider ADR description of enforcement
mechanisms must include: "WASM sandboxing prevents resource-constraint violations
(network, env-var, memory, time). It does NOT verify the logic correctness of
the leaf. Code review of leaf provider source code is required as the complementary
discipline-layer. Leaf providers must publish their source under an open license
and document their evaluation logic in their README."

---

## Attack T5-B — `no_std` + restricted-dep bypass: unsafe core spin loop

**The v3 position**: `no_std` + restricted-dep-check pre-screens leaf crates;
prevents high-level API misuse (network, env-vars via `std`).

**Attack**: a `no_std` leaf with only `core` dependencies can include:
```rust
#[no_mangle]
pub extern "C" fn evaluate(/*substrate params*/) -> bool {
    // Allowed: core::hint::spin_loop is in core; not a restricted dep
    loop { core::hint::spin_loop(); }
}
```

This is `no_std`, has only `core` dependencies, passes the restricted-dep-check,
and causes the audit to hang indefinitely (DoS). `core::hint::spin_loop()` is a
permitted `core` function; no restricted dep is needed to cause an unbounded loop.

Additionally: `unsafe` blocks in `core` allow raw pointer arithmetic, potentially
reading memory adjacent to the leaf's own stack frame (host-process memory
introspection attempt). Whether this succeeds depends on WASM/process isolation,
but `no_std` dep-check doesn't prevent `unsafe` from being written.

**Verdict**: LANDS. `no_std` dep-check is a useful pre-screen but is insufficient
as the ONLY enforcement mechanism because (a) permitted `core` functions enable
unbounded computation; (b) `unsafe` is not forbidden by dep-check; (c) logic
inversion requires no restricted deps. The ADR-019 must not present `no_std`
dep-check as "enforcement" — it's a "pre-screen" or "build-time hygiene check."

**Refinement T5-B-R**: Rename "Option B: `no_std` + restricted-dep-check at
build time" to "Option B: `no_std` + restricted-dep pre-screen (NOT enforcement)."
The description should clarify this is a necessary-but-not-sufficient condition,
complementary to WASM sandboxing (Option A) or subprocess isolation (Option C).
It cannot stand alone as the enforcement mechanism.

---

## Attack T5-C — Subprocess isolation on Windows: network not restricted

**The v3 position**: "subprocess isolation with timeout + memory cap" (Option C)
provides runtime, medium-cost, OS-level isolation.

**Attack on Windows (the platform in this project — win32)**:

Windows job objects enforce: memory limits (`MaximumWorkingSetSize`), CPU time
(`PerJobUserTimeLimit`). Windows job objects do NOT restrict network access — there
is no `seccomp`-equivalent for network syscalls on Windows. A leaf subprocess in a
Windows job object can freely call `WinSock` and exfiltrate data.

Additionally: `CREATE_BREAKAWAY_FROM_JOB` in a `CreateProcess` call creates a
grandchild process OUTSIDE the job object limits. If the audit doesn't prevent this
flag (and preventing it requires explicit job object configuration:
`JOB_OBJECT_LIMIT_BREAKAWAY_OK` must NOT be set), the leaf spawns an unconstrained
grandchild that escapes all limits.

**On Unix**: subprocess isolation is stronger — `seccomp` (Linux) can restrict
network syscalls; namespaces can isolate the filesystem. But antigen's current
platform is Windows.

**Verdict**: LANDS for Windows. Subprocess isolation is Platform-dependent and
is weaker on Windows than the generic description implies. ADR-019 must acknowledge
platform specificity: on Windows, subprocess isolation (Option C) does NOT restrict
network access unless additional mitigations (firewall rules, Windows Filtering
Platform) are applied.

**Refinement T5-C-R**: The v0.2+ leaf-provider ADR must include platform-specific
analysis: "On Unix: seccomp + namespaces can restrict network, filesystem, and
process creation within the subprocess. On Windows: job objects restrict memory and
CPU time but NOT network access. Windows-specific network restriction requires WFP
(Windows Filtering Platform) rules applied to the job's token — a significant
infrastructure addition. Recommendation: on Windows, combine Option C with
Option A (WASM) for network-restricted enforcement."

---

## Attack FA2-A — Fingerprint-scheme evolution: deferral timing is wrong if v0.2 touches fingerprinting

**The v3 position**: T7 deferred to "Future amendment." v3 notes the scope
field (aristotle F3) adds `site | file | package | workspace` scope to antigen
declarations.

**Attack**: v0.2 adds k-of-n threshold signatures AND CODEOWNERS interop AND
scope field behavior changes. If any of these change how `antigen_fingerprint`
computes fingerprints (e.g., incorporating scope into the hash; including a wider
context for `file` scope), all v0.1 sidecars with v0.1 fingerprints become
false-stale when audited against v0.2.

The T7 deferral to "Future amendment" (v0.5+) is wrong if v0.2 touches fingerprinting
scope. The timing of when T7 is needed depends on when `antigen_fingerprint`
changes — a dependency not stated in v3.

**Verdict**: LANDS. The T7 deferral timing is implicitly tied to fingerprint
scheme stability. If the scheme is not stable across v0.1 → v0.2, T7 is a
v0.2 critical path item.

---

## Attack FA2-B — Fingerprint migration is inherently trust-breaking

**The v3 position**: T7 deferred. No migration policy specified.

**Attack**: any migration tool that updates `signed_against_fingerprint` from v0.1
to v0.2 scheme is CHANGING the value that alice "signed." Alice signed against
fingerprint X (v0.1 scheme). After migration, the sidecar says she signed against
fingerprint Y (v0.2 scheme). Her attestation's semantic meaning has changed without
her consent.

Three migration options, none cost-free:
1. Re-attestation required: downgrade to Reachability until alice re-signs under
   v0.2 scheme. Tier-honest but creates mass downgrade + re-attestation burden.
2. Dual-scheme support: audit carries all prior scheme implementations; old sidecars
   are evaluated against their scheme version; new sidecars against current. Long-term
   maintenance debt.
3. Scheme stability commitment: `antigen_fingerprint` is immutable per ADR-019
   version. Any fingerprint change is a breaking change requiring a new major version.

**Verdict**: LANDS. The migration policy is a real architectural decision that
must be made at design time. Deferring it means v0.2 teams could be surprised by
the choice. The minimum viable preparation is adding a `fingerprint_scheme_version`
field to the sidecar schema NOW (in v0.1), before any scheme changes happen — this
field enables the audit to detect scheme mismatch and emit a specific hint rather
than false-stale.

**Refinement FA2-R (covers FA2-A and FA2-B)**:
1. `Ratification` schema: add `fingerprint_scheme_version: SchemaVersion` field
   (populated by `attest scaffold`; current scheme version stored alongside fingerprint).
2. New audit state: "Sidecar exists, `fingerprint_scheme_version` != current scheme" →
   `WitnessTier::Reachability`, hint `discipline-fingerprint-scheme-mismatch`.
   This distinguishes false-stale (scheme change) from real-stale (code change).
3. ADR-019 must state whether `antigen_fingerprint` scheme is STABLE across minor
   versions. If yes: commit it as a crate guarantee and add a test that prevents
   scheme changes without a major version bump. If no: name the migration policy
   now (Option 1, 2, or 3 above) and add the `fingerprint_scheme_version` field.
4. Delta chains: when scheme mismatch is detected in a delta chain entry, the audit
   must treat the ENTIRE chain as requiring Fresh re-attestation (the cumulative
   root fingerprint is now incomparable).

---

## Attack FA2-C — Delta chains become semantically broken across scheme versions

**Attack**: a delta chain established under v0.1 contains:
```
cumulative_root_fingerprint: <v0.1 hash of item>
prior_fingerprint: <v0.1 hash of item at prior state>
chain_depth: 2
```

v0.2 audit computes: v0.2 hash of current item ≠ v0.1 `prior_fingerprint`. The
audit cannot distinguish: (a) code changed (real stale); (b) fingerprint scheme
changed (false stale). The delta chain's anti-laundering guarantees are degraded —
the `cumulative_root_fingerprint` comparison is across schemes and is meaningless.

**Verdict**: LANDS (same root cause as FA2-B, specific manifestation in delta chains).
Covered by FA2-R item 4.

---

## Attack FA5-A — descended_from: audit has no access to parent predicate

**The v3 position**: `#[descended_from = "A::SignedZeroDiscipline"]` + per-consumer
ratification (R-A7). Crate B writes its own sidecar with its own predicate.

**Attack**: the audit evaluates Crate B's sidecar against Crate B's predicate.
There is NO cross-crate predicate lookup. The audit doesn't know what Crate A's
predicate was. Crate B can trivially write a weaker predicate — `signers(required = ["carol"])`
instead of A's `all_of([signers(required = ["alice", "bob"]), oracles_complete(...)])` —
and the audit reports Execution tier as if the full rigor of the parent predicate
was satisfied.

**Verdict**: LANDS. This is the core FA-5 attack. The `descended_from` relationship
is structural but carries no predicate-strength implication in the current design.

---

## Attack FA5-B — descended_from + weaker predicate creates cross-crate laundering path

**Attack**: downstream crates (Crate C imports Crate B) see Execution-tier
`SignedZeroDiscipline` for Crate B's `#[descended_from]` declaration. They don't
know Crate B's predicate was weaker than Crate A's. The audit output for Crate B
says `discipline-predicate-passed-substrate-current` — it doesn't say which
predicate passed. The cross-crate predicate-strength gap is invisible.

**Verdict**: LANDS. The audit hint is predicate-pass confirmation without predicate
content. Downstream crates cannot verify predicate strength from the hint alone.

---

## Attack FA5-C — descended_from semantic guarantee is undefined

**Attack**: v3 never specifies the predicate CONTRACT for `#[descended_from]`.
Three possible semantics:
- "I implement the same failure-class" (no predicate implications — weakening allowed)
- "I implement with at least the parent's rigor" (predicate strength must be ≥ parent)
- "I implement and have been reviewed against the parent's discipline doc" (doc reference required)

Without specification, `descended_from` is a label without behavioral enforcement.
The entire T10-R `required_signer_roles` improvement was a partial mitigation for one
dimension (signer roles), but the PREDICATE CONTRACT itself is undefined.

**Verdict**: LANDS. This is a fundamental specification gap, not a specific bypass.

**Refinement FA5-R (covers FA5-A through FA5-C)**:
1. ADR-019 must specify the predicate contract for `#[descended_from]`: explicitly
   state whether weakening is ALLOWED or PROHIBITED.
2. If weakening is ALLOWED: the audit must include the effective predicate expression
   in its output (the predicate text that was evaluated), so downstream consumers can
   assess predicate strength independently. The `discipline-predicate-passed-substrate-current`
   hint alone is insufficient — add `evaluated_predicate: String` to the audit output.
3. If weakening is PROHIBITED: a cross-crate predicate resolution mechanism is required.
   This is a significant infrastructure addition; name it as a v0.2+ ADR target.
4. Minimum viable for v0.1: weakening is ALLOWED but must be DECLARED. Add
   `weakened_from: Option<AntigenId>` and `weakening_rationale: Option<String>` to
   `Ratification` schema. When `weakened_from` is set, the audit emits
   `discipline-predicate-weakened-from-parent` hint and includes the rationale in output.
   This makes weakening explicit, auditable, and visible to downstream consumers.
   When `weakened_from` is absent and the antigen has a `#[descended_from]` declaration,
   the audit should warn `discipline-predicate-weakening-undeclared` — making
   undeclared weakening visible without prohibiting it.

---

## Summary: Attacks that LAND

**HIGH severity (blocks v0.1 correctness / design clarity)**:

**T2R-C (chain-cap has no floor)** — "hard cap" language is misleading; workspace
admin can set it to 999; needs an actual non-configurable floor. Refinement: specify
hardcoded floor (e.g., 10) that workspace config cannot exceed.

**FA5 (descended_from predicate contract undefined)** — The predicate contract for
`#[descended_from]` is never specified. Weakening is silently allowed and
invisible to audit consumers. Minimum viable: explicit `weakened_from` field +
`discipline-predicate-weakening-undeclared` warning. Audit must include evaluated
predicate expression in output.

**FA2 (fingerprint-scheme evolution migration policy undefined)** — The deferral
is risky if v0.2 touches fingerprinting scope. Minimum viable: add
`fingerprint_scheme_version` field to schema NOW; add `discipline-fingerprint-scheme-mismatch`
audit state. State explicitly whether the scheme is stable across minor versions.

**MEDIUM severity (design gaps with specific fix)**:

**TOL-A/B (kind-mismatch audit state missing)** — Immunity sidecar at Tolerance
site (and vice versa) hits undefined audit behavior. Fix: two new audit states with
explicit hints.

**T4-A (immunity + tolerance contradiction lacks incoherence hint)** — Site with
both `#[immune]` and `#[antigen_tolerance]` for same antigen produces logically
contradictory audit output with no diagnostic. Fix: `discipline-immunity-tolerance-contradiction`
hint at None tier.

**T2R-B (rationale minimum dropped in v3 folding)** — Prior T2-R specified minimum
character count; v3 folded to just "non-empty." Rubber-stamp rationale of "ok" satisfies
the check. Fix: minimum 20 chars (or workspace-configurable minimum).

**T5-A/B/C (enforcement mechanism descriptions need scope clarification)** — WASM
prevents resource violations but not logic inversion; `no_std` dep-check is a
pre-screen not enforcement; subprocess isolation is platform-dependent (weak on
Windows without additional network restriction). Fix: clarify scope of each option
in ADR-019's v0.2+ leaf-provider ADR section.

**T5-C-Windows (subprocess isolation doesn't restrict network on Windows)** — The
project runs on win32. Subprocess isolation (Option C) leaves network open on Windows
without WFP rules. Fix: platform-specific analysis in ADR-019.

**LOW severity (quality / documentation gaps)**:

**T2R-A (rotating signers bypass per-signer chain cap)** — Social-engineering
vector; structural safeguards can't prevent deliberate collusion. Document as known
limitation; lint warning for `any_of([signers(...),...])` patterns.

**TOL-C (compile-time validation gap for coexistent immune + tolerance macros)** —
Proc-macro doesn't reject `#[immune(X)] + #[antigen_tolerance(X)]` at compile time.
Fix: attribute-aware check in proc-macro expansion.

**TOL-D (migration burden for existing tolerances undocumented)** — All existing
`#[antigen_tolerance(X)]` become vibes-grade; teams need explicit migration guidance
in release notes.

**T4-B (compound_evidence boolean needs kind list)** — `compound_evidence: bool`
upgraded to `compound_evidence: Option<Vec<EvidenceKind>>` for targeted consumer logic.

---

## Attacks that DO NOT land

**T2R-A core behavior**: rotating through new signers where each signer does a
Fresh attestation is TIER-HONEST from each signer's perspective. The design
correctly doesn't prevent this — it can't without requiring all-signers-must-have-
reviewed-cumulatively, which is too strict a social requirement to encode structurally.
The tier-honesty claim is accurate; the documentation limitation is what lands.

**TOL-D core behavior**: vibes-grade for existing unannotated tolerances is CORRECT.
The behavior is intentional. Only the documentation guidance lands.

**T5 core tier-honesty framing**: "every tier verifies structure-of-evidence, not
truth" survives. WASM sandboxing doesn't need to verify logic correctness because
the tier-honesty framing already acknowledges that truth-verification is developer
responsibility. The refinements clarify the SCOPE of each enforcement mechanism,
not the framing itself.

---

## New frontier attacks (for next adversarial pass)

**NFA-1 — `oracles_complete` oracle content quality is unauditable**:
The `oracles_complete(files)` leaf checks that files exist with `status: complete`
markers. It cannot check oracle DEPTH or CORRECTNESS. A 2-sentence oracle and a
40-page mathematical proof both satisfy `oracles_complete`. This is explicitly
acknowledged in T10-R (prior pass) as "oracle depth is inherently a human-judgment
problem." But: the audit doesn't even emit the oracle file contents or size in its
output — consumers can't see what they're relying on. Should the audit at least
report oracle file size or line count as a supplementary output?

**NFA-2 — `fresh_within_days` combined with delta chain creates freshness anomaly**:
Predicate: `all_of([signers(required = ["alice"]), fresh_within_days(30)])`.
Alice does a Fresh attestation. Three weeks later, alice does a delta (chain_depth=1).
The delta's date is within 30 days. But the CONTENT alice reviewed was 3 weeks old
at delta time — she reviewed a delta, not the current state. The `fresh_within_days`
check uses the most recent signature date (which is the delta date), but the "reviewed
state" is the delta content, not necessarily the full current code. Is the
freshness predicate checking "someone looked at this recently" or "the full content
was reviewed recently"? These are different things when delta attestations are involved.

**NFA-3 — Schema version compatibility gap**:
v3 adds `fingerprint_scheme_version` (recommended above) and `evidence_provenance`
(scout S2). As schema fields accumulate across versions, old sidecars may be missing
new required fields. The serde deserialization behavior (reject missing fields /
default missing fields / warn on missing fields) is not specified. Missing a
`fingerprint_scheme_version` field on an old sidecar — should that be treated as
"scheme v1" (infer from absence) or "unknown scheme" (conservative Reachability)?

**NFA-4 — `attest gc` removes orphaned entries but not orphaned sidecars**:
`gc --dry-run` (default) / `gc --commit`. If a function is deleted from the source
file, its `#[immune]` or `#[antigen_tolerance]` annotation is gone but the `.attest/`
sidecar file remains on disk. `gc` finds orphaned entries (antigen presented in
source but no sidecar) — but does it find orphaned SIDECARS (sidecar exists but no
corresponding annotation in source)? The direction matters: an orphaned sidecar is
wasted space; an orphaned entry is a missing attestation. Both should be detected.

---

READY FOR REVIEW
