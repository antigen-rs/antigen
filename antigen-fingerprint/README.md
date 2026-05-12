# antigen-fingerprint

Fingerprint grammar parser + matcher for antigen failure-class declarations.

This crate is the workspace-internal substrate for the `#[antigen(fingerprint
= "...")]` value. It is shared between:

- `antigen-macros` (compile-time validation that fingerprints parse cleanly)
- `antigen` (scan-time matching against target-code ASTs)

Per ADR-010 Amendment 3 Clause E, both parsing and matching live here so the
macro and scan paths cannot drift (the bug ATK-001-2 documented).

The grammar is a custom DSL parsed via `syn::parse::ParseBuffer` (Path C per
ADR-010 Amendment 1) — NOT raw `syn::parse2::<syn::Expr>`, which cannot accept
the DSL syntax.

See [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen) for the
full project substrate and `docs/fingerprint-grammar.md` for the human-readable
grammar reference.
