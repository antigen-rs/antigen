//! Procedural macros for the antigen crate.
//!
//! This crate provides the five attribute macros that constitute the antigen
//! API surface:
//!
//! - [`#[antigen(...)]`](macro@antigen) — declare a named failure-class with a
//!   structural fingerprint (ADR-001, ADR-010)
//! - [`#[presents(...)]`](macro@presents) — mark code as exhibiting an antigen's
//!   structural pattern (vulnerability declaration)
//! - [`#[immune(...)]`](macro@immune) — declare immunity with a witness reference
//!   (test, proptest, phantom-type proof, or external-tool delegation)
//! - [`#[descended_from(...)]`](macro@descended_from) — propagate antigen markers
//!   through an inheritance chain (ADR-013, ADR-018 §propagation)
//! - [`#[antigen_tolerance(...)]`](macro@antigen_tolerance) — document an
//!   intentional opt-out with required rationale (ADR-011)
//!
//! Users typically import these via the [`antigen`](https://docs.rs/antigen)
//! crate (`use antigen::{antigen, presents, immune, descended_from,
//! antigen_tolerance};`) rather than depending on `antigen-macros` directly.
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
//! ## Known v1 limitations
//!
//! 1. **Macros are pure pass-through**: the proc-macros parse + validate the
//!    attribute syntax and emit the original item unchanged (ADR-001 identity
//!    transform). Cross-crate antigen discovery in v0.1.0-rc.1 works via
//!    source-walking the `.cargo/registry` tree (A3 sweep). A future ADR
//!    amendment may add metadata-emitting transforms (e.g.,
//!    `<!-- antigen:metadata:v1 {...} -->` doc-comment markers, or
//!    `#[cfg(doc)] pub static __ANTIGEN_META_*`) for the no-source-access
//!    case — verified-viable but post-A5 ADR territory per the A3 scope-lock.
//!
//! Span-aware error pointing (W4) and trybuild fixtures (A2 ratification)
//! both shipped in v0.1.0-rc.1.

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
/// **v0.1 status**: `#[descended_from]` is recognized and parsed but
/// propagation is not yet implemented. In v0.1, this attribute compiles
/// cleanly and is recorded by `cargo antigen scan` for future use.
/// Chain-walking and marker propagation (`#[presents]` / `#[immune]`
/// inheritance with witness re-validation) arrive in A3.
#[proc_macro_attribute]
pub fn descended_from(args: TokenStream, input: TokenStream) -> TokenStream {
    let _args = parse_macro_input!(args as parse::DescendedFromArgs);
    let input = proc_macro2::TokenStream::from(input);

    quote! { #input }.into()
}

/// Mark a site as a deliberate, non-vulnerable match against an antigen's
/// fingerprint. Per ADR-011.
///
/// # Arguments
///
/// - The antigen type name (positional)
/// - `rationale = "..."` (required) — human-readable justification; empty
///   string is rejected
/// - `until = "..."` (optional) — expiry tag (e.g., `"v1.0"`); empty string
///   is rejected (per aristotle reciprocal Phase 1-8)
/// - `see = [...]` (optional) — open-vocabulary string array of references
///
/// # Example
///
/// ```ignore
/// use antigen::antigen_tolerance;
///
/// #[antigen_tolerance(
///     PolarityInvertedClassMeet,
///     rationale = "This test fixture deliberately constructs the failure \
///                  pattern to verify the witness catches it.",
///     until = "v1.0",
///     see = ["GAP-BIT-EXACT-1"],
/// )]
/// fn test_polarity_inversion_caught() { /* ... */ }
/// ```
///
/// `cargo antigen scan` recognizes tolerance markers as explicit
/// acknowledgments of fingerprint matches; tolerated sites are reported in
/// a separate category from unaddressed presentations.
#[proc_macro_attribute]
pub fn antigen_tolerance(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::ToleranceArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    quote! { #input }.into()
}
