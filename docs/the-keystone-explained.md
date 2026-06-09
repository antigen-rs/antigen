# Why the keystone works — the Learning-Core from first principles

> The [concepts](concepts.md) page tells you *what* the Learning-Core loop is.
> [`the-felt-arc.md`](the-felt-arc.md) tells you what it *feels like* to use. This page
> answers the harder question: *why is it trustworthy?* — stripped to irreducible
> truths, grounded in the code itself, not the design prose.
>
> Four questions, each stripped to the primitive underneath:
> 1. Why must the generator (C) route through the self-tolerance gate (B)?
> 2. Why was "non-empty corpus" an incomplete proxy for safety?
> 3. Why does claim-scope-as-a-type close the over-claim?
> 4. Why does the loop close *safely* — and what is the honest scope of "safely"?

The Learning-Core is the one organ in antigen that *generates* failure-class memory
instead of applying it. Generating knowledge is exactly where a tool can claim more
than it proved. So the keystone's whole job is to generate *and never over-claim while
doing it*. Here is why it succeeds, and exactly where its guarantees stop.

---

## 1. Why C must route through B

The tempting framing — the one to strip away — is: *a learner generates candidates; a
separate validation step filters them for quality.* On that view B is a precision
booster, nice to have. **That framing is wrong, and the truth inverts it.**

The irreducible fact: **a failure-class generalizer's natural failure mode is to
manufacture its own false positive.** Anti-unification generalizes by dropping what
differs across a cluster and keeping what's shared. Take a panic-in-`Drop` cluster:

```rust
impl Drop for GuardA { fn drop(&mut self) { let _ = flush(self.h).take().unwrap(); } }   // panics
impl Drop for GuardB { fn drop(&mut self) { let _ = flush(self.h).take().expect("x"); } } // panics
impl Drop for CleanGuard { fn drop(&mut self) { let _ = flush(self.h).take().ok(); } }    // clean
```

All three share `.take()`; the two defects differ in their panic source (`unwrap` /
`expect`); the clean one differs in calling neither. The naive generalization drops
*all* the differing leaves and keeps only the shared skeleton: **"any `Drop` impl that
calls `take`."** That draft matches `CleanGuard` — clean code.
The generator, doing exactly what generalization *means*, produced a fingerprint that
flags the innocent — the naive draft binds the clean sibling (`autoimmunity_safety_gate.rs`
test A1 asserts exactly this, so you can run it yourself).

So B — the self-tolerance gate — is not bolted onto C to improve precision. **B is the
only thing standing between C's mechanism and self-inflicted autoimmunity** (antigen
flagging its own clean code). That is why their coupling is a *safety* constraint, not
a sequencing preference: ship C without B and it doesn't degrade gracefully — it
actively floods the codebase with false positives.

### "But a smarter generalizer spares the clean sibling — doesn't that retire B?"

It is a fair objection, and antigen *does* use a smarter generalizer. PROPOSE
anti-unifies **to disjunction**: the differing leaves become
`any_of([expect, unwrap])`, which is `NoMatch` on `CleanGuard` (it calls neither). The
draft becomes (arms in the matcher's deterministic order)

```text
all_of([item = impl, impl_of_trait("Drop"), body_calls("take"),
        any_of([body_calls("expect"), body_calls("unwrap")])])
```

and it spares the clean sibling *without B even running*. So is B retired?

**No** — and this is the load-bearing point. Anti-unify-to-disjunction *reduces*
autoimmunity; it does not *eliminate* it. If a disjunction arm happens to be a call the
clean code also makes, the draft binds clean despite being a disjunction. The
proof is decisive (`autoimmunity_safety_gate.rs` test F): a homogeneous cluster where a
clean `Drop` sibling *also* calls `.expect()` (on a safe value) produces a draft whose
`any_of([unwrap, expect])` binds the clean sibling through the shared `expect` arm. The
raw generalization is genuinely autoimmune — and `propose()` (which routes through B)
returns `None`, pruning it.

The residue a smart generalizer can't reach is **irreducible to cleverness.** Whether a
draft over-binds *this* codebase's clean code is a fact about *this codebase*, not about
the draft's shape — so it can only be settled by checking the draft against real clean
code. That is the bedrock reason `C ══ B` is non-negotiable.

### How the line is enforced — type, not convention

The code makes this structural. `propose()` (`learn/propose.rs`) is the **only**
function that returns a *promotable* fingerprint, and it routes every draft through the
gate:

```rust
pub fn propose(cluster: &[syn::Item], clean_corpus: &[syn::Item]) -> Option<Fingerprint> {
    let draft = anti_unify(cluster)?;            // a HYPOTHESIS — not promotable
    self_tolerance::promote_if_safe(draft, clean_corpus)  // the only gate to "promotable"
}
```

The raw `anti_unify` *is* exposed — but it is explicitly labeled a **hypothesis** (for
inspection), and it returns a bare draft, not a promotable one. There is no code path
that yields a promoted fingerprint without B passing. You cannot bypass the gate,
because there is nothing to bypass *to*.

---

## 2. Why "non-empty corpus" was an incomplete proxy

This is the sharpest first-principles question in the keystone, and worth a careful
answer because it shows antigen's own discipline catching antigen's own code.

Start from what B is *for*. B's promise is: *the promoted draft spares known-clean
code.* For that promise to **mean** anything, B has to have actually checked the draft
against clean code it *could have flagged*. The hazard is a B that reports "spared!"
without doing real work.

The first vacuity is obvious once named: `spare_clean(draft, &[])` is **vacuously
true** — an empty corpus has no clean item to bind, so "spares every clean item" holds
trivially. Promoting against an empty corpus is autoimmunity *with a green check*: B
verified nothing and reported safe. The gate refuses this structurally:

```rust
pub fn promote_if_safe(draft: Fingerprint, clean_corpus: &[syn::Item]) -> Option<Fingerprint> {
    if clean_corpus.is_empty() {
        return None; // "cannot certify safety against nothing"
    }
    if spare_clean(&draft, clean_corpus) { Some(draft) } else { None }
}
```

Note *where* the refusal lives: in the promotion **authority** (`promote_if_safe`), not
the **predicate** (`spare_clean`). The predicate stays honestly vacuously-true — it
reports a literal fact about an empty corpus. The authority that grants promotion is
what declines to act on a vacuous pass. That separation is itself a claim-scope move:
the predicate never lies about the empty case; the authority refuses to *trust* it.

But here is the deconstruction the proxy invites — and which a fresh reviewer caught in
antigen's own gate. Strip `is_empty()` down to what it *stands for*. The real
precondition is not "the corpus has ≥1 item." It is **"the corpus has ≥1 item the draft
could have bound."** A non-empty corpus full of items the draft cannot possibly match is
*also* vacuous: B iterates, finds no match — not because the draft is safe, but because
nothing in the corpus is in the draft's reach — and reports "spared!" The green check is
just as empty as the `&[]` case, and `is_empty()` does not catch it.

This is a textbook proxy-drift in antigen's own vocabulary: the implicit assumption
"non-empty ⟹ real test" was never made explicit and checked, so it drifted from the
property it proxied (`empty ⊊ vacuous`). The non-drifting form is to check the property
directly — **corpus-bindability**: at least one corpus item must be in the draft's
match-domain for B's pass to carry information. That hardening is the named precondition
for ever wiring the learner into production (see the roadmap). It is
*latent* today only because the learner has zero production callers — nothing wires
`propose()` yet, so no vacuous-pass can ship.

The lesson generalizes: **a proxy is safe exactly to the extent it cannot drift from
the property it stands for.** That antigen's own safety gate carried a proxy-drift — and
that a fresh pair of eyes found it — is not an embarrassment. It is the strongest
demonstration the discipline works: antigen catches this class in others; here it caught
it in itself.

---

## 3. Why claim-scope-as-a-type closes the over-claim

The framing to strip: *label your outputs honestly — say "match," not "verdict."* That
is a *discipline*, and disciplines drift. Someday someone renders a match as a verdict
because it's convenient, and the honest label quietly becomes a dishonest one.

The irreducible move is sharper: **make the over-claim unrepresentable.**

A scan match and an audited verdict are distinct variants of one sum type
(`finding.rs`, `FindingBody`):

```rust
pub enum FindingBody {
    // ...
    DialVerdict      { class: String, tier: DialTier },  // audit-time: a graded verdict
    FingerprintMatch { class: String, tier: DialTier },  // scan-time: a structural match
}
```

They carry the same fields, but they are **different shapes** — a `FingerprintMatch` is
not a `DialVerdict`, and no coercion turns one into the other. The scan stage emits
`FingerprintMatch`. The audit stage — the only stage that sees a site's
`#[defended_by]` / witness half — is the only producer of `DialVerdict`. So "this scan
match is an audited verdict" is not a false statement the code is *disciplined* not to
make; it is a value the type system *cannot construct* on the scan path. The machine can
state what it matched; it is structurally unable to state that it ratified.

Why this is the right closure and not over-engineering: it answers *where Goodhart
closes.* Antigen's catalog admission is deliberately permissive — you declare a class by
articulating it — because the protection was never at *entry*; it is at *labeling*. A
forged claim can wear `Heuristic`/`Imagined` provenance forever, but it can never wear
`Constructable`/`Encountered` without a real demonstration. The sum-type split is that
same *Goodhart-closes-at-labeling* principle realized in the type system: the honest
scope **is** the type, so you cannot game the label by being convenient with prose. The
human-readable shadow — *"a fingerprint match to inspect, not an audited verdict"* — is
what the editor render carries per-diagnostic; but the *guarantee* lives in the type,
not the sentence.

This generalizes to the whole **claim-scope honesty** discipline (see the
[glossary](glossary.md)). Every capability splits into:

- a **syntactic half** — decidable, machine-tractable: match a fingerprint, anti-unify
  a cluster, set-diff two commits' digests. antigen does this and reports a *fact* at a
  calibrated tier;
- a **semantic half** — *is this matched site a real defect? does this draft name a real
  class? was the removed guard required?* By **Rice's theorem** (1953) these
  non-trivial semantic properties of programs are undecidable. antigen does not — cannot
  — do this half; a human, a CI context, or an incident **ratifies**.

The syntactic/semantic line is exactly the decidable/undecidable line, which is exactly
where the machine must stop asserting and start labeling. The human-in-the-loop on the
semantic half is not UX caution; it is *computability-forced*. Antigen is
sound-without-completeness in the sense of Cousot & Cousot (1977): it never claims more
than it demonstrated. The sum-type split is that line *drawn in the type system* for the
scan/audit boundary.

> **A naming note.** This epistemic discipline is **claim-scope honesty** — never
> "frontier honesty." "Frontier" is reserved for the *spatial* coverage-frontier (what
> the scan did not reach). The two rhyme but are orthogonal: coverage-frontier = *what
> we didn't inspect*; claim-scope honesty = *don't over-claim what we DID inspect*.

---

## 4. Why the loop closes safely — and the honest scope of "safely"

The loop — cluster → propose → test (promote / prune), with self-tolerance holding it —
closes safely because every arc that *could* over-claim is gated by an arc that can only
report what it proved:

| arc | how it could over-claim | what gates it |
|---|---|---|
| cluster → propose | over-generalize to bind clean code | B (the spare-clean gate) |
| propose → promote | promote an over-binder | `promote_if_safe` refuses it (`autoimmunity_safety_gate.rs`) |
| promote → "a named class" | the machine can't do this at all | the human/incident ratifier (the semantic half) |
| the loop on antigen's own marks | claim a proof it doesn't have | the falsification gate (`learn_dogfood_propose.rs`) |

That is the case *for* "safely." Now the part a first-principles account must not let
slide — **the honest scope of the word "safely":**

1. **The safety property is proven for the *tested* scope.** Every case in
   `autoimmunity_safety_gate.rs` passes, including the decisive
   no-bypass proof (`anti_unify`'s own autoimmune output is pruned by `propose`) and
   the empty-corpus refusal.
2. **The empty-corpus refusal is complete for the empty case, incomplete in general**
   (§2): a non-empty corpus the draft *cannot bind* is still a vacuous-pass hole. Latent
   today (the learner is un-wired), real as the named precondition for wiring it.
3. **The falsification gate proves the *mechanism*, but the current dogfood draft
   over-fits.** Antigen's two genuinely-felt `#[dread]` twins are abstractly similar but
   concretely near-identical, so they anti-unify to a near-photocopy (no `any_of`
   disjunction) rather than a class *generalization*; the non-degeneracy check is
   one-sided (it confirms a shared signal is present, not that the draft generalizes).
   A recall increment, located honestly, not a hidden flaw.
4. **"Promote = `propose()` only" is enforced by routing, not yet by a newtype.** Both
   `anti_unify` and `promote_if_safe` return a bare `Fingerprint`; the type-enforced form
   (a `PromotedFingerprint` constructible only by the gate) is the hardening for when the
   learner is wired into a render.

So the loop closes *safely* in this precise sense: **the
learner is a library with zero production callers, the autoimmunity gate holds against
every tested over-reach, and the remaining holes are latent, located, and named as
preconditions for the next wave.** That is the whole claim, and no more.

Notice what the fourth section just did: a doc *about* claim-scope honesty **scoped
itself** — it named its own four limits rather than asserting an unqualified "the loop is
safe." That is not modesty for its own sake. A doc that taught claim-scope honesty while
over-claiming its subject would anti-teach. The discipline the keystone embodies is the
discipline this page had to obey to describe it.

---

## What this means for you, the adopter

- The Learning-Core is a **library** (`antigen::learn`) — there is **no**
  `cargo antigen propose` command. What ships is the *safety-governed learner*, not a
  user-facing verb. Don't go hunting for a command that doesn't exist.
- A bundled-catalog scan match is a **fact to inspect, not a verdict** — by construction
  (§3), not by politeness. Reading a verdict honestly is its own page:
  [`reading-a-verdict.md`](reading-a-verdict.md).
- The same gate that spares antigen's own clean sibling is the gate that will protect
  *your* clean code when the learner is wired in. The safety floor was built first, on
  antigen's own marks, before any user-facing learning surface — which is exactly the
  order a tool that takes autoimmunity seriously has to build in.

## Where to go next

- [`concepts.md`](concepts.md) — the Learning-Core loop and claim-scope honesty as
  *concepts* (this page is the *why* beneath them).
- [`the-felt-arc.md`](the-felt-arc.md) — what the loop *feels like*, walked on antigen's
  own marks.
- [`reading-a-verdict.md`](reading-a-verdict.md) — how to read a scan match vs an audited
  verdict.
- [`glossary.md`](glossary.md) — the anchored vocabulary (claim-scope honesty,
  coverage-frontier, marked-unknown).
