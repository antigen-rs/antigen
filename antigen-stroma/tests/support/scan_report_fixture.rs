//! Reusable `ScanReport` fixture — closes frame-v2#4
//! (`frame-v2-constitute-lowering-integration-test-dark`).
//!
//! ## Why this exists
//!
//! The constitute adapter (`antigen_stroma::constitute::adapter`) had NO integration test because
//! there was no way to construct a `ScanReport` with controlled items/files from a test. That gap
//! let 13 mutants survive in `module_chain_from_path` / `lower_scan_report` / `digests_at_line`
//! (the cargo-mutants sweep), and it BLOCKED frame-v2#1's born-red (which needs two items sharing an
//! `ItemTarget` in one file). This module is the missing substrate: a small builder that lets a test
//! assemble a `ScanReport` node-by-node and, where identity matters, back those nodes with a REAL
//! on-disk source file (so `digests_at_line`'s `std::fs::read_to_string` + `syn::parse_file` path is
//! exercised for real, not stubbed).
//!
//! ## Two construction modes
//!
//! - **In-memory node-bearing records** ([`ScanReportBuilder::presentation`] etc.) — for testing the
//!   pure path logic (`module_chain_from_path`, dedup keys, edge emptiness). The `(file, line)` need
//!   not point at a readable item; `lower_scan_report` falls back to `gap_digests` and the node still
//!   lands. This is enough for the module-chain and dedup-key mutants.
//! - **File-backed items** ([`FixtureFile`]) — for testing the identity path end-to-end. A
//!   `FixtureFile` locates real items in a real source file by their `syn` span, so a test can pin
//!   the exact `(file, line, ItemTarget)` a record must carry to make `digests_at_line` compute a
//!   genuine `IdentityDigest`. This is what frame-v2#1's born-red needs: two items that share an
//!   `ItemTarget` but sit at distinct lines with distinct bodies → distinct identity digests.
//!
//! Shared into a test file via `#[path = "support/scan_report_fixture.rs"] mod fixture;`.

#![allow(dead_code)] // Each consuming test uses a subset; the whole is the reusable surface.

use std::path::{Path, PathBuf};

use antigen::scan::{ItemTarget, MatchKind, Presentation, ScanReport};

/// Build a `ScanReport` with controlled node-bearing records.
///
/// Starts empty and accumulates records. Only the vecs a test drives are populated; the rest stay
/// empty. `files_scanned`/`parse_failures` are inert (the adapter ignores them).
pub struct ScanReportBuilder {
    report: ScanReport,
}

impl ScanReportBuilder {
    /// A fresh, fully-empty report. Every vec empty; no coverage; zero files scanned.
    pub const fn new() -> Self {
        Self {
            report: ScanReport {
                antigens: Vec::new(),
                presentations: Vec::new(),
                immunities: Vec::new(),
                tolerances: Vec::new(),
                lineage_edges: Vec::new(),
                deferred_defenses: Vec::new(),
                convergent_evidences: Vec::new(),
                recurrent_declarations: Vec::new(),
                mucosal_declarations: Vec::new(),
                prescriptive_declarations: Vec::new(),
                defenses: Vec::new(),
                generates_declarations: Vec::new(),
                marked_unknowns: Vec::new(),
                files_scanned: 0,
                parse_failures: Vec::new(),
                scan_coverage: None,
            },
        }
    }

    /// Add one node-bearing `Presentation` record at `(file, line)` carrying `item_target`.
    ///
    /// `Presentation` is the canonical node-bearing record; the adapter interns all node-bearing
    /// vecs identically, so testing via `presentations` exercises the same intern/dedup path the
    /// other vecs use. Every other `Presentation` field is filled with an inert value — only
    /// `(file, line, item_target)` steer the lowering.
    pub fn presentation(
        mut self,
        file: impl Into<PathBuf>,
        line: usize,
        item_target: ItemTarget,
    ) -> Self {
        self.report.presentations.push(Presentation {
            antigen_type: "FixtureAntigen".to_string(),
            file: file.into(),
            line,
            item_kind: "fixture".to_string(),
            item_target,
            match_kind: MatchKind::default(),
            canonical_path: None,
            inherited_from: None,
            structural_fingerprint: String::new(),
            requires_predicate: None,
            proof: None,
        });
        self
    }

    /// Finish and return the assembled `ScanReport`.
    pub fn build(self) -> ScanReport {
        self.report
    }
}

impl Default for ScanReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A real on-disk source file, parsed so tests can locate items by their `syn` span.
///
/// Wraps a fixture `.rs` under `tests/fixtures/`. Use [`FixtureFile::impls_of`] to find every
/// `impl <Type>` block and its 1-based start line (the line `digests_at_line` matches on). Because
/// the lines are DISCOVERED (not hard-coded), an edit that shifts the fixture's layout can't silently
/// mis-target — the test pins the count/lines it expects and fails loud if the specimen changed.
pub struct FixtureFile {
    /// Path RELATIVE to `source_root` — this is the `file` a record must carry (the adapter joins
    /// `source_root.join(file)` before reading).
    pub rel_path: PathBuf,
    /// The `source_root` a `SourceWitness`/`lower_scan_report` call must use to resolve `rel_path`.
    pub source_root: PathBuf,
    parsed: syn::File,
}

/// One located item in a [`FixtureFile`]: its 1-based start line and its `ItemTarget`.
#[derive(Debug, Clone)]
pub struct LocatedItem {
    /// 1-based line of the item's `syn` span start — the value `Presentation::line` must carry so
    /// `digests_at_line` finds this exact item.
    pub line: usize,
    /// The `ItemTarget` this item lowers to (mirrors the scanner's classification for the arms the
    /// fixture uses).
    pub item_target: ItemTarget,
}

impl FixtureFile {
    /// Load a fixture file. `source_root` is the dir the `file` paths are relative to;
    /// `rel_path` is the file's path under it (the value records carry). Parses once with
    /// `syn::parse_file` — the same parser `digests_at_line` uses, so spans line up exactly.
    pub fn load(source_root: impl Into<PathBuf>, rel_path: impl Into<PathBuf>) -> Self {
        let source_root = source_root.into();
        let rel_path = rel_path.into();
        let abs = source_root.join(&rel_path);
        let src = std::fs::read_to_string(&abs)
            .unwrap_or_else(|e| panic!("fixture file unreadable at {}: {e}", abs.display()));
        let parsed = syn::parse_file(&src)
            .unwrap_or_else(|e| panic!("fixture file must parse at {}: {e}", abs.display()));
        Self {
            rel_path,
            source_root,
            parsed,
        }
    }

    /// The single top-level `struct <name>` in this file, with its 1-based span-start line and
    /// `ItemTarget::Struct(name)`. Panics if zero or more than one match — the modtree specimens each
    /// hold exactly one struct so the located line is unambiguous.
    pub fn struct_named(&self, name: &str) -> LocatedItem {
        use syn::spanned::Spanned;

        let mut found: Vec<LocatedItem> = Vec::new();
        for item in &self.parsed.items {
            if let syn::Item::Struct(s) = item {
                if s.ident == name {
                    found.push(LocatedItem {
                        line: item.span().start().line,
                        item_target: ItemTarget::Struct(name.to_string()),
                    });
                }
            }
        }
        assert_eq!(
            found.len(),
            1,
            "specimen invariant: expected exactly one `struct {name}` in {}, found {}",
            self.rel_path.display(),
            found.len()
        );
        found.pop().unwrap()
    }

    /// The single top-level `struct <name>`'s 1-based span START and END lines (inclusive), plus its
    /// `ItemTarget`. `syn` folds outer attrs into the item span, so `start` is the first attr line and
    /// `end` is the closing brace. Lets a test address TWO distinct lines that both fall INSIDE the one
    /// item (the multi-attr-different-line containment case). Asserts `end > start` so the specimen
    /// genuinely spans multiple lines (a single-line item can't exhibit the come-apart).
    pub fn struct_span(&self, name: &str) -> (usize, usize, ItemTarget) {
        use syn::spanned::Spanned;

        let mut found = Vec::new();
        for item in &self.parsed.items {
            if let syn::Item::Struct(s) = item {
                if s.ident == name {
                    let span = item.span();
                    found.push((span.start().line, span.end().line));
                }
            }
        }
        assert_eq!(
            found.len(),
            1,
            "specimen invariant: expected exactly one `struct {name}` in {}, found {}",
            self.rel_path.display(),
            found.len()
        );
        let (start, end) = found.pop().unwrap();
        assert!(
            end > start,
            "specimen invariant: `struct {name}` must span multiple lines (start={start}, end={end}) \
             for the multi-attr-different-line containment case"
        );
        (start, end, ItemTarget::Struct(name.to_string()))
    }

    /// Every inherent `impl <target_type>` block whose target type renders to `target_type`, with its
    /// 1-based span-start line. Ordered by line. This is the primitive frame-v2#1 needs: two
    /// `impl Foo` blocks share an `ItemTarget` but sit at distinct lines with distinct bodies.
    pub fn inherent_impls_of(&self, target_type: &str) -> Vec<LocatedItem> {
        use quote::ToTokens;
        use syn::spanned::Spanned;

        let mut out = Vec::new();
        for item in &self.parsed.items {
            if let syn::Item::Impl(imp) = item {
                if imp.trait_.is_some() {
                    continue; // inherent only
                }
                let rendered = imp.self_ty.to_token_stream().to_string();
                if rendered == target_type {
                    out.push(LocatedItem {
                        line: item.span().start().line,
                        item_target: ItemTarget::Impl {
                            trait_path: None,
                            target_type: target_type.to_string(),
                        },
                    });
                }
            }
        }
        out.sort_by_key(|l| l.line);
        out
    }
}

/// The absolute path to a fixture file under `tests/fixtures/<sub>`, and the fixtures dir as the
/// `source_root`. Returns `(source_root, rel_path)` where `source_root.join(rel_path)` is the file.
///
/// `CARGO_MANIFEST_DIR` points at the crate root (`antigen-stroma/`) even for integration tests, so
/// `tests/fixtures/...` resolves regardless of the working directory the test runner uses.
pub fn fixture_path(rel: &str) -> (PathBuf, PathBuf) {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = crate_root.join("tests").join("fixtures");
    (source_root, PathBuf::from(rel))
}

/// Like [`fixture_path`] but rooted at a fixture SUB-directory, so `rel` can be a `src/…`-relative
/// path that `module_chain_from_path`'s `strip_prefix("src")` recognizes. Used by the module-tree
/// specimens: `fixture_root("frame_v2_modtree", "src/lib.rs")` →
/// `source_root = tests/fixtures/frame_v2_modtree`, `rel = "src/lib.rs"`.
pub fn fixture_root(sub: &str, rel: &str) -> (PathBuf, PathBuf) {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = crate_root.join("tests").join("fixtures").join(sub);
    (source_root, PathBuf::from(rel))
}
