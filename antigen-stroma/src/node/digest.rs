//! STEP 2a — the two-digest split (ADR-067 §A.2). CONFIG/OUTPUT applied to the digest.
//!
//! Two DISTINCT types so one field can never name both:
//! - [`IdentityDigest`] — BLAKE3, collision-RESISTANT, signing tier. The integrity half. FNV-1a is
//!   engineer-collidable and NOT admissible for identity (the born-red FNV-collision ATK proves it).
//! - [`ShapeDigest`] — FNV-1a (reused from `antigen-fingerprint`), cheap-recomputable, the
//!   clustering + near-miss + ADR-068 clause-7 BACKDATE key. Strips name; drift-allowed.
//!
//! **DIFFERENT strip-sets** (aristotle A6 / adversarial GAP-3): `IdentityDigest` KEEPS semantic attrs
//! (signing); `ShapeDigest` STRIPS name (clustering/backdate). Do not unify the preimages.

/// Collision-resistant signing digest (BLAKE3).
///
/// Preimage = canonicalized item tokens ONLY (the implementer's lean, pending adr-reviewer): path +
/// cfg are SIBLING identity fields, not folded in, keeping this a pure function of the item's own
/// bytes (recomputable, parity-guardable).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentityDigest(pub [u8; 32]);

impl IdentityDigest {
    /// The collision-resistant signing digest: BLAKE3 over the canonicalized item-token preimage.
    ///
    /// BLAKE3 is the ADR-067 §A.2 identity tier — FNV-1a is engineer-collidable and FORBIDDEN here
    /// (using it would make the stroma engineer-collidable, a security-class silent failure). A pure
    /// function of the preimage bytes: same tokens ⇒ same digest (tamper-evidence requires
    /// reproducibility); distinct preimages ⇒ distinct digests with collision resistance.
    #[must_use]
    pub fn of_tokens(canonical_tokens: &[u8]) -> Self {
        Self(*blake3::hash(canonical_tokens).as_bytes())
    }

    /// The end-to-end identity digest of a parsed item: BLAKE3 over [`canonical_identity_tokens`].
    ///
    /// This is the §4.3 tamper-evidence form — it KEEPS load-bearing antigen attrs in the preimage
    /// (so forging `#[presents]` changes the identity) while STRIPPING pure-annotation antigen attrs
    /// (so toggling `#[diagnostic]` does not). Prefer this over hand-feeding [`Self::of_tokens`] with
    /// caller-canonicalized bytes — the canonicalizer is the seam that decides the strip-set.
    #[must_use]
    pub fn of_item(item: &syn::Item) -> Self {
        Self::of_tokens(&canonical_identity_tokens(item))
    }
}

/// The antigen-owned attrs that are PURE ANNOTATION — documentary marks that do NOT change what the
/// item *claims*, so they are STRIPPED from the identity preimage (toggling one must not churn
/// identity). This is the complement of the load-bearing set within `ANTIGEN_OWNED_ATTRS`.
///
/// **The §4.3 come-apart (the sharpest config/output cut):** the identity digest must be
/// tamper-evident on a forged CLAIM (a grade, a defense-grant, a lineage, a tolerance) yet stable
/// under a pure documentary edit. So the strip-set is NOT "all antigen attrs" (that would make
/// forging `#[presents]` invisible — tamper-evidence defeated) and NOT "no antigen attrs" (that
/// would churn identity on every marker edit). It is exactly the pure half below; everything else
/// antigen-owned is KEPT (the conservative default — keeping a borderline attr only risks
/// identity-churn-on-edit, while stripping a load-bearing one risks an INVISIBLE FORGERY, far worse).
///
/// The universe (`ANTIGEN_OWNED_ATTRS`) is owned by `antigen-fingerprint` under
/// `digest_strip_list_completeness_guard`; this list is the IDENTITY-tier partition of it. A new
/// antigen macro lands in `ANTIGEN_OWNED_ATTRS` first; its load-bearing-ness is decided HERE. The
/// born-red `ATK-FRAME-DIGEST-STRIP` validates the partition (forge a load-bearing attr → identity
/// changes; toggle a pure one → identity stable).
const PURE_ANTIGEN_ATTRS: &[&str] = &[
    // The bare attestation WRAPPERS — containers, not claims.
    "antigen",
    "immune",
    "antigen_generates",
    // Witness-CLASSIFICATION labels — they classify an existing witness, they do not GRANT status.
    "polyclonal",
    "monoclonal",
    "adcc",
    "clonal",
    "igg",
    "diagnostic",
    // The recurrent-pattern documentary family — annotations, not status-claims.
    "itch",
    "recurrence_anchor",
    "crystallize",
    "chronic",
    "saturate",
    "strand",
    "panel",
    "rx",
    "refer",
    "biopsy",
    "ddx",
    "culture",
    "triage",
];

/// The antigen-owned attrs that are LOAD-BEARING — KEPT in the identity preimage.
///
/// Each makes a forgeable CLAIM (a grade, a defense-grant, a lineage, a tolerance/trust grant) a
/// tamper would target, so keeping it means forging one CHANGES the signing digest (tamper-evident).
///
/// Made explicit (rather than left as "everything not pure") so the partition is a CHECKED, complete
/// classification: the `partition_guard` tests assert `PURE ∪ LOAD_BEARING == ANTIGEN_OWNED_ATTRS`
/// with empty intersection — a new antigen macro added to `ANTIGEN_OWNED_ATTRS` FORCES a deliberate
/// load-bearing/pure decision here, never a silent default. (Mirrors `antigen-fingerprint`'s
/// `digest_strip_list_completeness_guard` shape.)
const LOAD_BEARING_ANTIGEN_ATTRS: &[&str] = &[
    // Grade / presentation claims — the magnitude+certainty a tamper would inflate or deflate.
    "presents",
    "dread",
    "aura",
    "red_flag",
    // Witness / attestation MARKERS — an authored claim "this is defended / observed".
    "defended_by",
    // Lineage / cross-reactivity claims — "this derives from / reacts with that".
    "descended_from",
    "crossreactive",
    // Defense GRANTS — they SUPPRESS detection; forging one silences the immune response.
    "anergy",
    "immunosuppress",
    "poxparty",
    "orient",
    // Tolerance / boundary-trust GRANTS — security decisions a tamper would forge to gain trust.
    "antigen_tolerance",
    "mucosal",
    "mucosal_delegate",
    "mucosal_tolerant",
    // Isolation / rollback-authority decisions.
    "quarantine",
    "triage_commit",
];

/// Whether an attribute is a PURE-annotation antigen attr (stripped from the identity preimage).
///
/// Matches the attribute's LAST path segment against [`PURE_ANTIGEN_ATTRS`] — the same last-segment
/// convention `antigen-fingerprint` uses, so `#[antigen::diagnostic]` and `#[diagnostic]` both match.
fn is_pure_antigen_attr(attr: &syn::Attribute) -> bool {
    last_segment_in(attr, PURE_ANTIGEN_ATTRS)
}

/// Whether an attribute is a LOAD-BEARING antigen attr (the §4.3 complement of `is_pure_antigen_attr`).
///
/// A load-bearing attr's presence is a forgeable CLAIM (a grade, a defense/tolerance grant, a
/// lineage) that the identity digest KEEPS so a forge is tamper-evident. Public because an organ
/// reasoning about tamper-surface wants to ask "is this attr security-load-bearing?" directly.
#[must_use]
pub fn is_load_bearing_antigen_attr(attr: &syn::Attribute) -> bool {
    last_segment_in(attr, LOAD_BEARING_ANTIGEN_ATTRS)
}

/// Whether an attribute's LAST path segment is in `set` (the shared antigen-attr-matching convention).
fn last_segment_in(attr: &syn::Attribute, set: &[&str]) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|seg| set.contains(&seg.ident.to_string().as_str()))
}

/// Produce the canonical IDENTITY-token preimage of a parsed item — the §4.3 seam.
///
/// Clones the item, retains every attribute EXCEPT the pure-annotation antigen attrs
/// (`PURE_ANTIGEN_ATTRS`), and re-renders to a canonical token byte string (proc-macro2's `Display`
/// gives a single canonical spacing, so the preimage is formatting-insensitive). Load-bearing antigen
/// attrs and ALL non-antigen attrs survive — a forged `#[presents]` changes these bytes; a toggled
/// `#[diagnostic]` does not. This is the seam [`IdentityDigest::of_item`] and the constitute adapter
/// route through; it is where the strip decision lives (one tracker, not duplicated per-call-site).
#[must_use]
pub fn canonical_identity_tokens(item: &syn::Item) -> Vec<u8> {
    use quote::ToTokens;
    let stripped = strip_pure_antigen_attrs(item);
    stripped.into_token_stream().to_string().into_bytes()
}

/// Clone a `syn::Item` with its top-level pure-annotation antigen attrs removed (load-bearing antigen
/// attrs + all non-antigen attrs retained). The per-variant dispatch reaches each item kind's `attrs`.
fn strip_pure_antigen_attrs(item: &syn::Item) -> syn::Item {
    /// Retain all attrs except pure-antigen ones, in place on a cloned item.
    macro_rules! retain_on {
        ($it:expr) => {{
            let mut cloned = $it.clone();
            cloned.attrs.retain(|a| !is_pure_antigen_attr(a));
            cloned
        }};
    }
    match item {
        syn::Item::Struct(it) => syn::Item::Struct(retain_on!(it)),
        syn::Item::Enum(it) => syn::Item::Enum(retain_on!(it)),
        syn::Item::Union(it) => syn::Item::Union(retain_on!(it)),
        syn::Item::Trait(it) => syn::Item::Trait(retain_on!(it)),
        syn::Item::Type(it) => syn::Item::Type(retain_on!(it)),
        syn::Item::Const(it) => syn::Item::Const(retain_on!(it)),
        syn::Item::Static(it) => syn::Item::Static(retain_on!(it)),
        syn::Item::Fn(it) => syn::Item::Fn(retain_on!(it)),
        syn::Item::Impl(it) => syn::Item::Impl(retain_on!(it)),
        syn::Item::Mod(it) => syn::Item::Mod(retain_on!(it)),
        // Item kinds without a top-level antigen-attr surface: clone unchanged (no pure attr to strip).
        other => other.clone(),
    }
}

/// Fast shape digest (FNV-1a). Reuse `antigen_fingerprint`'s `structural_shape_digest` — name-
/// insensitive, the clustering/backdate key. Strips `ANTIGEN_OWNED_ATTRS` (the maintained strip-list).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeDigest(pub String);

impl ShapeDigest {
    /// The name-insensitive shape digest: delegate to `antigen_fingerprint::structural_shape_digest`.
    ///
    /// Parses `item_source` to a `syn::Item` and dispatches to the fingerprint crate's shape digest,
    /// which strips `ANTIGEN_OWNED_ATTRS` AND normalizes the top-level ident to a placeholder — so
    /// two structurally-identical items with different NAMES share a shape digest (the clustering /
    /// backdate property). We delegate rather than reimplement: the strip-list completeness is guarded
    /// in `antigen-fingerprint` (`digest_strip_list_completeness_guard`), so routing through it keeps
    /// a single tracker. Unparseable source falls through to a raw-token digest (a degraded shape key;
    /// it cannot be name-normalized, so it is name-SENSITIVE — acceptable for the clustering role,
    /// never used for identity).
    #[must_use]
    pub fn of_item(item_source: &str) -> Self {
        use antigen_fingerprint::structural_shape_digest;

        // Dispatch on the parsed item kind — each arm has a `ShapeNormalize` impl that placeholder-
        // normalizes the ident, giving name-insensitivity for free.
        let digest = match syn::parse_str::<syn::Item>(item_source) {
            Ok(syn::Item::Struct(it)) => structural_shape_digest(&it),
            Ok(syn::Item::Enum(it)) => structural_shape_digest(&it),
            Ok(syn::Item::Union(it)) => structural_shape_digest(&it),
            Ok(syn::Item::Trait(it)) => structural_shape_digest(&it),
            Ok(syn::Item::Type(it)) => structural_shape_digest(&it),
            Ok(syn::Item::Const(it)) => structural_shape_digest(&it),
            Ok(syn::Item::Static(it)) => structural_shape_digest(&it),
            Ok(syn::Item::Fn(it)) => structural_shape_digest(&it),
            // Non-ident-bearing items (impl/macro/use/…) and unparseable source: digest the raw
            // bytes. This is name-SENSITIVE (no ident to normalize) — a degraded, conservative shape
            // key, never promoted to the identity tier.
            _ => {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                item_source.hash(&mut h);
                format!("raw:{:016x}", h.finish())
            },
        };
        Self(digest)
    }
}

#[cfg(test)]
mod partition_guard {
    //! The §4.3 completeness guard — the IDENTITY-tier strip partition must EXACTLY cover the
    //! universe `ANTIGEN_OWNED_ATTRS`. Closes a `ParallelStateTrackersDiverge` instance: our
    //! `PURE_ANTIGEN_ATTRS` + `LOAD_BEARING_ANTIGEN_ATTRS` partition the SAME truth the fingerprint
    //! crate's `ANTIGEN_OWNED_ATTRS` enumerates — nothing FORCED them to agree until now.
    //!
    //! When a new antigen macro lands in `ANTIGEN_OWNED_ATTRS` (the fingerprint crate, under its own
    //! `digest_strip_list_completeness_guard`), it is neither pure nor load-bearing HERE until someone
    //! classifies it. This guard makes that classification MANDATORY + LOUD: an unclassified new attr
    //! fails here, not silently. (Mirrors the fingerprint guard's read-both-surfaces-as-text shape.)

    use std::collections::BTreeSet;

    use super::{LOAD_BEARING_ANTIGEN_ATTRS, PURE_ANTIGEN_ATTRS};

    /// Extract the `"name"` string literals from the `ANTIGEN_OWNED_ATTRS` array in the fingerprint
    /// crate's `src/digest.rs` (read as TEXT — the const is private to that crate).
    fn antigen_owned_attrs() -> BTreeSet<String> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../antigen-fingerprint/src/digest.rs");
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

        let decl = "const ANTIGEN_OWNED_ATTRS: &[&str] = &[";
        let start = src.find(decl).unwrap_or_else(|| {
            panic!("`{decl}` not found — the fingerprint strip-list moved/renamed")
        }) + decl.len();
        let body = &src[start..];
        let end = body
            .find("];")
            .expect("unterminated ANTIGEN_OWNED_ATTRS array");
        let body = &body[..end];

        let mut names = BTreeSet::new();
        for line in body.lines() {
            // Skip `//`-comment text (the array's comments never contain a quote).
            let code = line.split("//").next().unwrap_or("");
            let mut rest = code;
            while let Some(open) = rest.find('"') {
                let after = &rest[open + 1..];
                let close = after
                    .find('"')
                    .expect("string literal must close on its line");
                names.insert(after[..close].to_string());
                rest = &after[close + 1..];
            }
        }
        assert!(
            !names.is_empty(),
            "extracted ZERO names from ANTIGEN_OWNED_ATTRS — the parse broke (guard would be vacuous)"
        );
        names
    }

    #[test]
    fn pure_and_load_bearing_are_disjoint() {
        let pure: BTreeSet<&str> = PURE_ANTIGEN_ATTRS.iter().copied().collect();
        let load: BTreeSet<&str> = LOAD_BEARING_ANTIGEN_ATTRS.iter().copied().collect();
        let both: Vec<&&str> = pure.intersection(&load).collect();
        assert!(
            both.is_empty(),
            "an attr is BOTH pure and load-bearing — the partition is ambiguous: {both:?}"
        );
    }

    #[test]
    fn partition_has_no_phantom_names() {
        // Every name WE classify must actually be antigen-owned (no typos, no stale entries).
        let owned = antigen_owned_attrs();
        for name in PURE_ANTIGEN_ATTRS.iter().chain(LOAD_BEARING_ANTIGEN_ATTRS) {
            assert!(
                owned.contains(*name),
                "`{name}` is in the §4.3 partition but NOT in ANTIGEN_OWNED_ATTRS — a phantom \
                 (typo, or a strip-list entry that was removed). Remove it or fix the name."
            );
        }
    }

    #[test]
    fn partition_covers_every_owned_attr() {
        // THE load-bearing assertion: every antigen-owned attr MUST be classified (pure OR
        // load-bearing). An unclassified one is the silent gap — a new macro `canonical_identity_tokens`
        // would KEEP (safe) but UNDELIBERATELY. Force the decision.
        let owned = antigen_owned_attrs();
        let classified: BTreeSet<String> = PURE_ANTIGEN_ATTRS
            .iter()
            .chain(LOAD_BEARING_ANTIGEN_ATTRS)
            .map(|s| (*s).to_string())
            .collect();
        let unclassified: Vec<&String> = owned.difference(&classified).collect();
        assert!(
            unclassified.is_empty(),
            "antigen-owned attrs are UNCLASSIFIED in the §4.3 identity partition: {unclassified:?}. \
             A new macro landed in ANTIGEN_OWNED_ATTRS without a load-bearing/pure decision here. \
             Add each to LOAD_BEARING_ANTIGEN_ATTRS (a forgeable claim — the §4.3 default) or \
             PURE_ANTIGEN_ATTRS (documentary). Never leave it to default silently."
        );
    }
}
