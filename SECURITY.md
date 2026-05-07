# Security Policy

## Reporting a vulnerability

The antigen project is in design phase. The current `0.0.1` placeholder crates contain
no functional code that processes input or performs sensitive operations.

If you discover a security vulnerability in any antigen crate (current or future),
please **do not open a public issue**. Instead:

1. Email the project maintainers (security disclosure path will be added before v0.1)
2. Use GitHub's [security advisory](https://github.com/antigen-rs/antigen/security/advisories/new)
   feature to report privately
3. Provide as much detail as you can, including:
   - The affected crate(s) and version(s)
   - Steps to reproduce
   - Potential impact
   - Suggested mitigation if available

We will acknowledge receipt within 7 days and provide an estimated timeline for a fix.

## Supported versions

| Version | Supported          |
| ------- | ------------------ |
| 0.0.1   | Placeholder; no security surface |
| 0.1+    | TBD when v0.1 ships |

## Disclosure policy

Once a fix is available:
- A coordinated disclosure timeline is agreed with the reporter
- A security advisory is published via GitHub's advisory system
- A new patch version is released with the fix
- Credits are given to the reporter unless they prefer to remain anonymous

## Scope

This policy applies to all crates in the antigen workspace:
- `antigen` (core library)
- `cargo-antigen` (cargo subcommand)
- Any future workspace member crates (`antigen-stdlib`, etc.)

It does NOT apply to:
- Issues with witness mechanisms (those are clippy/kani/prusti/proptest issues; report
  to those projects)
- Bugs in failure-class detection (these are correctness issues, not security; use
  normal issue reporting)
- Vulnerabilities in projects that depend on antigen (report to those projects)

## Reservation note

The `0.0.1` crates are namespace reservations and do not currently process untrusted
input or perform privileged operations. The security surface effectively does not exist
yet. This document serves as the policy framework for when antigen ships functional
code.
