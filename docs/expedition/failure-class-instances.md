# Failure-Class Instances — Real-World Rust Ecosystem

> Concrete examples from the Rust ecosystem populating each of the 8 first-principles
> failure classes from `design-intent.md`. The taxonomy is the lens; this document is
> the test of whether the lens resolves real bugs. If a class is sparse, we say so —
> sparseness is signal, not a defect to paper over.

## How to read this

Each entry has:
- **Name** — kebab-case identifier suitable for `#[antigen(name = "...")]`
- **Class** — which of the 8 classes it instantiates
- **What went wrong** — one paragraph
- **Why it fits the class** — the structural fit, not just the surface symptom
- **Source** — URL or RUSTSEC ID

A given bug can structurally instantiate more than one class; we file under the
*primary* shape. Where a bug sits at a class boundary, we note the secondary class.

---

## 1. Frame-translation

> Semantic interpretation drifts when crossing context boundaries — each frame is
> locally consistent but the interface between them is broken.

### Instances

#### `path-join-absolute-replaces-base`
- **Class**: Frame-translation
- **What went wrong**: `Path::join(base, p)` silently *replaces* the base path when
  `p` is absolute. In one frame ("path concatenation") the API name suggests
  composition; in another frame (POSIX/Windows path semantics) absolute paths are
  already complete and override any base. Code that joins user-controlled segments
  to a sandbox root can be tricked into escaping the sandbox if any segment starts
  with `/` or a Windows drive letter. The frames disagree at the boundary: the
  caller's mental model is "concatenate," the implementation's mental model is
  "resolve."
- **Why it fits**: Two frames (caller intent / OS semantics) are each internally
  consistent but produce different answers; the API does not force the translation
  to be explicit. Clippy lint `clippy::join_absolute_paths` exists precisely
  because the frame mismatch is recurring.
- **Source**: <https://github.com/rust-lang/rust-clippy/issues/10655>,
  <https://users.rust-lang.org/t/rationale-behind-replacing-paths-while-joining/104288>

#### `chrono-naive-vs-tz-aware-datetime`
- **Class**: Frame-translation
- **What went wrong**: chrono distinguishes `NaiveDateTime`, `DateTime<Utc>`,
  `DateTime<Local>`, and `DateTime<FixedOffset>` types. Functions that convert
  between frames (especially `naive_local()` / `naive_utc()` / DST-ambiguous
  windows that return `MappedLocalTime`) routinely produce silent shifts when
  callers conflate "wall clock" and "instant." The bug is not chrono's; it's
  that the type-level frame distinction is necessary precisely because every
  ad-hoc treatment that ignores it produces wrong answers at DST boundaries.
- **Why it fits**: Local time, UTC, and naive time are three different frames;
  conversion is non-trivial (DST, leap seconds, ambiguous local times). Each
  frame is internally consistent. The boundary between them is where mistakes
  happen.
- **Source**: <https://docs.rs/chrono/latest/chrono/struct.DateTime.html> (see
  `MappedLocalTime` and DST handling docs)

#### `serde-untagged-enum-first-match-wins`
- **Class**: Frame-translation
- **What went wrong**: serde's `#[serde(untagged)]` enums attempt deserialization
  variant-by-variant in declaration order, returning the first that succeeds.
  When variants overlap in their JSON shape (e.g., a struct variant whose fields
  are all optional and a "fallback" variant that accepts any object), the JSON
  frame and the Rust enum frame disagree about which variant is intended. Errors
  collapse to the unhelpful "data did not match any variant of untagged enum" —
  the boundary loses all the per-variant diagnostic information.
- **Why it fits**: JSON's frame ("which schema does this match?") and Rust's
  frame ("which variant did the user mean?") diverge when shapes overlap.
  The translation is order-dependent and produces wrong answers silently.
- **Source**: <https://github.com/serde-rs/serde/issues/2066>,
  <https://github.com/serde-rs/json/issues/1155>,
  <https://www.gustavwengel.dk/serde-untagged-enum-errors-are-bad>

#### `tokio-broadcast-clone-not-sync`
- **Class**: Frame-translation
- **What went wrong** (RUSTSEC-2025-0023): tokio's `broadcast` channel cloned
  values across receiver threads but only required `T: Send`, not `T: Sync`.
  In the channel author's frame, "we send `T` to multiple receivers" felt like
  a `Send` operation; in the type-system frame, the actual operation was a
  parallel `Clone::clone(&T)` from multiple threads, which requires `Sync`.
  The two frames look identical for `Sync` types and diverge for `Send + !Sync`
  types.
- **Why it fits**: The "it's a channel, channels are about Send" frame and the
  "we're calling `&T -> T` from multiple threads, that's Sync" frame disagree
  exactly where the bug bites. Bonander's report named the disagreement
  precisely.
- **Source**: <https://rustsec.org/advisories/RUSTSEC-2025-0023.html>,
  <https://github.com/tokio-rs/tokio/pull/7232>

---

## 2. Forgotten-lesson

> A failure-class motivates a fix; the fix lands; the *memory* of why was needed
> stays in someone's head. New types in the same family inherit the design but
> not the immunity.

### Instances

#### `mem-uninitialized-into-maybe-uninit`
- **Class**: Forgotten-lesson
- **What went wrong**: `std::mem::uninitialized::<T>()` was deprecated in 1.39
  in favor of `MaybeUninit<T>` because the `&mut T` to uninitialized memory it
  produced was instant UB for almost every `T`. The lesson — "you cannot have
  a typed reference to uninitialized memory" — is now encoded in `MaybeUninit`'s
  API. But every new `unsafe` code site that wants "just an empty buffer" still
  reaches for the equivalent shape (`Vec::set_len`, raw arrays, etc.) and
  re-derives the lesson. The fix lives in one type; the failure class is a
  family.
- **Why it fits**: The corrective design (MaybeUninit) carries no marker that
  says "if you're touching uninitialized memory in any other shape, the same
  hazard applies." New shapes in the family inherit the *availability* of
  unsafe but not the *immunity* against the original failure mode.
- **Source**: <https://doc.rust-lang.org/std/mem/fn.uninitialized.html>,
  <https://rust-lang.github.io/rfcs/1892-uninitialized-uninhabited.html>

#### `mutex-guard-sync-auto-trait-leak`
- **Class**: Forgotten-lesson (with implicit-coupling secondary)
- **What went wrong**: `MutexGuard<T>` was accidentally `Sync` for `T: Send`
  because `Sync` is an auto-trait derived from fields. With rayon scope, two
  threads could call `Cell::set` through the same `&MutexGuard` — a data race
  reachable from purely safe code. The lesson is "auto-traits over `unsafe`
  invariants are dangerous"; the corrective `unsafe impl !Sync` was added.
  Every new wrapper around interior-mutability primitives must rederive the
  lesson — it does not propagate structurally.
- **Why it fits**: The patch is local to `MutexGuard`. The class — "auto-traits
  silently leak through to library types whose internal invariants don't tolerate
  them" — has no encoded memory. Issue #43981 explicitly calls out that
  auto-Sync is harmful for the family, but the family-level marker doesn't exist.
- **Source**: <https://www.ralfj.de/blog/2017/06/09/mutexguard-sync.html>,
  <https://github.com/rust-lang/rust/issues/43981>

#### `leakpocalypse-scoped-threads`
- **Class**: Forgotten-lesson
- **What went wrong**: Pre-1.0 `thread::scoped` was sound under the assumption
  that `JoinGuard`'s destructor *would run*. `Rc` cycles + `mem::forget` (which
  is safe) broke that assumption. The library was removed; the lesson — "destructors
  cannot be relied on for soundness; treat `Drop` as best-effort and design
  invariants around `mem::forget` being legal" — became foundational. Crossbeam
  and later `std::thread::scope` were redesigned to not require destructor
  execution. New `Drop`-based RAII patterns recurrently rediscover the lesson;
  there is no structural marker that says "this pattern must not depend on
  Drop running."
- **Why it fits**: The corrected design (current `std::thread::scope`) carries
  the lesson in its shape, but only inside that one API. Every new "I'll use
  Drop to release a resource" library has to re-learn that the pattern is fine
  for cleanup but not for soundness.
- **Source**: <https://cglab.ca/~abeinges/blah/everyone-poops/>,
  <https://github.com/rust-lang/rust/issues/24292>

#### `actix-web-unsound-cell-redux`
- **Class**: Forgotten-lesson
- **What went wrong**: actix-web's `actix-service` shipped a bespoke `Cell`
  type that handed out multiple mutable references — exactly the failure-class
  that motivates `RefCell`'s runtime checks and the entire borrow checker.
  The lesson was widely known in the ecosystem, but the file was a fresh
  context where the lesson had no anchor; the maintainer's frame ("I want
  zero-cost interior mutability") routed past the lesson. The community
  blow-up that followed was about the recurrence as much as the bug.
- **Why it fits**: The "interior mutability needs aliasing discipline" lesson
  is foundational and dates to pre-1.0 Rust. Yet a high-profile crate
  re-derived a broken version. The lesson lived in human memory and in the
  shape of `RefCell`/`Cell`/`UnsafeCell`, but no structural marker said "any
  new `Cell`-shaped wrapper must satisfy these invariants."
- **Source**: <https://github.com/actix/actix-web/issues/289>,
  <https://github.com/actix/actix-web/pull/968>,
  <https://steveklabnik.com/writing/a-sad-day-for-rust>

---

## 3. Implicit-coupling

> Changes to A break B through unstated dependency. A's behavior changed in a
> way "obviously fine" from A's perspective but B was relying on the old
> behavior.

### Instances

#### `rust-1.80-fromiterator-box-str-inference-break`
- **Class**: Implicit-coupling
- **What went wrong**: Rust 1.80 added `impl FromIterator<...> for Box<str>`.
  This was a non-breaking addition by stdlib's rules. But thousands of
  downstream crates' type inference relied on there being *exactly one* matching
  impl in scope, including (notoriously) every version of the `time` crate
  before 0.3.55. Adding the impl made inference ambiguous and broke 5000+
  crates. The new impl was "obviously fine" from stdlib's frame; downstream
  was implicitly coupled to the *absence* of additional impls.
- **Why it fits**: The dependency is unstated and arises from inference,
  not from any explicit API contract. The change is a textbook minor-version
  addition that breaks consumers via a coupling no one named.
- **Source**: <https://devclass.com/2024/08/19/rust-1-80-0-breaks-existing-code-such-as-time-crate-exposes-compatibility-snag-with-type-inference/>,
  <https://internals.rust-lang.org/t/type-inference-breakage-in-1-80-has-not-been-handled-well/21374>

#### `cargo-feature-unification-default-leak`
- **Class**: Implicit-coupling
- **What went wrong**: When crate A specifies `default-features = false` for
  dependency D, and crate B (also in the build graph) does not, cargo's
  feature unification turns D's default features back on for everyone. A
  "obviously fine" addition to B (just depending on D normally) silently
  re-enables features in D that A explicitly disabled. The coupling is the
  *shared* feature set across the entire build graph; neither A nor B
  declares it.
- **Why it fits**: The coupling is global to the build graph and unstated in
  any single Cargo.toml. RFC 2957 (resolver v2) and RFC 3692 (per-workspace
  feature unification) exist exactly because this implicit coupling produces
  non-local breakage.
- **Source**: <https://doc.rust-lang.org/cargo/reference/features.html>,
  <https://rust-lang.github.io/rfcs/3692-feature-unification.html>

#### `optional-dependency-implicit-feature`
- **Class**: Implicit-coupling
- **What went wrong**: Until `dep:` syntax (Rust 2021 / cargo features2),
  declaring an optional dependency D implicitly created a feature named D.
  Downstream crates could enable that feature by name. Removing the optional
  dependency from A — which A's author treats as an internal refactor — broke
  every downstream that had written `features = ["d"]`. The feature was never
  declared by A; it existed by virtue of the optional dependency's name.
- **Why it fits**: A's "internal" change (drop a dep) breaks B through a
  coupling A never declared. The fix (`dep:` syntax) makes the coupling
  explicit, but every crate predating it carries the implicit version.
- **Source**: <https://rust-lang.github.io/rfcs/2957-cargo-features2.html>,
  <https://predr.ag/blog/semver-in-rust-tooling-breakage-and-edge-cases/>

#### `tokio-runtime-coupling-via-feature-flag`
- **Class**: Implicit-coupling
- **What went wrong**: A library that internally calls `tokio::spawn` or uses
  `tokio::sync::Mutex` is implicitly coupled to (a) the tokio runtime being
  the executor, and (b) tokio's specific feature set. Library authors often
  expose this only through a Cargo feature like `tokio` or `runtime-tokio`,
  but in practice the library also fails to work inside async-std, smol, or
  embassy — the runtime coupling is part of the call graph, not the
  feature graph. webrtc-rs's announcement of v0.17 as the *final* tokio-coupled
  release explicitly called out this pattern.
- **Why it fits**: The user reads "I depend on this library" and infers no
  runtime constraint; the library actually requires a specific runtime to be
  ambiently installed. The dependency is real but unstated in the type system.
- **Source**: <https://corrode.dev/blog/async/>,
  <https://webrtc.rs/blog/2026/01/31/webrtc-v0.17.0-feature-freeze-sansio-shift.html>

#### `semver-pub-use-of-dependency`
- **Class**: Implicit-coupling
- **What went wrong**: A crate that does `pub use foo_dep::SomeType;` makes
  `SomeType` part of its public API. When `foo_dep` makes a major version
  bump, the host crate's API silently breaks: callers see a different type
  even though the host's own version didn't change. The coupling — that
  the host's public API now contains a foreign type whose stability is not
  managed by the host — is implicit in the `pub use`.
- **Why it fits**: The `pub use` line states "I expose this name." It does
  not state "I have made my crate's semver constraints transitively dependent
  on `foo_dep`'s." The breakage is downstream of an unstated structural
  property.
- **Source**: <https://predr.ag/blog/semver-in-rust-tooling-breakage-and-edge-cases/>,
  <https://github.com/rust-lang/cargo/issues/8736>

---

## 4. Stale-context

> Using outdated information confidently. Substrate-over-memory failure: the
> developer (or compiler, or build system) trusts a cached/inferred state
> rather than checking current reality.

### Instances

#### `cratedepression-rustdecimal-typosquat`
- **Class**: Stale-context (with boundary-violation secondary)
- **What went wrong**: The malicious `rustdecimal` crate typosquatted on
  `rust_decimal` (3.5M+ downloads). Developers reaching for the well-known
  crate from memory, without re-checking the canonical name, pulled a
  malicious package that exfiltrated CI environment state. The "stale
  context" is the developer's memory of the crate's name vs the registry's
  current state.
- **Why it fits**: The class is exactly "trusting your model of what's true
  rather than checking the substrate." The substrate (crates.io) has both
  packages; the memory has only the well-known one; the typo lands in the
  malicious one. Cargo provides no friction at the trust boundary.
- **Source**: <https://www.sentinelone.com/labs/cratedepression-rust-supply-chain-attack-infects-cloud-ci-pipelines-with-go-malware/>,
  <https://flyingduck.io/blogs/typosquatting-attack-rust-crate>

#### `faster-log-async-println-supply-chain`
- **Class**: Stale-context
- **What went wrong** (Sept 2025): Two malicious crates `faster_log` and
  `async_println` mimicked the legitimate `fast_log`, copying source,
  features, and docs verbatim, and added code that scanned `.rs` files for
  Solana/Ethereum private keys and exfiltrated them. 8,424 downloads
  before takedown. Developers who installed them had outdated context about
  which name was canonical.
- **Why it fits**: Same shape as `rustdecimal` — the developer's mental model
  of the crate name is the stale context; the malicious crate exploits the
  gap between memory and substrate.
- **Source**: <https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/>,
  <https://thehackernews.com/2025/09/malicious-rust-crates-steal-solana-and.html>

#### `rust-1.80-time-crate-stale-pin`
- **Class**: Stale-context
- **What went wrong**: Many projects had `time = "0.3.x"` for some x < 55.
  Builds worked fine for years on Rust ≤ 1.79. When Rust 1.80 shipped, those
  builds broke because the pinned `time` version was incompatible with the
  new compiler. The "stale context" is the lockfile's confident pin to an
  old `time` whose constraints were quietly false on a newer rustc.
- **Why it fits**: The build graph had encoded knowledge ("`time` 0.3.30 works")
  that became stale silently. The freshness was never re-verified at the
  rustc/dependency boundary.
- **Source**: <https://github.com/brycx/pasetors/issues/129>,
  <https://devclass.com/2024/08/19/rust-1-80-0-breaks-existing-code-such-as-time-crate-exposes-compatibility-snag-with-type-inference/>

#### `openssl-sys-vendored-stale-build-cache`
- **Class**: Stale-context
- **What went wrong**: openssl-sys' build script reads a long list of
  environment variables (`OPENSSL_DIR`, `OPENSSL_NO_VENDOR`, vendored
  feature flag, target-prefixed variants). Cached build artifacts and
  CI environments routinely encode the *previous* build's environment;
  changing the runtime environment (e.g., switching CI runners) without
  invalidating the cache produces builds that link against the wrong
  OpenSSL — sometimes silently producing TLS that "works" until it
  doesn't. The build script's input is a moving target; the cache
  remembers a stale one.
- **Why it fits**: The build's correctness depends on environment that
  the build artifact doesn't fully encode in its fingerprint.
- **Source**: <https://github.com/rust-openssl/rust-openssl/issues/1430>,
  <https://github.com/sfackler/rust-openssl/issues/1398>

---

## 5. Premature-abstraction

> Generalized too early; the abstraction was made against limited evidence and
> is now load-bearing for code that doesn't fit.

### Instances

#### `mem-uninitialized-original-design`
- **Class**: Premature-abstraction (with forgotten-lesson dual)
- **What went wrong**: `std::mem::uninitialized::<T>()` was designed when the
  evidence was "C lets you have uninitialized stack slots, Rust should too."
  The abstraction "produce a `T` from nothing, unsafely" was made before the
  language had a clear story for invalid bit patterns and validity invariants.
  Once `bool`, `&T`, `enum` validity invariants were nailed down, every
  call site of `mem::uninitialized` for non-trivial `T` was retroactively UB.
  The deprecation note explicitly calls this "one of the worst mistakes of
  the language."
- **Why it fits**: The abstraction was load-bearing across the ecosystem
  before its invariants were understood. Replacing it required a separate
  type (`MaybeUninit`) because the original signature was unsalvageable.
- **Source**: <https://doc.rust-lang.org/std/mem/fn.uninitialized.html>,
  <https://rust-lang.github.io/rfcs/1892-uninitialized-uninhabited.html>

#### `unsafe-destructor-blind-to-params-attr`
- **Class**: Premature-abstraction
- **What went wrong**: When dropck was being designed for collections, the
  attribute `unsafe_destructor_blind_to_params` was introduced as a "short-term
  band-aid" for a single class of types (drop impls that don't access generic
  params). It became load-bearing for `Vec`, `HashMap`, etc., before the
  more principled `#[may_dangle]` / `dropck_eyepatch` design was finished.
  The original attribute's RFC explicitly says it's a workaround. It then
  shipped.
- **Why it fits**: Generalization was made on partial evidence; the abstraction
  became load-bearing before the right design was in place. Subsequent fixes
  (RFC 1238, RFC 1327, the `[T; 0]` dropck issue #110288) trace back to that
  premature commitment.
- **Source**: <https://rust-lang.github.io/rfcs/1238-nonparametric-dropck.html>,
  <https://rust-lang.github.io/rfcs/1327-dropck-param-eyepatch.html>,
  <https://github.com/rust-lang/rust/issues/110288>

#### `trustedlen-overpromised-unsafe-contract`
- **Class**: Premature-abstraction
- **What went wrong**: `TrustedLen` was added as an internal trait so
  `FromIterator` / `Extend` could pre-allocate exact capacity. The contract
  ("`size_hint` is exact or larger-than-`usize::MAX`") was specified before
  iterator adapters were fully analyzed. `Skip<I>` and other adapters that
  shorten an iterator can't soundly implement `TrustedLen`, but the line
  between "trustworthy" and "not" turns out to be subtle. Issue #89948
  documents `SpecExtend`-via-`TrustedLen` as outright unsound.
- **Why it fits**: The trait formalized "trust the iterator" before the
  set of iterators that can be trusted was characterized; soundness holes
  followed.
- **Source**: <https://github.com/rust-lang/rust/issues/89948>,
  <https://doc.rust-lang.org/std/iter/trait.TrustedLen.html>

#### `pin-via-coercion-unsoundness`
- **Class**: Premature-abstraction
- **What went wrong** (Issue #153438, March 2026): The `pin!()` macro was
  designed assuming `Pin` coercions through `&mut` were safe. Self-referential
  types violate the uniqueness guarantee of `&mut`, and the macro's reliance
  on coercion let `Drop` impls observe the unpinned reference, breaking the
  Pin invariant. The async-block workaround (carve out `&mut` uniqueness when
  `T: !Unpin`) is itself "unsound-but-safe-from-miscompilation" by design.
- **Why it fits**: `Pin`'s abstraction was stabilized for async/await before
  self-reference's interaction with `&mut` uniqueness was settled in the
  unsafe-code-guidelines. The abstraction is now load-bearing for the entire
  futures ecosystem; the carve-outs accumulate.
- **Source**: <https://github.com/rust-lang/rust/issues/153438>,
  <https://github.com/rust-lang/unsafe-code-guidelines/issues/148>

#### `gat-stabilized-with-known-limitations`
- **Class**: Premature-abstraction (mild)
- **What went wrong**: GATs stabilized in 1.65 with the lang team explicitly
  noting "limitations in the initial stabilization that we plan to remove in
  the future." Concerns were raised that downstream patterns (lending
  iterators, async traits) would lock in shapes that the future GATs design
  couldn't gracefully extend. So far the bet has held — but the structural
  pattern of "stabilize partial design, extend later" *is* the premature-
  abstraction shape, and lessons from `mem::uninitialized` and
  `unsafe_destructor_blind_to_params` say it occasionally fails.
- **Why it fits**: This is the inverse case — premature-abstraction risk
  evaluated and accepted, with explicit hooks for later extension. Worth
  cataloging as evidence that the failure class can be *mitigated* by
  naming it ahead of time. (Whether the bet pays off long-term is open.)
- **Source**: <https://blog.rust-lang.org/2022/10/28/gats-stabilization/>,
  <https://github.com/rust-lang/rust/issues/44265>

---

## 6. Incompatible-merger

> Two correct things combined produce wrong things — autoimmunity-shape: the
> components look like they belong together but don't.

### Instances

#### `rc-plus-mem-forget-equals-leakpocalypse`
- **Class**: Incompatible-merger
- **What went wrong**: `Rc<T>` and `mem::forget` are each correct in isolation.
  Their combination — `Rc` cycles let you retain `JoinGuard` indefinitely
  without running its destructor — broke pre-1.0 `thread::scoped`, which had
  *also* been correct in isolation under the (now-known-false) assumption that
  `Drop` runs. The composition is the bug, not any single piece.
- **Why it fits**: Three components (Rc, mem::forget, scoped threads), each
  individually sound, compose into UB. Each was reasoned about in its own
  frame; the merger crossed all of them.
- **Source**: <https://cglab.ca/~abeinges/blah/everyone-poops/>,
  <https://github.com/rust-lang/rust/issues/24292>

#### `mutex-cell-rayon-data-race`
- **Class**: Incompatible-merger
- **What went wrong**: `Mutex<Cell<i32>>` is fine. `rayon::join` is fine.
  `MutexGuard<Cell<i32>>` being auto-`Sync` is the bug — the *merger* of
  these correct components produces a data race in safe code. The fix
  (`unsafe impl !Sync for MutexGuard`) is a composition-level constraint
  that did not exist in any single component.
- **Why it fits**: Each component is well-tested in isolation. The merger
  produces a wrong thing because no component is responsible for the
  cross-cutting invariant (Sync over interior-mutable state).
- **Source**: <https://www.ralfj.de/blog/2017/06/09/mutexguard-sync.html>

#### `tokio-block-in-place-plus-block-on-plus-mutex`
- **Class**: Incompatible-merger
- **What went wrong** (Issue #7892): Each of `tokio::task::block_in_place`,
  `Handle::block_on`, and `tokio::sync::Mutex` is correct in its documented
  use. Their *combination* — using `block_in_place` to enter a sync context,
  calling `block_on` from inside, and acquiring a `tokio::sync::Mutex` —
  reliably deadlocks because tokio's runtime semantics across all three
  don't compose. tokio maintainers explicitly note the runtime is "behaving
  correctly" in each, just not jointly.
- **Why it fits**: Three locally-correct components compose into a deadlock.
  No single piece is buggy; the merger is.
- **Source**: <https://github.com/tokio-rs/tokio/issues/7892>,
  <https://www.e6data.com/blog/deadlocking-tokio-mutex-without-holding-lock>

#### `catch-unwind-plus-poisoned-mutex`
- **Class**: Incompatible-merger
- **What went wrong**: `Mutex` poisoning relies on the panicking thread
  *aborting* (or at least propagating the panic) so observers see the
  poisoned state and react. `catch_unwind` is sound on its own. Their
  combination breaks the mutex's invariant: a thread can take a lock,
  start mutating shared state, panic, have `catch_unwind` swallow the
  panic, *not* unwind out of the lock guard's drop, and continue. The
  poison flag fires, but the application keeps running with corrupted
  shared state.
- **Why it fits**: `catch_unwind` is correct. `Mutex` poisoning is correct.
  Together they break the structural assumption ("a poisoned mutex's data
  is observed by someone who treats it as suspect") that neither component
  states.
- **Source**: <https://sunshowers.io/posts/on-poisoning/>,
  <https://matklad.github.io/2020/12/12/notes-on-lock-poisoning.html>,
  <https://github.com/rust-lang/rust/issues/86027>

#### `vendored-openssl-plus-system-library-link-mismatch`
- **Class**: Incompatible-merger
- **What went wrong**: `openssl-sys`' `vendored` feature builds a static
  OpenSSL into the final binary. Other crates in the same build graph may
  link to the system OpenSSL (via different `*-sys` shims). Both decisions
  are individually fine. Linked together, you get a binary with two OpenSSL
  versions, leading to symbol clashes, ABI mismatches, or runtime "works
  on my machine" behavior.
- **Why it fits**: Two correct linking choices, made independently, produce
  a broken composition. No `Cargo.toml` declares the cross-component
  invariant ("only one OpenSSL per binary").
- **Source**: <https://github.com/rust-openssl/rust-openssl/issues/1398>,
  <https://medium.com/rustaceans-security/that-vendored-openssl-most-of-us-rely-on-probably-needs-a-patch-aae8fea5160f>

---

## 7. Boundary-violation

> Trust-boundary check skipped at a structural boundary. Unchecked input
> poisons the downstream.

### Instances

#### `hyper-content-length-plus-prefix`
- **Class**: Boundary-violation
- **What went wrong** (RUSTSEC-2021-0078, CVE-2021-32715): hyper's HTTP/1
  parser accepted `Content-Length: +123` instead of rejecting per RFC 7230.
  When hyper sat behind a proxy that *did* reject the leading `+`, the two
  parsers disagreed about message boundaries, enabling request smuggling.
  The boundary in question is the trust boundary between proxy and origin
  server; the unchecked content-length poisons every request after.
- **Why it fits**: The parser's trust boundary (HTTP-spec-compliant input)
  was not enforced at the structural site (header parsing). Each downstream
  message inherits the corrupted framing.
- **Source**: <https://rustsec.org/advisories/RUSTSEC-2021-0078.html>,
  <https://github.com/advisories/GHSA-6hfq-h8hq-87mf>

#### `hyper-multiple-transfer-encoding`
- **Class**: Boundary-violation
- **What went wrong** (RUSTSEC-2021-0020 / CVE-2021-21299): hyper accepted
  multiple `Transfer-Encoding` headers and treated the request as chunked
  even when an upstream proxy interpreted the framing differently. Same
  shape as RUSTSEC-2021-0078: trust-boundary check at the parser layer was
  weaker than the proxy's, enabling desync.
- **Why it fits**: Identical structural shape — two parsers, one trusts more
  permissively, the boundary leaks framing.
- **Source**: <https://rustsec.org/advisories/RUSTSEC-2021-0079.html>,
  <https://blog.firosolutions.com/exploits/request-smuggling-rust-hyper/>

#### `zip-rs-zip-slip-symlink-traversal`
- **Class**: Boundary-violation
- **What went wrong** (CVE-2025-29787): The `zip` crate extracted entries
  whose paths contained `..` or symlinks resolving outside the target
  directory. The trust-boundary check ("entry path stays inside extraction
  root") was implemented but did not cover symlinks. Malicious archives
  could write arbitrary files. Same class as the JVM zip-slip, the Python
  zip-slip, etc.
- **Why it fits**: A structural boundary (filesystem extraction root) had
  a check that wasn't sound for the full input space (symlinks). The
  unchecked input goes downstream and writes files outside the trust zone.
- **Source**: <https://www.sentinelone.com/vulnerability-database/cve-2025-29787/>,
  <https://security.snyk.io/vuln/SNYK-RUST-ZIP-9460813>

#### `sudo-rs-path-traversal`
- **Class**: Boundary-violation
- **What went wrong** (RUSTSEC-2023-0069): sudo-rs had a path-traversal
  vulnerability in privileged file resolution. A trust boundary
  (privilege escalation surface) had a path check that didn't account
  for some edge cases; user-controlled input crossed the boundary
  unsanitized.
- **Why it fits**: Privileged sudo is one of the highest-stakes trust
  boundaries in a Unix system; the structural check failed at exactly
  that site.
- **Source**: <https://rustsec.org/advisories/RUSTSEC-2023-0069.html>

#### `serde-derive-precompiled-binary-trust-expansion`
- **Class**: Boundary-violation
- **What went wrong**: serde_derive 1.0.171–1.0.184 shipped a precompiled
  `serde_derive_x86_64-unknown-linux-gnu` binary inside the source crate.
  Cargo invokes proc-macros at compile time; running an opaque pre-built
  binary at compile time crosses a trust boundary that source-distributed
  crates traditionally don't cross. The maintainer's frame ("we already
  trust the source code, why not the binary?") collapsed the distinction
  the boundary was protecting. Reverted under community pressure.
- **Why it fits**: A trust boundary (source review at compile time) was
  invisibly weakened by routing through a pre-built artifact. The check
  ("did a human read the code?") that defenders rely on was structurally
  removed.
- **Source**: <https://news.ycombinator.com/item?id=37189462>,
  <https://github.com/serde-rs/serde/issues/2538>

---

## 8. Optionality-collapse

> Conditional structure becomes unconditional through routing. e.g., team-lead
> routes "lean X but Y's call" and downstream sees "team-lead said X."
> Information loss in composition.

> **Coverage note**: This is the *hardest* class to populate from public
> bug reports. Most public bugs are bug-shaped (something panics, leaks,
> mis-compiles). Optionality-collapse is more often *miscommunication-shaped*
> and lives in design discussion or post-hoc reflection. The instances below
> stretch toward the structural shape; some are weaker fits than entries
> in other classes.

### Instances

#### `result-question-mark-error-conversion-loss`
- **Class**: Optionality-collapse
- **What went wrong**: The `?` operator calls `From::from` on the error
  type, converting (say) `io::Error` into a domain `Error` enum. If the
  destination enum has a catch-all variant (`Error::Other(String)` via a
  generic blanket `From`), the structural information about *why* the
  underlying error matters — its `kind()`, its source chain, its retryability —
  collapses into a string. Downstream code receives "Error::Other(\"connection refused\")"
  instead of `io::ErrorKind::ConnectionRefused`. The conditional ("if this is
  a retryable network error, retry") becomes unconditional ("treat all errors
  the same"). The thiserror/anyhow ecosystems both have failure modes around
  this.
- **Why it fits**: The original error carries conditional structure (variant
  tag, kind, source chain). The `?`-driven conversion through a too-general
  `From` impl flattens it. The downstream sees a unified shape and routes
  uniformly, losing the option to discriminate.
- **Source**: <https://www.lpalmieri.com/posts/error-handling-rust/>,
  <https://mmapped.blog/posts/12-rust-error-handling>

#### `must-use-let-underscore-discard`
- **Class**: Optionality-collapse
- **What went wrong**: `Result<T, E>` is `#[must_use]`. The compiler will warn
  if you ignore one. But `let _ = expr;` silences the warning, and many code
  reviewers and authors use it as a "yes, I considered this" marker.
  Structurally, the conditional ("either succeeded or failed; you must
  branch") collapses into an unconditional ("we got past this line"). The
  type system carried the option; the routing through `_` discards it.
  This is *the* shape of the class: the option is preserved at the source
  but lost at the routing site.
- **Why it fits**: Conditional structure (`Result` is fundamentally a sum
  type carrying two paths) becomes unconditional through a routing
  operator (`let _ = ...`) that doesn't preserve the conditional shape
  in any structural marker downstream.
- **Source**: <https://github.com/rust-lang/rust-clippy/issues/22>,
  <https://users.rust-lang.org/t/what-is-the-best-way-to-ignore-a-result/55187>

#### `untagged-enum-variant-collapse`
- **Class**: Optionality-collapse (with frame-translation secondary)
- **What went wrong**: serde's `#[serde(untagged)]` enum deserialization
  preserves the conditional structure ("data is variant A or variant B")
  in the source JSON, but at the boundary it routes through "first match
  wins" and emits a single variant. Downstream code never sees that
  variant B *also* matched (which it might have, ambiguously). The
  conditional ("which variant?") becomes unconditional ("variant A,
  full stop"). When the user's intent was variant B, the bug is silent.
- **Why it fits**: The structural option (multi-variant ambiguity) exists
  in the input and is collapsed by the routing layer. Information that
  could have been preserved (e.g., a `Tied(A, B)` variant or an ambiguity
  diagnostic) is discarded.
- **Source**: <https://github.com/serde-rs/serde/issues/2066>,
  <https://users.rust-lang.org/t/serde-untagged-enum-ruins-precise-errors/54128>

#### `option-unwrap-or-default-silent-substitution`
- **Class**: Optionality-collapse
- **What went wrong**: `Option<T>::unwrap_or_default()` silently substitutes
  `T::default()` for `None`. Downstream code receives a `T` with no
  structural marker that the value was synthesized rather than provided.
  The conditional ("present or absent") becomes unconditional ("a `T`,
  trust it"). For numeric types, `Default::default() == 0`, which is
  often the worst silent substitution (zero in a denominator, zero in
  a count, zero as a sentinel for missing data).
- **Why it fits**: The structural option (Some/None) is collapsed by a
  routing operator that emits a unified type. Empirically, `unwrap_or_default()`
  in data-pipeline code has been the source of "silent zeros propagate
  downstream" bugs in many domains.
- **Source**: <https://corrode.dev/blog/rust-option-handling-best-practices/>

---

## Coverage assessment

### Easy to populate (5+ strong instances available)
- **Boundary-violation** — Web frameworks, archive parsers, sudo-rs, serde
  precompiled binaries. Many CVEs land here directly.
- **Implicit-coupling** — Cargo features, semver, async-runtime coupling,
  rust-1.80 inference break. Multiple high-impact, well-documented instances.
- **Incompatible-merger** — Composition bugs are common in async + locking;
  also classic `Rc` + `mem::forget`, `MutexGuard` + auto-traits.

### Medium to populate (3-4 strong instances)
- **Stale-context** — Supply chain typosquats fit perfectly; build-cache
  staleness fits structurally.
- **Frame-translation** — Path semantics, time/timezone, untagged enum
  shape mismatches. Fewer pure CVE-style instances; more design-pattern
  failures.
- **Forgotten-lesson** — Strong canonical instances (`mem::uninitialized`,
  scoped threads, MutexGuard Sync, actix-web Cell). The class is real but
  populated mostly by *historical* fixes rather than recurring CVEs.
- **Premature-abstraction** — Strong instances exist in stdlib history;
  fewer in third-party crates because authors typically ship abstractions
  with limitations declared.

### Hard to populate (≤3 instances, structural fit weaker)
- **Optionality-collapse** — This is the hardest class to find clean
  public examples for. The shape is real (we've all seen it in design
  discussions and post-mortems) but it tends not to surface as a *bug
  report* — it surfaces as a *design regret*. Most public Rust bugs are
  shaped as "X panics" or "Y leaks" or "Z is UB," not "the option that
  should have remained in the type system was structurally erased."
  The strongest fits (`?`-error-conversion loss, `let _ =`
  discard, `unwrap_or_default` substitution) are pattern-level rather
  than incident-level.

### Recommendations for the taxonomy

1. **Optionality-collapse may need refinement.** It's a real failure shape,
   but its public visibility is low. Two possibilities: (a) the class is
   correctly named and just under-reported (as antigen would predict —
   structural memory is exactly what's missing for this class); (b) the
   class shape is too broad and would benefit from sub-classes
   (e.g., "error-information-loss," "default-value-substitution,"
   "ambiguity-resolution-via-ordering"). Worth a JBD-team conversation
   before locking. The fact that it's underrepresented in bug reports may
   be the *signal that antigen is needed for this class specifically*.

2. **Forgotten-lesson and premature-abstraction are duals.** The same fix
   often instantiates both: "the lesson was learned (forgotten-lesson) and
   the abstraction that learned it was committed too early (premature-
   abstraction)." `mem::uninitialized` is the canonical example.
   The taxonomy may want to acknowledge this duality rather than treating
   them as orthogonal.

3. **Frame-translation has a strong type-systems flavor; consider sub-typing.**
   Path/timezone/parser-disagreement frames are a coherent sub-shape;
   trait-bound mismatches across crate boundaries (orphan rule, coherence)
   are another; serialization-format mismatches (untagged enums) are a
   third. These might warrant sub-class names for clarity.

4. **Boundary-violation occasionally overlaps with stale-context.** Supply
   chain attacks especially can fit either or both. Recommendation: file
   under boundary-violation if the failure is "input crossed the boundary
   unchecked," under stale-context if the failure is "the developer's
   mental model of the substrate was wrong." The CrateDepression incident
   sits primarily in stale-context; the zip-slip and HTTP smuggling
   incidents sit primarily in boundary-violation.

5. **Implicit-coupling has the cleanest boundary.** It's well-defined and
   well-populated. Use it as the reference shape when calibrating other
   classes.

### Summary statistics

| Class | Instances catalogued | Strength |
|---|---|---|
| 1. Frame-translation | 4 | Medium-strong |
| 2. Forgotten-lesson | 4 | Strong (historical canon) |
| 3. Implicit-coupling | 5 | Strong |
| 4. Stale-context | 4 | Strong |
| 5. Premature-abstraction | 5 | Strong (historical canon) |
| 6. Incompatible-merger | 5 | Strong |
| 7. Boundary-violation | 5 | Very strong (CVE-rich) |
| 8. Optionality-collapse | 4 | Pattern-level, weaker incident-level |

**Total: 36 catalogued instances** across the 8 classes, with sources for
each. Sufficient to seed `antigen-stdlib`'s initial declarations and to
ground the taxonomy in real ecosystem evidence. The taxonomy survives the
test: every class can be populated, and the population reveals real
structural shape rather than collapsing into a single "everything is just
a bug" category.
