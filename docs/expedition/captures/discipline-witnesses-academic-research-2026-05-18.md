# Capture — Academic Research Pass on Discipline-Witnesses

> **Date**: 2026-05-18
> **Author**: single-instance Claude operating in academic-researcher mode,
> against discipline-witnesses-v2.md after the v2 draft and the
> aristotle/naturalist/adversarial self-passes landed.
> **Status**: append-only capture, written incrementally as research
> proceeded. The file IS the artifact.
> **Scope**: maps the existing landscape of attestation / predicate /
> provenance / policy systems that have framed problems structurally
> similar to substrate-witnesses, so the team launch enters with the
> design space illuminated rather than groping for prior art mid-pass.
> Antigen will make its own decisions; this capture's job is to make the
> decision space visible.

> **What this capture is NOT**: a recommendation for antigen's design. It
> deliberately surfaces what other systems have done and what trade-offs
> they made. Where antigen's posture (tier-honesty, code-locality,
> compose-don't-compete, anti-YAGNI, recognition-not-design) suggests a
> different choice, that's named — but as differentiation, not refutation.
> Many of the systems below are operating in different problem spaces with
> different constraints. Plurality is the point.

> **Methodology note**: WebSearch + WebFetch on primary sources (specs,
> RFCs, project READMEs) preferred over secondary commentary. Where claims
> rest on a single source or I couldn't locate authoritative substrate, it
> says so explicitly. The capture flags places where I made structural
> inferences vs cited substrate.

---

## Table of contents

1. **in-toto** — software supply chain attestation framework
2. **SLSA** — Supply-chain Levels for Software Artifacts
3. **Sigstore** — keyless signing for software (Fulcio / Rekor / cosign)
4. **TUF** — The Update Framework (role-based trust, threshold signatures)
5. **DSSE** — Dead Simple Signing Envelope
6. **PASETO** — Platform-Agnostic Security Tokens
7. **cargo-deny** — Rust dependency policy enforcement
8. **cargo-vet** — supply chain audits for Rust crates
9. **Open Policy Agent / Rego** — predicate language design
10. **Salsa** — incremental computation provenance
11. **GitHub CODEOWNERS / branch-protection / required-reviewers**
12. **Dependabot / Renovate / OSV-Scanner advisory ratification**
13. **npm/Yarn audit dismissals + GitHub Security Advisories suppression**
14. **GUAC** — graph of supply-chain artifacts and claims
15. **Synthesis** — what antigen can absorb, what antigen should
    differentiate, what's genuinely novel about substrate-witnesses

---

## 1. in-toto — software supply chain attestation framework

**Primary sources**: [in-toto/attestation spec README](https://github.com/in-toto/attestation/blob/main/spec/README.md), [Statement format](https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md), [in-toto specification (layouts)](https://github.com/in-toto/docs/blob/master/in-toto-spec.md), [Predicate spec](https://github.com/in-toto/attestation/blob/main/spec/v1/predicate.md), [SCAI / arxiv 2210.05813](https://arxiv.org/abs/2210.05813).

### Analogous primitive

The closest in-toto cognate of an antigen substrate-witness is **Statement + Predicate carried in a DSSE Envelope, verified against a Layout**.

- **Statement** (the carrier): `{_type, subject[], predicateType, predicate}`. The Statement binds the attestation to one or more subject artifacts matched purely by content digest. Roughly the in-toto equivalent of antigen's `Ratification` schema instance — a typed JSON record about some thing.
- **Predicate** (the payload schema): identified by `predicateType` URI (`https://slsa.dev/provenance/v1`, `https://in-toto.io/Link/v1`, `https://in-toto.io/attestation/scai/attribute-report/v0.2`, etc.). The predicate is the schema-discriminator. The framework explicitly invites new predicate types: "Users are expected to choose an existing predicate type that fits their needs, or develop a new one if no existing one satisfies." Antigen's substrate-witness leaf-primitive enum is the closed-grammar analog; in-toto's predicate-type registry is the open-grammar analog.
- **Envelope** (DSSE — see §5): authenticates Statement origin.
- **Layout** (the predicate language at the verification layer): a signed document that says which steps must occur, who (which functionary public keys) must sign each step's link metadata, and what artifacts flow between steps.

### Trade-offs in-toto made

**Open predicate-type grammar, closed layout-rule grammar.** This split is interesting and worth foregrounding for antigen:

- Predicate **types** are open — anyone can define a new `predicateType` URI with arbitrary JSON schema; verifiers must understand the predicate to validate its content. The spec is explicit: "the semantics of predicate interpretation are up to the producer and consumer."
- Layout **rules** are closed: `MATCH`, `ALLOW`, `DISALLOW`, `REQUIRE`, `CREATE`, `DELETE`, `MODIFY`, with glob-pattern matching and explicit firewall-style sequential evaluation. From in-toto-spec.md: "Artifact rules reside in the `expected_products` and `expected_materials` fields of a step and are applied sequentially on a queue of materials or products... If an artifact is successfully consumed by a rule, it is removed from the queue and cannot be consumed by subsequent rules." Implicit `ALLOW *` at the end; the spec recommends explicit `DISALLOW *` as the closing entry.
- **Sublayouts** compose: a step's verification can be deferred to a nested layout, recursively. Lets a top-level layout federate trust to sub-trees with their own functionaries.

**Distributed trust, signed metadata, end-to-end verification.** in-toto's purpose is verifying the supply chain end-to-end from the client side — verifier reads the layout, walks the link metadata, verifies signatures, walks artifact rules, runs inspections.

**Functionaries are public keys, not human identities.** Trust at the key layer. Identity is whoever holds the key; the layout names key fingerprints, not roles in the org-chart sense.

### What antigen could learn

1. **Open predicate-type registry pattern is a real escape valve.** in-toto designed for "we won't know all the predicate shapes our ecosystem needs," and the answer was URI-discriminated types. Antigen's v2 draft makes the opposite call (closed combinator grammar, R5 "Predicate-language ceiling"). The in-toto experience is the canonical example of open with all of openness's costs: every verifier needs to ship code for every predicate type it might encounter; unknown predicates either fail-closed (breaks adoption) or pass-through-unverified (silent tier-honesty violation). Antigen avoids both failure modes by closing the grammar — at the cost of expressiveness that the v2 draft argues is correctly fenced off into `test_fn` witnesses when needed.
2. **The Statement / Predicate / Envelope split is genuinely useful.** Even when the consumer doesn't care about the predicate content, the Statement-level metadata (subject digests, predicate type) lets generic tooling do *something* — index, route, count, alert on missing types. Antigen's `Ratification` already separates `antigen` identifier + `source_file` + `items[]` (Statement-ish) from per-leaf content (Predicate-ish). The split is implicit; could be made explicit if the design wanted generic substrate-witness tooling (queries across all sidecars regardless of predicate content).
3. **Artifact-rule grammar has primitives v2's combinator set lacks.** `MATCH ... FROM step X` is a *binding* primitive — it says "the artifact at this position in this step must be the same artifact at that position in that other step." Antigen has nothing analogous for cross-antigen, cross-file binding relationships. Probably out of scope for v0.1; worth knowing the shape exists if antigen ever needs to express "if antigen A is satisfied at file X, antigen B must also be satisfied at file Y."
4. **Inspection vs verification as two distinct passes.** in-toto runs inspections (client-side artifact unpacking / metadata validation) as a separate phase from verification (checking that all required steps occurred and signatures match). Antigen has audit that conflates these. May not need to split; the distinction is what makes in-toto able to do client-side checks against a server-side-generated layout.

### What antigen should differentiate from

1. **Antigen is code-locality; in-toto is artifact-locality.** in-toto's substrate lives in `.link` files generated by each step, an `.layout` file signed by the project owner, and inspection results at verification time — all conceptually about artifacts (files produced by build steps). Antigen's substrate lives in `.attest/` sidecars adjacent to source — about code at presentation sites. Different abstraction layer. The germinal-center vs central-registry tension the v2 draft names (§"Per-cell antigen processing → code-locality") is, in in-toto terms, layout-side vs step-side; in-toto chose hybrid (steps generate links locally, layout aggregates centrally), antigen chose pure-distributed.
2. **Tier-honesty has no in-toto equivalent.** in-toto's verification is binary: layout-passed or layout-failed. There's no "we resolved 4 of 5 things" reporting state. Antigen's tier + audit-hint structure is genuinely novel here — the cost of in-toto's binary model is that soft violations either cause hard failures (everyone disables them) or silent passes (no signal at all). Antigen's gradient is the better answer for the developer-facing discipline-witness use case. (in-toto operates at a different layer where binary is correct — you either trust the build or you don't ship the artifact.)
3. **in-toto requires signing infrastructure; antigen ADR-007-defers this.** Antigen ships `Signer.signature: Option<Signature>` as a slot reserved for crypto-signing, with git-trust as default. in-toto requires DSSE-Envelope-wrapped signatures everywhere; no opt-out. The bar is higher; the adoption friction is correspondingly higher. Antigen's "git-trust as default" is the v0.1 right answer for a developer-discipline tool; in-toto's "always signed" is the right answer for a supply-chain integrity tool. Different problem; different default.
4. **Pre-authentication encoding (PAE) and payload-type binding** (see §5 DSSE) is a real concern for in-toto because attestations move across trust boundaries (shipped from build to consumer). Antigen's sidecars largely stay in the repo; PAE is overkill for v0.1. But if antigen ever ships `attest export` for cross-repo aggregation, PAE-style binding becomes load-bearing.

### Quick map to antigen vocabulary

| in-toto | antigen |
|---|---|
| Layout | (no direct analog; closest is the `requires = ...` predicate body on `#[immune]`) |
| Layout step | Antigen declaration |
| Link metadata | `Ratification` JSON sidecar |
| Functionary (public key) | `Signer.name + role` (git-trust) or `+ signature` (crypto-signing slot) |
| `predicateType` URI | Leaf-primitive enum variant (closed) |
| Artifact rules | Combinator grammar (`all_of`/`any_of`/`not`) |
| DSSE Envelope | (deferred — `Signer.signature` slot) |
| `MATCH ... FROM step X` (cross-step binding) | (no analog; probably out of scope) |
| Inspection | (no analog; audit conflates) |
| Sublayout (nested verification) | (no analog; `descended_from` is the closest, but operates at antigen-class level, not predicate level) |

---

## 2. SLSA — Supply-chain Levels for Software Artifacts

**Primary sources**: [SLSA v1.0 levels](https://slsa.dev/spec/v1.0/levels), [SLSA v1.2 levels](https://slsa.dev/spec/v1.2/levels), [SLSA provenance predicate](https://slsa.dev/spec/v1.0-rc1/provenance), SLSA Verification Summary Attestation (VSA).

### Analogous primitive

SLSA is **a leveled compliance framework over in-toto's attestation substrate**. Its analog of an antigen substrate-witness is the **SLSA Provenance predicate** (a specific `predicateType` for in-toto Statements), describing how an artifact was built. Verification is consumer-driven: the consumer reads the provenance, evaluates whether it meets their expected level, and decides.

### Trade-offs SLSA made

**Discrete levels, not continuous gradient.** Three build levels currently active:
- **Build L1**: Provenance exists; "trivial to bypass or forge."
- **Build L2**: Provenance is signed by hosted build platform; tamper-evident.
- **Build L3**: Hardened build platform, isolated signing key, even malicious insider cannot forge.

**Producer claims a level by following the requirements; consumer verifies independently.** The spec is explicit that consumers perform downstream verification of provenance by validating authenticity — comparing actual provenance against expected provenance. There's no central authority that certifies "this build is L3"; the consumer reads the provenance, checks that the build platform's identity matches a known L3-compliant platform, and assigns the level themselves.

**No explicit "claimed level" vs "verified level" terminology.** The SLSA spec describes levels as a producer property (what the producer's build pipeline supports) and verification as a consumer property (what the consumer can confirm). The gap between the two — "this build claims L3 but the signing key isn't actually isolated, so the verified level is really L2" — isn't named directly. Left as implicit consumer-side judgment. **This is notable for antigen**: antigen's tier-honesty discipline is, in part, explicit naming of this gap on the verifier side. SLSA could be read as an example of *what tier-honesty looks like when it's left implicit*.

**Verification Summary Attestation (VSA) is the SLSA-native way to compress "I verified this artifact at level X against policy Y."** A VSA is a separate predicate (`https://slsa.dev/verification_summary/v1`) that a downstream verifier emits, saying "I'm a trusted verifier, and I claim this artifact meets level X." Lets one organization's verification work be reused by another.

### What antigen could learn

1. **The level/tier framing has the same "discrete steps along a verification-strength gradient" shape as antigen's `WitnessTier`.** Both: small number of named states; each state has a definition; consumers gate on the state. Antigen's tier mapping is more granular (Reachability vs Execution vs FormalProof) and per-witness-class (each witness type maps its work onto the same tier ladder), whereas SLSA's levels are per-pipeline (the whole build process gets a level). Worth knowing the precedent exists and predates antigen.
2. **VSA pattern as a compression primitive.** Once antigen has many sidecars across many crates, a workspace-scope summary attestation ("for this revision, the audit ran and 47/47 substrate-witnesses passed at Execution tier") could be a useful artifact for CI-of-CI scenarios. Probably v0.3+ territory; relevant to roadmap.md:151's cross-organization-registry direction.
3. **Discrete levels make explicit-consumer-gating easier than continuous-gradient.** A CI gate "require >= L2" is easy to write; "require Reachability OR Execution OR FormalProof depending on which witness class" is harder. Antigen's choice of per-witness tier complicates downstream gating relative to SLSA's whole-pipeline level. Possibly fine — antigen's consumers are different (the same project's CI vs cross-org artifact consumption) — but the differential cost is real and worth surfacing.

### What antigen should differentiate from

1. **SLSA's level is about the production process, not the evidence content.** L3 means "the build pipeline is hardened" not "the build's output was verified to be correct." Antigen's tier is about the verification work the audit performed. Different axis. Conflating them would weaken antigen — Execution-tier doesn't say anything about the build pipeline's integrity, and shouldn't.
2. **SLSA's verification is fundamentally artifact-centric** (binary blob with hash X). Antigen's verification is source-centric (function/struct/impl at a code site). The artifact-centric model loses meaning when the question is "did the human reviewer ratify the discipline," because reviewers don't ratify binary blobs — they ratify code they read. Antigen's code-locality is the right move for the discipline-witness use case; SLSA-style artifact-locality is wrong-layer here.
3. **SLSA hasn't (yet) tackled the "this attestation is stale because the input changed" problem at the granularity antigen needs.** SLSA's provenance is point-in-time about a specific artifact digest; a new build produces a new provenance for a new digest, no staleness state needed. Antigen's `signed_against_fingerprint` machinery (the affinity-maturation rhyme) is specifically tackling the "function changed; signer's claim no longer about current code" problem — a finer-grained staleness than SLSA has reason to define.

---

## 3. Sigstore — keyless signing for software (Fulcio / Rekor / cosign)

**Primary sources**: [Sigstore overview](https://docs.sigstore.dev/about/overview/), [How it works](https://www.sigstore.dev/how-it-works).

### Analogous primitive

Sigstore is the **signing infrastructure** — not a predicate language or attestation framework. Its analog to "the `Signer.signature` slot in antigen's `Ratification` schema" is the full keyless-signing identity flow.

### Trade-offs Sigstore made

**Keyless signing via short-lived certificates bound to OIDC identity.** Instead of long-lived signing keys (operationally horrible — key rotation, hardware tokens, lost laptops), Sigstore mints a fresh ephemeral keypair per signing operation, gets it signed by Fulcio (the certificate authority) in exchange for a verified OIDC identity token (typically GitHub Actions workflow identity, GitHub user, Google account), and persists the signing event in Rekor (an append-only transparency log).

**Three independent components, each with its own trust model.**
- **Fulcio**: trusts OIDC identity providers; issues short-lived certs binding identity to public key.
- **Rekor**: trusts cryptographic hash chains; provides public auditability and inclusion proofs.
- **cosign**: trusts both, orchestrates the signing flow.

**Verification depends on the verifier knowing which identity is trustworthy.** Sigstore proves "this artifact was signed by someone who controlled the GitHub Actions workflow at `github.com/foo/bar`" — but doesn't tell the verifier whether that identity is authorized for the artifact's purpose. The verifier supplies the policy ("I expect signatures from this GitHub repo's workflow").

### What antigen could learn

1. **Keyless signing via OIDC + transparency log is the right architecture for distributed teams.** Once antigen has crypto-signing (the `Signer.signature` slot activates), Sigstore is the obvious recommendation. Implementation cost is low — `sigstore` Rust crate exists; the trust model is widely understood. Don't reinvent.
2. **OIDC identity is the natural rendezvous between "git author email" (current antigen default) and "verified cryptographic identity" (future).** A signer entry could carry both `name: "alice"` (human-readable) AND `verified_identity: "alice@example.com via google"` (Sigstore-resolved). Doesn't require any v0.1 schema change; just a slot reserved.
3. **Transparency log (Rekor) is the auditability primitive missing from git-trust.** Git commits prove "this trailer was written," but a malicious actor with commit rights can fake the trailer. Rekor inclusion proves "this signing event happened at time T and is now publicly auditable; tampering with this would require rewriting the log." Antigen's git-trust default is fine for in-team workflows; Rekor-backed signatures are the right default for "I want to prove externally that the team did this discipline."

### What antigen should differentiate from

1. **Sigstore is infrastructure, not policy.** Sigstore answers "who signed this?" but not "is the signature meaningful for this purpose?" Antigen's predicate language (`signers(required = ["alice", "bob"])`) is the policy layer that sits on top of an identity-signing infrastructure. Don't conflate. If antigen ever delegates signing to Sigstore, the predicate-evaluation work stays in antigen.
2. **Sigstore's trust model assumes external attackers; antigen's discipline-witnesses largely assume internal good-faith.** Sigstore is hardened against "someone on the internet wants to forge a release"; antigen's discipline-witnesses assume "the team writing the code wants the discipline to be real, and the audit catches accidental drift." The threat model gap means a lot of Sigstore's complexity (PKI, transparency-log inclusion proofs, certificate revocation) is overkill for v0.1 antigen. Worth a future amendment when the threat model expands.

---

## 4. TUF — The Update Framework (role-based trust, threshold signatures)

**Primary sources**: [TUF specification (latest)](https://theupdateframework.github.io/specification/latest/).

### Analogous primitive

TUF is **a role-based metadata signing framework** with strong compromise-resilience properties. Its analog to antigen's substrate-witness is **a metadata file signed by a quorum of role keys**, with threshold signatures and expiration as first-class concepts.

### Trade-offs TUF made

**Four top-level roles, each with distinct responsibilities:**
- **Root**: delegates trust to all other top-level roles; keys offline; rarely rotated.
- **Targets**: signs metadata about which target files are trusted; can delegate to sub-roles for subsets.
- **Snapshot**: signs metadata that nails down the current versions of all targets metadata (prevents mix-and-match attacks).
- **Timestamp**: signs the snapshot's current hash; uses an online key; compromise is minimal because freshness-only.

**Threshold signatures: k of n.** A role can require k valid signatures out of n authorized keys for its metadata to be considered valid. Compromise of k-1 keys doesn't break the role.

**Mandatory expiration.** Every metadata file carries an `expires` timestamp; clients MUST NOT trust expired files. Roles re-sign on a regular cadence; failure to re-sign = the role's trust drops.

**Delegation.** Targets role can delegate trust to sub-roles for specific path patterns, with their own keys/thresholds. Delegations are signed by the delegator. Terminating delegations stop search down the chain.

**Compromise resilience as a design goal.** TUF is explicit: even partial key compromise should not break the system. Online keys (timestamp) protect freshness only, not authenticity. Offline keys (root, targets) provide authenticity. Threshold protects against single-key compromise.

### What antigen could learn

1. **Threshold signatures map cleanly onto antigen's `signers(required = [...])` primitive.** The current v0.1 design requires all named signers to have signed; this is n-of-n. Adding `signers(required_threshold = K, candidates = [...])` (where K < length of candidates) is a small extension that preserves tier-honesty and matches a real pattern from TUF's threshold story. Worth a v0.2 amendment slot.
2. **Mandatory expiration is a useful pattern for safety-critical disciplines.** v0.1 antigen has `fresh_within_days(N)` which is the opt-in version; making it default-required for certain antigen classes ("safety discipline antigens must re-attest every 90 days") is a TUF-like move. Probably v0.2+; the optionality in v0.1 is correct for adoption.
3. **Role-based delegation hierarchy could match antigen's `descended_from` chain.** A descendant antigen's substrate-witness could *delegate* to a parent's signers ("if the parent antigen was signed by `math-researcher` for this file, that signature also covers the descendant"). Concrete pattern to consider when `descended_from` propagation lands. Risk: delegation chains get complex fast; TUF's experience suggests strict path-prefix semantics and terminating-delegations are the discipline that keeps it tractable.
4. **Compromise-resilience is an actual design property, not an aspiration.** TUF's discipline is that no single key compromise breaks the system. Antigen's git-trust default has the opposite property — anyone with commit rights can fake a signer entry. v0.1 acceptance of this is reasonable for in-team workflows; the moment antigen claims to be a real verification surface (CI gate gating release), the threshold-signature primitive needs to land.

### What antigen should differentiate from

1. **TUF has explicit role hierarchy; antigen has role-as-string.** TUF's root → targets → delegated chain is a structural property of the metadata; antigen's `Signer.role: Option<String>` is a free-form label. This is a deliberate v0.1 simplification (most teams don't have formal role hierarchies for discipline review), but it caps the trust-modeling antigen can do. If antigen ever ratifies a "role registry" primitive, TUF's hierarchy is the prior art.
2. **TUF is built for external trust extension (clients downloading updates from servers they don't fully trust); antigen's discipline-witnesses are internal trust artifacts.** Different threat model; different ceremony cost trade-offs. TUF's per-role offline-key discipline is operational overhead that pays off only when external trust is the use case. Antigen's git-trust default is the right v0.1 choice; TUF-style ceremony comes in if/when antigen serves external trust use cases.

---

## 5. DSSE — Dead Simple Signing Envelope

**Primary sources**: [DSSE envelope spec](https://github.com/secure-systems-lab/dsse/blob/master/envelope.md), [DSSE protocol spec](https://github.com/secure-systems-lab/dsse/blob/master/protocol.md).

### Analogous primitive

DSSE is **the wrapper that turns a signed payload into a payload-type-bound signature**, preventing a category of confusion attacks. Its analog to antigen is the envelope around a `Ratification` JSON sidecar — once antigen ships crypto-signing, the question of "how is the signature computed over the JSON" needs a real answer, and DSSE is the canonical right answer.

### Trade-offs DSSE made

**Pre-authentication encoding (PAE) binds payload-type to payload-bytes in the signature.**

The PAE function: `"DSSEv1" + SP + LEN(type) + SP + type + SP + LEN(body) + SP + body`. The signature is computed over this string, not over the raw payload. Result: an attacker can't take a signed payload, re-label it with a different `payloadType`, and have the signature still verify. The payload-type is part of what's signed.

**Why this matters**: without PAE, you can ship the same JSON bytes as a SLSA Provenance predicate, then later re-label as an antigen Ratification (or vice versa), and the signature stays valid — letting the consumer parse the same bytes under the wrong schema. PAE prevents this by making the type a signed input.

**Three-tuple structure: (payload, payloadType, signatures[]).**
- `payload`: Base64-encoded serialized body.
- `payloadType`: MIME-style type identifier (e.g., `application/vnd.in-toto+json`).
- `signatures[]`: array of (keyid, sig) pairs — supports multi-signing.

### What antigen could learn

1. **If antigen ships crypto-signing, use DSSE. Don't roll your own envelope.** The PAE work is non-obvious and easy to get wrong; the multi-signature shape matches `signers[]`; the spec is short and well-reviewed; existing Rust libraries exist (`sigstore` crate uses DSSE internally).
2. **Multi-signature in the envelope (rather than per-signer signature embedded in payload) is cleaner.** The Ratification JSON stays clean (no signature fields cluttering the schema); the envelope carries all signatures. The "signer commented out their signature" failure mode disappears (you either parse the envelope and have signers, or you don't).
3. **Payload-type binding becomes load-bearing once attestations move across repos.** v0.1 antigen with code-locality keeps sidecars in-tree; an attacker would have to alter both the JSON and the producing macro, which is the normal attack surface. If antigen ever exports sidecars (cross-org sharing per roadmap.md:151), DSSE-style binding becomes critical.

### What antigen should differentiate from

1. **DSSE assumes you have signatures; v0.1 antigen has git-trust as default.** The PAE story is overkill if there are no signatures to compute. The slot is what matters in v0.1; DSSE comes later.
2. **DSSE doesn't address content-validity, only authentication-validity.** A DSSE-wrapped payload proves "this was signed by these keys for this type"; it does not prove "the predicate's content evaluates to true." Antigen's predicate evaluation is content-validity; sits above DSSE-style authentication. Confusion here would weaken both layers.

---

## 6. PASETO — Platform-Agnostic Security Tokens

**Primary sources**: [PASETO spec on GitHub](https://github.com/paseto-standard/paseto-spec).

### Analogous primitive

PASETO is **a token format with fixed algorithm choice per version** — the JWT-replacement that learned from JWT's algorithm-confusion footguns. The analog to antigen is the general design lesson about closed-vs-open algorithm choice: PASETO chose closed, and the design lesson is widely accepted as correct.

### Trade-offs PASETO made

**No algorithm agility.** JWT lets the producer pick an algorithm (`alg: HS256`, `alg: RS256`, even `alg: none`); the verifier must accept whatever the producer chose. This is the root of multiple real-world JWT attacks (the `alg: none` attack; the public-key-as-HMAC-key attack). PASETO fixes one algorithm per version:
- v1 (legacy): RSA-PSS-SHA384, AES-256-CTR + HMAC-SHA384
- v2: Ed25519, XChaCha20-Poly1305
- v3, v4: progressively newer primitives

**Two-axis classification: (version, purpose).** Purpose is `local` (symmetric, server-only) or `public` (asymmetric, verifiable by anyone). Each combination has exactly one algorithm. No negotiation; no confusion.

**Strict separation of concerns.** A token is "v2.public" or "v2.local"; verifiers parse the prefix and use the corresponding fixed primitive. Verifier doesn't read algorithm metadata from inside the token; can't be tricked.

### What antigen could learn

1. **The general lesson — closed grammar over open grammar at the security boundary.** PASETO's design choice is the small-scale version of what antigen is doing with combinator grammar (v2 R5 "Predicate-language ceiling"). Both refuse to let producers choose algorithms / predicate shapes / expansion paths at use-time. Both pay an expressiveness cost in exchange for not-having-to-think-about-confusion-attacks. The trade-off has been validated in multiple security domains; antigen's choice is in good company.
2. **Versioning the language itself is what enables future-evolution-without-confusion.** PASETO v2/v3/v4 each have their own fixed algorithms; old versions don't break because their algorithms stay fixed; new versions add new primitives without back-compat hazard. Antigen's `schema_version: 1` (v2 R3) is the same move — versioning the substrate format so future versions can extend without breaking. PASETO's design suggests versioning the combinator grammar itself might be a useful slot to reserve too: `requires_v1 = all_of([...])` vs eventual `requires_v2 = all_of_v2([...])` if v2 introduces semantics that would silently change v1 evaluation. (Probably overkill for v0.1; flagged so the team can decide.)

### What antigen should differentiate from

1. **PASETO operates at the byte-level; antigen operates at the source-code level.** Different stacks; the PASETO lesson is structural, not procedural.
2. **PASETO has no analog to antigen's "tier-honesty" or "audit-hint" reporting.** PASETO is verify-or-fail; either the token is valid or it isn't. PASETO doesn't need to report "we did some of the verification work"; the verification is atomic. Antigen's gradient is in a different problem space.

---

## 7. cargo-deny — Rust dependency policy enforcement

**Primary sources**: [cargo-deny advisories config](https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html), [cargo-deny cfg overview](https://embarkstudios.github.io/cargo-deny/checks/cfg.html), [cargo-audit audit.toml example](https://github.com/rustsec/rustsec/blob/main/cargo-audit/audit.toml.example).

### Analogous primitive

cargo-deny is **a declarative TOML policy language over the Rust dependency graph**. It checks licenses, advisories, banned crates, and source registries. Its analog to antigen's predicate language is **the entire TOML configuration shape** — closed grammar; named primitives; sequential per-rule evaluation against the substrate (Cargo.lock + RustSec database).

### Trade-offs cargo-deny made

**Pure declarative, closed grammar.** No conditional logic; no expressions; no user-defined functions. Configuration is "this license is allowed," "this advisory is ignored," "this crate is denied." That's it.

**Per-rule allow / deny / exceptions structure.**
- `[licenses]`: `allow = [...]`, `deny = [...]`, `exceptions = [...]` with per-crate-specific allowances.
- `[advisories]`: `ignore = [...]` with two formats: bare ID string, or `{id, reason}` object with explanation. Also: severity threshold, informational/unmaintained/unsound handling.
- `[bans]`: `deny = [...]` with `skip = [...]` and `skip-tree = [...]` for transitive exception.
- `[sources]`: `unknown-registry = 'deny'|'warn'|'allow'`, registry allowlist.

**`unused-ignored-advisory` warning is the tier-honesty primitive cargo-deny ships.** When an advisory you've ignored is no longer relevant (the vulnerable crate was upgraded out), cargo-deny tells you. The ignore was tier-honest at the time; the world moved on; the ignore is now stale and should be removed. Same shape as antigen's `discipline-substrate-stale` audit hint, applied to a different substrate.

**Ignore-with-reason is opt-in but encouraged.** Bare-string ignores are valid TOML; object-with-reason is a soft community pattern for code-review-able discipline.

### What antigen could learn

1. **`unused-ignored-advisory` is a clean precedent for "this attestation no longer corresponds to current substrate."** Antigen's discipline-substrate-stale handles the case where an antigen signature pins to an old fingerprint. cargo-deny's pattern is the dual: the attestation (ignore) still exists, but the substrate (vulnerable crate) no longer exists. Antigen should ship a parallel hint: `discipline-attestation-orphaned` for attestations whose presenting site no longer exists. v2 draft already has `attest gc` for this (CLI command); the audit-side hint version would be its diagnostic counterpart.
2. **Object-with-reason format for ignores is widely accepted in the Rust ecosystem.** cargo-deny, cargo-audit, and cargo-vet all support some variant. Antigen's `Signer` struct already carries `role` and `date`; adding `Signer.rationale: Option<String>` (or equivalent on the predicate-leaf-failure side) for "why this leaf is currently failing but tolerated" would slot in cleanly. v2 already mandates rationale on `#[immune]` via ADR-005 Am 2; extending to per-leaf-bypass is consistent posture.
3. **Per-rule documentation linking is a quiet adoption boost.** cargo-deny's RUSTSEC-XXXX-YYYY entries link to advisory docs; ignores reference those docs. Antigen's `discipline_doc` field is the same shape (per v2 draft, "Strengthened" item 3).

### What antigen should differentiate from

1. **cargo-deny is per-dependency-graph; antigen is per-code-site.** cargo-deny checks each crate version once; antigen checks each function/struct/impl per-presentation. Granularity difference. cargo-deny doesn't need fingerprint-pinning because crate versions are already content-addressed; antigen needs fingerprint-pinning because source code mutates within a version.
2. **cargo-deny has no "tier" notion — it's binary fail/pass.** Antigen's tier gradient is the discipline-witness-specific elaboration. cargo-deny's `severity_threshold` is the closest analog; antigen's tier+hint structure is richer.
3. **cargo-deny's ignore is ambient cover; antigen's tier-honesty fights ambient cover.** A cargo-deny ignore-string with no reason hides what was suppressed; an antigen substrate-witness predicate-failure-with-hints exposes exactly which leaf failed. The cargo-deny laziness (bare-string-ignore) is the trap antigen's design explicitly avoids.

---

## 8. cargo-vet — supply chain audits for Rust crates

**Primary sources**: [cargo-vet audit criteria](https://mozilla.github.io/cargo-vet/audit-criteria.html), [cargo-vet recording audits](https://mozilla.github.io/cargo-vet/recording-audits.html), [cargo-vet how it works](https://mozilla.github.io/cargo-vet/how-it-works.html), [Google Rust crate audits criteria](https://github.com/google/rust-crate-audits).

### Analogous primitive

cargo-vet is **a distributed audit ledger format for Rust crates with custom criteria support**. Its analog to antigen is *extremely close* — possibly the closest analog in the entire landscape. cargo-vet's `audits.toml` is doing for cross-crate trust what antigen's `.attest/` is doing for in-crate trust.

### Trade-offs cargo-vet made

**Custom criteria as named primitives.** Built-in: `safe-to-run`, `safe-to-deploy`. Projects can define arbitrary additional criteria in `audits.toml`. Example custom criterion (from Google): `ub-risk-0` through `ub-risk-4` + thorough variants, forming an explicit implication chain.

**Per-criterion human-readable description.** Each criterion has a paragraph-long description "to check for" when evaluating. When auditing, the auditor confirms the criterion holds; when verifying, the consumer reads the criterion description and decides whether their threat model matches.

**Audit record structure (audits.toml):**
```toml
[[audits.bar]]
version = "1.2.3"
who = "Alice Foo <alicefoo@example.com>"
criteria = "safe-to-deploy"
```

Exactly one of `version` / `delta` / `violation` per entry. `who` is git-trust style (name + email, no signature). `criteria` references named criterion from `criteria` section.

**Delta audits as first-class primitive.** Instead of re-auditing the whole crate at every version, an auditor can claim "I reviewed the diff between 1.2.3 and 1.2.4 and the relevant properties are preserved." Composes: a chain of deltas from a fully-audited base version = full coverage at the latest version.

**Distributed aggregation.** Projects can `import` other projects' audits; cargo-vet's verification walks the union. When aggregating, criterion descriptions must be byte-identical across sources (no semantic merging — they're trusted-as-named).

**Decentralized, in-tree storage.** Audits live in `supply-chain/audits.toml` in the consuming project's repo, alongside `imports.lock` recording the imported audit sets.

**No cryptographic signing.** Git-trust is the entire trust model; audits are TOML entries committed by repo maintainers; tampering would require git history rewrite, which is itself a tamper-evident operation.

### What antigen could learn

1. **Custom criteria as the open-extension primitive over a closed framework.** cargo-vet has two built-in criteria; everything else is project-defined. The framework's job is to provide the namespace (criterion name), the auditing ceremony (record-an-entry), and the verification machinery (criterion N+1 implies criterion N); projects supply the *semantics* (what does `safe-to-deploy` mean for our threat model?). Antigen's leaf-primitive enum is more closed than this; the v2 draft argues for that explicitly (R5). cargo-vet is the canonical counter-example showing what *open at the criterion layer* looks like. Worth noting: cargo-vet's openness works because the criteria are *human-evaluated*; antigen's leaves are *machine-evaluated*, which forces closure at a different layer.
2. **Delta audits are the affinity-maturation primitive cargo-vet ships.** When code changes, you don't need to re-audit from scratch — audit the delta. Antigen's `signed_against_fingerprint` + the v2 stale-detection logic is doing the same thing at finer granularity (per-function, not per-crate-version). cargo-vet's experience suggests delta-audits should be a *first-class action* in antigen's CLI, not just an implicit consequence of re-signing. Consider: `cargo antigen attest delta --from <old-fingerprint> --to <new-fingerprint> --as math-researcher` that explicitly records a delta-audit, preserving the chain.
3. **Imported audits as the trust-extension primitive.** cargo-vet lets project A import project B's audits with a single import line; aggregation is automatic. Antigen's equivalent would be cross-crate ratification import — "I trust antigen-witnesses-research-rigor crate's ratifications for crates in their dependency tree." Roadmap.md:151 territory; cargo-vet has shipped the pattern.
4. **In-tree storage with no crypto** is the right adoption-friction default for v0.1. cargo-vet has been adopted at scale by Mozilla, Google, and others using only git-trust; that's proof of concept that the v0.1 antigen posture (`Signer.signature: Option<Signature>` reserved-slot, git-trust default) is sound. Crypto-signing is the v0.2+ ceremony for teams that need it.

### What antigen should differentiate from

1. **cargo-vet operates over a content-addressed corpus (crates.io versions); antigen operates over a mutating corpus (source files in the repo).** cargo-vet doesn't need per-function granularity because (crate-name, version) is the natural unit. Antigen needs per-item granularity because (file, fn) mutates without changing crate version. Different problem; different granularity.
2. **cargo-vet has open criteria (human-evaluated); antigen has closed leaves (machine-evaluated).** Don't import cargo-vet's open-criteria pattern into antigen's leaf-primitive set — it would force the audit to either trust the user-defined leaf's semantics (tier-honesty violation: audit reports verification it can't perform) or refuse to evaluate it (defeats the purpose). Closed leaves + named compositions (Tier 2 in v2 draft) is the right hybrid.
3. **cargo-vet's verification is graph-walking (does every dependency have an audit at the required criterion?); antigen's verification is per-site predicate-evaluation.** Different verification model. cargo-vet can't say "this audit is partial"; antigen can. The tier-honesty richness is antigen's.
4. **Note**: cargo-vet is the system most aligned with antigen's posture overall — distributed, in-tree, git-trust, declarative, custom-extensible-with-discipline. If antigen wants a precedent for "this approach has been adopted at scale," cargo-vet is it.

---

## 9. Open Policy Agent / Rego — predicate language design

**Primary sources**: [OPA Rego policy language docs](https://www.openpolicyagent.org/docs/policy-language), [Datalog vs Rego comparison](https://www.openpolicyagent.org/docs/comparison-to-other-systems).

### Analogous primitive

Open Policy Agent (OPA) is **a general-purpose policy engine with a custom declarative language (Rego)** based on Datalog. Its analog to antigen's predicate language is *the path antigen explicitly didn't take* — Turing-bounded-but-still-expressive language vs closed-combinator-grammar. The contrast is illuminating.

### Trade-offs OPA made

**Datalog-derived, decidable, not Turing-complete.** Rego doesn't have traditional loops or recursion. It uses existential quantification by default (queries succeed if *some* binding satisfies the conditions), with `every` for universal quantification. Negation-as-failure. Set comprehensions. Pure logic-programming idiom.

**Decidability over expressiveness.** OPA explicitly chose decidability (and the resulting performance guarantees) over maximum expressiveness. From the docs: "Like other applications which support declarative query languages, OPA is able to optimize queries to improve performance." Knowing every query terminates is what makes OPA usable as an inline policy engine.

**Open at the rule-definition layer.** Anyone can write any Rego rule. The closed-set is at the *evaluator* — what operations are available — not at the *rule corpus*. Rules are user-authored over data the user supplies.

**Verification by query.** Rather than verifying a closed predicate over substrate, OPA exposes an evaluator that lets the user *query* the policy: "given this input, what does the policy say?" The answer is a value (decision: allow/deny/etc), not a tier.

**Explanation via metadata annotations, not built-in.** Rules can carry metadata describing intent ("why does this rule exist?") but the engine doesn't expose *which rule fired* by default — debugging mode is opt-in.

### What antigen could learn

1. **Decidability-over-expressiveness is the right trade-off for verification engines.** OPA's choice is widely validated in the policy-engine space. Antigen's closed-combinator-grammar is the stricter version of the same trade-off. Both: the verifier is guaranteed to terminate; the producer trades expressive freedom for verifier-confidence. Worth naming explicitly in antigen's ADR: "Rego made this trade-off at the decidability-over-Turing-completeness boundary; antigen makes it at the closed-combinator-over-Datalog boundary; one direction tighter."
2. **Annotation-based intent documentation is a quiet adoption boost.** OPA's metadata-as-comments pattern is a way for rules to carry their own documentation. Antigen's leaf-primitives could carry per-leaf metadata that audit reports surface ("this `ratified_doc` is checking the v1.0+ frontmatter at `docs/x.md` because antigen-X declared that as its `discipline_doc`"). v2 draft already specifies per-leaf-failure detail; the success-side metadata is the dual.
3. **Closed-vs-open at the *evaluator* layer is the dimension that matters for confusion attacks.** OPA's open at the rule layer is fine because the evaluator is closed; antigen's closed at the leaf layer is what locks down the evaluator. The lesson: as long as the evaluator is closed, openness elsewhere is manageable.

### What antigen should differentiate from

1. **OPA's expressiveness ceiling is much higher than antigen's combinator grammar.** Rego can compute things; antigen's combinators only combine. The difference is intentional: antigen targets a much narrower problem (verifying named substrate satisfies named predicates), and the narrower closed grammar gives stronger tier-honesty guarantees. OPA's expressiveness is what enables OPA's use cases (Kubernetes admission policies, application authorization); antigen would gain nothing from importing that expressiveness and lose the closed-combinator-grammar discipline.
2. **OPA's "decision" is a value, not a tier.** Decision-as-value (allow/deny/some-structured-object) is what makes OPA flexible. Antigen's decision-as-tier-plus-hint is what makes antigen's verification reportable in a uniform way. Different problem; different output shape.
3. **OPA assumes the policy author knows the data shape**; antigen's leaves *recognize* substrate shape (ratified_doc knows about frontmatter; signers knows about sidecar array). This is the recognition-not-design discipline (ADR-006) at the engine layer: antigen's engine bakes in knowledge of common substrate; OPA's engine is substrate-agnostic. Antigen's choice is appropriate for its narrower problem; OPA's choice is appropriate for being a general policy engine.

---

## 10. Salsa — incremental computation provenance

**Primary sources**: [Salsa book overview](https://salsa-rs.github.io/salsa/overview.html), [Salsa red-green algorithm](https://salsa-rs.github.io/salsa/reference/algorithm.html), [Salsa durability](https://docs.rs/salsa/latest/salsa/struct.Durability.html), [rust-analyzer durable incrementality post](https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html).

### Analogous primitive

Salsa is **an incremental computation framework that tracks input-derivation provenance to enable cached re-evaluation**. Its analog to antigen is *the dependency-tracking machinery* — Salsa tracks which inputs each derivation read, so that when an input changes, only affected derivations re-execute. The structural rhyme to antigen is at the fingerprint/staleness layer.

### Trade-offs Salsa made

**Inputs vs tracked functions.** Inputs are explicit (`#[salsa::input]`); tracked functions are pure derivations that depend (transitively) on inputs. Salsa tracks *which inputs each function read* to build the dependency graph.

**Red-green algorithm for invalidation.** Rather than fingerprint-based invalidation (which would always re-execute when any input changed), Salsa runs a smart re-evaluation: when an input changes, only derivations whose actually-read inputs changed *value* get re-executed. Derivations that *didn't actually depend on the changed input value* skip re-execution even though they touched the input. Result: much finer-grained incrementality than naive caching.

**Durability levels.** Durability is "how likely a value is to change." `Durability::LOW` for things like edited source code; `Durability::HIGH` for interned values. If a tracked function depends only on high-durability inputs, changing a low-durability input doesn't trigger any reconsideration — the dependency-walk can skip whole subtrees. Optimization for the common case (rapid edits in one file shouldn't invalidate analyses of unrelated files).

**Tracked structs as intermediate values.** Tracked structs represent intermediate derivations that may themselves be inputs to other derivations. Salsa carries provenance through the whole computation graph.

### What antigen could learn

1. **The red-green algorithm is the right pattern for sophisticated fingerprint staleness.** Antigen's v2 fingerprint-pinned signature is naive: any code change invalidates all signatures pinned to the old fingerprint. Red-green-style would be smarter: a signature stays valid if the *aspect of the code the discipline depends on* didn't change. v2 draft doesn't propose this (and probably shouldn't for v0.1 — Salsa-style fine-grained dependency tracking is hard), but it's the obvious v0.2+ direction. A naturalist note: this is also the affinity-maturation rhyme deepened — antibodies aren't invalidated by every antigen mutation, only by mutations to the binding epitope.
2. **Durability levels as antigen-class metadata.** Some discipline antigens describe rapidly-changing concerns (performance thresholds that drift with infra changes); some describe stable concerns (numerical-stability invariants that are properties of the algorithm forever). Tagging antigen declarations with a durability hint would let the audit treat them differently: low-durability antigens require frequent re-attestation; high-durability antigens have longer freshness windows. Probably v0.2+; useful slot to reserve.
3. **Tracked-struct pattern as the intermediate-attestation primitive.** A "this team has reviewed `docs/protocol.md`" attestation can itself be an input to "this function attests against `docs/protocol.md` v1.2+ which the team reviewed at version 1.2." Salsa tracks this kind of chain natively. Antigen's `descended_from` is the closest cognate; the chain-of-attestations pattern is the more general primitive.

### What antigen should differentiate from

1. **Salsa is a *computation* framework; antigen is a *verification* framework.** Salsa caches *results*; antigen verifies *claims*. The dependency-tracking analogy is structural, not behavioral. Salsa's invalidation is "this cached value is stale because its inputs changed"; antigen's invalidation is "this attestation no longer corresponds to current code because its substrate changed." Same shape, different semantic.
2. **Salsa's "input" is data; antigen's "input" is structured presentation-site + sidecar.** The granularity difference matters for v0.1 — antigen's fingerprint is per-item-source; Salsa-style red-green would require tracking *which fields* of which item-source each leaf-primitive actually consumed. Probably not worth it in v0.1; the fine-grained tracking is what makes Salsa expensive to use.

---

## 11. GitHub CODEOWNERS / branch-protection / required-reviewers

**Primary sources**: [About code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners), [Managing branch protection rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/managing-a-branch-protection-rule), [Available rules for rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets).

### Analogous primitive

GitHub CODEOWNERS + branch protection is **a forge-side role-based gating mechanism for code review**. Its analog to antigen's substrate-witness is the role-resolution piece: who is allowed to ratify *this part of the code*?

### Trade-offs GitHub made

**CODEOWNERS as path-glob-to-user/team mapping.** `*.rs @rust-team`, `/src/numerics/ @math-team @alice`. Path-globs match changed files; the matched team/user becomes a required reviewer.

**Branch protection ratifies "this branch requires CODEOWNERS approval."** Without branch protection, CODEOWNERS is advisory; with it, the platform enforces "at least one approval from required reviewers from CODEOWNERS for this path." Multiple-approvals via `Required number of approvals` (1, 2, 3, ...).

**Stale review dismissal.** "Dismiss stale pull request approvals when new commits are pushed" — when the PR's diff changes, the approving review is dismissed; the PR cannot merge until re-approved. This is *exactly* antigen's `signed_against_fingerprint` pattern, except at PR-diff granularity rather than per-function fingerprint granularity.

**Identity is GitHub-account-bound.** No federation; no portable identity. CODEOWNERS knows about `@alice` only in the context of `github.com/foo/bar`'s collaborator list.

### What antigen could learn

1. **Stale-on-change-of-substrate is a battle-tested pattern.** GitHub has shipped this since 2018 and it works. Antigen's adoption of the same pattern at finer granularity (per-fingerprint-per-item rather than per-PR-diff) is *more discipline*, not less, and the precedent supports the design choice. v2 R7 (recognition-vs-design ratio) is supported here — fingerprint-pinned-signatures-go-stale-on-change is *recognition* of an established pattern, not novel design.
2. **CODEOWNERS-style role resolution as the optional `required_role` field.** v2 draft already names this as a v0.2 slot (T2 open question). The reading is: CODEOWNERS is the canonical pattern; antigen should support resolving `required = ["@math-team"]` against a CODEOWNERS-style file (in-tree, no forge-API needed). The v0.1 `signers(required = [literal-names])` is fine; v0.2 should add the role-resolution layer.
3. **Required-N-of-list with explicit count is the natural composition for compound review.** GitHub's `Required number of approvals = 2` against multiple required reviewers is the threshold-signature pattern at the forge layer. Antigen's `signers(required_threshold = 2, candidates = [...])` would match the pattern adopters already know.
4. **The "dismiss stale review" toggle being opt-in is interesting.** GitHub gives teams the choice — some teams want stale-on-change discipline, some don't (because they trust review-quality enough that minor changes shouldn't re-trigger). Antigen's design is "stale-on-change is mandatory" via fingerprint pinning. Worth knowing the precedent says "make it configurable"; v2 design is "make it mandatory" because tier-honesty demands it. Different posture; antigen's stricter posture is defensible.

### What antigen should differentiate from

1. **CODEOWNERS lives on the forge; antigen lives in-tree.** This is a fundamental difference. CODEOWNERS is *visible* in-tree (it's a file at `.github/CODEOWNERS` or `CODEOWNERS`), but *enforced* by the forge (branch protection); without GitHub, there's no enforcement. Antigen's enforcement is local — `cargo antigen audit` runs against the working tree regardless of forge. Different trust boundary; different deployment model. Antigen can integrate with CODEOWNERS (read the file, resolve roles) without depending on it for enforcement.
2. **PR-diff granularity vs per-item fingerprint granularity.** GitHub's stale-dismissal is "any change to the PR re-dismisses all approvals." Antigen's stale-detection is "only changes to the specific item invalidate the specific signature." Antigen's granularity is structurally better for large changes that touch many items but only meaningfully change a few — fewer false-positive re-reviews.
3. **GitHub CODEOWNERS is identity-on-forge; antigen is identity-on-git-config.** A `@alice` review on GitHub is a different identity from an `alice@example.com` git commit. Antigen's git-trust default reads `git config user.name`/`user.email`; CODEOWNERS reads `@alice` (GitHub username). Bridging these is a small mapping problem; v0.2 forge-API integration is where it lives.

---

## 12. Dependabot / Renovate / OSV-Scanner — advisory ratification + dependency dashboard

**Primary sources**: [Renovate dependency dashboard](https://docs.renovatebot.com/key-concepts/dashboard/), [Renovate configuration options](https://docs.renovatebot.com/configuration-options/), [GitHub Dependabot dismissed alerts](https://docs.github.com/en/code-security/dependabot/dependabot-alerts/managing-dependabot-alerts).

### Analogous primitive

Dependabot/Renovate are **automated dependency-update bots with dashboard-based human ratification**. Their analog to antigen is the *dashboard-driven gate state* — a centralized view of "what's pending review" with explicit approve/dismiss states, each carrying a reason.

### Trade-offs Dependabot/Renovate made

**Dashboard as the central state-tracking surface.** Renovate's dependency dashboard issue summarizes every PR in flight, every blocked update, every ignored dep. Single issue per repo; markdown checklist for explicit approval.

**`dependencyDashboardApproval` toggle gates PR creation.** Renovate can require human checkbox-tick on the dashboard before a PR is even created. Approval is the substrate (a checkbox tick on the dashboard issue) for the gate.

**`ignoreDeps` as the suppression primitive.** Free-form `ignoreDeps: ["lodash", "moment"]` says "do not update these." No reason required (though configurable for the gate to demand one).

**Automerge as the auto-ratification primitive.** `automerge: true` says "if tests pass, merge without human approval." Toggleable per-update-type, per-package, etc. Major versions typically require human review; patch/minor often automerged.

**Dismissed-alert state with required reason.** GitHub's Dependabot Alerts: dismissed alerts carry a structured reason (won't-fix / used-in-test / no-bandwidth / inaccurate / risky-without-context) plus an optional comment. Reason is structured; comment is freeform.

### What antigen could learn

1. **Dashboard view of pending discipline-attestations is a CLI primitive worth shipping.** `cargo antigen attest list --pending` is v2 draft territory (CLI R6). Renovate's dashboard format suggests the right output shape: per-antigen-per-item rows with "what's blocking" and "who can resolve" columns. Markdown table output for paste-into-PR is the bridge UX.
2. **Structured dismissal reasons as discipline.** Dependabot's dismiss-with-reason is a tier-honesty primitive — the suppression is *named* so reviewers can audit it. Antigen's substrate-witness predicate-fail-with-hints is the structural parallel; if antigen ever ships per-leaf-bypass (rare; design-debatable), the bypass must carry a structured reason matching Dependabot's pattern.
3. **`automerge` as a teaching-moment for "what should NEVER be auto-ratified."** Renovate's experience is that major-version updates and dependency-tree changes should always require human review. Antigen's discipline-witnesses are explicitly the same shape — there is no automerge path for `discipline-substrate-stale` because re-attestation is *exactly* the discipline being asserted. The slot doesn't even exist in v2 draft; that's the right design.

### What antigen should differentiate from

1. **Dependabot's `ignoreDeps` is ambient cover; antigen actively fights this.** The whole point of antigen's tier-honesty is that suppression-without-substrate is a tier-honesty violation. Dependabot's ignoreDeps with no required reason is the laundering surface antigen's design prevents.
2. **Renovate's dashboard is a *forge-side* artifact; antigen's audit output is a *local* artifact.** Renovate posts an issue; antigen prints to stdout. Different surface; different consumers. If antigen ever ships a forge-integrated dashboard, Renovate's UX is the prior art.
3. **Dependency updates are exogenous; discipline-witnesses are endogenous.** Renovate manages "the world outside changed; what do we do about it?" Antigen manages "we declared this discipline; is it still satisfied?" Different problem shapes; the dashboard pattern transfers but the underlying gating semantics differ.

---

## 13. npm/Yarn audit + GitHub Security Advisories suppression

**Primary sources**: [npm audit](https://docs.npmjs.com/cli/v11/commands/npm-audit/), [GitHub Code Scanning alert dismissal](https://docs.github.com/en/code-security/code-scanning/managing-code-scanning-alerts/resolving-code-scanning-alerts), [SARIF suppression spec](https://docs.oasis-open.org/sarif/sarif/v2.1.0/cs01/sarif-v2.1.0-cs01.html).

### Analogous primitive

npm audit + GitHub Security Advisories represent **the advisory-suppression pattern at scale**. The analog to antigen is how *dismissal with audit trail* is structured — a real-world ecosystem with millions of dismissals operates here.

### Trade-offs the npm/GHA pattern made

**Structured dismissal reasons.** GitHub Code Scanning: `false-positive`, `won't-fix`, `used-in-tests`. Each carries an optional comment for justification. Dismissal events are recorded in the alert timeline.

**SARIF as the cross-tool format for suppressions.** Static Analysis Results Interchange Format (SARIF) is an OASIS standard; `suppressions[]` array on each result carries kind + justification + timestamp. Lets any analyzer's output flow into any consumer's dashboard with consistent dismissal semantics.

**`//lgtm` and `//codeql` inline comments as the source-locality variant.** Some teams prefer in-source suppression annotations over external suppression lists. CodeQL supports parsing source comments to populate SARIF suppression entries.

**npm audit lacks first-class dismissal.** npm's `audit` command reports advisories but has no built-in dismissal mechanism in `package-lock.json`. Teams either use `npm audit --audit-level=high` (severity threshold) or rely on external tools (`.npmauditignore`, GitHub Dependabot dismissal). This gap is widely complained about and is a real adoption friction signal.

### What antigen could learn

1. **In-source suppression annotations have an established pattern (lgtm/codeql).** Antigen's `#[immune(X, requires = ...)]` is the in-source ratification side; the in-source *bypass* side ("yes, we know this fails, here's why") would be a parallel attribute. Probably not v0.1; the design discussion in v2 explicitly fences off "vibes-grade attestation" — there should be *no* `witness = trust_me` escape hatch. But the codeql pattern shows there's a legitimate place for *acknowledged failure* (different from suppression) in mature tools.
2. **SARIF as a future export format.** If antigen ever wants its audit output consumable by generic IDE/CI dashboards (VS Code Problems pane, GitHub Code Scanning), SARIF is the lingua franca. v0.3+ territory; ADR-002 (compose-don't-compete) supports adopting an existing format rather than inventing one.
3. **The npm-audit-gap is a cautionary tale.** npm's lack of structured dismissal is a real adoption friction — teams hack around it with `.npmauditignore` files and bespoke CI scripts. Antigen avoiding this gap by *not having a dismissal primitive* (predicate-fail = predicate-fail; re-attest or fix the code) is the cleaner design. The cost is that teams under deadline pressure will work around antigen's strictness in ways the tool can't see; the benefit is that antigen never lies about what was verified.

### What antigen should differentiate from

1. **npm audit / GHA dismissal is *post-hoc rationalization*; antigen's `#[immune]` is *forward-looking ratification*.** Different temporal direction. Dismissal says "we accept this risk"; ratification says "we declare this discipline holds." Both have legitimate uses; conflating them weakens both.
2. **GitHub's `won't-fix` reason is the explicit "accept the tradeoff" version of the v2 draft's generated-code escape valve.** Worth noting that the antigen escape valve is opinionated about *where* it applies (generator boundary, per ADR-002), whereas GHA's `won't-fix` is unscoped. Antigen's scoping discipline is the differentiation.

---

## 14. GUAC — Graph of Understanding Artifact Composition

**Primary sources**: [GUAC project](https://guac.sh/), [GUAC docs](https://docs.guac.sh/), [GUAC GitHub](https://github.com/guacsec/guac).

### Analogous primitive

GUAC is **a knowledge graph aggregating SBOMs, SLSA attestations, and other supply-chain metadata into a queryable graph database**. Its analog to antigen is *the cross-artifact aggregation layer* — what does workspace-scope antigen aggregation look like at the cross-repo / cross-org level?

### Trade-offs GUAC made

**Ingest, normalize, link.** GUAC takes SBOMs (SPDX, CycloneDX) and attestations (SLSA, in-toto), normalizes them into nodes/edges in a graph database (PostgreSQL or Neo4j), then exposes a GraphQL API for queries.

**Cross-source identity reconciliation.** Different SBOMs may name the same artifact differently (different package URLs, different content digests for different formats). GUAC's normalization pass attempts to reconcile these into single graph nodes.

**Queryable across the whole graph.** "Querying for a given artifact may return its SBOM, provenance, build chain, project scorecard, vulnerabilities, and recent lifecycle events — and those for its transitive dependencies." This is the SLSA/in-toto data, but aggregated into a queryable view.

**Designed for org-scale policy decisions.** GUAC's intended consumers are organizations asking "show me all artifacts in my fleet that depend on log4j 2.x"; this is the cross-cutting query antigen sidecars can't answer in isolation.

### What antigen could learn

1. **Aggregation across many sidecars requires a query primitive.** v2 draft's `cargo antigen attest list` is a per-workspace query; a multi-workspace query primitive (or an export-to-format-X primitive) is the v0.3+ direction GUAC has explored. Don't reinvent the graph-database angle; an export-to-SARIF or export-to-GraphQL adapter is the lightweight path.
2. **Identity reconciliation across sources is a hard problem you don't want to discover late.** GUAC's experience: when two attestations reference "the same thing" via different identifiers, the normalization layer is the load-bearing piece. Antigen's identity is `(crate, file, item_path, antigen_name)` — already pretty robust within-workspace; cross-workspace would need a federated identity scheme. Slot worth reserving; v0.1 doesn't need to ship.
3. **Knowledge-graph traversal is the answer to "show me all places where this discipline applies."** "Which crates in my fleet present `SignedZeroDiscipline`?" is a graph query, not a per-crate predicate evaluation. Antigen's audit answers per-crate; cross-crate queries are GUAC-shaped. Roadmap.md:151 territory.

### What antigen should differentiate from

1. **GUAC operates at the artifact/SBOM layer; antigen operates at the source-code layer.** Different layer of the supply chain. GUAC wouldn't be the right substrate for antigen's primary verification work (it doesn't know about source files), but could be a downstream consumer of antigen audit results aggregated to artifact level.
2. **GUAC requires a database; antigen is database-less.** v0.1 antigen runs entirely on filesystem + git; that's deliberate (low adoption friction). GUAC requires a Postgres or Neo4j deployment. If antigen ever offers a "workspace federation" feature, the design space is "lightweight filesystem-based federation" vs "heavyweight GUAC-style aggregation." The former matches antigen's posture better.

---

## 15. Synthesis

This section names what antigen can absorb, what antigen should differentiate from, and what's genuinely novel about antigen's substrate-witness primitive that the literature hasn't solved.

### Cross-cutting trade-off mapped against the literature

**Closed vs open grammar at the predicate layer:**

| System | Verification language | Predicate types | Verdict on antigen's choice |
|---|---|---|---|
| in-toto | Closed (artifact rules) | **Open** (predicateType URI) | Counter-example showing the cost of open |
| SLSA | (no language; level-as-state) | Closed (one provenance schema) | Aligned posture |
| Sigstore | (no policy layer) | N/A | N/A |
| TUF | Closed (metadata schema) | Closed | Aligned posture |
| DSSE | N/A | Closed (payload-type binding) | Aligned posture |
| PASETO | N/A | Closed (version-as-algorithm) | **The lesson antigen is repeating** |
| cargo-deny | Closed (per-check TOML schema) | Closed | Aligned posture |
| cargo-vet | Closed (audit-entry schema) | **Open** (custom criteria) | Counter-example; works because criteria are human-evaluated |
| OPA / Rego | **Open** (Datalog-derived) | **Open** | The path antigen explicitly didn't take |
| Salsa | Closed (Rust function signatures) | Closed | Different problem |
| CODEOWNERS | Closed (path-glob) | N/A | Aligned posture |
| Renovate | Closed (TOML/JSON config) | Closed | Aligned posture |
| GHA suppression | Closed (SARIF) | Closed | Aligned posture |
| GUAC | (graph query, GraphQL) | Open at ingestion | Different layer |

**The verdict from the landscape**: closed-grammar-at-the-evaluator-layer is the dominant pattern. Open-grammar systems (Rego, in-toto's predicateType, cargo-vet's custom criteria) all rely on either *separate evaluators per predicate type* (in-toto) or *human evaluation* (cargo-vet) or *Datalog's decidability guarantees* (Rego). Antigen's choice (closed combinator grammar with sealed-but-extensible leaf set) is in the closed-camp mainstream.

**The differentiation worth foregrounding**: antigen's grammar is closed at *both* the combinator layer (only `all_of`/`any_of`/`not`) *and* the leaf layer (sealed enum of substrate-readers). Most systems are closed at one layer and open at the other. The double-closure is what enables tier-honesty at the audit layer — without it, the audit can't make calibrated claims about "what was verified."

### Tier-honesty / verification-strength reporting — the antigen-specific differentiation

**Survey of how existing systems report verification strength:**

| System | Reporting model | Gradient? |
|---|---|---|
| in-toto | Binary (pass/fail) | No |
| SLSA | Level (L1/L2/L3) | Yes, but per-pipeline not per-witness |
| Sigstore | Binary (signed + log-included or not) | No |
| TUF | Per-role-threshold-met (effectively binary per role) | No |
| DSSE | Binary (signature verifies or not) | No |
| cargo-deny | Per-check binary + severity threshold | Severity is reportable |
| cargo-vet | Per-criterion implication chain | Implication chain is the closest thing |
| OPA | Decision-as-value (allow/deny/structured) | Custom per-policy |
| GHA suppression | Suppressed + reason structured enum | Reason is reportable |

**Antigen's `WitnessTier × AuditHint` matrix is genuinely novel.** No existing system maps "depth of verification work performed" × "kind of evidence produced" as orthogonal axes the way antigen does. The closest is SLSA-level × VSA-policy-name, but those operate at the pipeline level and aren't structurally orthogonal.

This is also where antigen has the most to teach the broader ecosystem. The ADR-005 Amendment 3 discipline (audit reports lower bound of verification work, never upper bound) is a contribution back to the supply-chain-attestation conversation if antigen ever publishes the structural memory it's accumulating.

### Code-locality vs central-registry — antigen aligned with a minority

**Survey of locality choices:**

| System | Substrate location |
|---|---|
| in-toto | Hybrid (link files local, layout central) |
| SLSA | Central (build platform emits; consumer reads via registry) |
| Sigstore | Central (Rekor log) |
| TUF | Central (server-side metadata) |
| cargo-deny | Local (deny.toml in-tree) |
| cargo-vet | Local (audits.toml in-tree) + import from remote |
| CODEOWNERS | Local (in-tree file) + forge-enforced |
| Dependabot | Forge-side (alerts in repo settings) + local (.github files) |
| GHA suppression | Local (SARIF in PRs) + forge-side (dismissed-alerts) |
| GUAC | Central (database) |

**Antigen's pure-local choice** (`.attest/` sidecars in-tree, audit runs against the working tree) puts it in good company with cargo-deny, cargo-vet, CODEOWNERS-source-of-truth. The systems that go central (Sigstore, TUF, GUAC) all do so because their threat model is *external trust extension* — clients downloading things from untrusted sources need a tamper-evident server-side artifact. Antigen's threat model is *internal discipline maintenance*; the central-registry overhead doesn't pay off.

The v2 draft's "Per-cell antigen processing → code-locality" biology rhyme is supported by this landscape: the systems that need central are the ones bridging trust across boundaries; the systems that stay local are the ones operating within a single trust domain.

### Fingerprint pinning + stale detection — partial precedent, novel granularity

**Survey:**

| System | Stale detection mechanism |
|---|---|
| in-toto | Layout signatures + per-step link integrity |
| SLSA | Provenance is point-in-time per artifact-digest |
| Sigstore | Certificate expiration + Rekor inclusion time |
| TUF | Mandatory expiration + threshold re-sign cadence |
| cargo-deny | `unused-ignored-advisory` warning |
| cargo-vet | Per-version + delta-audit chain |
| Salsa | Red-green algorithm on input-value change |
| GitHub PR | Dismiss-stale-reviews-on-push (per-PR-diff) |

**The closest precedents are cargo-vet's delta-audits and GitHub's stale-review dismissal**, but both operate at coarser granularity (per-crate-version, per-PR-diff). Antigen's per-item-fingerprint pinning is structurally finer-grained.

The novel contribution is that antigen's stale-detection generates a *distinguishable audit state* (`discipline-substrate-stale` hint at Reachability tier) rather than just invalidating the attestation. This is the affinity-maturation rhyme operationalized: signatures that *would* pass against prior code, against drifted current code, downgrade to Reachability with a hint that re-attestation is the path back to Execution. No existing system does this with the same fidelity.

### Cross-artifact trust extension — antigen explicitly defers; ecosystem has answers when ready

**Survey:**

| System | Cross-artifact trust mechanism |
|---|---|
| in-toto | Sublayouts (nested federated trust) |
| SLSA | Verification Summary Attestations (VSAs) |
| TUF | Delegation hierarchy with terminating delegations |
| cargo-vet | Imports from other projects' audits.toml |
| GUAC | Graph-database aggregation across artifacts |

**Antigen's v0.1 stance is "deferred."** The `descended_from` propagation chain is the in-crate version; cross-crate trust extension is roadmap.md:151 territory. When antigen does ship this, the literature is rich:
- cargo-vet's imports pattern is the closest match (filesystem + git, no infrastructure)
- TUF's delegation hierarchy is the disciplined formal version
- in-toto's sublayouts are the verification-walkable version
- SLSA's VSA is the compressed-summary version
- GUAC's graph-database is the heavyweight version

The cargo-vet pattern (imports of named, signed audit sets) is the cleanest match for antigen's posture — distributed, in-tree, git-trust-by-default. When v0.2+ extends cross-crate ratification, importing from this pattern would be low-friction.

### What's genuinely novel about antigen that the literature hasn't solved

After surveying ~14 systems, three things stand out as *not present in the existing landscape*:

1. **Code-site-locality at item granularity.** Most systems anchor attestations to artifacts (binary blobs) or to dependency-graph nodes (crate versions). cargo-vet anchors to (crate, version). CODEOWNERS anchors to (file-glob). Antigen anchors to (crate, file, item_path, antigen_name) — finer than any existing system. The cost: more sidecars. The benefit: stale-detection can be fingerprint-pinned at the item level, generating the affinity-maturation discipline naturally rather than requiring per-PR-diff workflow tooling.
2. **Tier-honesty as a first-class verification-output property.** No existing system explicitly maps "the audit's report names the lower bound of work performed, never the upper bound" as a discipline. SLSA's level/VSA gets close but is per-pipeline; Sigstore/DSSE/TUF are binary; cargo-vet uses implication chains but doesn't formalize "audit reports what it actually verified." Antigen's `WitnessTier × AuditHint` is the original contribution.
3. **Substrate-witness-as-extension-of-existing-audit-vocabulary, not new-category.** The v2 draft's structural move ("witnesses currently check code-side substrate; extend them to check non-code substrate as well, as long as the audit remains tier-honest") is unique. Most systems either build a new attestation primitive (in-toto's predicate-types) or extend an existing one with new fields (cargo-vet's custom criteria). Antigen's move is to *preserve the witness primitive* and *change what substrate the witness reads*, with the tier-honesty discipline forcing the audit to honestly report what it learned. This is the recognition-not-design ratio at the architectural level.

The first is a structural choice cargo-vet could have made and didn't (because crate-version-granularity matches their problem). The second is genuinely original — the ADR-005 Am 3 discipline is what makes it work; the discipline itself is novel. The third is a design move that preserves an architectural invariant (witnesses are witnesses regardless of substrate) at the cost of accepting that substrate-witnesses cap at Execution rather than FormalProof — the v2 draft's biology-grounded argument for this cap is, to my knowledge, not present in the literature on attestation languages.

### What antigen should genuinely absorb (concrete list)

In rough order of "obviously useful for v0.1 or near":
1. **DSSE envelope when crypto-signing lands.** Don't roll your own; PAE is non-obvious.
2. **`unused-ignored-advisory` parallel** — `discipline-attestation-orphaned` hint for sidecars whose items vanished. (Mostly covered by `attest gc`; add the diagnostic.)
3. **cargo-vet's delta-audit primitive** — explicit `attest delta` command preserves chain semantics during code evolution. Avoids forcing full re-attest on every drift.
4. **cargo-vet's imports pattern as the cross-crate-ratification prior art** when v0.2+ ships it.
5. **SARIF export adapter** for IDE/CI dashboard integration. v0.3+.
6. **TUF's threshold-signatures** as the `signers(required_threshold = K, candidates = [...])` v0.2 amendment.
7. **CODEOWNERS-style role resolution** as the v0.2 `required_role` interop (already in v2 draft as T2).
8. **Sigstore identity-bound signatures** as the v0.3+ `Signer.signature` activation path.
9. **PASETO version-discipline pattern** — consider versioning the combinator grammar itself, not just the schema.

### What antigen should NOT absorb (concrete list)

1. **OPA/Rego expressiveness.** Closed combinator grammar is the right posture; Rego-grade expressiveness would weaken tier-honesty.
2. **in-toto's open `predicateType` registry pattern.** Closed leaf-primitive enum is the antigen choice; making it open would invite the "every verifier needs to ship code for every predicate type" tax.
3. **GUAC's graph-database deployment model.** Database-less is the right v0.1 posture; cross-workspace aggregation can be filesystem-based (cargo-vet imports pattern).
4. **npm-audit's missing-dismissal pattern.** The gap is widely complained about. Antigen's "predicate-fail = predicate-fail; no dismissal primitive" is the stricter discipline; preserve it.
5. **CODEOWNERS's forge-coupling.** Antigen runs locally; integrating CODEOWNERS-as-input is fine, but the enforcement layer stays in `cargo antigen audit`, not in any forge.
6. **SLSA-level conflation of pipeline-hardness with output-correctness.** Antigen's tier is about verification-work-performed, not about the pipeline's integrity. Don't merge axes.

### Honest gaps in my research

Places I made structural inferences rather than citing substrate:
- I did not chase down Renovate's full source-locality / dashboard-state schema; the configuration-options page is dense and I sampled.
- I did not verify GUAC's claimed normalization mechanics; the project's primary docs gloss this and the implementation would need direct code reading.
- I did not consult primary in-toto layout examples (only spec docs); a real layout's expressiveness would sharpen the "what antigen lacks vs in-toto" claim.
- The PASETO lesson is structural-by-analogy, not domain-direct; treat as illustration rather than direct prior-art.
- I assumed but did not verify that cargo-vet's adoption-at-scale claim holds (Mozilla and Google both use it, but the deployment maturity story would need primary sourcing).
- The OPA/Rego "decidability over Turing-completeness" framing is the standard one; I did not verify the formal decidability proofs that justify the claim.

If any of these become load-bearing for an antigen design decision, sharpen the substrate before relying on the inference.

---

## What this capture is for

- **Team-launch context**: when the academic-researcher role agent runs against v2, this capture is the seed material — attack the frontier, don't rediscover known landscape. The closed-vs-open grammar trade-off, the tier-honesty novelty, the code-locality posture, and the cross-artifact trust deferral are all already mapped; team-academic-researcher should attack what's missing, what I sampled too shallowly, and where the landscape is moving since I last looked.
- **Aristotle Phase 1-8 anchor**: when team-aristotle deconstructs v2's load-bearing claims, the landscape-comparison section names which claims have ecosystem-precedent backing (good for confidence in the claim) and which are genuinely novel (good for caution about untested assumptions).
- **Naturalist cross-domain anchor**: the biology rhymes in v2 draft find structural matches in the literature too (affinity-maturation rhymes with cargo-vet delta-audits; germinal-center-locality rhymes with cargo-vet in-tree storage; threshold-signatures rhyme with TUF's k-of-n primitive). Naturalist may want to verify whether the metaphor predicts these matches or merely accommodates them post-hoc.
- **ADR substrate**: when ADR-019 drafts cite "compose-don't-compete with existing ecosystem tools," this capture is the literature backing that claim. Concrete tools to cite: cargo-vet (audit pattern), cargo-deny (declarative TOML), CODEOWNERS (role-resolution), DSSE (signature envelope when crypto lands), Sigstore (keyless identity when crypto lands).
- **Future post-mortem**: if antigen ships and adoption struggles, this capture shows what the design was *vs* the landscape, so reasoning about "did we differentiate correctly" can happen against fixed substrate.

READY FOR REVIEW
