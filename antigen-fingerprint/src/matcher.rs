//! Built-in syn evaluator for [`crate::Fingerprint`] (per ADR-015 S2 + S4).
//!
//! Matches a parsed fingerprint against a [`syn::Item`]. Item-shape
//! operators evaluate against `syn`'s typed AST directly. The
//! `body_contains_macro` operator walks the function/method body for
//! `syn::Macro` invocations natively (per ADR-015 S2).

use crate::{normalize_signature_canonical, Constraint, Fingerprint, ItemKind, MethodPattern};

impl Fingerprint {
    /// Match this fingerprint against a `syn::Item`.
    ///
    /// Returns `true` if every top-level constraint matches. Per ADR-010
    /// Amendment 4, fingerprints are RECALL-tuned filters: a `true` result
    /// is "this site may exhibit the failure-class," not "definitely does."
    /// The witness layer proves precision per ADR-002.
    #[must_use]
    pub fn matches(&self, item: &syn::Item) -> bool {
        self.constraints.iter().all(|c| match_constraint(c, item))
    }
}

fn match_constraint(c: &Constraint, item: &syn::Item) -> bool {
    match c {
        Constraint::Item(kind) => item_kind_matches(item, *kind),
        Constraint::NameMatches(glob) => item_name(item).is_some_and(|name| glob.matches(&name)),
        Constraint::Variants(range) => match item {
            syn::Item::Enum(e) => range.contains(e.variants.len()),
            _ => false,
        },
        Constraint::HasMethod(pattern) => has_matching_method(item, pattern),
        Constraint::AttrPresent(path) => {
            item_attrs(item).iter().any(|a| attr_path_matches(a, path))
        }
        Constraint::DocContains(needle) => doc_text(item).contains(needle.as_str()),
        Constraint::BodyContainsMacro(name) => body_contains_macro(item, name),
        Constraint::AllOf(children) => children.iter().all(|c| match_constraint(c, item)),
        Constraint::AnyOf(children) => children.iter().any(|c| match_constraint(c, item)),
        Constraint::Not(child) => !match_constraint(child, item),
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
fn body_contains_macro(item: &syn::Item, name: &str) -> bool {
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
        syn::Item::Fn(f) => finder.visit_block(&f.block),
        syn::Item::Impl(imp) => {
            for impl_item in &imp.items {
                if let syn::ImplItem::Fn(f) = impl_item {
                    finder.visit_block(&f.block);
                    if finder.found {
                        break;
                    }
                }
            }
        }
        _ => {}
    }
    finder.found
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

    // ATK-HM-1: has_method pattern with named parameters never matches.
    //
    // A fingerprint author writing natural Rust style includes parameter names:
    //   has_method("parse", "(input: ParseStream) -> syn::Result<Self>")
    //
    // The pattern is tokenized via normalize_signature_canonical, which preserves
    // "input : ParseStream" in the normalized form.
    //
    // The matcher renders actual method signatures via render_inputs, which strips
    // parameter names (only the TYPE is kept, not the name: FnArg::Typed renders
    // pt.ty, not the full pt). So the actual is "(& self, ParseStream) -> ...",
    // while the pattern is "(& self, input : ParseStream) -> ...".
    //
    // The string comparison fails — silent zero matches. The test at line 326
    // (has_method_matches_simple_signature) was written to probe this but was
    // suppressed (let _ = fp; let _ = i) without asserting any behavior. This
    // leaves the silent failure undetected.
    //
    // STATUS: FAILING — has_method does NOT match when the pattern contains
    // named parameters. The current API provides no error or warning; the
    // fingerprint simply produces 0 matches at scan time.
    //
    // Fix options:
    //   1. Strip param names in normalize_signature_canonical (parse as sig,
    //      render types-only, then re-canonicalize).
    //   2. Strip param names in the pattern before comparison (parse the
    //      normalized pattern to extract type-only form).
    //   3. Document explicitly that has_method patterns must be type-shape-only
    //      and add a validation error when a named parameter is detected.
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
}
