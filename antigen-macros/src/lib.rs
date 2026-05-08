//! Procedural macros for the antigen crate.
//!
//! This crate provides the four core attribute macros that constitute the antigen
//! API surface:
//!
//! - [`#[antigen(...)]`](macro@antigen) — declare a named failure-class with a
//!   structural fingerprint
//! - [`#[presents(...)]`](macro@presents) — mark code as exhibiting an antigen's
//!   structural pattern (vulnerability declaration)
//! - [`#[immune(...)]`](macro@immune) — declare immunity with a witness reference
//! - [`#[descended_from(...)]`](macro@descended_from) — propagate antigen markers
//!   through structural derivation
//!
//! ## Design philosophy (v1)
//!
//! The macros are **mostly identity transformations**. Their job is to validate the
//! attribute syntax at compile time and pass the input through unchanged. The
//! semantic work — scanning the codebase, matching presentations against
//! immunities, validating witnesses — lives in the `cargo-antigen` tooling, which
//! parses source AST independently via `syn`.
//!
//! This keeps runtime overhead at zero (the macros generate no code beyond the
//! original input) and means antigen declarations don't affect compilation speed,
//! linker behavior, or binary size. The cost lives entirely at `cargo antigen scan`
//! time, which runs out-of-band.
//!
//! See ADR-010 (fingerprint grammar v1) and the project's `docs/expedition/`
//! directory for the design rationale.
//!
//! ## Status
//!
//! `0.0.1` is a placeholder reserving the crate name. The macros below validate
//! syntax and pass through; the full validation/scanning loop ships with the
//! companion `cargo-antigen` crate.
//!
//! ## Known v1 limitations (easy wins for the JBD team)
//!
//! Search this crate for `TODO(team)` to find specific spots that the antigen
//! JBD team can sharpen quickly. Top items:
//!
//! 1. **No span-aware error pointing**: parse errors point to `Span::call_site()`
//!    rather than the offending token. The team should thread spans through the
//!    parser so error messages point to the EXACT bad token (per
//!    rust-analyzer's diagnostic conventions).
//! 2. **No trybuild test fixtures**: testing-patterns.md describes the trybuild
//!    pattern for proc-macro errors, but no fixtures ship in v0.0.1. Adding
//!    them is straightforward and gives the team confidence that error
//!    messages stay helpful as the parser evolves.
//! 3. **Macros are pure pass-through**: future versions may emit `#[doc(hidden)]`
//!    inventory items so the cargo-antigen tooling can discover declarations
//!    without re-parsing source. The team can decide whether the inventory
//!    pattern is worth the complexity vs the current source-parse approach.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod parse;

/// Declare a named failure-class with a structural fingerprint.
///
/// # Arguments
///
/// - `name = "..."` (required) — kebab-case identifier for the failure-class
/// - `fingerprint = "..."` (required) — structural pattern (see ADR-010)
/// - `family = "..."` (optional) — parent class, typically one of the 8
///   first-principles failure classes
/// - `summary = "..."` (optional) — human-readable description
/// - `references = [...]` (optional) — open-vocabulary list of references
///   (URLs, ADR/DEC IDs, CVE numbers, RFC numbers, etc.)
///
/// # Examples
///
/// Layer 1 (minimum viable — just `name` and `fingerprint`):
///
/// ```ignore
/// use antigen::antigen;
///
/// #[antigen(
///     name = "panicking-in-drop",
///     fingerprint = "impl Drop with unwrap/expect/panic in body",
/// )]
/// pub struct PanickingInDrop;
/// ```
///
/// Layer 2 (enriched — adds `family`, `summary`, `references`):
///
/// ```ignore
/// #[antigen(
///     name = "panicking-in-drop",
///     family = "boundary-violation",
///     fingerprint = "impl Drop with unwrap/expect/panic in body",
///     summary = "Drop impls must not panic; double-panic causes process abort.",
///     references = ["https://doc.rust-lang.org/std/ops/trait.Drop.html#panics"],
/// )]
/// pub struct PanickingInDrop;
/// ```
///
/// The struct must be declared as a unit struct (no fields). The macro validates
/// the attribute arguments and passes the struct through unchanged.
#[proc_macro_attribute]
pub fn antigen(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::AntigenArgs);
    let input = parse_macro_input!(input as syn::ItemStruct);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    if !matches!(input.fields, syn::Fields::Unit) {
        return syn::Error::new_spanned(
            &input,
            "#[antigen] must be applied to a unit struct (e.g., `pub struct Name;`)",
        )
        .to_compile_error()
        .into();
    }

    let name_string = &args.name;
    let attr_doc = format!(
        " antigen `{name_string}` — declares a named failure-class.\n\n Use \
         `cargo antigen scan` to find sites presenting this antigen; \
         `cargo antigen audit` to validate witness coverage."
    );

    let expanded = quote! {
        #[doc = #attr_doc]
        #input
    };

    expanded.into()
}

/// Mark code as exhibiting a known antigen's structural pattern (vulnerability
/// declaration).
///
/// # Arguments
///
/// Single positional argument: the antigen type name.
///
/// # Example
///
/// ```ignore
/// use antigen::presents;
///
/// #[presents(PanickingInDrop)]
/// impl Drop for MyType {
///     fn drop(&mut self) { /* might panic */ }
/// }
/// ```
///
/// `cargo antigen scan` flags every `#[presents]` site that lacks a corresponding
/// `#[immune]` declaration. This declaration is the *vulnerability surface* — it
/// says "this code exhibits the structural pattern."
///
/// To express both vulnerability AND verified immunity, apply both attributes to
/// the same item:
///
/// ```ignore
/// #[presents(PanickingInDrop)]
/// #[immune(PanickingInDrop, witness = no_panic_test)]
/// impl Drop for SafeType { ... }
/// ```
#[proc_macro_attribute]
pub fn presents(args: TokenStream, input: TokenStream) -> TokenStream {
    let _args = parse_macro_input!(args as parse::PresentsArgs);
    let input = proc_macro2::TokenStream::from(input);

    quote! { #input }.into()
}

/// Declare immunity to a known antigen, with a witness that proves the immunity.
///
/// # Arguments
///
/// - The antigen type name (positional)
/// - `witness = ...` (required) — reference to a test, proptest, lint, formal-
///   verification proof, or phantom-type construction that proves immunity
/// - `rationale = "..."` (optional) — human-readable description of why the
///   witness applies
///
/// # Example
///
/// ```ignore
/// use antigen::immune;
///
/// #[immune(
///     PanickingInDrop,
///     witness = no_panic_in_drop_test,
///     rationale = "SafeType::drop uses Result-returning paths only.",
/// )]
/// impl Drop for SafeType { ... }
/// ```
///
/// `cargo antigen scan` validates that the witness identifier resolves to a real
/// test/proptest/lint/proof. Witnesses that don't exist or don't run successfully
/// invalidate the immunity claim.
///
/// `#[immune]` does not require `#[presents]` on the same item. Pre-emptive
/// immunity is acceptable: declaring immunity to ensure future modifications stay
/// covered, even when the current code doesn't structurally match the antigen.
#[proc_macro_attribute]
pub fn immune(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::ImmuneArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    quote! { #input }.into()
}

/// Propagate antigen markers from a parent function/type/method to a derived one.
///
/// # Arguments
///
/// Single positional argument: a path to the parent item.
///
/// # Example
///
/// ```ignore
/// use antigen::descended_from;
///
/// #[descended_from(crate::other_module::parent_function)]
/// fn refined_function(...) { ... }
/// ```
///
/// `cargo antigen scan` walks `#[descended_from]` chains. Markers (`#[presents]`,
/// `#[immune]`) on the parent propagate to the descendant — with re-validation
/// of witnesses against the descendant's actual behavior. Witness divergence
/// (signature change, new edge case) invalidates inherited immunity and prompts
/// re-justification.
#[proc_macro_attribute]
pub fn descended_from(args: TokenStream, input: TokenStream) -> TokenStream {
    let _args = parse_macro_input!(args as parse::DescendedFromArgs);
    let input = proc_macro2::TokenStream::from(input);

    quote! { #input }.into()
}
