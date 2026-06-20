---
name: Bug report
about: Report a bug in the antigen tooling itself (not a failure-class antigen detected by it)
title: '[bug] '
labels: bug
assignees: ''
---

> **Note**: this is for bugs in the antigen tooling itself — a crash, a wrong scan or
> audit result, a fingerprint that matches the wrong code. For a *failure-class* you'd
> like antigen to recognize, use the failure-class proposal template instead.

## Affected crate(s)

- [ ] antigen (lib)
- [ ] cargo-antigen (bin)
- [ ] antigen-fingerprint
- [ ] antigen-attestation
- [ ] antigen-macros
- [ ] Other (specify): _____________________________________________

## Version(s)

<!-- the output of `cargo antigen --version`; see crates.io for the latest -->>

## Rust version

<!-- Output of `rustc --version` -->

## Operating system

<!-- e.g., Ubuntu 24.04, macOS 14, Windows 11 -->

## Steps to reproduce

1.
2.
3.

## Expected behavior

<!-- What should have happened? -->

## Actual behavior

<!-- What actually happened? Include error output, stack traces, etc. -->

## Minimal reproduction

<!-- Ideally a Cargo project (or single file) that demonstrates the bug. -->

```rust
// minimal repro here
```

## Have you checked

- [ ] The [README](https://github.com/antigen-rs/antigen/blob/main/README.md) for
      design-phase status disclaimers
- [ ] Existing issues for similar reports
- [ ] The [CHANGELOG](https://github.com/antigen-rs/antigen/blob/main/CHANGELOG.md) for
      known unreleased fixes
