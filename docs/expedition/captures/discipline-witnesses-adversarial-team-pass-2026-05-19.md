# Capture — Adversarial Team-Pass on Discipline-Witnesses v2 + Aristotle F-Findings

> **Date**: 2026-05-19
> **Author**: team-adversarial (Sonnet 4.6, adversarial role)
> **Relation to prior captures**: attacks the v2 + aristotle-team-pass
> (F1-F9) frontier. The self-attack capture (adversarial-self-attack-2026-05-18)
> settled eight attacks and named refinements R-A1..R-A9. The aristotle
> team-pass settled F1-F8 and named the frontier questions for this pass
> explicitly. This capture attacks THOSE frontier questions (F7, F5, F8,
> F4, F1 as named by aristotle) plus additional surfaces aristotle didn't
> visit (tier-overclaim edges, predicate-language closure gaps,
> `.attest/` convention conflicts, CLI workflow gaps, R-A7 cross-crate
> ratification worst cases).
> **Discipline**: I do not inherit prior passes' findings as settled unless
> they genuinely hold under hostile scrutiny. Where a finding ratifies under
> attack, that's named. Where an attack lands, that's a new finding.
> **Status**: append-only capture

---

## Posture

The adversarial posture from the self-attack capture carries forward:
no benefit-of-the-doubt, "by construction" is suspect, "obviously"
is a red flag, edge cases get explored rather than acknowledged.

The self-pass attacks settled the easy surfaces. This pass operates at
the frontier aristotle explicitly named: F7 (leaf-provider trust),
F5 (carry-forward laundering), F8 (EvidenceKind overclaim), F4
(closed-set tool invocation boundary), F1 (discipline-vs-machinery
unification drift). Plus my own additional attack surfaces from reading
the full substrate.

Every attack below is a named specific failure mode — not a vague
concern. Where a refinement is needed, it names WHAT specifically.
Where the design holds, it names WHY it holds rather than just saying
"survives."

---

## Attack T1 — F7 (leaf-provider trust): the leaf-contract doesn't survive compile-time

**The aristotle position (F7)**: leaf-providers re-introduce
trust-the-witness at the leaf level. Mitigation: (a) leaf-contract
specification (deterministic, terminating, side-effect-bounded,
declared-tier), (b) default cap at Reachability for third-party leaves,
(c) workspace-config opt-in for higher tiers.

**Attack**: the leaf-contract is documented in a README or Rust trait
with doc-comments. Nothing enforces it at compile time or audit runtime.
A malicious leaf-provider crate implements the trait, claims it's
deterministic and terminating, and ships a leaf that:
- Reads environment variables (`std::env::var`) beyond the declared
  substrate-reads (silent side-channel; env vars can carry secrets)
- Calls `std::thread::sleep(Duration::from_secs(rand::thread_rng().gen_range(0..60)))` — non-deterministic, non-terminating under adversarial conditions, not detectable by the trait definition
- Returns `true` for inputs where alice's GitHub username appears in
  a network-fetched allowlist (network call during audit — side effect
  outside declared substrate)
- Panics on specific input fingerprints to targeted-denial-of-service
  the audit for specific code sites

**Can any of aristotle's three mitigations catch these?**

Mitigation (a) — leaf-contract spec: NO. A contract in documentation
doesn't prevent non-compliant implementation. The leaf author claims
compliance; the audit cannot introspect the leaf's source to verify.
This is the same problem as trust-the-witness, now at the leaf level.

Mitigation (b) — default cap at Reachability: PARTIAL. Capping
at Reachability means the malicious leaf CANNOT inflate tier by
itself. A user who opts in via (c) is the target. But Reachability
cap doesn't prevent the leaf from (i) causing audit hangs,
(ii) exfiltrating environment-var data, (iii) crashing the audit
process. Tier-honesty is preserved; safety is not.

Mitigation (c) — workspace-config opt-in: NO PROTECTION beyond tier.
Opting in allows the leaf to report Execution-tier when it returns
true. An adversarial leaf that always returns true and has a
network-call side effect is now reporting false-Execution-tier. The
workspace-config opt-in is trust-extension WITHOUT verification —
which is the problem it was supposed to solve.

**What enforcement is actually robust?**

Three realistic options; none is cost-free:

**Option 1 — WASM-style sandboxing**: compile the leaf provider as
a WASM module; the audit's WASM runtime enforces (i) no network
access, (ii) no env-var access, (iii) memory cap, (iv) execution
timeout. This is the most robust enforcement. Cost: significant
infrastructure (WASM runtime dep; leaf authors must compile to
`wasm32-unknown-unknown`; ABI boundary). Precedent: Extism, Wasmer,
Wazero are battle-tested; Rust WASM compilation is mature. The cost
is real but not prohibitive for a future ADR. WASM sandboxing is
the only option that catches ALL four of the malicious behaviors
listed above.

**Option 2 — `no_std` + restricted imports enforcement**: require
leaf-provider crates to be `no_std` with explicit `[dependencies]`
allowlist (no `std::net`, no `std::env`, only `core` + approved
data-struct crates). Cargo's `[package.features]` can't enforce this
at compile time without a custom build script or unstable flag;
enforcement is audit-time (cargo-deny-style dep-graph check on the
leaf crate's transitive closure). This catches network calls and
env-var reads if they route through restricted deps. Does NOT catch:
non-termination (timing), panics, side channels via allowed deps.

**Option 3 — timeout + memory cap at runtime without sandboxing**:
the audit wraps leaf invocation in a separate process with OS-level
resource limits (Unix `rlimit`, Windows job objects). This bounds
timing and memory. Does NOT prevent env-var reads, network calls,
or false-Execution-tier reporting from a leaf that always returns
true.

**Verdict on the attack**: LANDS. The three aristotle mitigations
are insufficient for a full trust-the-witness defense at the leaf
level. The leaf-contract is documentation-trust, not enforcement.
Default-Reachability-cap prevents tier-inflation from non-opted-in
leaves but doesn't prevent sandboxing-escaped side effects.
Workspace-config opt-in doesn't verify the leaf is actually
contract-compliant before granting higher-tier reporting.

**The design holds at v0.1 because v0.1 has a sealed leaf set.**
The attack is entirely at v0.2+'s leaf-provider extension point.
But aristotle's F7 summary says the leaf-contract is the mitigation;
the attack shows the leaf-contract alone is insufficient without
runtime enforcement. The v0.2+ leaf-provider ADR must name which
enforcement mechanism it ships (WASM, `no_std`+restricted-dep-check,
or timeout+memory-cap) — documentation-trust alone is not sufficient.

**Refinement T1-R**: The v0.2+ leaf-provider ADR must specify
RUNTIME ENFORCEMENT of the leaf-contract, not just the contract
spec. The minimum viable enforcement for v0.2+ is Option 3
(timeout + memory cap via subprocess isolation) with Option 1
(WASM) as the long-term target. Option 2 (`no_std` dep-check) is
a build-time pre-screen but not a runtime enforcement mechanism.
Document explicitly in the deferral note in ADR-019: "Deferred
pending runtime enforcement design, not just contract specification."

**Weakest realistic attack bypassing each mitigation specifically**:
- Bypasses leaf-contract spec: any implementation that compiles and claims to be compliant.
- Bypasses Reachability cap: workspace-config opt-in to Execution. A leaf that always returns `true` is then reporting false-Execution-tier. The only mitigation is the workspace admin trusting the leaf author.
- Bypasses workspace-config opt-in: social engineering the workspace admin ("our leaf has been in production at 12 major Rust projects, it's obviously safe").

---

## Attack T2 — F5 (carry-forward laundering): the size-cap and diff-review are both bypassable

**The aristotle position (F5)**: `attest sign --carry-forward-from X`
lets alice assert "I reviewed the diff from X to current and it
preserves compliance." Schema needs `Signer.basis` field
(`full_review` | `carry_forward_from(X)` | `doc_version_carry_forward`).
F5 noted that "an adversarial pass should attack: can `carry_forward_from X`
be abused to launder bad changes? Mitigation: diff-size cap, time-cap,
required doc-reference."

**Attack**: constructing a chain of carry-forwards that smuggles a
substantial semantic change through individually-small diffs.

Alice's function `compute_bounds` has a `SignedZeroDiscipline` attestation.
The function has 80 lines. Alice signs with `full_review`. Then:

1. **Commit 1**: refactor variable names (10 lines changed). Alice
   `--carry-forward-from A` — diff is 10 lines, within any plausible
   size cap. Basis = `carry_forward_from(A)`.

2. **Commit 2**: extract helper function (15 lines changed). Alice
   `--carry-forward-from B`. Diff 15 lines, within cap.

3. **Commit 3**: change the helper function's logic to remove a
   signed-zero guard (12 lines changed). Alice `--carry-forward-from C`.
   Diff 12 lines, within cap. **This is the malicious commit.** But
   viewed as a 12-line diff from the previous state, it might plausibly
   look like "just another refactor" — especially if the guard is now
   in a slightly different form (e.g., moved to a separate inline path).

4. **Commit 4**: inline the helper back (9 lines changed). Alice
   `--carry-forward-from D`. The function now looks very different from
   the original (basis signed by alice at A) but each step in the chain
   was individually carry-forwarded.

The `Signer.basis = carry_forward_from(A)` chain means: alice's
original `full_review` is now several carry-forward steps removed
from the current code. The predicate passes; fingerprint is current
(alice resigned at each step); audit reports Execution-tier.

**Does a diff-size cap fix this?**

NO. Four 12-line diffs are the same total change as one 48-line diff.
The size cap per step doesn't prevent cumulative laundering. It's
trivial to batch the malicious change into size-cap-sized steps.

**Does a time-cap fix this?**

NO. The chain above could unfold over 4 days if the cap is "per week."
Or over 4 hours if there's no time cap. Time caps bound the window
for a single carry-forward, not the depth of the carry-forward chain.

**Does requiring an explicit doc-reference fix this?**

PARTIALLY. If carry-forward requires alice to name which invariant
section of the discipline doc still holds, a malicious alice would have
to explicitly claim "signed-zero guard is maintained per section 3.2."
A reviewer (or future audit) can then check that claim against the code.
This shifts the failure from "silent laundering" to "explicit false claim
in the record." Stronger, but still depends on downstream reviewer
reading the chain.

**What actually constrains this?**

Three real constraints the design needs to be explicit about:

**Constraint 1 — Chain-depth cap**: carry-forward chains must not
exceed N steps before a `full_review` resets the chain. After N
carry-forwards, the audit should demote to Reachability with hint
`discipline-carry-forward-chain-too-deep` and require `full_review`.
The chain depth IS capturable from the `Signer.basis` field history.
N = 3 is a defensible initial value; workspace-config adjustable.

**Constraint 2 — Cumulative diff tracking**: the basis record should
carry the fingerprint at the ORIGINAL `full_review`, not just the
immediately prior fingerprint. This lets the audit (or a human
reviewer) compare current code against alice's full-review fingerprint,
not just against the most recent carry-forward step. Without this,
the carry-forward chain is invisible in the sidecar; you can only see
the most recent step.

**Constraint 3 — Required attestation-level reviewers for
carry-forward**: `attest sign --carry-forward-from X` should require
an explicit `--rationale "..."` argument documenting why the diff
is compliance-preserving. The rationale is stored in the basis field.
CLI refuses if rationale is absent or fewer than N characters
(e.g., 20 chars minimum — prevents `--rationale "ok"`). This is
lightweight but creates an explicit record for future audit.

**Verdict on the attack**: LANDS. A diff-size cap and time-cap alone
do not prevent cumulative laundering via carry-forward chains. The
design needs chain-depth cap + cumulative fingerprint tracking +
required rationale to make carry-forward tier-honest rather than
an escape hatch.

**Refinement T2-R**: Schema `Signer.basis` must carry:
- `basis_kind`: `full_review | carry_forward | doc_version_carry_forward`
- `carry_forward_from_fingerprint`: the immediately prior fingerprint
- `original_full_review_fingerprint`: the fingerprint of the LAST
  `full_review` in the chain (not the carry-forward chain root —
  the most recent actual full review)
- `carry_forward_chain_depth`: an integer counting carry-forward steps
  since the last `full_review`
- `rationale`: required non-empty string when basis_kind is
  `carry_forward`

Audit should demote to Reachability with `discipline-carry-forward-chain-exceeded`
hint when `carry_forward_chain_depth` exceeds workspace-configured
cap (default 3). This is the missing ratchet that makes carry-forward
honest.

---

## Attack T3 — F8 (EvidenceKind overclaim): compound-evidence "stronger together" illusion

**The aristotle position (F8)**: `EvidenceKind` as first-class axis
parallel to `WitnessTier` and `AuditHint`. v0.1 ships three kinds
(`TypeSystemProof | Behavioral | SubstrateState`). Each kind has its
structural ceiling.

**Attack**: a discipline-antigen with BOTH a behavioral test witness AND
substrate-witness signatures. The audit reports:

```
witness_tier: Execution
audit_hint: discipline-substrate-validated-and-current
evidence_kind: SubstrateState    ← from substrate-witness pass
```

AND separately (for the same `#[immune]` declaration with both witnesses):

```
witness_tier: Execution
audit_hint: function-covered-by-test
evidence_kind: Behavioral    ← from behavioral witness pass
```

A downstream consumer sees two `Execution`-tier entries with different
`EvidenceKind` values. Does this produce an "additive confidence"
illusion? Specifically: does the consumer (a CI gate, a dashboard, a
safety-critical reviewer) interpret "Execution-Behavioral AND
Execution-SubstrateState" as STRONGER than "Execution-Behavioral
alone"?

**The structural problem**: the two witnesses are testing DIFFERENT
things. The behavioral test verifies runtime behavior; the substrate
signatures verify "alice says the discipline is followed." These are
complementary but not additive in the formal sense — you can't combine
them to get FormalProof (or "super-Execution"). But a naive UI that
shows "2 witnesses, both at Execution" might create the impression
of stronger evidence.

**Is this a new problem or inherited?**

The problem EXISTS without EvidenceKind — two behavioral test witnesses
at Execution tier already create the same question. But EvidenceKind
makes it WORSE because different kinds create the appearance of
independent verification from independent angles, which IS a real
probabilistic argument ("behavioral AND substrate-state, independent
failure modes") but only when the independence assumption holds.

**When does independence NOT hold?**

- Alice writes the behavioral test AND signs the substrate witness.
  Her judgment is one source; independence is zero. But the audit
  reports two witnesses from two independent kinds.
- The behavioral test tests the happy path; the substrate witness
  certifies the discipline. The test doesn't cover the edge case the
  discipline was written about. Two witnesses, neither catching the
  failure mode.
- The behavioral test is a unit test that passes always because the
  test is wrong. Alice signs the substrate. The antigen now has
  compound evidence supporting a false claim.

**What does the EvidenceKind axis ADD to this problem?**

Before EvidenceKind: two witnesses, both Execution. Consumer sees
"two witnesses at Execution; stronger than one."

After EvidenceKind: two witnesses, both Execution, different kinds.
Consumer sees "two witnesses at Execution from different evidence
kinds; multi-angle verification." The MULTI-ANGLE framing is the
additive-confidence illusion at its strongest.

**Does the single-axis `WitnessTier` prevent this?**

No — it was already present without EvidenceKind. EvidenceKind
makes it more visible (different kinds = "multi-angle") but doesn't
create the problem.

**What restriction makes compound-evidence tier-honest?**

Option A — Compound witnesses don't aggregate: each witness is
reported independently; no compound-witness "score" is derived.
The consumer sees the individual evidence entries; aggregation is
consumer responsibility. This is tier-honest but puts the burden
on every downstream consumer to not aggregate naively.

Option B — Evidence-kind independence check: when multiple witnesses
report for the same site, the audit outputs a `independence_caveat`
field noting which witnesses share author identity, which share
"domain of verification" (same scope of code coverage), which are
independent. This is underdetermined — the audit can't know whether
two witnesses are truly independent without semantic analysis. Too
hard.

Option C — Explicit warning on compound reporting: when the audit
reports multiple witnesses at Execution for the same site, it emits
a `compound_evidence_note: "multiple witnesses at Execution-tier for
this site; independence between witnesses is not verified by the
audit; downstream consumers should not treat compound evidence as
stronger than the strongest single witness unless they can establish
independence."` This is honest, low-cost, and doesn't require the
audit to solve the independence problem.

**Verdict on the attack**: PARTIAL LAND. EvidenceKind doesn't create
the additive-confidence illusion — it already existed. But EvidenceKind
does make the illusion stronger by framing compound evidence as
"multi-angle." The design needs to be explicit that compound evidence
is NOT treated as a tier-aggregating operation; the audit reports each
witness independently and consumers must reason about independence.

**Refinement T3-R**: Audit output for a site with multiple witnesses
must:
1. Report each witness as a separate entry (already implied; make
   explicit)
2. NOT derive a compound tier higher than the highest individual
   witness tier
3. When multiple witnesses share `EvidenceKind`, emit a note (the
   strongest hint already emitted is fine; no compound hint invented)
4. When multiple witnesses have DIFFERENT `EvidenceKind`, emit a
   `compound_evidence` boolean = `true` with documentation note
   "multiple evidence kinds present; independence not audited"

This is Option A + Option C combined. Cost: one boolean field on the
compound case. Tier-honesty is preserved; consumer has explicit signal
that they're seeing compound evidence and must reason about independence.

---

## Attack T4 — F4 (closed-set tool invocation): "cargo is a closed-set tool" argument

**The aristotle position (F4)**: the prohibition is specifically on
"witness-author-defined code." Leaves may invoke "closed-set ecosystem
tools" (git, cargo audit, clippy, kani) under the same tier-honesty
discipline. `signed_trailer` already invokes git.

**Attack**: this opens a door. What's the weakest justification a
leaf-author could use to claim a tool is "closed-set"?

Scenario 1 — A leaf author ships `cargo_build_passes` that invokes
`cargo build` with the full workspace. Justification: "cargo is a
known closed-set tool." Attack: `cargo build` is deterministic when
`Cargo.lock` is pinned, but:
- It executes `build.rs` scripts from all deps (arbitrary code)
- It can invoke proc-macros (arbitrary code generation)
- It can download crates if the cache is stale (network calls)
- Build time is unbounded if a dep has a compilation-intensive build
- `cargo build` success proves "code compiles," not compliance with
  the discipline the antigen represents

Scenario 2 — `cargo run --bin my_oracle` as a leaf: "cargo run is a
closed-set tool; running the oracle binary is an extension of that."
Attack: `my_oracle` is author-defined code. The `cargo run` wrapper
doesn't make author-defined code closed-set. This is the trust-the-
witness problem with one extra indirection.

Scenario 3 — `curl https://api.mycompany.com/discipline-check` as a
leaf: "curl is a closed-set tool (part of every CI environment);
calling our internal API is an extension of that." Attack: curl IS
a known tool, but the payload it fetches and the server it calls are
author-defined. The "tool is closed-set" claim doesn't transfer to
the arguments it receives.

**What is the actual boundary that holds vs collapses under
contributor pressure?**

The aristotle framing (F4 Phase 3 Strip A) identified three categories:
1. Author-defined code: prohibited
2. Closed-set ecosystem tools: permitted
3. Closed-set audit primitives: permitted

The attack shows category 2 collapses without a sharper definition.
"Closed-set ecosystem tool" needs to mean:
- **Fixed binary from a known upstream project**: git, cargo-audit,
  clippy — these have their own version discipline, release processes,
  and audit trails
- **Fixed interface invoked by the leaf**: the leaf calls `git log
  --pretty=format:%s %H` — fixed args; the tool's output is deterministic
  for fixed input
- **No author-defined arguments or payloads beyond leaf parameters**:
  the leaf may parameterize which key to look for in `git interpret-trailers`;
  it may not parameterize which binary to invoke or fetch code from a URL

The collapsing scenarios all violate point 3: `cargo build` executes
build scripts (author-defined arguments hidden in Cargo.toml);
`cargo run --bin my_oracle` passes author-defined binary; `curl URL`
fetches author-defined payload.

**A bright-line rule**: a leaf invokes a closed-set tool if and only if
all of these hold:
1. The binary is named in the leaf's source code (not taken as a
   parameter from the use-site or the leaf-provider's configuration)
2. The binary has its own release process and version pinning in the
   leaf-provider's Cargo.toml or workspace toolchain spec
3. The binary does NOT execute user-supplied code (build scripts,
   proc-macros, eval, dynamic dispatch to user-defined binaries)
4. The binary's invocation arguments are fixed in the leaf's source
   code except for the leaf's declared substrate-parameters

Under this rule:
- `git interpret-trailers --parse` with fixed args: CLOSED-SET (passes all 4)
- `cargo audit --json`: CLOSED-SET (cargo audit doesn't execute user code)
- `cargo build`: NOT CLOSED-SET (fails rule 3 — executes build scripts)
- `cargo run --bin my_oracle`: NOT CLOSED-SET (fails rules 3+4)
- `curl URL`: NOT CLOSED-SET (fails rule 3 — fetches user-defined payload)
- `clippy::lint`: CLOSED-SET (clippy has its own release; lint checks are
  in clippy's source; user-defined lints are through `declare_lint!` which
  is a separate concern)

**Verdict on the attack**: LANDS. F4's "closed-set ecosystem tool"
framing is correct in principle but would collapse in practice without
a bright-line rule that prevents `cargo build`, `cargo run`, and
`curl` from being claimed as closed-set tools.

**Refinement T4-R**: The leaf-contract specification (v0.2+ ADR)
must include the four-point bright-line rule above. Additionally: the
ADR-019 predicate-language ceiling section must be revised from
"closed-set ecosystem tool invocation permitted" to "closed-set tool
invocation permitted under [four-point rule]" with explicit
non-examples (cargo build, cargo run, curl with external URLs) to
prevent contributor-pressure erosion of the boundary.

---

## Attack T5 — F1 (discipline-vs-machinery unification drift): where's the guardrail?

**The aristotle position (F1)**: discipline-level unification (both
substrate-witnesses and cross-crate witnesses follow the same
tier-honesty discipline) is real and should be named. Machinery-level
unification (shared parser code) is a category error and must not happen.
ADR-019 should cite this asymmetry explicitly.

**Attack**: what's the failure mode if a future maintainer mistakes
discipline-level unification for machinery-level unification and tries
to share parser code?

Concrete failure path: a new maintainer reads "substrate-witnesses and
cross-crate witnesses are both 'predicate evaluation over substrate not
this code'." This framing (which F1 endorses at the discipline level)
could motivate refactoring the audit module to:

1. Unify the JSON-sidecar reader and the Rust-source AST walker behind
   a `SubstrateReader` trait.
2. Parameterize the tier-honesty machinery over `SubstrateReader`
   implementations.
3. Share the tier-emission helpers.

Steps 1-2 are the machinery-level unification error. Step 3 is fine.

**What breaks concretely?**

- JSON-sidecar reader is synchronous, file-local, schema-typed via serde.
  A `SubstrateReader` trait would need to accept `serde_json::Value` or
  a dynamic type.
- Rust-source AST walker is asynchronous (walking potentially many files),
  type-agnostic at the reader level (syn parses tokens, not typed structs),
  and coupled to the fingerprint-resolution pipeline.
- Unifying them behind a trait would either (a) force the JSON path to
  accept dynamic values (losing compile-time schema validation — tier-
  honesty surface degrades) or (b) force the Rust-source path to serialize
  through serde_json (losing syn's static typing — audit precision degrades).

The failure is SILENT: a `SubstrateReader`-unified audit might pass all
existing tests because the tests don't exercise the precision that was
lost in the unification. The audit would compile, report tiers, and
produce plausible output — but with degraded schema validation on the
sidecar path or degraded AST precision on the Rust-source path.

**What documentation prevents this drift?**

Currently: nothing explicit. ADR-019 would say "discipline-level
unification; machinery is parallel" but the reasoning why is buried in
F1's Phase 1-8 stripping (which a new maintainer would not read unless
they were explicitly pointed to it).

**The minimum viable guardrail**: a comment block in the audit module,
adjacent to the tier-emission helpers, that reads:

```rust
// MAINTAINER NOTE: Substrate-witness (sidecar-reading) and cross-crate
// witness (Rust-source-reading) share TIER-HONESTY DISCIPLINE (this
// tier-emission module) but NOT RECOGNITION MACHINERY. The sidecar path
// uses serde_json + schema validation; the cross-crate path uses syn +
// AST walking. These are NOT candidates for unification behind a shared
// trait — unification would sacrifice compile-time schema correctness
// or AST precision respectively. See ADR-019 §"Discipline vs machinery
// unification" for the reasoning.
```

Plus a test in `adversarial_tests.rs` that explicitly exercises sidecar
schema validation (not just successful parsing) — a test that passes
malformed sidecar JSON and asserts the audit reports `discipline-sidecar-
schema-invalid` hint, not a generic parse error. This makes the schema-
validation precision visible in the test suite and prevents the "just
make it compile" refactor from silently removing it.

**Verdict on the attack**: LANDS (as a documentation/governance gap,
not a current code defect). There is no current guardrail preventing a
future maintainer from attempting machinery-level unification motivated
by F1's discipline-level-unification claim. The failure mode is a
silent precision-degradation in schema validation or AST typing that
passes all existing tests.

**Refinement T5-R**: Two items required:
1. In-code comment block in the audit module's tier-emission section
   explicitly forbidding machinery-level unification with reasoning
   and ADR-019 cite.
2. An adversarial test that exercises sidecar schema validation
   precision specifically (malformed JSON → specific schema-invalid
   hint, not generic error). This makes the precision observable
   in CI and prevents silent regression.

---

## Attack T6 — Tier-overclaim worst-case in v0.1

**What this attacks**: not a specific F-finding, but the v0.1
tier-honesty surface overall. What's the worst realistic tier-overclaim
scenario in v0.1?

**Setup**: v0.1 ships with sealed leaf set, git-trust signing,
no crypto-signing, and the `discipline-substrate-validated-and-current`
→ Execution tier mapping.

**Worst-case scenario**: Alice creates a sidecar for `compute_bounds`
with `SignedZeroDiscipline`. She sets:
- `signers`: `[{name: "alice", role: "math-researcher", date: "2026-05-19", signed_against_fingerprint: <current fingerprint>}]`
- The predicate requires `signers(required = ["alice"], against = "current")`
- She invokes `cargo antigen attest sign` using her git config identity

The audit reports: `WitnessTier::Execution`, hint `discipline-substrate-validated-and-current`.

**What has actually been verified?**

1. A JSON file exists with alice's name and a date.
2. The name in the JSON matches alice's git config user.name.
3. The `signed_against_fingerprint` in the JSON matches the current
   function fingerprint.

**What has NOT been verified?**

1. Whether alice actually reviewed the function against the discipline.
2. Whether alice is the "alice" who is qualified to make this attestation.
3. Whether alice's git config identity matches her actual identity
   (trivially bypassable per self-pass Attack 4 / R-A5).
4. Whether the discipline doc (`SignedZeroDiscipline.discipline_doc`)
   actually specifies what alice claims to have verified against.
5. Whether the predicate specifying `required = ["alice"]` is the
   correct predicate for this antigen (could be a stricter requirement
   in the real discipline).
6. Whether the oracle files are complete (if `oracles_complete` is not
   in the predicate, omitting it is silent).

The audit reports Execution-tier with `discipline-substrate-validated-and-current`.
A CI gate requiring Execution-tier passes. A dashboard shows green.

**Is this tier-overclaim?**

By the v2 framing ("every tier verifies structure-of-evidence, not truth;
truth is developer's responsibility"), this is honest: the audit verified
the structure of evidence (predicate passes, fingerprint current). The
audit doesn't claim to have verified alice's judgment.

BUT: the `discipline-substrate-validated-and-current` hint carries
"validated-and-current" language that could be interpreted as "the
discipline is verified." A consumer reading this hint might infer
stronger verification than actually occurred.

**Verdict**: the tier assignment is defensible under the "structure-of-
evidence" framing, but the audit-hint name is too confident. The hint
should clarify that what was validated is the PREDICATE AGAINST THE
SIDECAR, not the discipline itself.

**Refinement T6-R**: Rename `discipline-substrate-validated-and-current`
to `discipline-predicate-passed-substrate-current`. The new name is
accurate: the predicate passed, and the substrate is current. It doesn't
imply the discipline was independently verified. Additionally, audit output
should include the predicate expression that passed, so consumers can see
WHAT was evaluated, not just that it passed.

---

## Attack T7 — Predicate-language closure: valid-but-nonsensical predicates

**What this attacks**: the closed combinator grammar with sealed leaf
primitives (R5 / v2 §"Predicate-language ceiling"). Can someone construct
a "valid" predicate that is semantically nonsensical?

**Attack 1 — Tautological always-pass**:
```rust
any_of([
    ratified_doc(min_version = "0.1"),      // easy to satisfy
    signers(required = []),                 // schema-rejected per R-A6
])
```
R-A6 (schema rejects zero-leaf compositions) catches `signers(required = [])`.
But the tautological structure survives: if the antigen has a discipline doc
with min_version = "0.1" and the doc exists at all, this predicate always
passes. A single-leaf predicate `ratified_doc(min_version = "0.1")` is
valid, semantically reasonable, and says essentially "the discipline doc
exists in some form." An adversary uses this as the ENTIRE predicate for
a high-severity discipline antigen — bypassing signer requirements and
oracle checks.

**Does the schema prevent this?** NO. A predicate with one leaf is valid.

**Should the schema prevent this?** No — because the antigen author gets
to decide what the discipline requires. A real discipline might genuinely
require only "the discipline doc exists." The predicate language is a
mechanism, not a policy. The antigen declaration's `discipline_doc` field
and the witness predicate together constitute the discipline's requirements;
the schema validates structure, not adequacy.

BUT: the audit should report WHICH leaves were in the predicate and what
each evaluated to — so a reviewer can see "this antigen was attested with
only `ratified_doc`, no signers, no oracles." This is already in load-
bearing item 2 (witness expressions need parse + serialize + replay with
per-leaf output). The predicate transparency is the defense, not schema
rejection.

**Attack 2 — Contradictory always-fail**:
```rust
all_of([
    signers(required = ["alice"]),
    not(signers(required = ["alice"])),
])
```
This is structurally valid combinator grammar. Semantically: Alice must
sign AND Alice must not sign. Always evaluates to `false`. The audit
reports `discipline-predicate-failed` with per-leaf output showing both
leaves' evaluation.

**Is this a problem?** It prevents the code from ever being attested.
It's a user error (probably a copy-paste mistake or a `not()` applied
to the wrong subtree). The schema cannot reject it at parse time because
both leaves are valid.

**The failure is detectable but not preventable at the schema layer.** The
audit correctly reports failure. But the developer might not understand
WHY it fails without careful reading of the per-leaf output. A lint check
(`cargo antigen attest lint`) that detects contradictory predicates would
help — but this is a tooling enhancement, not a predicate-language defect.

**Attack 3 — Fresh-within-days making the predicate time-dependent**:
```rust
all_of([
    signers(required = ["alice"]),
    fresh_within_days(1),
])
```
This predicate requires re-attestation daily. Alice signs once; next day,
the audit reports `discipline-substrate-stale` because freshness expired.
This is intended behavior — some disciplines genuinely require frequent
re-attestation. But:

1. CI runs overnight: green Monday morning, red Monday night if alice's
   signature was on Sunday.
2. A well-intentioned team member increases `fresh_within_days` to 365
   to stop CI failing — effectively disabling freshness enforcement.
3. There's no workspace-minimum for `fresh_within_days` — a value of
   36500 (100 years) is valid and defeats the freshness primitive entirely.

**Refinement T7-R**: Predicate-language linting command
(`cargo antigen attest lint`) should check:
- `fresh_within_days(N)` where N > workspace-configured max (default 365):
  warn "freshness period exceeds workspace maximum; effective enforcement
  may be weakened"
- `all_of([X, not(X)])` patterns (contradictory predicates): warn
  "predicate is always-false; attestation cannot succeed"
- No `signers(...)` or `oracles_complete(...)` leaves at all (predicate
  relies only on doc existence): info "this predicate verifies doc
  existence only; no human sign-off required"

These are lint-level warnings, not schema rejections. The schema stays
closed; the lint provides semantic guidance.

---

## Attack T8 — `.attest/` folder convention: collision and cargo-clean risks

**What this attacks**: the v2 §"Location convention" establishing
`.attest/` subfolder adjacent to source files.

**Collision risk**: does any existing Rust project convention use `.attest/`?

`.cargo/`, `target/`, `.git/`, `.github/`, `.vscode/`, `.idea/` are
established conventions. `.attest/` is NOT an established Rust convention
(confirmed by absence in Cargo documentation and cargo source conventions).
However:

1. The dot-prefix makes it a hidden directory on Unix, which is reasonable
   for metadata folders. But on Windows it is NOT hidden by default (requires
   `FILE_ATTRIBUTE_HIDDEN` system attribute). A developer on Windows would
   see `src/numerics.attest/` as a visible directory next to `src/numerics.rs`,
   which might be confusing.

2. **The `cargo test` convention**: Cargo creates a `target/` directory but
   doesn't touch source files' adjacent directories. `.attest/` directories
   adjacent to source files would be treated as source by most Rust tools:
   - `cargo fmt` will not enter them (correct)
   - `cargo doc` will not process them (correct)
   - `rustfmt` will not process them (correct)
   - BUT: IDEs with "exclude hidden directories from search" settings vary
   - **rustanalyzer**: rust-analyzer's file watching might trigger on changes
     to `.attest/` JSON files, causing spurious re-analysis. Low severity but
     real noise.

3. **`cargo clean` behavior**: `cargo clean` removes the `target/` directory.
   It does NOT remove `.attest/` folders (they're in source, not build output).
   This is CORRECT behavior — sidecars are source, not build artifacts. No
   issue here.

4. **`git clean -fd` risk**: `git clean -fd` removes untracked files. Sidecars
   NOT committed to git would be deleted by `git clean -fd`. This is a real
   adoption risk: a developer creates a sidecar with `attest scaffold`, doesn't
   commit it immediately, runs `git clean -fd` to tidy up, and loses the sidecar
   permanently. The sidecar content is irreplaceable if it included oracle markers
   or signer entries.

**Verdict on collision**: `.attest/` is safe from Rust-ecosystem collisions.
The Windows visibility issue is cosmetic. The `git clean -fd` risk is real and
operationally significant.

**Refinement T8-R**:
1. CLI `attest scaffold` should immediately `git add` the created sidecar
   (or print a prominent warning: "IMPORTANT: commit this sidecar before running
   `git clean -fd` or it will be lost"). The warning approach preserves user
   control; the auto-add approach is less surprising for the common case.
2. ADR-019 should specify that `.attest/` folders and their contents are
   version-controlled source artifacts, not generated build output. This
   establishes the ownership model explicitly.
3. Consider whether `.attest/` folders should be added to `.gitattributes`
   with `linguist-generated=false` (or similar) to prevent GitHub from
   collapsing them as "generated code" in diffs — sidecars are human-authored
   and should be visible in PR review.

---

## Attack T9 — CLI workflow gap: reviewer is not committer

**What this attacks**: `cargo antigen attest sign` uses `git config
user.name` to identify the signer. The assumption is that the person
running the CLI command is the person attesting compliance.

**Attack**: standard PR workflow where the reviewer (alice) reviews
bob's code and approves. Bob commits the merge. Alice cannot
run `attest sign` because:
- She doesn't have a local checkout with the exact branch state
- She cannot commit to the repo (only bob can, or it requires a merge
  commit flow)
- If alice signs on her machine and sends the JSON to bob, bob can
  modify alice's entry before committing (there's no signature to protect it)

**The `--on-behalf-of` risk**: a natural CLI extension would be
`cargo antigen attest sign --on-behalf-of alice --role math-researcher`.
This reintroduces the bypass risk: bob can now sign AS alice without
alice's knowledge. Git-trust relies on `git config user.name` to bind
identity; `--on-behalf-of` removes the binding entirely.

**Is there a tier-honest solution?**

Option A — GitHub-native approval as a leaf: a `github_pr_approved_by`
leaf checks the PR's approval state via GitHub API. "Alice approved PR
#123 which included this file." This is forge-API integration (deferred
to v0.2+ per R1). But it's the correct structural answer for the
reviewer-not-committer workflow.

Option B — `signed_trailer` leaf as proxy: alice adds a Git trailer
`Discipline-Verified-By: alice <alice@example.com> SignedZeroDiscipline`
to her review comment or commit message (if she's a co-author). Bob
includes this in the commit. `signed_trailer` leaf evaluates it. This
is the v0.1 workaround — it uses git's existing identity mechanism,
requires no new CLI, and is detectable by the audit. But: (i) alice
must know to add the trailer format; (ii) trailers are in commit
messages, not `.attest/` sidecars, so the audit must check BOTH
places.

Option C — Wait for crypto-signing (v0.4+): alice signs the sidecar
cryptographically on her machine; sends the signed entry (not the
raw JSON) to bob; bob includes it in the commit. The signature proves
alice's private key was used, even if bob committed the file. This
is the structurally sound long-term answer but requires the
`Signer.signature` slot to be activated.

**Verdict on the attack**: LANDS as a v0.1 adoption-blocker for teams
with standard PR-review-then-merge workflows. The self-pass Attack 6
(which isn't in the self-attack file but was flagged in the
self-attack frontier list, item 6) was exactly this concern.

**Refinement T9-R**: v0.1 should document the reviewer-not-committer
workflow explicitly with the Option B workaround (`signed_trailer`
as the v0.1 mechanism for reviewer attestation). ADR-019 should
acknowledge this as a known limitation of git-trust default and
name v0.2+ forge-API integration (Option A) and v0.4+ crypto-signing
(Option C) as the progression. The CLI `attest sign` documentation
should say explicitly: "If the attester cannot run this command
locally, use `signed_trailer` leaf with a git commit trailer, or
wait for v0.4+ crypto-signing support."

---

## Attack T10 — R-A7 (cross-crate per-consumer ratification): worst-case for adopters

**The position (R-A7)**: cross-crate discipline-antigens require
per-consumer ratification. Sidecar lives in consuming crate; predicate
evaluates against consuming crate's substrate. Discipline judgment
doesn't transfer.

**Attack**: what's the worst-case for adopters?

Scenario: Crate A declares `SignedZeroDiscipline` with a predicate that
requires `signers(required = ["alice", "bob"], against = "current")` AND
`oracles_complete(["docs/oracles/signed-zero-proofs.md"])`. The oracle
doc lives in Crate A's repo.

Crate B imports Crate A as a dependency and uses `#[descended_from = "A::SignedZeroDiscipline"]`
on their implementation. Per R-A7, Crate B must provide their OWN sidecar
at `crateB/src/foo.attest/SignedZeroDiscipline.json`.

**Worst-case dimensions**:

1. **Predicate portability**: Crate B's predicate might specify
   `signers(required = ["carol", "dave"])` and
   `oracles_complete(["docs/crateB-signed-zero-proofs.md"])`. These are
   different people and different oracle files from Crate A's predicate.
   That's fine — per-consumer ratification means per-consumer discipline.
   But it also means: Crate A's discipline doc says "must be ratified by
   two math-researchers"; Crate B's signers are carol and dave who might
   not be math-researchers; there's no enforcement that Crate B's signers
   satisfy Crate A's signer-role requirements.

2. **Role-qualification gap**: Crate A's `discipline_doc` says "requires
   two people with formal numerical-methods training." Crate B's sidecar
   names `carol` (a junior developer) and `dave` (a product manager).
   The audit reports Execution-tier because the predicate passed (carol
   and dave signed against current fingerprint). The discipline's INTENT
   is violated; the predicate's STRUCTURE is satisfied.

   This is not specific to cross-crate — it exists for single-crate
   cases too (alice could be a PM signing a math discipline). But it's
   WORSE in cross-crate because Crate B's consumers may not know Crate A's
   discipline intent.

3. **Oracle portability**: Crate B's `oracles_complete` references
   `docs/crateB-signed-zero-proofs.md`. Crate B must create and complete
   this oracle. If Crate A's oracle is a 40-page mathematical proof,
   Crate B might create a 2-sentence oracle that says "we checked; it's
   fine." The oracle-completeness check passes; the depth of the oracle
   is invisible to the audit.

4. **Scale of burden**: if Crate A declares 20 discipline antigens, and
   Crate B imports Crate A for a small function, Crate B must ratify all
   20 antigens they've `#[descended_from]`-ed. This could be prohibitive
   for adoption of crates with rich discipline antigen taxonomies.

**Does per-consumer ratification SOLVE the trust problem?**

Structurally: YES, in the sense that Crate B's discipline compliance
is explicitly stated and auditable. NO, in the sense that Crate B
can trivially satisfy the structural requirements (predicate passes)
without satisfying the discipline's INTENT (qualified reviewers,
substantive oracles).

**The biological rhyme (R-A7)**: passive immunity (maternal IgG) doesn't
verify what the infant encountered; active immunity (Crate B's own
ratification) doesn't verify that the encounter was substantive. The
biology rhyme holds for the mechanism but not for the quality of the
immune response.

**Verdict on the attack**: PARTIAL LAND. R-A7 is structurally correct
(per-consumer ratification is the right default). But three gaps remain:
1. Signer-role requirements from the declaring crate are not enforced
   on the consuming crate's signers.
2. Oracle depth is invisible to the audit.
3. Adoption burden could be prohibitive for heavy discipline-antigen
   hierarchies.

**Refinement T10-R**:
1. The `discipline_doc` field (or the antigen declaration's metadata)
   should optionally carry `required_signer_roles: Vec<String>` (e.g.,
   `["numerical-methods-researcher"]`) that consuming crates must
   satisfy in their signers' `role` fields. The predicate evaluator
   checks this at audit time. This lets declaring crates set
   MINIMUM ROLE REQUIREMENTS that survive per-consumer ratification.
2. Oracle depth is inherently a human-judgment problem; the audit
   cannot solve it. Document explicitly that oracle content quality
   is developer responsibility, not audit responsibility.
3. For v0.1, document the adoption burden explicitly: "Crates that
   import heavy discipline-antigen taxonomies should expect per-antigen
   ratification cost. Workspace-default inheritance (deferred to v0.2)
   will reduce this burden for well-aligned codebases."

---

## Summary: Attacks that LAND, refinements for v3

Listed in priority order by severity to v0.1 correctness / adoption:

**T2-R (carry-forward laundering) — HIGH PRIORITY**: schema needs
chain-depth cap + cumulative fingerprint tracking + required rationale.
Without these, `--carry-forward-from` is an escape hatch, not a
tier-honest carry-forward mechanism.

**T6-R (tier-overclaim hint name) — HIGH PRIORITY**: rename
`discipline-substrate-validated-and-current` to
`discipline-predicate-passed-substrate-current`. Current name implies
the discipline was independently verified when only the predicate's
structure was verified.

**T1-R (leaf-provider trust) — HIGH PRIORITY FOR V0.2+**: ADR-019's
deferral note must name runtime enforcement requirement, not just
contract specification. The v0.2+ leaf-provider ADR is on the critical
path for tier-honesty at the leaf level; documentation-trust alone
is not sufficient.

**T4-R (closed-set tool boundary) — MEDIUM PRIORITY**: four-point
bright-line rule for "closed-set tool" in the predicate-language ceiling
section. Without it, contributor pressure erodes the boundary via
"cargo is a closed-set tool" arguments.

**T9-R (reviewer-not-committer workflow) — MEDIUM PRIORITY**: document
the v0.1 limitation explicitly; name Option B (`signed_trailer`)
as the workaround; name v0.2+ and v0.4+ progression paths.

**T8-R (`git clean -fd` risk) — MEDIUM PRIORITY**: `attest scaffold`
warning + `.gitattributes` guidance + ownership model documentation.

**T5-R (discipline-vs-machinery unification guardrail) — MEDIUM PRIORITY**:
in-code comment block + adversarial schema-validation precision test.

**T3-R (compound-evidence EvidenceKind) — LOW PRIORITY**: `compound_evidence`
boolean field on audit output for sites with multiple witnesses. Prevents
"additive confidence" illusion without requiring consumers to reason
about independence.

**T7-R (predicate-language lint) — LOW PRIORITY**: `cargo antigen attest
lint` command for semantic nonsense detection (always-false predicates,
excessive freshness periods, signerless predicates on high-severity antigens).

**T10-R (cross-crate signer-role portability) — LOW PRIORITY FOR V0.1,
MEDIUM FOR V0.2**: optional `required_signer_roles` field on antigen
declarations; minimum role requirements survive per-consumer ratification.

---

## Attacks that DO NOT land (the design holds)

**T6 core tier-honesty framing**: "every tier verifies structure-of-
evidence, not truth" is structurally correct. The attack on the tier
assignment itself doesn't land — only the hint name lands.

**T7 predicate-language closure as Turing tarpit**: the closed combinator
grammar with sealed leaf set is correct. The tautological-always-pass
and always-fail scenarios are user errors, not design defects. Predicate
transparency (per-leaf output in audit report) is the defense.

**T8 collision risk**: `.attest/` is safe from Rust-ecosystem collisions.
The `cargo clean` concern doesn't apply (sidecars are source, not build
output). The `git clean -fd` concern is real but a documentation/UX
issue, not a design flaw.

**T3 EvidenceKind creating the compound-evidence illusion**: the illusion
predated EvidenceKind; EvidenceKind makes it more visible (which is actually
good). The refinement (compound_evidence boolean) is a consumer-guidance
improvement, not a design correction.

---

## Frontier attacks for the next adversarial pass

These surfaces haven't been attacked yet and represent the next adversarial
frontier after v3 lands:

**FA-1 — Schema migration with carry-forward chains**: v1→v2 migration
plus carry-forward chains. If `Signer.basis` is a new v2 field, sidecars
with v1 `Signer` records have no `basis` field. Does the migration tool
set `basis_kind = full_review` (upgrading the claim) or `basis_kind =
legacy_unknown` (downgrading the claim)? The former is tier-overclaim;
the latter creates unnecessary noise.

**FA-2 — Fingerprint-scope mismatch between fingerprint versions**:
`antigen_fingerprint` will evolve across antigen versions. What happens
when a sidecar contains `signed_against_fingerprint` computed by
antigen v0.1's fingerprinting scheme, and the audit runs antigen v0.2
with a refined fingerprinting scheme? The comparison will always
show "stale" even for code that hasn't changed. This is the `cargo lock
format changed` problem applied to attestation sidecars.

**FA-3 — Scope-field interaction with carry-forward**:
F3 adds `scope:` field (site | file | module | crate | workspace).
Carry-forward (`--carry-forward-from X`) is defined in terms of
item fingerprint. For `scope: file`, the fingerprint is a hash of all
items in the file. A carry-forward across a file-scope sidecar pins
the file-hash. If a different item in the same file changes (but NOT
the item the sidecar covers), the file-hash changes and all items in
the sidecar become stale. This forces re-attestation of items whose
code didn't change, which is the wrong behavior.

**FA-4 — `attest gc` race condition in multi-developer workflows**:
One developer runs `attest gc` (removes orphaned entries); another
developer is mid-way through adding `#[immune]` to a site but hasn't
yet created the sidecar. `gc` won't touch their in-progress work (no
sidecar exists yet for the new site). But if the first developer's `gc`
runs BETWEEN "add the macro to the code" and "create the sidecar," and
a CI audit runs at that moment, it reports `None` tier for the new site.
This is fine — the in-progress state is correctly None-tier. But the
developer might panic seeing CI red. Audit output should clearly
distinguish "antigen presented with no sidecar" (new work in progress)
from "antigen presented with orphaned sidecar" (stale work).

**FA-5 — `descended_from` propagation with substrate-witness predicates**:
Crate A declares `SignedZeroDiscipline` with a specific predicate.
Crate B uses `#[descended_from = "A::SignedZeroDiscipline"]`. Does Crate B
inherit the predicate from Crate A's declaration, or does Crate B write
its own predicate? R-A7 says per-consumer ratification, which implies
Crate B writes its own predicate. But `descended_from` carries connotation
of "derived from, includes the obligations of." If Crate B's predicate
is WEAKER than Crate A's (no oracle requirement, fewer signers),
`descended_from` is semantically misleading. Should `descended_from`
require at least as-strong a predicate as the parent, or is weakening
explicitly allowed?

**FA-6 — EvidenceKind ordering and consumer gate monotonicity**:
F8 ships `EvidenceKind = TypeSystemProof | Behavioral | SubstrateState`.
Is there an implicit ordering between these kinds? If a consumer wants
"at least Behavioral evidence," does `TypeSystemProof` satisfy the gate?
Is `TypeSystemProof` "better than" `Behavioral` or just different? The
`WitnessTier` axis is explicitly ordered (Reachability < Execution <
FormalProof). `EvidenceKind` was framed as "parallel axis" not as an
ordered scale. But without an ordering, a CI gate cannot write "require
at least Behavioral or better" in a meaningful way.

---

READY FOR REVIEW
