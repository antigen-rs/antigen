# Antigen stdlib catalog — the failure-class families

> A scan-and-find catalog of the failure-classes antigen ships ready-to-use in
> its stdlib. Each entry tells you, in user terms: **what it catches**, its
> **tier** (how loud / how trustworthy the verdict is), its **fingerprint shape**
> (what the scanner actually looks for), and — for the families that have one — a
> link to a **runnable example** whose source pairs a bad path against its safe
> sibling so you can read the structural difference.
>
> **A precise word on "spared," because the console can mislead a newcomer.** In
> the example *source*, the safe sibling is "spared" in the sense that the
> **fingerprint does not bind it** (no `from_reader`, no `unwrap`, a panic-free
> `Drop`, …). But the examples deliberately put `#[presents(...)]` on *both* the
> bad and the safe sibling (to teach the affinity-pair), and an explicit
> `#[presents]` is an **author declaration** that surfaces in `scan`/`audit`
> **regardless of whether the fingerprint matched**. So when you run `audit`, both
> siblings appear as presentations — you do **not** see the safe one vanish. The
> one place you *do* see a fingerprint genuinely spare a site is an **un-marked**
> sibling (e.g. `drop_panic`'s `NotReallyDrop`, which has no `#[presents]` and
> simply never appears). Read "spared" as *the fingerprint doesn't bind it*, not as
> *it won't show up in the console* — those are different, and only the first is
> always true.
>
> **Two lessons, two demos — both are real, and they teach different things.**
> *Lesson 1* (the family examples below): a *safe* site can still **present** the
> shape — `#[presents]` is "this site is in this failure-class's territory," not a
> vulnerability claim — so both siblings list, and the safe one is spared *by the
> fingerprint*, not by vanishing. *Lesson 2* (**the showcase**:
> [`family_unsafe_send_sync`](#async-soundness)): what it looks like when the
> fingerprint **genuinely spares a site at the console** — scan that fixture and you
> get **exactly one** site (the bad `unsafe impl Send`), because its safe sibling is
> *un-marked*, so the fingerprint doesn't bind it and it's simply **absent**. (The
> family examples can't show this because they deliberately `#[presents]`-mark both
> siblings to teach Lesson 1.)
>
> And to *watch the fingerprint itself separate clean from dirty*, **read the guard
> tests** — they're clearer than any example, because each family's `_binds_` and
> `_spares_` cases sit side by side with plain-English rationale:
> [`antigen/tests/stdlib_family_fingerprints.rs`](../antigen/tests/stdlib_family_fingerprints.rs)
> has a pair for *every* family — e.g.
> `unbounded_deserialization_binds_from_reader_call` (the bad `from_reader` shape
> **matches**) right beside `unbounded_deserialization_spares_from_slice_namesake`
> (the clean `from_slice` namesake **doesn't**); and
> [`antigen/tests/spares_namesake_contract.rs`](../antigen/tests/spares_namesake_contract.rs)
> pins the harder namesake cases. `cargo test` confirms they *hold*; **reading**
> them is where you *see* the separation. No example re-engineering needed — the
> thesis is demonstrable today.
>
> **A heads-up so nothing reads as a dead link**: two families have no *runnable*
> example (their tell is a real `unsafe` primitive, which `unsafe_code = "forbid"`
> blocks from a compiled crate) — but they **do** have a **scan fixture** you point
> the scanner at: `async-soundness` and `unsafe-soundness` ship their members and a
> `cargo antigen scan --root antigen/tests/fixtures/...` fixture (commands in their
> sections below). And `crypto-misuse` is **chartered**: the failure-class is
> identified and tracked, but **no member ships yet** (no honest fingerprint exists
> in the current grammar), so there is deliberately nothing to scan. If you go looking for
> a `cargo run --example` for those three and don't find one, you didn't miss
> anything — it isn't there by design (scan the fixture instead, or — for crypto —
> there's simply nothing yet).
>
> These families live in [`antigen/src/stdlib/`](../antigen/src/stdlib/). Import a
> member by type name and reference it from a `#[presents(...)]` site:
>
> ```rust
> use antigen::stdlib::unsafe_soundness::TransmuteSizeOrLifetimeMismatch;
> use antigen::presents;
>
> #[presents(TransmuteSizeOrLifetimeMismatch)]
> fn reinterpret(/* ... */) { /* ... */ }
> ```
>
> For the *authoritative* tier/fingerprint/summary of any member, the source
> docstring in `antigen/src/stdlib/<family>.rs` is ground truth — this catalog
> mirrors it.

---

## How to read a tier

Every member carries an honest **confidence tier** — antigen's own discipline is
that a verdict never claims more certainty than its fingerprint earns.

| Tier | What it promises | How to act on a match |
|---|---|---|
| **named** | High-confidence. The fingerprint's effective codomain *is* the defect population (a rare/std-specific call, or a defect-slice anchor). "If it doesn't fire, you're covered." | Treat a match as a real site to defend or tolerate. |
| **suspected** | A correlator. The shape co-occurs with the defect but a common name or a co-presence can also fire on idiomatic-correct code. A labeled recall hole is acceptable at this tier. | Treat a match as a *prompt to look*, not a verdict. Tolerate the benign ones explicitly. |
| **chartered** | The failure-class is real and recorded, but **no honest fingerprint exists yet** in the shipped grammar. Nothing ships — better honest-deferred than dishonest-shipped. | Nothing to scan yet; the class is identified so the graduation path is tracked. |

Tier is a property of the *fingerprint shape*, not a field you set — see
[`witness-tiers.md`](witness-tiers.md) for the gradient and ADR-039 §C for the
admission discipline (`provenance` = how solid the class is; the dial-tier = how
loud this instance is — orthogonal axes).

> **One word to disambiguate:** **`named`** in this catalog is the *confidence
> tier* above — a high-confidence fingerprint — **not** the separate sense in which
> antigen "names" a failure-class (declares it). A `suspected` or `chartered` family
> is still a *named* (declared) failure-class; it just isn't at the `named` *tier*.

A note on the fingerprint grammar: `body_calls("name")` matches a call by its
**last path segment** with no path resolution (so `from_reader` matches
`serde_json::from_reader` *and* any `Foo::from_reader`); `impl_of_trait("Drop")`
matches a real trait impl (not an inherent method merely *named* `drop`);
`is_unsafe` reads the `unsafe` qualifier; `derives(...)` / `serde_arg(...)` read
attribute tokens; `not(...)` / `all_of([...])` / `any_of([...])` compose. Full
DSL in [`fingerprint-grammar.md`](fingerprint-grammar.md).

---

## The families at a glance

The **Example** column tells you what you can run/scan today — `yes` (a runnable
`cargo run --example`), `scan fixture` (no runnable example — the members ship but
their `unsafe` tell can't compile under `unsafe_code = "forbid"`, so scan a fixture
with `cargo antigen scan --root antigen/tests/fixtures/...` — command in the
section), or `n/a` (chartered: nothing ships, nothing to scan, by design).

| Family | Member(s) | Tier | Catches | Example |
|---|---|---|---|---|
| [Deserialization-Trust-Boundary](#deserialization-trust-boundary) | `UnboundedDeserialization` | **named** | streaming `from_reader` DoS surface | yes |
| | `DeserializeWithoutDenyUnknownFields` | **suspected** | silent unknown-field drop at the trust boundary | yes |
| [Time-and-Ordering-Hazards](#time-and-ordering-hazards) | `SystemTimeUnwrapPanic` | **suspected** | `SystemTime` clock-skew panic (silent in tests) | yes |
| [Drop-and-Panic-Discipline](#drop-and-panic-discipline) | `PanicInDrop` | **named** | a `Drop` impl that can panic → process abort | yes |
| [Panic-on-Index](#panic-on-index) | `GetUncheckedWithoutProof` | **named** | `get_unchecked` — out-of-bounds is UB, not a panic | yes |
| [Resource-Lifecycle-Leak](#resource-lifecycle-leak) | `DeliberateLeakNotDocumented` | **suspected** | `mem::forget` / `Box::leak` skipping `Drop` | yes |
| [Async-Soundness](#async-soundness) | `UnsafeSendSync` | **named** | a hand-written `unsafe impl Send/Sync` | scan fixture |
| [Numeric-Truncation-Overflow](#numeric-truncation-overflow) | `SizeOfInElementCount` | **suspected** | `size_of`-in-element-count raw-copy foot-cannon | yes |
| [Unsafe-Soundness-Boundary](#unsafe-soundness-boundary) | `TransmuteSizeOrLifetimeMismatch` | **named** | `transmute` size/lifetime/mutability mismatch | scan fixture |
| | `UninitMemoryAssumedInit` | **named** | reading uninitialized memory as initialized | scan fixture |
| | `UnvalidatedFromUtf8Unchecked` | **named** | `from_utf8_unchecked` on non-validated bytes | scan fixture |
| [Crypto-Misuse](#crypto-misuse-chartered) | *(none — chartered)* | **chartered** | non-constant-time secret comparison (deferred) | n/a |

Plus the three [marked-unknown markers](#marked-unknown-markers) —
`#[aura]` / `#[dread]` / `#[red_flag]` — for the danger you *feel* but can't
name yet (example: [`marked_unknown.rs`](../antigen/examples/marked_unknown.rs)).

**The count, explicitly** (to dissolve the 8-vs-9): **8 families ship members
(11 members total)**, and **crypto-misuse is chartered** — the class is identified,
but no honest call-only fingerprint exists in the shipped grammar yet, so it ships
no member. So: 8 shipping families + 1 chartered = 9 families in the catalog, 11
shipped members.

---

## Deserialization-Trust-Boundary

**Source**: [`antigen/src/stdlib/deserialization.rs`](../antigen/src/stdlib/deserialization.rs) ·
**Example**: [`antigen/examples/deserialization.rs`](../antigen/examples/deserialization.rs) ·
**Category**: `FunctionalCorrectness`

The deep tier of the shipped Mucosal-Boundary family — deserialization is *the*
place untrusted bytes cross into typed-Rust land (the gut mucosa, the
largest/busiest trust surface). Biology cognate: `deny_unknown_fields` is the
tight-junction that decides what crosses the gut wall; its absence is a leaky
gut.

### `UnboundedDeserialization` — **named**

**Catches**: a *streaming* `from_reader` deserialization with no size/depth/
recursion limit — a DoS surface (the std-documented non-terminating-stream /
stack-exhaustion harm; recorded across ≥3 RUSTSEC advisories 2022→2026).

**Fingerprint**: `body_calls("from_reader")`

**Why named**: `from_reader` is rare/std-specific — a domain type rarely has a
`from_reader` method, so the needle self-anchors onto the defect population.

**How the defense works (the surface-flag / witness-proof split)**: a
`.take(limit)`-capped reader is the std anti-DoS idiom — but the bounded form
**still presents** the surface (the risky `from_reader` *is* present), so the
design is "the surface is flagged; the *witness* proves the defense at audit," not
"the fingerprint spares the bounded form." A `not(take)` guard would silently
suppress real DoS sites whenever an unrelated `Iterator::take` appeared — a silent
false-negative that breaks the named tier's promise. That is the *design
principle*; for the **gap between this principle and what the console shows
today**, read the box below before you scan — it matters.

> **What `audit` actually shows here today (read before you run it).** The example
> marks *both* `load_unbounded` and `load_bounded` with `#[presents]`, and a single
> `#[defended_by]` witness (`load_bounded_is_capped_test`). Run `cargo antigen
> audit --root antigen/examples` and you'll see **both** sites reported
> `✓ defended at Reachability`, both credited to that one witness — **not**
> `load_unbounded` undefended next to `load_bounded` defended. That's because the
> current audit credits a `#[defended_by]` witness at the **antigen-type**
> granularity, not per-site: one witness for `UnboundedDeserialization` marks every
> `UnboundedDeserialization` presents-site defended. So the surface-flag /
> witness-proof *split* is a real design principle, but this example does **not**
> visibly separate the two sites at the console — both render defended. (And
> `UnboundedDeserialization` does **not** appear in `scan`'s
> fingerprint-*candidate* list, because both its sites are explicitly
> `#[presents]`-marked — an explicit mark is an author declaration that surfaces as
> a *presentation*, not as a fingerprint candidate.) This "bad site shows defended"
> is **inherent to type-granular witness crediting**, not an example bug — no
> example change fixes it without a tool change. Today's crediting is
> **type-granular**: the honest current state, not a defect; the principle above is
> the durable teaching. (A finer **site-granular** model — a witness crediting only
> the site it exercises — is a graduation path; see
> [`roadmap.md`](roadmap.md).) To *see* the fingerprint bind/spare directly, read
> the guard tests (the "two lessons" note at the top points the way).

> `from_slice` / `from_str` are deliberately **excluded**: a slice is a *bounded*
> source (so `from_slice` is not an unbounded vector — and it fired on the
> bounded-slice fix itself plus safe constructors like `GenericArray::from_slice`,
> ADR-039 §C Amd-1); `from_str` would collide with every `i32::from_str`. (The
> in-memory deep-nesting recursion DoS is a distinct harm with its own remedy — a
> separate member, not a widened fingerprint; see [`roadmap.md`](roadmap.md).)

### `DeserializeWithoutDenyUnknownFields` — **suspected**

**Catches**: a `#[derive(Deserialize)]` type that does *not* set
`#[serde(deny_unknown_fields)]` — unknown input fields are silently dropped,
masking API drift and smuggled fields (the cleanest attribute presence-AND-
absence tell in the stdlib).

**Fingerprint**: `all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])`

**Why suspected**: not every `Deserialize` sits at a trust boundary; the member
graduates to named when paired with a trust-boundary marker. Known caveat:
`#[serde(flatten)]` re-opens the boundary in a way the syntactic tell can't see.

---

## Time-and-Ordering-Hazards

**Source**: [`antigen/src/stdlib/time_ordering.rs`](../antigen/src/stdlib/time_ordering.rs) ·
**Example**: [`antigen/examples/time_ordering.rs`](../antigen/examples/time_ordering.rs) ·
**Category**: `FunctionalCorrectness`

The flagship **silent-in-tests / panic-in-prod** shape. Biology cognate:
circadian / signaling-timing failure — a clock running backwards corrupts the
cascade timing.

### `SystemTimeUnwrapPanic` — **suspected**

**Catches**: a `SystemTime::duration_since` clock read whose `Result` is
`unwrap`/`expect`-ed. (The `.elapsed()` form is *not* in the fingerprint — a
documented, recoverable false-negative; see the exclusion box below.) The
system clock can run *backwards* (NTP correction, manual set, VM pause) →
`duration_since` returns `Err` → the `.unwrap()` panics **in production but never
in tests** (test machines don't NTP-skew mid-test). The textbook bug the test
suite structurally cannot reach.

**Fingerprint**: `all_of([body_calls("duration_since"), any_of([body_calls("unwrap"), body_calls("expect")])])`

**Why suspected**: the shipped grammar has no method-chain leaf, so this is the
*co-occurrence* form — a `duration_since` call AND an `unwrap`/`expect` in the
same body. Co-occurrence correlates with the panic-chain but doesn't prove it
(the `unwrap` could guard a different `Result`). It also carries a known namesake
FP: the infallible `Instant::duration_since` shares the name, and the only
discriminator is the receiver *type*, which scan can't resolve. (Its graduation
to named — a precise method-chain leaf plus receiver-type resolution — is a
recorded path; see [`roadmap.md`](roadmap.md).)

> `elapsed` is **excluded** from the anchor: it would fire on
> `Instant::now().elapsed()` — but `Instant` is monotonic and
> `Instant::elapsed()` returns `Duration` (can't panic-on-skew). That's the
> textbook *"use `Instant` instead of `SystemTime`"* fix — the member's own clean
> sibling. A needle that fires on the fix is dropped at every tier. (Recall cost:
> `SystemTime::elapsed().unwrap()` is a known, recoverable false-negative — see
> [`roadmap.md`](roadmap.md).)

---

## Drop-and-Panic-Discipline

**Source**: [`antigen/src/stdlib/drop_panic.rs`](../antigen/src/stdlib/drop_panic.rs) ·
**Example**: [`antigen/examples/drop_panic.rs`](../antigen/examples/drop_panic.rs) ·
**Category**: `FunctionalCorrectness`

Teardown footguns. A panic during `Drop` *while another panic is unwinding*
aborts the process — and the destructor's own cleanup is skipped → leaked
resources even on the unwinding path. Biology cognate: apoptosis gone wrong —
programmed cell death that triggers a catastrophic cascade instead of clean
teardown.

### `PanicInDrop` — **named**

**Catches**: a real `Drop` impl whose body reaches a panic source — either a
call-shaped `.unwrap()` / `.expect()` **or** a macro-shaped `panic!` /
`unreachable!` / `todo!` / `unimplemented!`.

**Fingerprint**: `all_of([item = impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), body_calls("expect"), body_contains_macro("panic"), body_contains_macro("unreachable"), body_contains_macro("todo"), body_contains_macro("unimplemented")])])`

**Why named**: the `impl_of_trait("Drop")` anchor is precise — it matches the
real `Drop` trait, not an inherent method merely *named* `drop` (the example's
`NotReallyDrop` is correctly spared). This is the v2 of the older `basic.rs`
`PanickingInDrop`, which over-fired on non-`Drop` impls and missed `.unwrap()`-
shaped panics (it was macro-only). Covering *both* call- and macro-shaped panics
is the point — `.unwrap()` is the more common teardown panic.

**Witness**: a panic-free drop body, OR the risky op wrapped to catch/log, OR a
`std::thread::panicking()` check before the risky op.

---

## Panic-on-Index

**Source**: [`antigen/src/stdlib/panic_on_index.rs`](../antigen/src/stdlib/panic_on_index.rs) ·
**Example**: [`antigen/examples/panic_on_index.rs`](../antigen/examples/panic_on_index.rs) ·
**Category**: `FunctionalCorrectness`

Out-of-bounds access classes. The build-now member is the **unsafe** form —
where an out-of-bounds index is Undefined Behavior, not a clean panic. Biology
cognate: proprioception / spinal-reflex failure — acting past the body's valid
range without the protective reflex (the bounds check) that normally fires.

### `GetUncheckedWithoutProof` — **named**

**Catches**: a call to `get_unchecked` / `get_unchecked_mut` — the unchecked-
indexing escape hatch whose out-of-bounds case is **UB** (silent memory
corruption, not a crash).

**Fingerprint**: `any_of([body_calls("get_unchecked"), body_calls("get_unchecked_mut")])`

**Why named**: both are slice/`Vec`-specific method names with no stdlib
collision — a clean call-shape, and the UB-on-OOB class is real and
miri-catchable.

**Witness**: a `// SAFETY:` comment proving the index is in-bounds + a miri run,
OR the checked `.get(i)` with a handled `None`.

> The **panic** form (`expr[i]` indexing with an input-derived index) is an
> Index-*operator* tell, not a call leaf, so the shipped grammar (`body_calls`
> reaches only call expressions) can't express it — this member ships the
> `get_unchecked` call form only. (The operator-leaf that would add the panic form
> is a recorded graduation path; see [`roadmap.md`](roadmap.md).)

---

## Resource-Lifecycle-Leak

**Source**: [`antigen/src/stdlib/resource_lifecycle.rs`](../antigen/src/stdlib/resource_lifecycle.rs) ·
**Example**: [`antigen/examples/resource_lifecycle.rs`](../antigen/examples/resource_lifecycle.rs) ·
**Category**: `FunctionalCorrectness`

Leaks: resources whose `Drop` never fires. The sibling of Drop-and-Panic on the
Drop-Lifecycle axis (this family = drop *never-fires*; drop-panic = drop
*fires-but-explodes* — not merged, distinct remedies). Biology cognate: failure
of apoptosis / efferocytosis — cells that should die and be cleared instead
persist.

### `DeliberateLeakNotDocumented` — **suspected**

**Catches**: a call to an explicit-leak primitive — `mem::forget` / `Box::leak`
/ `Vec::leak` — which deliberately skips `Drop`. Legitimate for `'static`
upgrades, but a silent leak if misused; the witness antigen asks for is the
*documented rationale*.

**Fingerprint**: `any_of([body_calls("forget"), body_calls("leak")])`

**Why suspected**: `forget` / `leak` are bare common last-segments with no
narrowing anchor — `body_calls` matches the last segment, so a domain
`cache.forget()` / `permissions.leak()` also fires. A positive tell at the named
(loud) tier would overclaim, so the honest tier is suspected. (Note: the *class*
is `provenance = Constructable` — `mem::forget` demonstrably skips `Drop` — even
though this *instance's* dial sits at suspected; provenance and dial-tier are
orthogonal.) (Its graduation to named — path / semantic resolution narrowing the
codomain to the real leak primitives — is a recorded path; see
[`roadmap.md`](roadmap.md).)

---

## Async-Soundness

**Source**: [`antigen/src/stdlib/async_soundness.rs`](../antigen/src/stdlib/async_soundness.rs) ·
**Scan fixture**: [`antigen/tests/fixtures/family_unsafe_send_sync/lib.rs`](../antigen/tests/fixtures/family_unsafe_send_sync/lib.rs) ·
**Category**: `FunctionalCorrectness`

Concurrency-boundary footguns. Biology cognate: the innate barrier of the
concurrency boundary — an `unsafe impl Send/Sync` is a mislabeled self/non-self
marker.

> **Why a scan fixture, not a runnable example**: the member's tell is a real
> `unsafe impl Send`, and the workspace sets `unsafe_code = "forbid"` (an
> un-overridable forbid), so it can't live in a compiled example crate. The scanner
> reads source as *text* (it doesn't compile it), so the affinity-pair lives in a
> fixture you **scan** (not `cargo run`):
>
> ```sh
> cargo run --bin cargo-antigen -- antigen scan --root antigen/tests/fixtures/family_unsafe_send_sync
> ```
>
> **This is the catalog's Lesson-2 showcase** (see the "two lessons, two demos"
> note at the top) — what it looks like when the **fingerprint genuinely spares a
> site at the console**: scan reports **exactly one** site, the bad `unsafe impl
> Send for RawHandle` (`:37`), and the safe sibling (`impl Clone for RawHandle`) is
> **un-marked**, so the fingerprint doesn't bind it and it never appears. (Contrast
> the L1 family examples, which deliberately `#[presents]`-mark both siblings so
> both surface — that's the "present ≠ vulnerable" lesson.)

### `UnsafeSendSync` — **named**

**Catches**: a hand-written `unsafe impl Send for T` / `unsafe impl Sync for T`
— an author-asserted cross-thread safety the compiler cannot verify. ~40% of
unsound RUSTSEC advisories root here (raw pointers, `*mut`, interior non-`Sync`).

**Fingerprint**: `all_of([item = impl, is_unsafe, any_of([impl_of_trait("Send"), impl_of_trait("Sync")])])`

**Why named**: a hand-written `unsafe impl Send/Sync` is an explicit soundness
assertion — its presence is itself a strong signal, RUSTSEC-backed.

**Witness**: a documented `// SAFETY:` argument the sensor layer reads, OR a kani
proof of the `Send`/`Sync` invariant.

---

## Numeric-Truncation-Overflow

**Source**: [`antigen/src/stdlib/numeric_truncation.rs`](../antigen/src/stdlib/numeric_truncation.rs) ·
**Example**: [`antigen/examples/numeric_truncation.rs`](../antigen/examples/numeric_truncation.rs) ·
**Category**: `FunctionalCorrectness`

Silent numeric-corruption classes. Biology cognate: silent mutation — a base-pair
flip that produces a still-folding protein (compiles, runs, returns *a* value),
but the wrong one; corruption propagates with no immediate symptom.

### `SizeOfInElementCount` — **suspected**

**Catches**: a raw-memory copy (`copy_nonoverlapping`) co-located with a
`size_of` — the famous byte-count-where-element-count-expected foot-cannon.
`ptr::copy_nonoverlapping(src, dst, n * size_of::<T>())` over-copies by a factor
of `sizeof(T)` because the count arg is in *elements*, not bytes → out-of-bounds
→ UB. clippy has a correctness lint (`size_of_in_element_count`) for exactly
this.

**Fingerprint**: `all_of([body_calls("copy_nonoverlapping"), body_calls("size_of")])`

**Why suspected** (demoted from named, ADR-039 §C Amd-1): the co-presence
*correlates* with the dangerous region but **cannot pinpoint** the defect — it
fires on idiomatic-correct both-calls code too (a byte-buffer copy, a separate-
bounds `size_of`), which a named tier could not carry. Its own anti-correlated
**fix** — `copy(n)` with an element count and no `size_of` — *is* spared (the
`all_of` needs both calls), so it is **demoted, not dropped**. (Its graduation to
named is *type-aware* — arg-position **and** pointee-type, since the correct `*u8`
byte-copy idiom still FPs without the pointee type — the resolved-type tier, **not**
a syntactic operator-leaf; see [`roadmap.md`](roadmap.md).)

> This member is the worked example of the seal's tier-honesty discipline: it was
> over-claimed at named, then corrected to suspected with the fingerprint
> **unchanged** — the fix was the tier, not the shape.

---

## Unsafe-Soundness-Boundary

**Source**: [`antigen/src/stdlib/unsafe_soundness.rs`](../antigen/src/stdlib/unsafe_soundness.rs) ·
**Scan fixture**: [`antigen/tests/fixtures/family_unsafe_soundness/lib.rs`](../antigen/tests/fixtures/family_unsafe_soundness/lib.rs) ·
**Category**: `FunctionalCorrectness`

Soundness holes reachable from safe-looking code — the `unsafe`-primitive
call-shapes where a wrong invariant is UB, not a panic. Every needle here is a
**rare/std-specific** unsafe primitive: a domain type won't have a method by that
name, so the needle alone restricts the codomain to the defect population (the
self-anchor rule) — that's why all three are **named**. The fingerprint fires on
the *presence* of the call; the precise size/lifetime/validity check that would
distinguish a sound use from an unsound one is a recorded graduation path (see
[`roadmap.md`](roadmap.md)). Biology cognate: the breached self/non-self membrane — a wrong
`unsafe` invariant is a forged MHC marker.

> **Why a scan fixture, not a runnable example** (same reason as async-soundness):
> every tell is a real `unsafe` primitive (`transmute` / `assume_init` /
> `from_utf8_unchecked`), which `unsafe_code = "forbid"` blocks from a compiled
> crate — so the three affinity-pairs live in a fixture you **scan**:
>
> ```sh
> cargo run --bin cargo-antigen -- antigen scan --root antigen/tests/fixtures/family_unsafe_soundness
> ```
>
> Scan reports six presentations — the bad path **and** the safe sibling of each of
> the three members — because this fixture `#[presents]`-marks *both* siblings
> (unlike the async fixture's un-marked spare). So read the BAD/GOOD pairs in the
> *source* for the fingerprint difference (`transmute` vs an `as` cast;
> `assume_init` vs a plain value; `from_utf8_unchecked` vs the checked
> `from_utf8`); the console lists both, per the "word on spared" caveat at the top.

### `TransmuteSizeOrLifetimeMismatch` — **named**

**Catches**: a `mem::transmute` / `transmute_copy` call — the most dangerous
single function in Rust. A size/lifetime/mutability mismatch is instant UB (rustc
`mutable_transmutes` is deny-by-default — `&T → &mut T` is UB).

**Fingerprint**: `any_of([body_calls("transmute"), body_calls("transmute_copy")])`

**Witness**: a documented layout guarantee (`#[repr(...)]`) + a miri run, OR a
checked conversion instead of the transmute.

### `UninitMemoryAssumedInit` — **named**

**Catches**: reading uninitialized memory as initialized —
`MaybeUninit::assume_init` / `mem::uninitialized`. Treating uninitialized memory
as a valid value is instant UB (clippy `uninit_assumed_init` / `uninit_vec`;
`mem::uninitialized` is deprecated *because* it's almost always UB).

**Fingerprint**: `any_of([body_calls("assume_init"), body_calls("uninitialized")])`

> `zeroed` was **dropped** (it fires on the recommended-safe `bytemuck::zeroed`,
> a clean-sibling collision); `set_len` was **dropped from this named member**
> (risky-vs-safe turns on receiver type *and* arg value, neither syntactic, ADR-039
> §C Amd-1). The recall hole is documented; a dedicated `suspected` `set_len` member
> is a recorded charter behind the semantic tier (see [`roadmap.md`](roadmap.md)).

**Witness**: a `// SAFETY:` proving full initialization before the read, OR
miri/kani.

### `UnvalidatedFromUtf8Unchecked` — **named**

**Catches**: `str::from_utf8_unchecked` / `_mut` on non-validated bytes — a UB
`str` (rustc `invalid_from_utf8_unchecked`). The call skips the UTF-8 validity
check; a `str` containing invalid UTF-8 is UB and every downstream `str`
operation may misbehave.

**Fingerprint**: `any_of([body_calls("from_utf8_unchecked"), body_calls("from_utf8_unchecked_mut")])`

**Witness**: the bytes were validated (or are a known-UTF-8 constant), proved by
a `// SAFETY:` + a check / miri.

---

## Crypto-Misuse *(chartered)*

**Source**: [`antigen/src/stdlib/crypto_misuse.rs`](../antigen/src/stdlib/crypto_misuse.rs) ·
**Status**: **chartered — no shipped member yet** ·
**Category**: `FunctionalCorrectness`

The RUSTSEC `crypto-failure` category seen from the developer side. Rust crypto
libraries mostly *avoid* insecure defaults, so the recurring failure-class is
developer **misuse** — reaching past the safe API for the dangerous one, or
omitting the safe step.

The flagship member `NonConstantTimeSecretComparison` (a secret/MAC compared in
non-constant time — a timing-attack oracle) is a **real, recurring** failure-class
(GHSA-q7pg-9pr4-mrp2 httpsig-rs HMAC timing attack). **But no honest *call-only*
fingerprint can express it in the shipped grammar**, so it is chartered, not
shipped:

- Anchoring on a crypto verify entrypoint (`verify` / `hmac_verify`) and firing
  on the *absence* of a constant-time compare **anti-aligns with the defect** — it
  fires loudest on the *safe* path. `ring::hmac::verify` is the **correct** API
  and is constant-time internally (no visible `ct_eq` call), so the fingerprint
  would falsely flag the recommended API as the bug (the clean-sibling-collision
  shape).
- The real defect — a **hand-rolled `==` / byte-loop on a secret** — has no
  distinctive call at all. It's an *operator* (`==`) on a *secret-typed value*,
  which would need **both** a `security_sensitive_name` name-leaf (the data-context)
  and an `==` operator-leaf (`ExprBinary`) — neither of which the call-only
  `body_calls` grammar can see.

So it stays chartered: **better honest-deferred than dishonest-shipped** — a
shipped call-only form would actively mislead by flagging the *correct*
`ring::hmac::verify`. (The grammar leaves that would let this family ship its
member at the `suspected` tier are a recorded graduation path; see
[`roadmap.md`](roadmap.md).)

---

## Marked-Unknown markers

**Source**: [`antigen-macros`](../antigen-macros/) (ADR-041) ·
**Example**: [`antigen/examples/marked_unknown.rs`](../antigen/examples/marked_unknown.rs)

Not failure-class fingerprints — three declarable **⊥ markers** for the single
most perishable piece of knowledge in software: the *felt-but-unnamed danger*,
the unease that evaporates the moment you context-switch or an agent compacts.
You record it **structurally, at the site, before it's gone** — without having to
name the failure-class yet.

They sit *off* the dial's classification axis (at ⊥, the unnameable), on a
**magnitude × existence-certainty** plane, and surface at the dial's **non-gating
floor** — they never gate (cannot fail CI) and never nag.

| Marker | Corner | Meaning |
|---|---|---|
| **`#[aura(trigger = "...")]`** | low magnitude | "something *may* be off, can't name it, check later." |
| **`#[dread(trigger = "...")]`** | high magnitude, low certainty (*angor animi*) | "something *is* wrong here, can't name it, look now." |
| **`#[red_flag(trigger = "...")]`** | high existence-certainty (the clinical sense-of-alarm) | "I'm *sure* something is wrong, can't name it, act now." Auto-escalates on first match. |

**The `trigger` field is REQUIRED** (ADR-041 guard 3): a triggerless (or empty/
whitespace) marker is a **compile error** — a marked-unknown with no stated
trigger is the contentless "this seems off" graffiti the primitive exists to
prevent (the rationale-as-required-field discipline, ADR-005 Amd2).

**Where they surface today**: `cargo antigen scan` reads each marker's doc-marker
and surfaces it in the JSON report under the top-level **`report.marked_unknowns`**
array — each entry carrying `marker`, `magnitude`, `existence_certainty`,
`trigger`, and `file`. (Internally these also emit as `FindingBody::MarkedUnknown`
records into the pipeline's unified `Finding` population at the ADR-041 emit-seam,
where a `#[red_flag]` auto-escalates severity — but that escalated severity lives
on the *internal Finding*, not on the scan-report projection, whose `severity`
field reads `null` today.) The **human-readable** scan report does *not* render
marked-unknowns yet — the audit-time confidence dial that surfaces them is a later
wave — so today a marker is a structural record you *query* (via `--format json`),
not a console line. The mark is never lost; that is the whole point.

```sh
cargo run --bin cargo-antigen -- antigen scan --root antigen/examples --format json
# → report.marked_unknowns: three entries (aura / dread / red-flag), each with its trigger
```

---

## Dogfood antigens (antigen's own)

Distinct from the families above: these are **antigen-internal** failure-classes —
classes antigen observed in *its own* development substrate and marks on its own
source (`antigen/src/stdlib/dogfood.rs`). They are **not part of the bundled catalog**
and are **not imported by adopters**; they exist so antigen eats its own dog food.
A `cargo antigen scan` on an external crate (even with `--bundled-catalog`) never
surfaces them.

### `SilentIntentNullification` — *dogfood*

A surface appears to accept or honor an adopter's declared intent but does not realize
it — the intent is silently nullified between declaration and effect. It is the
**parent** of two witness-distinct children, both silent: `ActiveArgumentDiscard`
(parse-side, behavioral witness) and `CapabilityOmissionAtLowering` (lowering-side,
structural witness).

Antigen marks its *own* drift-detector with it: `antigen::learn::adwin::detect` carries
`#[presents(SilentIntentNullification)]` because a blind detector axis collapsed into a
confident verdict would be exactly this class — the immune system declaring immunity to
its own silent-miscalibration failure-mode (the recall fingerprint covers the
`"silently nullified"` / `"silent-miscalibration"` phrasing). A scan of antigen's own
tree surfaces the mark as a **candidate, not a verdict**:

```
$ cargo antigen scan --root antigen
  antigen\src\learn\adwin.rs:118  SilentIntentNullification on enum [fingerprint match]
  antigen\src\stdlib\dogfood.rs:494  SilentIntentNullification on struct [fingerprint match]
```

For the full dogfood roster and each class's witness-structure, see the source
docstrings in
[`antigen/src/stdlib/dogfood.rs`](../antigen/src/stdlib/dogfood.rs).

---

## See also

- [`examples-guide.md`](examples-guide.md) — a runnable walkthrough lesson for
  each family example (scan a bad path next to its safe sibling)
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — the full fingerprint DSL
- [`witness-tiers.md`](witness-tiers.md) — the confidence/tier gradient
- [`macros.md`](macros.md) — the macro reference (incl. the marked-unknown markers)
- [`decisions.md`](decisions.md) — ADR-039 §C (the tier-honesty admission
  discipline), ADR-040 (the fingerprint grammar leaves), ADR-041 (the
  marked-unknown markers)
- The per-family source docstrings in
  [`antigen/src/stdlib/`](../antigen/src/stdlib/) — ground truth for every tier,
  fingerprint, and witness
