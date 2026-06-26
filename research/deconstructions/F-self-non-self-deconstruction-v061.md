# F-self/non-self — first-principles deconstruction (v0.6.1, tip 8b2e040)

Outward+inward. Workspace R:/antigen-061-self-non-self READ-ONLY.

## Irreducible atoms of "self" here

1. **antigen has TWO orthogonal "self" notions that are never unified — and that's the deepest finding.**
   - **Identity-self** = the *structural digest* (`antigen-fingerprint/src/digest.rs:1` `structural_digest`): a content hash that answers "is this the same item I signed against, or did it drift?" Self = "the code as it was when last attested." Non-self = drift. This is the *attestation* self.
   - **Tolerance-self** = the **clean corpus** (`antigen/src/learn/self_tolerance.rs:1` — literally "antigen's thymus"). Self = "every item in a known-clean corpus that a draft must SPARE." Non-self = a defect-shape that binds. This is the *learning* self.
   These two never touch. The attestation-self is per-item ("did THIS drift from its signed form"); the tolerance-self is per-corpus ("does this draft over-bind the clean family"). Both are real, both are load-bearing, neither is primitive — both are *derived from a third thing they don't name*: **captured intent**.

2. **The *why* (intent) already exists, three times, but only ever as an opaque LEAF PAYLOAD — never as a structural primitive.**
   - `SignerBasis::Fresh{reasoning: Option<String>}` (schema.rs:528) — "what the signer checked," optional, free text, *not enforced as a predicate*.
   - `SignerBasis::DeltaFrom{rationale: String}` (schema.rs:542) — required-non-empty, but anti-laundering machinery treats it as a *quality gate on a string*, not a claim the audit reasons over.
   - `LifeEvent::Ratified{why: String}` (life_record.rs:134) — the "leaf-payload exception" (ADR-020): free human text AT a typed leaf, "rendered but NEVER parsed."
   Intent is everywhere as decoration and nowhere as structure. The codebase has a *standing doctrine* (ADR-020 leaf-payload exception) that intent MUST stay opaque — that is the unexamined assumption.

3. **The one place intent IS structural is `LifeEvent`'s typed event-stream — and it works.** `life_record.rs:99` `LifeEvent` + `Trend` (life_record.rs:63) + `check_story_coherence` (life_record.rs:351): a hand-authored *typed* directional claim (`Trend::Improving`) is structurally re-validated against the *derived* trajectory. This is the existence-proof: when antigen makes a claim TYPED, it can witness story-vs-struct drift "by construction." The `why` text on the same event stays opaque — so the record carries BOTH a typed-intent (the Trend) and an opaque-intent (the why), side by side, and only reasons over the typed half.

4. **Negative selection is the immunological self-model done right, but it operates on the wrong "self."** `promote_if_safe` (self_tolerance.rs:513) rejects a draft that binds clean code = autoimmunity. But "clean" is asserted by the operator (corpus-bounded, self_tolerance.rs:26), NOT derived from what the code was MEANT to be. The thymus presents a corpus, not an intent. AIRE's analog — "present the self-antigens so the system tolerates them" — is HALF-built: the presentation surface (clean corpus) exists; the *self-antigen* (captured intent = what each item was meant to be) does not.

5. **The discriminator's keystone cell silently assumes intent.** `discriminator.rs:26` — "a class whose shape is GONE but still carries a live WITNESS is WELL-DEFENDED, not obsolete — the witness is the plausible REASON the shape is gone." That "plausible reason" IS an intent inference, made structurally invisible: the verdict reads a `defended: bool`, but the *meaning* ("the guard held, on purpose") is exactly the captured-why that lives nowhere typed. The whole obsolete/well-defended split is an intent-judgment wearing a boolean.

## The intent-as-self seed, located
- Opaque-but-present: `SignerBasis::Fresh.reasoning` (schema.rs:533), `DeltaFrom.rationale` (schema.rs:568), `LifeEvent::Ratified.why` (life_record.rs:136).
- Typed-and-working (the template): `LifeEvent::Narrated{claimed: Trend}` + `check_story_coherence` (life_record.rs:120, 351).
- Identity-self: `structural_digest` (digest.rs), `current_fingerprint`/`signed_against_fingerprint` (schema.rs:98, 487).
- Tolerance-self: clean corpus + `promote_if_safe` (self_tolerance.rs).

## 0.6.1 candidates
- **C1 — Type the intent the way `Trend` is typed.** Add a typed `IntentClaim` leaf next to the opaque `reasoning`/`rationale`/`why` (NOT replacing it — leaf-payload exception keeps the prose). Smallest real version: a typed `expected: Trend`/`expected_disposition` on a Fresh attestation, re-validated against the item's own life-record trajectory exactly as `check_story_coherence` already does. Reuses the proven shape; makes intent witnessable-by-construction at the attestation site, not just the narration site. Feeds learning-core: the discriminator's "plausible reason" becomes a typed claim it can check.
- **C2 — Name the two selves and the missing bridge.** A short doc/ADR-draft note that attestation-self (digest-drift) and tolerance-self (clean-corpus) are two projections of one captured-intent, currently un-unified. This is the structural finding that unblocks treating intent as the primitive both derive from.
- **C3 — Make the clean corpus an intent-presentation, not an operator assertion.** Seed: a clean-item's *attested why* (it's clean *because* X) is the self-antigen AIRE presents. The corpus stops being a bare `&[syn::Item]` the operator vouches for and becomes items-carrying-their-intent — a stronger thymus.

## Unique angle (the ONE thing)
**Self/non-self is NOT the primitive — it's a derived projection of captured intent, and antigen already proved it can type intent (the `Trend`/coherence machinery) but installed a DOCTRINE (ADR-020 leaf-payload exception) that forbids doing so anywhere else.** Everyone else will look for where to ADD an intent field. First-principles sees that the intent field exists three times already, working code to type it exists once already, and the real move is to LIFT THE SELF-IMPOSED CEILING that keeps intent opaque everywhere except the one narration site. The non-self the system flags (drift, autoimmune-bind) is always "deviation from intent" — but intent is the thing the architecture is structurally committed to NOT representing. Flip that one assumption and self/non-self stops being a primitive and becomes `deviation(captured_intent)`, with both existing selves as special cases (digest-drift = deviation from intent-at-signing; autoimmune-bind = deviation from intent-of-clean).

## Phase-8 void (forced rejection)
If intent CANNOT be typed (ADR-020 holds absolutely) — then antigen can never have a true self, only proxies (digest, corpus). The void's shape: every self-notion in the codebase is a *proxy measurement* of an unrepresented intent. The proxies' very multiplicity (two unconnected selves + three opaque why-fields + one typed Trend) is the hint that the real referent — captured intent — exerts force everywhere while being named nowhere. That force IS the missing primitive.

## Waking notes
Next: pull thread on whether `IntentClaim` (C1) should be a new `LifeEvent` variant or a `SignerBasis` field — the `check_story_coherence` reuse argues for life-record-side. Untested surface: how `propose.rs` / `maturation.rs` would consume a typed intent. Route C1/C2 to navigator for the build voyage.
