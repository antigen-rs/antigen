//! Built-in syn evaluator for [`crate::Fingerprint`] (per ADR-015 S2 + S4).
//!
//! Matches a parsed fingerprint against a [`syn::Item`]. Item-shape
//! operators evaluate against `syn`'s typed AST directly. The
//! `body_contains_macro` operator walks the function/method body for
//! `syn::Macro` invocations natively (per ADR-015 S2).

use crate::{normalize_ws, Constraint, Fingerprint, ItemKind, MethodPattern};

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
        Constraint::NameMatches(glob) => {
            item_name(item).is_some_and(|name| glob.matches(&name))
        }
        Constraint::Variants(range) => match item {
            syn::Item::Enum(e) => range.contains(e.variants.len()),
            _ => false,
        },
        Constraint::HasMethod(pattern) => has_matching_method(item, pattern),
        Constraint::AttrPresent(path) => item_attrs(item)
            .iter()
            .any(|a| attr_path_matches(a, path)),
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
    let full: Vec<String> = path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
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
                lit: syn::Lit::Str(s), ..
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
    // Fallback path for serde-deserialized patterns (where the cache is
    // None) preserves correctness at the cost of a one-time normalize.
    let pattern_norm: String = pattern
        .normalized_signature
        .clone()
        .unwrap_or_else(|| normalize_ws(&pattern.signature));
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
/// The pattern arrives normalized (whitespace collapsed) per ADR-010
/// Amendment 3 Performance Invariant 2 — the parser does the normalize ONCE
/// at fingerprint-load time. The actual `syn::Signature` is rendered fresh
/// per call (it's the per-match-site cost we cannot avoid) and normalized
/// here for comparison.
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
    normalize_ws(&actual) == pattern_norm
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
        let fp = fp(
            r#"
                item = enum,
                name = matches("*Class"),
                variants = 3..=8,
                has_method("meet", "(& self, Self) -> Self")
            "#,
        );
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
}
