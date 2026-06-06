//! Built-in syn evaluator for [`crate::Fingerprint`] (per ADR-015 S2 + S4).
//!
//! Matches a parsed fingerprint against a [`syn::Item`]. Item-shape
//! operators evaluate against `syn`'s typed AST directly. The
//! `body_contains_macro` operator walks the function/method body for
//! `syn::Macro` invocations natively (per ADR-015 S2).

use crate::{
    Constraint, Fingerprint, ItemKind, MethodPattern, QualifierKind, normalize_signature_canonical,
};

/// Three-valued predicate-evaluation result (ADR-010 Amendment 6).
///
/// Replaces the two-valued `bool` that conflated three distinct meanings.
/// `Undefined` is the load-bearing addition: it means "this predicate has no
/// locus on this item-class" (e.g. `body_contains_macro` on a struct — no
/// function body to search), as distinct from `NoMatch` ("searched, condition
/// absent"). Keeping these apart at the type level kills the vacuous-`not`
/// hazard: `not(Undefined) = Undefined`, NOT `Match` — so
/// `all_of([item = struct, not(body_contains_macro("panic"))])` evaluates to
/// `Undefined` (doesn't fire) rather than vacuously matching every struct.
///
/// Biology cognate (PMID 11238607): an assay run where its preconditions don't
/// hold (wrong tissue / window-period) is *indeterminate*, not negative — and
/// you cannot negate an indeterminate into a definite. `not(Undefined) =
/// Undefined` is the DSL form of that clinical invariant.
///
/// `pub` but crate-private in effect: the enclosing `matcher` module is private
/// (`mod matcher;`), so this does not leak to the public API. The public
/// `Fingerprint::matches` surface stays `bool` via the Level-2 projection;
/// Level-1 results stay internal until the v0.3 advisory tooling
/// ("fingerprint X was undefined on N items — domain mismatch?") needs to read
/// intermediate `Undefined`s, at which point `matcher` (and this) can be exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match3 {
    /// Predicate evaluated; condition present.
    Match,
    /// Predicate evaluated; condition absent.
    NoMatch,
    /// Predicate has no locus on this item-class — the question is malformed
    /// here (e.g. a body-content predicate on a bodyless item).
    Undefined,
}

impl Match3 {
    /// Lift a definite `bool` into `Match3`. For predicates that always have a
    /// locus on every item (item-kind, name, docs, attrs): the question is
    /// always well-posed, so the result is `Match`/`NoMatch`, never `Undefined`.
    const fn from_bool(b: bool) -> Self {
        if b { Self::Match } else { Self::NoMatch }
    }

    /// Kleene-strong negation. `Undefined` is closed under negation — it does
    /// NOT flip to a definite value (the vacuous-`not` defect, ADR-010 Amd6).
    const fn not(self) -> Self {
        match self {
            Self::Match => Self::NoMatch,
            Self::NoMatch => Self::Match,
            Self::Undefined => Self::Undefined,
        }
    }
}

impl Fingerprint {
    /// Match this fingerprint against a `syn::Item`.
    ///
    /// Returns `true` if every top-level constraint matches. Per ADR-010
    /// Amendment 4, fingerprints are RECALL-tuned filters: a `true` result
    /// is "this site may exhibit the failure-class," not "definitely does."
    /// The witness layer proves precision per ADR-002.
    ///
    /// Level-2 projection (ADR-010 Amd6): the fingerprint fires IFF the
    /// top-level expression evaluates to `Match3::Match`. Both `NoMatch` and
    /// `Undefined` project to "doesn't fire" — an item where the predicate has
    /// no locus is not flagged. Multiple top-level constraints compose under
    /// the same Kleene-strong `all_of` algebra as an explicit `all_of`.
    #[must_use]
    pub fn matches(&self, item: &syn::Item) -> bool {
        match_all_of(&self.constraints, item) == Match3::Match
    }
}

/// Kleene-strong conjunction over a constraint slice (the `all_of` algebra).
///
/// Any definite `NoMatch` short-circuits to `NoMatch`; otherwise all-`Match`
/// is `Match`; otherwise (a mix with at least one `Undefined` and no `NoMatch`)
/// the definedness gap propagates as `Undefined`. An empty slice is vacuously
/// `Match` (parse-time rules forbid empty `all_of`).
fn match_all_of(children: &[Constraint], item: &syn::Item) -> Match3 {
    let mut saw_undefined = false;
    for c in children {
        match match_constraint(c, item) {
            Match3::NoMatch => return Match3::NoMatch,
            Match3::Undefined => saw_undefined = true,
            Match3::Match => {},
        }
    }
    if saw_undefined {
        Match3::Undefined
    } else {
        Match3::Match
    }
}

/// Kleene-strong disjunction over a constraint slice (the `any_of` algebra).
///
/// Any definite `Match` short-circuits to `Match`; otherwise all-`NoMatch` is
/// `NoMatch`; otherwise the definedness gap propagates as `Undefined`.
fn match_any_of(children: &[Constraint], item: &syn::Item) -> Match3 {
    let mut saw_undefined = false;
    for c in children {
        match match_constraint(c, item) {
            Match3::Match => return Match3::Match,
            Match3::Undefined => saw_undefined = true,
            Match3::NoMatch => {},
        }
    }
    if saw_undefined {
        Match3::Undefined
    } else {
        Match3::NoMatch
    }
}

fn match_constraint(c: &Constraint, item: &syn::Item) -> Match3 {
    match c {
        // Leaf predicates with a locus on EVERY item-class are always
        // well-posed → definite Match/NoMatch, never Undefined.
        Constraint::Item(kind) => Match3::from_bool(item_kind_matches(item, *kind)),
        Constraint::NameMatches(glob) => {
            Match3::from_bool(item_name(item).is_some_and(|name| glob.matches(&name)))
        },
        Constraint::Variants(range) => Match3::from_bool(match item {
            syn::Item::Enum(e) => range.contains(e.variants.len()),
            _ => false,
        }),
        Constraint::HasMethod(pattern) => Match3::from_bool(has_matching_method(item, pattern)),
        Constraint::AttrPresent(path) => {
            Match3::from_bool(item_attrs(item).iter().any(|a| attr_path_matches(a, path)))
        },
        Constraint::DocContains(needle) => {
            Match3::from_bool(doc_text(item).contains(needle.as_str()))
        },
        // body_contains_macro is the one v0.2 leaf with a partial domain: it
        // returns Undefined on bodyless item-classes (ADR-010 Amd6).
        Constraint::BodyContainsMacro(name) => body_contains_macro(item, name),
        // body_calls is its call-shaped twin (ADR-040): same partial domain
        // (Undefined on bodyless item-classes), matching free/path calls by
        // last segment and method calls by method ident.
        Constraint::BodyCalls(name) => body_calls(item, name),
        // is_async/is_unsafe/is_const (ADR-040 G1): same partial-domain shape —
        // Match/NoMatch on item-classes that CAN carry the qualifier, Undefined
        // where there is no locus for it (so not(is_async) on a struct does not
        // vacuously match — ADR-010 Amd6).
        Constraint::Qualifier(kind) => qualifier_match(item, *kind),
        // impl_of_trait (ADR-040 G3): on an impl, the trait-path last segment is
        // the answer (inherent impl → NoMatch); on a non-impl → Undefined.
        Constraint::ImplOfTrait(name) => impl_of_trait(item, name),
        // derives/serde_arg (ADR-040 G1b): full-domain attr introspection, like
        // attr_present — a definite Match/NoMatch on every item (absent = NoMatch).
        Constraint::Derives(name) => Match3::from_bool(item_derives(item, name)),
        Constraint::SerdeArg(name) => Match3::from_bool(item_has_serde_arg(item, name)),
        Constraint::AllOf(children) => match_all_of(children, item),
        Constraint::AnyOf(children) => match_any_of(children, item),
        Constraint::Not(child) => match_constraint(child, item).not(),
    }
}

const fn item_kind_matches(item: &syn::Item, kind: ItemKind) -> bool {
    matches!(
        (item, kind),
        (syn::Item::Struct(_), ItemKind::Struct)
            | (syn::Item::Enum(_), ItemKind::Enum)
            | (syn::Item::Trait(_), ItemKind::Trait)
            | (syn::Item::Fn(_), ItemKind::Fn)
            | (syn::Item::Impl(_), ItemKind::Impl)
            | (syn::Item::Type(_), ItemKind::Type)
            | (syn::Item::Mod(_), ItemKind::Mod)
            | (syn::Item::Const(_), ItemKind::Const)
            | (syn::Item::Static(_), ItemKind::Static)
            | (syn::Item::Union(_), ItemKind::Union)
    )
}

fn item_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Struct(s) => Some(s.ident.to_string()),
        syn::Item::Enum(e) => Some(e.ident.to_string()),
        syn::Item::Trait(t) => Some(t.ident.to_string()),
        syn::Item::Fn(f) => Some(f.sig.ident.to_string()),
        syn::Item::Type(t) => Some(t.ident.to_string()),
        syn::Item::Mod(m) => Some(m.ident.to_string()),
        syn::Item::Const(c) => Some(c.ident.to_string()),
        syn::Item::Static(s) => Some(s.ident.to_string()),
        syn::Item::Union(u) => Some(u.ident.to_string()),
        // `impl` blocks have no name; for `impl T for U` we use U's ident if
        // it's a single-segment path. Most users will combine
        // `item = impl` with other constraints rather than `name = matches`.
        syn::Item::Impl(i) => match &*i.self_ty {
            syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Struct(s) => &s.attrs,
        syn::Item::Enum(e) => &e.attrs,
        syn::Item::Trait(t) => &t.attrs,
        syn::Item::Fn(f) => &f.attrs,
        syn::Item::Impl(i) => &i.attrs,
        syn::Item::Type(t) => &t.attrs,
        syn::Item::Mod(m) => &m.attrs,
        syn::Item::Const(c) => &c.attrs,
        syn::Item::Static(s) => &s.attrs,
        syn::Item::Union(u) => &u.attrs,
        _ => &[],
    }
}

/// Match an attribute path against the user-supplied `path` string.
///
/// Two acceptances:
/// 1. The attribute's full path (rendered with `::` separators) equals the
///    needle (e.g. `clippy::panic` matches `#[clippy::panic]`).
/// 2. The attribute's last segment equals the needle (e.g. `repr` matches
///    `#[repr(u8)]`).
fn attr_path_matches(attr: &syn::Attribute, needle: &str) -> bool {
    let path = &attr.path();
    // Last-segment shortcut.
    if let Some(last) = path.segments.last() {
        if last.ident == needle {
            return true;
        }
    }
    // Full-path render.
    let full: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    let rendered = full.join("::");
    rendered == needle
}

/// Doc-comment text for an item: concatenation of the values of every
/// `#[doc = "..."]` attribute on the item.
fn doc_text(item: &syn::Item) -> String {
    let mut out = String::new();
    for a in item_attrs(item) {
        if !a.path().is_ident("doc") {
            continue;
        }
        // `#[doc = "..."]` — name-value form.
        if let syn::Meta::NameValue(nv) = &a.meta {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
            {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&s.value());
            }
        }
    }
    out
}

/// Whether any method in an `impl` block has the given name AND a signature
/// whose normalized text matches the pattern's pre-normalized form.
fn has_matching_method(item: &syn::Item, pattern: &MethodPattern) -> bool {
    let syn::Item::Impl(imp) = item else {
        return false;
    };
    // PI-2: read the pattern's pre-normalized form computed at parse time.
    // Fallback for serde-deserialized patterns (where the cache is None):
    // canonicalize fires on every call — O(N items) per pattern on the serde
    // path vs O(1) for the parsed path. Acceptable in v0.1 because the scan
    // CLI always parses fingerprints from source; serde deserialization is not
    // the hot path. Fix when it bites: call ensure_normalized() on Fingerprint
    // before scanning, or change to &mut MethodPattern to write back on first use.
    //
    // The fallback MUST use the same canonicalization the parser uses
    // (`normalize_signature_canonical`) — a plain `normalize_ws` here would
    // re-introduce the `&self` / `& self` spacing bug for serde-loaded
    // patterns. A3.5 onboarding sweep fix.
    //
    // Amendment 5 OQ1 STRICT: when the lazy canonicalization fails
    // (proc_macro2 can't tokenize the serde-loaded `signature` field),
    // return `false` — no match. A malformed pattern cannot be brought
    // to a canonical form, so by construction it cannot match any real
    // signature. The matcher has no error channel; "never matches" is
    // the structurally honest answer at this layer. The fingerprint
    // parser (which DOES have an error channel) is the correct place
    // for the user-visible diagnostic.
    let pattern_norm = match pattern.normalized_signature.clone() {
        Some(s) => s,
        None => match normalize_signature_canonical(&pattern.signature) {
            Some(s) => s,
            None => return false,
        },
    };
    for impl_item in &imp.items {
        if let syn::ImplItem::Fn(f) = impl_item {
            if f.sig.ident == pattern.name && signature_matches(&f.sig, &pattern_norm) {
                return true;
            }
        }
    }
    false
}

/// Compare a method signature's input/output shape against the pattern's
/// pre-normalized form.
///
/// The pattern arrives canonicalized by `normalize_signature_canonical`
/// at fingerprint-load time (ADR-010 Amendment 3 Performance Invariant 2).
/// The actual `syn::Signature` is rendered fresh per call (it's the
/// per-match-site cost we cannot avoid) and routed through the SAME
/// canonicalization so the comparison is symmetric.
///
/// The symmetric canonicalization matters because the rendered output
/// mixes `proc_macro2`-tokenized parts (`& self`, `& mut self`) with
/// manually-joined separators (`", "`). The pattern goes through
/// `proc_macro2` wholesale at parse time. Without routing the actual output
/// through the same canonicalization, a `(Self, Self)` pattern
/// (`proc_macro2` renders as
/// `(Self , Self)`) would never match the matcher's `(Self, Self)` (manual
/// join). A3.5 onboarding sweep fix.
fn signature_matches(sig: &syn::Signature, pattern_norm: &str) -> bool {
    use quote::ToTokens;

    let inputs_rendered = render_inputs(sig);
    let output_rendered = match &sig.output {
        syn::ReturnType::Default => String::new(),
        syn::ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
    };
    let actual = if output_rendered.is_empty() {
        format!("({inputs_rendered})")
    } else {
        format!("({inputs_rendered}) -> {output_rendered}")
    };
    // The `actual` string was just built from `to_token_stream()` output —
    // by construction it's valid Rust syntax, so canonicalization should
    // never return None here. The match arm preserves "no match" semantics
    // defensively (if some future syn shape produces output that doesn't
    // round-trip, we get a false negative rather than a panic).
    normalize_signature_canonical(&actual).as_deref() == Some(pattern_norm)
}

fn render_inputs(sig: &syn::Signature) -> String {
    use quote::ToTokens;
    let parts: Vec<String> = sig
        .inputs
        .iter()
        .map(|input| match input {
            // `self`, `&self`, `&mut self` → render token-wise (yields "self", "& self", "& mut self").
            syn::FnArg::Receiver(r) => r.to_token_stream().to_string(),
            // Typed args: render only the type, NOT the pattern. The user
            // signature is shape-only ("(Self, Self) -> Self"), not
            // parameter-name-aware.
            syn::FnArg::Typed(pt) => pt.ty.to_token_stream().to_string(),
        })
        .collect();
    parts.join(", ")
}

/// Walk the function/method body for a macro invocation whose path's last
/// segment equals `name`.
fn body_contains_macro(item: &syn::Item, name: &str) -> Match3 {
    use syn::visit::Visit;

    struct MacroFinder<'a> {
        needle: &'a str,
        found: bool,
    }

    impl<'ast> Visit<'ast> for MacroFinder<'_> {
        fn visit_macro(&mut self, mac: &'ast syn::Macro) {
            if !self.found {
                if let Some(last) = mac.path.segments.last() {
                    if last.ident == self.needle {
                        self.found = true;
                        return;
                    }
                }
                syn::visit::visit_macro(self, mac);
            }
        }
    }

    let mut finder = MacroFinder {
        needle: name,
        found: false,
    };
    match item {
        // Has a function body to search → definite Match/NoMatch.
        syn::Item::Fn(f) => {
            finder.visit_block(&f.block);
            Match3::from_bool(finder.found)
        },
        syn::Item::Impl(imp) => {
            for impl_item in &imp.items {
                if let syn::ImplItem::Fn(f) = impl_item {
                    finder.visit_block(&f.block);
                    if finder.found {
                        break;
                    }
                }
            }
            Match3::from_bool(finder.found)
        },
        // No function body on this item-class — the question "does the body
        // contain macro X" has no locus here. UNDEFINED, not vacuous-false
        // (ADR-010 Amd6): this is what kills the vacuous-`not` hazard.
        _ => Match3::Undefined,
    }
}

/// Walk the function/method body for a *call* to a function or method whose name
/// matches `name`. The call-shaped twin of [`body_contains_macro`] (ADR-040).
///
/// Two call shapes are matched, mirroring how Rust spells calls:
/// - **free / path calls** (`syn::Expr::Call`): `foo()`, `std::process::exit(1)`
///   — matched on the *last path segment* of the callee path (so a qualified
///   `std::process::exit` matches `body_calls("exit")`), exactly the
///   last-segment discipline `body_contains_macro` uses for macro paths.
/// - **method calls** (`syn::Expr::MethodCall`): `x.unwrap()`, `r.expect(..)`
///   — matched on the *method identifier*.
///
/// Same partial domain as the macro twin: a definite `Match`/`NoMatch` for
/// item-classes that have a function body (`fn`, `impl` methods), and
/// `Match3::Undefined` for item-classes with no body locus — so
/// `not(body_calls(X))` inside `all_of` stays sound (ADR-010 Amd6, the
/// vacuous-`not` guard). Last-segment / method-ident matching means an *aliased*
/// call (`use std::process::exit as quit; quit()`) is NOT detected — the same
/// honest limitation `body_contains_macro` carries for aliased macros.
fn body_calls(item: &syn::Item, name: &str) -> Match3 {
    use syn::visit::Visit;

    struct CallFinder<'a> {
        needle: &'a str,
        found: bool,
    }

    impl<'ast> Visit<'ast> for CallFinder<'_> {
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            if self.found {
                return;
            }
            // The callee of a free/path call is itself an expression; a plain
            // path call (`foo()`, `a::b::c()`) is `Expr::Path`. Match on its
            // last path segment (the function name), mirroring the macro twin.
            if let syn::Expr::Path(p) = call.func.as_ref() {
                if let Some(last) = p.path.segments.last() {
                    if last.ident == self.needle {
                        self.found = true;
                        return;
                    }
                }
            }
            // Recurse into the callee + args so nested calls are still seen.
            syn::visit::visit_expr_call(self, call);
        }

        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            if self.found {
                return;
            }
            if call.method == self.needle {
                self.found = true;
                return;
            }
            // Recurse into the receiver + args (e.g. `a.foo().bar()` — both).
            syn::visit::visit_expr_method_call(self, call);
        }
    }

    let mut finder = CallFinder {
        needle: name,
        found: false,
    };
    match item {
        syn::Item::Fn(f) => {
            finder.visit_block(&f.block);
            Match3::from_bool(finder.found)
        },
        syn::Item::Impl(imp) => {
            for impl_item in &imp.items {
                if let syn::ImplItem::Fn(f) = impl_item {
                    finder.visit_block(&f.block);
                    if finder.found {
                        break;
                    }
                }
            }
            Match3::from_bool(finder.found)
        },
        // No function body on this item-class — "does the body call X" has no
        // locus here. UNDEFINED, not vacuous-false (ADR-010 Amd6).
        _ => Match3::Undefined,
    }
}

/// Evaluate an item-qualifier leaf (`is_async` / `is_unsafe` / `is_const`,
/// ADR-040 G1) over `item`.
///
/// Partial domain (ADR-010 Amd6): the qualifier question is **well-posed**
/// (`Match`/`NoMatch`) only on the item-classes that *can* carry it — and
/// `Undefined` everywhere else, so `not(is_async)` does NOT vacuously match an
/// item-class with no asyncness locus (e.g. a `struct`). The loci:
/// - `Async` / `Const` → `fn` only (read `Signature.asyncness` / `.constness`).
/// - `Unsafe` → `fn` (an `unsafe fn`, `Signature.unsafety`), `impl` (an
///   `unsafe impl`, `ItemImpl.unsafety`), AND `trait` (an `unsafe trait`,
///   `ItemTrait.unsafety`) — the three places `unsafe` can sit on an item.
const fn qualifier_match(item: &syn::Item, kind: QualifierKind) -> Match3 {
    match (kind, item) {
        // is_async — fn locus only.
        (QualifierKind::Async, syn::Item::Fn(f)) => Match3::from_bool(f.sig.asyncness.is_some()),
        // is_const — fn locus only. (The `const` *qualifier* on a function, NOT
        // the `item = const` item-kind.)
        (QualifierKind::Const, syn::Item::Fn(f)) => Match3::from_bool(f.sig.constness.is_some()),
        // is_unsafe — fn OR impl OR trait locus. A `trait` HAS the unsafe locus
        // (`unsafe trait Foo {}`, `ItemTrait.unsafety`), so it must be a definite
        // Match/NoMatch — NOT `Undefined`. Omitting this arm was a FALSE-Undefined
        // (the item has the locus but the arm didn't enumerate it — the ⊥-collapse
        // class wearing the partial-domain invariant's clothing; ADR-010 Amd6
        // contrapositive: "don't claim Undefined where the question is well-posed").
        (QualifierKind::Unsafe, syn::Item::Fn(f)) => Match3::from_bool(f.sig.unsafety.is_some()),
        (QualifierKind::Unsafe, syn::Item::Impl(imp)) => Match3::from_bool(imp.unsafety.is_some()),
        (QualifierKind::Unsafe, syn::Item::Trait(t)) => Match3::from_bool(t.unsafety.is_some()),
        // No locus for this qualifier on this item-class — the question has no
        // answer here. UNDEFINED, not vacuous-false (ADR-010 Amd6).
        _ => Match3::Undefined,
    }
}

/// Evaluate `impl_of_trait("<name>")` (ADR-040 G3) over `item`.
///
/// On an `impl` item the question "is this an impl of trait `name`?" is
/// well-posed: `Match` iff the impl has a trait path whose LAST segment equals
/// `name` (so `impl_of_trait("Drop")` fires on `impl Drop for V`, the canonical
/// "is this ACTUALLY a Drop impl, not just a method named drop" check); an
/// inherent `impl V {}` (no trait) is a definite `NoMatch`. On any non-`impl`
/// item-class there is no trait-impl locus → `Undefined` (so
/// `not(impl_of_trait(X))` stays sound, ADR-010 Amd6).
///
/// Last-segment matching is the same syntactic discipline the call/macro leaves
/// use — `impl std::ops::Drop for V` matches `impl_of_trait("Drop")`. This reads
/// ONE impl item's own trait path; the cross-item question "does `V` impl Drop
/// *anywhere* in the program" is a different (G4 / charter) concern.
fn impl_of_trait(item: &syn::Item, name: &str) -> Match3 {
    match item {
        syn::Item::Impl(imp) => {
            let fires = imp.trait_.as_ref().is_some_and(|(_, path, _)| {
                path.segments.last().is_some_and(|seg| seg.ident == name)
            });
            Match3::from_bool(fires)
        },
        // No trait-impl locus on this item-class. UNDEFINED, not vacuous-false.
        _ => Match3::Undefined,
    }
}

/// `derives("<name>")` (ADR-040 G1b): does any `#[derive(...)]` on the item list
/// a path whose LAST segment equals `name`? Syntactic (no path resolution), per
/// the derive/path-collision honesty note — a user type also named `Hash` is
/// indistinguishable here, and the dial carries that as the honest false-positive.
fn item_derives(item: &syn::Item, name: &str) -> bool {
    item_attrs(item).iter().any(|attr| {
        if !attr_path_matches(attr, "derive") {
            return false;
        }
        let mut found = false;
        // `#[derive(A, B::C)]` — walk the comma-separated derive paths and match
        // any one's last segment. `parse_nested_meta` errors on a malformed
        // derive list; treat that as "no match" (the attr is broken, not ours).
        let _ = attr.parse_nested_meta(|meta| {
            if meta
                .path
                .segments
                .last()
                .is_some_and(|seg| seg.ident == name)
            {
                found = true;
            }
            Ok(())
        });
        found
    })
}

/// `serde_arg("<name>")` (ADR-040 G1b): does any `#[serde(...)]` on the item carry
/// an argument whose path's LAST segment equals `name`? Matches presence
/// regardless of any `= value` (so `serde_arg("rename_all")` fires on
/// `#[serde(rename_all = "camelCase")]`, and `serde_arg("deny_unknown_fields")`
/// on the bare-flag `#[serde(deny_unknown_fields)]`).
fn item_has_serde_arg(item: &syn::Item, name: &str) -> bool {
    item_attrs(item).iter().any(|attr| {
        if !attr_path_matches(attr, "serde") {
            return false;
        }
        let mut found = false;
        // `parse_nested_meta` visits each comma-separated arg; the `meta.value()`
        // (the `= "..."` part, if any) is left unconsumed, which is fine — we only
        // care that the argument's path is present. A malformed `#[serde(...)]`
        // errors out → no match (not our concern to validate serde's own syntax).
        let _ = attr.parse_nested_meta(|meta| {
            if meta
                .path
                .segments
                .last()
                .is_some_and(|seg| seg.ident == name)
            {
                found = true;
            }
            // Consume an optional `= value` so the walk continues past
            // `rename_all = "camelCase"` to the next arg without erroring.
            if meta.input.peek(syn::Token![=]) {
                if let Ok(v) = meta.value() {
                    let _ = v.parse::<syn::Lit>();
                }
            }
            Ok(())
        });
        found
    })
}

#[cfg(test)]
mod tests {
    use crate::Fingerprint;

    fn item(src: &str) -> syn::Item {
        syn::parse_str::<syn::Item>(src).expect("test item parses")
    }

    fn fp(src: &str) -> Fingerprint {
        Fingerprint::parse(src).expect("test fingerprint parses")
    }

    // ========================================================================
    // ADR-040 Increment 1 (KEYSTONE) — body_calls(path)
    //
    // These are the adversarial tests-first definition-of-done for the
    // `body_calls` leaf matcher. They are *asymmetric*: each binds-bad on a
    // body that calls the needle AND spares-good on a clean sibling that does
    // not — so a test that passes against both the gap and the fix is
    // impossible (a no-op `body_calls` that always-matches fails spare-good;
    // one that never-matches fails binds-bad).
    //
    // Ground truth (probed against syn 2.x): `.unwrap()`/`.expect()` are
    // `syn::ExprMethodCall` (matched by method ident); `std::process::exit(1)`
    // and `mem::forget(x)` are `syn::ExprCall` with an `Expr::Path` callee
    // (matched by LAST path segment). `todo!()`/`unreachable!()`/`panic!()` are
    // `syn::Macro`, NOT calls — they belong to `body_contains_macro`, and
    // `body_calls` must NOT claim them (that would be a false-positive and a
    // duplicate of the macro twin).
    // ========================================================================

    /// KEYSTONE affinity-pair, METHOD-CALL arm. A `Drop` impl whose body
    /// panics via `.unwrap()` (an `ExprMethodCall`, invisible to the
    /// macro-only `body_contains_macro`) must be matched; a clean sibling
    /// `Drop` impl with no such call must be spared. This is the exact silent
    /// gap ADR-040 names: `PanickingInDrop`'s shipped macro-only fingerprint
    /// misses `.unwrap()`-shaped panics.
    #[test]
    fn body_calls_matches_unwrap_method_call_spares_clean() {
        let fp = fp(r#"all_of([item = impl, body_calls("unwrap")])"#);
        // binds-bad: the call-shaped panic path the macro twin is blind to.
        assert!(
            fp.matches(&item(
                "impl Drop for Bad { fn drop(&mut self) { self.h.take().unwrap(); } }"
            )),
            "body_calls(\"unwrap\") must match a Drop impl that calls .unwrap()"
        );
        // spares-good: a clean sibling with no .unwrap() call.
        assert!(
            !fp.matches(&item(
                "impl Drop for Good { fn drop(&mut self) { let _ = self.h.take(); } }"
            )),
            "body_calls(\"unwrap\") must NOT match a Drop impl with no .unwrap() call"
        );
    }

    /// KEYSTONE affinity-pair, PATH-CALL arm. A free/path function call
    /// (`std::process::exit(1)`) is a `syn::ExprCall`, a DIFFERENT AST node
    /// than a method call. This test forces the impl to hook
    /// `visit_expr_call` too — an impl that only hooked `visit_expr_method_call`
    /// would pass the method-call test above yet FAIL here (the asymmetry that
    /// guards against a half-built keystone). Last-segment discipline:
    /// `body_calls("exit")` matches the qualified `std::process::exit`.
    #[test]
    fn body_calls_matches_path_call_by_last_segment_spares_clean() {
        let fp = fp(r#"all_of([item = fn, body_calls("exit")])"#);
        // binds-bad: a path call matched on its LAST segment.
        assert!(
            fp.matches(&item("fn run() { std::process::exit(1); }")),
            "body_calls(\"exit\") must match a fn that calls std::process::exit() (last segment)"
        );
        // spares-good: a sibling that calls a DIFFERENT path function.
        assert!(
            !fp.matches(&item("fn run() { std::mem::drop(()); }")),
            "body_calls(\"exit\") must NOT match a fn that does not call exit"
        );
    }

    /// KEYSTONE — the macro/call boundary. `body_calls` must NOT fire on a
    /// macro invocation (`panic!()`), even though the needle text appears: a
    /// macro is `syn::Macro`, not a call. This guards the false-positive that
    /// would make `body_calls` a redundant, wrong duplicate of
    /// `body_contains_macro`. The clean sibling (a real `panic` *function*
    /// call) confirms the matcher distinguishes the two by AST node, not by
    /// the spelled name.
    #[test]
    fn body_calls_does_not_match_macro_invocation() {
        let fp = fp(r#"all_of([item = fn, body_calls("panic")])"#);
        // spare: panic!() is a MACRO, not a call — body_calls must stay silent.
        assert!(
            !fp.matches(&item(r#"fn boom() { panic!("x"); }"#)),
            "body_calls(\"panic\") must NOT match the panic! MACRO (that is body_contains_macro's job)"
        );
        // bind: a real function literally named `panic` IS a call.
        assert!(
            fp.matches(&item("fn boom() { panic(); }")),
            "body_calls(\"panic\") MUST match a real fn call named panic()"
        );
    }

    /// KEYSTONE — partial domain / Undefined (ADR-010 Amd6). `body_calls` on a
    /// bodyless item-class (a struct) has no locus → `Undefined`, NOT a
    /// vacuous match. The vacuous-`not` guard: `all_of([item = struct,
    /// not(body_calls("unwrap"))])` must evaluate to Undefined (does NOT fire)
    /// rather than matching every struct. This is the soundness contract the
    /// macro twin carries; `body_calls` must carry it identically.
    #[test]
    fn body_calls_undefined_on_bodyless_item_keeps_not_sound() {
        // A bare body_calls on a struct does not fire (Undefined projects to
        // "doesn't fire").
        let bare = fp(r#"body_calls("unwrap")"#);
        assert!(
            !bare.matches(&item("struct S { x: u32 }")),
            "body_calls on a bodyless struct must not fire (Undefined, not Match)"
        );
        // The load-bearing case: not(body_calls(...)) under all_of must NOT
        // vacuously match every struct — Undefined is closed under negation.
        let negated = fp(r#"all_of([item = struct, not(body_calls("unwrap"))])"#);
        assert!(
            !negated.matches(&item("struct S { x: u32 }")),
            "all_of([item=struct, not(body_calls(unwrap))]) must be Undefined (not fire) — \
             the vacuous-not guard (ADR-010 Amd6)"
        );
    }

    /// KEYSTONE — the shipped `PanickingInDrop` silent-gap closure, stated as a
    /// regression contract (ADR-040 "done well"). The OLD macro-only shape is
    /// SILENT on a `.unwrap()`-in-Drop; the NEW `body_calls` shape FIRES. This
    /// asserts BOTH halves in one test so the gap-closure cannot regress
    /// silently: if someone reverts `body_calls` to delegate to macros, the
    /// `new_fires` assertion fails.
    #[test]
    fn body_calls_closes_panicking_in_drop_unwrap_silent_gap() {
        let drop_with_unwrap =
            item("impl Drop for V { fn drop(&mut self) { self.h.take().unwrap(); } }");

        // The shipped PanickingInDrop fingerprint (basic.rs:44-52) — macro-only.
        let old_macro_only = fp(r#"
            all_of([
                item = impl,
                any_of([
                    body_contains_macro("panic"),
                    body_contains_macro("unreachable"),
                    body_contains_macro("todo"),
                    body_contains_macro("unimplemented")
                ])
            ])
        "#);
        assert!(
            !old_macro_only.matches(&drop_with_unwrap),
            "PRECONDITION: the macro-only fingerprint is SILENT on .unwrap()-in-Drop \
             (this is the gap ADR-040 closes)"
        );

        // The new call-aware shape fires on the same site.
        let new_with_calls = fp(r#"
            all_of([
                item = impl,
                any_of([
                    body_contains_macro("panic"),
                    body_calls("unwrap"),
                    body_calls("expect")
                ])
            ])
        "#);
        assert!(
            new_with_calls.matches(&drop_with_unwrap),
            "body_calls closes the gap: the call-aware fingerprint FIRES on .unwrap()-in-Drop"
        );
    }

    /// KEYSTONE — `body_calls` must recurse into NESTED bodies: a `.unwrap()`
    /// hidden in a closure (`.map(|x| x.unwrap())`), a nested fn, or a nested
    /// block (an `if`/`match` arm) is the common real-world shape — if the walk
    /// stopped at the top level it would silently miss them. The `syn::visit`
    /// walk descends; this pins it so a future "optimization" can't shallow it.
    #[test]
    fn body_calls_recurses_into_nested_bodies() {
        let unwrap = fp(r#"all_of([item = fn, body_calls("unwrap")])"#);
        assert!(
            unwrap.matches(&item(
                "fn f(v: Vec<Option<u32>>) { v.iter().map(|x| x.unwrap()).count(); }"
            )),
            "body_calls must see .unwrap() inside a CLOSURE"
        );
        assert!(
            unwrap.matches(&item(
                "fn outer() { fn inner() -> u32 { None::<u32>.unwrap() } let _ = inner(); }"
            )),
            "body_calls must see .unwrap() inside a NESTED fn"
        );
        assert!(
            unwrap.matches(&item(
                "fn f(o: Option<u32>) { if true { let _ = o.unwrap(); } }"
            )),
            "body_calls must see .unwrap() inside a nested if-block"
        );
        // spare: a clean sibling with the same nesting depth but no unwrap.
        assert!(
            !unwrap.matches(&item(
                "fn f(v: Vec<u32>) { v.iter().map(|x| x + 1).count(); }"
            )),
            "body_calls must spare a closure that does NOT call unwrap"
        );
        // path-call inside a closure recurses too.
        let exit = fp(r#"all_of([item = fn, body_calls("exit")])"#);
        assert!(
            exit.matches(&item("fn f() { let g = || std::process::exit(1); g(); }")),
            "body_calls must see a path call inside a CLOSURE"
        );
    }

    /// KEYSTONE — raw-ident soundness END-TO-END (closes the parse-OK-but-silent-miss
    /// window). The parse-side gate (`rejects_body_calls_non_ident_name_loudly`)
    /// ACCEPTS raw idents like `r#fn` / `r#async` because they ARE single
    /// identifiers. This test verifies they then actually FIRE at the matcher —
    /// so an accepted-but-never-matching raw-ident name (a silent miss the gate
    /// was added to prevent) cannot slip through the MATCH side either. If a
    /// future change compared against a de-raw'd ident string while parse still
    /// accepted `r#fn`, this test goes red.
    #[test]
    fn body_calls_raw_ident_fires_end_to_end() {
        // a raw-ident FREE call: r#fn() (fn is a keyword → must be written r#fn).
        let raw_fn = fp(r#"all_of([item = fn, body_calls("r#fn")])"#);
        assert!(
            raw_fn.matches(&item("fn outer() { r#fn(); }")),
            "body_calls(\"r#fn\") must FIRE on a real r#fn() call (raw idents match end-to-end)"
        );
        // a raw-ident METHOD call: x.r#async().
        let raw_async = fp(r#"all_of([item = fn, body_calls("r#async")])"#);
        assert!(
            raw_async.matches(&item("fn f(x: T) { x.r#async(); }")),
            "body_calls(\"r#async\") must FIRE on a real x.r#async() method call"
        );
        // spare: a different raw-ident name must NOT match.
        assert!(
            !raw_fn.matches(&item("fn outer() { r#async(); }")),
            "body_calls(\"r#fn\") must spare a call to a different raw-ident fn"
        );
    }

    // ========================================================================
    // ADR-040 Increment 2 — G1: item qualifier presence/absence
    // (is_async / is_unsafe / is_const). Adversarial tests-first definition-of-
    // done (grammar-leaf-defining-tests.md). Each binds-bad AND spares-good; the
    // partial-domain (Undefined-on-no-locus) trap is pinned so not(is_*) on a
    // wrong-locus item cannot vacuously match (ADR-010 Amd6).
    // ========================================================================

    /// G1 — `is_async` presence/absence. Binds an async fn, spares a sync sibling.
    /// The absence case (`not(is_async)`) must work inside `all_of` under the anchor
    /// rule — the `BlockingCallInAsyncFn` family needs `all_of([is_async, body_calls(...)])`.
    #[test]
    fn is_async_matches_async_fn_spares_sync() {
        let fp = fp("is_async");
        assert!(
            fp.matches(&item("async fn a() {}")),
            "is_async must match an async fn"
        );
        assert!(
            !fp.matches(&item("fn s() {}")),
            "is_async must NOT match a sync fn"
        );
    }

    /// G1 — `is_unsafe` on BOTH fn and impl (the two loci that carry `unsafe`).
    /// `UnsafeSendSync` needs `is_unsafe` on the impl; `RawPtrDerefInSafeFn` needs the
    /// ABSENCE on a fn. Asymmetric on each.
    #[test]
    fn is_unsafe_matches_unsafe_fn_and_impl_spares_safe() {
        let fp = fp("is_unsafe");
        assert!(
            fp.matches(&item("unsafe fn u() {}")),
            "is_unsafe must match unsafe fn"
        );
        assert!(
            fp.matches(&item("unsafe impl Send for Foo {}")),
            "is_unsafe must match unsafe impl"
        );
        assert!(
            !fp.matches(&item("fn s() {}")),
            "is_unsafe must NOT match a safe fn"
        );
        assert!(
            !fp.matches(&item("impl Send for Foo {}")),
            "is_unsafe must NOT match a safe impl"
        );
    }

    /// G1 — the THIRD `unsafe` locus: an `unsafe trait`. A `trait` HAS the unsafe
    /// locus (`ItemTrait.unsafety`), so `is_unsafe` on it must be a definite
    /// Match/NoMatch — NOT `Undefined`. Omitting the `(Unsafe, Trait)` arm was a
    /// FALSE-Undefined (the item has the locus, the arm didn't enumerate it). This
    /// is the notary/adversarial-flagged fix; it also lets `UnsafeSendSync`-shaped
    /// fingerprints reach an `unsafe trait` Send-marker.
    #[test]
    fn is_unsafe_matches_unsafe_trait_definite_not_undefined() {
        let bare = fp("is_unsafe");
        assert!(
            bare.matches(&item("unsafe trait Scary {}")),
            "is_unsafe must match an unsafe trait (a definite Match, not Undefined)"
        );
        // A SAFE trait must be a definite NoMatch (well-posed), so the anchored
        // absence form distinguishes it — proving it is NOT Undefined.
        let absent = fp(r"all_of([item = trait, not(is_unsafe)])");
        assert!(
            absent.matches(&item("trait Calm {}")),
            "anchored not(is_unsafe) matches a SAFE trait (definite NoMatch on is_unsafe → not = Match)"
        );
        assert!(
            !absent.matches(&item("unsafe trait Scary {}")),
            "anchored not(is_unsafe) spares an unsafe trait"
        );
    }

    /// G1 — the ABSENCE case under the anchor rule. `RawPtrDerefInSafeFn`: a fn
    /// that is NOT unsafe is the tell, but a bare `not(is_unsafe)` must be a PARSE ERROR
    /// (anti-graffiti, ADR-010 Amd3 OQ3). Only `all_of([anchor, not(is_unsafe)])` is legal.
    /// This test pins that `not(is_unsafe)` works ONLY anchored, and the anchored form
    /// is asymmetric.
    #[test]
    fn is_unsafe_absence_only_works_anchored() {
        // bare not(is_unsafe) is a parse error (anchor rule).
        assert!(
            Fingerprint::parse("not(is_unsafe)").is_err(),
            "bare not(is_unsafe) must be rejected"
        );
        // anchored absence: a fn that is NOT unsafe.
        let fp = fp(r"all_of([item = fn, not(is_unsafe)])");
        assert!(
            fp.matches(&item("fn safe_one() {}")),
            "anchored not(is_unsafe) matches a safe fn"
        );
        assert!(
            !fp.matches(&item("unsafe fn unsafe_one() {}")),
            "anchored not(is_unsafe) spares an unsafe fn"
        );
    }

    /// G1 — `is_const`, fn locus. Asymmetric. (The `const` *qualifier* on a fn,
    /// distinct from the `item = const` item-kind.)
    #[test]
    fn is_const_matches_const_fn_spares_runtime() {
        let fp = fp("is_const");
        assert!(
            fp.matches(&item("const fn c() -> u32 { 0 }")),
            "is_const must match a const fn"
        );
        assert!(
            !fp.matches(&item("fn r() -> u32 { 0 }")),
            "is_const must NOT match a non-const fn"
        );
    }

    /// G1 — partial-domain (ADR-010 Amd6): `is_async` on a struct has no locus →
    /// Undefined, so the bare form does not fire AND `not(is_async)` does not
    /// vacuously match every struct. This is the silent-failure trap for G1.
    #[test]
    fn is_async_undefined_on_struct_keeps_not_sound() {
        assert!(
            !fp("is_async").matches(&item("struct S;")),
            "is_async on a struct must not fire (Undefined)"
        );
        let negated = fp(r"all_of([item = struct, not(is_async)])");
        assert!(
            !negated.matches(&item("struct S;")),
            "all_of([item=struct, not(is_async)]) must be Undefined (not vacuously match every struct)"
        );
    }

    // ========================================================================
    // ADR-040 Increment 2 — G3: trait-impl identity (impl_of_trait, presence AND
    // absence). Adversarial tests-first definition-of-done. Reads ONE impl item's
    // own trait-path last segment (an inherent impl → NoMatch; a non-impl →
    // Undefined). The cross-item "does Type impl X anywhere" form is G4/charter.
    // ========================================================================

    /// G3 — `impl_of_trait`, presence. Reads the impl's trait path last segment.
    /// `UnsafeSendSync` needs `impl_of_trait("Send")`. Asymmetric.
    #[test]
    fn impl_of_trait_matches_trait_impl_spares_other_and_inherent() {
        let fp = fp(r#"impl_of_trait("Send")"#);
        assert!(
            fp.matches(&item("unsafe impl Send for Foo {}")),
            "impl_of_trait(Send) matches impl Send"
        );
        assert!(
            !fp.matches(&item("impl Sync for Foo {}")),
            "impl_of_trait(Send) spares impl Sync"
        );
        assert!(
            !fp.matches(&item("impl Foo { fn m(&self) {} }")),
            "impl_of_trait(Send) spares an inherent impl"
        );
    }

    /// G3 — the keystone: `impl_of_trait("Drop")` asserts a `Drop` impl is ACTUALLY
    /// `Drop`, not merely an inherent impl with a method *named* `drop` (which the
    /// shipped `PanickingInDrop` fingerprint can't distinguish). Binds the real
    /// `Drop` impl, spares an inherent impl whose method is just named `drop`.
    #[test]
    fn impl_of_trait_drop_distinguishes_real_drop_from_named_method() {
        let fp = fp(r#"all_of([item = impl, impl_of_trait("Drop")])"#);
        assert!(
            fp.matches(&item("impl Drop for V { fn drop(&mut self) {} }")),
            "impl_of_trait(Drop) must match a real Drop impl"
        );
        assert!(
            !fp.matches(&item("impl V { fn drop(&mut self) {} }")),
            "impl_of_trait(Drop) must NOT match an inherent impl with a method merely NAMED drop"
        );
    }

    /// G3 — `not(impl_of_trait(...))` ABSENCE, anchored. Bare absence rejected
    /// (anti-graffiti); anchored absence asymmetric. SCOPED to "THIS impl item is
    /// some OTHER trait" — a single-item, well-posed question (the cross-item
    /// "type lacks trait X program-wide" form is G4/charter, NOT this tier).
    #[test]
    fn impl_of_trait_absence_only_anchored() {
        assert!(
            Fingerprint::parse(r#"not(impl_of_trait("Eq"))"#).is_err(),
            "bare not(impl_of_trait) must be rejected (anti-graffiti)"
        );
        let fp = fp(r#"all_of([item = impl, not(impl_of_trait("Send"))])"#);
        assert!(
            fp.matches(&item("impl Sync for Foo {}")),
            "anchored not(impl_of_trait(Send)) fires on an impl that is some OTHER trait"
        );
        assert!(
            !fp.matches(&item("impl Send for Foo {}")),
            "anchored not(impl_of_trait(Send)) spares an impl that IS Send"
        );
    }

    /// G3 — partial-domain: `impl_of_trait` on a struct (not an impl) → Undefined,
    /// keeps `not()` sound.
    #[test]
    fn impl_of_trait_undefined_on_non_impl() {
        assert!(
            !fp(r#"impl_of_trait("Send")"#).matches(&item("struct S;")),
            "impl_of_trait on a struct must not fire (Undefined)"
        );
        let negated = fp(r#"all_of([item = struct, not(impl_of_trait("Send"))])"#);
        assert!(
            !negated.matches(&item("struct S;")),
            "not(impl_of_trait) on a struct must be Undefined, not vacuous match"
        );
    }

    // ========================================================================
    // ADR-040 Increment 2 — G1b: derive-list / serde-arg introspection +
    // attribute-absence. Adversarial tests-first definition-of-done. Syntactic
    // last-ident membership (no path resolution — the derive/path-collision is
    // the honest false-positive the dial carries). attr_absent is the anchored
    // negation of the shipped attr_present (no new operator).
    // ========================================================================

    /// G1b — derive-list membership. `derives("Hash")` is true iff Hash is in a
    /// `#[derive(...)]` on the item. Asymmetric: a struct deriving Hash binds; one
    /// deriving only Clone spares.
    #[test]
    fn derives_matches_member_spares_nonmember() {
        let fp = fp(r#"derives("Hash")"#);
        assert!(
            fp.matches(&item("#[derive(Hash, Clone)] struct S { x: u32 }")),
            "derives(Hash) must match a struct that derives Hash"
        );
        assert!(
            !fp.matches(&item("#[derive(Clone, Debug)] struct S { x: u32 }")),
            "derives(Hash) must NOT match a struct that does not derive Hash"
        );
    }

    /// G1b — the DANGEROUS split: derives(Hash) but NOT derives(Eq), anchored.
    /// The `derive(Hash)`-without-`Eq` family.
    #[test]
    fn derives_hash_without_eq_anchored() {
        let fp = fp(r#"all_of([item = struct, derives("Hash"), not(derives("Eq"))])"#);
        assert!(
            fp.matches(&item("#[derive(Hash)] struct Bad { x: u32 }")),
            "Hash-without-Eq must fire on a struct deriving only Hash"
        );
        assert!(
            !fp.matches(&item(
                "#[derive(Hash, Eq, PartialEq)] struct Good { x: u32 }"
            )),
            "Hash-without-Eq must spare a struct that also derives Eq"
        );
    }

    /// G1b — attr-arg introspection: `deny_unknown_fields` ∈ `#[serde(...)]`.
    /// `DeserializeWithoutDenyUnknownFields`. Asymmetric on the arg's presence.
    #[test]
    fn serde_arg_deny_unknown_fields_membership() {
        let fp = fp(r#"all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])"#);
        assert!(
            fp.matches(&item(
                r#"#[derive(Deserialize)] #[serde(rename_all = "camelCase")] struct Cfg { a: u32 }"#
            )),
            "must fire on a Deserialize struct whose serde args lack deny_unknown_fields"
        );
        assert!(
            !fp.matches(&item(
                r"#[derive(Deserialize)] #[serde(deny_unknown_fields)] struct Cfg { a: u32 }"
            )),
            "must spare a Deserialize struct that DOES set deny_unknown_fields"
        );
    }

    /// G1b — `attr_absent` is the anchored negation of `attr_present`; a bare
    /// absence must be rejected (anti-graffiti) and the anchored form asymmetric.
    #[test]
    fn attr_absent_anchored_asymmetric() {
        // A bare attribute-absence is rejected: `not(...)` is only legal anchored.
        assert!(
            Fingerprint::parse(r#"not(attr_present("non_exhaustive"))"#).is_err(),
            "a bare attribute-absence (bare not) must be rejected (anti-graffiti)"
        );
        let fp = fp(r#"all_of([item = enum, not(attr_present("non_exhaustive"))])"#);
        assert!(
            fp.matches(&item("enum E { A, B }")),
            "non_exhaustive-absent must fire on a plain enum"
        );
        assert!(
            !fp.matches(&item("#[non_exhaustive] enum E { A, B }")),
            "non_exhaustive-absent must spare an enum that IS non_exhaustive"
        );
    }

    #[test]
    fn item_struct_matches() {
        let fp = fp("item = struct");
        assert!(fp.matches(&item("struct Foo;")));
        assert!(!fp.matches(&item("enum Foo { A }")));
    }

    #[test]
    fn item_enum_matches() {
        let fp = fp("item = enum");
        assert!(fp.matches(&item("enum E { A, B }")));
        assert!(!fp.matches(&item("struct S;")));
    }

    #[test]
    fn name_glob_matches_struct() {
        let fp = fp(r#"name = matches("*Class")"#);
        assert!(fp.matches(&item("struct DeterminismClass;")));
        assert!(!fp.matches(&item("struct Foo;")));
    }

    #[test]
    fn variants_range_matches() {
        let fp = fp("variants = 2..=4");
        assert!(fp.matches(&item("enum E { A, B, C }")));
        assert!(!fp.matches(&item("enum E { A }")));
        assert!(!fp.matches(&item("enum E { A, B, C, D, E }")));
    }

    #[test]
    fn variants_does_not_match_struct() {
        let fp = fp("variants = 1..=10");
        assert!(!fp.matches(&item("struct S;")));
    }

    // ATK-FP-VARIANTS-ZERO: variants = 0..=0 is valid and matches an empty enum.
    // Empty enums (`enum Never {}`) are a real pattern in Rust for unreachable types.
    #[test]
    fn variants_zero_range_matches_empty_enum() {
        let fp = fp("variants = 0..=0");
        assert!(
            fp.matches(&item("enum Never {}")),
            "variants = 0..=0 must match an empty enum"
        );
        assert!(
            !fp.matches(&item("enum E { A }")),
            "variants = 0..=0 must NOT match a one-variant enum"
        );
    }

    #[test]
    fn has_method_matches_simple_signature() {
        let fp = fp(r#"has_method("meet", "(& self, other: Self) -> Self")"#);
        let i = item(
            "impl Foo {
                fn meet(&self, other: Self) -> Self { other }
                fn other(&self) {}
            }",
        );
        // Note: the user's pattern doesn't carry parameter names; our
        // renderer produces "(& self, Self) -> Self" — let's see how the
        // matcher behaves and adjust expectations.
        let _ = fp;
        let _ = i;
        // Actual coverage in the next test (shape-only).
    }

    #[test]
    fn has_method_shape_only_signature() {
        let fp = fp(r#"has_method("meet", "(& self, Self) -> Self")"#);
        let i = item(
            "impl Foo {
                fn meet(&self, other: Self) -> Self { other }
            }",
        );
        assert!(fp.matches(&i));
    }

    #[test]
    fn has_method_does_not_match_wrong_signature() {
        let fp = fp(r#"has_method("meet", "(Self, Self) -> Self")"#);
        let i = item(
            "impl Foo {
                fn meet(&self, other: Self) -> Self { other }
            }",
        );
        assert!(!fp.matches(&i));
    }

    #[test]
    fn attr_present_matches_repr() {
        let fp = fp(r#"attr_present("repr")"#);
        let i = item("#[repr(u8)] enum E { A }");
        assert!(fp.matches(&i));
    }

    #[test]
    fn attr_present_full_path() {
        let fp = fp(r#"attr_present("clippy::panic")"#);
        let i = item("#[clippy::panic] fn f() {}");
        assert!(fp.matches(&i));
    }

    #[test]
    fn doc_contains_matches() {
        let fp = fp(r#"doc_contains("strength")"#);
        let i = item("/// Has strength bits.\nstruct S;");
        assert!(fp.matches(&i));
    }

    #[test]
    fn body_contains_macro_panic() {
        let fp = fp(r#"body_contains_macro("panic")"#);
        let i = item("fn f() { panic!(\"oops\"); }");
        assert!(fp.matches(&i));
    }

    #[test]
    fn body_contains_macro_inside_impl_method() {
        let fp = fp(r#"body_contains_macro("panic")"#);
        let i = item("impl Drop for X { fn drop(&mut self) { panic!(\"oops\"); } }");
        assert!(fp.matches(&i));
    }

    #[test]
    fn body_contains_macro_negative() {
        let fp = fp(r#"body_contains_macro("panic")"#);
        let i = item("fn f() { let x = 1; }");
        assert!(!fp.matches(&i));
    }

    // ATK-FP-BODY-MACRO-ALIAS: body_contains_macro uses last-segment ident matching.
    // A macro renamed via `use panic as p; p!()` invokes as path `[p]`, so
    // body_contains_macro("panic") returns false for `p!()`.
    // This is documented behavior (last-segment match), NOT a bug to fix — but
    // it IS a documented limitation that adopters must understand: fingerprints
    // using body_contains_macro("panic") will NOT fire when the macro is aliased.
    // This test asserts the current behavior so it doesn't silently change.
    #[test]
    fn body_contains_macro_aliased_invocation_is_not_detected() {
        let fp = fp(r#"body_contains_macro("panic")"#);
        // `p` is an alias for `panic!` — invoked as single-segment path.
        // Last segment is `p`, not `panic` — body_contains_macro("panic") returns false.
        let i = item("fn f() { use std::panic as p; p!(\"aliased panic\"); }");
        // CURRENT BEHAVIOR: false (aliased invocation not detected).
        // If this assertion starts FAILING, it means aliased macros are now detected —
        // update the documented limitation in the fingerprint grammar docs.
        assert!(
            !fp.matches(&i),
            "ATK-FP-BODY-MACRO-ALIAS: body_contains_macro('panic') must NOT fire for \
             aliased macro invocation `p!()` (alias of panic!). If this fails, \
             the implementation now detects aliases — document the new capability."
        );
    }

    // ATK-FP-BODY-MACRO-QUALIFIED: body_contains_macro uses last-segment ident.
    // A qualified path like `std::panic!()` has last segment `panic` — MATCHES.
    // This tests that qualified invocations DO fire (correct behavior).
    #[test]
    fn body_contains_macro_qualified_path_matches_on_last_segment() {
        let fp = fp(r#"body_contains_macro("panic")"#);
        // std::panic!() — last segment is `panic` — SHOULD match.
        let i = item("fn f() { std::panic!(\"qualified panic\"); }");
        assert!(
            fp.matches(&i),
            "ATK-FP-BODY-MACRO-QUALIFIED: body_contains_macro('panic') must fire for \
             qualified invocation std::panic!() since last segment is 'panic'."
        );
    }

    // ATK-FP-HAS-METHOD-TRAIT-SILENT-MISS: has_method returns false for item = trait.
    // has_matching_method() only handles syn::Item::Impl; for trait definitions it
    // early-returns false. A fingerprint all_of([item = trait, has_method("drop")])
    // would NEVER match any trait — has_method always fails for traits.
    // This is a documented limitation (not a bug to fix now): has_method is
    // impl-block-scoped. Test locks in the behavior so a future fix doesn't
    // accidentally land without detection.
    #[test]
    fn has_method_on_trait_item_always_returns_false() {
        let fp = fp(r#"all_of([item = trait, has_method("drop", "(&mut self)")])"#);
        // A trait that defines exactly `fn drop(&mut self)` — structurally matches
        // the method pattern IF has_matching_method checked traits. It doesn't.
        let i = item("trait Droppable { fn drop(&mut self); }");
        assert!(
            !fp.matches(&i),
            "ATK-FP-HAS-METHOD-TRAIT: has_method must return false for item = trait \
             (has_matching_method only handles Item::Impl, not Item::Trait). \
             If this fails, has_method now supports trait defs — document the new capability."
        );
    }

    // ATK-FP-DOC-MULTILINE: doc_contains works across multiple doc comment lines.
    #[test]
    fn doc_contains_multiline_doc_comment() {
        let fp = fp(r#"doc_contains("SENTINEL")"#);
        // Three doc lines — needle appears in the second.
        let i = item("/// First line.\n/// Contains SENTINEL here.\n/// Third line.\nstruct S;");
        assert!(
            fp.matches(&i),
            "doc_contains must match across multiple doc lines"
        );
    }

    // ATK-FP-DOC-RAW-ATTR: doc_contains works with #[doc = "..."] attribute form.
    #[test]
    fn doc_contains_raw_doc_attribute() {
        let fp = fp(r#"doc_contains("SENTINEL")"#);
        let i = item(r#"#[doc = "Contains SENTINEL."] struct S;"#);
        assert!(
            fp.matches(&i),
            "doc_contains must match raw #[doc = \"...\"] attributes"
        );
    }

    // ATK-FP-NOT-DOC-UNDOCUMENTED: not(doc_contains("X")) matches items with NO docs.
    //
    // doc_contains("X") returns false for undocumented items (empty doc text doesn't
    // contain "X"). Therefore not(doc_contains("X")) returns TRUE for undocumented items.
    // This is CORRECT behavior but potentially surprising for adopters who write:
    //   all_of([item = struct, not(doc_contains("unsafe"))])
    // expecting to match only "structs with docs that omit 'unsafe'" — but actually
    // matching ALL structs with no docs as well. This test locks the behavior.
    #[test]
    fn not_doc_contains_matches_undocumented_item() {
        let fp = fp(r#"all_of([item = struct, not(doc_contains("unsafe"))])"#);
        // A struct with no doc comment at all: doc_contains returns false, not() inverts to true.
        assert!(
            fp.matches(&item("pub struct NoDoc;")),
            "ATK-FP-NOT-DOC-UNDOCUMENTED: all_of([item=struct, not(doc_contains('unsafe'))]) \
             must match an undocumented struct — empty doc text doesn't contain 'unsafe', \
             so not(doc_contains) is true. Adopters should be aware this matches ALL structs \
             without 'unsafe' in docs, including those with NO docs at all."
        );
        // A struct with docs that DO contain "unsafe": not(doc_contains) is false.
        assert!(
            !fp.matches(&item("/// This is unsafe usage.\npub struct DocUnsafe;")),
            "ATK-FP-NOT-DOC-UNDOCUMENTED: all_of([item=struct, not(doc_contains('unsafe'))]) \
             must NOT match a struct whose doc contains 'unsafe'."
        );
        // A struct with docs that DON'T contain "unsafe": not(doc_contains) is true.
        assert!(
            fp.matches(&item("/// Safe to use always.\npub struct DocSafe;")),
            "ATK-FP-NOT-DOC-UNDOCUMENTED: all_of([item=struct, not(doc_contains('unsafe'))]) \
             must match a struct whose doc does NOT contain 'unsafe'."
        );
    }

    #[test]
    fn all_of_matches() {
        let fp = fp(r#"all_of([item = enum, name = matches("*Class")])"#);
        assert!(fp.matches(&item("enum FooClass { A }")));
        assert!(!fp.matches(&item("enum Foo { A }")));
        assert!(!fp.matches(&item("struct FooClass;")));
    }

    #[test]
    fn any_of_matches() {
        let fp = fp("any_of([item = struct, item = enum])");
        assert!(fp.matches(&item("struct S;")));
        assert!(fp.matches(&item("enum E { A }")));
        assert!(!fp.matches(&item("trait T {}")));
    }

    #[test]
    fn not_inside_all_of() {
        let fp = fp(r#"all_of([item = enum, not(name = matches("Test*"))])"#);
        assert!(fp.matches(&item("enum FooClass { A }")));
        assert!(!fp.matches(&item("enum TestEnum { A }")));
    }

    #[test]
    fn polarity_inverted_class_meet_canonical() {
        // The canonical fingerprint from ADR-010 Amendment 1.
        let fp = fp(r#"
                item = enum,
                name = matches("*Class"),
                variants = 3..=8,
                has_method("meet", "(& self, Self) -> Self")
            "#);
        let canonical = item(
            "
            #[repr(u8)]
            enum DeterminismClass {
                Strong,
                Medium,
                Weak,
            }
        ",
        );
        // No `meet` method here, so the fp does NOT match.
        assert!(!fp.matches(&canonical));

        // Add an impl block separately for the meet test:
        let with_meet = item(
            "
            impl DeterminismClass {
                fn meet(&self, other: Self) -> Self { other }
            }
            ",
        );
        // `with_meet` is item-impl, not item-enum, so item=enum gates it out.
        assert!(!fp.matches(&with_meet));

        // Demonstrate the structural pieces match individually:
        let item_enum_only = fp_only("item = enum");
        let name_only = fp_only(r#"name = matches("*Class")"#);
        let variants_only = fp_only("variants = 3..=8");
        assert!(item_enum_only.matches(&canonical));
        assert!(name_only.matches(&canonical));
        assert!(variants_only.matches(&canonical));
    }

    fn fp_only(src: &str) -> Fingerprint {
        Fingerprint::parse(src).unwrap()
    }

    // FIXED (option 1): normalize_signature_canonical strips parameter names;
    // has_method now matches whether or not the pattern includes named parameters.
    #[test]
    fn has_method_named_parameter_in_pattern_still_matches() {
        // Pattern with named parameter — natural Rust style that users will write.
        let fp = fp(r#"has_method("meet", "(&self, other: Self) -> Self")"#);
        let i = item(
            "impl Foo {
                fn meet(&self, other: Self) -> Self { other }
            }",
        );
        assert!(
            fp.matches(&i),
            "ATK-HM-1: has_method pattern '(&self, other: Self) -> Self' must \
             match 'fn meet(&self, other: Self) -> Self'. \
             render_inputs strips parameter names (yields 'Self', not 'other: Self'), \
             but normalize_signature_canonical preserves the 'other :' token in the \
             pattern. String comparison '(& self , other : Self) -> Self' != \
             '(& self , Self) -> Self' — 0 matches. \
             Fix: strip param names in normalize_signature_canonical before comparison, \
             OR add a validation error when named params are detected in the pattern."
        );
    }

    #[test]
    fn has_method_named_self_type_param_matches() {
        // ATK-HM-2: pattern with a named param whose TYPE is `Self` (no &self receiver).
        // `strip_param_names_in_sig_pattern` wraps as `fn __pat__(x: Self) -> Self` and
        // calls syn::parse2 — in a non-impl context syn may reject `Self` as a param
        // type, causing parse failure and fallback to the raw tokenized form
        // "(x : Self) -> Self". But render_inputs produces "(Self) -> Self" for the
        // actual sig, so the match would fail silently.
        let fp = fp(r#"has_method("clone_self", "(x: Self) -> Self")"#);
        let i = item(
            "impl Foo {
                fn clone_self(x: Self) -> Self { x }
            }",
        );
        assert!(
            fp.matches(&i),
            "ATK-HM-2: pattern '(x: Self) -> Self' must match 'fn clone_self(x: Self) -> Self'. \
             If syn rejects 'fn __pat__(x: Self) -> Self' (Self not valid outside impl/trait), \
             strip_param_names_in_sig_pattern returns None and the fallback tokenized form \
             '(x : Self) -> Self' does not match render_inputs' '(Self) -> Self'. \
             Silent miss — the fallback path does not handle named params with Self types."
        );
    }

    #[test]
    fn has_method_mut_self_receiver_with_named_params_matches() {
        // ATK-HM-3: &mut self receiver + named typed param. Receiver path is
        // symmetric (both strip_param_names and render_inputs use to_token_stream
        // for Receiver). But verify there's no off-by-one or spacing issue.
        let fp = fp(r#"has_method("push", "(&mut self, value: u32)")"#);
        let i = item(
            "impl Stack {
                fn push(&mut self, value: u32) { let _ = value; }
            }",
        );
        assert!(
            fp.matches(&i),
            "ATK-HM-3: pattern '(&mut self, value: u32)' must match \
             'fn push(&mut self, value: u32)'. Receiver should survive unchanged \
             through both strip_param_names and render_inputs; typed param name \
             'value:' must be stripped by strip_param_names_in_sig_pattern."
        );
    }

    #[test]
    fn has_method_named_params_no_self_all_primitive_types_matches() {
        // ATK-HM-4: no self receiver, named params with primitive types.
        // strip_param_names_in_sig_pattern should strip 'a:' and 'b:' leaving only types.
        let fp = fp(r#"has_method("add", "(a: u32, b: u32) -> u32")"#);
        let i = item(
            "impl Calc {
                fn add(a: u32, b: u32) -> u32 { a + b }
            }",
        );
        assert!(
            fp.matches(&i),
            "ATK-HM-4: pattern '(a: u32, b: u32) -> u32' must match \
             'fn add(a: u32, b: u32) -> u32'. strip_param_names_in_sig_pattern \
             must strip both 'a:' and 'b:' leaving '(u32, u32) -> u32'."
        );
    }

    #[test]
    fn has_method_lifetime_param_in_signature_matches() {
        // ATK-HM-5: lifetime in signature pattern.
        // strip_param_names_in_sig_pattern wraps as fn __pat__<'a>(&'a self, x: &'a str) -> &'a str
        // Both the Receiver (&'a self) and the typed param (&'a str) go through to_token_stream.
        // If lifetime breaks the pattern → name strip path → silent miss.
        let fp = fp(r#"has_method("as_str", "(&self, extra: &str) -> &str")"#);
        let i = item(
            "impl Wrapper {
                fn as_str(&self, extra: &str) -> &str { extra }
            }",
        );
        assert!(
            fp.matches(&i),
            "ATK-HM-5: pattern '(&self, extra: &str) -> &str' must match \
             'fn as_str(&self, extra: &str) -> &str'. Named param 'extra:' must \
             be stripped; reference types '&str' should survive normalization."
        );
    }

    // ATK-FP-NOT-BODY-VACUOUS (CLOSED by ADR-010 Amd6 Match3): not(body_contains_macro(X))
    // on a bodyless item-class is no longer vacuously true.
    //
    // Before Match3, body_contains_macro() returned `false` for structs/enums/traits
    // (the `_ => {}` arm), so not(false)=true → all_of([item=struct, not(...)]) matched
    // EVERY struct. The adopter intended "structs without panic!" but got all structs —
    // the fingerprint fired everywhere it was meant to filter.
    //
    // Under Match3, body_contains_macro on a struct returns `Undefined` (no body-locus —
    // the question is malformed here, like an assay run on the wrong tissue). The
    // Kleene-strong algebra gives `not(Undefined) = Undefined`, and
    // `all_of([Match(item=struct), Undefined]) = Undefined`. The Level-2 fingerprint-fires
    // projection maps `Undefined` → doesn't fire. So the fingerprint correctly does NOT
    // match a plain struct: the malformed-here predicate cannot vacuously pass.
    #[test]
    fn not_body_contains_macro_on_struct_is_undefined_not_vacuous() {
        let fp = fp(r#"all_of([item = struct, not(body_contains_macro("panic"))])"#);

        // body_contains_macro on a struct is Undefined; not(Undefined)=Undefined;
        // all_of(Match, Undefined)=Undefined; projection → doesn't fire.
        assert!(
            !fp.matches(&item("pub struct PlainStruct;")),
            "ATK-FP-NOT-BODY-VACUOUS (CLOSED): all_of([item=struct, not(body_contains_macro('panic'))]) \
             must NOT match a plain struct — body_contains_macro on a bodyless item is Undefined, \
             not(Undefined)=Undefined, all_of propagates Undefined, projection → doesn't fire. \
             The vacuous-not hazard is closed at the type level (ADR-010 Amd6)."
        );
        assert!(
            !fp.matches(&item(
                "#[derive(Debug)] pub struct DerivedStruct { x: u32 }"
            )),
            "ATK-FP-NOT-BODY-VACUOUS (CLOSED): a struct with fields must NOT match either — \
             fields are not a fn body, so body_contains_macro is still Undefined and the \
             fingerprint does not fire vacuously."
        );

        // A function that DOES call panic! should NOT match (item=struct gates it: the
        // item-kind leaf is a definite NoMatch, short-circuiting all_of to NoMatch).
        assert!(
            !fp.matches(&item("fn uses_panic() { panic!(\"oops\"); }")),
            "ATK-FP-NOT-BODY-VACUOUS: item=struct in all_of must gate out functions."
        );
    }

    // ATK-FP-NOT-BODY-FN-CORRECT: not(body_contains_macro(X)) on a function
    // correctly excludes functions that use the macro.
    //
    // This confirms the NON-vacuous case works correctly: for fn items,
    // body_contains_macro returns the actual result, so not() is meaningful.
    // all_of([item = fn, not(body_contains_macro("panic"))]) should:
    //   - match functions that don't use panic!
    //   - NOT match functions that use panic!
    #[test]
    fn not_body_contains_macro_on_fn_is_not_vacuous() {
        let fp = fp(r#"all_of([item = fn, not(body_contains_macro("panic"))])"#);

        // Function WITHOUT panic! -- not(false) = true -- correctly matches.
        assert!(
            fp.matches(&item("fn no_panic() { let x = 1; }")),
            "ATK-FP-NOT-BODY-FN-CORRECT: all_of([item=fn, not(body_contains_macro('panic'))]) \
             must match a function that does NOT use panic!."
        );

        // Function WITH panic! -- not(true) = false -- correctly excluded.
        assert!(
            !fp.matches(&item("fn with_panic() { panic!(\"oops\"); }")),
            "ATK-FP-NOT-BODY-FN-CORRECT: all_of([item=fn, not(body_contains_macro('panic'))]) \
             must NOT match a function that uses panic!."
        );
    }

    // ATK-FP-NOT-IN-ANY-OF-REJECTED: not() directly under any_of is a parse error.
    //
    // ADR-010 Amendment 3 OQ3 closes the De Morgan loophole: any_of([not(A), not(B)])
    // is equivalent to not(all_of([A, B])) -- it re-creates top-level negation.
    // The parser rejects not() directly under any_of.
    //
    // This test verifies the rejection happens at parse time (not silently accepted).
    #[test]
    fn not_directly_under_any_of_is_rejected_at_parse_time() {
        let result = crate::Fingerprint::parse(r"any_of([not(item = fn), item = struct])");
        assert!(
            result.is_err(),
            "ATK-FP-NOT-IN-ANY-OF: not() directly under any_of must be rejected at parse time \
             (ADR-010 Amendment 3 OQ3 -- De Morgan loophole). Got: {:?}",
            result
        );
    }

    // ATK-FP-ALL-OF-ONLY-NOTS-REJECTED: all_of with only not() children is rejected.
    //
    // ADR-010 Amendment 3 OQ3 requires at least one positive matcher sibling in
    // any all_of that contains not(). all_of([not(A), not(B)]) is rejected.
    #[test]
    fn all_of_containing_only_nots_is_rejected_at_parse_time() {
        let result =
            crate::Fingerprint::parse(r#"all_of([not(item = fn), not(name = matches("Test*"))])"#);
        assert!(
            result.is_err(),
            "ATK-FP-ALL-OF-ONLY-NOTS: all_of containing only not() children must be \
             rejected (ADR-010 Amendment 3 OQ3 -- requires at least one positive matcher). \
             Got: {:?}",
            result
        );
    }

    // ========================================================================
    // ADR-010 Amendment 6 — Match3 Kleene-strong algebra (Level-1 invariants)
    //
    // The user-facing tests above exercise the Level-2 projection (fires? only
    // if Match). These exercise the Level-1 leaf algebra directly: the spec is
    // explicit that `Undefined` must PROPAGATE through combinators and must NOT
    // collapse to `NoMatch` inside `all_of`. Collapsing it would re-introduce
    // the vacuous-not defect; these tests pin the propagation at the type level.
    // ========================================================================

    use super::{Match3, match_constraint};

    #[test]
    fn match3_not_is_kleene_strong() {
        // not(Undefined) = Undefined — the load-bearing clinical invariant
        // (you cannot negate an indeterminate into a definite, ADR-010 Amd6).
        assert_eq!(Match3::Match.not(), Match3::NoMatch);
        assert_eq!(Match3::NoMatch.not(), Match3::Match);
        assert_eq!(
            Match3::Undefined.not(),
            Match3::Undefined,
            "not(Undefined) must stay Undefined — collapsing it to Match is the \
             vacuous-not defect Match3 exists to kill"
        );
    }

    #[test]
    fn match3_body_predicate_undefined_on_bodyless_definite_on_fn() {
        // body_contains_macro: Undefined on a struct (no locus), definite on a fn.
        let body_fp = crate::Fingerprint::parse(r#"body_contains_macro("panic")"#).unwrap();
        let c = &body_fp.constraints[0];

        assert_eq!(
            match_constraint(c, &item("pub struct S;")),
            Match3::Undefined,
            "body_contains_macro on a bodyless struct must be Undefined, not NoMatch"
        );
        assert_eq!(
            match_constraint(c, &item("fn uses() { panic!(); }")),
            Match3::Match,
            "body_contains_macro on a fn that calls the macro must be Match"
        );
        assert_eq!(
            match_constraint(c, &item("fn clean() { let _ = 1; }")),
            Match3::NoMatch,
            "body_contains_macro on a fn that does NOT call the macro must be NoMatch (definite)"
        );
    }

    #[test]
    fn match3_undefined_propagates_through_all_of_not_collapsing() {
        // all_of([item=struct, body_contains_macro("panic")]) on a struct:
        // item=struct is Match (definite), body_contains_macro is Undefined.
        // all_of(Match, Undefined) MUST be Undefined (definedness gap preserved),
        // NOT NoMatch (which would collapse the type-level distinction).
        let fp =
            crate::Fingerprint::parse(r#"all_of([item = struct, body_contains_macro("panic")])"#)
                .unwrap();
        assert_eq!(
            match_constraint(&fp.constraints[0], &item("pub struct S;")),
            Match3::Undefined,
            "all_of(Match, Undefined) must propagate Undefined, not collapse to NoMatch"
        );

        // A definite NoMatch sibling DOES short-circuit all_of to NoMatch:
        // item=fn is NoMatch on a struct → the whole all_of is NoMatch (definite
        // failure dominates the undefined gap).
        let fp2 = crate::Fingerprint::parse(r#"all_of([item = fn, body_contains_macro("panic")])"#)
            .unwrap();
        assert_eq!(
            match_constraint(&fp2.constraints[0], &item("pub struct S;")),
            Match3::NoMatch,
            "a definite NoMatch sibling must short-circuit all_of to NoMatch"
        );
    }

    #[test]
    fn match3_undefined_propagates_through_any_of_not_collapsing() {
        // any_of([item=fn, body_contains_macro("panic")]) on a struct:
        // item=fn is NoMatch (definite), body_contains_macro is Undefined.
        // any_of(NoMatch, Undefined) MUST be Undefined, NOT NoMatch.
        let fp = crate::Fingerprint::parse(r#"any_of([item = fn, body_contains_macro("panic")])"#)
            .unwrap();
        assert_eq!(
            match_constraint(&fp.constraints[0], &item("pub struct S;")),
            Match3::Undefined,
            "any_of(NoMatch, Undefined) must propagate Undefined, not collapse to NoMatch"
        );

        // A definite Match sibling DOES short-circuit any_of to Match.
        let fp2 =
            crate::Fingerprint::parse(r#"any_of([item = struct, body_contains_macro("panic")])"#)
                .unwrap();
        assert_eq!(
            match_constraint(&fp2.constraints[0], &item("pub struct S;")),
            Match3::Match,
            "a definite Match sibling must short-circuit any_of to Match"
        );
    }
}
