---
name: Antigen-stdlib candidate
about: Propose a specific failure pattern to include in the eventual antigen-stdlib library
title: '[stdlib] '
labels: antigen-stdlib-candidate
assignees: ''
---

## Antigen name

<!-- snake_case or kebab-case identifier -->

## Family

<!-- Which of the 8 first-principles failure classes does this fit?
     If multiple, which is primary?
     - frame-translation
     - forgotten-lesson
     - implicit-coupling
     - stale-context
     - premature-abstraction
     - incompatible-merger
     - boundary-violation
     - optionality-collapse -->

## Brief summary

<!-- One paragraph: what failure does this antigen recognize? -->

## Real-world instances

<!-- 2-5 specific Rust ecosystem failures that fit this antigen.
     The more, the better — antigen-stdlib is curated based on evidence. -->

## Proposed structural fingerprint

<!-- What structural pattern would `cargo antigen scan` match against to flag
     vulnerable code? Initial draft is fine; the team will refine. -->

## Suggested witness type

<!-- What kind of witness would prove immunity?
     - `#[test]` function (cargo test enforces)
     - `proptest!` block (property test)
     - clippy lint (delegate to `clippy::lint_name`)
     - kani::proof (delegate to formal verification)
     - Phantom-type construction (compile-time proof)
     - Multiple options? -->

## Adoption considerations

<!-- - How common is this failure in the Rust ecosystem?
     - Would a new project likely hit this without immunity?
     - Are there existing tools that already detect it (and which witness type would delegate)? -->

## Reference material

<!-- Links to:
     - Bug reports / CVEs / RustSec advisories
     - Blog posts about the failure class
     - Existing tools that detect adjacent patterns
     - Academic papers (if any) -->
