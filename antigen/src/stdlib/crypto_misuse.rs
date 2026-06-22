//! # Crypto-Misuse Family — CHARTERED (no shipped member yet)
//!
//! The RUSTSEC `crypto-failure` category seen from the developer side. The
//! load-bearing framing (arxiv 1806.04929 "How Usable are Rust Cryptography
//! APIs?"): Rust crypto libraries mostly *avoid* insecure defaults — so the
//! recurring failure-class is developer **misuse** (reaching past the safe API
//! for the dangerous one, or omitting the safe step), not bad defaults.
//!
//! Biology cognate: **using the immune machinery wrong** — reading a self/
//! non-self marker non-constant-time is the timing leak (the pathogen learning
//! the antibody's exact shape by watching response latency).
//!
//! ## Status: CHARTERED — no honest shipped-grammar fingerprint exists yet
//!
//! The flagship member `NonConstantTimeSecretComparison` (a secret/MAC compared
//! in non-constant time — a timing-attack oracle) is a **real, recurring**
//! failure-class (GHSA-q7pg-9pr4-mrp2 httpsig-rs HMAC timing attack; the RUSTSEC
//! `crypto-failure` category includes non-constant-time operations). **But no
//! honest *call-only* fingerprint can express it in the shipped grammar** — so it
//! is chartered, not shipped, pending the deferred name/operator grammar leaves.
//!
//! **Why there is no honest call-only anchor (confirmed from two independent
//! angles: codomain-reasoning + empirical crate-API verification).** A first
//! attempt anchored on a crypto verify entrypoint
//! (`verify` / `hmac_verify` / `verify_mac`) and fired on the *absence* of a
//! constant-time compare (`not(body_calls("ct_eq"))`). That **anti-aligns with the
//! defect** — it fires loudest on the *safe* path:
//! - `ring::hmac::verify(key, msg, tag)` is the **correct** API and is
//!   constant-time **internally** (the constant-time work is inside `ring`, with
//!   no visible `ct_eq` call). So the fingerprint would **falsely bind a
//!   `ring::hmac::verify` call** — looks undefended, gets flagged — and a named
//!   crate's recommended API is reported as the bug. This is the
//!   clean-sibling-collision shape (cf. the `Instant::elapsed` drop): the anchor's
//!   codomain *includes the clean path*.
//! - `verify` / `hmac_verify` are the **names of the safe operation** (a crypto
//!   lib's `mac.verify(tag)` does the constant-time compare itself), so anchoring
//!   on verify-presence anchors on the safe pattern's *vocabulary*.
//!
//! **The real defect has no distinctive call.** The vulnerable pattern
//! (GHSA-q7pg-9pr4-mrp2) is a **hand-rolled `==` / manual byte-loop on a secret /
//! MAC** — an **operator** (`==`) on a **secret-typed value**, with no
//! crypto-entrypoint call at all. So the honest fingerprint is
//! `all_of([<security-sensitive-name anchor>, not(any_of([body_calls("ct_eq"),
//! body_calls("constant_time_eq")]))])` — and it needs **both deferred leaves**:
//! the `security_sensitive_name` name-leaf (the data-context: does this fn hold
//! secret bytes it might hand-compare?) and the `==` operator-leaf (the precise
//! positive tell, `ExprBinary`). `body_calls` sees only `ExprCall` /
//! `ExprMethodCall` — neither operators nor a data-context. **Neither leaf ships
//! in the current grammar.**
//!
//! ## Graduation path (when the deferred leaves land)
//!
//! When the next grammar increment lands the `security_sensitive_name` name-leaf
//! (queued top-priority) — and ideally the `==` operator-leaf — this family ships
//! `NonConstantTimeSecretComparison` as
//! `all_of([security_sensitive_name, not(any_of([body_calls("ct_eq"),
//! body_calls("constant_time_eq")]))])` at the **suspected / heuristic** tier
//! (`subtle::ct_eq` is the confirmed safe-step needle). Until then it stays
//! charter — **better honest-deferred than dishonest-shipped** (a shipped form
//! would actively mislead by flagging `ring::hmac::verify`, the correct API, as
//! the bug).
//!
//! **Substrate:** GHSA-q7pg-9pr4-mrp2 (the no-call-tell hand-rolled defect) +
//! `ring::hmac::verify` constant-time-internal doc (the anti-correlation proof) +
//! arxiv 1806.04929 (the misuse-not-defaults framing).
