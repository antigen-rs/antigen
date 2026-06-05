# The Failure-Class Cookbook

> Most docs are organized by *what the tool offers*. This one is organized by
> *what you have* — the code situation in front of you right now. Find your
> situation, get the antigen + the witness + the command to confirm it bound. Every
> recipe is grounded in a shipped stdlib member; the canonical reference is
> [`stdlib-families.md`](stdlib-families.md), and the per-member tier/fingerprint
> truth lives there.
>
> **How to read a recipe.** Each has the same shape: *You have →* (your situation),
> *The antigen →* (import + `#[presents]`), *The defense →* (the witness that proves
> you've handled it), *Confirm →* (scan/audit it). The point is the inversion: you
> don't learn antigen's vocabulary first and then go looking for uses — you start
> from your code and arrive at the vocabulary you need.

---

## A note on tiers before you start

Two of the recipes below are **`named`** (high-confidence: if the fingerprint
doesn't fire, you're covered) and two are **`suspected`** (a correlator: a match is
*a prompt to look*, not a verdict — it may also fire on idiomatic-correct code).
The recipe tells you which, because it changes how you act on a match. A `named`
match is a real site to defend or tolerate; a `suspected` match is a prompt to
look, and you tolerate the benign ones explicitly. (Full tier semantics:
[`stdlib-families.md`](stdlib-families.md) and [`witness-tiers.md`](witness-tiers.md).)

---

## You have: untrusted input crossing into typed structs

### …deserialized from a stream (`from_reader`)

**You have** — code that deserializes from a reader/byte-stream
(`serde_json::from_reader(socket)`) with no size, depth, or recursion bound. This
is a DoS surface: a non-terminating or deeply-nested stream blows the stack or
allocates unboundedly. (Recorded harm across ≥3 RUSTSEC advisories, 2022→2026.)

**The antigen** (`named`):
```rust
use antigen::stdlib::deserialization::UnboundedDeserialization;
use antigen::presents;

#[presents(UnboundedDeserialization)]
fn load(socket: impl std::io::Read) -> Config { /* from_reader(...) */ }
```

**The defense** — a **bounded reader** (`.take(limit)`) or a depth guard
(`serde_stacker`) on the deserialization path, proved by a `#[defended_by]` test
that exercises the cap. The `.take(limit)`-capped form *still presents* the surface
(the risky shape is genuinely there) and is **spared by the witness at audit**, not
by the fingerprint.

**Confirm:**
```sh
cargo antigen scan --root .   # the from_reader site surfaces as a presentation
cargo antigen audit --root .  # the #[defended_by] witness marks it defended
```

> Note: `from_slice` / `from_str` are deliberately *not* flagged — a slice is a
> bounded source. If your input is already fully in memory, the unbounded risk (if
> any) is the upstream read, not the deser call.

### …with a `Deserialize` struct that silently drops unknown fields

**You have** — a `#[derive(Deserialize)]` type at a trust boundary (config, auth,
payment payloads) *without* `#[serde(deny_unknown_fields)]`. Unknown input fields
are silently dropped — masking API drift and smuggled fields.

**The antigen** (`suspected` — not every `Deserialize` is at a trust boundary, so a
match is a prompt to look):
```rust
use antigen::stdlib::deserialization::DeserializeWithoutDenyUnknownFields;
```

**The defense** — set `#[serde(deny_unknown_fields)]` (the tight-junction that
rejects unknown fields), OR a documented "lenient-by-design" tolerance, OR a
validating wrapper. Caveat: `#[serde(flatten)]` re-opens the boundary in a way the
syntactic tell can't see.

**Confirm:** `cargo antigen scan --root .` — the struct without the arg surfaces;
the one that sets `deny_unknown_fields` is spared by the fingerprint.

---

## You have: a `Drop` impl, or a deliberate leak

### …a `Drop` that can panic

**You have** — an `impl Drop for T` whose body can reach a panic source: a
call-shaped `.unwrap()` / `.expect()`, or a macro-shaped `panic!` / `unreachable!`
/ `todo!` / `unimplemented!`. A panic in `Drop` *during an in-flight unwind* aborts
the process and skips cleanup — leaked resources even on `panic=unwind`.

**The antigen** (`named`):
```rust
use antigen::stdlib::drop_panic::PanicInDrop;
```

**The defense** — make the drop body panic-free; OR wrap the risky op to catch/log;
OR check `std::thread::panicking()` before the risky op so the destructor stays
quiet during an in-flight unwind.

**Confirm:** `cargo antigen scan --root .` — the real `Drop` impl with a reachable
panic source surfaces (and an inherent method merely *named* `drop` is correctly
spared — the anchor is `impl_of_trait("Drop")`, the real trait).

### …a deliberate leak (`mem::forget` / `Box::leak` / `Vec::leak`)

**You have** — a call to an explicit-leak primitive. Legitimate for `'static`
upgrades, but a silent leak if misused — and the next reader can't tell which.

**The antigen** (`suspected` — `forget`/`leak` are common method names, so a match
is a prompt to look: a domain `cache.forget()` will also fire):
```rust
use antigen::stdlib::resource_lifecycle::DeliberateLeakNotDocumented;
```

**The defense** — the witness this antigen asks for is the **documented rationale**:
`Box::leak` for a known-`'static` singleton is fine *if you say so*. Or: the
resource isn't actually leaked.

**Confirm:** `cargo antigen scan --root .` — leak-primitive calls surface; document
the legitimate ones, tolerate them explicitly.

---

## You have: an `unsafe` primitive

> The unsafe-soundness members are all `named` — every needle is a rare/std-specific
> unsafe primitive a domain type won't collide with. A match is a real soundness
> question. (These ship with a scan fixture rather than a runnable example, because
> the workspace forbids `unsafe`; see
> [`stdlib-families.md` § Unsafe-Soundness](stdlib-families.md#unsafe-soundness-boundary).)

### …a `transmute`

**You have** — a `mem::transmute` / `transmute_copy` call. The most dangerous single
function in Rust: a size/lifetime/mutability mismatch is instant UB
(rustc `mutable_transmutes` is deny-by-default — `&T → &mut T` is UB).

**The antigen** (`named`): `use antigen::stdlib::unsafe_soundness::TransmuteSizeOrLifetimeMismatch;`

**The defense** — a documented layout guarantee (`#[repr(...)]`) + a miri run, OR
replace the transmute with a checked conversion.

### …a `MaybeUninit::assume_init` / `mem::uninitialized`

**You have** — reading uninitialized memory as initialized. Instant UB.

**The antigen** (`named`): `use antigen::stdlib::unsafe_soundness::UninitMemoryAssumedInit;`

**The defense** — a `// SAFETY:` proving full initialization before the read, OR
miri/kani.

> The `zeroed` and `set_len` arms were deliberately **dropped** from this member —
> they fired on the recommended-safe `bytemuck::zeroed` and on benign domain
> setters. (That correction is its own [war-story](war-stories/the-self-catch.md).)

### …a `str::from_utf8_unchecked`

**You have** — building a `str` from non-validated bytes. A `str` with invalid
UTF-8 is UB; every downstream `str` op may misbehave.

**The antigen** (`named`): `use antigen::stdlib::unsafe_soundness::UnvalidatedFromUtf8Unchecked;`

**The defense** — the bytes were validated (or are a known-UTF-8 constant), proved
by a `// SAFETY:` + a check, or just use the checked `str::from_utf8`.

### …a hand-written `unsafe impl Send` / `Sync`

**You have** — an `unsafe impl Send for T` / `unsafe impl Sync for T`. You're
asserting cross-thread safety the compiler cannot check. ~40% of unsound RUSTSEC
advisories root here (raw pointers, `*mut`, interior non-`Sync`).

**The antigen** (`named`): `use antigen::stdlib::async_soundness::UnsafeSendSync;`

**The defense** — a documented `// SAFETY:` argument the sensor layer reads, OR a
kani proof of the `Send`/`Sync` invariant.

> This is the family with the cleanest console demo: scan
> `antigen/tests/fixtures/family_unsafe_send_sync` and you get **exactly one** site
> (the bad `unsafe impl Send`) — its safe sibling is un-marked, so the fingerprint
> genuinely spares it and it never appears. It's the clearest place to *watch the
> fingerprint separate clean from dirty.*

---

## You have: unchecked indexing or a raw copy

### …a `get_unchecked` / `get_unchecked_mut`

**You have** — the unchecked-indexing escape hatch. An out-of-bounds index here is
**UB**, not a clean panic — silent memory corruption.

**The antigen** (`named`): `use antigen::stdlib::panic_on_index::GetUncheckedWithoutProof;`

**The defense** — a `// SAFETY:` comment proving the index is in-bounds + a miri
run, OR the checked `.get(i)` with a handled `None`.

### …a raw `copy_nonoverlapping` near a `size_of`

**You have** — `ptr::copy_nonoverlapping(src, dst, n * size_of::<T>())`. The count
arg is in **elements**, not bytes — multiplying by `size_of` over-copies by
`sizeof(T)` → out-of-bounds → UB. (clippy has a correctness lint for exactly this.)

**The antigen** (`suspected` — the co-presence correlates with the defect but also
fires on idiomatic-correct both-calls code, so a match is a prompt to look):
```rust
use antigen::stdlib::numeric_truncation::SizeOfInElementCount;
```

**The defense** — the count is a plain element count (no `size_of` multiplier), OR a
`// SAFETY:` argument that the byte/element units are correct, OR miri. The fix
`copy(n)` with no `size_of` is spared by the fingerprint.

---

## You have: a clock read that can panic

### …`SystemTime::duration_since(...).unwrap()`

**You have** — a `SystemTime` clock read whose `Result` is `unwrap`/`expect`-ed.
The system clock can run *backwards* (NTP correction, manual set, VM pause) →
`duration_since` returns `Err` → the `.unwrap()` panics **in production but never
in tests** (test machines don't NTP-skew mid-test). The textbook bug the test suite
structurally cannot reach.

**The antigen** (`suspected` — a co-occurrence form; the infallible
`Instant::duration_since` shares the name, a known within-tier false positive):
```rust
use antigen::stdlib::time_ordering::SystemTimeUnwrapPanic;
```

**The defense** — handle the `Result` (`.unwrap_or(Duration::ZERO)`, a `match`), OR
use `Instant` instead of `SystemTime` for the measurement (the textbook fix —
`Instant` is monotonic and can't run backwards).

**Confirm:** `cargo antigen scan --root .`

---

## You have: a danger you feel but can't name yet

This isn't a failure-class — it's the thing that comes *before* one. You're reading
code and something is wrong here, but you can't yet say what class it is. That
unease is the single most perishable piece of knowledge in software: it evaporates
the moment you context-switch. Record it **structurally, at the site, before it's
gone**.

**The markers** (from `antigen::{aura, dread, red_flag}`):
- `#[aura(trigger = "...")]` — "something *may* be off, check later."
- `#[dread(trigger = "...")]` — "something *is* wrong here, can't name it, look now."
- `#[red_flag(trigger = "...")]` — "I'm *sure* something is wrong, can't name it,
  act now."

The `trigger` field is **required** — a marked-unknown with no stated trigger is
contentless graffiti, so it's a compile error. These never gate CI and never nag;
they're a structural note-to-future-self that survives the context-switch.

**Confirm:**
```sh
cargo antigen scan --root . --format json   # markers surface under report.marked_unknowns
```

---

## You have: a failure-class antigen doesn't ship yet

The catalog ships eight families — but the whole point of antigen is that *you*
declare the failure-classes you've encountered. The recipe is the same shape as
everything above, you just write the `#[antigen]` declaration yourself:

```rust
use antigen::antigen;

#[antigen(
    name = "your-failure-class",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"body_calls("the_dangerous_call")"#,
    family = "your-family",
    summary = "What this catches, in one sentence.",
    references = ["the advisory / issue / commit where you encountered it"],
)]
pub struct YourFailureClass;
```

The discipline that keeps your declarations honest is the same one the stdlib
follows: declare from a *real encountered instance* (recognition, not speculation —
[ADR-006](decisions.md)), and pick the tier the fingerprint earns (a bare common
method name is `suspected`, not `named`). For the full authoring guide see
[`fingerprint-grammar.md`](fingerprint-grammar.md) and [`macros.md`](macros.md);
for *why* tier-honesty matters, the [war-story](war-stories/the-self-catch.md) is
the cautionary tale.

---

## The recipe index

| You have | Antigen | Tier |
|---|---|---|
| streaming `from_reader` deser, unbounded | `UnboundedDeserialization` | named |
| `Deserialize` without `deny_unknown_fields` | `DeserializeWithoutDenyUnknownFields` | suspected |
| a `Drop` that can panic | `PanicInDrop` | named |
| `mem::forget` / `Box::leak` / `Vec::leak` | `DeliberateLeakNotDocumented` | suspected |
| `transmute` / `transmute_copy` | `TransmuteSizeOrLifetimeMismatch` | named |
| `assume_init` / `mem::uninitialized` | `UninitMemoryAssumedInit` | named |
| `str::from_utf8_unchecked` | `UnvalidatedFromUtf8Unchecked` | named |
| `unsafe impl Send` / `Sync` | `UnsafeSendSync` | named |
| `get_unchecked` / `get_unchecked_mut` | `GetUncheckedWithoutProof` | named |
| `copy_nonoverlapping` near `size_of` | `SizeOfInElementCount` | suspected |
| `SystemTime::duration_since(...).unwrap()` | `SystemTimeUnwrapPanic` | suspected |
| a danger you can't name yet | `#[aura]` / `#[dread]` / `#[red_flag]` | (markers) |
| something not in the stdlib | write your own `#[antigen]` | (your call) |

---

## See also

- [`stdlib-families.md`](stdlib-families.md) — the canonical catalog: every member's
  exact tier, fingerprint, and witness
- [`examples-guide.md`](examples-guide.md) — runnable example for each family
- [`macros.md`](macros.md) — the full macro reference
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — authoring your own antigens
- [`war-stories/the-self-catch.md`](war-stories/the-self-catch.md) — why the tiers
  in this cookbook are the *honest* tiers, told as the story of antigen catching its
  own over-claims
