//! The scan parsing engine — `ScanVisitor` + the syn-rendering helpers.
//!
//! Extracted from the former monolithic `scan.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). `ScanVisitor` is a `syn::visit::Visit` walker
//! that turns one parsed `.rs` file's AST into the typed declaration records
//! (`#[antigen]` / `#[presents]` / `#[immune]` / `#[defended_by]` / the family
//! macros / `#[descended_from]`), plus the `render_path` / `render_type` /
//! `attr_is` / `extract_requires_predicate_from_attrs` utilities the walk +
//! synthesis passes share. It calls NO pass fn — a dependency-free parse leaf —
//! so the scan pipeline layers acyclically (parse <- {synthesis, finalize, walk}).
//! It holds no stop-authority (single-conductor invariant, ADR-036).
//!
//! ADR-036 refinement (recorded in the ADR's supersede-note for this build): the
//! parsing engine is its OWN module rather than folded into `walk.rs` as the
//! original file-list implied — folding it would create a
//! `walk -> finalize -> synthesis -> walk` module cycle. The scan-time attribute
//! arg-parsers (`Scan*Args`) stay in the `scan` root (their white-box field-tests
//! live there); this leaf imports them from `super` (a child module reads its
//! parent's private types + fields).
//!
//! API-invisible: crate-internal; the pieces the passes share are re-exported
//! `pub(crate)` at the scan root (the visitor under `pub fn new` so its internal
//! bookkeeping stays encapsulated across the new module boundary).

use std::path::PathBuf;

use antigen_macros::presents;
use syn::visit::Visit;

use super::{
    AntigenDeclaration, ConvergentEvidence, ConvergentEvidenceKind, Defense, DeferredDefense,
    DeferredDefenseKind, GeneratesDeclaration, Immunity, ItemTarget, LineageEdge, MatchKind,
    MucosalDeclaration, MucosalKindTag, ParseFailure, PrescriptiveDeclaration, PrescriptiveKind,
    Presentation, RecurrentDeclaration, RecurrentKind, ScanReport, Toleration,
};
use super::{
    ScanAnergyArgs, ScanAntigenArgs, ScanClonalArgs, ScanCrossreactiveArgs, ScanDiagnosticArgs,
    ScanGeneratesArgs, ScanIggArgs, ScanImmuneArgs, ScanImmunosuppressArgs, ScanMucosalArgs,
    ScanOrientArgs, ScanPoxpartyArgs, ScanPrescriptiveArgs, ScanPresentsArgs, ScanRecurrentArgs,
    ScanToleranceArgs,
};

/// AST visitor that extracts antigen-related attributes.
#[presents(ScanVisitorDigestAssignmentOmission)]
pub struct ScanVisitor<'a> {
    file_path: PathBuf,
    report: &'a mut ScanReport,
    /// Context stack for nested items. The current top of stack is the
    /// enclosing-impl context for any `visit_impl_item_fn` call — so that
    /// a method's `ItemTarget::ImplFn` knows which impl block it lives in.
    /// W3 (sweep A2): structural item-identity tracking.
    impl_stack: Vec<(Option<String>, String)>,
    /// Context stack for nested traits — analogous to `impl_stack`, but
    /// for `visit_trait_item_fn` so trait methods carry the enclosing
    /// trait identifier in `ItemTarget::TraitFn`.
    trait_stack: Vec<String>,
    /// Structural digest of the item currently being visited, set by each
    /// `visit_item_*` before it calls [`Self::check_attrs`] so that
    /// `extract_immune` / `extract_tolerance` can stamp the defended item's
    /// digest onto the substrate-witness record without threading it through
    /// every `check_attrs` call site. Empty between items.
    current_item_digest: String,
}

impl<'a> ScanVisitor<'a> {
    /// Construct a fresh visitor for one file, targeting `report`.
    ///
    /// The collection pass (`scan_workspace`) builds one per `.rs` file and
    /// drives it via [`syn::visit::Visit::visit_file`]; the context stacks +
    /// digest start empty. A constructor (rather than `pub` fields) keeps the
    /// visitor's internal bookkeeping encapsulated across the module boundary the
    /// ADR-036 decomposition introduces.
    pub const fn new(file_path: PathBuf, report: &'a mut ScanReport) -> Self {
        Self {
            file_path,
            report,
            impl_stack: Vec::new(),
            trait_stack: Vec::new(),
            current_item_digest: String::new(),
        }
    }
}

impl ScanVisitor<'_> {
    /// Compute 1-indexed line number for a span by counting newlines in source up
    /// to the span's start.
    ///
    /// Resolve the source line of a specific `#[attr]` invocation via
    /// `syn::spanned::Spanned::span().start().line`. Each per-instance call
    /// reports the line of *that* invocation rather than the first match in
    /// the file (the pre-fix heuristic that broke ATK-A2-002 for multi-
    /// instance scenarios).
    ///
    /// Falls back to `0` only if the span info is unavailable (which on
    /// stable rustc with `proc-macro2`'s default features is rare; a 0
    /// return means "we don't know," which is honest).
    pub fn line_of_attr(attr: &syn::Attribute) -> usize {
        use syn::spanned::Spanned;
        attr.span().start().line
    }

    fn extract_antigen(&mut self, item: &syn::ItemStruct, attr: &syn::Attribute) {
        let type_name = item.ident.to_string();
        let line = Self::line_of_attr(attr);

        if let syn::Meta::List(list) = &attr.meta {
            match syn::parse2::<ScanAntigenArgs>(list.tokens.clone()) {
                Ok(args) => {
                    let category: Vec<crate::category::AntigenCategory> = args
                        .category
                        .iter()
                        .filter_map(|s| crate::category::AntigenCategory::parse_category(s))
                        .collect();
                    self.report.antigens.push(AntigenDeclaration {
                        name: args.name,
                        type_name,
                        file: self.file_path.clone(),
                        line,
                        family: args.family,
                        summary: args.summary,
                        fingerprint: args.fingerprint,
                        canonical_path: None,
                        category,
                        provenance: args.provenance,
                        presentation: args.presentation,
                    });
                }
                Err(_) => {
                    // Malformed attribute: record with empty name so scan output
                    // surfaces the file for investigation rather than silently skipping.
                    self.report.antigens.push(AntigenDeclaration {
                        name: String::new(),
                        type_name,
                        file: self.file_path.clone(),
                        line,
                        family: None,
                        summary: None,
                        fingerprint: None,
                        canonical_path: None,
                        category: Vec::new(),
                        provenance: None,
                        presentation: None,
                    });
                }
            }
        }
    }

    fn extract_presents(
        &mut self,
        attr: &syn::Attribute,
        all_attrs: &[syn::Attribute],
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        let (antigen_type, requires_predicate, proof) = if let syn::Meta::List(list) = &attr.meta {
            // ADR-029 R5: `#[presents]` may now carry site-attached evidence
            // (`requires = <predicate>`, `proof = <expr>`), so parse the full
            // arg form, not a bare `syn::Path`. The antigen is still the leading
            // positional path; its last segment is the bare type name regardless
            // of qualifier (the W3 structural form — ATK-A2-001).
            match syn::parse2::<ScanPresentsArgs>(list.tokens.clone()) {
                Ok(args) => (args.antigen_type, args.requires_predicate, args.proof),
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[presents] attribute: {e}"),
                    });
                    return;
                }
            }
        } else {
            return;
        };
        // Two-channel substrate-witness discovery (same as `extract_immune`):
        // the source attribute is primary; the `antigen:requires:v1:` doc marker
        // the macro emits is the fallback for already-expanded source.
        let requires_predicate =
            requires_predicate.or_else(|| extract_requires_predicate_from_attrs(all_attrs));
        let line = Self::line_of_attr(attr);
        self.report.presentations.push(Presentation {
            antigen_type,
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
            match_kind: MatchKind::ExplicitMarker,
            canonical_path: None,
            inherited_from: None,
            structural_fingerprint: self.current_item_digest.clone(),
            requires_predicate,
            proof,
        });
    }

    fn extract_immune(
        &mut self,
        attr: &syn::Attribute,
        all_attrs: &[syn::Attribute],
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            // Scan records the witness expression verbatim; validity
            // classification (Test, Proptest, PhantomType, Function, External)
            // and behavioral verification (cargo test invocation) are the
            // audit module's responsibility. ADR-005 sub-clause F: the
            // trust boundary at "immunity claim" is checked by audit, not
            // by scan — scan provides the substrate, audit decides validity.
            let args = match syn::parse2::<ScanImmuneArgs>(list.tokens.clone()) {
                Ok(args) => args,
                Err(e) => {
                    // Malformed #[immune] args: record a parse failure rather
                    // than silently inserting a ghost immunity record with empty
                    // antigen_type and witness. A ghost record would pass
                    // WitnessStatus::Missing detection only if the empty-string
                    // check fires, and would produce a misleading "0 unaddressed
                    // presentations" result. ADR-005: every trust boundary requires
                    // a validation check; malformed immunity claims are not claims.
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[immune] attribute: {e}"),
                    });
                    return;
                }
            };
            // ADR-019 §P3b: substrate-witness discovery has two channels.
            // The primary channel parses `requires = <predicate>` directly
            // from the source attribute (`args.requires_predicate`). The
            // fallback channel reads the `antigen:requires:v1:<json>` doc
            // marker the macro emits — useful when scanning crates already
            // compiled with rc.1 macros, or in any case the source attribute
            // didn't survive a build-script rewrite. Source wins because the
            // doc marker is only present POST macro expansion, and `syn`
            // parses the WRITTEN source. This is the rc.2 fix: rc.1 relied
            // exclusively on the doc-marker channel, which never engaged
            // because scan walks written source.
            let requires_predicate = args
                .requires_predicate
                .clone()
                .or_else(|| extract_requires_predicate_from_attrs(all_attrs));
            let line = Self::line_of_attr(attr);
            self.report.immunities.push(Immunity {
                antigen_type: args.antigen_type,
                witness: args.witness,
                requires_predicate,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
                canonical_path: None,
                structural_fingerprint: self.current_item_digest.clone(),
            });
        }
    }

    /// Extract a `#[defended_by(antigen_type)]` code-tier witness registration
    /// (ADR-029). Mirrors `extract_presents`'s single-positional-`syn::Path`
    /// parse: the body is the bare antigen type the witness defends. The
    /// cross-reference to the `#[presents]` sites it covers is computed at
    /// audit time — scan only records the registration.
    fn extract_defended_by(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        let antigen_type = if let syn::Meta::List(list) = &attr.meta {
            match syn::parse2::<syn::Path>(list.tokens.clone()) {
                Ok(path) => path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default(),
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[defended_by] attribute: {e}"),
                    });
                    return;
                }
            }
        } else {
            // No `(...)` body: a bare `#[defended_by]` with no antigen is not a
            // registration — it declares a witness for nothing. Surface it
            // rather than recording a ghost defense with an empty antigen_type.
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "#[defended_by] requires an antigen type argument, \
                        e.g. #[defended_by(ParallelStateTrackersDiverge)]"
                    .to_string(),
            });
            return;
        };

        if antigen_type.is_empty() {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "#[defended_by] antigen type resolved to an empty path".to_string(),
            });
            return;
        }

        let line = Self::line_of_attr(attr);
        self.report.defenses.push(Defense {
            antigen_type,
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
            // Intra-workspace by default; the `--include-deps` driver stamps the
            // canonical_path post-scan for cross-crate defenses (ADR-017, like
            // immunities/presentations).
            canonical_path: None,
        });
    }

    /// Extract a `#[antigen_generates(X, rationale = "...")]` declaration on a
    /// macro definition (ADR-014). Records a [`GeneratesDeclaration`] keyed by
    /// the macro identifier used at invocation sites — see
    /// [`Self::macro_name_for_generates`] for resolution.
    fn extract_generates(
        &mut self,
        attr: &syn::Attribute,
        all_attrs: &[syn::Attribute],
        item_target: &ItemTarget,
    ) {
        let syn::Meta::List(list) = &attr.meta else {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "#[antigen_generates] requires arguments, e.g. \
                        #[antigen_generates(PanickingInDrop, rationale = \"...\")]"
                    .to_string(),
            });
            return;
        };
        let args = match syn::parse2::<ScanGeneratesArgs>(list.tokens.clone()) {
            Ok(a) => a,
            Err(e) => {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!("malformed #[antigen_generates] attribute: {e}"),
                });
                return;
            }
        };

        if args.antigen_type.is_empty() {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "#[antigen_generates] antigen type resolved to an empty path".to_string(),
            });
            return;
        }
        // ADR-014 §Sub-clause F: a generation claim without rationale is not a
        // claim. Mirror the macro-side validate() at scan time so the source-walk
        // path enforces the same discipline (the macro may not have expanded).
        if args.rationale.trim().is_empty() {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: format!(
                    "#[antigen_generates({})] requires a non-empty `rationale = \"...\"` \
                     — the macro author must justify what the expansion presents",
                    args.antigen_type
                ),
            });
            return;
        }

        let macro_name = Self::macro_name_for_generates(all_attrs, item_target);
        if macro_name.is_empty() {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: format!(
                    "#[antigen_generates({})] could not resolve a macro name to register \
                     — apply it to a #[proc_macro_derive(Name)] / #[proc_macro_attribute] fn \
                     or a `macro_rules!` definition",
                    args.antigen_type
                ),
            });
            return;
        }

        let line = Self::line_of_attr(attr);
        self.report
            .generates_declarations
            .push(GeneratesDeclaration {
                antigen_type: args.antigen_type,
                rationale: args.rationale,
                macro_name,
                file: self.file_path.clone(),
                line,
                canonical_path: None,
            });
    }

    /// Resolve the macro identifier a `#[antigen_generates]` declaration
    /// registers — the name that appears at INVOCATION sites:
    /// - `#[proc_macro_derive(Name)]` / `#[proc_macro_derive(Name, attributes(..))]`
    ///   → `Name` (matches `#[derive(Name)]`);
    /// - `#[proc_macro_attribute]` → the annotated fn's name (matches `#[name]`);
    /// - a `macro_rules! name` item (`ItemTarget::Fn` carrying the macro ident,
    ///   per `visit_item_macro`) → that name (matches `name!(..)`);
    /// - otherwise the item's own name (fn fallback).
    fn macro_name_for_generates(all_attrs: &[syn::Attribute], item_target: &ItemTarget) -> String {
        // Prefer the derive name from a sibling `#[proc_macro_derive(Name, ..)]`.
        for a in all_attrs {
            if attr_is(a, "proc_macro_derive") {
                if let syn::Meta::List(list) = &a.meta {
                    // First token of the derive args is the derive name.
                    if let Ok(path) = syn::parse2::<syn::Path>(list.tokens.clone()) {
                        if let Some(seg) = path.segments.last() {
                            return seg.ident.to_string();
                        }
                    }
                    // `#[proc_macro_derive(Name, attributes(..))]`: parse just the
                    // leading ident before the comma.
                    if let Some(proc_macro2::TokenTree::Ident(id)) =
                        list.tokens.clone().into_iter().next()
                    {
                        return id.to_string();
                    }
                }
            }
        }
        // Fallback: the item's own name (proc-macro-attribute fn, or macro_rules
        // ident which `visit_item_macro` records as `ItemTarget::Fn(name)`).
        match item_target {
            ItemTarget::Fn(name) => name.clone(),
            _ => String::new(),
        }
    }

    fn extract_tolerance(
        &mut self,
        attr: &syn::Attribute,
        all_attrs: &[syn::Attribute],
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanToleranceArgs>(list.tokens.clone()) {
                Ok(args) => args,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[antigen_tolerance] attribute: {e}"),
                    });
                    return;
                }
            };
            // Per ADR-011 §Mechanics §1: rationale required + non-empty.
            // Scan side enforces the same boundary the macro enforces — a
            // tolerance without rationale is silent suppression.
            if args.rationale.is_empty() {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: "#[antigen_tolerance] requires non-empty rationale".to_string(),
                });
                return;
            }
            // Same two-channel discovery as immunity (source-attr primary,
            // doc-marker fallback). See `extract_immune` for the full
            // rationale — this branch is the tolerance-side mirror.
            let requires_predicate = args
                .requires_predicate
                .clone()
                .or_else(|| extract_requires_predicate_from_attrs(all_attrs));
            let line = Self::line_of_attr(attr);
            self.report.tolerances.push(Toleration {
                antigen_type: args.antigen_type,
                rationale: args.rationale,
                until: args.until,
                see: args.see,
                requires_predicate,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
                canonical_path: None,
                structural_fingerprint: self.current_item_digest.clone(),
            });
        }
    }

    // ============================================================================
    // Deferred-Defense Family extraction methods (ADR-023)
    // ============================================================================

    fn extract_anergy(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanAnergyArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[anergy] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.deferred_defenses.push(DeferredDefense {
                kind: DeferredDefenseKind::Anergy,
                antigen_type: args.antigen_type,
                text: args.reason,
                until: if args.until.is_empty() {
                    None
                } else {
                    Some(args.until)
                },
                expected_co_stimulation: args.expected_co_stimulation,
                signed_by: args.signed_by,
                see: Vec::new(),
                // anergy carries no duration cap (it does not auto-expire).
                since: None,
                duration_cap: None,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_immunosuppress(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanImmunosuppressArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[immunosuppress] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.deferred_defenses.push(DeferredDefense {
                kind: DeferredDefenseKind::Immunosuppress,
                antigen_type: args.antigen_type,
                text: args.rationale,
                until: if args.until.is_empty() {
                    None
                } else {
                    Some(args.until)
                },
                expected_co_stimulation: None,
                signed_by: args.signed_by,
                see: Vec::new(),
                // since + duration_cap are now TYPED fields (no longer `see[]`
                // string tags) so the audit can compute elapsed-days vs cap and
                // emit ImmunosuppressDurationCapExceeded.
                since: args.since.clone(),
                duration_cap: args.duration_cap,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_poxparty(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanPoxpartyArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[poxparty] attribute: {e}"),
                    });
                    return;
                }
            };
            let mut see = Vec::new();
            if let Some(name) = &args.name {
                see.push(format!("exercise:{name}"));
            }
            if let Some(rationale) = &args.rationale {
                see.push(format!("rationale:{rationale}"));
            }
            let line = Self::line_of_attr(attr);
            self.report.deferred_defenses.push(DeferredDefense {
                kind: DeferredDefenseKind::Poxparty,
                antigen_type: args.antigen_type,
                text: args.exercise_type,
                until: if args.until.is_empty() {
                    None
                } else {
                    Some(args.until)
                },
                expected_co_stimulation: None,
                signed_by: args.signed_by,
                see,
                // poxparty carries no duration cap (cfg-gated, not time-capped).
                since: None,
                duration_cap: None,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_orient(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        // #[orient] with no args (bare attribute) is valid — acknowledge
        // orientation period with zero configuration.
        match &attr.meta {
            syn::Meta::List(list) => {
                let args = match syn::parse2::<ScanOrientArgs>(list.tokens.clone()) {
                    Ok(a) => a,
                    Err(e) => {
                        self.report.parse_failures.push(ParseFailure {
                            file: self.file_path.clone(),
                            error: format!("malformed #[orient] attribute: {e}"),
                        });
                        return;
                    }
                };
                let line = Self::line_of_attr(attr);
                let mut adr_see = args.see.clone();
                if let Some(adr) = &args.adr {
                    adr_see.push(format!("adr:{adr}"));
                }
                self.report.deferred_defenses.push(DeferredDefense {
                    kind: DeferredDefenseKind::Orient,
                    antigen_type: args.antigen_type,
                    text: String::new(),
                    until: None,
                    expected_co_stimulation: None,
                    signed_by: None,
                    see: adr_see,
                    since: None,
                    duration_cap: None,
                    file: self.file_path.clone(),
                    line,
                    item_kind: item_kind.to_string(),
                    item_target,
                });
            }
            syn::Meta::Path(_) => {
                // Bare `#[orient]` — valid, record with empty fields.
                let line = Self::line_of_attr(attr);
                self.report.deferred_defenses.push(DeferredDefense {
                    kind: DeferredDefenseKind::Orient,
                    antigen_type: None,
                    text: String::new(),
                    until: None,
                    expected_co_stimulation: None,
                    signed_by: None,
                    see: Vec::new(),
                    since: None,
                    duration_cap: None,
                    file: self.file_path.clone(),
                    line,
                    item_kind: item_kind.to_string(),
                    item_target,
                });
            }
            syn::Meta::NameValue(_) => {
                // `#[orient = value]` is not a valid orient invocation; ignore.
            }
        }
    }

    fn extract_descended_from(&mut self, attr: &syn::Attribute, item_target: &ItemTarget) {
        // ADR-013: `#[descended_from]` is meaningful only on antigen-type
        // declarations (unit `struct` and class-shaped `enum`). Other
        // placements — impl blocks, free functions, traits, methods —
        // surface as parse_failures so the user sees what got dropped
        // rather than the visitor silently no-op'ing them.
        let child = match item_target {
            ItemTarget::Struct(name) | ItemTarget::Enum(name) => name.clone(),
            other => {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!(
                        "#[descended_from] on `{}` is not a type declaration; \
                         this attribute is meaningful only on `struct` and `enum` \
                         antigen declarations",
                        other.label()
                    ),
                });
                return;
            }
        };

        let syn::Meta::List(list) = &attr.meta else {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "malformed #[descended_from] attribute: expected `(parent)`".to_string(),
            });
            return;
        };

        // Body is a single positional `syn::Path`, mirroring
        // `extract_presents`. Last segment becomes the bare parent type
        // name — module-path qualification is an A3+ ADR-class question
        // (ATK-A3-005), so for now we keep names bare.
        let parent = match syn::parse2::<syn::Path>(list.tokens.clone()) {
            Ok(path) => path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            Err(e) => {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!("malformed #[descended_from] attribute: {e}"),
                });
                return;
            }
        };

        if parent.is_empty() {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "#[descended_from] requires a parent path argument".to_string(),
            });
            return;
        }

        let line = Self::line_of_attr(attr);
        self.report.lineage_edges.push(LineageEdge {
            child,
            parent,
            file: self.file_path.clone(),
            line,
            parent_canonical_path: None,
            child_canonical_path: None,
        });
    }

    fn check_attrs(&mut self, attrs: &[syn::Attribute], item_kind: &str, item_target: &ItemTarget) {
        for attr in attrs {
            if attr_is(attr, "presents") {
                self.extract_presents(attr, attrs, item_kind, item_target.clone());
            } else if attr_is(attr, "immune") {
                self.extract_immune(attr, attrs, item_kind, item_target.clone());
            } else if attr_is(attr, "antigen_tolerance") {
                self.extract_tolerance(attr, attrs, item_kind, item_target.clone());
            } else if attr_is(attr, "descended_from") {
                self.extract_descended_from(attr, item_target);
            } else if attr_is(attr, "defended_by") {
                self.extract_defended_by(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "antigen_generates") {
                self.extract_generates(attr, attrs, item_target);
            // Deferred-Defense Family (ADR-023)
            } else if attr_is(attr, "anergy") {
                self.extract_anergy(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "immunosuppress") {
                self.extract_immunosuppress(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "poxparty") {
                self.extract_poxparty(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "orient") {
                self.extract_orient(attr, item_kind, item_target.clone());
            // Convergent-Evidence Family (ADR-024)
            } else if attr_is(attr, "diagnostic") {
                self.extract_diagnostic(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "clonal") {
                self.extract_clonal(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "igg") {
                self.extract_igg(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "crossreactive") {
                self.extract_crossreactive(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "polyclonal") {
                self.extract_convergent_marker(
                    attr,
                    item_kind,
                    item_target.clone(),
                    ConvergentEvidenceKind::Polyclonal,
                );
            } else if attr_is(attr, "monoclonal") {
                self.extract_convergent_marker(
                    attr,
                    item_kind,
                    item_target.clone(),
                    ConvergentEvidenceKind::Monoclonal,
                );
            } else if attr_is(attr, "adcc") {
                self.extract_convergent_marker(
                    attr,
                    item_kind,
                    item_target.clone(),
                    ConvergentEvidenceKind::Adcc,
                );
            } else {
                // v0.2 families (recurrent-emergence + mucosal-boundary)
                // dispatch in a sibling helper to keep check_attrs concise.
                self.check_v02_family_attr(attr, item_kind, item_target);
            }
        }
    }

    /// Dispatch the v0.2 recurrent-emergence + mucosal-boundary attribute
    /// families (ADR-024 §Family 2, ADR-027). Split out of `check_attrs` so
    /// the primary attribute matcher stays readable.
    fn check_v02_family_attr(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: &ItemTarget,
    ) {
        // Recurrent-Emergence Family (ADR-024 §Family 2)
        if attr_is(attr, "itch") {
            self.extract_recurrent(attr, item_kind, item_target.clone(), RecurrentKind::Itch);
        } else if attr_is(attr, "recurrence_anchor") {
            self.extract_recurrent(
                attr,
                item_kind,
                item_target.clone(),
                RecurrentKind::RecurrenceAnchor,
            );
        } else if attr_is(attr, "crystallize") {
            self.extract_recurrent(
                attr,
                item_kind,
                item_target.clone(),
                RecurrentKind::Crystallize,
            );
        } else if attr_is(attr, "chronic") {
            self.extract_recurrent(attr, item_kind, item_target.clone(), RecurrentKind::Chronic);
        } else if attr_is(attr, "saturate") {
            self.extract_recurrent(
                attr,
                item_kind,
                item_target.clone(),
                RecurrentKind::Saturate,
            );
        } else if attr_is(attr, "strand") {
            self.extract_recurrent(attr, item_kind, item_target.clone(), RecurrentKind::Strand);
        // Mucosal Boundary Family (ADR-027 + Amendment 1)
        } else if attr_is(attr, "mucosal") {
            self.extract_mucosal(
                attr,
                item_kind,
                item_target.clone(),
                MucosalKindTag::Mucosal,
            );
        } else if attr_is(attr, "mucosal_delegate") {
            self.extract_mucosal(
                attr,
                item_kind,
                item_target.clone(),
                MucosalKindTag::MucosalDelegate,
            );
        } else if attr_is(attr, "mucosal_tolerant") {
            self.extract_mucosal(
                attr,
                item_kind,
                item_target.clone(),
                MucosalKindTag::MucosalTolerant,
            );
        // Prescriptive Work-Orchestration Family (ADR-033)
        } else if attr_is(attr, "panel") {
            self.extract_prescriptive(
                attr,
                item_kind,
                item_target.clone(),
                PrescriptiveKind::Panel,
            );
        } else if attr_is(attr, "rx") {
            self.extract_prescriptive(attr, item_kind, item_target.clone(), PrescriptiveKind::Rx);
        } else if attr_is(attr, "refer") {
            self.extract_prescriptive(
                attr,
                item_kind,
                item_target.clone(),
                PrescriptiveKind::Refer,
            );
        } else if attr_is(attr, "biopsy") {
            self.extract_prescriptive(
                attr,
                item_kind,
                item_target.clone(),
                PrescriptiveKind::Biopsy,
            );
        } else if attr_is(attr, "ddx") {
            self.extract_prescriptive(attr, item_kind, item_target.clone(), PrescriptiveKind::Ddx);
        } else if attr_is(attr, "triage") {
            self.extract_prescriptive(
                attr,
                item_kind,
                item_target.clone(),
                PrescriptiveKind::Triage,
            );
        } else if attr_is(attr, "culture") {
            self.extract_prescriptive(
                attr,
                item_kind,
                item_target.clone(),
                PrescriptiveKind::Culture,
            );
        } else if attr_is(attr, "quarantine") {
            self.extract_prescriptive(
                attr,
                item_kind,
                item_target.clone(),
                PrescriptiveKind::Quarantine,
            );
        }
    }

    /// Scan-extract a mucosal-boundary declaration (ADR-027 + Amendment 1).
    /// All three primitives share the loosely-typed `ScanMucosalArgs`
    /// capture; per-primitive required-field + delegate-kind-matching
    /// validation is the audit layer's job (Change 5 three-tier diagnosis).
    fn extract_mucosal(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
        tag: MucosalKindTag,
    ) {
        let line = Self::line_of_attr(attr);
        let args = match &attr.meta {
            syn::Meta::List(list) => match syn::parse2::<ScanMucosalArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed mucosal-boundary attribute: {e}"),
                    });
                    return;
                }
            },
            syn::Meta::Path(_) => ScanMucosalArgs::default(),
            syn::Meta::NameValue(_) => return,
        };
        self.report.mucosal_declarations.push(MucosalDeclaration {
            tag,
            boundary_kind: args.boundary_kind,
            rationale: args.rationale,
            handled_by: args.handled_by,
            accepts: args.accepts,
            reviewed_by: args.reviewed_by,
            until: args.until,
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
        });
    }

    /// Scan-extract a recurrent-emergence declaration (ADR-024 §Family 2).
    ///
    /// All six primitives share the loosely-typed `ScanRecurrentArgs` capture
    /// (mirroring `ScanAntigenArgs`'s forward-compat posture per ADR-009).
    /// The `kind` discriminant is supplied by the dispatch site; per-kind
    /// required-field validation is the audit layer's job, not scan's
    /// (scan is recall-tuned per ADR-010; precision lives in audit).
    fn extract_recurrent(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
        kind: RecurrentKind,
    ) {
        let line = Self::line_of_attr(attr);
        let args = match &attr.meta {
            syn::Meta::List(list) => match syn::parse2::<ScanRecurrentArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed recurrent-emergence attribute: {e}"),
                    });
                    return;
                }
            },
            // Bare `#[chronic]` etc. without args — recall it with empty
            // fields; audit surfaces the missing-required-field condition.
            syn::Meta::Path(_) => ScanRecurrentArgs::default(),
            syn::Meta::NameValue(_) => return,
        };
        self.report
            .recurrent_declarations
            .push(RecurrentDeclaration {
                kind,
                name: args.name,
                antigen_type: args.antigen_type,
                description: args.description,
                instances: args.instances,
                since: args.since,
                rationale: args.rationale,
                from_itches: args.from_itches,
                anchored_by: args.anchored_by,
                managed_by: args.managed_by,
                contributing_to: args.contributing_to,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
    }

    /// Scan-extract a prescriptive work-orchestration declaration (ADR-033). All
    /// eight primitives share the loosely-typed [`ScanPrescriptiveArgs`] capture
    /// (mapping per-shape field names onto the shared declaration slots);
    /// per-kind required-field validation is the macro's (parse-time) + audit's
    /// job. Scan is recall-tuned (ADR-010).
    fn extract_prescriptive(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
        kind: PrescriptiveKind,
    ) {
        let line = Self::line_of_attr(attr);
        let args = match &attr.meta {
            syn::Meta::List(list) => {
                match syn::parse2::<ScanPrescriptiveArgs>(list.tokens.clone()) {
                    Ok(a) => a,
                    Err(e) => {
                        self.report.parse_failures.push(ParseFailure {
                            file: self.file_path.clone(),
                            error: format!("malformed prescriptive attribute: {e}"),
                        });
                        return;
                    }
                }
            }
            // Bare `#[panel]` etc. without args — recall with empty fields; the
            // audit surfaces the missing-required-field condition.
            syn::Meta::Path(_) => ScanPrescriptiveArgs::default(),
            syn::Meta::NameValue(_) => return,
        };
        self.report
            .prescriptive_declarations
            .push(PrescriptiveDeclaration {
                kind,
                items: args.items,
                filled_by: args.filled_by,
                reviewed_by: args.reviewed_by,
                ordered_by: args.ordered_by,
                frame: args.frame,
                need_text: args.need_text,
                label: args.label,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
                // NFA-21: pin who-step satisfaction to the item's current
                // structural digest. `current_item_digest` is set by the
                // visit_item_* method immediately before `check_attrs`
                // dispatches here, so it reflects the annotated item's code.
                structural_fingerprint: self.current_item_digest.clone(),
            });
    }

    fn extract_diagnostic(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanDiagnosticArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[diagnostic] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.convergent_evidences.push(ConvergentEvidence {
                kind: ConvergentEvidenceKind::Diagnostic,
                modality_classes: args.modality_classes,
                min_independent: args.min_independent,
                witness: None,
                iterations: None,
                seed_kind: None,
                historical_span: None,
                min_reattestations: None,
                witnesses: Vec::new(),
                fingerprints: Vec::new(),
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_clonal(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanClonalArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[clonal] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.convergent_evidences.push(ConvergentEvidence {
                kind: ConvergentEvidenceKind::Clonal,
                modality_classes: Vec::new(),
                min_independent: None,
                witness: args.witness,
                iterations: args.iterations,
                seed_kind: args.seed_kind,
                historical_span: None,
                min_reattestations: None,
                witnesses: Vec::new(),
                fingerprints: Vec::new(),
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_igg(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanIggArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[igg] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.convergent_evidences.push(ConvergentEvidence {
                kind: ConvergentEvidenceKind::Igg,
                modality_classes: Vec::new(),
                min_independent: None,
                witness: None,
                iterations: None,
                seed_kind: None,
                historical_span: args.historical_span,
                min_reattestations: args.min_reattestations,
                witnesses: args.witnesses,
                fingerprints: Vec::new(),
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_crossreactive(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanCrossreactiveArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[crossreactive] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.convergent_evidences.push(ConvergentEvidence {
                kind: ConvergentEvidenceKind::Crossreactive,
                modality_classes: Vec::new(),
                min_independent: None,
                witness: None,
                iterations: None,
                seed_kind: None,
                historical_span: None,
                min_reattestations: None,
                witnesses: Vec::new(),
                fingerprints: args.fingerprints,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    /// Common extractor for the three marker primitives (no required
    /// args): `#[polyclonal]`, `#[monoclonal]`, `#[adcc]`. Records the
    /// site with `kind = <kind>` and all other fields default.
    fn extract_convergent_marker(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
        kind: ConvergentEvidenceKind,
    ) {
        let line = Self::line_of_attr(attr);
        self.report.convergent_evidences.push(ConvergentEvidence {
            kind,
            modality_classes: Vec::new(),
            min_independent: None,
            witness: None,
            iterations: None,
            seed_kind: None,
            historical_span: None,
            min_reattestations: None,
            witnesses: Vec::new(),
            fingerprints: Vec::new(),
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
        });
    }
}

/// Render a `syn::Type` to its canonical token-stream string. Used to
/// extract a string identifier for `impl Trait for Type` blocks. The
/// rendering normalizes whitespace via `quote::ToTokens`. For W3 we only
/// need a stable string for equality matching — A3 cross-crate work will
/// likely want a richer canonical form (e.g., resolved module paths).
pub fn render_type(ty: &syn::Type) -> String {
    use quote::ToTokens;
    ty.to_token_stream().to_string()
}

/// Whether an attribute's path matches a given antigen attribute name.
///
/// `syn::Path::is_ident("X")` only returns true for single-segment paths.
/// Path-qualified attribute forms — `#[antigen::immune(...)]`,
/// `#[crate::presents(...)]`, `#[my::module::antigen(...)]` — produce
/// multi-segment paths that `is_ident` rejects, causing the scan to
/// silently drop them. The fix: an attribute's path matches `name`
/// either when it's the bare ident, OR when its *last segment* is the
/// ident.
///
/// This is the path-segment-aware analog of `is_ident` and is the only
/// matcher used inside `ScanVisitor`. Using last-segment equality is
/// cheap and the same heuristic Rust itself uses to find the macro
/// being invoked — name resolution happens elsewhere.
pub fn attr_is(attr: &syn::Attribute, name: &str) -> bool {
    let path = attr.path();
    path.is_ident(name) || path.segments.last().is_some_and(|s| s.ident == name)
}

/// Extract the `antigen:requires:v1:<json>` predicate from a sibling doc attr.
///
/// The `#[immune(requires = ...)]` macro (P3b) emits:
///   `#[doc = " antigen:requires:v1:<json>"]`
/// as a sibling attribute on the annotated item. Scan finds it by looking
/// for a doc attribute whose string value starts with the marker prefix.
fn extract_requires_predicate_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
    const MARKER_PREFIX: &str = "antigen:requires:v1:";
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(nv) = &attr.meta {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
            {
                let val = s.value();
                let trimmed = val.trim();
                if let Some(json) = trimmed.strip_prefix(MARKER_PREFIX) {
                    return Some(json.to_string());
                }
            }
        }
    }
    None
}

/// Render a `syn::Path` similarly. Used for the trait portion of
/// `impl Trait for Type` so that `Drop` and `core::ops::Drop` produce
/// distinct strings (which is correct — they're different items in
/// Rust's name resolution, even when they alias).
pub fn render_path(path: &syn::Path) -> String {
    use quote::ToTokens;
    path.to_token_stream().to_string()
}

impl<'ast> Visit<'ast> for ScanVisitor<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        for attr in &item.attrs {
            if attr_is(attr, "antigen") {
                self.extract_antigen(item, attr);
            }
        }
        let target = ItemTarget::Struct(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "struct", &target);
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let trait_path = item.trait_.as_ref().map(|(_, path, _)| render_path(path));
        let target_type = render_type(&item.self_ty);
        let target = ItemTarget::Impl {
            trait_path: trait_path.clone(),
            target_type: target_type.clone(),
        };
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl", &target);
        // Push impl context so visit_impl_item_fn can build ImplFn targets.
        self.impl_stack.push((trait_path, target_type));
        syn::visit::visit_item_impl(self, item);
        self.impl_stack.pop();
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        // ATK-A2-TOPLEVEL-CONST: route a free-standing const's attrs through
        // check_attrs so `#[presents]` on a top-level/module const is not
        // silently ignored (same blind-spot class as enum variants + impl
        // consts).
        let target = ItemTarget::Const(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "const", &target);
        syn::visit::visit_item_const(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        // Same blind-spot class as visit_item_const: route a free-standing
        // `static`'s attrs through check_attrs so `#[presents]` on it is not
        // silently ignored. Closed preemptively (ADR-007) — the fixture
        // atk_a2_static_presents proves the need.
        let target = ItemTarget::Static(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "static", &target);
        syn::visit::visit_item_static(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let target = ItemTarget::Fn(item.sig.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "fn", &target);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let target = self.impl_stack.last().map_or_else(
            || ItemTarget::Fn(item.sig.ident.to_string()),
            |(trait_path, target_type)| ItemTarget::ImplFn {
                trait_path: trait_path.clone(),
                target_type: target_type.clone(),
                fn_name: item.sig.ident.to_string(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl_fn", &target);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        // ATK-A2-IMPL-CONST: route an associated const's attrs through
        // check_attrs so `#[presents]` on an impl-block const is not silently
        // ignored (the same blind-spot as enum variants). Falls back to a bare
        // Fn target if somehow visited outside an impl (shouldn't happen).
        let target = self.impl_stack.last().map_or_else(
            || ItemTarget::Fn(item.ident.to_string()),
            |(trait_path, target_type)| ItemTarget::ImplConst {
                trait_path: trait_path.clone(),
                target_type: target_type.clone(),
                const_name: item.ident.to_string(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl_const", &target);
        syn::visit::visit_impl_item_const(self, item);
    }

    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        // ATK-A2-IMPL-ITEM-TYPE: an impl-block associated type
        // (`type Foo = Bar;`) carries attrs too — `#[presents]` on it was
        // silently dropped (same blind-spot class as impl_item_const). Route it
        // through check_attrs. Target is the associated-type name (reusing
        // TypeAlias rather than minting a near-duplicate variant, mirroring how
        // visit_trait_item_const reuses ImplConst).
        let target = ItemTarget::TypeAlias(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl_type", &target);
        syn::visit::visit_impl_item_type(self, item);
    }

    fn visit_impl_item_macro(&mut self, item: &'ast syn::ImplItemMacro) {
        // ATK-A2-IMPL-ITEM-MACRO: route a macro invocation inside an impl block
        // through check_attrs so `#[presents]` on patterns like
        // `#[presents(X)] delegate!()` is not silently ignored.
        // Same blind-spot class as impl_item_fn/const/type — the attrs field
        // exists and is valid, but without this override it is never visited.
        let mac_name = item
            .mac
            .path
            .segments
            .last()
            .map_or_else(|| "(macro)".to_string(), |s| s.ident.to_string());
        let target = self.impl_stack.last().map_or_else(
            || ItemTarget::Fn(mac_name.clone()),
            |(trait_path, target_type)| ItemTarget::ImplConst {
                trait_path: trait_path.clone(),
                target_type: target_type.clone(),
                const_name: mac_name.clone(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl_macro", &target);
        syn::visit::visit_impl_item_macro(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let target = ItemTarget::Trait(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait", &target);
        // Push trait context so visit_trait_item_fn produces TraitFn targets
        // identifying the enclosing trait.
        self.trait_stack.push(item.ident.to_string());
        syn::visit::visit_item_trait(self, item);
        self.trait_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let target = self.trait_stack.last().map_or_else(
            || ItemTarget::Fn(item.sig.ident.to_string()),
            |trait_name| ItemTarget::TraitFn {
                trait_name: trait_name.clone(),
                fn_name: item.sig.ident.to_string(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait_fn", &target);
        syn::visit::visit_trait_item_fn(self, item);
    }

    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        // Same blind-spot class as the impl/top-level const cases: route a
        // trait-associated const's attrs through check_attrs. Reuses
        // ItemTarget::ImplConst with the trait as the target type (an
        // associated const on a named type/trait) to avoid a near-duplicate
        // variant — label renders as `Trait::CONST`.
        let target = self.trait_stack.last().map_or_else(
            || ItemTarget::Const(item.ident.to_string()),
            |trait_name| ItemTarget::ImplConst {
                trait_path: None,
                target_type: trait_name.clone(),
                const_name: item.ident.to_string(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait_const", &target);
        syn::visit::visit_trait_item_const(self, item);
    }

    fn visit_trait_item_type(&mut self, item: &'ast syn::TraitItemType) {
        // ATK-A2-TRAIT-ITEM-TYPE: a trait associated-type declaration
        // (`type Item;`) carries attrs too — `#[presents]` on it was silently
        // dropped (same blind-spot class as trait_item_const). These are real
        // contract sites (e.g. a mucosal boundary like `Iterator::Item`). Route
        // through check_attrs with the associated-type name as target.
        let target = ItemTarget::TypeAlias(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait_type", &target);
        syn::visit::visit_trait_item_type(self, item);
    }

    fn visit_trait_item_macro(&mut self, item: &'ast syn::TraitItemMacro) {
        // ATK-A2-TRAIT-ITEM-MACRO: route a macro invocation inside a trait body
        // through check_attrs so `#[presents]` on trait-body macro expansions
        // (blanket-impl helpers, proc-macro trait-body generators) is not silently
        // ignored. Same blind-spot class as trait_item_fn/const/type.
        let mac_name = item
            .mac
            .path
            .segments
            .last()
            .map_or_else(|| "(macro)".to_string(), |s| s.ident.to_string());
        let target = self.trait_stack.last().map_or_else(
            || ItemTarget::Fn(mac_name.clone()),
            |trait_name| ItemTarget::TraitFn {
                trait_name: trait_name.clone(),
                fn_name: mac_name.clone(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait_macro", &target);
        syn::visit::visit_trait_item_macro(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        // Type aliases (`type Foo = ...;`) carry attributes too. ATK-W3-005:
        // without this handler, attributes on type aliases would fall back
        // to ItemTarget::Unknown, and two unrelated Unknown items collide
        // on equality. Tracking the alias name keeps each alias as its own
        // distinct match target.
        let target = ItemTarget::TypeAlias(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "type_alias", &target);
        syn::visit::visit_item_type(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        for attr in &item.attrs {
            if attr_is(attr, "antigen") {
                // ATK-A2-007: silently dropping #[antigen] on enums eats the
                // class-enum pattern (the frame-translation antigen's primary
                // use case). Surface the situation as a parse_failure so the
                // user sees it, rather than the previous `let _ = attr` no-op.
                // The macro itself still rejects non-unit structs at compile
                // time; this scan-side diagnostic catches enum cases that
                // wouldn't reach the macro (e.g., a user investigating "why
                // doesn't my class enum scan as an antigen?").
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!(
                        "#[antigen] on enum `{}` is not supported in v0.1; \
                         antigen declarations must be unit structs (e.g., \
                         `pub struct {};`). Enum-shaped failure-classes are \
                         tracked by ADR-010 Amendment 1's class-enum operator \
                         in a future grammar version.",
                        item.ident, item.ident
                    ),
                });
            }
        }
        let target = ItemTarget::Enum(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "enum", &target);

        // ATK-A2-ENUM-VARIANT: descend into variants so a variant-level
        // attribute (e.g. `#[presents(X)]` on one variant) is not silently
        // ignored. `syn::visit::visit_item_enum` walks the variants but never
        // routes their attrs through `check_attrs`, so without this loop the
        // presentation is invisible to failure-class memory. The enclosing-enum
        // digest stands in for each variant (a variant has no independent
        // structural digest of its own).
        let enum_name = item.ident.to_string();
        for variant in &item.variants {
            let variant_target = ItemTarget::EnumVariant {
                enum_name: enum_name.clone(),
                variant_name: variant.ident.to_string(),
            };
            self.check_attrs(&variant.attrs, "enum_variant", &variant_target);
        }

        syn::visit::visit_item_enum(self, item);
    }

    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        // ATK-A2-MACRO-RULES: route a macro_rules! item's attrs through
        // check_attrs so #[presents] on a macro definition is not silently
        // ignored. Same blind-spot class as enum variants and impl consts.
        // ItemTarget::Const reuses an existing string-carrying target variant;
        // the name is the macro identifier or "(anonymous)" for unnamed macros.
        let name = item
            .ident
            .as_ref()
            .map_or_else(|| "(anonymous)".to_string(), ToString::to_string);
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "macro", &target);
        syn::visit::visit_item_macro(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        // ATK-A2-USE-ITEM: route a use/re-export item's attrs through check_attrs
        // so #[presents] on a use declaration (e.g. a dangerous capability re-export
        // at a trust boundary) is not silently ignored. Same blind-spot class as
        // macro_rules! (above), enum variants, and impl consts.
        use quote::ToTokens;
        let path_str = item.tree.to_token_stream().to_string();
        let target = ItemTarget::Const(path_str);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "use", &target);
        syn::visit::visit_item_use(self, item);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        let name = item.ident.to_string();
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "extern crate", &target);
        syn::visit::visit_item_extern_crate(self, item);
    }

    fn visit_item_foreign_mod(&mut self, item: &'ast syn::ItemForeignMod) {
        use quote::ToTokens;
        let abi_str = item.abi.to_token_stream().to_string();
        let target = ItemTarget::Const(abi_str);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "foreign mod", &target);
        syn::visit::visit_item_foreign_mod(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let name = item.ident.to_string();
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "mod", &target);
        syn::visit::visit_item_mod(self, item);
    }

    fn visit_item_trait_alias(&mut self, item: &'ast syn::ItemTraitAlias) {
        let name = item.ident.to_string();
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait alias", &target);
        syn::visit::visit_item_trait_alias(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        let name = item.ident.to_string();
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "union", &target);
        syn::visit::visit_item_union(self, item);
    }
}
