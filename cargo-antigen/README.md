# cargo-antigen

**Cargo subcommand for the [antigen](https://crates.io/crates/antigen) project —
scan, audit, and learn structural failure-class antibodies in Rust codebases.**

> **Status: working beta, published on crates.io.** The
> subcommands below are real and runnable today. The CLI surface is stabilizing
> toward `1.0` and may still change between beta releases.

## What this crate is

The cargo subcommand companion to [`antigen`](https://crates.io/crates/antigen).
Where the `antigen` crate gives you the macros to *declare* failure-classes,
`cargo antigen` is what *reads* those declarations across a workspace — scanning
for unaddressed sites, auditing whether each defense has a witness, and (the v0.5
keystone) *proposing* new fingerprints by learning from clusters you mark.

```text
$ cargo antigen --help
Commands:
  scan         Scan the workspace for antigen presentations and report unaddressed ones
  propose      Propose a candidate failure-class fingerprint from a cluster of marked sites
  audit        Comprehensive immunity coverage report — witness resolution and tier validation
  attest       Manage `.attest/<Antigen>.json` substrate-witness sidecars (ADR-019)
  tolerate     Manage tolerance-ratification sidecars (ADR-019 §tolerance tier)
  oracle       Manage Oracle artifact-class records (ADR-021 §D3)
  verify       Drive Supply-Chain Defense Family verifications (ADR-025)
  vcs          Drive VCS-Information-Loss Family observations (ADR-026)
  mucosal-map  Map mucosal trust boundaries across the workspace (ADR-027 + Amd 1)
  fingerprint  Print the structural fingerprint of a scanned item
  mine         Mine a repository's `.git` for the SZZ `(defect, fix)` corpus
```

## Installation

```sh
cargo install cargo-antigen
```

Then `cargo antigen <subcommand>` is available in any cargo workspace.

## The three you reach for first

**`scan`** walks the workspace and reports the marked sites. Run against a tree
with no antigen declarations of its own, the bundled stdlib catalog still gives
real findings:

```text
$ cargo antigen scan
Scanned 345 files, found 28862 antigen-related declarations:
  - 167 antigen declarations
  - 157 explicit #[presents] markers
  - 28504 fingerprint matches (candidate sites — see below)
  - 10 tolerated sites (#[antigen_tolerance])
  - 7 #[defended_by] declarations
```

A fingerprint match is a **candidate to inspect, not an audited verdict** — the
scan says "this structure resembles a known class," never "this is a bug."

**`audit`** resolves each declared defense to its witness and reports the tier:

```text
$ cargo antigen audit
Audited 24 defense(s):
  - 22 declared (witness identifier found in workspace — not yet semantically verified)
  - 0 external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)
  - 2 ambiguous (witness name resolves to multiple workspace functions)
  - 0 broken (witness identifier not found)
  - 0 missing (no witness identifier)
```

**`propose`** is the learning core. You mark a cluster of sites with `#[dread]`
(a *felt* unease), point it at a clean corpus, and antigen anti-unifies a
candidate fingerprint — then checks whether it generalizes against your clean
corpus before suggesting it:

```text
$ cargo antigen propose --cluster-root ./cluster --clean-root ./clean
== drafted a candidate — routed to a human ratifier ==

Antigen anti-unified a draft from your `dread` marks, but cannot
certify it GENERALIZES against your clean corpus (no near-miss: no clean
sibling is one discriminating constraint from binding the draft). So it
routes the candidate to a HUMAN ratifier rather than promote it.
```

That route-to-human outcome is the gate being honest — the machine supplies the
syntactic half of a fingerprint; a human ratifies the semantic half. A runnable
fixture lives in
[`examples/propose-demo`](https://github.com/antigen-rs/antigen/tree/main/examples/propose-demo).

## Why a cargo extension

Failure-class memory only works if it's structurally checked by tooling —
otherwise it drifts like documentation. The cargo extension makes scanning,
auditing, and proposing first-class development actions: runnable in CI
(`scan --strict`, `audit --strict`, `mine --min-pairs`) and integrated with the
normal workflow.

## License

Dual-licensed under [MIT](https://github.com/antigen-rs/antigen/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/antigen-rs/antigen/blob/main/LICENSE-APACHE).
