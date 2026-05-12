# antigen-macros

**Procedural macros for the [`antigen`](https://crates.io/crates/antigen) crate.**

This crate is the proc-macro implementation backing `antigen`. Most users should
depend on [`antigen`](https://crates.io/crates/antigen) directly, which re-exports
these macros. Direct use of `antigen-macros` is supported but not the primary path.

**Macros provided** (all re-exported from `antigen`):

- `#[antigen(...)]` — declare a named failure-class with a structural fingerprint
- `#[presents(...)]` — mark code as exhibiting a failure-class pattern
- `#[immune(...)]` — declare immunity with a witness reference
- `#[descended_from(...)]` — propagate failure-class markers through structural derivation
- `#[antigen_tolerance(...)]` — mark deliberate exceptions with rationale

See the [`antigen` crate](https://crates.io/crates/antigen) for usage documentation
and [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen) for the
full project substrate.

## License

Dual-licensed under [MIT](https://github.com/antigen-rs/antigen/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/antigen-rs/antigen/blob/main/LICENSE-APACHE).
