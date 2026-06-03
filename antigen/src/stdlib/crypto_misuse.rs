//! # Crypto-Misuse Family — stdlib antigens (beta.2 voyage, ADR-040 grammar)
//!
//! The RUSTSEC `crypto-failure` category seen from the developer side. The
//! load-bearing framing (arxiv 1806.04929 "How Usable are Rust Cryptography
//! APIs?"): Rust crypto libraries mostly *avoid* insecure defaults — so the
//! recurring failure-class is developer **misuse** (reaching past the safe API
//! for the dangerous one, or omitting the safe step), not bad defaults. That is
//! why these are call-anchored, absence-of-the-safe-step tells.
//!
//! Biology cognate: **using the immune machinery wrong** — reading a self/
//! non-self marker non-constant-time is the timing leak (the pathogen learning
//! the antibody's exact shape by watching response latency). Distinct from
//! "no defense" (mucosal undefended-boundary): this is *misuse of a present
//! defense*.
//!
//! ## Antigen-category (ADR-028)
//!
//! Members are `FunctionalCorrectness`: the verb (the comparison, the cipher
//! call) produces a wrong *observable effect* (a timing oracle, a recoverable
//! ciphertext). The witness exercises behaviour (a constant-time-compare proof,
//! a nonce-uniqueness proof) — it does not check substrate.
//!
//! ## How these antigens are evaluated
//!
//! Unlike the supply-chain / vcs / recurrent families (external-substrate,
//! verify-only), crypto-misuse members carry a **syntactic fingerprint** — they
//! are matched by the AST-walking scanner against the call-shaped tell. The
//! tell is `body_calls` on a crypto entrypoint plus the **absence** of the
//! constant-time safe step (`not(body_calls(...))`) — the absence-grammar
//! driver, call-flavored.
//!
//! ## Tier honesty (ADR-039 provenance ladder)
//!
//! `NonConstantTimeSecretComparison` ships at the **heuristic** tier: the
//! `body_calls("verify")` tell *correlates* with a non-constant-time secret
//! comparison without a verifiable-constructable causal demonstration of *this
//! site's* secret-ness (the entrypoint ident-list is a placeholder). Per the
//! ADR-039 §C honest-labeling invariant, the class is admitted but the tier
//! states "heuristic / correlational, not causal" — it sits passive-by-default
//! and dial-gated, never masquerading as `constructable`/`encountered`. The
//! provenance label is surfaced onto the `Finding` (ADR-039 §C) at emit time;
//! it is not declared here. The admitting-specimen (the affinity-pair example)
//! is what an `affinity-pair`-grade `constructable` claim would require — this
//! member ships the pair as its exhibit while remaining honestly heuristic on
//! the entrypoint ident-list.
//!
//! ## Fast-follow (flagged on the campsite, harbor-master ruling)
//!
//! The crypto-compare entrypoint ident-lists (`"verify"` / `"ct_eq"`) are
//! **placeholders**. Firming them via a RUSTSEC `crypto-failure` enumeration
//! (and the deferred `security_sensitive_name` name-content leaf, charter) is a
//! next-increment fast-follow; the member is honest at the heuristic tier with
//! the placeholders in place.

use crate::antigen;

// ============================================================================
// 1. NonConstantTimeSecretComparison
// ============================================================================

/// A secret/MAC/token verified through a non-constant-time comparison — a
/// timing-attack oracle.
///
/// **Where in the wild:** arxiv 1806.04929 — "HMAC `verify()` uses constant-time
/// comparison; its use is NOT enforced, no warnings on the raw-digest getter."
/// Comparing a MAC / token / password-hash with a non-constant-time path leaks
/// the secret one byte of latency at a time. The RUSTSEC `crypto-failure`
/// category explicitly includes non-constant-time operations.
///
/// **Tell:** a crypto verify entrypoint (`verify` / `hmac_verify` / `verify_mac`)
/// is called **without** an adjacent constant-time comparison (`ct_eq` /
/// `constant_time_eq`, the `subtle::ConstantTimeEq` step). The *absence* of the
/// constant-time call is the tell — `all_of([any_of([body_calls("verify"),
/// body_calls("hmac_verify"), body_calls("verify_mac")]),
/// not(any_of([body_calls("ct_eq"), body_calls("constant_time_eq")]))])`. The
/// clean sibling (which *does* call a constant-time compare) is spared by the
/// `not` branch. The entrypoint set is a **wide-net** `any_of` (not single-needle)
/// because `body_calls` matches by last segment: a single `"verify"` needle
/// would silently miss `hmac_verify` / `verify_mac`.
///
/// **Tier:** **heuristic** (ADR-039) — the entrypoint ident-lists are
/// placeholders, correlational not causal. Passive-by-default; security-severity
/// earns surfacing weight even at the heuristic tier.
///
/// **Witness:** a constant-time comparison is present (`subtle::ConstantTimeEq`
/// / `ring`'s constant-time verify), OR a proof the compared value is not
/// secret.
///
/// **Category:** `FunctionalCorrectness` — the comparison verb produces a wrong
/// *observable effect* (a measurable timing differential), not a wrong
/// representation.
#[antigen(
    name = "non-constant-time-secret-comparison",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([any_of([body_calls("verify"), body_calls("hmac_verify"), body_calls("verify_mac")]), not(any_of([body_calls("ct_eq"), body_calls("constant_time_eq")]))])"#,
    family = "crypto-misuse",
    summary = "A crypto verify path (verify / hmac_verify / verify_mac) compares a secret/MAC/token without a constant-time comparison (ct_eq / constant_time_eq) present — a timing-attack oracle. Heuristic tier (placeholder entrypoint ident-lists); the absence of the constant-time step is the tell.",
    references = [
        "https://arxiv.org/abs/1806.04929",
        "RUSTSEC#crypto-failure",
        "ADR-040",
    ]
)]
pub struct NonConstantTimeSecretComparison;
