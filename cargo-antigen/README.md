# cargo-antigen

**Cargo subcommand for the [antigen](https://crates.io/crates/antigen) project — scan, build, and apply structural failure-class antibodies in Rust codebases.**

> **Status: Design phase — `0.0.1` reserves the namespace.** Real subcommand
> implementations are under active design at
> [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen).

## What this crate is (intended)

The cargo subcommand companion to [`antigen`](https://crates.io/crates/antigen). Will provide:

- `cargo antigen scan` — find all `#[presents]` markers without corresponding `#[immune]` declarations
- `cargo antigen new <name>` — scaffold a new antigen declaration
- `cargo antigen vaccinate <antigen> <pattern>` — apply a known immunity pattern across a structural family
- `cargo antigen audit` — comprehensive immunity coverage report

## Why a cargo extension

Failure-class memory only works if it's structurally checked by tooling — otherwise it
drifts like documentation. The cargo extension makes scanning, vaccinating, and auditing
first-class development actions, runnable in CI and integrated with normal workflow.

## Installation (when 1.0 ships)

```sh
cargo install cargo-antigen
```

Then `cargo antigen <subcommand>` will be available in any cargo workspace.

## Status

The published `0.0.1` is a placeholder that reserves the binary name and prints a
design-phase notice. See the
[design documents](https://github.com/antigen-rs/antigen/tree/main/docs/expedition) for
the active design.

## License

Dual-licensed under [MIT](https://github.com/antigen-rs/antigen/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/antigen-rs/antigen/blob/main/LICENSE-APACHE).
