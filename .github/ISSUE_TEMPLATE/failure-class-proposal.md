---
name: Failure-class proposal
about: Propose a refinement to an existing failure class, or a new failure class
title: '[failure-class] '
labels: failure-class-proposal
assignees: ''
---

## What failure class are you proposing or refining?

<!-- Pick one: -->
- [ ] Refinement to existing class (which one?): _________________________________
- [ ] New class outside the existing 8

## Real-world instances

<!-- We need at least 3 real-world Rust ecosystem instances that exhibit this failure class.
     Prefer: actual bugs found in published crates, RustSec advisories, RFC discussions,
     post-mortem blog posts, GitHub issues. -->

1. _Instance description + URL/source_
2. _Instance description + URL/source_
3. _Instance description + URL/source_

## Structural fingerprint

<!-- What structural pattern do these instances share? Be specific.
     e.g., "class enum + reverse-discriminant-ordering + meet operation" -->

## Distinguishing features

<!-- How is this class distinct from the existing 8?
     If it's a refinement, how does it relate to the parent class? -->

## Relationship to existing tools

<!-- Do clippy / kani / prusti / proptest / cargo-mutants address this? Partially? Not at all?
     If existing tools cover it, antigen's role is to NAME the class and compose the existing
     coverage rather than reinventing detection. -->

## Suggested antigen name

<!-- e.g., `polarity-inverted-class-meet`, `lock-order-inversion`, `nan-comparison-trap` -->

## Witness candidates

<!-- What kind of witness would prove immunity?
     - Test? Proptest? Phantom-type? Formal-verification proof? Clippy lint? -->
