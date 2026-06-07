//! Renders (v0.4 ADR-043 §E) — serializers + transports over the **one**
//! catalog-match spine ([`crate::scan::catalog_match_findings`]).
//!
//! A render takes the unified [`Finding`](crate::finding::Finding) population the
//! spine produces and shapes it for a particular consumer. Renders add **no**
//! second match engine and **no** new verdict — they re-present what the spine
//! already proved, preserving its claim-scope (a fingerprint match stays a
//! scan-fact, never an audited verdict).
//!
//! Shipped renders:
//! - [`session_prime`](crate::render::session_prime) (render D) — a batch digest
//!   for a fresh / compacted agent: group by cluster, rank by severity +
//!   blast-radius, take the top-N.
//! - [`flycheck`](crate::render::flycheck) (render B) — the cargo/rustc
//!   `--message-format=json` serializer so an editor's flycheck consumes antigen
//!   findings as compiler diagnostics, with no custom LSP server.
//!
//! Render A (CLI `--bundled-catalog`) lives in `cargo-antigen`; render C
//! (agent-query MCP) is sequenced after on the v0.4 route (push-gated).

pub mod flycheck;
pub mod session_prime;
