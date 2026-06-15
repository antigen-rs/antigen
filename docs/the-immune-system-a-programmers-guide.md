# The Immune System: A Programmer's Guide to Antigen

> A narrative course, not a table. We start with one pathogen crossing one
> membrane, and build the whole immune system one mechanism at a time — and at
> each step, the moment the biology makes a primitive *inevitable*, antigen's
> version of it lands. By the end you'll understand both: real immunology, and why
> antigen is shaped the way it is. The claim of [ADR-003](decisions.md) is that the
> biological metaphor is **load-bearing, not decorative** — that it *predicts* what
> antigen needs. This guide is the argument for that claim, told as a story.

---

## Before we start: the asymmetry that makes a defense system necessary

A vertebrate body cannot pre-enumerate the threats it will face. It will meet
pathogens that did not exist when it was born. It cannot inspect every molecule
that crosses its skin — generation of threats outpaces any centralized inspection.
So evolution did not build a *checklist*. It built a **recognition-and-memory
system** that surfaces itself: structure that knows what danger looks like, holds
the lesson of every past encounter, and flags new instances against that memory
without anyone having to remember to look.

Software is now in the same bind. Code is generated faster than it can be read —
by humans in AI-pair workflows, by agents that lose context across sessions, by
teams that ship faster than anyone inspects. The historical assumption that "the
team has read everything" stopped being true years ago. (See [the README](../README.md)
for the full framing of this asymmetry.) So antigen makes the same move biology
made: not a checklist, but **structural memory that surfaces itself.**

That parallel is the whole reason the metaphor is load-bearing. Two systems facing
the same problem — recognition under an inspection deficit — converge on the same
architecture. So when biology has a mechanism, it is worth asking: *what is
antigen's version?* Often, the question answers itself. Let's walk it.

---

## Chapter 1 — A pathogen crosses the membrane (recognition)

A bacterium drifts up against a cell. Before any antibody, before any memory, the
body's **innate** immune system acts. Sentinel cells carry **Pattern Recognition
Receptors** — receptors shaped to match the molecular silhouettes that *pathogens
in general* share (the sugar coats of bacteria, the double-stranded RNA of viruses).
The receptor doesn't know *this* bacterium. It knows the *shape that bacteria share*.

Here is the first inevitability. If you want to recognize a *class* of danger
rather than a specific instance, you need a **structural pattern matcher** — a
thing that fires on a shape, not an identity.

> **Antigen's version lands here:** `cargo antigen scan` walks your code's AST and
> fires on **structural fingerprints** — `body_calls("transmute")`,
> `impl_of_trait("Drop")` — exactly the way a PRR fires on a molecular silhouette.
> The fingerprint recognizes the *shape* of a failure-class, not one specific bug.
> This is the innate layer: it runs without you marking anything, the way innate
> immunity runs without prior exposure.

But innate recognition is coarse. It tells you *something bacterial is here*, not
*how dangerous, how sure, defend or tolerate*. The body needs more, and so does the
tool. Onward.

---

## Chapter 2 — The cell shows what it found (presentation)

A cell that has taken something in does a remarkable thing: it chops the intruder
into fragments and **displays one on its surface**, held in a groove called the
**MHC molecule**, like a sentry holding up a captured banner: *this is what I found
inside me.* This is **antigen presentation** — and it is the literal origin of the
word this whole tool is named for. (The biological *antigen* is the fragment that
gets presented; see the [glossary](glossary.md).)

The inevitability: recognition is private until it's **shown**. A defense system
needs a way for a site to declare *I am presenting this shape* — to make the
internal recognition visible to the rest of the system.

> **Antigen's version:** `#[presents(SomeFailureClass)]`. You put it on a code site
> to declare "this site is in this failure-class's territory" — it is holding up
> the banner. Crucially, presenting is **not** the same as being infected: a cell
> presenting a fragment isn't necessarily sick, and a site marked `#[presents]`
> isn't necessarily vulnerable. *Present ≠ compromised.* This distinction will
> matter enormously later (it's why a *safe* code site can still carry
> `#[presents]` — it's declaring "I'm in this territory," not "I'm a bug").

---

## Chapter 3 — The body remembers (B-cell memory)

The first time you meet a pathogen, the adaptive response is slow — days to mount.
But the body keeps **memory B-cells** afterward, each holding the recognition
pattern for that specific threat. The second encounter is fast, because the memory
is already there. This is why vaccination works, and why you get chickenpox once.
**The lesson of the encounter persists as structure.**

This is antigen's deepest inevitability — its entire reason to exist. When you fix
a bug, you learn something about *why a class of code fails*. That lesson lives in
your head, a commit message, a Slack thread, a docstring that drifts. None of those
are drift-resistant. Six months later, structurally identical code appears in
another module, and the lesson is gone.

> **Antigen's version:** `#[antigen(name = "...", fingerprint = "...")]`. A
> declaration is a **memory B-cell**: it holds the recognition pattern for a
> failure-class as durable, checked structure. The lesson of the encounter survives
> the refactor, the session boundary, the personnel change. This is the move the
> whole tool is built around — B-cell memory → persistent failure-class
> declarations. [ADR-003](decisions.md) cites exactly this as the proof the
> metaphor predicts primitives: *biology had memory cells, so antigen has
> declarations.*

---

## Chapter 4 — The antibody, and what counts as proof (the witness)

Memory recognizes; it does not neutralize. The thing that actually *handles* a
pathogen is the **antibody** — a protein that binds the threat and marks it for
destruction. Memory says "I know this shape." The antibody is the demonstrated
response.

The inevitability here is subtle and it's where most tools go wrong. Recognizing a
danger is not the same as *defending against it*, and a defense system must not
confuse the two. You can't just *declare* yourself immune; immunity is something
that has to be **demonstrated** by an actual responding agent.

> **Antigen's version — and a hard-won correction:** the antibody is the
> **witness**. In antigen, a defense is `#[defended_by(X)]` on a *test* — a real,
> runnable thing that exercises the defense. Antigen calls the antibody a *witness*
> in its API (see the [glossary](glossary.md)). Early antigen had a `#[immune]`
> marker you stamped on a site to *declare* immunity — and that was a design error
> the biology would have warned against: it let a site claim "I'm immune" with no
> antibody to show for it. The correction (ADR-029) was *observe, don't declare* —
> immunity is **observed** from the evidence (a witness you can run), never claimed
> at the site. The biology was right: no antibody, no immunity.

> **Silence test (first sidebar).** Biology has a rich answer to "what counts as a
> response" — antibodies, killer T-cells, complement. But it is *silent* on "what
> counts as a *proof* of a response" — a cell doesn't grade its own antibodies on a
> rigor scale. That's a place the metaphor goes quiet, and it predicts where antigen
> had to invent beyond biology: the **witness-tier gradient** (FormalProof /
> Execution / Reachability / None — see [`witness-tiers.md`](witness-tiers.md)) is a
> software-native addition, because *software can ask "how strong is this proof?" in
> a way a body can't.* When the biology is quiet, antigen is on its own — and that's
> a signal, not a gap.

---

## Chapter 5 — Lineage (clonal expansion and inheritance)

When a B-cell finds its target, it doesn't stay one cell. It **expands into a
clone** — and the clones mutate slightly, competing to bind the threat better.
Lineage matters: the daughter cells inherit the parent's recognition, refined.

The inevitability: failure-classes have **families**. A use-after-free is a kind of
memory-unsafety; a specific deserialization-DoS is a kind of unbounded-input
problem. When you name a parent class, its structure should **propagate** to the
children.

> **Antigen's version:** `#[descended_from(Parent)]`. A child failure-class inherits
> the parent's presentations through the lineage — clonal expansion as a taxonomy.
> Name the family, then name the specific variants; the memory flows down the
> lineage. (With one discipline the biology also enforces: inheritance does **not**
> transitively grant immunity — each descendant must *re-attest* its own defense,
> because a daughter cell still has to make its own antibody. See
> [`decisions.md`](decisions.md), ADR-005 sub-clause F.)

---

## Chapter 6 — Tolerance (and why the immune system must NOT fire on self)

Here is the mechanism that separates a real immune system from a paranoid one. The
body is full of *self* — its own proteins, its own cells. An immune system that
fired on everything it recognized would destroy its host. So the body runs
**tolerance**: regulatory T-cells (**Tregs**) actively *suppress* responses against
self. Recognizing something is not a license to attack it.

The inevitability — and antigen learned this one the hard way, in public (see
[`war-stories/the-self-catch.md`](war-stories/the-self-catch.md)). A fingerprint
will sometimes fire on code that is *fine* — a safe sibling, a recommended idiom,
the very fix for the bug. The system needs a way to say "yes, this matches, and
it's deliberately accepted" without weakening the pattern.

> **Antigen's version:** `#[antigen_tolerance(X, rationale = "...")]` — peripheral
> tolerance for a legitimate match. It is the Treg: it suppresses the response on a
> reviewed, accepted site, *with a stated reason* (the rationale is required, the
> way tolerance in the body is specific, not blanket). And the deeper lesson Tregs
> teach shows up in antigen's confidence **tiers**: a `named` fingerprint promises
> high precision *precisely because* it has been checked not to fire on its own
> clean siblings. The day antigen's own fingerprints fired on safe code — on
> `bytemuck::zeroed`, on antigen's own `from_slice` — was an *autoimmune* episode,
> and the fix (ADR-039 §C Amendment 1) was a tolerance mechanism: prove a `named`
> pattern spares the safe namesake before you trust it. **The immune system that
> can't tolerate self is a disease; so is the linter that flags correct code.**

---

## Chapter 7 — Innate vs adaptive (the two surfaces)

Step back and the immune system has two layers working together. **Innate**
immunity is fast, general, always-on, and requires no prior marking — the PRRs from
Chapter 1. **Adaptive** immunity is specific, learned, and built from explicit
encounters — the memory cells and antibodies. Neither is sufficient alone; together
they cover the space.

> **Antigen's version:** the **passive surface** (fingerprint scan — innate; fires
> on shape with no marking) and the **active surface** (explicit markers like
> `#[presents]` / `#[defended_by]` — adaptive; built from your declarations). A
> healthy antigen deployment uses both: scan finds candidate shapes you never
> marked; your declarations carry the lessons you've explicitly learned. The whole
> [catalog](stdlib-families.md) is the *adaptive* memory antigen ships pre-loaded —
> a vaccination, in effect: pre-formed memory for failure-classes you haven't
> personally encountered yet.

---

## Chapter 8 — Affinity maturation (how the body generates *new* antibodies — and selects them against self)

Every chapter so far has been about *applying* recognition: a receptor matches a
shape, a memory cell persists it, an antibody demonstrates the response. But where
do *new* antibodies come from? When the body meets a threat it has no good
antibody for, it does not wait for a better one to arrive by luck. It **builds**
one. In the **germinal center**, activated B-cells divide rapidly and
**hypermutate** their receptor genes — generating a swarm of slightly-different
candidates. The variants that bind the threat better are selected and expand; the
rest die. The immune system runs a fast local search over receptor-space.

But generation is exactly where a defense system can turn on its host. A
hypermutated receptor can land on a *self* protein as easily as on the threat —
that is the natural failure mode of mutating receptors at random. So the germinal
center does not just select *for* binding the threat; it selects **against**
binding self. A newly-matured B-cell that gained self-reactivity is culled before
it ever leaves. Generating new recognition and screening it against self are not
two features — they are **one process**, because the first without the second is
autoimmunity. The screen is only as good as the self it is shown: the germinal
center reliably culls reactivity to *ubiquitous* self, while rare or
tissue-restricted self-antigens can escape and seed autoimmunity later. The
selection is corpus-bound — it spares the self it actually sees.

The inevitability: a failure-class memory that can only ever hold lessons a human
*typed in by hand* is bounded by human throughput — the very asymmetry
([Chapter 1](#chapter-1--a-pathogen-crosses-the-membrane-recognition)) antigen
exists to beat. The system has to be able to **propose a new failure-class** from
the structural evidence it already carries: cluster the sites that share a shape,
generalize the shape into a candidate fingerprint. But a generalizer's natural
failure mode is to over-generalize — to drop so much that the draft also matches
*clean* code. Proposing a new fingerprint and screening it against known-clean
code are **one process**, for exactly the body's reason: the first without the
second floods your codebase with false positives. That is antigen's autoimmunity.
And as in the body, the screen is corpus-bound: it can only spare the clean code
it is shown.

> **Antigen's version:** the **Learning-Core** (`antigen::learn`) — antigen's
> germinal center. It runs **cluster → propose → test (promote / prune)**:
>
> - **Propose (`propose()`)** *anti-unifies* a cluster of marked sites into a
>   draft fingerprint — the shared skeleton kept, the per-member differences folded
>   into an `any_of([...])` disjunction. The draft is a **hypothesis**, never an
>   auto-asserted class — the hypermutated candidate, not yet a licensed antibody.
> - **Select against self (`promote_if_safe`)** — the negative-selection step. A
>   draft is *promotable* only if it **spares a clean corpus**: known-good sibling
>   code the draft must not flag. A draft that binds clean code is the
>   self-reactive clone — it is **pruned**, never promoted. (And the gate refuses
>   an *empty* corpus outright — "cannot certify safety against nothing" — because
>   a screen that checked no self is not a screen.)
>
> This is the germinal center's lesson made structural: in the code, `propose()` is
> the **only** path that returns a *promotable* fingerprint, and it routes every
> draft through the spare-clean gate. There is no way to promote a draft without
> the self-screen passing — the coupling is **type-enforced**, not a convention you
> remember. The raw generalizer *is* exposed, but it is explicitly labeled a
> hypothesis for inspection: it hands back a bare draft, not a promotable one. You
> cannot bypass the screen, because there is nothing to bypass *to*. The guarantee
> is exactly as scoped as the body's: a promoted fingerprint spares the clean
> corpus it was *shown* — supply a representative corpus and the screen is
> meaningful; supply a thin one and clean code outside it can still be flagged,
> the same way the germinal center spares only the self it samples.
>
> And antigen built it in the biology's order — the **self-screen first**. The
> Learning-Core laid the negative-selection floor *before* any user-facing
> generation surface; only then did it gain its CLI verb, **`cargo antigen
> propose`** (see [`cli-reference.md`](cli-reference.md#propose)) — the
> *safety-governed* learner first, the generation surface on top of the screen,
> never the other way around. A defense system that takes autoimmunity seriously
> builds the self-screen before it builds the generator, the way the germinal
> center evolved selection-against-self alongside hypermutation, not after it. The full first-principles account of *why* the loop closes safely —
> and the honest scope of "safely" — is
> [`the-keystone-explained.md`](the-keystone-explained.md); you can watch the
> mechanism run on antigen's own marks in
> [`war-stories/learning-from-its-own-wounds.md`](war-stories/learning-from-its-own-wounds.md).

> **Run the screen yourself.** The negative-selection gate is exercised by
> `antigen/tests/autoimmunity_safety_gate.rs` (`cargo test --test
> autoimmunity_safety_gate -p antigen`). The decisive case is
> `propose_prunes_when_anti_unifys_own_disjunction_binds_clean`: the *generalizer's
> own output* is self-reactive (its `any_of` arm binds a clean sibling), and
> `propose()` — routing through the gate — prunes it. The generator cannot license
> its own false positive.

---

## Chapter 9 — When the system turns on itself (dysregulation)

A real immune system can go wrong in characteristic ways, and naming them is part
of understanding it. **Autoimmunity** is the system firing on self (Chapter 6's
tolerance failing). **Sepsis** is a response so dysregulated it harms the host more
than the threat. **Anaphylaxis** is an over-reaction to something harmless.

> **Antigen's version — with a naming lesson worth pausing on.** Antigen recognizes
> these dysregulation states, but it was careful about *how*. Autoimmunity is a
> **pathology**, not a discipline — so antigen does **not** ship a `#[autoimmune]`
> marker (a site-marker named for a disease would read backwards, as if you'd
> *want* to mark code autoimmune). Instead, autoimmunity surfaces as a **screen**:
> an audit pass that flags *fingerprints over-firing on their own clean siblings*.
> That's the correct shape: autoimmunity is something you *detect in the defense
> system itself*, not something you declare on a site.
>
> **What v0.4 actually shipped (claim-scope honesty):** the screen's *mechanism*
> ships in v0.4 as the **self-tolerance gate** — the library `antigen::learn::
> self_tolerance` (`spare_clean` / `promote_if_safe`), the negative-selection check
> that refuses to promote any learned fingerprint that binds a known-clean sibling
> (see [`concepts.md`](concepts.md), the Learning-Core loop). It is a **library
> gate**, not a `cargo antigen autoimmune-check` command — there is no such
> subcommand, the same library-not-command scope the Learning-Core ships
> under. (This naming call is itself in the [README](../README.md)'s biology table —
> and it's a small instance of the metaphor doing real work: the biology told us
> autoimmunity is a system-level pathology, so the tool surfaces it as a system-level
> screen, not a site-level marker.)

---

## Chapter 10 — The boundaries (mucosa) and the silence that predicts where antigen is young

Most pathogens don't enter through sterile tissue; they enter through **mucosal
surfaces** — the gut, the lungs — vast, busy boundaries where the outside meets the
inside. The body invests enormously in mucosal defense, because *that's where the
trust boundary actually is.* The gut wall's **tight junctions** (controlled by, in
antigen's running metaphor, the `deny_unknown_fields` discipline) decide what
crosses.

> **Antigen's version:** the mucosal-boundary family and its deepest tier, the
> [deserialization-trust-boundary](stdlib-families.md) family — because
> deserialization is *the* place untrusted bytes cross into typed-Rust land, the gut
> mucosa of a program. The biology pointed straight at it: the busiest trust surface
> in the body is the mucosa, so the busiest trust surface in a program (parsing
> untrusted input) is where antigen invests its deepest family.

> **Silence test (the where-antigen-is-young sidebar).** Run the metaphor to its edge
> and notice where biology goes *quiet*, because that's where antigen is young. Biology
> is dense on **sensing** (PRRs), **comparing** (self/non-self), and **acting**
> (antibodies, complement) — and antigen has primitives for each. But biology is
> comparatively *silent* on **routing policy** — the immune system doesn't have a
> "decision-maker" weighing which response to mount as a deliberate policy; it's
> distributed and emergent. That silence is informative: it predicts that antigen's
> **routing/orchestration** layer (which finding goes to whom, with what priority)
> is the *under-built* edge — and indeed, the marked-unknown markers' `severity`
> field is a reserved routing hint that nothing consumes yet. **Where the biology is
> quiet, antigen is young.** Reading the silence is how you read the
> [roadmap](roadmap.md).

---

## The map, now that you've walked it

You met each of these as a mechanism with a problem it solves; here they are as a
reference, but the reference only means something because you walked the story:

| Biology | The problem it solves | Antigen's primitive |
|---|---|---|
| Pattern Recognition Receptors | recognize a *class* of danger by shape | `cargo antigen scan` structural fingerprints |
| MHC presentation | make internal recognition visible | `#[presents(X)]` |
| B-cell memory | make the lesson of an encounter persist | `#[antigen(name=...)]` declarations |
| Antibody / witness | *demonstrate* a response, don't just claim it | `#[defended_by(X)]` on a test |
| Clonal expansion / lineage | propagate recognition to family variants | `#[descended_from(Parent)]` |
| Treg tolerance | don't fire on self | `#[antigen_tolerance]` + the tier discipline |
| Innate + adaptive | cover shape *and* learned specifics | passive scan + active markers |
| Affinity maturation / germinal center | generate *new* recognition, selected against self | the **Learning-Core** (`antigen::learn`, shipped as a library): `propose()` anti-unifies a cluster into a draft; `promote_if_safe` promotes only a draft that spares the clean corpus (the self-screen) |
| Dysregulation | name the ways the defense itself fails | the **self-tolerance gate** (`antigen::learn::self_tolerance`, shipped v0.4 as a library) detects autoimmunity — a fingerprint over-firing on clean siblings |
| Mucosa | invest defense where the trust boundary is | the deserialization-trust-boundary family |

The full forward-looking version of this map — every immune primitive antigen
*could* grow into as adoption surfaces real instances — lives in
[`immune-system-primitive-map.md`](internal/immune-system-primitive-map.md). That document is
*recognition substrate*: a catalog of what biology already has answers to that
antigen will eventually need answers to.

---

## Why this is the argument for ADR-003

[ADR-003](decisions.md) makes a strong claim: the biological metaphor is *load-
bearing* — it doesn't decorate the design, it *generates* it. This guide is the
evidence. At every chapter, the biology made a primitive **inevitable** before
antigen built it: biology had memory cells, so declarations were inevitable; biology
demonstrated responses rather than declaring them, so the witness was inevitable;
biology tolerated self, so the tolerance primitive and the precision-tier discipline
were inevitable; biology *generated* new antibodies and screened them against self in
one process, so a self-tolerant learner — generate-and-screen as a single gated step —
was inevitable, which is exactly why the learning core is the generator *and* its
negative-selection gate, type-coupled; biology defended its mucosa hardest, so the
deserialization family was inevitable.

And the silence tests show the metaphor's *integrity*: it doesn't claim to predict
everything. Where biology is quiet (routing policy), antigen is young — and the
metaphor is honest enough to tell you so. A metaphor that predicted *everything*
would be a metaphor you were forcing; one that predicts the built primitives and
goes quiet exactly where antigen is young is a metaphor that's *real*. That's the
difference between a decoration and a discovery framework. When the biology predicts
a primitive, [the project builds it](decisions.md). That's not a slogan. It's the
record this guide just walked you through.

---

## See also

- [`stdlib-families.md`](stdlib-families.md) — the shipped failure-class families,
  as organs in this immune system
- [`war-stories/the-self-catch.md`](war-stories/the-self-catch.md) — the autoimmune
  episode of Chapter 6, told in full: antigen's fingerprints firing on its own clean
  code, and the tolerance mechanism that fixed it
- [`immune-system-primitive-map.md`](internal/immune-system-primitive-map.md) — the full
  forward map: every biological primitive antigen could grow into
- [`decisions.md`](decisions.md) — ADR-003 (the metaphor is load-bearing), ADR-029
  (observe-don't-declare, Chapter 4's correction)
- [`glossary.md`](glossary.md) — every term anchored to its biological referent and
  Rust analog
- [`concepts.md`](concepts.md) — the architectural concepts behind the story
