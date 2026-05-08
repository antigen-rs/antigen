//! Source-AST scanner for antigen declarations.
//!
//! Walks Rust source files in a workspace, extracts `#[antigen]`, `#[presents]`,
//! `#[immune]`, and `#[descended_from]` attribute invocations, and produces a
//! [`ScanReport`] suitable for further analysis or rendering.
//!
//! This module is the engine behind `cargo antigen scan`. It's also exposed for
//! custom integrations (e.g., a project's own CI harness, IDE plugins, or
//! programmatic audit tooling).
//!
//! ## Status (v0.0.1)
//!
//! Initial implementation. Discovers attribute invocations and matches presentations
//! against immunities at the same item level. Future versions will add:
//!
//! - `#[descended_from]` propagation walks
//! - Cross-crate antigen declaration discovery
//! - Witness validation (delegating to clippy/kani/proptest as appropriate)
//! - Fingerprint structural matching against unmarked code (the
//!   recognition-not-yet-marked half of scan)
//! - Performance optimizations (incremental scan, parallel file walks)
//!
//! ## Known v1 limitations (easy wins for the JBD team)
//!
//! Search this file for `TODO(team)` to find specific spots that the antigen JBD
//! team can sharpen quickly without redesigning anything.
//!
//! 1. **Line numbers are heuristic** — see `ScanVisitor::line_of_attr` (private);
//!    finds the FIRST occurrence of the attribute name in the source, not the
//!    actual span of the specific invocation. Replace with
//!    `syn::spanned::Spanned::span().start().line` once syn's span info is
//!    reliable on the team's toolchain. (W4 territory: span-aware error
//!    messages on the macro side, with the scan-side line story landing in a
//!    follow-up.)
//! 2. **Witness validation is presence-only** — the scan records the witness
//!    identifier but doesn't verify it resolves to a real function or that the
//!    function actually exercises behavior matching the antigen. The audit
//!    subcommand (sweep A2/A3) lifts this; W7 sharpens witness-validity tier
//!    semantics for v0.1.
//!
//! W3 (sweep A2) replaced the prior 20-line proximity heuristic in
//! [`ScanReport::unaddressed_presentations`] with structural item-identity
//! matching via [`ItemTarget`] + [`ItemTarget::addresses`]. See those types.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use syn::parse::Parse;
use syn::visit::Visit;
use walkdir::WalkDir;

// ============================================================================
// Scan-side attribute argument parsers
//
// These mirror the proc-macro-side parsers in antigen-macros but live in a
// regular (non-proc-macro) crate so they can be used at scan time. Both must
// produce identical results for the same input — the canonical representation
// is `syn::LitStr::value()`, which correctly unescapes string literal content.
//
// The proc-macro crate cannot be re-exported as a library (proc-macro = true
// crates export only their macro entry points), so we duplicate the parsing
// logic here. Any change to the attribute grammar must be reflected in both.
// ============================================================================

/// Scan-time parse of `#[antigen(name = "...", fingerprint = "...", ...)]`.
struct ScanAntigenArgs {
    name: String,
    fingerprint: Option<String>,
    family: Option<String>,
    summary: Option<String>,
}

impl Parse for ScanAntigenArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Ident, LitStr, Token};
        let mut name: Option<String> = None;
        let mut fingerprint: Option<String> = None;
        let mut family: Option<String> = None;
        let mut summary: Option<String> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name = Some(lit.value());
                }
                "fingerprint" => {
                    let lit: LitStr = input.parse()?;
                    fingerprint = Some(lit.value());
                }
                "family" => {
                    let lit: LitStr = input.parse()?;
                    family = Some(lit.value());
                }
                "summary" => {
                    let lit: LitStr = input.parse()?;
                    summary = Some(lit.value());
                }
                "references" => {
                    // Consume the array without storing (not used in scan output yet).
                    let _arr: syn::ExprArray = input.parse()?;
                }
                _ => {
                    // Unknown field: consume the value expression and continue.
                    let _: syn::Expr = input.parse()?;
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            name: name.unwrap_or_default(),
            fingerprint,
            family,
            summary,
        })
    }
}

/// Scan-time parse of `#[immune(AntigenType, witness = expr, ...)]`.
struct ScanImmuneArgs {
    antigen_type: String,
    witness: String,
}

impl Parse for ScanImmuneArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Ident, Path, Token};
        // First token is the antigen path.
        let antigen_path: Path = input.parse()?;
        let antigen_type = antigen_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let mut witness = String::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let val: syn::Expr = input.parse()?;
            if key == "witness" {
                // Render the witness expression as its token string — this is the
                // identifier or path the user wrote, e.g. `my_test_fn` or
                // `clippy::no_panic_in_drop`. We use `quote::ToTokens` to get
                // a canonical rendering without depending on string heuristics.
                use quote::ToTokens;
                witness = val.to_token_stream().to_string();
            }
            // rationale and other fields: consume silently.
        }

        Ok(Self {
            antigen_type,
            witness,
        })
    }
}

/// A single antigen declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigenDeclaration {
    /// The kebab-case antigen name from `#[antigen(name = "...")]`.
    pub name: String,
    /// The Rust type name the attribute is applied to (e.g., `PanickingInDrop`).
    pub type_name: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number of the antigen attribute.
    pub line: usize,
    /// Optional family classification (e.g., "boundary-violation").
    pub family: Option<String>,
    /// Optional human-readable summary.
    pub summary: Option<String>,
    /// Optional fingerprint string (uninterpreted in v1; ADR-010 grammar
    /// implementation lands in a future sweep).
    pub fingerprint: Option<String>,
}

/// Identity of the Rust item that an antigen-related attribute is applied to.
///
/// W3 (sweep A2): replaces the old proximity heuristic in
/// `unaddressed_presentations` with structural matching. `Presentation` and
/// `Immunity` carry an `item_target` that names the *item they live on*; two
/// declarations address each other if and only if their item targets are
/// equal (and they're in the same file and reference the same antigen).
///
/// The variants mirror the visitor entry points:
/// - `Struct`, `Enum`, `Trait`: top-level type declarations
/// - `Fn`: a free function
/// - `Impl`: an `impl ... for ...` or inherent `impl ...` block
/// - `ImplFn`: a method inside an impl block (with its enclosing impl
///   target captured so two methods named `drop` on different types
///   don't collide)
/// - `Unknown`: visitor fallback for shapes we don't yet model (e.g.,
///   free constants); kept rather than asserted so scans never panic on
///   third-party code with novel item shapes.
///
/// `trait_path` on `Impl`/`ImplFn` is the trait being implemented (e.g.,
/// `Drop` from `impl Drop for X`); `None` for inherent impls. The path is
/// captured as a string after canonical rendering — full-path equality is
/// W3's invariant, but A3 cross-crate matching may need richer
/// representation later.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemTarget {
    /// A top-level struct declaration. Holds the struct identifier.
    Struct(String),
    /// A top-level enum declaration. Holds the enum identifier.
    Enum(String),
    /// A top-level trait declaration. Holds the trait identifier.
    Trait(String),
    /// A free function. Holds the function identifier.
    Fn(String),
    /// A type alias declaration (`type Foo = ...;`). Holds the alias
    /// identifier. ATK-W3-005: without this, type aliases fall back to
    /// `Unknown`, and two unrelated Unknown items collide on equality.
    TypeAlias(String),
    /// An `impl ... for ...` or inherent `impl ...` block. The trait
    /// portion is `None` for inherent impls and `Some(rendered_path)`
    /// otherwise. Methods inside the impl carry an `ImplFn` target that
    /// references the same `target_type` and `trait_path`.
    Impl {
        /// The trait being implemented, rendered to its canonical token
        /// string. `None` for inherent impls (no trait).
        trait_path: Option<String>,
        /// The implementing type, rendered to its canonical token string.
        target_type: String,
    },
    /// A method inside an impl block. `target_type` and `trait_path`
    /// mirror the enclosing `Impl` target so that two methods with the
    /// same name on different types do not collide for matching purposes.
    ImplFn {
        /// Trait of the enclosing impl, if any.
        trait_path: Option<String>,
        /// Type of the enclosing impl.
        target_type: String,
        /// The method name.
        fn_name: String,
    },
    /// A method declared inside a `trait` definition. Pairs with
    /// `ImplFn { trait_path: Some(trait_name), fn_name, .. }` — the
    /// presents-on-trait-method + immune-on-impl-method pattern is one
    /// of the W3 README's adversarial cases. Holds the trait name and
    /// method name; matching bridges `TraitFn` ↔ `ImplFn` explicitly.
    TraitFn {
        /// The enclosing trait identifier.
        trait_name: String,
        /// The method name.
        fn_name: String,
    },
    /// Visitor fallback for shapes we don't yet model (e.g., free
    /// constants, modules with attribute-bearing macro-expansion).
    /// Kept rather than asserted so scans never panic on third-party
    /// code with novel item shapes. Carries the source line so that two
    /// Unknown items at different positions are not falsely equal —
    /// ATK-W3-005 caught the previous unit-variant form colliding on
    /// equality across unrelated items. The line is a best-effort
    /// discriminator; perfect identity for unhandled shapes requires
    /// per-shape visitor methods (deferred).
    Unknown {
        /// Best-effort line number; used as a tie-breaker for equality.
        line: usize,
    },
}

impl ItemTarget {
    /// Best-effort short name for diagnostic output. Not used for matching.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Struct(n) | Self::Enum(n) | Self::Trait(n) | Self::Fn(n) | Self::TypeAlias(n) => {
                n.clone()
            }
            Self::Impl {
                trait_path: Some(t),
                target_type,
            } => format!("impl {t} for {target_type}"),
            Self::Impl {
                trait_path: None,
                target_type,
            } => format!("impl {target_type}"),
            Self::ImplFn {
                trait_path: Some(t),
                target_type,
                fn_name,
            } => format!("<{target_type} as {t}>::{fn_name}"),
            Self::ImplFn {
                trait_path: None,
                target_type,
                fn_name,
            } => format!("{target_type}::{fn_name}"),
            Self::TraitFn {
                trait_name,
                fn_name,
            } => format!("trait {trait_name}::{fn_name}"),
            Self::Unknown { line } => format!("<unknown at line {line}>"),
        }
    }

    /// Whether this item-target addresses another for the purposes of the
    /// presents+immune match. The relation is reflexive and symmetric.
    ///
    /// W3 (sweep A2) — the "addresses" relation is wider than strict
    /// equality, per the A2 README's matching rules:
    ///
    /// - Same kind, same name (Struct/Enum/Trait/Fn/TypeAlias) → match.
    /// - Two `Impl` blocks for the same base type (regardless of trait
    ///   being implemented) → match. Generics are normalised away so
    ///   `Container<T>` and `Container<i32>` share a base type.
    /// - Two `ImplFn` items on the same base type with the same method
    ///   name → match (regardless of whether the impls implement the
    ///   same trait).
    /// - `TraitFn(T, f)` ↔ `ImplFn { trait_path: Some(T), fn_name: f, .. }`
    ///   → match. Handles the README's case (a): presents on a trait
    ///   method, immune on the impl method.
    /// - `Unknown` never matches anything — never a false negative on
    ///   unclassified items (per ATK-W3-005's premise).
    /// - Heterogeneous variants don't match.
    ///
    /// The relaxation is intentional: false positives in the matcher
    /// surface as unaddressed presentations the user can investigate;
    /// false negatives silently green-light a vulnerability. Err on the
    /// side of matching legitimate presents+immune pairings.
    #[must_use]
    #[allow(
        clippy::match_same_arms,
        reason = "the explicit `Unknown` arm is the load-bearing invariant — \
                  Unknown items must NEVER match anything, including each other. \
                  Keeping it explicit (even though it duplicates the `_` wildcard's \
                  body) makes the invariant readable and refactor-safe."
    )]
    pub fn addresses(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown { .. }, _) | (_, Self::Unknown { .. }) => false,
            (Self::Struct(a), Self::Struct(b))
            | (Self::Enum(a), Self::Enum(b))
            | (Self::Trait(a), Self::Trait(b))
            | (Self::Fn(a), Self::Fn(b))
            | (Self::TypeAlias(a), Self::TypeAlias(b)) => a == b,
            (
                Self::Impl {
                    target_type: t1, ..
                },
                Self::Impl {
                    target_type: t2, ..
                },
            ) => normalize_type_name(t1) == normalize_type_name(t2),
            (
                Self::ImplFn {
                    target_type: t1,
                    fn_name: f1,
                    ..
                },
                Self::ImplFn {
                    target_type: t2,
                    fn_name: f2,
                    ..
                },
            ) => normalize_type_name(t1) == normalize_type_name(t2) && f1 == f2,
            (
                Self::TraitFn {
                    trait_name,
                    fn_name: tf,
                },
                Self::ImplFn {
                    trait_path: Some(t),
                    fn_name: imf,
                    ..
                },
            )
            | (
                Self::ImplFn {
                    trait_path: Some(t),
                    fn_name: imf,
                    ..
                },
                Self::TraitFn {
                    trait_name,
                    fn_name: tf,
                },
            ) => trait_name == t && tf == imf,
            (
                Self::TraitFn {
                    trait_name: t1,
                    fn_name: f1,
                },
                Self::TraitFn {
                    trait_name: t2,
                    fn_name: f2,
                },
            ) => t1 == t2 && f1 == f2,
            _ => false,
        }
    }
}

/// Strip generic parameters from a `quote::ToTokens`-rendered type name.
/// `"Container < T >"` → `"Container"`. Used for impl-block matching so
/// that `impl<T> Container<T>` and `impl Container<i32>` share an
/// addressable identity at the type level.
fn normalize_type_name(rendered: &str) -> String {
    let s = rendered.trim();
    s.find('<')
        .map_or_else(|| s.to_string(), |idx| s[..idx].trim().to_string())
}

/// A `#[presents(antigen_type)]` declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    /// The antigen type referenced (last path segment, e.g., `PanickingInDrop`).
    pub antigen_type: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated (impl, fn, struct, etc.).
    pub item_kind: String,
    /// Item identity for structural matching against `Immunity`. W3 (sweep A2).
    pub item_target: ItemTarget,
}

/// An `#[immune(antigen_type, witness = ...)]` declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Immunity {
    /// The antigen type referenced.
    pub antigen_type: String,
    /// The witness expression as a string (validated lazily).
    pub witness: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural matching against `Presentation`. W3 (sweep A2).
    pub item_target: ItemTarget,
}

/// Aggregate result of scanning a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanReport {
    /// All discovered antigen declarations.
    pub antigens: Vec<AntigenDeclaration>,
    /// All discovered `#[presents]` sites.
    pub presentations: Vec<Presentation>,
    /// All discovered `#[immune]` sites.
    pub immunities: Vec<Immunity>,
    /// Files scanned successfully.
    pub files_scanned: usize,
    /// Files that failed to parse.
    pub parse_failures: Vec<(PathBuf, String)>,
}

/// A presentation that has no matching immunity declaration on the same item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaddressedPresentation {
    /// The presentation itself.
    pub presentation: Presentation,
    /// True if the antigen referenced is found in the scan report.
    pub antigen_known: bool,
}

impl ScanReport {
    /// Find presentations that lack a corresponding immunity declaration.
    ///
    /// W3 (sweep A2) — structural item-identity matching. A `Presentation`
    /// and an `Immunity` "address each other" when:
    ///
    /// - they reference the same `antigen_type`, AND
    /// - they're in the same source file, AND
    /// - their `item_target` values are equal (i.e., they're applied to
    ///   the same Rust item).
    ///
    /// This replaces the pre-W3 20-line proximity heuristic, which produced
    /// false positives in multi-impl files (immunity on `impl X` matched
    /// presentation on `impl Y` if their attributes happened to be within
    /// 20 lines) and false negatives when long doc-comments separated paired
    /// declarations on the same item.
    ///
    /// Cross-file matching remains out of scope here — different items can
    /// share names across modules, and the structural identity of an
    /// "item" extends to its containing module path. That's A3 territory
    /// (cross-crate scan + `#[descended_from]` propagation).
    #[must_use]
    pub fn unaddressed_presentations(&self) -> Vec<UnaddressedPresentation> {
        let known_antigens: std::collections::HashSet<&str> =
            self.antigens.iter().map(|a| a.type_name.as_str()).collect();

        let mut result = Vec::new();
        for p in &self.presentations {
            let has_matching_immunity = self.immunities.iter().any(|i| {
                i.antigen_type == p.antigen_type
                    && i.file == p.file
                    && i.item_target.addresses(&p.item_target)
            });
            if !has_matching_immunity {
                result.push(UnaddressedPresentation {
                    presentation: p.clone(),
                    antigen_known: known_antigens.contains(p.antigen_type.as_str()),
                });
            }
        }
        result
    }

    /// Total count of antigen-related declarations found.
    #[must_use]
    pub fn total_declarations(&self) -> usize {
        self.antigens.len() + self.presentations.len() + self.immunities.len()
    }
}

/// Scan a directory tree, reading every `.rs` file and extracting antigen
/// declarations.
///
/// `excluded_dirs` is a list of directory names (not full paths) to skip during
/// the walk; the default is `["target", ".git", "node_modules"]` if `None` is
/// passed.
pub fn scan_workspace(root: &Path, excluded_dirs: Option<&[&str]>) -> std::io::Result<ScanReport> {
    let default_exclusions = ["target", ".git", "node_modules"];
    let exclusions = excluded_dirs.unwrap_or(&default_exclusions);

    let mut report = ScanReport::default();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !exclusions.iter().any(|x| *x == name)
            } else {
                true
            }
        })
    {
        let Ok(entry) = entry else { continue };

        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };

        match syn::parse_file(&content) {
            Ok(file) => {
                let mut visitor = ScanVisitor {
                    file_path: entry.path().to_path_buf(),
                    source: &content,
                    report: &mut report,
                    impl_stack: Vec::new(),
                    trait_stack: Vec::new(),
                };
                visitor.visit_file(&file);
                report.files_scanned += 1;
            }
            Err(e) => {
                report
                    .parse_failures
                    .push((entry.path().to_path_buf(), e.to_string()));
            }
        }
    }

    Ok(report)
}

/// AST visitor that extracts antigen-related attributes.
struct ScanVisitor<'a> {
    file_path: PathBuf,
    source: &'a str,
    report: &'a mut ScanReport,
    /// Context stack for nested items. The current top of stack is the
    /// enclosing-impl context for any `visit_impl_item_fn` call — so that
    /// a method's `ItemTarget::ImplFn` knows which impl block it lives in.
    /// W3 (sweep A2): structural item-identity tracking.
    impl_stack: Vec<(Option<String>, String)>,
    /// Context stack for nested traits — analogous to `impl_stack`, but
    /// for `visit_trait_item_fn` so trait methods carry the enclosing
    /// trait identifier in `ItemTarget::TraitFn`.
    trait_stack: Vec<String>,
}

impl ScanVisitor<'_> {
    /// Compute 1-indexed line number for a span by counting newlines in source up
    /// to the span's start.
    ///
    /// TODO(team): currently returns the FIRST occurrence of the attribute name
    /// in the file, regardless of which instance is being processed. This means
    /// multi-instance scenarios report the same line number for every instance.
    /// Replace with a real span-tracking approach: pass the `&syn::Attribute` in
    /// and use `syn::spanned::Spanned::span().start().line` (verify it returns
    /// usable line numbers on the team's stable toolchain; if not, walk byte
    /// offsets via `proc_macro2::Span::byte_range` once that's stable).
    fn line_of_attr(&self, attr_name: &str) -> usize {
        // Heuristic: find the first occurrence of the attribute name in the source.
        for (i, line) in self.source.lines().enumerate() {
            if line.contains(&format!("#[{attr_name}")) {
                return i + 1;
            }
        }
        0
    }

    fn extract_antigen(&mut self, item: &syn::ItemStruct, attr: &syn::Attribute) {
        let type_name = item.ident.to_string();
        let line = self.line_of_attr("antigen");

        if let syn::Meta::List(list) = &attr.meta {
            match syn::parse2::<ScanAntigenArgs>(list.tokens.clone()) {
                Ok(args) => {
                    self.report.antigens.push(AntigenDeclaration {
                        name: args.name,
                        type_name,
                        file: self.file_path.clone(),
                        line,
                        family: args.family,
                        summary: args.summary,
                        fingerprint: args.fingerprint,
                    });
                }
                Err(_) => {
                    // Malformed attribute: record with empty name so scan output
                    // surfaces the file for investigation rather than silently skipping.
                    self.report.antigens.push(AntigenDeclaration {
                        name: String::new(),
                        type_name,
                        file: self.file_path.clone(),
                        line,
                        family: None,
                        summary: None,
                        fingerprint: None,
                    });
                }
            }
        }
    }

    fn extract_presents(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        let antigen_type = if let syn::Meta::List(list) = &attr.meta {
            // Parse the body as a `syn::Path` rather than splitting the
            // `quote::ToTokens` rendering on `::`. The string form contains
            // whitespace artifacts (`" my_crate :: Foo "`) that the prior
            // tactical-fix code recovered from with a `.trim()` — but the
            // structural fix is to never produce the string in the first
            // place. ATK-A2-001's pre-W3 hotfix landed in commit b9440b2;
            // this is the W3 structural form. Path's last segment is the
            // bare type name regardless of qualifier.
            match syn::parse2::<syn::Path>(list.tokens.clone()) {
                Ok(path) => path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default(),
                Err(e) => {
                    self.report.parse_failures.push((
                        self.file_path.clone(),
                        format!("malformed #[presents] attribute: {e}"),
                    ));
                    return;
                }
            }
        } else {
            return;
        };
        let line = self.line_of_attr("presents");
        self.report.presentations.push(Presentation {
            antigen_type,
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
        });
    }

    fn extract_immune(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        if let syn::Meta::List(list) = &attr.meta {
            // TODO(team): witness validation is presence-only. The audit subcommand
            // (sweep A2/A3) should: (1) resolve the witness identifier to an item
            // in the workspace, (2) verify it's a #[test], proptest!, or recognized
            // delegated tool reference, (3) optionally invoke it via cargo test and
            // verify it asserts the expected property. Currently we just record
            // the witness expression verbatim.
            let (antigen_type, witness) = match syn::parse2::<ScanImmuneArgs>(list.tokens.clone()) {
                Ok(args) => (args.antigen_type, args.witness),
                Err(e) => {
                    // Malformed #[immune] args: record a parse failure rather
                    // than silently inserting a ghost immunity record with empty
                    // antigen_type and witness. A ghost record would pass
                    // WitnessStatus::Missing detection only if the empty-string
                    // check fires, and would produce a misleading "0 unaddressed
                    // presentations" result. ADR-005: every trust boundary requires
                    // a validation check; malformed immunity claims are not claims.
                    self.report.parse_failures.push((
                        self.file_path.clone(),
                        format!("malformed #[immune] attribute: {e}"),
                    ));
                    return;
                }
            };
            let line = self.line_of_attr("immune");
            self.report.immunities.push(Immunity {
                antigen_type,
                witness,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn check_attrs(&mut self, attrs: &[syn::Attribute], item_kind: &str, item_target: &ItemTarget) {
        for attr in attrs {
            if attr.path().is_ident("presents") {
                self.extract_presents(attr, item_kind, item_target.clone());
            } else if attr.path().is_ident("immune") {
                self.extract_immune(attr, item_kind, item_target.clone());
            }
        }
    }
}

/// Render a `syn::Type` to its canonical token-stream string. Used to
/// extract a string identifier for `impl Trait for Type` blocks. The
/// rendering normalizes whitespace via `quote::ToTokens`. For W3 we only
/// need a stable string for equality matching — A3 cross-crate work will
/// likely want a richer canonical form (e.g., resolved module paths).
fn render_type(ty: &syn::Type) -> String {
    use quote::ToTokens;
    ty.to_token_stream().to_string()
}

/// Render a `syn::Path` similarly. Used for the trait portion of
/// `impl Trait for Type` so that `Drop` and `core::ops::Drop` produce
/// distinct strings (which is correct — they're different items in
/// Rust's name resolution, even when they alias).
fn render_path(path: &syn::Path) -> String {
    use quote::ToTokens;
    path.to_token_stream().to_string()
}

impl<'ast> Visit<'ast> for ScanVisitor<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        for attr in &item.attrs {
            if attr.path().is_ident("antigen") {
                self.extract_antigen(item, attr);
            }
        }
        let target = ItemTarget::Struct(item.ident.to_string());
        self.check_attrs(&item.attrs, "struct", &target);
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let trait_path = item.trait_.as_ref().map(|(_, path, _)| render_path(path));
        let target_type = render_type(&item.self_ty);
        let target = ItemTarget::Impl {
            trait_path: trait_path.clone(),
            target_type: target_type.clone(),
        };
        self.check_attrs(&item.attrs, "impl", &target);
        // Push impl context so visit_impl_item_fn can build ImplFn targets.
        self.impl_stack.push((trait_path, target_type));
        syn::visit::visit_item_impl(self, item);
        self.impl_stack.pop();
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let target = ItemTarget::Fn(item.sig.ident.to_string());
        self.check_attrs(&item.attrs, "fn", &target);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let target = self.impl_stack.last().map_or_else(
            || ItemTarget::Fn(item.sig.ident.to_string()),
            |(trait_path, target_type)| ItemTarget::ImplFn {
                trait_path: trait_path.clone(),
                target_type: target_type.clone(),
                fn_name: item.sig.ident.to_string(),
            },
        );
        self.check_attrs(&item.attrs, "impl_fn", &target);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let target = ItemTarget::Trait(item.ident.to_string());
        self.check_attrs(&item.attrs, "trait", &target);
        // Push trait context so visit_trait_item_fn produces TraitFn targets
        // identifying the enclosing trait.
        self.trait_stack.push(item.ident.to_string());
        syn::visit::visit_item_trait(self, item);
        self.trait_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let target = self.trait_stack.last().map_or_else(
            || ItemTarget::Fn(item.sig.ident.to_string()),
            |trait_name| ItemTarget::TraitFn {
                trait_name: trait_name.clone(),
                fn_name: item.sig.ident.to_string(),
            },
        );
        self.check_attrs(&item.attrs, "trait_fn", &target);
        syn::visit::visit_trait_item_fn(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        // Type aliases (`type Foo = ...;`) carry attributes too. ATK-W3-005:
        // without this handler, attributes on type aliases would fall back
        // to ItemTarget::Unknown, and two unrelated Unknown items collide
        // on equality. Tracking the alias name keeps each alias as its own
        // distinct match target.
        let target = ItemTarget::TypeAlias(item.ident.to_string());
        self.check_attrs(&item.attrs, "type_alias", &target);
        syn::visit::visit_item_type(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        for attr in &item.attrs {
            if attr.path().is_ident("antigen") {
                // ATK-A2-007: silently dropping #[antigen] on enums eats the
                // class-enum pattern (the frame-translation antigen's primary
                // use case). Surface the situation as a parse_failure so the
                // user sees it, rather than the previous `let _ = attr` no-op.
                // The macro itself still rejects non-unit structs at compile
                // time; this scan-side diagnostic catches enum cases that
                // wouldn't reach the macro (e.g., a user investigating "why
                // doesn't my class enum scan as an antigen?").
                self.report.parse_failures.push((
                    self.file_path.clone(),
                    format!(
                        "#[antigen] on enum `{}` is not supported in v0.1; \
                         antigen declarations must be unit structs (e.g., \
                         `pub struct {};`). Enum-shaped failure-classes are \
                         tracked by ADR-010 Amendment 1's class-enum operator \
                         in a future grammar version.",
                        item.ident, item.ident
                    ),
                ));
            }
        }
        let target = ItemTarget::Enum(item.ident.to_string());
        self.check_attrs(&item.attrs, "enum", &target);
        syn::visit::visit_item_enum(self, item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scan_report_has_no_unaddressed() {
        let report = ScanReport::default();
        assert!(report.unaddressed_presentations().is_empty());
    }

    #[test]
    fn antigen_args_parses_name_and_fingerprint() {
        let tokens: proc_macro2::TokenStream = r#"
            name = "frame-translation",
            fingerprint = "item: enum, has_method(\"meet\", \"(Self, Self) -> Self\")"
        "#
        .parse()
        .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "frame-translation");
        // The fingerprint must be correctly unescaped — not contain raw backslash-quote.
        let fp = args.fingerprint.unwrap();
        assert!(
            fp.contains("has_method(\"meet\""),
            "fingerprint should contain unescaped double-quotes, got: {fp:?}"
        );
        assert!(
            !fp.contains(r#"\""#),
            "fingerprint must not contain raw backslash-quote escape sequences, got: {fp:?}"
        );
    }

    #[test]
    fn antigen_args_parses_optional_fields() {
        let tokens: proc_macro2::TokenStream =
            r#"name = "panicking-in-drop", fingerprint = "impl Drop", family = "boundary-violation", summary = "Drop impl can panic""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "panicking-in-drop");
        assert_eq!(args.family.as_deref(), Some("boundary-violation"));
        assert_eq!(args.summary.as_deref(), Some("Drop impl can panic"));
    }

    #[test]
    fn immune_args_parses_antigen_type_and_witness() {
        let tokens: proc_macro2::TokenStream = r"PanickingInDrop, witness = no_panic_in_drop_test"
            .parse()
            .unwrap();
        let args = syn::parse2::<ScanImmuneArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type, "PanickingInDrop");
        assert_eq!(args.witness, "no_panic_in_drop_test");
    }

    #[test]
    fn immune_args_parses_path_witness() {
        let tokens: proc_macro2::TokenStream =
            r"FrameTranslation, witness = clippy :: no_panic_in_drop"
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanImmuneArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type, "FrameTranslation");
        // witness is the token-stream rendering of the expression
        assert!(args.witness.contains("no_panic_in_drop"));
    }

    // ========================================================================
    // Cross-parser equivalence fixtures
    //
    // These fixtures must stay in sync with `antigen-macros::parse::tests::
    // ANTIGEN_PARSER_FIXTURES`. The invariant: for any input the proc-macro
    // parser accepts as valid, the scan parser must produce equivalent semantic
    // content for the four overlapping fields (name, fingerprint, family,
    // summary). The two parsers live in different crates by structural
    // necessity (proc-macro = true crates can't be linked as libraries) and
    // their drift was the substance of ATK-001-2 in pre-team scaffolding.
    //
    // When adding a fixture here, add the matching one in antigen-macros's
    // parse.rs ANTIGEN_PARSER_FIXTURES table. Field-additions to the antigen
    // attribute grammar must add fixtures in BOTH crates to lock the
    // equivalence.
    // ========================================================================

    // (input, expected_name, expected_fingerprint, expected_family, expected_summary)
    type ScanFixture = (
        &'static str,
        &'static str,
        &'static str,
        Option<&'static str>,
        Option<&'static str>,
    );

    const SCAN_PARSER_FIXTURES: &[ScanFixture] = &[
        (
            r#"name = "panicking-in-drop", fingerprint = "impl Drop with panic""#,
            "panicking-in-drop",
            "impl Drop with panic",
            None,
            None,
        ),
        (
            r#"name = "frame-translation", fingerprint = "class enum + meet", family = "semantic-drift", summary = "Polarity inverts at the frame boundary""#,
            "frame-translation",
            "class enum + meet",
            Some("semantic-drift"),
            Some("Polarity inverts at the frame boundary"),
        ),
        (
            r#"name = "x", fingerprint = "item: enum, has_method(\"meet\", \"(Self, Self) -> Self\")""#,
            "x",
            r#"item: enum, has_method("meet", "(Self, Self) -> Self")"#,
            None,
            None,
        ),
        (
            r#"summary = "S", family = "F", fingerprint = "FP", name = "n""#,
            "n",
            "FP",
            Some("F"),
            Some("S"),
        ),
        (
            r#"name = "x", fingerprint = "y", references = ["GAP-1", "DEC-2"]"#,
            "x",
            "y",
            None,
            None,
        ),
        (
            "name = \"multi-line\",\n\tfingerprint = \"shape\",\n\tfamily = \"family\"",
            "multi-line",
            "shape",
            Some("family"),
            None,
        ),
    ];

    #[test]
    fn scan_parser_accepts_all_macro_fixtures() {
        for (input, exp_name, exp_fp, exp_family, exp_summary) in SCAN_PARSER_FIXTURES {
            let tokens: proc_macro2::TokenStream = input
                .parse()
                .unwrap_or_else(|e| panic!("fixture failed to tokenize: {input:?}: {e}"));
            let args = syn::parse2::<ScanAntigenArgs>(tokens)
                .unwrap_or_else(|e| panic!("scan parser rejected fixture {input:?}: {e}"));
            assert_eq!(&args.name, exp_name, "name mismatch for fixture: {input:?}");
            assert_eq!(
                args.fingerprint.as_deref(),
                Some(*exp_fp),
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
    fn scan_parser_tolerates_unknown_fields() {
        // Asymmetry from macro side: scan tolerates unknown fields silently
        // (forward-compat for fields added later that this scan binary doesn't
        // know yet). Macro side rejects them strictly.
        let tokens: proc_macro2::TokenStream =
            r#"name = "x", fingerprint = "y", future_field = "irrelevant""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "x");
        assert_eq!(args.fingerprint.as_deref(), Some("y"));
    }

    #[test]
    fn scan_parser_tolerates_missing_required_fields() {
        // Asymmetry from macro side: scan defaults to empty rather than
        // erroring. Malformed declarations get aggregated into parse_failures
        // upstream rather than blocking the scan.
        let tokens: proc_macro2::TokenStream = r#"name = "only-name""#.parse().unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "only-name");
        assert_eq!(args.fingerprint, None);
    }

    // ========================================================================
    // Property tests (W1) — proptest invariants over the scan-side parser.
    //
    // These are the scan-side half of the cross-parser equivalence story.
    // Their invariants mirror the macro-side proptests in
    // `antigen-macros/src/parse.rs::parser_props`. Both sides share the same
    // input strategies (literal-copied; if you change one, change the other
    // in the same commit) — this is the by-construction substitute for
    // running both parsers in one binary, which the proc-macro crate
    // separation forbids.
    //
    // Cross-parser invariants asserted (macro-side P1-P8 mirror):
    //
    //   I1 (P1 mirror) — round-trip on intersection: any input the macro
    //        side accepts, the scan side accepts and produces equivalent
    //        semantic content for name/fingerprint/family/summary.
    //
    //   I2 (P2 mirror) — order-invariance: shuffling fields doesn't change
    //        the parsed result on the scan side.
    //
    //   I3 (asymmetry) — scan tolerates what macro rejects:
    //        - unknown fields: macro errors, scan silently consumes
    //        - missing required: macro errors, scan defaults to empty
    //        These asymmetries are intentional (forward-compat + non-blocking
    //        scan progress on partial workspaces) and are documented as
    //        properties so they don't accidentally regress.
    //
    // The macro-side parser reports were tested under `parser_props` in the
    // sister crate. Together, the two test modules form the W1 floor that
    // ADR-001 Amendment 1 C5 (drift-detection-at-scan-time) makes
    // load-bearing.
    // ========================================================================

    mod parser_props {
        use super::super::*;
        use proc_macro2::TokenStream;
        use proptest::prelude::*;

        /// Rust strict + reserved keywords that cannot appear as path
        /// segments. Kept in sync with `antigen-macros/src/parse.rs::
        /// parser_props::RUST_KEYWORDS` (literal-shared by convention; if
        /// you change one, change the other in the same commit).
        const RUST_KEYWORDS: &[&str] = &[
            "as", "async", "await", "box", "break", "const", "continue", "crate", "do", "dyn",
            "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
            "macro", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
            "struct", "super", "trait", "true", "type", "union", "unsafe", "use", "where", "while",
            "yield", "abstract", "become", "final", "override", "priv", "try",
        ];

        // --- Strategies (kept literally identical to macro side; see
        //     antigen-macros/src/parse.rs::parser_props). ---

        fn valid_kebab() -> impl Strategy<Value = String> {
            proptest::collection::vec(
                (
                    proptest::char::range('a', 'z'),
                    proptest::collection::vec(
                        prop_oneof![
                            proptest::char::range('a', 'z'),
                            proptest::char::range('0', '9'),
                        ],
                        0..8usize,
                    ),
                )
                    .prop_map(|(first, rest)| {
                        let mut s = String::with_capacity(rest.len() + 1);
                        s.push(first);
                        for c in rest {
                            s.push(c);
                        }
                        s
                    }),
                1..5usize,
            )
            .prop_map(|segments| segments.join("-"))
        }

        fn valid_text(max_len: usize) -> impl Strategy<Value = String> {
            proptest::collection::vec(
                prop_oneof![
                    proptest::char::range(' ', '~').prop_filter("excluded chars", |c| {
                        *c != '\\' && *c != '"' && *c != '\0'
                    }),
                ],
                1..=max_len,
            )
            .prop_map(|chars| chars.into_iter().collect())
        }

        fn lit(s: &str) -> String {
            format!("{s:?}")
        }

        fn render_antigen_body(
            name: &str,
            fingerprint: &str,
            family: Option<&str>,
            summary: Option<&str>,
        ) -> String {
            let mut parts = vec![
                format!("name = {}", lit(name)),
                format!("fingerprint = {}", lit(fingerprint)),
            ];
            if let Some(f) = family {
                parts.push(format!("family = {}", lit(f)));
            }
            if let Some(s) = summary {
                parts.push(format!("summary = {}", lit(s)));
            }
            parts.join(", ")
        }

        proptest! {
            // I1 — equivalence-on-intersection: any input the macro side
            // accepts (i.e., any input render_antigen_body produces from
            // valid_kebab + valid_text), the scan side accepts and produces
            // matching semantic content. The macro-side counterpart asserts
            // its own acceptance + identical extraction; together they lock
            // cross-parser drift.
            #[test]
            fn scan_parser_round_trip_on_macro_inputs(
                name in valid_kebab(),
                fingerprint in valid_text(64),
                family in proptest::option::of(valid_text(32)),
                summary in proptest::option::of(valid_text(64)),
            ) {
                let body = render_antigen_body(&name, &fingerprint, family.as_deref(), summary.as_deref());
                let tokens: TokenStream = body.parse().expect("body must tokenize");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan must accept macro-acceptable input");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
                prop_assert_eq!(args.family.as_deref(), family.as_deref());
                prop_assert_eq!(args.summary.as_deref(), summary.as_deref());
            }

            // I2 — order-invariance: scan side, like macro side, must not
            // depend on field order.
            #[test]
            fn scan_parser_order_invariant(
                name in valid_kebab(),
                fingerprint in valid_text(48),
                family in valid_text(24),
                summary in valid_text(48),
            ) {
                let canonical = format!(
                    "name = {}, fingerprint = {}, family = {}, summary = {}",
                    lit(&name), lit(&fingerprint), lit(&family), lit(&summary),
                );
                let reversed = format!(
                    "summary = {}, family = {}, fingerprint = {}, name = {}",
                    lit(&summary), lit(&family), lit(&fingerprint), lit(&name),
                );
                let a: ScanAntigenArgs = syn::parse2(canonical.parse::<TokenStream>().unwrap()).unwrap();
                let b: ScanAntigenArgs = syn::parse2(reversed.parse::<TokenStream>().unwrap()).unwrap();
                prop_assert_eq!(&a.name, &b.name);
                prop_assert_eq!(&a.fingerprint, &b.fingerprint);
                prop_assert_eq!(&a.family, &b.family);
                prop_assert_eq!(&a.summary, &b.summary);
            }

            // I3a (asymmetry) — scan tolerates unknown fields. For ANY
            // valid base input plus an arbitrary unknown field, scan still
            // parses and recovers name/fingerprint correctly.
            #[test]
            fn scan_parser_tolerates_arbitrary_unknown_field(
                name in valid_kebab(),
                fingerprint in valid_text(32),
                unknown in "[a-z][a-z_]{2,12}".prop_filter(
                    "must not collide with known fields or Rust keywords",
                    |s| {
                        !matches!(s.as_str(), "name" | "fingerprint" | "family" | "summary" | "references")
                            && !RUST_KEYWORDS.contains(&s.as_str())
                    },
                ),
                unknown_val in valid_text(16),
            ) {
                let body = format!(
                    "name = {}, fingerprint = {}, {} = {}",
                    lit(&name), lit(&fingerprint), unknown, lit(&unknown_val),
                );
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan tolerates unknown fields");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
            }

            // I3b (asymmetry) — scan tolerates missing required fields:
            // an input with only `name` (or only `fingerprint`) parses,
            // with the other field as `None` / empty. Macro side errors.
            #[test]
            fn scan_parser_tolerates_missing_fingerprint(
                name in valid_kebab(),
            ) {
                let body = format!("name = {}", lit(&name));
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan tolerates missing fingerprint");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint, None);
            }

            // I4 — references field is consumed silently (not stored in
            // the scan output today, but must not error). Macro side
            // stores into Vec<String>.
            #[test]
            fn scan_parser_consumes_references_array(
                name in valid_kebab(),
                fingerprint in valid_text(32),
                refs in proptest::collection::vec(valid_text(24), 0..6usize),
            ) {
                let refs_lit: Vec<String> = refs.iter().map(|s| lit(s)).collect();
                let body = format!(
                    "name = {}, fingerprint = {}, references = [{}]",
                    lit(&name), lit(&fingerprint), refs_lit.join(", "),
                );
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan parses references");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
            }

            // I5 — ScanImmuneArgs: any (path, witness-path) parses and
            // exposes the last segment as antigen_type. Mirrors the
            // macro-side P7 property.
            //
            // Identifier strategies skip Rust keywords: even though our
            // regex generates legal-looking strings like "as" or "fn",
            // syn rejects them as path segments. Filtering the keyword
            // set out is the by-construction property — "valid path
            // segments parse" — rather than the false stronger one
            // "any [a-z][a-z_0-9]* parses".
            #[test]
            fn scan_immune_extracts_last_path_segment(
                antigen in "[A-Z][A-Za-z0-9]{0,16}",
                witness_segments in proptest::collection::vec(
                    "[a-z][a-z_0-9]{0,8}".prop_filter(
                        "must not be a Rust keyword",
                        |s| !RUST_KEYWORDS.contains(&s.as_str()),
                    ),
                    1..4usize,
                ),
            ) {
                let witness = witness_segments.join("::");
                let body = format!("{antigen}, witness = {witness}");
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanImmuneArgs>(tokens).expect("body parses");
                prop_assert_eq!(args.antigen_type.as_str(), antigen.as_str());
                // witness_segments.last() is the trailing identifier; the
                // scan parser canonicalises whitespace via quote::ToTokens,
                // so the rendered witness contains all segments separated
                // by " :: ".
                let last = witness_segments.last().unwrap();
                prop_assert!(args.witness.contains(last.as_str()),
                    "rendered witness {:?} should contain trailing segment {:?}", args.witness, last);
            }

            // I6 — ScanImmuneArgs: a qualified-path antigen extracts only
            // the last segment as the antigen_type (the matching surface
            // antigen scan/audit reasons against). This is a regression
            // anchor for ATK-A2-001 — the path-split corruption that the
            // adversarial pass surfaced.
            #[test]
            fn scan_immune_qualified_antigen_path_extracts_last_segment(
                module_segs in proptest::collection::vec(
                    "[a-z][a-z_0-9]{0,6}".prop_filter(
                        "must not be a Rust keyword",
                        |s| !RUST_KEYWORDS.contains(&s.as_str()),
                    ),
                    1..3usize,
                ),
                antigen in "[A-Z][A-Za-z0-9]{0,12}",
                witness in "[a-z][a-z_0-9]{0,12}".prop_filter(
                    "must not be a Rust keyword",
                    |s| !RUST_KEYWORDS.contains(&s.as_str()),
                ),
            ) {
                let qualified = format!("{}::{}", module_segs.join("::"), antigen);
                let body = format!("{qualified}, witness = {witness}");
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanImmuneArgs>(tokens).expect("body parses");
                prop_assert_eq!(args.antigen_type.as_str(), antigen.as_str(),
                    "qualified antigen path {:?} must yield bare last-segment antigen_type", qualified);
            }
        }
    }
}
