#!/usr/bin/env bash
# ci-local.sh — run antigen's CI gate locally before pushing.
#
# This MIRRORS .github/workflows/ci.yml exactly (the gate that actually blocks a
# push), so "green here" => "green in CI" for the single-toolchain jobs. It is
# the source of truth for the *local* gate; camp.toml's [gates] is kept aligned
# with it (and with ci.yml). If you change ci.yml, change this and camp.toml too.
#
# What it CAN'T replicate (CI-only, by design): the test matrix across
# {ubuntu, windows, macos} x {stable, beta}. This runs your *current* toolchain
# only. The cross-platform/beta legs still run in CI.
#
# RTK note: cargo/grep/rg/git are excluded from the rtk Bash hook, so cargo
# output is raw — no `command cargo` workaround needed.

set -euo pipefail

# Workflow-level env from ci.yml (applies to every job): warnings are errors.
export RUSTFLAGS="-D warnings"
export CARGO_TERM_COLOR=always

cd "$(dirname "$0")/.."

step() { printf '\n=== %s ===\n' "$1"; }

step "[1/7] check (cargo check --workspace --all-targets)"
cargo check --workspace --all-targets

step "[2/7] fmt (nightly: rustfmt.toml uses unstable style_edition opts)"
cargo +nightly fmt --all -- --check

step "[3/7] clippy (-D warnings)"
cargo clippy --workspace --all-targets -- -D warnings

step "[4/7] test (--workspace --all-targets)"
cargo test --workspace --all-targets

step "[5/7] doc (-D warnings, private items)"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items

step "[6/7] msrv (cargo check @ 1.95.0)"
cargo +1.95.0 check --workspace

step "[7/7] dogfood (informational — antigen scans + audits itself; never blocks)"
cargo run --bin cargo-antigen -- antigen scan || true
cargo run --bin cargo-antigen -- antigen audit || true

printf '\n=== CI-LOCAL GREEN — safe to push (matrix/beta legs still run in CI) ===\n'
