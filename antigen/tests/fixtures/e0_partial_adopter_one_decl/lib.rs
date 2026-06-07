// E0 edge fixture — a PARTIAL ADOPTER: one local #[antigen] declaration AND a
// real flagship footgun (get_unchecked). Parsed-as-text.
//
// The adversarial question this pins: under `--bundled-catalog` auto-detect
// (inject only when zero in-tree antigens), this crate has ONE antigen, so the
// bundled catalog is SUPPRESSED — the user explicitly asked for the catalog but
// gets no flagship coverage because they declared a single local class. Does the
// flagship footgun below get caught, or silently missed?

#[antigen(
    name = "local-thing",
    fingerprint = r#"any_of([body_calls("this_specific_local_call")])"#,
    family = "local"
)]
pub struct LocalThing;

// A real flagship footgun — get_unchecked (panic-on-index, Constructable). The
// local antigen's fingerprint does NOT cover this; only the bundled catalog does.
pub fn fast_index(buf: &[u8], i: usize) -> u8 {
    unsafe { *buf.get_unchecked(i) }
}
