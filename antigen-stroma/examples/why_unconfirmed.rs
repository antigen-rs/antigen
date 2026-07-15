//! Why unconfirmed — categorize antigen's syntactic-only call edges by WHY SCIP didn't confirm
//! them, instead of calling the whole set "phantom" (a verdict [`dual_tier_barcode`]'s reading
//! measured but never earned).
//!
//! [`dual_tier_barcode`] measures ~3300 syntactic (`syn`-extracted) call edges that SCIP does not
//! confirm and tier-stamps them `Syntactic` (SCIP-unconfirmed). That tier-stamp is honest about
//! CONFIDENCE; it says nothing about WHY the edge went unconfirmed. A syntactic-only edge is at
//! least four different things with different remedies:
//!
//! 1. **CTOR-ARTIFACT** — `Some(x)` / `Ok(x)` / `Foo(x)`: an enum-variant or tuple-struct
//!    constructor that `syn`'s `ExprCall` visitor miscounts as a function call. Not a real edge.
//! 2. **COLLISION-ARTIFACT** — the bare-fn-name projection (`dual_tier_barcode`'s node identity)
//!    merges distinct symbols sharing a name (`from`, `new`, `eq`, ...), inventing an edge between
//!    two symbols that were never actually connected. Not a real edge.
//! 3. **DISPATCH-CANDIDATE** — `dyn` dispatch, a generic trait method, a macro-expanded call, a
//!    fn-pointer, a closure: the edge IS real; static SCIP resolution just cannot see it. This is
//!    the interesting bucket — a map of where antigen's polymorphism lives, not noise to explain
//!    away.
//! 4. **MACRO** — the callee is a macro invocation (`vec!`, `format!`, ...); `syn`'s `ExprCall`
//!    shape cannot distinguish a macro call from a function call.
//! 5. **NOT-IN-SCIP** — the callee name has no `SymbolInformation` at all in the index (external
//!    crate, macro-generated code SCIP never indexed, or not a fn in the first place).
//! 6. **UNEXPLAINED** — none of the above: the genuinely-absent residue, where `syn` may simply
//!    have hallucinated a call that does not exist.
//!
//! ## What SCIP actually offers (this file reads it for the first time)
//!
//! Every prior falsifier (`antigen-stroma-experiment`'s `debug_scip.rs`, and
//! [`ingest_scip`][antigen_stroma::scip::ingest_scip] itself) reads only `Document.occurrences`.
//! The SCIP index also carries `Document.symbols: Vec<SymbolInformation>` and
//! `Index.external_symbols: Vec<SymbolInformation>`, each with a `kind` (`Function`, `Method`,
//! `TraitMethod`, `EnumMember`, `Struct`, `Enum`, `Macro`, `Trait`, `Field`, `TypeParameter`,
//! `Variable`, ...) and `relationships: Vec<Relationship>` (`is_implementation` is the
//! trait→impl dispatch map). This file reads that surface, so the categorization below is
//! SCIP-definitive where SCIP's own semantic classification allows it, rather than guessing from
//! surface syntax (an uppercase first letter, a hardcoded macro-name list) the way `debug_scip.rs`
//! had to.
//!
//! ## Step 0 — measure the instrument before trusting it
//!
//! The categorization below is only as good as the SCIP index's `SymbolInformation` population.
//! [`measure_instrument`] prints how much of it rust-analyzer actually filled in — total count,
//! how many carry a real `kind`, how many carry `relationships`, how many of those are
//! `is_implementation` — loudly, BEFORE any bucket count is trusted.
//!
//! Usage: `cargo run -p antigen-stroma --example why_unconfirmed -- [path/to.scip]`
//! (defaults to `target/antigen.scip`, produced by `rust-analyzer scip . --output …`).

// A stats-display example: usize→f64 precision loss on counts is irrelevant here.
#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use protobuf::Message;
use scip::types::symbol_information::Kind as ScipKind;
use scip::types::{Index, SymbolInformation};
use syn::visit::Visit;
use walkdir::WalkDir;

// ── Syntactic extraction (ported from `antigen-stroma/examples/dual_tier_barcode.rs`, itself
//    ported from `antigen-stroma-experiment/src/main.rs`) ──────────────────────────────────────

/// Visitor that records `(caller_fn, callee_name)` syntactic call-edges — unchanged from
/// `dual_tier_barcode`'s `CallVisitor`.
struct CallVisitor {
    /// Current enclosing function name stack (for nested fns/closures).
    fn_stack: Vec<String>,
    /// Collected edges: (`caller_fn`, `callee_name`).
    edges: Vec<(String, String)>,
}

impl CallVisitor {
    const fn new() -> Self {
        Self {
            fn_stack: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn current_fn(&self) -> Option<&str> {
        self.fn_stack.last().map(String::as_str)
    }
}

impl<'ast> Visit<'ast> for CallVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_trait_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let (Some(caller), Some(callee)) = (self.current_fn(), extract_call_name(&node.func)) {
            self.edges.push((caller.to_string(), callee));
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if let Some(caller) = self.current_fn() {
            self.edges
                .push((caller.to_string(), node.method.to_string()));
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Extract the bare function name from a call expression's function part (ported, unchanged).
fn extract_call_name(expr: &syn::Expr) -> Option<String> {
    match expr {
        // `foo()`, `Foo::bar()`, `crate::mod::fn()` — last segment is the name.
        syn::Expr::Path(ep) => ep.path.segments.last().map(|s| s.ident.to_string()),
        // `self.field()` (method via field access) — use the field name.
        syn::Expr::Field(ef) => match &ef.member {
            syn::Member::Named(n) => Some(n.to_string()),
            syn::Member::Unnamed(_) => None,
        },
        // Closures, macro calls, etc. — not extractable as a name.
        _ => None,
    }
}

/// Extract syntactic `(caller_fn, callee_name)` call-edges from one Rust source file (ported).
fn syn_edges_from_file(path: &Path) -> Vec<(String, String)> {
    let Ok(src) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(parsed) = syn::parse_file(&src) else {
        return Vec::new();
    };
    let mut visitor = CallVisitor::new();
    visitor.visit_file(&parsed);
    visitor.edges
}

/// Walk the workspace's crate `src/` dirs and collect every syntactic `(caller, callee)` name
/// pair, deduplicated (ported, unchanged) — the exact same set `dual_tier_barcode` measured.
fn extract_syn_pairs(workspace_root: &Path) -> HashSet<(String, String)> {
    const CRATE_DIRS: &[&str] = &[
        "antigen/src",
        "antigen-stroma/src",
        "antigen-fingerprint/src",
        "antigen-macros/src",
        "antigen-attestation/src",
        "cargo-antigen/src",
    ];
    let mut pairs = HashSet::new();
    for dir in CRATE_DIRS {
        let src_root = workspace_root.join(dir);
        if !src_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&src_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ex| ex == "rs"))
        {
            pairs.extend(syn_edges_from_file(entry.path()));
        }
    }
    pairs
}

// ── SCIP bare-name reduction ────────────────────────────────────────────────────────────────

/// Reduce a path-like SCIP symbol segment (everything before the descriptor's trailing
/// punctuation has already been stripped by the caller) to its display name — strips a
/// bracketed type-qualifier prefix and backtick-escaping. Shared tail for both
/// [`symbol_fn_name`] (Method-descriptor-shaped symbols) and [`bare_name_general`] (every other
/// descriptor shape).
fn last_segment(before_split: &str) -> Option<&str> {
    let segment = before_split
        .rsplit(['/', '#'])
        .next()
        .filter(|s| !s.is_empty())?;
    // Strip a backtick-escaped type-qualifier prefix: `[`TypeName`]method_name` → `method_name`.
    if let Some(bracket_end) = segment.rfind(']') {
        let after_bracket = &segment[bracket_end + 1..];
        if !after_bracket.is_empty() {
            return Some(after_bracket);
        }
    }
    // Strip backtick escaping from a simple identifier: `` `name` `` → `name`.
    if segment.starts_with('`') && segment.ends_with('`') && segment.len() > 2 {
        return Some(&segment[1..segment.len() - 1]);
    }
    Some(segment)
}

/// Extract the bare function name from a SCIP symbol string (ported from `dual_tier_barcode`,
/// unchanged) — e.g. `rust-analyzer cargo antigen 0.6.0 scan/scan_file().` → `scan_file`. Only
/// matches the Method-descriptor shape (`name(disambiguator).`); every other descriptor shape
/// (Type `#`, Term `.`, Namespace `/`, ...) is handled by [`bare_name_general`] instead.
fn symbol_fn_name(symbol: &str) -> Option<&str> {
    let without_suffix = symbol.strip_suffix(").")?;
    let paren_pos = without_suffix.rfind('(')?;
    last_segment(&without_suffix[..paren_pos])
}

/// Extract the bare display-name segment from ANY SCIP symbol, regardless of descriptor kind —
/// the generalization [`symbol_fn_name`] deliberately does not attempt. Needed here because
/// `SymbolInformation` (unlike `ingest_scip`'s occurrence-only reads) covers every symbol kind:
/// `Struct#`/`Enum#`/`Trait#` (Type descriptor), `EnumMember.`/`Field.`/`Variable.` (Term
/// descriptor), `Method(...).`/`TraitMethod(...).` (Method descriptor), and so on.
/// `local `-prefixed symbols are excluded — no stable cross-document identity (the same exclusion
/// `ingest_scip`'s `is_fn_symbol` already applies).
fn bare_name_general(symbol: &str) -> Option<&str> {
    if symbol.starts_with("local ") {
        return None;
    }
    if let Some(name) = symbol_fn_name(symbol) {
        return Some(name);
    }
    // Non-Method descriptor: exactly one trailing delimiter to strip, then the last segment.
    let trimmed = symbol.trim_end_matches(['#', '.', ':', ')', ']']);
    last_segment(trimmed)
}

/// Reduce every `Resolved` SCIP reconstruction to a bare-name `(caller, callee)` pair (ported from
/// `dual_tier_barcode`, unchanged) — the same SCIP-confirmed set the ~3300-syntactic-only
/// measurement was taken against.
fn scip_resolved_pairs(recons: &[EdgeReconstruction]) -> HashSet<(String, String)> {
    let mut pairs = HashSet::new();
    for recon in recons {
        let EdgeReconstruction::Resolved(edge) = recon else {
            continue;
        };
        if let (Some(caller), Some(callee)) = (
            symbol_fn_name(&edge.src_symbol),
            symbol_fn_name(&edge.dst_symbol),
        ) {
            pairs.insert((caller.to_string(), callee.to_string()));
        }
    }
    pairs
}

// ── Step 0: measure the instrument ──────────────────────────────────────────────────────────

/// Every `SymbolInformation` in the index, from both feeders (`Document.symbols` +
/// `Index.external_symbols`) — one iterator so step 0 and step 2 read the exact same set.
fn all_symbol_information(index: &Index) -> impl Iterator<Item = &SymbolInformation> {
    index
        .documents
        .iter()
        .flat_map(|doc| doc.symbols.iter())
        .chain(index.external_symbols.iter())
}

/// `n / total` as a percentage, `0.0` when `total` is zero (printing helper).
fn pct(n: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        n as f64 / total as f64 * 100.0
    }
}

/// Print whether rust-analyzer actually populated `SymbolInformation.kind` / `.relationships` /
/// `Relationship.is_implementation` in this index — MUST run before any bucket count downstream is
/// trusted, because CTOR-ARTIFACT/MACRO/DISPATCH-CANDIDATE all depend on this data existing.
fn measure_instrument(index: &Index) {
    let all: Vec<&SymbolInformation> = all_symbol_information(index).collect();
    let total = all.len();
    let with_kind = all
        .iter()
        .filter(|s| s.kind.enum_value_or_default() != ScipKind::UnspecifiedKind)
        .count();
    let with_relationships = all.iter().filter(|s| !s.relationships.is_empty()).count();
    let is_implementation_rels: usize = all
        .iter()
        .map(|s| {
            s.relationships
                .iter()
                .filter(|r| r.is_implementation)
                .count()
        })
        .sum();

    println!("== Step 0: measuring the instrument (does SCIP populate SymbolInformation?) ==");
    println!("  total SymbolInformation (Document.symbols + Index.external_symbols): {total}");
    println!(
        "  with a non-Unknown kind:   {with_kind:6}  ({:.1}%)",
        pct(with_kind, total)
    );
    println!(
        "  with any relationships:    {with_relationships:6}  ({:.1}%)",
        pct(with_relationships, total)
    );
    println!(
        "  total is_implementation relationships (trait→impl edges): {is_implementation_rels}"
    );
    if total == 0 {
        println!(
            "\n  ** CRITICAL: zero SymbolInformation in this index. rust-analyzer's `scip` \
             subcommand did not populate Document.symbols / Index.external_symbols on this run. \
             Every bucket below that depends on SCIP kind/relationship data collapses to \
             NOT-IN-SCIP — the categorization is NOT SCIP-definitive here, it is name-based \
             heuristics only, with no SCIP backing. **\n"
        );
    } else if with_kind == 0 {
        println!(
            "\n  ** CRITICAL: {total} symbols present but NONE carry a resolved kind — the \
             CTOR-ARTIFACT / MACRO / DISPATCH-CANDIDATE buckets below cannot be SCIP-definitive \
             on this run; every syntactic-only edge falls through to COLLISION-CANDIDATE / \
             NOT-IN-SCIP / UNEXPLAINED regardless of its true category. **\n"
        );
    } else {
        println!();
    }
}

// ── Step 2: name → {kinds, symbols, is_implementation} aggregation ─────────────────────────

/// What every SCIP symbol sharing a bare name tells us: the set of `Kind`s it can be (more than
/// one kind at the same bare name is itself a collision signal), how many DISTINCT full symbol
/// strings reduce to this name (2+ is the definitive collision witness — a kind-set of size 1 can
/// still hide multiple identically-kinded, distinctly-scoped symbols), and whether any of them
/// carries a trait-implementation relationship.
#[derive(Default)]
struct SymbolAgg {
    /// Every distinct `Kind` any symbol reducing to this bare name carries.
    kinds: HashSet<ScipKind>,
    /// Every distinct full SCIP symbol string reducing to this bare name — 2+ is the collision
    /// witness the bare-name projection cannot itself see.
    symbols: HashSet<String>,
    /// True if ANY symbol at this bare name carries an `is_implementation` relationship (a
    /// trait→impl dispatch edge).
    has_is_implementation: bool,
}

/// Build the `bare name -> SymbolAgg` map from every `SymbolInformation` in the index — the
/// projection the categorization below queries by CALLEE name.
fn build_name_map(index: &Index) -> HashMap<String, SymbolAgg> {
    let mut map: HashMap<String, SymbolAgg> = HashMap::new();
    for sym in all_symbol_information(index) {
        let Some(name) = bare_name_general(&sym.symbol) else {
            continue;
        };
        let agg = map.entry(name.to_string()).or_default();
        agg.kinds.insert(sym.kind.enum_value_or_default());
        agg.symbols.insert(sym.symbol.clone());
        if sym.relationships.iter().any(|r| r.is_implementation) {
            agg.has_is_implementation = true;
        }
    }
    map
}

// ── Step 3: categorization ───────────────────────────────────────────────────────────────────

/// The mutually-exclusive bucket a syntactic-only edge falls into, keyed on its CALLEE, checked
/// in this priority order (first match wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Bucket {
    /// Definitive from SCIP: the callee is an enum-variant/struct/enum constructor `syn` mistook
    /// for a function call. Not a real edge.
    CtorArtifact,
    /// The callee is a macro; `syn`'s `ExprCall` shape cannot distinguish a macro invocation from
    /// a function call.
    Macro,
    /// The edge is real; SCIP's static resolution cannot see it (dyn dispatch / generic trait
    /// method / macro-expanded call / fn-pointer / closure).
    DispatchCandidate,
    /// The bare-name projection merged 2+ distinct SCIP symbols — the edge may be an artifact of
    /// that merge, not a real connection between the actual caller and callee.
    CollisionCandidate,
    /// No `SymbolInformation` at all for this callee name (external crate, macro-generated code
    /// SCIP never indexed, or not a fn).
    NotInScip,
    /// None of the above — the genuinely-absent residue.
    Unexplained,
}

impl Bucket {
    const fn label(self) -> &'static str {
        match self {
            Self::CtorArtifact => "CTOR-ARTIFACT",
            Self::Macro => "MACRO",
            Self::DispatchCandidate => "DISPATCH-CANDIDATE",
            Self::CollisionCandidate => "COLLISION-CANDIDATE",
            Self::NotInScip => "NOT-IN-SCIP",
            Self::Unexplained => "UNEXPLAINED",
        }
    }
}

/// Categorize one syntactic-only edge by its CALLEE's aggregated SCIP identity, in strict
/// priority order.
fn categorize(callee: &str, name_map: &HashMap<String, SymbolAgg>) -> Bucket {
    let Some(agg) = name_map.get(callee) else {
        return Bucket::NotInScip;
    };
    if agg
        .kinds
        .iter()
        .any(|k| matches!(k, ScipKind::EnumMember | ScipKind::Struct | ScipKind::Enum))
    {
        return Bucket::CtorArtifact;
    }
    if agg.kinds.contains(&ScipKind::Macro) {
        return Bucket::Macro;
    }
    if agg
        .kinds
        .iter()
        .any(|k| matches!(k, ScipKind::Method | ScipKind::TraitMethod))
        || agg.has_is_implementation
    {
        return Bucket::DispatchCandidate;
    }
    if agg.symbols.len() >= 2 {
        return Bucket::CollisionCandidate;
    }
    Bucket::Unexplained
}

// ── Printing (split out of `main` to stay under the line budget) ──────────────────────────────

/// Print the priority-ordered bucket breakdown: count, percentage of the syntactic-only set, and
/// up to 5 example edges per bucket.
fn print_buckets(buckets: &HashMap<Bucket, Vec<&(String, String)>>, total: usize) {
    const LADDER: [Bucket; 6] = [
        Bucket::CtorArtifact,
        Bucket::Macro,
        Bucket::DispatchCandidate,
        Bucket::CollisionCandidate,
        Bucket::NotInScip,
        Bucket::Unexplained,
    ];
    println!("== Step 3/4: categorized syntactic-only edges (priority order, first match wins) ==");
    for bucket in LADDER {
        let empty = Vec::new();
        let edges = buckets.get(&bucket).unwrap_or(&empty);
        println!(
            "\n  {:<20} {:5}  ({:.1}% of syntactic-only)",
            bucket.label(),
            edges.len(),
            pct(edges.len(), total)
        );
        for (caller, callee) in edges.iter().take(5) {
            println!("      {caller} -> {callee}");
        }
    }
}

/// Print the cross-check comparisons: the SCIP-definitive CTOR-ARTIFACT count against the
/// pre-SCIP uppercase-first-letter proxy (`debug_scip.rs`'s heuristic, before this file existed to
/// do better), and the DISPATCH-CANDIDATE bucket split by signal strength.
///
/// The signal-strength split is the honest caveat Step 0 forces: with `relationships` unpopulated
/// (the common case — see [`measure_instrument`]), DISPATCH-CANDIDATE cannot distinguish "one real
/// fn, the call-site is just unresolvable" (bare name → a SINGLE SCIP symbol — dyn dispatch,
/// generic trait method, fn-pointer, closure, macro-expansion) from "an ordinary same-named method
/// on unrelated types" (bare name → 2+ DISTINCT symbols — `is_empty`/`clone`/`new` on totally
/// unrelated structs is exactly the COLLISION-CANDIDATE shape, just wearing a Method-kind hat,
/// with no `is_implementation` data available to tell true cross-impl polymorphism apart from
/// coincidence).
fn print_cross_checks(
    syntactic_only: &[&(String, String)],
    buckets: &HashMap<Bucket, Vec<&(String, String)>>,
    name_map: &HashMap<String, SymbolAgg>,
) {
    let empty = Vec::new();
    let ctor_artifact_count = buckets.get(&Bucket::CtorArtifact).unwrap_or(&empty).len();
    let dispatch = buckets.get(&Bucket::DispatchCandidate).unwrap_or(&empty);

    let uppercase_heuristic = syntactic_only
        .iter()
        .filter(|(_, callee)| callee.chars().next().is_some_and(char::is_uppercase))
        .count();
    let dispatch_single_symbol = dispatch
        .iter()
        .filter(|(_, callee)| {
            name_map
                .get(callee.as_str())
                .is_some_and(|agg| agg.symbols.len() == 1)
        })
        .count();
    let dispatch_multi_symbol = dispatch.len() - dispatch_single_symbol;

    println!("\n  -- cross-checks --");
    println!(
        "  CTOR-ARTIFACT (SCIP-definitive): {ctor_artifact_count:5}   vs   \
         uppercase-first-letter heuristic (over ALL syntactic-only edges): {uppercase_heuristic:5}"
    );
    println!(
        "  DISPATCH-CANDIDATE: {:5} total — {dispatch_single_symbol:5} map to a SINGLE SCIP \
         symbol (stronger signal: one real fn, call-site just unresolvable), {dispatch_multi_symbol:5} \
         map to 2+ DISTINCT symbols (weaker signal — indistinguishable from COLLISION-CANDIDATE \
         without populated `relationships` data on this run)",
        dispatch.len()
    );
}

// ── Main ────────────────────────────────────────────────────────────────────────────────────

fn main() {
    let scip_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    // antigen-stroma's own CARGO_MANIFEST_DIR is `<workspace>/antigen-stroma`; its parent is the
    // workspace root regardless of the cwd `cargo run` was invoked from.
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen-stroma has a parent directory (the workspace root)");
    println!("== Why unconfirmed: categorizing antigen's syntactic-only call edges ==\n");

    let Ok(bytes) = std::fs::read(&scip_path) else {
        println!(
            "(could not read {scip_path} — run `rust-analyzer scip . --output {scip_path}` first)"
        );
        return;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        println!("(could not parse {scip_path} as a SCIP protobuf index)");
        return;
    };

    // Step 0 — measure the instrument before trusting anything downstream of it.
    measure_instrument(&index);

    // Step 1 — recompute the syntactic-only (caller, callee) name-pair set exactly as
    // `dual_tier_barcode` does (same feeders, same reduction) — the ~3300-edge set in question.
    let syn_pairs = extract_syn_pairs(workspace_root);
    let recons = ingest_scip(Path::new(&scip_path));
    let scip_pairs = scip_resolved_pairs(&recons);
    let syntactic_only: Vec<&(String, String)> = syn_pairs.difference(&scip_pairs).collect();

    println!("== Step 1: recomputing the syntactic-only set ==");
    println!("  syntactic (syn-walk) pairs:   {}", syn_pairs.len());
    println!("  SCIP-resolved pairs:          {}", scip_pairs.len());
    println!(
        "  syntactic-only (unconfirmed): {}  ({:.1}% of syntactic)\n",
        syntactic_only.len(),
        pct(syntactic_only.len(), syn_pairs.len())
    );

    // Step 2 — name -> SCIP identity map, built from the SAME index step 0 measured.
    let name_map = build_name_map(&index);
    println!("== Step 2: bare-name → SCIP identity map ==");
    println!(
        "  distinct bare names carrying SCIP identity: {}\n",
        name_map.len()
    );

    // Step 3/4 — categorize every syntactic-only edge by its callee, priority-ordered.
    let mut buckets: HashMap<Bucket, Vec<&(String, String)>> = HashMap::new();
    for &pair in &syntactic_only {
        buckets
            .entry(categorize(&pair.1, &name_map))
            .or_default()
            .push(pair);
    }
    print_buckets(&buckets, syntactic_only.len());
    print_cross_checks(&syntactic_only, &buckets, &name_map);

    let empty = Vec::new();
    let unexplained = buckets.get(&Bucket::Unexplained).unwrap_or(&empty).len();
    println!(
        "\n== Read ==\n  UNEXPLAINED residue: {unexplained} / {} ({:.1}%) — the genuinely-absent \
         syntactic hallucinations, net of every SCIP-definitive and heuristic explanation above.",
        syntactic_only.len(),
        pct(unexplained, syntactic_only.len())
    );
}
