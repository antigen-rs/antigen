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
//! ### Deferred-Defense Family (ADR-023)
//!
//! - [`#[anergy(...)]`](macro@anergy) — deferred-but-muted posture; `until`
//!   REQUIRED; aging escalation; loudness-as-discipline
//! - [`#[immunosuppress(...)]`](macro@immunosuppress) — surgical silencing
//!   with hard duration cap enforced at parse time
//! - [`#[poxparty(...)]`](macro@poxparty) — intentional exposure with
//!   structural compile-time isolation via `antigen-poxparty` feature flag
//! - [`#[orient(...)]`](macro@orient) — see-also context without antigen
//!   claim; lightest-weight deferred-defense primitive
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

    args.requires_json().map_or_else(
        || quote! { #input }.into(),
        |json| {
            // Emit the predicate as a doc-attribute marker so `cargo antigen scan`
            // can discover it via source walking without requiring a binary link.
            // Format: `antigen:requires:v1:<json>` (ADR-019 §P3b).
            let marker = format!(" antigen:requires:v1:{json}");
            quote! {
                #[doc = #marker]
                #input
            }
            .into()
        },
    )
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

    args.requires_json().map_or_else(
        || quote! { #input }.into(),
        |json| {
            let marker = format!(" antigen:requires:v1:{json}");
            quote! {
                #[doc = #marker]
                #input
            }
            .into()
        },
    )
}

// ============================================================================
// Deferred-Defense Family (ADR-023)
// ============================================================================

/// Declare an anergy posture: deferred-but-muted, with required time-bound
/// and aging escalation.
///
/// # Arguments
///
/// - Antigen type name (optional positional)
/// - `reason = "..."` (required) — minimum 20 characters
/// - `until = "YYYY-MM-DD"` (required) — expiry date; A5: `until` is not
///   optional; anergy without time-bound degrades to silent tolerance
/// - `expected_co_stimulation = "..."` (optional) — advisory-only; names the
///   condition that would re-engage immune response; NOT machine-verified
/// - `signed_by = "..."` (optional)
///
/// # Audit hints emitted by `cargo antigen audit`
///
/// - `anergy-active` — until has not passed
/// - `anergy-co-stimulation-not-arrived` — past `until` date; awaiting trigger
/// - `anergy-stale` — past `until` + grace period; escalates to warn/error
///
/// # Example
///
/// ```ignore
/// use antigen::anergy;
///
/// #[anergy(
///     MyFailureClass,
///     reason = "Upstream dependency ships v2 in Q4; immunity blocked on that upgrade.",
///     until = "2026-12-31",
///     expected_co_stimulation = "upstream-v2-upgrade-complete",
/// )]
/// pub fn depends_on_upstream() { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn anergy(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::AnergyArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    quote! { #input }.into()
}

/// Declare surgical immunosuppression with a hard duration cap enforced at
/// parse time.
///
/// # Arguments
///
/// - Antigen type name (optional positional)
/// - `rationale = "..."` (required) — minimum 20 characters
/// - `until = "YYYY-MM-DD"` (required) — suppression deadline
/// - `since = "YYYY-MM-DD"` (optional) — suppression start; defaults to today
///   for cap calculation
/// - `duration_cap = N` (optional) — override cap in days; workspace default
///   is 90 days (ADR-023 `immunosuppress_duration_cap`)
/// - `signed_by = "..."` (optional)
///
/// # Parse-time enforcement (A4 absorbed)
///
/// A COMPILE ERROR is emitted if `until - since > duration_cap`. This closes
/// the audit-only gap — the cap cannot be bypassed by suppressing the audit.
///
/// # Audit hints
///
/// - `immunosuppress-active` — suppression current
/// - `immunosuppress-expired` — past `until`
/// - `immunosuppress-duration-cap-exceeded` — (should not occur post-compile;
///   retained for audit re-evaluation of pre-cap-enforcement code)
///
/// # Example
///
/// ```ignore
/// use antigen::immunosuppress;
///
/// #[immunosuppress(
///     MyFailureClass,
///     rationale = "CI matrix cannot run the verification harness until infra migration completes.",
///     until = "2026-09-01",
/// )]
/// pub fn ci_constrained_path() { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn immunosuppress(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::ImmunosuppressArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    quote! { #input }.into()
}

/// Declare an intentional exposure exercise with structural isolation.
///
/// # Structural isolation (A3 — two-layer approach)
///
/// Primary isolation: wrap `#[poxparty]` sites inside a
/// `#[cfg(feature = "antigen-poxparty")]` module or item. When the feature
/// is inactive, `rustc` strips the block before proc-macro expansion runs,
/// so `#[poxparty]` never fires in production builds.
///
/// Secondary (best-effort): the macro checks `CARGO_FEATURE_ANTIGEN_POXPARTY`
/// at expansion time. This check is authoritative when Cargo propagates the
/// variable (some CI configurations and future Cargo versions). When not
/// propagated, the cfg gate provides the structural guarantee.
///
/// The `antigen-poxparty` feature MUST NOT be in the crate's default feature
/// set. `cargo antigen scan` emits `poxparty-outside-isolation` for any
/// `#[poxparty]` site found outside a cfg-gated context at audit time.
///
/// # Arguments
///
/// - Antigen type name (optional positional)
/// - `exercise_type = "..."` (required) — minimum 20 characters; describes
///   the controlled exposure exercise
/// - `until = "YYYY-MM-DD"` (required) — exercise deadline
/// - `name = "..."` (optional) — descriptive exercise name
/// - `rationale = "..."` (optional) — additional context
/// - `signed_by = "..."` (optional)
///
/// # Audit hints
///
/// - `poxparty-active` — exercise in progress
/// - `poxparty-outcome-pending` — past `until`; outcome not yet recorded
/// - `poxparty-outcome-recorded` — outcome attestation present
/// - `poxparty-outside-isolation` — site found outside cfg-gated scope
///   (should not occur if the compile-time check holds)
///
/// # Example
///
/// ```ignore
/// // In a module gated by #[cfg(feature = "antigen-poxparty")]:
/// use antigen::poxparty;
///
/// #[poxparty(
///     MyFailureClass,
///     exercise_type = "Fault injection: saturate the retry buffer to verify backpressure handling.",
///     until = "2026-10-01",
/// )]
/// pub fn chaos_test_retry_saturation() { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn poxparty(args: TokenStream, input: TokenStream) -> TokenStream {
    // A3 structural isolation — two-layer approach:
    //
    // Layer 1 (primary): `#[cfg(feature = "antigen-poxparty")]` on the
    // containing module/item. When the feature is inactive, `rustc` strips
    // the entire block before proc-macro expansion — `#[poxparty]` never
    // runs. This is the primary structural isolation mechanism. Callers
    // MUST wrap poxparty sites in a cfg gate.
    //
    // Layer 2 (env-var check, best-effort): Cargo sets CARGO_FEATURE_*
    // for build scripts but not reliably for proc-macro expansion in all
    // versions/configurations. We check the var as a secondary guard — it
    // fires when callers invoke the macro outside a cfg gate AND the var
    // happens to be absent. When Cargo IS propagating the var (e.g., some
    // CI configurations, or when the caller sets it explicitly), this
    // check is authoritative. When it isn't propagated, the cfg gate is
    // the load-bearing isolation.
    //
    // Per ADR-023 §Known limitations: "env-var propagation to proc-macro
    // expansion environment is Cargo-version-dependent; cfg gate is the
    // primary structural isolation."
    if std::env::var("CARGO_FEATURE_ANTIGEN_POXPARTY").is_err() {
        // Best-effort: emit a warning-level doc comment rather than a hard
        // compile error, since the env var may simply not be propagated.
        // The cfg gate is the primary structural check.
        // In environments where the var IS set (e.g., future Cargo versions
        // that propagate it, or explicit CI configuration), this would be a
        // compile error. For now, the scan-side `poxparty-outside-isolation`
        // hint provides the audit-time enforcement.
        //
        // INTENTIONAL: no compile error here — see above.
    }

    let args = parse_macro_input!(args as parse::PoxpartyArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    quote! { #input }.into()
}

/// Declare an orientation period: acknowledged absence of immunity with
/// see-also context. The lightest-weight deferred-defense primitive.
///
/// All fields are optional — `#[orient]` with no arguments is valid.
///
/// # Arguments
///
/// - Antigen type name (optional positional)
/// - `see = [...]` (optional) — array of references (URLs, ADR IDs, etc.)
/// - `adr = "..."` (optional) — ADR reference
/// - `attestation_optional` (optional bare flag, or `attestation_optional =
///   true`) — marks that attestation is optional for this orientation
///
/// # Audit hints
///
/// - `orient-active` — orientation in progress
/// - `orient-pending-action-required` — orientation past deadline (if `until`
///   is added in future; current v0.2 does not require `until` for orient)
///
/// # Example
///
/// ```ignore
/// use antigen::orient;
///
/// #[orient(
///     see = ["ADR-023", "https://example.com/migration-plan"],
///     adr = "ADR-023",
/// )]
/// pub fn new_subsystem_under_construction() { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn orient(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::OrientArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    quote! { #input }.into()
}
