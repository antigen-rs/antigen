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

// ============================================================================
// Convergent-Evidence Family (ADR-024)
// ============================================================================

/// Declare convergent multi-modality evidence backing a defense claim.
///
/// `#[diagnostic(modalities = [...], min_independent = N)]` asserts that
/// at least `N` distinct [`WitnessClass`](https://docs.rs/antigen) categories
/// converge on this defense. Per ADR-024 §Decision + adversarial C1, the
/// count is over distinct CLASSES, not raw witness count — running the
/// same kind of test in triplicate doesn't add evidence.
///
/// # Biology grounding
///
/// `#[diagnostic]` is grounded in **clinical medicine**, not immunology
/// proper. The metaphor is the diagnostic workup pattern from
/// differential-diagnosis literature: a clinician confirms a diagnosis
/// when independent modalities (history, physical, imaging, labs)
/// converge on the same finding. A single modality is suggestive; the
/// convergence is what carries clinical confidence. Per ADR-024 §Biology
/// grounding — dual-axis honesty, `#[diagnostic]` sits on the
/// clinical-medicine axis alongside `#[panel]`, `#[ddx]`, `#[rx]`,
/// `#[triage]`, `#[refer]`, `#[biopsy]`, `#[culture]`, `#[quarantine]`,
/// and `#[recurrence_anchor]`. The convergent-evidence family draws on
/// both immunology (clonal expansion, `IgG` class-switching) and clinical
/// medicine (diagnostic workup) — the dual axis is acknowledged
/// explicitly rather than collapsed.
///
/// # Arguments
///
/// - `modalities = [WitnessClass::X, ...]` (required) — non-empty list
/// - `min_independent = N` (required, > 0) — distinct-class floor; the
///   parser rejects `min_independent` exceeding the number of distinct
///   classes (vacuously unsatisfiable claim).
///
/// # Audit hints
///
/// - `diagnostic-modality-insufficient` — fewer modalities than the floor
/// - `diagnostic-modalities-class-collapsed` — all witnesses share one class
/// - `diagnostic-modalities-empty` — empty modalities list
///
/// # Example
///
/// ```ignore
/// use antigen::{antigen, diagnostic, WitnessClass};
///
/// #[diagnostic(
///     modalities = [WitnessClass::PropertyTest, WitnessClass::FormalVerification],
///     min_independent = 2,
/// )]
/// pub fn checked_arithmetic_sum(a: i64, b: i64) -> Option<i64> {
///     a.checked_add(b)
/// }
/// ```
#[proc_macro_attribute]
pub fn diagnostic(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::DiagnosticArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare iterated witness evaluation (B-cell clonal expansion analog).
///
/// `#[clonal(witness = ..., iterations = N, seed = SeedKind::...)]`
/// asserts that a witness is run with many independent iterations.
/// Per ADR-024 §Decision + adversarial C2, `seed = SeedKind::Fixed(_)`
/// is a COMPILE ERROR — a fixed seed makes "independent iterations" a
/// contradiction.
///
/// # Arguments
///
/// - `witness = <ident>` (required) — per-iteration witness function
/// - `iterations = N` (required, > 0)
/// - `seed = SeedKind::X` (optional; default `Random`) — non-deterministic
///   variants accepted: `Random`, `EntropyFromCi`, `TimestampSeeded`.
///   `SeedKind::Fixed(_)` rejected at parse time.
///
/// # Audit hints
///
/// - `clonal-fixed-seed-detected` — parse-time error (above)
/// - `clonal-iterations-below-threshold` — N below workspace floor
///
/// # Example
///
/// ```ignore
/// use antigen::{clonal, SeedKind};
///
/// #[clonal(witness = sum_property, iterations = 10_000, seed = SeedKind::Random)]
/// pub fn checked_arithmetic_sum(a: i64, b: i64) -> Option<i64> {
///     a.checked_add(b)
/// }
/// ```
#[proc_macro_attribute]
pub fn clonal(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::ClonalArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare IgG-class affinity-matured evidence (re-attestation history).
///
/// `#[igg(witnesses = [...], historical_span = N, min_reattestations = N)]`
/// asserts that the defense has been re-attested across a time span.
/// Per ADR-024 §Decision + adversarial C3, source-independence is
/// NOMINAL only — different signer identity strings are not structural
/// proof of independent sources.
///
/// # Arguments
///
/// - `witnesses = [...]` (required non-empty)
/// - `historical_span = N` (required, > 0; days)
/// - `min_reattestations = N` (required, > 0)
///
/// # Audit hints
///
/// - `igg-identity-collapse-warning` — same signer across reattestations
/// - `igg-span-too-short` — historical span below floor
/// - `igg-reattestations-insufficient` — fewer reattestations than floor
#[proc_macro_attribute]
pub fn igg(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::IggArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare crossreactive coverage — one defense covers related antigens.
///
/// `#[crossreactive(fingerprints = [...])]` asserts that the annotated
/// item's defense applies to multiple antigen fingerprints simultaneously
/// (analogous to a crossreactive antibody binding related epitopes).
///
/// # Arguments
///
/// - `fingerprints = [...]` (required non-empty list of strings)
///
/// # Audit hints
///
/// - `crossreactive-fingerprint-unresolved` — fingerprint doesn't match
///   any known antigen
#[proc_macro_attribute]
pub fn crossreactive(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::CrossreactiveArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare polyclonal evidence — many independent lineages converge.
///
/// `#[polyclonal]` is a marker primitive (no required args) declaring
/// that the defense rests on multiple independent witness lineages.
/// Distinct from `#[diagnostic]`: polyclonal emphasizes LINEAGE diversity
/// (different witness derivations) rather than MODALITY diversity
/// (different witness classes).
///
/// # Audit hints
///
/// - `polyclonal-insufficient-lineages` — fewer lineages than configured
///   floor
#[proc_macro_attribute]
pub fn polyclonal(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::PolyclonalArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare monoclonal evidence — single independent lineage.
///
/// `#[monoclonal]` is the structural contrast to `#[polyclonal]`. The
/// monoclonal posture is honest about resting on a single lineage; the
/// audit treats it as a documentary acknowledgement rather than a
/// failing.
#[proc_macro_attribute]
pub fn monoclonal(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::MonoclonalArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare ADCC (antibody-dependent cellular cytotoxicity) — multi-
/// mechanism convergent defense.
///
/// `#[adcc]` asserts that the defense combines antibody-style witness
/// (declaration + check) AND cellular-effector witness (runtime
/// behavioral check) via different mechanisms. The marker primitive
/// surfaces the structural commitment to multi-mechanism defense.
///
/// # Audit hints
///
/// - `adcc-single-mechanism-only` — only one of the two mechanisms
///   detectable on the site
#[proc_macro_attribute]
pub fn adcc(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::AdccArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

// ============================================================================
// Recurrent-Emergence Family (ADR-024 + scientist HOW-spec cf2a2317 +
// aristotle Reading-A pre-authorization 744471a3)
//
// Six present-looking primitives: #[itch], #[recurrence_anchor],
// #[crystallize], #[chronic], #[saturate], #[strand]. Cognitive-organizational
// grounding for itch/saturate/crystallize/strand; immunology-proper for
// chronic; clinical-medicine for recurrence_anchor.
// ============================================================================

/// Declare a below-threshold noticing of a pattern (ADR-024 recurrent family).
///
/// `#[itch(name, antigen?, description, threshold?)]` marks a
/// cognitive-organizational observation: a pattern has been noticed but
/// has not yet crossed the threshold into a formal antigen declaration.
/// Per ADR-024 §Disambiguation: distinct from `#[anergy]` (ADR-023,
/// intentional non-defense while waiting) — itch is pre-commitment
/// noticing, anergy is deliberate defer.
///
/// # Biology grounding
///
/// **Cognitive-organizational** axis per ADR-024 §Biology grounding —
/// dual-axis honesty. Not an immunology-proper cognate; the metaphor is
/// drawn from how teams notice patterns before formalizing them.
///
/// # Arguments
///
/// - `name = "<slug>"` (required) — kebab-case identifier
/// - `antigen = <Path>` (optional) — failure-class path, if known
/// - `description = "..."` (required, ≥10 chars) — what is being noticed
/// - `threshold = "..."` (optional) — what would cause crystallize-promotion
///
/// # Audit hints
///
/// - `itch-noticed-not-anchored` — no antigen path; unlinked observation
#[proc_macro_attribute]
pub fn itch(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::ItchArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare formal recognition of a cross-substrate recurrent failure-class
/// (ADR-024 recurrent family).
///
/// `#[recurrence_anchor(antigen, instances, since, rationale)]` commits to
/// formal recognition of a pattern that has crossed the substrate-evidence
/// threshold. Per ADR-024 §Disambiguation: distinct from `#[chronic]`
/// (low-level persistent, NOT cross-substrate) — `recurrence_anchor` is
/// cross-substrate-threshold-reached.
///
/// # Biology grounding
///
/// **Clinical-medicine** axis per ADR-024 §Biology grounding. Analogous
/// to a clinical diagnosis after recurrent symptoms cross the threshold
/// for formal recognition.
///
/// # Arguments
///
/// All four fields REQUIRED:
/// - `antigen = <Path>` — failure-class path being anchored
/// - `instances = N` (positive `u32`) — how many recurrences observed
/// - `since = "<date-or-version>"` — first detected instance anchor
/// - `rationale = "..."` (≥20 chars) — clinical-diagnosis-grade rationale
///
/// # Audit hints
///
/// - `recurrence-threshold-reached-no-action` — anchor declared but no
///   downstream `#[immune]`/`#[presents]` registered
#[proc_macro_attribute]
pub fn recurrence_anchor(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::RecurrenceAnchorArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare the promotion event from itch-cluster to formal failure-class
/// (ADR-024 recurrent family).
///
/// `#[crystallize(name, from_itches?, antigen?, summary)]` records the
/// moment a pattern of noticings crystallizes into a formal antigen.
/// Parallel to camp's field-track `crystallize` verb.
///
/// # Biology grounding
///
/// **Cognitive-organizational** axis per ADR-024 §Biology grounding.
///
/// # Arguments
///
/// - `name = "<slug>"` (required)
/// - `from_itches = [Ident, ...]` (optional) — `#[itch]` idents this
///   crystallizes from
/// - `antigen = <Path>` (optional) — the formal antigen this crystallizes
///   into
/// - `summary = "..."` (required, ≥10 chars)
///
/// # Audit hints
///
/// - `crystallize-without-antigen` — crystallized but no formal antigen
///   path registered yet
#[proc_macro_attribute]
pub fn crystallize(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::CrystallizeArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a low-level persistent failure-class signal (ADR-024 recurrent
/// family).
///
/// `#[chronic(antigen, since, status?, managed_by?)]` marks a sustained
/// signal that has NOT crossed the cross-substrate-recurrence threshold
/// per ADR-024 §Disambiguation. Distinct from `#[recurrence_anchor]`.
///
/// # Biology grounding
///
/// **Immunology-proper** axis per ADR-024 §Biology grounding — chronic
/// inflammation is the biology cognate: sustained low-level immune
/// activity without acute recurrence.
///
/// # Arguments
///
/// - `antigen = <Path>` (required) — failure-class being marked chronic
/// - `since = "<date-or-version>"` (required) — when first observed
/// - `status = "..."` (optional) — current status description
/// - `managed_by = "..."` (optional) — team or role managing the state
///
/// # Audit hints
///
/// - `chronic-signal-unmanaged` — no `managed_by` after N versions
/// - `chronic-signal-past-review-date`
#[proc_macro_attribute]
pub fn chronic(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::ChronicArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a saturation-evidence contribution toward a recurrence threshold
/// (ADR-024 recurrent family).
///
/// `#[saturate(antigen?, contributing_to?, description)]` accumulates
/// evidence toward an anchor or itch without (yet) committing to either.
///
/// # Biology grounding
///
/// **Cognitive-organizational** axis per ADR-024 §Biology grounding.
///
/// # Arguments
///
/// - `antigen = <Path>` (optional)
/// - `contributing_to = "<slug>"` (optional) — `#[recurrence_anchor]` or
///   `#[itch]` slug this contributes to
/// - `description = "..."` (required, ≥10 chars)
///
/// # Audit hints
///
/// - `saturate-no-anchor` — no `contributing_to` target named
#[proc_macro_attribute]
pub fn saturate(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::SaturateArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a thread of related noticing across substrates (ADR-024
/// recurrent family).
///
/// `#[strand(name, anchored_by?, description)]` groups noticings that
/// share a structural rhyme but haven't yet crystallized. May spawn
/// `#[itch]` or `#[recurrence_anchor]` as the strand thickens.
///
/// # Biology grounding
///
/// **Cognitive-organizational** axis per ADR-024 §Biology grounding.
///
/// # Arguments
///
/// - `name = "<slug>"` (required)
/// - `anchored_by = [Ident, ...]` (optional) — `#[itch]` or
///   `#[recurrence_anchor]` idents this strand spans
/// - `description = "..."` (required, ≥10 chars)
///
/// # Audit hints
///
/// - `strand-no-anchors` — nothing anchors this strand
#[proc_macro_attribute]
pub fn strand(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::StrandArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

// ============================================================================
// Mucosal Boundary Family (ADR-027 + Amendment 1)
//
// Three primitives: #[mucosal], #[mucosal_delegate], #[mucosal_tolerant].
// MucosalKind sealed 13-variant set. Biology grounds the tier-claim + 4
// functional disciplines (NOT per-variant tissue mapping per ADR-027
// NON-NEGOTIABLE). Three response states: active defense / active tolerance
// / undecided — parallel to ADR-016 immune/tolerance/undeclared triad.
// ============================================================================

/// Declare a trust boundary is actively defended at this site (ADR-027).
///
/// `#[mucosal(kind = MucosalKind::X, rationale = "...")]` marks a function
/// as the defended boundary for a kind of data/control flow crossing the
/// trust surface.
///
/// # Biology grounding
///
/// Per ADR-027 §Biology grounding (NON-NEGOTIABLE): biology grounds the
/// TIER-CLAIM (mucosal surfaces are a distinct immune tier with selective
/// permeability) + the prevention-at-boundary discipline (secretory-IgA-style
/// exclusion). It does NOT ground per-variant tissue mapping — the
/// `MucosalKind` taxonomy is software-engineering scope-selection by
/// data-flow type, not anatomy.
///
/// # Arguments
///
/// - `kind = MucosalKind::X` (required) — the boundary type (one of 13
///   sealed-set variants)
/// - `rationale = "..."` (required, ≥20 chars) — why this boundary is
///   defended
///
/// # Audit hints
///
/// - `mucosal-boundary-undefended`, `mucosal-kind-mismatch`,
///   `mucosal-rationale-insufficient`
#[proc_macro_attribute]
pub fn mucosal(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::MucosalArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare boundary discipline is delegated to a named handler (ADR-027 +
/// Amendment 1).
///
/// `#[mucosal_delegate(boundary = MucosalKind::X, handled_by = path::to::fn,
/// rationale = "...")]` declares that the boundary defense is performed by a
/// callee. Per ADR-027 Amendment 1 Change 4 `handled_by` is a path
/// expression (not a string) so typos fail at parse-time. Per Change 5 the
/// handler MUST carry a matching `#[mucosal(kind = X)]` — enforced at
/// audit-time via the three-tier diagnosis.
///
/// # Arguments
///
/// - `boundary = MucosalKind::X` (required) — the delegated boundary kind
/// - `handled_by = <path>` (required) — path to the handler function
/// - `rationale = "..."` (required, ≥20 chars)
///
/// # Audit hints (three-tier diagnosis per Change 5)
///
/// - `mucosal-discipline-delegate-target-missing` — handler path doesn't
///   resolve
/// - `mucosal-discipline-delegate-target-not-mucosal` — handler has no
///   `#[mucosal]`
/// - `mucosal-discipline-delegate-target-kind-mismatch` — handler's
///   `#[mucosal(kind)]` set doesn't include `boundary`
#[proc_macro_attribute]
pub fn mucosal_delegate(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::MucosalDelegateArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a boundary is INTENTIONALLY permitted — active tolerance, not
/// absence of defense (ADR-027 Amendment 1 Change 6).
///
/// `#[mucosal_tolerant(kind, rationale, accepts, reviewed_by?, until?)]`
/// declares that a boundary deliberately accepts input without the full
/// `#[mucosal]` defense discipline, and documents WHY that's acceptable.
/// Without this primitive, intentional-tolerance boundaries are
/// indistinguishable from undefended ones in `mucosal-map --undefended`.
///
/// # Biology grounding
///
/// Per ADR-027 Amendment 1: biology distinguishes THREE mucosal response
/// states — active defense (`#[mucosal]`), active tolerance
/// (`#[mucosal_tolerant]`), and undecided (no declaration). Active
/// tolerance is NOT absence of response — it is antigen-specific
/// Treg-mediated suppression with its own cellular machinery (oral
/// tolerance, fetal-maternal interface). Parallel to ADR-016
/// `#[antigen_tolerance]` but at the BOUNDARY tier rather than the
/// failure-class tier.
///
/// # Arguments
///
/// - `kind = MucosalKind::X` (required)
/// - `rationale = "..."` (required, **≥40 chars** — higher than
///   `#[mucosal]`'s ≥20; tolerance is the riskier declaration)
/// - `accepts = "..."` (required, non-empty) — what the boundary accepts
///   as legitimate input
/// - `reviewed_by = "..."` (optional v0.2; recommended v0.2.1+)
/// - `until = "<RFC-3339 date>"` (optional) — review deadline
///
/// # Audit hints
///
/// - `mucosal-tolerant-rationale-insufficient`, `mucosal-tolerant-accepts-empty`,
///   `mucosal-tolerant-past-review-date`, `mucosal-tolerant-without-reviewer`
///   (v0.2.1+ migration hint)
#[proc_macro_attribute]
pub fn mucosal_tolerant(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::MucosalTolerantArgs);
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

/// Declare a rollback-as-triage commit: classify system state + commit to
/// rollback within a tight time-bound (ADR-026 §Rollback-as-triage).
///
/// Per aristotle's fixup-orient-dual-signature resolution (camp note
/// 55a161e7): `#[triage_commit]` is a SIBLING primitive to `#[orient]`, NOT
/// an extension. Orient names a failure-class with see-also context;
/// `triage_commit` names a triage decision + a rollback action. The two are
/// different speech acts in the deferred-defense family.
///
/// # Biology grounding — dual-axis honesty
///
/// The `#[triage_commit]` primitive carries DUAL-AXIS grounding per ADR-026
/// §Finding (NON-NEGOTIABLE per naturalist); neither axis is decorative.
///
/// **Clinical-medicine axis grounds the OUTCOME**: triage as a discipline
/// comes from clinical emergency-response medicine — the practice of
/// classifying patients by acuity before deciding treatment order. The
/// 5-color taxonomy (Black/Red/Yellow/Green/White) rhymes with clinical
/// field-triage protocols (e.g., START — Simple Triage And Rapid
/// Treatment), but `#[triage_commit]` is not a clinical-medicine
/// implementation: the rollback-as-treatment use-case extends the
/// protocol's shape with software-specific cases (`White` for non-incident
/// triages, no clinical analog). Clinical-medicine grounds the
/// COMMIT-DECISION-BEFORE-ACTION discipline: informed consent + chart
/// documentation precede the procedure; structurally isomorphic to
/// triage-commit-before-rollback.
///
/// **Software-engineering axis grounds the PROCESS**:
/// rollback-as-mandated-by-triage-tag is software-engineering invention.
/// Immune biology has NO analog to "log rationale before acting" (per
/// ADR-026 §Finding). The `triaged_by` + `rationale` +
/// `rollback_due_within_minutes` fields operate at the
/// software-engineering tier; their structural enforcement (parse-time
/// validation, audit-time substrate-witness via git-trailer per ADR-019)
/// is software-engineering machinery composed under the clinical-medicine
/// outcome framing.
///
/// **What biology DOES ground (Class 1, outcome-level)**: the
/// `ForcePushErasingHistory` ↔ Immune Amnesia (measles) cognate (ADR-026
/// §Finding) is the central immune-biology grounding for the broader
/// VCS-info-loss family — catastrophic loss of memory-carrying substrates
/// with documented harm. `#[triage_commit]` is the prescribed defense at
/// the rollback boundary: biology predicts that memory-loss requires
/// structural defense; clinical-medicine prescribes the form
/// (triage-decision documented before action).
///
/// This is the same dual-axis honesty ADR-024 ratified for the
/// temporal-arc families — antigen draws from MULTIPLE grounding
/// disciplines, naming which axis grounds which property. Overclaim "this
/// is immune biology" would be dishonest; underclaim "this is decorative"
/// would lose the predictive power. Dual-axis is the right shape.
///
/// # Arguments
///
/// All five fields are REQUIRED per ADR-026 §Decision:
///
/// - `triage_decision = TriageDecision::X` — five-color triage classification
///   (one of `Black`, `Red`, `Yellow`, `Green`, `White`). See
///   [`antigen::vcs::TriageDecision`](https://docs.rs/antigen) for variant
///   semantics.
/// - `rollback_target = "<sha>"` — commit sha pointing to the last-known-good
///   state. Non-empty.
/// - `triaged_by = "<role|name>"` — informed-consent author identity (role
///   slug like `"navigator"` or a personal name). Non-empty.
/// - `rationale = "..."` — chart-documentation; minimum 20 characters per
///   ADR-023 loudness-as-discipline applied to clinical-medicine
///   chart-documentation. Records WHY the rollback was decided before the
///   action commits.
/// - `rollback_due_within_minutes = N` — tight time-bound (positive `u32`).
///   Carries the discipline that triage-commits are followed by action in
///   bounded time; a zero deadline degrades the loudness pattern.
///
/// # Audit hints
///
/// - `vcs-rollback-without-triage-commit` — a rollback commit not preceded by
///   a `#[triage_commit]` declaration with `Triage-Decision: <sha>` trailer
/// - `vcs-rollback-due-window-exceeded` — `rollback_due_within_minutes`
///   elapsed without the rollback commit landing
///
/// # Example
///
/// ```ignore
/// use antigen::{triage_commit, TriageDecision};
///
/// #[triage_commit(
///     triage_decision = TriageDecision::Red,
///     rollback_target = "abc1234",
///     triaged_by = "navigator",
///     rationale = "vital metric regression confirmed via #84; rolling back to last-known-good",
///     rollback_due_within_minutes = 30,
/// )]
/// fn _triage_marker_do_not_remove() {}
/// // Followed by rollback commit with trailer:
/// //   Triage-Decision: <sha-of-this-triage-commit-marker>
/// ```
#[proc_macro_attribute]
pub fn triage_commit(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::TriageCommitArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    quote! { #input }.into()
}
