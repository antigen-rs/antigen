//! Substrate-witness predicate language — closed combinator grammar
//! over sealed leaf primitives (ADR-019 §M2).
//!
//! ## Grammar
//!
//! ```text
//! predicate := leaf
//!           | all_of([predicate, ...])
//!           | any_of([predicate, ...])
//!           | not(predicate)
//! ```
//!
//! ## Leaf primitives (v0.1 sealed set)
//!
//! - [`Leaf::RatifiedDoc`] — discipline doc exists at path with optional
//!   frontmatter-version floor + anchor + sibling-JSON
//! - [`Leaf::Signers`] — sidecar `signers[]` contains required names,
//!   optionally with roles, optionally against current fingerprint only
//! - [`Leaf::SignedTrailer`] — `git interpret-trailers` reports matching
//!   trailers on commits touching this item
//! - [`Leaf::OraclesComplete`] — listed oracle files exist with status:
//!   complete
//! - [`Leaf::FreshWithinDays`] — most recent signature's date within N
//!   days of audit time
//!
//! ## Closed-set bright-line rule (ADR-019 §Decision + T4-R)
//!
//! Adding a new leaf that invokes external tooling requires the 4-point
//! rule: (1) binary named in leaf source; (2) has own release process;
//! (3) does NOT execute user-supplied code; (4) invocation args fixed in
//! leaf source except for declared substrate-parameters.
//!
//! v0.1 leaves: only [`Leaf::SignedTrailer`] invokes external tooling
//! (`git interpret-trailers`); all four points satisfied.
//!
//! ## Schema invariants enforced at construction
//!
//! - `all_of([])` and `any_of([])` rejected at construction time
//!   (refinement R-A6 — zero-leaf compositions are not meaningful).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Maximum nesting depth for predicate trees.
///
/// Adversarial NFA-11 guard. Predicates deeper than this are rejected at
/// `validate()` time before the recursive `walk()` reaches dangerous
/// stack depth. 64 is far beyond any legitimate predicate (real
/// predicates rarely exceed depth 5) while protecting against crafted
/// sidecars.
pub const MAX_PREDICATE_DEPTH: usize = 64;

/// Substrate-witness predicate AST.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Predicate {
    /// A single leaf primitive.
    Leaf(Leaf),
    /// `all_of([...])` — every child predicate must pass (co-stimulation
    /// per naturalist R-N2; missing signal → anergy).
    AllOf {
        /// Child predicates; all must pass.
        children: Vec<Self>,
    },
    /// `any_of([...])` — at least one child predicate must pass
    /// (redundant pathways per naturalist R-N2; classical-vs-alternative
    /// complement).
    AnyOf {
        /// Child predicates; at least one must pass.
        children: Vec<Self>,
    },
    /// `not(...)` — child must NOT pass (inhibitory checkpoints per
    /// naturalist R-N2; CTLA-4 / PD-1 / Tregs).
    Not {
        /// Child predicate to negate.
        child: Box<Self>,
    },
}

impl Predicate {
    /// Build an `all_of` combinator. Rejects empty child lists.
    ///
    /// # Errors
    ///
    /// Returns [`PredicateParseError::ZeroLeafComposition`] if `children`
    /// is empty.
    pub fn all_of(children: Vec<Self>) -> Result<Self, PredicateParseError> {
        if children.is_empty() {
            return Err(PredicateParseError::ZeroLeafComposition {
                combinator: "all_of",
            });
        }
        Ok(Self::AllOf { children })
    }

    /// Build an `any_of` combinator. Rejects empty child lists.
    ///
    /// # Errors
    ///
    /// Returns [`PredicateParseError::ZeroLeafComposition`] if `children`
    /// is empty.
    pub fn any_of(children: Vec<Self>) -> Result<Self, PredicateParseError> {
        if children.is_empty() {
            return Err(PredicateParseError::ZeroLeafComposition {
                combinator: "any_of",
            });
        }
        Ok(Self::AnyOf { children })
    }

    /// Build a `not` combinator. (Named `not` to match the predicate-language
    /// grammar; this is not the `std::ops::Not` trait method.)
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn not(child: Self) -> Self {
        Self::Not {
            child: Box::new(child),
        }
    }

    /// Build a leaf-only predicate.
    #[must_use]
    pub const fn leaf(leaf: Leaf) -> Self {
        Self::Leaf(leaf)
    }

    /// Walk the predicate tree post-order. Yields every node (combinators
    /// and leaves) for inspection / validation by callers.
    pub fn walk<F: FnMut(&Self)>(&self, f: &mut F) {
        match self {
            Self::Leaf(_) => f(self),
            Self::AllOf { children } | Self::AnyOf { children } => {
                for c in children {
                    c.walk(f);
                }
                f(self);
            }
            Self::Not { child } => {
                child.walk(f);
                f(self);
            }
        }
    }

    /// Count the number of leaf primitives in this predicate tree.
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        let mut count: usize = 0;
        self.walk(&mut |p| {
            if matches!(p, Self::Leaf(_)) {
                count = count.saturating_add(1);
            }
        });
        count
    }

    /// Re-validate semantic invariants on a parsed-from-JSON predicate.
    /// `Predicate::all_of` / `any_of` constructors enforce the no-empty-
    /// children rule at construction, but a deserialized predicate could
    /// carry empty children (raw serde doesn't run the constructor).
    /// Call after deserialization.
    ///
    /// # Errors
    ///
    /// Returns the first [`PredicateParseError::ZeroLeafComposition`]
    /// encountered.
    pub fn validate(&self) -> Result<(), PredicateParseError> {
        // Phase 1: iterative depth check (avoids recursion for NFA-11 guard).
        // Uses an explicit stack of (node, depth) pairs. Must run BEFORE walk()
        // to protect the recursive walk() from stack overflow on deep predicates.
        let mut stack: Vec<(&Self, usize)> = vec![(self, 0)];
        while let Some((node, depth)) = stack.pop() {
            if depth > MAX_PREDICATE_DEPTH {
                return Err(PredicateParseError::NestingDepthExceeded {
                    max_depth: MAX_PREDICATE_DEPTH,
                });
            }
            match node {
                Self::AllOf { children } | Self::AnyOf { children } => {
                    for c in children {
                        stack.push((c, depth + 1));
                    }
                }
                Self::Not { child } => {
                    stack.push((child, depth + 1));
                }
                Self::Leaf(_) => {}
            }
        }

        // Phase 2: content validation via walk() (safe — depth already checked).
        let mut err: Option<PredicateParseError> = None;
        self.walk(&mut |p| {
            if err.is_some() {
                return;
            }
            match p {
                Self::AllOf { children } if children.is_empty() => {
                    err = Some(PredicateParseError::ZeroLeafComposition {
                        combinator: "all_of",
                    });
                }
                Self::AnyOf { children } if children.is_empty() => {
                    err = Some(PredicateParseError::ZeroLeafComposition {
                        combinator: "any_of",
                    });
                }
                Self::Leaf(Leaf::Signers { required, .. }) if required.is_empty() => {
                    err = Some(PredicateParseError::EmptySignersList);
                }
                Self::Leaf(Leaf::OraclesComplete { files }) if files.is_empty() => {
                    err = Some(PredicateParseError::EmptyOraclesList);
                }
                Self::Leaf(Leaf::SignedTrailer { count, .. }) if *count == 0 => {
                    err = Some(PredicateParseError::ZeroTrailerCount);
                }
                Self::Leaf(Leaf::RatifiedDoc { anchor: Some(a), .. }) if a.is_empty() => {
                    err = Some(PredicateParseError::EmptyAnchor);
                }
                Self::Leaf(Leaf::RatifiedDoc {
                    min_version: Some(v),
                    ..
                }) if v.is_empty() => {
                    err = Some(PredicateParseError::EmptyMinVersion);
                }
                _ => {}
            }
        });
        err.map_or(Ok(()), Err)
    }
}

/// The closed set of leaf primitives for v0.1.
///
/// Each variant carries its leaf-specific parameters as named fields.
/// Adding a new leaf is a v0.2+ amendment to ADR-019 that MUST run the
/// 4-point bright-line review (see module docs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum Leaf {
    /// `ratified_doc(path?, min_version?, anchor?, sibling_json?)` —
    /// discipline doc exists; optional frontmatter `version >= min_version`;
    /// optional `anchor` present in the doc; optional sibling JSON
    /// (e.g., `<doc>.attest.json`) parses + matches schema.
    ///
    /// When `path` is absent, the leaf resolves the doc via the
    /// sidecar's `ItemRatification::doc_ref` field (one indirection
    /// allowed; ADR-019 §M2).
    RatifiedDoc {
        /// Explicit doc path. When absent, the leaf resolves via the
        /// sidecar's `ItemRatification::doc_ref` field.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,
        /// Minimum required version (read from doc's frontmatter
        /// `version:` field).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_version: Option<String>,
        /// Optional anchor substring that must be present in the doc
        /// content (heading slug, named section, etc.).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<String>,
        /// `true` to require a sibling `<doc>.attest.json` exists +
        /// parses. Sibling JSON content is opaque to this leaf (other
        /// leaves can validate it via `oracles_complete` or similar).
        #[serde(default)]
        sibling_json: bool,
    },

    /// `signers(required, roles?, against?)` — sidecar `signers[]`
    /// contains all `required` names; optional `roles` map asserts named
    /// signers carry a specific role; `against = "current" | "any"`
    /// (default `"current"`) controls whether stale signatures count.
    Signers {
        /// Required signer names. Set semantics; order doesn't matter.
        required: Vec<String>,
        /// Optional role assertion: signer name → expected role.
        #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
        roles: std::collections::BTreeMap<String, String>,
        /// Currency policy: against the current fingerprint only (default)
        /// or any signature ever recorded for the signer at this item.
        #[serde(default)]
        against: SignerCurrency,
        /// If non-empty, all signers must carry a `SignatureStrength` in this
        /// allow-list; any other strength causes the predicate to fail.
        /// Empty means all strengths are accepted (default).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        signature_allow: Vec<crate::tier::SignatureStrength>,
        /// If set, signers whose strength is below this level produce a
        /// `SignatureTypeBelowPreferred` audit hint (predicate still passes).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        signature_prefer: Option<crate::tier::SignatureStrength>,
    },

    /// `signed_trailer(key, role?, count?)` — git log on commits
    /// touching this item has `count` trailer entries matching `key`,
    /// optionally with the trailer's role tag set to `role`.
    ///
    /// `git interpret-trailers` is the canonical parser (satisfies the
    /// 4-point bright-line rule per ADR-019 §M2).
    SignedTrailer {
        /// Trailer key, e.g., `"Discipline-Verified-By"`.
        key: String,
        /// Optional role-tag constraint (e.g., `"math-researcher"`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        /// Required count of matching trailers; default 1.
        #[serde(default = "default_trailer_count")]
        count: u32,
    },

    /// `oracles_complete(files)` — listed oracle files exist with
    /// `status: complete` in their YAML frontmatter (or sidecar JSON).
    OraclesComplete {
        /// Paths to oracle files, relative to workspace root.
        files: Vec<PathBuf>,
    },

    /// `fresh_within_days(n)` — most recent signature in the sidecar
    /// has a `date` field within `n` days of audit-evaluation time.
    /// Tolerance-claims often use shorter `n` than immunity claims
    /// (tolerance is more accountable; ADR-019 M2 example: 90 days for
    /// tolerance vs 180 for immunity).
    FreshWithinDays {
        /// Maximum age in days.
        days: u32,
    },
}

/// Default for [`Leaf::SignedTrailer::count`].
const fn default_trailer_count() -> u32 {
    1
}

/// Signer currency policy (default for [`Leaf::Signers::against`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignerCurrency {
    /// Only count signatures whose `signed_against_fingerprint` matches
    /// the current item fingerprint. Stale signatures don't count.
    #[default]
    Current,
    /// Count any signature ever recorded for this signer at this item,
    /// regardless of currency. Useful for "ever-signed-by" gates.
    Any,
}

/// Build-time combinator name discriminator (used in error reporting).
pub type CombinatorName = &'static str;

/// Combinator kind — exported for callers that walk the predicate tree
/// generically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// All children must pass.
    AllOf,
    /// At least one child must pass.
    AnyOf,
    /// Child must not pass (logical inversion).
    Not,
}

/// Errors that can occur when constructing or validating a predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateParseError {
    /// An `all_of([])` or `any_of([])` composition is meaningless and
    /// rejected at parse time (refinement R-A6, ADR-019 §M2).
    ZeroLeafComposition {
        /// Which combinator carried the empty child list.
        combinator: CombinatorName,
    },
    /// A `signers(required = [])` leaf is a semantic no-op: it always passes
    /// regardless of signer state, vacuously bypassing identity checks
    /// (adversarial NFA-7). Rejected at the same layer as zero-leaf
    /// compositions per R-A6.
    EmptySignersList,
    /// An `oracles_complete(files = [])` leaf is a semantic no-op: it always
    /// passes (vacuous truth on an empty iterator), vacuously bypassing oracle
    /// checks (adversarial NFA-8). Same class as `EmptySignersList`.
    EmptyOraclesList,
    /// A `signed_trailer(count = 0)` leaf always passes — "require zero
    /// trailers" is vacuously true even with no trailers present (adversarial
    /// NFA-9). Default count is 1; an explicit 0 bypasses the identity check.
    ZeroTrailerCount,
    /// Predicate tree nesting depth exceeds the allowed maximum (adversarial
    /// NFA-11). Deep recursion in `walk()` can stack-overflow on pathologically
    /// nested predicates deserialized from crafted sidecars. Rejected at
    /// `validate()` time before the recursive walk reaches dangerous depth.
    NestingDepthExceeded {
        /// The maximum nesting depth allowed.
        max_depth: usize,
    },
    /// A `ratified_doc(anchor = "")` leaf carries an empty anchor string.
    /// In Rust, `str::contains("")` is always `true`, so an empty anchor
    /// vacuously bypasses the anchor-presence check — any doc passes
    /// regardless of whether it contains the intended section marker
    /// (adversarial NFA-14). Rejected at `validate()` time.
    EmptyAnchor,
    /// A `ratified_doc(min_version = "")` leaf carries an empty min-version
    /// string. `compare_versions(any_version, "")` always returns Greater or
    /// Equal, so an empty `min_version` vacuously passes the version floor check
    /// for any document with a non-empty version field (adversarial NFA-15).
    /// Rejected at `validate()` time.
    EmptyMinVersion,
}

impl std::fmt::Display for PredicateParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroLeafComposition { combinator } => write!(
                f,
                "predicate combinator `{combinator}` has no children; \
                 zero-leaf compositions are rejected at parse time \
                 (ADR-019 refinement R-A6)"
            ),
            Self::EmptySignersList => write!(
                f,
                "signers leaf has an empty `required` list; \
                 an empty-required signers leaf is a semantic no-op \
                 that vacuously bypasses identity checks (ADR-019 R-A6)"
            ),
            Self::EmptyOraclesList => write!(
                f,
                "oracles_complete leaf has an empty `files` list; \
                 an empty-files oracle leaf is a semantic no-op \
                 that vacuously bypasses oracle checks (ADR-019 R-A6)"
            ),
            Self::ZeroTrailerCount => write!(
                f,
                "signed_trailer leaf has `count = 0`; \
                 requiring zero trailers is vacuously true and bypasses \
                 the trailer identity check (ADR-019 R-A6)"
            ),
            Self::NestingDepthExceeded { max_depth } => write!(
                f,
                "predicate nesting depth exceeds maximum of {max_depth}; \
                 deep recursion in walk() can stack-overflow on crafted sidecars \
                 (adversarial NFA-11)"
            ),
            Self::EmptyAnchor => write!(
                f,
                "ratified_doc leaf has an empty anchor string; \
                 str::contains('') is always true and vacuously bypasses the \
                 anchor-presence check (adversarial NFA-14)"
            ),
            Self::EmptyMinVersion => write!(
                f,
                "ratified_doc leaf has an empty min_version string; \
                 compare_versions(any_version, '') is always Greater or Equal and \
                 vacuously bypasses the version floor check (adversarial NFA-15)"
            ),
        }
    }
}

impl std::error::Error for PredicateParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_of_rejects_empty_children() {
        let err = Predicate::all_of(vec![]).unwrap_err();
        assert!(matches!(
            err,
            PredicateParseError::ZeroLeafComposition {
                combinator: "all_of"
            }
        ));
    }

    #[test]
    fn any_of_rejects_empty_children() {
        let err = Predicate::any_of(vec![]).unwrap_err();
        assert!(matches!(
            err,
            PredicateParseError::ZeroLeafComposition {
                combinator: "any_of"
            }
        ));
    }

    #[test]
    fn all_of_accepts_single_child() {
        let leaf = Predicate::leaf(Leaf::FreshWithinDays { days: 90 });
        let pred = Predicate::all_of(vec![leaf]).unwrap();
        assert_eq!(pred.leaf_count(), 1);
    }

    #[test]
    fn nested_predicate_leaf_count_correct() {
        let pred = Predicate::all_of(vec![
            Predicate::leaf(Leaf::FreshWithinDays { days: 90 }),
            Predicate::any_of(vec![
                Predicate::leaf(Leaf::OraclesComplete {
                    files: vec![PathBuf::from("a.md")],
                }),
                Predicate::not(Predicate::leaf(Leaf::Signers {
                    required: vec!["alice".to_string()],
                    roles: std::collections::BTreeMap::new(),
                    against: SignerCurrency::Current,
                    signature_allow: vec![],
                    signature_prefer: None,
                })),
            ])
            .unwrap(),
        ])
        .unwrap();
        assert_eq!(pred.leaf_count(), 3);
    }

    #[test]
    fn validate_rejects_empty_signers_required_list_nfa7() {
        // BUG REGRESSION TEST (adversarial NFA-7): `Signers { required: [] }`
        // is a semantic no-op — it always passes regardless of signer state.
        // This allows bypassing the signer check entirely via a vacuous leaf.
        // The predicate grammar already rejects `all_of([])` and `any_of([])`
        // for the same reason (R-A6); empty `required` must be rejected at the
        // same layer.
        //
        // Constructing via JSON (deserialization bypasses any future constructor
        // guard, so the validate() path is the canonical enforcement point).
        //
        // This test FAILS against the buggy code where validate() does not
        // check for empty Signers.required.
        // Construct via Rust (not JSON) to avoid serde tag-nesting complexity.
        // The key invariant is that validate() catches the empty-required leaf
        // regardless of how the predicate was built — construction or serde.
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec![],
            roles: std::collections::BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let err = pred
            .validate()
            .expect_err("signers leaf with empty required list must be rejected by validate()");
        assert!(
            matches!(err, PredicateParseError::EmptySignersList),
            "expected EmptySignersList, got {err:?}"
        );
    }

    #[test]
    fn validate_catches_deserialized_empty_combinator() {
        // Construct an empty `all_of` directly (bypassing the constructor) by
        // round-tripping through JSON.
        let pred_json = r#"{"kind":"all_of","children":[]}"#;
        let pred: Predicate = serde_json::from_str(pred_json).unwrap();
        let err = pred.validate().unwrap_err();
        assert!(matches!(
            err,
            PredicateParseError::ZeroLeafComposition {
                combinator: "all_of"
            }
        ));
    }

    #[test]
    fn validate_passes_for_well_formed_predicate() {
        let pred =
            Predicate::all_of(vec![Predicate::leaf(Leaf::FreshWithinDays { days: 90 })]).unwrap();
        assert!(pred.validate().is_ok());
    }

    #[test]
    fn signers_currency_defaults_to_current() {
        // Round-trip via JSON without explicit `against` field.
        let json = r#"{"kind":"leaf","leaf":{"name":"signers","required":["alice"]}}"#;
        // serde's tag-on-Predicate works on the outer Predicate; constructing
        // the leaf direct is the cleaner path here.
        let leaf_json = r#"{"name":"signers","required":["alice"]}"#;
        let leaf: Leaf = serde_json::from_str(leaf_json).unwrap();
        match leaf {
            Leaf::Signers { against, .. } => assert_eq!(against, SignerCurrency::Current),
            _ => panic!("expected Signers leaf"),
        }
        // The outer Predicate JSON form is also acceptable but takes different
        // shape; tested in the round_trip test instead.
        let _ = json;
    }

    #[test]
    fn signed_trailer_count_defaults_to_one() {
        let leaf_json = r#"{"name":"signed_trailer","key":"Discipline-Verified-By"}"#;
        let leaf: Leaf = serde_json::from_str(leaf_json).unwrap();
        match leaf {
            Leaf::SignedTrailer { count, .. } => assert_eq!(count, 1),
            _ => panic!("expected SignedTrailer leaf"),
        }
    }

    #[test]
    fn predicate_round_trip_via_serde_json() {
        let pred = Predicate::all_of(vec![
            Predicate::leaf(Leaf::Signers {
                required: vec!["alice".to_string(), "bob".to_string()],
                roles: std::collections::BTreeMap::new(),
                against: SignerCurrency::Current,
                signature_allow: vec![],
                signature_prefer: None,
            }),
            Predicate::leaf(Leaf::FreshWithinDays { days: 180 }),
        ])
        .unwrap();
        let json = serde_json::to_string(&pred).unwrap();
        let parsed: Predicate = serde_json::from_str(&json).unwrap();
        assert_eq!(pred, parsed);
    }

    #[test]
    fn validate_rejects_excessive_nesting_depth_nfa11() {
        // BUG REGRESSION TEST (adversarial NFA-11): `walk()` is recursive.
        // A crafted sidecar with a pathologically nested predicate (thousands
        // of `not(not(not(...)))` levels) would stack-overflow the audit
        // process. validate() now runs an iterative depth check BEFORE the
        // recursive walk() so deeply-nested predicates are rejected safely.
        //
        // Build a predicate at depth MAX_PREDICATE_DEPTH + 1 (one too deep).
        let mut pred = Predicate::leaf(Leaf::FreshWithinDays { days: 1 });
        for _ in 0..=MAX_PREDICATE_DEPTH {
            pred = Predicate::not(pred);
        }
        // This is MAX_PREDICATE_DEPTH + 1 not-layers deep → must be rejected.
        let err = pred
            .validate()
            .expect_err("predicate exceeding max depth must be rejected");
        assert!(
            matches!(
                err,
                PredicateParseError::NestingDepthExceeded {
                    max_depth: MAX_PREDICATE_DEPTH
                }
            ),
            "expected NestingDepthExceeded, got {err:?}"
        );
    }

    #[test]
    fn validate_accepts_predicate_at_max_depth() {
        // A predicate at exactly MAX_PREDICATE_DEPTH nesting must be accepted.
        let mut pred = Predicate::leaf(Leaf::FreshWithinDays { days: 1 });
        for _ in 0..MAX_PREDICATE_DEPTH {
            pred = Predicate::not(pred);
        }
        assert!(
            pred.validate().is_ok(),
            "predicate at exactly max depth must be accepted"
        );
    }

    #[test]
    fn validate_rejects_zero_trailer_count_nfa9() {
        // BUG REGRESSION TEST (adversarial NFA-9): `SignedTrailer { count: 0 }`
        // always passes — "require zero trailers" is vacuously true even with
        // no trailers at all. Same class as NFA-7 (empty required) and NFA-8
        // (empty files). The default is 1; an explicit 0 bypasses the check.
        let pred = Predicate::leaf(Leaf::SignedTrailer {
            key: "Discipline-Verified-By".to_string(),
            role: None,
            count: 0,
        });
        let err = pred
            .validate()
            .expect_err("signed_trailer with count=0 must be rejected by validate()");
        assert!(
            matches!(err, PredicateParseError::ZeroTrailerCount),
            "expected ZeroTrailerCount, got {err:?}"
        );
    }

    #[test]
    fn validate_rejects_empty_oracles_files_list_nfa8() {
        // BUG REGRESSION TEST (adversarial NFA-8): `OraclesComplete { files: [] }`
        // is a semantic no-op — it always passes (vacuous truth on empty iterator).
        // This is the same class as NFA-7 (empty Signers.required). An author
        // who writes `oracles_complete([])` has checked zero oracles but the
        // audit reports full oracle satisfaction.
        let pred = Predicate::leaf(Leaf::OraclesComplete { files: vec![] });
        let err = pred.validate().expect_err(
            "oracles_complete leaf with empty files list must be rejected by validate()",
        );
        assert!(
            matches!(err, PredicateParseError::EmptyOraclesList),
            "expected EmptyOraclesList, got {err:?}"
        );
    }

    #[test]
    fn walk_visits_all_nodes() {
        let pred = Predicate::all_of(vec![
            Predicate::leaf(Leaf::FreshWithinDays { days: 90 }),
            Predicate::not(Predicate::leaf(Leaf::OraclesComplete {
                files: vec![PathBuf::from("a.md")],
            })),
        ])
        .unwrap();
        let mut count = 0;
        pred.walk(&mut |_| count += 1);
        // 2 leaves + 1 not + 1 all_of = 4 nodes
        assert_eq!(count, 4);
    }
}
