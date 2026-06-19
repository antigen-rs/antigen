# antigen-attestation

**Ratification schema, substrate-witness predicate evaluator, and Oracle
artifact-class for [antigen](https://crates.io/crates/antigen).**

This crate holds antigen's *discipline-witness* machinery — the part that lets a
failure-class defense be backed by ratified evidence on disk rather than a bare
assertion. It is shared between:

- `antigen-macros` — compile-time validation of `requires = ...` predicate
  expressions on `#[presents]` and `#[antigen_tolerance]`.
- `antigen` — audit-time evaluation of substrate-witness predicates against
  `.attest/<Antigen>.json` sidecars.
- `cargo-antigen` — the `cargo antigen attest | tolerate | oracle` subcommands.

Three coupled pieces ship together:

1. **Predicate language** — a closed combinator grammar (`all_of` / `any_of` /
   `not`) over a sealed set of leaf primitives (`signers`, `ratified_doc`,
   `signed_trailer`, `oracles_complete`, `fresh_within_days`). Closed by design:
   an arbitrary predicate cannot be smuggled in.
2. **Ratification schema** — the single serde-derived source of truth for
   `.attest/<AntigenName>.json` sidecars. One schema covers both immunity and
   tolerance ratifications via a `RatificationKind` discriminator, and evolves
   additively (no migration framework).
3. **Oracle artifact-class** — structurally distinguished discipline artifacts
   with a state machine (Draft → Complete → Deprecated / Retired / Revoked),
   dedicated stewards, a provenance trail, and a tagged-union reference type
   (file / URL / DOI / arXiv / GitHub issue / other).

Trust is **categorical, not ordinal**: the three signature tiers
(`TextStamp | GitTrust | CryptoSigned`) are declared per-antigen by the project
(`signers(signature_allow = [...], signature_prefer = ...)`), not ranked by
antigen. Audit output stays tier-honest along three axes —
`WitnessTier × AuditHint × EvidenceKind` — so "not evaluated" never collapses
into "passed."

## Usage

Most users depend on [`antigen`](https://crates.io/crates/antigen), which
re-exports the public API. Depend on `antigen-attestation` directly when you need
the ratification schema or predicate evaluator on their own. See
[`docs/decisions.md`](https://github.com/antigen-rs/antigen/blob/main/docs/decisions.md)
(ADR-019 / ADR-020 / ADR-021) for the schema rationale and
[`docs/witness-tiers.md`](https://github.com/antigen-rs/antigen/blob/main/docs/witness-tiers.md)
for the audit-tier reference.

## License

Dual-licensed under [MIT](https://github.com/antigen-rs/antigen/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/antigen-rs/antigen/blob/main/LICENSE-APACHE).
