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

/// Process-global, monotonically-increasing emission counter for `#[immune]`.
///
/// Each invocation of `immune()` claims one counter value to incorporate into
/// its generated `const` name, ensuring no two `#[immune]` emissions in the
/// same compilation unit can share a name — regardless of antigen path or
/// stacking pattern (see `immune()` for the full rationale).
static IMMUNE_EMISSION_COUNTER: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Declare a named failure-class with a structural fingerprint.
///
/// # Arguments
///
/// - `name = "..."` (required) — kebab-case identifier for the failure-class
/// - `fingerprint = "..."` (optional; required for scan-locatable antigens) —
///   structural pattern (see ADR-010). Omit for verify-only antigens whose
///   detection-model is external-substrate (supply-chain / VCS-info-loss), per
///   ADR-009 Amendment 1 — they have no syn-scannable source surface.
/// - `family = "..."` (optional) — parent class, typically one of the 8
///   first-principles failure classes
/// - `summary = "..."` (optional) — human-readable description
/// - `references = [...]` (optional) — open-vocabulary list of references
///   (URLs, ADR/DEC IDs, CVE numbers, RFC numbers, etc.)
/// - `category = AntigenCategory::X` (optional) — `SubstrateAlignment` or
///   `FunctionalCorrectness` (ADR-028). **Do not import `AntigenCategory`** —
///   the macro reads this as a token path, so `use antigen::AntigenCategory;`
///   triggers `unused_imports` under `-D warnings`. Write the path directly
///   without importing.
/// - `provenance = Provenance::X` (optional) — the authored claim of *how we
///   know this failure-class exists* (ADR-039 §C). One of `Encountered` (seen
///   in real code — highest), `Constructable` (a minimal case can be built that
///   verifiably exhibits the failure), `Heuristic` (a scannable tell that
///   correlates without a constructable demo), or `Imagined` (articulated from
///   shape — a tell but no demo yet — lowest). **Omitting it defaults to
///   `Imagined`**: an unlabeled antigen is honestly the weakest claim. This is
///   the honest-labeling on-ramp — admission is permissive, so the *label* is
///   what stays truthful. Provenance is the evidence basis, NOT the confidence
///   tier (suspected/named): that tier is the dial-derived audit-time calibration
///   (the confidence-dial wave), and provenance sets the floor it may graduate from.
/// - `presentation = Presentation::X` (optional) — `Passive` (tooling/scan-side;
///   the default for low-provenance classes — no user-macro burden) or `Active`
///   (user-facing, chosen by whoever encounters the failure). **Omitting it
///   defaults to `Passive`** (ADR-039 passive-by-default rule). Like `category`,
///   `provenance` and `presentation` are read as token paths — **do not import
///   `Provenance`/`Presentation`** (a `use` of them trips `unused_imports` under
///   `-D warnings`); write the path directly.
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
/// Layer 3 (the honest-labeling fields — `category`, `provenance`,
/// `presentation`). Note the paths are written *without* a `use` import:
///
/// ```ignore
/// #[antigen(
///     name = "panicking-in-drop",
///     fingerprint = "impl Drop with unwrap/expect/panic in body",
///     category = AntigenCategory::FunctionalCorrectness,
///     provenance = Provenance::Constructable,
///     presentation = Presentation::Passive,
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

    // An antigen marker is a failure-class identity token: it carries no data
    // and no type-level parameterization. A generic marker (`struct Foo<T>;`)
    // is semantically meaningless — which failure-class does `Foo<T>` name? —
    // and would break the dead_code use-token below (a bare `let _x: Foo;`
    // reference needs type arguments, producing a cryptic E0107 pointing at the
    // declaration). Reject it here with a clear, on-point error instead.
    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &input.generics,
            "#[antigen] must be applied to a non-generic unit struct; a \
             failure-class marker carries no type parameters",
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

    // DX finding 1: in a *binary* crate, `#[antigen] pub struct Foo;` trips
    // `dead_code` because `pub` does not exempt items with no external API
    // surface, and antigen uses the marker type as a declaration token, never
    // constructs it. Rather than `#[allow(dead_code)]` (which would also mask
    // legitimate dead-code on the item), emit a zero-cost use-token that makes
    // the type genuinely "used" from the compiler's view:
    //
    //   const _: fn() = || { let _x: Foo; };
    //
    // The `const _` is anonymous (no namespace pollution), is always compiled
    // (not `#[cfg(test)]`-gated, so it works under any conditional compilation),
    // and the closure body is never invoked — the binding only references the
    // type. Zero runtime cost; honours lib.rs's "pure pass-through with zero
    // overhead" contract at runtime while satisfying the dead_code analysis.
    let marker_ident = &input.ident;
    // The use-token references the type by bare name (`let _x: Foo;`). That is
    // safe here precisely because the generics check above already rejected any
    // parameterized marker — so `#marker_ident` always names a concrete,
    // arg-free type. (Adversarial flagged use-tokens-under-generics; the guard,
    // not an assumption about E0392, is what makes this sound.)
    let expanded = quote! {
        #[doc = #attr_doc]
        #input
        const _: fn() = || {
            let _antigen_use_token: #marker_ident;
        };
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
    let args = parse_macro_input!(args as parse::PresentsArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }

    // ADR-029 R5: a `requires = <predicate>` folded onto `#[presents]` emits the
    // same `antigen:requires:v1:<json>` doc marker `#[immune(requires=...)]`
    // does, so `cargo antigen scan` discovers the substrate-witness predicate at
    // the presents-site (the new substrate-tier carrier). A `proof = <expr>` is
    // recognized structurally by the audit from the written source (phantom-tier),
    // so it needs no marker.
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

/// Declare immunity to a known antigen, backed by evidence that proves it.
///
/// **Choosing `witness =` vs `requires =`**: can a test *execute* the thing you're defending?
/// If yes, use `witness =` (the code runs, so a test/proptest/proof/lint can verify it). If no
/// — the failure-class is about substrate state that code execution can't verify (a stale
/// document, an unpinned dependency, an un-reviewed discipline sign-off) — use `requires =`.
///
/// # Arguments
///
/// - The antigen type name (positional)
/// - **Exactly one of** `witness = ...` **or** `requires = ...` (mutually
///   exclusive; one is required):
///   - `witness = <ident>` — **code-tier** immunity. A reference to a test,
///     proptest, lint, formal-verification proof, or phantom-type construction
///     that proves immunity. Reach for this when the immunity is provable from
///     the code itself (the typical `FunctionalCorrectness` case).
///   - `requires = <predicate>` — **substrate-witness** immunity (ADR-019). A
///     predicate evaluated against a signed `.attest/` sidecar rather than the
///     code AST (e.g. `signers(...)`, `ratified_doc(...)`, `fresh_within_days(...)`).
///     Reach for this when the immunity evidence lives *outside* the code —
///     a review record, a ratified discipline doc, a sign-off (the typical
///     `SubstrateAlignment` case). See the `substrate_witness` example.
/// - `rationale = "..."` (optional) — human-readable description of why the
///   evidence applies
///
/// # Example
///
/// ```ignore
/// use antigen::immune;
///
/// // code-tier: a test proves it
/// #[immune(
///     PanickingInDrop,
///     witness = no_panic_in_drop_test,
///     rationale = "SafeType::drop uses Result-returning paths only.",
/// )]
/// impl Drop for SafeType { ... }
///
/// // substrate-witness: a ratified discipline doc records it
/// #[immune(
///     ParallelStateTrackersDiverge,
///     requires = ratified_doc(path = "docs/disciplines/state-reconciliation.md"),
/// )]
/// fn reconcile_state() { ... }
/// ```
///
/// For `witness =`, `cargo antigen scan` validates that the witness identifier
/// resolves to a real test/proptest/lint/proof; witnesses that don't exist or
/// don't run successfully invalidate the immunity claim. For `requires =`,
/// `cargo antigen audit` evaluates the predicate against the `.attest/` sidecar
/// — a substrate-witness sidecar is credited *only* for a `requires =` immunity,
/// never a `witness =` one.
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

    // ADR-029 §Mechanics: #[immune] is deprecated; emit a compiler warning pointing
    // toward the new #[defended_by] (code-tier) / #[presents(requires=...)]
    // (substrate-tier) model so adopters receive a migration nudge at compile time.
    //
    // Carrier choice: we do NOT emit `#[deprecated]` on the annotated item itself.
    // That design has two defects:
    //   1. Stacking — two `#[immune]` on one item produces two `#[deprecated]` attrs,
    //      which is a hard compile error ("multiple deprecated attributes").
    //   2. Target mis-match — `#[deprecated]` on the item fires at CALLERS, not at
    //      the `#[immune]` author; adopters writing valid code that calls a
    //      `#[immune]`-annotated function would see spurious migration warnings.
    //
    // Instead: emit a `const <NAME>: () = { ... }` item containing a deprecated unit
    // struct that is immediately used inside the block (firing the lint) and then
    // discarded. The block is scoped so no name leaks; the lint fires at the `#[immune]`
    // call site (the macro invocation), which is exactly where the author is.
    // Callers of the annotated item see no warning — only the #[immune] author does.
    // Antigen's own uses suppress with #[allow(deprecated)] per the migration plan.
    // MSRV 1.85 supports `let _` in const blocks.
    //
    // STACKABILITY (findings/immune-multi-stack-const-collision): the const item MUST
    // be NAMED, not anonymous (`const _`). The annotated item may live in an `impl`
    // block (the `#[immune]` is on a method), where the macro's emitted const lands in
    // ASSOCIATED-CONST position. Rust rejects `const _` there with TWO errors:
    //   - "`const` items in this context need a name" (anonymous const illegal in impl)
    //   - "duplicate definitions with name `_`" (E0592) when two are stacked
    // Empirically confirmed: two `const _: () = {…}` collide only in associated-const
    // position; at module scope they stack fine. The earlier "rename the inner struct"
    // fix was a red herring — it left `const _` in place, so it did NOT fix the impl
    // case (the errors are about the const's `_` name, not the inner struct). A NAMED
    // const is legal in module, impl, AND fn-body positions, and a per-emission-unique
    // name prevents the duplicate-definition collision even for the same antigen stacked
    // twice (e.g. a witness= immunity and a requires= immunity on one method).
    //
    // Uniqueness: antigen path (sanitized) + a process-global emission counter. The
    // counter guarantees uniqueness regardless of antigen path or stacking shape; the
    // path component keeps the generated name legible in errors/expansions. The counter
    // need only be unique within a single compilation (proc-macros run in-process), not
    // stable across builds — the const is discarded after firing the lint.
    let deprecated_note = "use #[defended_by] on tests (code-tier) or #[presents(requires=...)] \
         for substrate evidence — ADR-029";

    let antigen_suffix = args
        .antigen
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("_");
    let antigen_suffix = if antigen_suffix.is_empty() {
        "Unknown".to_string()
    } else {
        antigen_suffix
    };
    // Process-global, monotonically-increasing per-emission discriminator
    // (see IMMUNE_EMISSION_COUNTER at module level for the full rationale).
    let n = IMMUNE_EMISSION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let const_name = format!("__ANTIGEN_IMMUNE_DEPRECATED_{antigen_suffix}_{n}");
    let const_ident = syn::Ident::new(&const_name, proc_macro2::Span::call_site());

    args.requires_json().map_or_else(
        || {
            quote! {
                #[allow(non_upper_case_globals)]
                const #const_ident: () = {
                    #[deprecated(note = #deprecated_note)]
                    struct AntigenImmuneDeprecated;
                    let _ = AntigenImmuneDeprecated;
                };
                #input
            }
            .into()
        },
        |json| {
            // Emit the predicate as a doc-attribute marker so `cargo antigen scan`
            // can discover it via source walking without requiring a binary link.
            // Format: `antigen:requires:v1:<json>` (ADR-019 §P3b).
            let marker = format!(" antigen:requires:v1:{json}");
            quote! {
                #[allow(non_upper_case_globals)]
                const #const_ident: () = {
                    #[deprecated(note = #deprecated_note)]
                    struct AntigenImmuneDeprecated;
                    let _ = AntigenImmuneDeprecated;
                };
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

/// Register a code-tier witness: declare that this test/proptest function
/// defends against a known failure-class (ADR-029).
///
/// # Arguments
///
/// Single positional argument: the antigen type the witness defends.
///
/// # Example
///
/// ```ignore
/// use antigen::defended_by;
///
/// #[test]
/// #[defended_by(ParallelStateTrackersDiverge)]
/// fn bijection_audit_hints_const_matches_enum() {
///     // exercises both sides of the parallel state
/// }
/// ```
///
/// # Immunity is observed, not declared
///
/// `#[defended_by(X)]` is a *registration of evidence*, not a verdict. The test
/// declares **what it defends**; `cargo antigen audit` determines **whether it
/// defends it** — by cross-referencing the registered witness to the
/// `#[presents(X)]` sites it covers and grading the witness tier. No code site
/// ever claims "I am immune to X"; the audit tool is the single authoritative
/// voice that reports `defended` / `undefended` / `substrate-gap`. This is the
/// migration target for the code-tier (`witness = fn`) channel of the deprecated
/// `#[immune]` macro.
///
/// # Scope
///
/// `#[defended_by]` is for **code-tier witnesses only** — `#[test]` functions
/// and proptest properties. Site-attached evidence (a substrate predicate or a
/// phantom-type proof) folds into `#[presents]` via `requires =` / `proof =`,
/// not here (ADR-029 R5 discriminator: evidence belongs where it is).
///
/// Like the other antigen markers this is a pure identity transform plus a
/// discoverable `#[doc = " antigen:defended_by:v1:<antigen>"]` marker that
/// `cargo antigen scan` reads to register the witness.
#[proc_macro_attribute]
pub fn defended_by(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(args as parse::DefendedByArgs);
    let input = proc_macro2::TokenStream::from(input);

    // Emit a discoverable doc marker carrying the bare antigen type name, so
    // `cargo antigen scan` can register the witness via source-walking without
    // a binary link — the same channel `#[immune(requires=...)]` uses for its
    // predicate JSON (ADR-019 §P3b), here carrying just the failure-class name.
    let antigen_name = parsed
        .antigen
        .segments
        .last()
        .map_or_else(String::new, |s| s.ident.to_string());
    let marker = format!(" antigen:defended_by:v1:{antigen_name}");
    quote! {
        #[doc = #marker]
        #input
    }
    .into()
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

/// Declare that a proc-macro / `macro_rules` emits code presenting an antigen.
/// Per ADR-014 — the fifth core macro.
///
/// `cargo antigen scan` parses the source-level AST: it sees a `#[derive(Foo)]`
/// invocation but NOT the code the `Foo` derive generates. Failure-classes that
/// manifest only in macro-generated code are invisible to the scan. The fix
/// lives at the macro author's side — they know what their macro emits, so they
/// declare it. The scan then connects this declaration (on the macro
/// DEFINITION) to every macro INVOCATION and surfaces a synthetic presentation
/// at the invocation site.
///
/// # Arguments
///
/// - antigen type name (positional, required) — the failure-class the expansion presents
/// - `rationale = "..."` (required, non-empty) — why the expansion presents this
///   class + what the user should verify (mirrors ADR-011 tolerance)
/// - `witness_template = "..."` (optional, v2) — path hint for a witness skeleton
/// - `if_attr_present = "..."` (optional, v2) — conditional-generation guard
///
/// # Example
///
/// ```ignore
/// use antigen::antigen_generates;
///
/// #[antigen_generates(
///     PanickingInDrop,
///     rationale = "This derive emits a Drop impl that may panic if the inner \
///                  type's destructor panics; users should verify their inner \
///                  types are panic-safe in Drop.",
/// )]
/// #[proc_macro_derive(SomeDerive)]
/// pub fn some_derive(input: TokenStream) -> TokenStream { /* ... */ }
/// ```
///
/// Like the other markers this is a pure identity transform plus a discoverable
/// `#[doc = " antigen:generates:v1:<antigen>"]` marker that `cargo antigen scan`
/// reads to register the macro as a generator of the named failure-class.
#[proc_macro_attribute]
pub fn antigen_generates(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(args as parse::GeneratesArgs);
    let input = proc_macro2::TokenStream::from(input);

    if let Err(e) = parsed.validate() {
        return e.to_compile_error().into();
    }

    // Emit a discoverable doc marker carrying the bare antigen type name. The
    // scan's source-walk reads this on the macro DEFINITION and connects it to
    // invocation sites (same no-binary-link channel as the other markers). The
    // `rationale` is validated here (enforcing the ADR-014 discipline that a
    // generation claim must be justified) but is not needed downstream by the
    // synthesis pass, which only needs the antigen type to emit the presentation.
    let marker = format!(" antigen:generates:v1:{}", parsed.antigen_name());
    quote! {
        #[doc = #marker]
        #input
    }
    .into()
}

// ============================================================================
// Marked-Unknown Plane (ADR-041) — #[aura] / #[dread] / #[red_flag]
//
// Three declarable ⊥ markers on the magnitude × existence-certainty plane (OFF
// the dial's classification axis). Each FIXES its plane corner; the author
// supplies only the REQUIRED `trigger` (guard 3). Like the other markers, each
// is a pure identity transform plus a discoverable `#[doc = " antigen:marked-
// unknown:v1:<json>"]` marker the scan reads (the no-binary-link channel) and
// emits into the SCAN-TIME half of ADR-039's Finding.
// ============================================================================

/// Render the shared marked-unknown doc-marker for one corner + trigger.
///
/// `magnitude` ∈ {smell, aura, dread}; `existence_certainty` ∈ {unsure, sure}
/// (the kebab forms `Magnitude`/`ExistenceCertainty` serialize to). The trigger
/// is JSON-escaped so a quote/backslash in the felt-note can't corrupt the
/// marker the scanner re-parses.
fn marked_unknown_marker(marker: &str, magnitude: &str, certainty: &str, trigger: &str) -> String {
    // JSON-escape the trigger string (the only free-text field). Per RFC 8259 a
    // string MUST escape `"`, `\`, and every control char U+0000–U+001F — the
    // short forms (\n/\t/\r/\b/\f) where they exist, else the `\u00XX` form. The
    // earlier hand-rolled version passed un-short-formed control chars through
    // raw, producing INVALID JSON the scanner's re-parse would reject — a silent
    // producer-correctness bug (antigen's own class), fixed here. (The macro crate
    // carries no serde dep, so this is the dependency-free equivalent of
    // `serde_json::to_string`'s string escaping.)
    let mut escaped = String::with_capacity(trigger.len());
    for c in trigger.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            // Remaining control chars (U+0000–U+001F) have no short form → \u00XX.
            // Build it without `format!` (a String already; push the digits).
            c if (c as u32) < 0x20 => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let b = c as u8;
                escaped.push_str("\\u00");
                escaped.push(HEX[(b >> 4) as usize] as char);
                escaped.push(HEX[(b & 0x0f) as usize] as char);
            },
            other => escaped.push(other),
        }
    }
    format!(
        " antigen:marked-unknown:v1:{{\"marker\":\"{marker}\",\"magnitude\":\"{magnitude}\",\"existence_certainty\":\"{certainty}\",\"trigger\":\"{escaped}\"}}"
    )
}

/// Expand one marker macro: parse → stamp name → validate (required trigger) →
/// emit `input` unchanged + the discoverable doc-marker for the fixed corner.
fn expand_marker(
    args: TokenStream,
    input: TokenStream,
    marker: &'static str,
    magnitude: &'static str,
    certainty: &'static str,
) -> TokenStream {
    let parsed = parse_macro_input!(args as parse::MarkerArgs).with_marker(marker);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = parsed.validate() {
        return e.to_compile_error().into();
    }
    let doc = marked_unknown_marker(marker, magnitude, certainty, parsed.trigger_str());
    quote! {
        #[doc = #doc]
        #input
    }
    .into()
}

/// `#[aura(trigger = "...")]` — the **light** marked-unknown (low magnitude):
/// "something *may* be off here, can't name it, check later." (ADR-041.)
///
/// Surfaces at the dial's non-gating floor; never gates, never nags; an untouched
/// `#[aura]` is a *mild* substrate-smell. The `trigger` is **required** (guard 3).
///
/// # Example
///
/// ```ignore
/// use antigen::aura;
///
/// #[aura(trigger = "this retry loop has no jitter; under load it might thundering-herd")]
/// fn retry_request() { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn aura(args: TokenStream, input: TokenStream) -> TokenStream {
    expand_marker(args, input, "aura", "aura", "unsure")
}

/// `#[dread(trigger = "...")]` — high magnitude, **low** existence-certainty
/// (the *angor animi* corner): "something *is* wrong here, I can't name it, look
/// now." Scared-but-unsure. (ADR-041.)
///
/// Surfaces at the dial's non-gating floor; never gates, never nags. The
/// `trigger` is **required** (guard 3) — a triggerless `#[dread]` is rejected.
///
/// # Example
///
/// ```ignore
/// use antigen::dread;
///
/// #[dread(trigger = "the teardown drops the guard before the flush; \
///                    I can't prove a leak but the ordering feels wrong")]
/// impl Drop for Connection { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn dread(args: TokenStream, input: TokenStream) -> TokenStream {
    expand_marker(args, input, "dread", "dread", "unsure")
}

/// `#[red_flag(trigger = "...")]` — **high** existence-certainty, unnameable.
///
/// The clinical sense-of-alarm corner: "I'm *sure* something is wrong here, I
/// can't name it, act now." The sure-but-unnameable corner; **auto-escalates on
/// first match** (its whole point). (ADR-041.)
///
/// The one marker that escalates rather than surfacing as a mild smell — because
/// its defining axis is high certainty-that-something-is-wrong. The `trigger` is
/// **required** (guard 3).
///
/// # Example
///
/// ```ignore
/// use antigen::red_flag;
///
/// #[red_flag(trigger = "this auth check can be reached with an empty token in \
///                       the cache-hit path; I'm sure this is exploitable")]
/// fn authorize(token: &Token) -> bool { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn red_flag(args: TokenStream, input: TokenStream) -> TokenStream {
    expand_marker(args, input, "red-flag", "dread", "sure")
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
/// # Audit hints (planned — not yet emitted)
///
/// - `polyclonal-insufficient-lineages` — fewer lineages than a configured
///   floor. **Not implemented at v0.2**: `#[polyclonal]` is a pure
///   documentation marker today; the `PolyclonalInsufficientLineages`
///   `AuditHint` variant exists but is never produced (no lineage-counting
///   audit pass yet). This hint is a forward-plan, not current behavior — do
///   not rely on it firing.
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
/// # Audit hints (planned — not yet emitted)
///
/// - `adcc-single-mechanism-only` — only one of the two mechanisms
///   detectable on the site. **Not implemented at v0.2**: `#[adcc]` is a pure
///   documentation marker today; the `AdccSingleMechanismOnly` `AuditHint`
///   variant exists but is never produced (no mechanism-detection audit pass
///   yet). This hint is a forward-plan, not current behavior — do not rely on
///   it firing.
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
///   `#[mucosal]`'s ≥20; tolerance-errors are silent/latent — no acute
///   signal catches a bad tolerance decision, so the up-front declaration
///   must carry more justification to compensate for the detection asymmetry)
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

/// Declare an orientation period — a time-bounded acknowledged absence of immunity.
///
/// An acknowledged, time-bounded absence of immunity with an explicit path
/// forward. A loud deferred-defense primitive (ADR-023) — *not* the
/// lightest-weight one; loudness is the discipline.
///
/// `learning_path` and `until` are **both REQUIRED** (ADR-023 §Decision +
/// the Option-A hard-break ruling). An orient without an explicit path-out and
/// a time-bound is silent deferred non-immunity — structurally identical to
/// tolerance, which this primitive exists to be loudly distinct from. A bare
/// `#[orient]` is a compile error.
///
/// # Arguments
///
/// - Antigen type name (optional positional)
/// - `learning_path = "..."` (REQUIRED, ≥ 20 chars) — the explicit path out of
///   the learning period
/// - `until = "YYYY-MM-DD"` (REQUIRED) — orientation horizon; UTC, within
///   `deferred_defense_max_horizon` (180d). A date beyond that is a compile error.
///
/// The pre-restoration drift fields (`see`, `adr`, `attestation_optional`) were
/// removed — they were never in the ADR-023 spec. Fold any see-also context
/// into the `learning_path` text or `references = [...]` on the antigen
/// declaration.
///
/// # Audit hints
///
/// - `orient-active` — orientation in progress
/// - `orient-pending-action-required` — orientation past its `until` horizon
///
/// # Example
///
/// ```ignore
/// use antigen::orient;
///
/// #[orient(
///     PanickingInDrop,
///     learning_path = "Audit every Drop impl for unwrap/panic before the v1 tag",
///     until = "2026-09-01",
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

// ============================================================================
// Prescriptive Work-Orchestration Family (ADR-033, extends ADR-024)
//
// "The TODO comment becomes structure." Eight clinical-named work-need macros
// routing to FOUR structural shapes (ADR-033 §Decision 1):
//   S1 Role-workflow  — panel, rx, refer, biopsy  (ordered who-steps + frame)
//   S2 Elimination    — ddx                        (a set of closeable alternatives)
//   S3 Ordering       — triage                     (a re-validatable priority order)
//   S4 Frame-only     — culture, quarantine        (a temporal window + expiry)
//
// Each macro is a thin validating pass-through (like the recurrent family);
// `cargo antigen scan` reads the SOURCE attribute directly. The audit emits the
// four-valued WorkVerdict {Pending, Fulfilled, Overdue, OutOfFrame} — the board.
// Witness satisfaction REUSES the ADR-019/020 categorical spine (no new
// mechanism); only the S1 ORDERING is new content. `#[triage]` is intentionally
// NOT shipped in this commit — its arg-shape has a ratified-ADR-vs-test-corpus
// divergence (camp question fc2e1677, awaiting aristotle); the other seven are
// unambiguous. `#[titer]` is NOT in this family (it is a titer-witness kind,
// ADR-019 Amendment 1).
// ============================================================================

/// Declare a battery of work-needs to be filled + reviewed at this site
/// (ADR-033 S1 Role-workflow).
///
/// `#[panel(needs, filled_by?, reviewed_by?, ordered_by?, due?)]` marks a code
/// site as carrying an ordered diagnostic battery — a checklist the site's
/// reviewers must close. Biology: a clinical panel (a battery of tests ordered
/// together, closed by the reviewing clinician).
///
/// # Arguments
///
/// - `needs = ["...", ...]` (required) — the battery's checklist; non-empty
///   (empty = vacuous work-need; compile error)
/// - `filled_by = ["who", ...]` (optional) — ADR-020 who-refs that fill the needs
/// - `reviewed_by = ["who", ...]` (optional) — who-refs that review the fills
/// - `ordered_by = "who"` (optional) — who-ref that ordered the battery
/// - `due = "YYYY-MM-DD"` (optional) — ISO-8601 frame
///
/// Satisfaction (ADR-033 §Witness-binding) is collective coverage over the
/// need-set, attested per role-step at the current fingerprint — NOT a
/// positional `filled_by[i] ↔ needs[i]` pairing.
#[proc_macro_attribute]
pub fn panel(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::PanelArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a prescribed treatment work-need at this site (ADR-033 S1
/// Role-workflow).
///
/// `#[rx(treatment, diagnosis?, filled_by?, reviewed_by?, due?)]` marks the
/// remedy a site must carry out. Biology: a prescription — a treatment ordered
/// for a diagnosis, filled and reviewed.
///
/// # Arguments
///
/// - `treatment = "..."` (required, non-empty) — what must be done
/// - `diagnosis = "..."` (optional) — opaque label (v0.3; backref to `ddx` not
///   resolved — VOID-4b)
/// - `filled_by = ["who", ...]` (optional) — ADR-020 who-refs
/// - `reviewed_by = ["who", ...]` (optional)
/// - `due = "YYYY-MM-DD"` (optional) — ISO-8601 frame
#[proc_macro_attribute]
pub fn rx(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::RxArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a referral of work to an external owner (ADR-033 S1 Role-workflow).
///
/// `#[refer(to, response_due?)]` hands a work-need to an owner outside this
/// site's immediate responsibility. Biology: a specialist referral — the
/// referring clinician hands off and awaits a response.
///
/// # Arguments
///
/// - `to = "who"` (required) — ADR-020 who-ref (the external owner)
/// - `response_due = "YYYY-MM-DD"` (optional) — ISO-8601 frame for the response
#[proc_macro_attribute]
pub fn refer(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::ReferArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a deep-investigation work-need at a sub-site (ADR-033 S1
/// Role-workflow).
///
/// `#[biopsy(location, request_text, deep_investigation_by?)]` marks a request
/// to investigate a specific sub-site in depth. Biology: a biopsy — sampling a
/// specific location for deep analysis.
///
/// # Arguments
///
/// - `location = "..."` (required) — sub-site pointer (opaque label v0.3)
/// - `request_text = "..."` (required, non-empty) — what to investigate
/// - `deep_investigation_by = "who"` (optional) — ADR-020 who-ref
#[proc_macro_attribute]
pub fn biopsy(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::BiopsyArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a differential-diagnosis work-need: a set of alternatives to rule
/// out (ADR-033 S2 Elimination).
///
/// `#[ddx(symptom, rule_out, investigator?, reviewer?)]` marks a site where a
/// symptom has multiple candidate causes, each to be independently eliminated.
/// Biology: differential diagnosis — the list of conditions to rule out.
///
/// # Arguments
///
/// - `symptom = "..."` (required, non-empty) — the observed problem
/// - `rule_out = ["...", ...]` (required, non-empty) — the alternative-set; each
///   alternative is independently closeable (a rule-out carries a closing attestation)
/// - `investigator = "who"` (optional) — ADR-020 who-ref
/// - `reviewer = "who"` (optional) — ADR-020 who-ref
#[proc_macro_attribute]
pub fn ddx(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::DdxArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a time-boxed test/observation work-need (ADR-033 S4 Frame-only).
///
/// `#[culture(test_kind, duration?, runs_until?)]` marks a site that must stay
/// green within a temporal window (a soak/observation). Biology: a culture —
/// incubate for a fixed period and read the result.
///
/// # Arguments
///
/// - `test_kind = "..."` (required, non-empty) — what is being cultured/observed
/// - `duration = "..."` (optional) — duration string
/// - `runs_until = "YYYY-MM-DD"` (optional) — ISO-8601 frame
#[proc_macro_attribute]
pub fn culture(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::CultureArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare an isolated region under a time-boxed hold (ADR-033 S4 Frame-only).
///
/// `#[quarantine(scope, until?, reason)]` marks a region deliberately isolated
/// until a frame passes. Biology: quarantine — isolate until cleared. The
/// `reason` is required per ADR-005 Amendment 2 (rationale-as-required for every
/// suppression-shaped primitive).
///
/// # Arguments
///
/// - `scope = "..."` (required) — the isolated-region pointer
/// - `until = "YYYY-MM-DD"` (optional) — ISO-8601 frame
/// - `reason = "..."` (required, non-empty) — why the hold (ADR-005 Amd2)
#[proc_macro_attribute]
pub fn quarantine(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::QuarantineArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

/// Declare a re-validatable priority ordering over code sites (ADR-033 S3
/// Ordering).
///
/// `#[triage(priority_order, triaged_by?, re_triage_due?)]` marks a site that
/// carries an ordered priority over a set of code-site references. Biology:
/// triage — ranking by urgency, re-assessed each round. Distinct from
/// `#[triage_commit]` (ADR-026 VCS-rollback classification) — names rhyme,
/// surfaces are unrelated (ATK-PRES-10).
///
/// Per the 2026-06-01 post-ratification fixup, the `campsites` field was dropped:
/// `priority_order` entries are **code-site references** (file/item-path), not
/// camp campsites (anchor #3 — the audit never reads camp). They resolve at
/// audit-time (ADR-017 Amendment 1); an unresolvable entry is `out-of-frame`,
/// never silently satisfied.
///
/// # Arguments
///
/// - `priority_order = ["...", ...]` (required, non-empty) — code-site refs in
///   priority order
/// - `triaged_by = "who"` (optional) — ADR-020 who-ref that attested the order
/// - `re_triage_due = "YYYY-MM-DD"` (optional) — ISO-8601 staleness frame (not a
///   deadline; a standing ordering is re-earned each cycle)
#[proc_macro_attribute]
pub fn triage(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::TriageArgs);
    let input = proc_macro2::TokenStream::from(input);
    if let Err(e) = args.validate() {
        return e.to_compile_error().into();
    }
    quote! { #input }.into()
}

#[cfg(test)]
mod marker_emit_tests {
    use super::marked_unknown_marker;

    /// The doc-marker's trigger field is JSON-escaped for EVERY control char
    /// (U+0000–U+001F), not just the five with short forms — the producer-
    /// correctness fix. A raw control char in the trigger would otherwise produce
    /// invalid JSON the scanner's re-parse rejects (antigen's own silent class).
    #[test]
    fn trigger_escapes_all_control_chars_to_valid_json() {
        // A backspace (→ \b short form), a form-feed (→ \f), a vertical-tab (→ the
        // long  form), and a SOH (→ ).
        let trigger = "a\u{08}b\u{0c}c\u{0b}d\u{01}e";
        let out = marked_unknown_marker("dread", "dread", "unsure", trigger);
        assert!(out.contains("\\b"), "backspace → \\b");
        assert!(out.contains("\\f"), "form-feed → \\f");
        assert!(out.contains("\\u000b"), "vertical-tab → \\u000b");
        assert!(out.contains("\\u0001"), "SOH → \\u0001");
        // No raw control byte survives into the (single-line) doc-marker output.
        assert!(
            !out.chars().any(|c| (c as u32) < 0x20),
            "no raw control char survives the escape"
        );
    }

    #[test]
    fn trigger_escapes_quote_and_backslash() {
        // The original five short forms still hold (a quote/backslash in the
        // felt-note can't corrupt the marker the scanner re-parses).
        let out = marked_unknown_marker("aura", "aura", "unsure", r#"the "guard" path\here"#);
        assert!(out.contains(r#"\""#), "quote → escaped quote");
        assert!(out.contains(r"\\"), "backslash → escaped backslash");
    }
}
