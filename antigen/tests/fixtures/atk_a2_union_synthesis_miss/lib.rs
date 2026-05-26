use antigen::antigen;

// ATK-A2-UNION-SYNTHESIS: fingerprint matching against union items is silently skipped.
//
// The synthesis pass (pass 2) calls item_kind_and_target() which returns None for
// syn::Item::Union, causing all union items to be skipped before fingerprint matching.
// This means no fingerprint can ever fire a FingerprintMatch against a union, even
// when the name matches exactly.
//
// This fixture declares an antigen whose fingerprint matches union-named items,
// and a union whose name matches. The synthesis pass should produce 1 FingerprintMatch
// but currently produces 0.

#[antigen(name = "unsafe-union-miss", fingerprint = "name = matches(\"UNSAFE_*\")")]
pub struct UnsafeUnionMiss;

// This struct DOES match and will be found — confirming the fingerprint works.
pub struct UNSAFE_StructSite;

// This union SHOULD match but is silently skipped by synthesis (item_kind_and_target
// returns None for syn::Item::Union).
pub union UNSAFE_Layout {
    pub integer: u64,
    pub bytes: [u8; 8],
}
