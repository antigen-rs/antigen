# antigen-attestation

Ratification schema + substrate-witness predicate evaluator + Oracle
artifact-class for the antigen failure-class memory ecosystem (ADR-019,
ADR-020, ADR-021).

This crate is the workspace-internal substrate for antigen's discipline-witness
machinery. It is shared between:

- `antigen-macros` (compile-time validation of `requires = ...` predicate
  expressions on `#[immune]` and `#[antigen_tolerance]`)
- `antigen` (audit-time evaluation of substrate-witness predicates against
  `.attest/<Antigen>.json` sidecars)
- `cargo-antigen` (CLI for `cargo antigen attest|tolerate|oracle` subcommand
  families)

Three coupled pieces ship together (per ADR-019 M1, ADR-007 anti-YAGNI):

1. **Predicate language** — closed combinator grammar (`all_of` / `any_of` /
   `not`) over a sealed set of leaf primitives (`signers`, `ratified_doc`,
   `signed_trailer`, `oracles_complete`, `fresh_within_days`).
2. **Ratification schema** — serde-derived single source of truth for
   `.attest/<AntigenName>.json` sidecars. Covers both immunity and tolerance
   ratifications via a `RatificationKind` discriminator. Additive-only
   schema evolution (no migration framework needed per ADR-021).
3. **Oracle artifact-class** — structurally distinguished discipline
   artifacts with state machine (Draft → Complete → Deprecated / Retired /
   Revoked), dedicated stewards, provenance trail, and tagged-union
   OracleRef (file / URL / DOI / arXiv / GitHub issue / other).

Tier-honesty preserved via three-axis output: `WitnessTier × AuditHint ×
EvidenceKind`. Three signature tiers: `TextStamp | GitTrust | CryptoSigned`
(categorical, not ordinal — trust is project-declared per-antigen via
`signers(signature_allow = [...], signature_prefer = ...)`).

For direct use, depend on the `antigen` crate which re-exports the public
API. See [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen)
for the full project substrate, ADR-019 / ADR-020 / ADR-021 in
`docs/decisions.md`, and `docs/witness-tiers.md` for the audit-tier
reference.
