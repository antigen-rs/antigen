//! Are the UNEXPLAINED edges `cfg`-gated? — testing the missing-axis hypothesis.
//!
//! After fixing caller attribution, ~70 syntactic-only edges survive every discriminator: NOT a
//! constructor artifact, NOT a macro, NOT dispatch-shaped, NOT a name collision, and the callee IS
//! in SCIP. They are specified by exclusion — which makes them the most interesting object in the
//! set, not the least.
//!
//! **The hypothesis:** the two instruments are reading *different code*. `syn` parses EVERY `cfg`
//! branch; a SCIP index covers only the **active** configuration. A call inside
//! `#[cfg(target_os = "linux")]` built on Windows is syn-visible and SCIP-invisible — and would
//! land in exactly this bucket. If so, the residue is not noise: it is the **fingerprint of an
//! unpopulated axis** (config-fiber) announcing itself through the only channel it has.
//!
//! **The control (without which the number means nothing):** a cfg-gated rate for the residue is
//! only signal against the *base rate* over all syntactic edges, and against the SCIP-CONFIRMED
//! edges. Enrichment — not raw percentage — is the finding.
//!
//! A pair counts as gated only when EVERY occurrence of it is cfg-gated: if one call site is
//! unconditional, SCIP had a chance to see it and the hypothesis does not apply.

// A single measurement + its report: one linear pass that reads best as one unit.
#![allow(clippy::cast_precision_loss, clippy::too_many_lines)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use protobuf::Message;
use scip::types::symbol_information::Kind as ScipKind;
use scip::types::{Index, SymbolInformation};
use syn::visit::Visit;
use walkdir::WalkDir;

const CRATE_DIRS: &[&str] = &[
    "antigen/src",
    "antigen-stroma/src",
    "antigen-fingerprint/src",
    "antigen-macros/src",
    "antigen-attestation/src",
    "cargo-antigen/src",
];

/// The `cfg(...)` condition on an item, if any (rendered from its tokens).
fn cfg_of(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find(|a| a.path().is_ident("cfg")).map(|a| {
        if let syn::Meta::List(l) = &a.meta {
            l.tokens.to_string()
        } else {
            "(bare)".to_string()
        }
    })
}

/// Walks a file recording `(caller, callee)` plus the stack of `cfg` conditions in force.
#[derive(Default)]
struct CfgCallVisitor {
    fn_stack: Vec<String>,
    cfg_stack: Vec<String>,
    edges: Vec<(String, String, Vec<String>)>,
}

impl CfgCallVisitor {
    fn push_cfg(&mut self, attrs: &[syn::Attribute]) -> bool {
        cfg_of(attrs).is_some_and(|c| {
            self.cfg_stack.push(c);
            true
        })
    }

    fn pop_cfg(&mut self, pushed: bool) {
        if pushed {
            self.cfg_stack.pop();
        }
    }

    fn record(&mut self, callee: String) {
        if let Some(caller) = self.fn_stack.last() {
            self.edges
                .push((caller.clone(), callee, self.cfg_stack.clone()));
        }
    }
}

impl<'ast> Visit<'ast> for CfgCallVisitor {
    fn visit_item_mod(&mut self, n: &'ast syn::ItemMod) {
        let p = self.push_cfg(&n.attrs);
        syn::visit::visit_item_mod(self, n);
        self.pop_cfg(p);
    }

    fn visit_item_impl(&mut self, n: &'ast syn::ItemImpl) {
        let p = self.push_cfg(&n.attrs);
        syn::visit::visit_item_impl(self, n);
        self.pop_cfg(p);
    }

    fn visit_item_fn(&mut self, n: &'ast syn::ItemFn) {
        let p = self.push_cfg(&n.attrs);
        self.fn_stack.push(n.sig.ident.to_string());
        syn::visit::visit_item_fn(self, n);
        self.fn_stack.pop();
        self.pop_cfg(p);
    }

    fn visit_impl_item_fn(&mut self, n: &'ast syn::ImplItemFn) {
        let p = self.push_cfg(&n.attrs);
        self.fn_stack.push(n.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, n);
        self.fn_stack.pop();
        self.pop_cfg(p);
    }

    fn visit_trait_item_fn(&mut self, n: &'ast syn::TraitItemFn) {
        let p = self.push_cfg(&n.attrs);
        self.fn_stack.push(n.sig.ident.to_string());
        syn::visit::visit_trait_item_fn(self, n);
        self.fn_stack.pop();
        self.pop_cfg(p);
    }

    fn visit_expr_call(&mut self, n: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = &*n.func {
            if let Some(seg) = p.path.segments.last() {
                self.record(seg.ident.to_string());
            }
        }
        syn::visit::visit_expr_call(self, n);
    }

    fn visit_expr_method_call(&mut self, n: &'ast syn::ExprMethodCall) {
        self.record(n.method.to_string());
        syn::visit::visit_expr_method_call(self, n);
    }
}

/// Per-pair tally: how many occurrences, how many of them cfg-gated, and which conditions.
#[derive(Default)]
struct PairCfg {
    total: usize,
    gated: usize,
    conds: Vec<String>,
}

impl PairCfg {
    /// Gated only if EVERY occurrence is — one unconditional site means SCIP had its chance.
    const fn all_gated(&self) -> bool {
        self.total > 0 && self.gated == self.total
    }
}

/// The trailing display segment of a SCIP symbol — ported VERBATIM from `why_unconfirmed` so this
/// measurement reproduces that baseline exactly. Splits only on `/` and `#`, then strips a
/// backtick-escaped type qualifier (`` [`TypeName`]method `` → `method`). Splitting on `.`/space
/// instead mangles `impl#[Args]parse()` into `[Args]parse` and silently destroys the pairing.
fn last_segment(before_split: &str) -> Option<&str> {
    let segment = before_split
        .rsplit(['/', '#'])
        .next()
        .filter(|s| !s.is_empty())?;
    if let Some(bracket_end) = segment.rfind(']') {
        let after_bracket = &segment[bracket_end + 1..];
        if !after_bracket.is_empty() {
            return Some(after_bracket);
        }
    }
    if segment.starts_with('`') && segment.ends_with('`') && segment.len() > 2 {
        return Some(&segment[1..segment.len() - 1]);
    }
    Some(segment)
}

/// The bare fn name of a Method-descriptor symbol (`name(disambiguator).`). Ported verbatim.
fn symbol_fn_name(symbol: &str) -> Option<&str> {
    let without_suffix = symbol.strip_suffix(").")?;
    let paren_pos = without_suffix.rfind('(')?;
    last_segment(&without_suffix[..paren_pos])
}

/// The bare display name of ANY symbol kind (Type `#`, Term `.`, Method `(...)`). Ported verbatim.
fn bare_name_general(symbol: &str) -> Option<&str> {
    if symbol.starts_with("local ") {
        return None;
    }
    if let Some(name) = symbol_fn_name(symbol) {
        return Some(name);
    }
    let trimmed = symbol.trim_end_matches(['#', '.', ':', ')', ']']);
    last_segment(trimmed)
}

#[derive(Default)]
struct SymbolAgg {
    kinds: HashSet<ScipKind>,
    symbols: HashSet<String>,
    has_is_implementation: bool,
}

fn all_symbol_information(index: &Index) -> impl Iterator<Item = &SymbolInformation> {
    index
        .documents
        .iter()
        .flat_map(|d| d.symbols.iter())
        .chain(index.external_symbols.iter())
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Bucket {
    Ctor,
    Macro,
    Dispatch,
    Collision,
    NotInScip,
    Unexplained,
}

impl Bucket {
    const fn label(self) -> &'static str {
        match self {
            Self::Ctor => "CTOR-ARTIFACT",
            Self::Macro => "MACRO",
            Self::Dispatch => "DISPATCH-CANDIDATE",
            Self::Collision => "COLLISION-CANDIDATE",
            Self::NotInScip => "NOT-IN-SCIP",
            Self::Unexplained => "UNEXPLAINED",
        }
    }
}

fn categorize(callee: &str, map: &HashMap<String, SymbolAgg>) -> Bucket {
    let Some(agg) = map.get(callee) else {
        return Bucket::NotInScip;
    };
    if agg
        .kinds
        .iter()
        .any(|k| matches!(k, ScipKind::EnumMember | ScipKind::Struct | ScipKind::Enum))
    {
        return Bucket::Ctor;
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
        return Bucket::Dispatch;
    }
    if agg.symbols.len() >= 2 {
        return Bucket::Collision;
    }
    Bucket::Unexplained
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        100.0 * n as f64 / d as f64
    }
}

fn main() {
    let root = Path::new(".");
    let scip_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!("== Are the UNEXPLAINED edges cfg-gated? (the missing-axis hypothesis) ==\n");

    // 1. Syntactic edges + their cfg context.
    let mut pair_cfg: HashMap<(String, String), PairCfg> = HashMap::new();
    for dir in CRATE_DIRS {
        let d = root.join(dir);
        if !d.exists() {
            continue;
        }
        for entry in WalkDir::new(&d).into_iter().filter_map(Result::ok) {
            if entry.path().extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let Ok(file) = syn::parse_file(&src) else {
                continue;
            };
            let mut v = CfgCallVisitor::default();
            v.visit_file(&file);
            for (caller, callee, cfgs) in v.edges {
                let e = pair_cfg.entry((caller, callee)).or_default();
                e.total += 1;
                if !cfgs.is_empty() {
                    e.gated += 1;
                    for c in cfgs {
                        if !e.conds.contains(&c) {
                            e.conds.push(c);
                        }
                    }
                }
            }
        }
    }
    let syn_pairs: HashSet<(String, String)> = pair_cfg.keys().cloned().collect();

    // 2. SCIP-resolved pairs (bare-name reduction, matching the other examples).
    let Ok(bytes) = std::fs::read(Path::new(&scip_path)) else {
        println!("could not read {scip_path}");
        return;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        println!("could not parse {scip_path}");
        return;
    };
    // Resolved-only, matching `why_unconfirmed`'s baseline exactly (Ambiguous edges are the
    // macro-degraded set — including them here would silently diverge from the set the
    // syntactic-only measurement was taken against).
    let recons = ingest_scip(Path::new(&scip_path));
    let mut scip_pairs: HashSet<(String, String)> = HashSet::new();
    for r in &recons {
        let EdgeReconstruction::Resolved(e) = r else {
            continue;
        };
        if let (Some(a), Some(b)) = (symbol_fn_name(&e.src_symbol), symbol_fn_name(&e.dst_symbol)) {
            scip_pairs.insert((a.to_string(), b.to_string()));
        }
    }

    // 3. name -> SCIP identity.
    let mut name_map: HashMap<String, SymbolAgg> = HashMap::new();
    for si in all_symbol_information(&index) {
        let Some(n) = bare_name_general(&si.symbol) else {
            continue;
        };
        let e = name_map.entry(n.to_string()).or_default();
        if let Ok(k) = si.kind.enum_value() {
            e.kinds.insert(k);
        }
        e.symbols.insert(si.symbol.clone());
        if si.relationships.iter().any(|r| r.is_implementation) {
            e.has_is_implementation = true;
        }
    }

    // 4. Split + measure.
    let confirmed: Vec<&(String, String)> = syn_pairs
        .iter()
        .filter(|p| scip_pairs.contains(*p))
        .collect();
    let only: Vec<&(String, String)> = syn_pairs
        .iter()
        .filter(|p| !scip_pairs.contains(*p))
        .collect();
    let gated = |ps: &[&(String, String)]| ps.iter().filter(|p| pair_cfg[**p].all_gated()).count();

    let base_gated = syn_pairs
        .iter()
        .filter(|p| pair_cfg[*p].all_gated())
        .count();
    println!("== The control: cfg-gated base rates ==");
    println!(
        "  ALL syntactic pairs:        {:5}   cfg-gated: {:4}  ({:.1}%)   ← the BASE RATE",
        syn_pairs.len(),
        base_gated,
        pct(base_gated, syn_pairs.len())
    );
    println!(
        "  SCIP-CONFIRMED pairs:       {:5}   cfg-gated: {:4}  ({:.1}%)",
        confirmed.len(),
        gated(&confirmed),
        pct(gated(&confirmed), confirmed.len())
    );
    println!(
        "  syntactic-only pairs:       {:5}   cfg-gated: {:4}  ({:.1}%)\n",
        only.len(),
        gated(&only),
        pct(gated(&only), only.len())
    );

    println!("== cfg-gated rate per bucket (enrichment over the base rate is the finding) ==");
    let mut by_bucket: HashMap<Bucket, Vec<&(String, String)>> = HashMap::new();
    for p in &only {
        by_bucket
            .entry(categorize(&p.1, &name_map))
            .or_default()
            .push(p);
    }
    for b in [
        Bucket::Ctor,
        Bucket::Macro,
        Bucket::Dispatch,
        Bucket::Collision,
        Bucket::NotInScip,
        Bucket::Unexplained,
    ] {
        let ps = by_bucket.get(&b).cloned().unwrap_or_default();
        let g = gated(&ps);
        let rate = pct(g, ps.len());
        let enrich = if base_gated == 0 {
            0.0
        } else {
            rate / pct(base_gated, syn_pairs.len())
        };
        println!(
            "  {:20} {:5} pairs   cfg-gated: {:4}  ({:5.1}%)   enrichment: {:.1}x",
            b.label(),
            ps.len(),
            g,
            rate,
            enrich
        );
    }

    // 5. The residue itself.
    let unexplained = by_bucket
        .get(&Bucket::Unexplained)
        .cloned()
        .unwrap_or_default();
    println!("\n== The residue: which cfg conditions gate them? ==");
    let mut cond_hist: BTreeMap<String, usize> = BTreeMap::new();
    for p in &unexplained {
        for c in &pair_cfg[*p].conds {
            *cond_hist.entry(c.clone()).or_default() += 1;
        }
    }
    if cond_hist.is_empty() {
        println!("  (none of the UNEXPLAINED pairs sit under any cfg)");
    } else {
        let mut v: Vec<_> = cond_hist.into_iter().collect();
        v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        for (c, n) in v.iter().take(12) {
            println!("    {n:4}  cfg({c})");
        }
    }
    // Our OWN pipeline asymmetry: `ingest_scip` drops self-recursion (`src == dst`), the syn walk
    // keeps it — so every recursive fn is manufactured into the residue by us, not by SCIP.
    let self_edges = unexplained.iter().filter(|p| p.0 == p.1).count();
    println!(
        "\n  SELF-EDGES (recursion) in the residue: {self_edges} / {}  ({:.1}%)  ← ingest_scip \
         drops `src == dst`; the syn walk keeps it. OUR asymmetry, not SCIP's.",
        unexplained.len(),
        pct(self_edges, unexplained.len())
    );

    println!("\n  sample UNEXPLAINED edges:");
    for p in unexplained.iter().take(8) {
        let pc = &pair_cfg[*p];
        let tag = if pc.all_gated() {
            format!("cfg({})", pc.conds.join(" ∧ "))
        } else {
            "no cfg".to_string()
        };
        println!("    {} -> {}   [{}]", p.0, p.1, tag);
    }
}
