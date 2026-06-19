# antigen-macros

**Procedural macros for the [`antigen`](https://crates.io/crates/antigen) crate.**

This crate is the proc-macro implementation backing `antigen`. Most users should
depend on [`antigen`](https://crates.io/crates/antigen) directly, which re-exports
these macros. Direct use of `antigen-macros` is supported but not the primary path.

**Core macros** (all re-exported from `antigen`):

- `#[antigen(...)]` — declare a named failure-class with a structural fingerprint
- `#[presents(...)]` — mark code as exhibiting a failure-class pattern; carries
  optional `requires = <predicate>` (substrate-tier) / `proof = <expr>`
  (phantom-tier) site-attached evidence
- `#[defended_by(...)]` — register a test/proptest as the observed code-tier
  witness for a failure-class
- `#[descended_from(...)]` — propagate failure-class markers through structural derivation
- `#[antigen_tolerance(...)]` — mark deliberate exceptions with rationale
- `#[dread]` / `#[aura]` / `#[red_flag]` — record a *felt* unease at a site, so
  `cargo antigen propose` can later anti-unify a candidate fingerprint from the
  cluster of marks

These are the macros most code reaches for; the crate ships a wider vocabulary
(the clinical and clonal families). See the
[`antigen` crate](https://crates.io/crates/antigen) for usage documentation and
the [macro reference](https://github.com/antigen-rs/antigen/tree/main/docs) for
the full set.

## License

Dual-licensed under [MIT](https://github.com/antigen-rs/antigen/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/antigen-rs/antigen/blob/main/LICENSE-APACHE).
