//! Item structural digest — a stable content hash of a defended item's source.
//!
//! This is a **distinct concept** from the [`Fingerprint`](crate::Fingerprint)
//! pattern DSL. A `Fingerprint` *recognizes* a failure-class shape across many
//! items (`item = enum, has_method(...)`); a **structural digest** is a single
//! item's identity-hash that *changes when that item's code changes*. The two
//! never mix: the fingerprint answers "does this item belong to failure-class
//! X?"; the digest answers "is this the same item I signed against, or did it
//! drift?".
//!
//! ## Why this exists
//!
//! The substrate-witness machinery (ADR-019) signs an item with a
//! `signed_against_fingerprint`, and audit compares it to the item's
//! `current_fingerprint` for `against = "current"` / `fresh_within_days`
//! staleness detection. Both fields are typed as free-form strings — but until
//! this module, **no canonical producer existed**. An adopter following the
//! documented workflow had no reachable way to obtain the value to sign
//! against (camp's binary-adopter finding 6), so `against = "current"` and
//! `fresh_within_days` were effectively dead features. This module is that
//! producer.
//!
//! ## Stability contract
//!
//! The digest is the input to a *persisted, signed-against* value. It MUST be:
//!
//! - **Deterministic across machines and Rust versions.** We hash with FNV-1a
//!   over a canonicalized token string — NOT [`std::hash::DefaultHasher`],
//!   whose output is explicitly unstable across toolchain versions. A digest
//!   signed today must reproduce byte-for-byte next year.
//! - **Insensitive to formatting.** Whitespace, indentation, and the exact
//!   spelling of comments do not change the digest, because we hash the
//!   `proc_macro2`-canonicalized token stream, not the raw source bytes.
//! - **Insensitive to antigen's own attributes.** Adding or removing
//!   `#[immune]`, `#[presents]`, `#[antigen_tolerance]`, the deferred-defense
//!   family, etc., does NOT change the digest of the defended item. Otherwise
//!   the act of attesting an item would invalidate the attestation — a vicious
//!   circle. Doc comments and non-antigen attributes ARE part of the digest
//!   (they're part of the item's real structure).
//! - **Sensitive to real structural change.** A change to a function body, a
//!   struct field, a signature, or a variant changes the digest.

use proc_macro2::TokenStream;
use quote::ToTokens;

/// FNV-1a 64-bit offset basis.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64-bit prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Compute the FNV-1a 64-bit hash of a byte slice.
///
/// FNV-1a is chosen over [`std::hash::DefaultHasher`] precisely because its
/// algorithm is fixed forever: the same bytes always produce the same digest,
/// independent of the Rust toolchain. That is the stability the substrate-
/// witness sign/audit cycle requires.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// The set of attribute path identifiers that antigen owns and that MUST be
/// excluded from an item's structural digest. Toggling any of these is an
/// attestation action, not a structural change to the defended item, so it
/// must not invalidate a signed-against digest.
const ANTIGEN_OWNED_ATTRS: &[&str] = &[
    // Core attestation macros.
    "antigen",
    "immune",
    "presents",
    "antigen_tolerance",
    "descended_from",
    "crossreactive",
    // Deferred-defense family (ADR-023).
    "anergy",
    "immunosuppress",
    "poxparty",
    "orient",
    // Rollback-as-triage (ADR-026).
    "triage_commit",
    // Mucosal boundary family.
    "mucosal",
    "mucosal_delegate",
    "mucosal_tolerant",
    // Witness / audit classification macros.
    "polyclonal",
    "monoclonal",
    "adcc",
    "clonal",
    "igg",
    "diagnostic",
    // Recurrent-pattern family.
    "itch",
    "recurrence_anchor",
    "crystallize",
    "chronic",
    "saturate",
    "strand",
];

/// Returns `true` if an attribute's path is one antigen owns (and therefore
/// must be excluded from the structural digest).
fn is_antigen_owned_attr(attr: &syn::Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|seg| ANTIGEN_OWNED_ATTRS.contains(&seg.ident.to_string().as_str()))
}

/// Render a `ToTokens` value to its canonical token string, hash it, and format
/// the result as a fixed-width lowercase-hex digest with a version prefix.
///
/// The version prefix (`fnv1a64:`) makes the digest self-describing: if a
/// future sweep upgrades the hash algorithm, old signed-against values remain
/// recognizable and a migration can be staged rather than silently breaking
/// every existing signature.
fn digest_tokens(tokens: &TokenStream) -> String {
    // `TokenStream`'s Display impl is the canonical-spacing renderer:
    // `proc_macro2` inserts a single space between tokens where one is needed
    // and elides it elsewhere, so two source spellings that differ only in
    // whitespace render identically. This is the canonicalization step.
    let canonical = tokens.to_string();
    let hash = fnv1a_64(canonical.as_bytes());
    format!("fnv1a64:{hash:016x}")
}

/// Compute the structural digest of a free-standing item, excluding antigen's
/// own attributes.
///
/// Accepts anything that renders to tokens *and* exposes its outer attributes
/// via the [`HasAttributes`] shim, so the antigen-owned attributes can be
/// stripped before hashing. See the [module docs](self) for the stability
/// contract this upholds.
#[must_use]
pub fn structural_digest<T: HasAttributes>(item: &T) -> String {
    let stripped = item.clone_without_antigen_attrs();
    // Re-render the attribute-stripped clone to tokens and hash.
    let mut ts = TokenStream::new();
    stripped.to_tokens(&mut ts);
    digest_tokens(&ts)
}

/// A `syn` item whose outer attributes can be inspected and rewritten, so the
/// digest can exclude antigen-owned attributes.
///
/// Implemented for the concrete `syn::Item*` node types the scan visitor walks.
/// The blanket strategy (clone, retain non-antigen attrs, re-tokenize) is the
/// same for every node; only the attribute field differs, so the trait carries
/// a single method that returns an attribute-stripped clone ready to tokenize.
pub trait HasAttributes: ToTokens + Clone {
    /// Return a clone of `self` with all antigen-owned outer attributes
    /// removed. The clone is what gets tokenized and hashed.
    #[must_use]
    fn clone_without_antigen_attrs(&self) -> Self;
}

/// Implement [`HasAttributes`] for a `syn` node whose attribute list lives at
/// `self.attrs`.
macro_rules! impl_has_attributes {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl HasAttributes for $ty {
                fn clone_without_antigen_attrs(&self) -> Self {
                    let mut cloned = self.clone();
                    cloned.attrs.retain(|a| !is_antigen_owned_attr(a));
                    cloned
                }
            }
        )+
    };
}

impl_has_attributes!(
    syn::ItemStruct,
    syn::ItemEnum,
    syn::ItemTrait,
    syn::ItemFn,
    syn::ItemType,
    syn::ItemImpl,
    syn::ItemConst,
    syn::ItemStatic,
    syn::ImplItemFn,
    syn::ImplItemConst,
    syn::ImplItemType,
    syn::TraitItemFn,
    syn::TraitItemConst,
    syn::TraitItemType,
    syn::ItemMacro,
    syn::ItemUse,
    syn::ItemExternCrate,
    syn::ItemForeignMod,
    syn::ItemMod,
    syn::ItemTraitAlias,
    syn::ItemUnion,
    syn::ImplItemMacro,
    syn::TraitItemMacro,
);

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn struct_digest(ts: TokenStream) -> String {
        let item: syn::ItemStruct = syn::parse2(ts).expect("parse struct");
        structural_digest(&item)
    }

    fn fn_digest(ts: TokenStream) -> String {
        let item: syn::ItemFn = syn::parse2(ts).expect("parse fn");
        structural_digest(&item)
    }

    #[test]
    fn digest_is_versioned_hex() {
        let d = struct_digest(quote! { struct Foo { x: u8 } });
        assert!(
            d.starts_with("fnv1a64:"),
            "digest must carry version prefix: {d}"
        );
        let hex = d.strip_prefix("fnv1a64:").unwrap();
        assert_eq!(hex.len(), 16, "fnv1a64 digest is 16 hex chars: {hex}");
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "digest body must be lowercase hex: {hex}"
        );
    }

    #[test]
    fn whitespace_does_not_change_digest() {
        // Both spellings carry a trailing comma; the only difference is
        // indentation, newlines, and inter-token spaces. The canonical token
        // renderer normalizes all of those, so the digest is identical.
        //
        // NB: the digest canonicalizes *whitespace*, not token-sequence
        // equivalents like trailing-comma-present vs trailing-comma-absent —
        // those are genuinely distinct token streams. In practice rustfmt
        // pins a consistent comma style, so an adopter's source does not
        // flip-flop between sign-time and audit-time. Whitespace-insensitivity
        // is the guarantee; trailing-comma normalization is explicitly not.
        let compact: syn::ItemStruct =
            syn::parse_str("struct Foo { x: u8, y: u16, }").expect("parse compact");
        let spaced: syn::ItemStruct =
            syn::parse_str("struct Foo {\n    x:   u8,\n    y: u16,\n}").expect("parse spaced");
        assert_eq!(
            structural_digest(&compact),
            structural_digest(&spaced),
            "whitespace must not change the structural digest"
        );
    }

    #[test]
    fn antigen_attrs_do_not_change_digest() {
        let bare = struct_digest(quote! { struct Foo { x: u8 } });
        let with_immune: syn::ItemStruct = syn::parse2(quote! {
            #[immune(SomeAntigen, witness = some_test)]
            struct Foo { x: u8 }
        })
        .expect("parse");
        assert_eq!(
            bare,
            structural_digest(&with_immune),
            "#[immune] must not change the defended item's digest"
        );

        let with_presents: syn::ItemStruct = syn::parse2(quote! {
            #[presents(SomeAntigen)]
            struct Foo { x: u8 }
        })
        .expect("parse");
        assert_eq!(
            bare,
            structural_digest(&with_presents),
            "#[presents] must not change the defended item's digest"
        );
    }

    // FIXED: all 26 antigen macro names are now in ANTIGEN_OWNED_ATTRS;
    // adding any antigen attr to an item does not change its structural digest.
    #[test]
    fn all_antigen_macros_do_not_change_digest() {
        let bare = struct_digest(quote! { struct Foo { x: u8 } });

        // mucosal family
        let with_mucosal: syn::ItemStruct = syn::parse2(quote! {
            #[mucosal(boundary_type = "actix-web")]
            struct Foo { x: u8 }
        })
        .expect("parse");
        assert_eq!(
            bare,
            structural_digest(&with_mucosal),
            "ATK-DIGEST-1: #[mucosal] must not change digest — it is an antigen \
             macro but is NOT in ANTIGEN_OWNED_ATTRS; adding it to a signed item \
             would silently invalidate the signature"
        );

        // polyclonal family (ActiveArgumentDiscard witness class)
        let with_polyclonal: syn::ItemStruct = syn::parse2(quote! {
            #[polyclonal]
            struct Foo { x: u8 }
        })
        .expect("parse");
        assert_eq!(
            bare,
            structural_digest(&with_polyclonal),
            "ATK-DIGEST-1: #[polyclonal] must not change digest — missing from ANTIGEN_OWNED_ATTRS"
        );

        // itch (recurrent family)
        let with_itch: syn::ItemStruct = syn::parse2(quote! {
            #[itch(antigen = "SomeClass", description = "tracked recurrence")]
            struct Foo { x: u8 }
        })
        .expect("parse");
        assert_eq!(
            bare,
            structural_digest(&with_itch),
            "ATK-DIGEST-1: #[itch] must not change digest — missing from ANTIGEN_OWNED_ATTRS"
        );
    }

    #[test]
    fn non_antigen_attrs_do_change_digest() {
        let bare = struct_digest(quote! { struct Foo { x: u8 } });
        let with_derive: syn::ItemStruct = syn::parse2(quote! {
            #[derive(Clone)]
            struct Foo { x: u8 }
        })
        .expect("parse");
        assert_ne!(
            bare,
            structural_digest(&with_derive),
            "#[derive] is real structure and MUST change the digest"
        );
    }

    #[test]
    fn body_change_changes_digest() {
        let a = fn_digest(quote! { fn f() -> u8 { 1 } });
        let b = fn_digest(quote! { fn f() -> u8 { 2 } });
        assert_ne!(a, b, "a body change must change the digest");
    }

    #[test]
    fn signature_change_changes_digest() {
        let a = fn_digest(quote! { fn f(x: u8) -> u8 { x } });
        let b = fn_digest(quote! { fn f(x: u16) -> u16 { x } });
        assert_ne!(a, b, "a signature change must change the digest");
    }

    #[test]
    fn field_change_changes_digest() {
        let a = struct_digest(quote! { struct Foo { x: u8 } });
        let b = struct_digest(quote! { struct Foo { x: u8, y: u8 } });
        assert_ne!(a, b, "adding a field must change the digest");
    }

    #[test]
    fn digest_is_deterministic() {
        let a = struct_digest(quote! { struct Foo { x: u8 } });
        let b = struct_digest(quote! { struct Foo { x: u8 } });
        assert_eq!(a, b, "the same item must always digest identically");
    }

    #[test]
    fn fnv1a_known_vector() {
        // FNV-1a 64-bit of the empty string is the offset basis itself.
        assert_eq!(fnv1a_64(b""), FNV_OFFSET);
        // FNV-1a 64-bit of "a" is a fixed, well-known value across all
        // conformant implementations — pins our algorithm to the spec.
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
    }
}
