//! Argument parsing for the antigen attribute macros.

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, Lit, LitStr, Path, Token};

/// Arguments to `#[antigen(...)]`.
#[allow(dead_code)] // family/summary/references are captured for validation but
                    // not currently used in macro expansion. They will be used
                    // when the macro emits richer #[doc] forwards or registers
                    // declarations for cross-crate discovery (future ADRs).
pub struct AntigenArgs {
    pub name: String,
    pub fingerprint: String,
    pub family: Option<String>,
    pub summary: Option<String>,
    pub references: Vec<String>,
}

/// Arguments to `#[presents(antigen_type)]`.
pub struct PresentsArgs {
    #[allow(dead_code)]
    pub antigen: Path,
}

/// Arguments to `#[immune(antigen_type, witness = ..., [rationale = ...])]`.
pub struct ImmuneArgs {
    #[allow(dead_code)]
    pub antigen: Path,
    pub witness: Option<Expr>,
    #[allow(dead_code)]
    pub rationale: Option<String>,
}

/// Arguments to `#[descended_from(parent_path)]`.
pub struct DescendedFromArgs {
    #[allow(dead_code)]
    pub parent: Path,
}

// ============================================================================
// AntigenArgs parsing
// ============================================================================

impl Parse for AntigenArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<String> = None;
        let mut fingerprint: Option<String> = None;
        let mut family: Option<String> = None;
        let mut summary: Option<String> = None;
        let mut references: Vec<String> = Vec::new();

        let pairs: Punctuated<MetaPair, Token![,]> =
            input.parse_terminated(MetaPair::parse, Token![,])?;

        for pair in pairs {
            match pair.key.to_string().as_str() {
                "name" => name = Some(pair.expect_string()?),
                "fingerprint" => fingerprint = Some(pair.expect_string()?),
                "family" => family = Some(pair.expect_string()?),
                "summary" => summary = Some(pair.expect_string()?),
                "references" => references = pair.expect_string_array()?,
                other => {
                    return Err(syn::Error::new(
                        pair.key.span(),
                        format!(
                            "unknown #[antigen] field `{other}`; expected one of: \
                                 name, fingerprint, family, summary, references"
                        ),
                    ))
                }
            }
        }

        let name = name
            .ok_or_else(|| syn::Error::new(input.span(), "#[antigen] requires `name = \"...\"`"))?;
        let fingerprint = fingerprint.ok_or_else(|| {
            syn::Error::new(input.span(), "#[antigen] requires `fingerprint = \"...\"`")
        })?;

        Ok(Self {
            name,
            fingerprint,
            family,
            summary,
            references,
        })
    }
}

impl AntigenArgs {
    // TODO(team): error messages currently point to Span::call_site() rather
    // than the offending token. Thread spans through the parser so each
    // validation error points to the EXACT bad token (e.g., the malformed name
    // string literal, not just the macro invocation). This significantly
    // improves the user experience and matches rust-analyzer's diagnostic
    // conventions.
    pub fn validate(&self) -> syn::Result<()> {
        if self.name.is_empty() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[antigen] `name` cannot be empty",
            ));
        }
        if !is_kebab_case(&self.name) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "#[antigen] `name = \"{}\"` must be kebab-case (lowercase with hyphens)",
                    self.name
                ),
            ));
        }
        if self.fingerprint.is_empty() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[antigen] `fingerprint` cannot be empty",
            ));
        }
        Ok(())
    }
}

// ============================================================================
// PresentsArgs parsing
// ============================================================================

impl Parse for PresentsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let antigen: Path = input.parse()?;
        Ok(Self { antigen })
    }
}

// ============================================================================
// ImmuneArgs parsing
// ============================================================================

impl Parse for ImmuneArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let antigen: Path = input.parse()?;
        let mut witness: Option<Expr> = None;
        let mut rationale: Option<String> = None;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "witness" => {
                    witness = Some(input.parse()?);
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[immune] field `{other}`; expected one of: witness, rationale"
                        ),
                    ));
                }
            }
        }

        Ok(Self {
            antigen,
            witness,
            rationale,
        })
    }
}

impl ImmuneArgs {
    pub fn validate(&self) -> syn::Result<()> {
        if self.witness.is_none() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[immune] requires `witness = ...` (a test, proptest, lint reference, \
                 formal-verification proof, or phantom-type construction). \
                 A marker without proof is not a claim.",
            ));
        }
        Ok(())
    }
}

// ============================================================================
// DescendedFromArgs parsing
// ============================================================================

impl Parse for DescendedFromArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parent: Path = input.parse()?;
        Ok(Self { parent })
    }
}

// ============================================================================
// Helpers
// ============================================================================

struct MetaPair {
    key: Ident,
    value: Expr,
}

impl Parse for MetaPair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        Ok(Self { key, value })
    }
}

impl MetaPair {
    fn expect_string(&self) -> syn::Result<String> {
        if let Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) = &self.value
        {
            Ok(s.value())
        } else {
            Err(syn::Error::new_spanned(
                &self.value,
                format!("expected a string literal for `{}`", self.key),
            ))
        }
    }

    fn expect_string_array(&self) -> syn::Result<Vec<String>> {
        if let Expr::Array(arr) = &self.value {
            let mut out = Vec::new();
            for elem in &arr.elems {
                if let Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(s), ..
                }) = elem
                {
                    out.push(s.value());
                } else {
                    return Err(syn::Error::new_spanned(
                        elem,
                        "expected a string literal in references array",
                    ));
                }
            }
            Ok(out)
        } else {
            Err(syn::Error::new_spanned(
                &self.value,
                format!("expected a string array for `{}`", self.key),
            ))
        }
    }
}

fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}

// ============================================================================
// Cross-parser equivalence fixtures
//
// These fixtures define the invariant: for any input the macro side accepts as
// valid, the scan side must produce equivalent semantic content for the four
// overlapping fields (name, fingerprint, family, summary). The same fixture
// table appears in `antigen/src/scan.rs` (ScanAntigenArgs tests) — keeping the
// inputs and expected outputs literally identical is what makes the
// equivalence inspectable.
//
// ATK-001-2 lesson: the brittle string-manipulation parser corrupted
// fingerprints with inner double-quotes silently. Property tests over both
// parsers prevent that class of drift from re-emerging.
//
// When adding a fixture here, add the matching one to scan.rs. When adding a
// new field to the antigen attribute grammar, add fixtures here AND to scan.rs
// to lock the equivalence.
// ============================================================================

/// Fixture tuple shape: `(input, expected_name, expected_fingerprint,
/// expected_family, expected_summary)`.
#[cfg(test)]
type AntigenFixture = (&'static str, &'static str, &'static str, Option<&'static str>, Option<&'static str>);

#[cfg(test)]
const ANTIGEN_PARSER_FIXTURES: &[AntigenFixture] = &[

    // 1. Smoke test: just the two required fields.
    (
        r#"name = "panicking-in-drop", fingerprint = "impl Drop with panic""#,
        "panicking-in-drop",
        "impl Drop with panic",
        None,
        None,
    ),
    // 2. All four fields populated.
    (
        r#"name = "frame-translation", fingerprint = "class enum + meet", family = "semantic-drift", summary = "Polarity inverts at the frame boundary""#,
        "frame-translation",
        "class enum + meet",
        Some("semantic-drift"),
        Some("Polarity inverts at the frame boundary"),
    ),
    // 3. Inner-quoted fingerprint (the ATK-001-2 regression case).
    (
        r#"name = "x", fingerprint = "item: enum, has_method(\"meet\", \"(Self, Self) -> Self\")""#,
        "x",
        r#"item: enum, has_method("meet", "(Self, Self) -> Self")"#,
        None,
        None,
    ),
    // 4. Reordered fields (order-invariance check).
    (
        r#"summary = "S", family = "F", fingerprint = "FP", name = "n""#,
        "n",
        "FP",
        Some("F"),
        Some("S"),
    ),
    // 5. References array present (macro stores; scan ignores; both must
    //    accept without error).
    (
        r#"name = "x", fingerprint = "y", references = ["GAP-1", "DEC-2"]"#,
        "x",
        "y",
        None,
        None,
    ),
    // 6. Multi-line whitespace (tab + newline) — common rustfmt output shape.
    (
        "name = \"multi-line\",\n\tfingerprint = \"shape\",\n\tfamily = \"family\"",
        "multi-line",
        "shape",
        Some("family"),
        None,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;

    #[test]
    fn antigen_parser_accepts_all_fixtures() {
        for (input, exp_name, exp_fp, exp_family, exp_summary) in ANTIGEN_PARSER_FIXTURES {
            let tokens: TokenStream = input
                .parse()
                .unwrap_or_else(|e| panic!("fixture failed to tokenize: {input:?}: {e}"));
            let args = syn::parse2::<AntigenArgs>(tokens)
                .unwrap_or_else(|e| panic!("macro parser rejected fixture {input:?}: {e}"));
            assert_eq!(&args.name, exp_name, "name mismatch for fixture: {input:?}");
            assert_eq!(
                &args.fingerprint, exp_fp,
                "fingerprint mismatch for fixture: {input:?}"
            );
            assert_eq!(
                args.family.as_deref(),
                *exp_family,
                "family mismatch for fixture: {input:?}"
            );
            assert_eq!(
                args.summary.as_deref(),
                *exp_summary,
                "summary mismatch for fixture: {input:?}"
            );
        }
    }

    #[test]
    fn antigen_parser_rejects_missing_name() {
        let tokens: TokenStream = r#"fingerprint = "x""#.parse().unwrap();
        assert!(syn::parse2::<AntigenArgs>(tokens).is_err());
    }

    #[test]
    fn antigen_parser_rejects_missing_fingerprint() {
        let tokens: TokenStream = r#"name = "x""#.parse().unwrap();
        assert!(syn::parse2::<AntigenArgs>(tokens).is_err());
    }

    #[test]
    fn antigen_parser_rejects_unknown_field() {
        let tokens: TokenStream = r#"name = "x", fingerprint = "y", bogus = "z""#.parse().unwrap();
        match syn::parse2::<AntigenArgs>(tokens) {
            Ok(_) => panic!("expected parse to reject unknown field `bogus`"),
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("unknown") && msg.contains("bogus"),
                    "expected unknown-field error mentioning `bogus`, got: {msg}"
                );
            }
        }
    }

    #[test]
    fn validate_rejects_empty_name() {
        let args = AntigenArgs {
            name: String::new(),
            fingerprint: "x".to_string(),
            family: None,
            summary: None,
            references: Vec::new(),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_kebab_name() {
        let args = AntigenArgs {
            name: "FooBar".to_string(),
            fingerprint: "x".to_string(),
            family: None,
            summary: None,
            references: Vec::new(),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn validate_accepts_kebab_name_with_digits() {
        let args = AntigenArgs {
            name: "frame-2-translation".to_string(),
            fingerprint: "x".to_string(),
            family: None,
            summary: None,
            references: Vec::new(),
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn validate_rejects_name_with_double_hyphen() {
        let args = AntigenArgs {
            name: "frame--translation".to_string(),
            fingerprint: "x".to_string(),
            family: None,
            summary: None,
            references: Vec::new(),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn validate_rejects_name_starting_with_hyphen() {
        let args = AntigenArgs {
            name: "-frame".to_string(),
            fingerprint: "x".to_string(),
            family: None,
            summary: None,
            references: Vec::new(),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn immune_parser_requires_witness() {
        let tokens: TokenStream = r"PanickingInDrop".parse().unwrap();
        let args = syn::parse2::<ImmuneArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn immune_parser_accepts_witness_path() {
        let tokens: TokenStream = r"PanickingInDrop, witness = my::module::test_fn"
            .parse()
            .unwrap();
        let args = syn::parse2::<ImmuneArgs>(tokens).unwrap();
        assert!(args.witness.is_some());
        assert!(args.validate().is_ok());
    }

    #[test]
    fn immune_parser_accepts_rationale() {
        let tokens: TokenStream =
            r#"X, witness = my_test, rationale = "checked manually""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ImmuneArgs>(tokens).unwrap();
        assert_eq!(args.rationale.as_deref(), Some("checked manually"));
    }
}
