//! Fingerprint + generates synthesis pass.
//!
//! Extracted from the former monolithic `scan.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). `synthesis_pass` emits synthetic
//! `FingerprintMatch` presentations for items that match a declared antigen's
//! fingerprint but were not explicitly annotated (with declaration-site
//! self-match suppression); `GeneratesInvocationVisitor` + `generates_synthesis_pass`
//! are its ADR-014 generates-synthesis arm. A post-collection pass driven by
//! `finalize`; it consumes the `parse` leaf (the `ScanVisitor` + the syn-render
//! helpers) but holds no stop-authority (single-conductor invariant, ADR-036).
//!
//! API-invisible: crate-internal (the finalize pass drives it); `synthesis_pass`
//! is re-exported `pub(crate)` at the scan module root.

use std::path::PathBuf;

use syn::visit::Visit;

use super::{
    attr_is, render_path, render_type, ItemTarget, MatchKind, Presentation, ScanReport, ScanVisitor,
};

/// Emit synthetic `FingerprintMatch` presentations for items that match a
/// declared antigen fingerprint but weren't explicitly annotated.
///
/// Called from [`scan_workspace`](crate::scan::scan_workspace) after the explicit-collection walk. Uses the
/// cached `(path, syn::File)` pairs from pass 1 — no re-reading or re-parsing.
/// Only top-level items are checked (`syn::File::items`); descent into `impl`
/// methods and `trait` methods is deferred to W6b/A3.
///
/// `declaration_sites` is the set of `(type_name, file)` pairs identifying
/// antigen declaration structs themselves. These are suppressed from
/// fingerprint-match reports — a declaration's own struct always matches its
/// own `doc_contains` fingerprint, producing noise with no signal (DX finding 4).
pub fn synthesis_pass(
    parsed_files: &[(PathBuf, syn::File)],
    fingerprints: &[(String, antigen_fingerprint::Fingerprint)],
    declaration_sites: &std::collections::HashSet<(String, PathBuf)>,
    report: &mut ScanReport,
) {
    for (file_path, parsed) in parsed_files {
        for syn_item in &parsed.items {
            let Some((kind_str, item_target)) = item_kind_and_target(syn_item) else {
                continue;
            };

            // Node-kind dispatch: skip fingerprints whose top-level item
            // constraint can't match this item's kind — cheap O(1) filter
            // per ADR-010 Amendment 3 Performance Invariant 4.
            let item_kind_for_dispatch = match syn_item {
                syn::Item::Struct(_) => Some(antigen_fingerprint::ItemKind::Struct),
                syn::Item::Enum(_) => Some(antigen_fingerprint::ItemKind::Enum),
                syn::Item::Trait(_) => Some(antigen_fingerprint::ItemKind::Trait),
                syn::Item::Fn(_) => Some(antigen_fingerprint::ItemKind::Fn),
                syn::Item::Impl(_) => Some(antigen_fingerprint::ItemKind::Impl),
                syn::Item::Type(_) => Some(antigen_fingerprint::ItemKind::Type),
                syn::Item::Mod(_) => Some(antigen_fingerprint::ItemKind::Mod),
                syn::Item::Const(_) => Some(antigen_fingerprint::ItemKind::Const),
                syn::Item::Static(_) => Some(antigen_fingerprint::ItemKind::Static),
                syn::Item::Union(_) => Some(antigen_fingerprint::ItemKind::Union),
                _ => None,
            };

            for (antigen_type, fp) in fingerprints {
                // Node-kind dispatch: if the fingerprint pins a required kind,
                // skip evaluation when this item's kind doesn't match.
                if let Some(required_kind) = fp.node_kind() {
                    if item_kind_for_dispatch != Some(required_kind) {
                        continue;
                    }
                }

                if !fp.matches(syn_item) {
                    continue;
                }

                // Self-match suppression: skip when the item IS the antigen's
                // own declaration struct (DX finding 4). The struct that carries
                // #[antigen] always matches its own fingerprint; this match has
                // no signal. Only suppress the exact struct, not other items in
                // the same file that legitimately match the fingerprint.
                let is_self_decl = matches!(&item_target, ItemTarget::Struct(s) if s == antigen_type)
                    && declaration_sites.contains(&(antigen_type.clone(), file_path.clone()));
                if is_self_decl {
                    continue;
                }

                // Deduplication: skip if an explicit #[presents] already covers
                // this (antigen_type, file, item) triple, OR if a tolerance
                // acknowledges the match — tolerated sites belong in the
                // "tolerated" state, not "fingerprint match" (5-state matrix,
                // ADR-001 Amendment 1 Change 2).
                let already_covered = report.presentations.iter().any(|p| {
                    p.match_kind == MatchKind::ExplicitMarker
                        && p.antigen_type == *antigen_type
                        && p.file == *file_path
                        && p.item_target.addresses(&item_target)
                }) || report.tolerances.iter().any(|t| {
                    t.antigen_type == *antigen_type
                        && t.file == *file_path
                        && t.item_target.addresses(&item_target)
                });
                if already_covered {
                    continue;
                }

                // Duplicate-emission guard: skip if an *identical*
                // FingerprintMatch was already emitted for this exact
                // `(antigen_type, file, item_target)` triple.
                //
                // The same antigen *type name* can be declared more than once
                // across a workspace (e.g. the stdlib `ContentHashMismatch` plus
                // several test-fixture `ContentHashMismatch` declarations, each
                // with its own fingerprint). Each declaration contributes a
                // `(type_name, fp)` entry to the synthesis fingerprint set, so an
                // item that matches more than one would otherwise produce N
                // byte-identical `FingerprintMatch` presentations at the same site
                // — pure noise that inflated scan output by ~3300 records on this
                // workspace.
                //
                // Identity here is **exact `item_target` equality**, NOT the
                // broader `addresses()` relation: `addresses()` deliberately
                // treats distinct impl blocks for the same base type (different
                // trait_path) as one addressable site, but those are genuinely
                // distinct presentation sites for *reporting* — collapsing them
                // would silently drop real matches (the impl-granularity question
                // belongs to `addresses()`/ADR-017, not to a dedup heuristic). We
                // only suppress the truly-identical re-emission.
                let duplicate_emitted = report.presentations.iter().any(|p| {
                    p.match_kind == MatchKind::FingerprintMatch
                        && p.antigen_type == *antigen_type
                        && p.file == *file_path
                        && p.item_target == item_target
                });
                if duplicate_emitted {
                    continue;
                }

                // Compute line from the item's first attribute or item span.
                let line = item_line(syn_item);

                let structural_fingerprint = match syn_item {
                    syn::Item::Struct(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Enum(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Trait(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Fn(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Type(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Impl(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Const(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Static(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Union(i) => antigen_fingerprint::structural_digest(i),
                    _ => String::new(),
                };

                report.presentations.push(Presentation {
                    antigen_type: antigen_type.clone(),
                    file: file_path.clone(),
                    line,
                    item_kind: kind_str.to_string(),
                    item_target: item_target.clone(),
                    match_kind: MatchKind::FingerprintMatch,
                    canonical_path: None,
                    inherited_from: None,
                    structural_fingerprint,
                    // Fingerprint-inferred presentations carry no declared
                    // site-attached evidence — the developer wrote no #[presents]
                    // marker, so there is no requires=/proof= to fold (ADR-029).
                    requires_predicate: None,
                    proof: None,
                });
            }
        }
    }
}

/// A macro invocation site discovered by [`GeneratesInvocationVisitor`].
struct MacroInvocation {
    /// The macro identifier used at the call site (derive name, bang-macro
    /// name, or attribute-macro name).
    macro_name: String,
    /// Source line of the invocation.
    line: usize,
    /// Identity of the item the invocation is attached to (the `#[derive]`'d
    /// item, for audit cross-reference) — `Unknown { line }` for bang-macro
    /// calls that aren't attached to a nameable item.
    item_target: ItemTarget,
}

/// Walk a parsed file and collect every macro invocation that names a known
/// generator: `#[derive(Name)]` attributes, attribute-macro `#[name]`
/// invocations, and bang-macro `name!(...)` calls. Only invocations whose name
/// is in `generators` are recorded (cheap filter; the workspace has few
/// generators).
struct GeneratesInvocationVisitor<'a> {
    generators: &'a std::collections::HashSet<String>,
    found: Vec<MacroInvocation>,
}

impl GeneratesInvocationVisitor<'_> {
    /// Record `#[derive(A, B, ...)]` + attribute-macro `#[name]` invocations
    /// carried by an item's attribute list, attributing them to `target`.
    fn scan_attrs(&mut self, attrs: &[syn::Attribute], target: &ItemTarget) {
        for attr in attrs {
            if attr_is(attr, "derive") {
                // `#[derive(A, B, C)]`: each path segment's last ident is a
                // derive-macro name.
                if let syn::Meta::List(list) = &attr.meta {
                    let parsed = list.parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    );
                    if let Ok(paths) = parsed {
                        for p in paths {
                            if let Some(seg) = p.segments.last() {
                                let name = seg.ident.to_string();
                                if self.generators.contains(&name) {
                                    self.found.push(MacroInvocation {
                                        macro_name: name,
                                        line: ScanVisitor::line_of_attr(attr),
                                        item_target: target.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
                continue;
            }
            // Attribute-macro invocation `#[name(...)]` / `#[name]` whose name is
            // a known generator (not derive, not a built-in antigen marker).
            if let Some(seg) = attr.path().segments.last() {
                let name = seg.ident.to_string();
                if self.generators.contains(&name) {
                    self.found.push(MacroInvocation {
                        macro_name: name,
                        line: ScanVisitor::line_of_attr(attr),
                        item_target: target.clone(),
                    });
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for GeneratesInvocationVisitor<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        // Attribute-bearing items: scan their attrs for derive/attribute-macro
        // invocations, attributed to the item's identity.
        if let Some((_, target)) = item_kind_and_target(item) {
            let attrs: &[syn::Attribute] = match item {
                syn::Item::Struct(i) => &i.attrs,
                syn::Item::Enum(i) => &i.attrs,
                syn::Item::Union(i) => &i.attrs,
                syn::Item::Fn(i) => &i.attrs,
                syn::Item::Trait(i) => &i.attrs,
                syn::Item::Type(i) => &i.attrs,
                syn::Item::Const(i) => &i.attrs,
                syn::Item::Static(i) => &i.attrs,
                syn::Item::Impl(i) => &i.attrs,
                syn::Item::Mod(i) => &i.attrs,
                _ => &[],
            };
            self.scan_attrs(attrs, &target);
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        // Bang-macro invocation `name!(...)`: the last path segment is the name.
        if let Some(seg) = mac.path.segments.last() {
            let name = seg.ident.to_string();
            if self.generators.contains(&name) {
                let line = seg.ident.span().start().line;
                self.found.push(MacroInvocation {
                    macro_name: name,
                    line,
                    item_target: ItemTarget::Unknown { line },
                });
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

/// Generates-synthesis pass (ADR-014 §Mechanics step 2): for every macro
/// INVOCATION whose name matches an `#[antigen_generates(X, ...)]` declaration
/// on a macro DEFINITION, emit a synthetic `Presentation` at the invocation
/// site presenting `X`.
///
/// Same-workspace only (§A3): the generator declarations and the invocations
/// are both discovered by walking this workspace. Cross-crate macro-output
/// recognition (§A4 — a `#[derive(SerdeFoo)]` invocation here matching a
/// generator declared in the `serde_foo` dep) requires the cross-crate
/// antigen-discovery mechanism and is deferred.
///
/// The synthetic presentation is `ExplicitMarker` (the macro author explicitly
/// declared the generation — it is not a heuristic fingerprint guess) with
/// `item_kind = "generated_<macro>"`. It is attributed to the INVOCATION item's
/// identity so a co-located `#[defended_by(X)]` / `#[antigen_tolerance(X)]`
/// addresses it (ADR-014 §Audit integration). Deduped against an existing
/// explicit/generated presentation for the same `(antigen_type, file,
/// item_target)`.
pub fn generates_synthesis_pass(parsed_files: &[(PathBuf, syn::File)], report: &mut ScanReport) {
    use std::collections::{HashMap, HashSet};

    // Index: macro_name -> set of (antigen_type) it generates. A macro can
    // carry multiple `#[antigen_generates]` declarations (ADR-014 allows
    // stacking), and two macros could share a name across crates (degenerate
    // intra-workspace) — union the antigen types per name.
    let mut by_macro: HashMap<String, Vec<String>> = HashMap::new();
    for g in &report.generates_declarations {
        by_macro
            .entry(g.macro_name.clone())
            .or_default()
            .push(g.antigen_type.clone());
    }
    let generator_names: HashSet<String> = by_macro.keys().cloned().collect();
    if generator_names.is_empty() {
        return;
    }

    for (file_path, parsed) in parsed_files {
        let mut visitor = GeneratesInvocationVisitor {
            generators: &generator_names,
            found: Vec::new(),
        };
        visitor.visit_file(parsed);

        for inv in visitor.found {
            let Some(antigen_types) = by_macro.get(&inv.macro_name) else {
                continue;
            };
            let item_kind = format!("generated_{}", inv.macro_name);
            for antigen_type in antigen_types {
                // Dedup: skip if an explicit #[presents] / a prior generated
                // presentation already covers this (antigen_type, file, item).
                let already = report.presentations.iter().any(|p| {
                    p.antigen_type == *antigen_type
                        && p.file == *file_path
                        && p.item_target == inv.item_target
                });
                if already {
                    continue;
                }
                report.presentations.push(Presentation {
                    antigen_type: antigen_type.clone(),
                    file: file_path.clone(),
                    line: inv.line,
                    item_kind: item_kind.clone(),
                    item_target: inv.item_target.clone(),
                    // Author-declared generation, not a heuristic fingerprint
                    // guess — treat as an explicit marker for matching (ADR-014
                    // §Mechanics: "Treated as #[presents] for matching").
                    match_kind: MatchKind::ExplicitMarker,
                    canonical_path: None,
                    inherited_from: None,
                    structural_fingerprint: String::new(),
                    requires_predicate: None,
                    proof: None,
                });
            }
        }
    }
}

/// Build a `(kind_str, ItemTarget)` pair from a top-level `syn::Item`.
/// Returns `None` for item kinds we don't model (macros, extern crates, etc.).
fn item_kind_and_target(item: &syn::Item) -> Option<(&'static str, ItemTarget)> {
    match item {
        syn::Item::Struct(s) => Some(("struct", ItemTarget::Struct(s.ident.to_string()))),
        syn::Item::Enum(e) => Some(("enum", ItemTarget::Enum(e.ident.to_string()))),
        syn::Item::Trait(t) => Some(("trait", ItemTarget::Trait(t.ident.to_string()))),
        syn::Item::Fn(f) => Some(("fn", ItemTarget::Fn(f.sig.ident.to_string()))),
        syn::Item::Type(t) => Some(("type", ItemTarget::TypeAlias(t.ident.to_string()))),
        syn::Item::Impl(i) => {
            let trait_path = i.trait_.as_ref().map(|(_, path, _)| render_path(path));
            let target_type = render_type(&i.self_ty);
            Some((
                "impl",
                ItemTarget::Impl {
                    trait_path,
                    target_type,
                },
            ))
        }
        syn::Item::Const(c) => Some(("const", ItemTarget::Const(c.ident.to_string()))),
        syn::Item::Static(s) => Some(("static", ItemTarget::Static(s.ident.to_string()))),
        syn::Item::Union(u) => Some(("union", ItemTarget::Union(u.ident.to_string()))),
        // `mod` items and other unmodeled kinds are skipped for synthesis.
        _ => None,
    }
}

/// Best-effort line number for a top-level `syn::Item` (line of its first
/// attribute if any, else the item's own span start).
fn item_line(item: &syn::Item) -> usize {
    use syn::spanned::Spanned;
    item.span().start().line
}
