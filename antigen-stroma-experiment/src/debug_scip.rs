// Analyze phantom categories: what kinds of edges are SCIP rejecting?

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use protobuf::Message;
use scip::types::{Index, SymbolRole};
use syn::visit::Visit;
use walkdir::WalkDir;

fn is_fn_symbol(symbol: &str) -> bool {
    if symbol.starts_with("local ") { return false; }
    symbol.ends_with(").")
}

fn parse_range(range: &[i32]) -> (i32, i32, i32, i32) {
    match range {
        [sl, sc, ec] => (*sl, *sc, *sl, *ec),
        [sl, sc, el, ec] => (*sl, *sc, *el, *ec),
        _ => (0, 0, 0, 0),
    }
}

fn symbol_fn_name(symbol: &str) -> Option<&str> {
    let without_suffix = symbol.strip_suffix(").")?;
    let paren_pos = without_suffix.rfind('(')?;
    let before_paren = &without_suffix[..paren_pos];
    let segment = before_paren
        .rsplit(|c| c == '/' || c == '#')
        .next()
        .filter(|s| !s.is_empty())?;
    if let Some(bracket_end) = segment.rfind(']') {
        let after_bracket = &segment[bracket_end + 1..];
        if !after_bracket.is_empty() {
            return Some(after_bracket);
        }
    }
    if segment.starts_with('`') && segment.ends_with('`') && segment.len() > 2 {
        return Some(&segment[1..segment.len()-1]);
    }
    Some(segment)
}

struct CallVisitor {
    fn_stack: Vec<String>,
    edges: Vec<(String, String)>,
}

impl CallVisitor {
    fn new() -> Self {
        Self { fn_stack: Vec::new(), edges: Vec::new() }
    }
}

fn extract_call_name(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Path(ep) => ep.path.segments.last().map(|s| s.ident.to_string()),
        syn::Expr::Field(ef) => {
            if let syn::Member::Named(n) = &ef.member { Some(n.to_string()) } else { None }
        }
        _ => None,
    }
}

impl<'ast> Visit<'ast> for CallVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        self.fn_stack.push(name);
        syn::visit::visit_item_fn(self, node);
        self.fn_stack.pop();
    }
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        self.fn_stack.push(name);
        syn::visit::visit_impl_item_fn(self, node);
        self.fn_stack.pop();
    }
    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let name = node.sig.ident.to_string();
        self.fn_stack.push(name);
        syn::visit::visit_trait_item_fn(self, node);
        self.fn_stack.pop();
    }
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        let callee = extract_call_name(&node.func);
        if let (Some(caller), Some(callee)) = (self.fn_stack.last(), callee) {
            self.edges.push((caller.clone(), callee));
        }
        syn::visit::visit_expr_call(self, node);
    }
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let callee = node.method.to_string();
        if let Some(caller) = self.fn_stack.last() {
            self.edges.push((caller.clone(), callee));
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

fn syn_edges_from_file(path: &Path) -> Vec<(String, String)> {
    let src = match std::fs::read_to_string(path) { Ok(s) => s, Err(_) => return vec![] };
    let parsed = match syn::parse_file(&src) { Ok(f) => f, Err(_) => return vec![] };
    let mut v = CallVisitor::new();
    v.visit_file(&parsed);
    v.edges
}

fn extract_syn_edges(root: &Path) -> HashMap<String, Vec<(String, String)>> {
    let mut result = HashMap::new();
    let crate_dirs = ["antigen/src", "antigen-stroma/src", "antigen-fingerprint/src",
        "antigen-macros/src", "antigen-attestation/src", "cargo-antigen/src"];
    for dir in &crate_dirs {
        let src_root = root.join(dir);
        if !src_root.exists() { continue; }
        for entry in WalkDir::new(&src_root).into_iter().filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ex| ex == "rs")) {
            let path = entry.path();
            let rel = path.strip_prefix(root).unwrap_or(path);
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let edges = syn_edges_from_file(path);
            if !edges.is_empty() {
                result.entry(rel_str).or_insert_with(Vec::new).extend(edges);
            }
        }
    }
    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scip_path = &args[1];
    let workspace_root = PathBuf::from(&args[2]);

    let bytes = std::fs::read(scip_path).expect("read");
    let index = Index::parse_from_bytes(&bytes).expect("parse");

    let workspace_crates = std::collections::HashSet::from([
        "antigen/src", "antigen-stroma/src", "antigen-fingerprint/src",
        "antigen-macros/src", "antigen-attestation/src", "cargo-antigen/src",
    ]);

    // Build SCIP name pairs.
    let mut scip_names = std::collections::HashSet::new();
    for doc in &index.documents {
        let path = doc.relative_path.replace('\\', "/");
        if !workspace_crates.iter().any(|c| path.starts_with(c)) { continue; }

        let def_fns: Vec<(i32, &str)> = doc.occurrences.iter()
            .filter(|occ| {
                occ.symbol_roles & (SymbolRole::Definition as i32) != 0
                    && is_fn_symbol(&occ.symbol)
            })
            .map(|occ| (parse_range(&occ.range).0, occ.symbol.as_str()))
            .collect();

        for occ in &doc.occurrences {
            let is_def = occ.symbol_roles & (SymbolRole::Definition as i32) != 0;
            if is_def || !is_fn_symbol(&occ.symbol) || occ.symbol.starts_with("local ") { continue; }
            let (ref_line, _, _, _) = parse_range(&occ.range);

            let encloser = if !occ.enclosing_range.is_empty() {
                let (esl, _, _, _) = parse_range(&occ.enclosing_range);
                def_fns.iter().find(|&&(sl, _)| sl == esl).map(|&(_, s)| s)
            } else {
                def_fns.iter().filter(|&&(sl, _)| sl <= ref_line)
                    .max_by_key(|&&(sl, _)| sl).map(|&(_, s)| s)
            };

            if let (Some(caller), Some(callee_name)) = (encloser.and_then(symbol_fn_name), symbol_fn_name(&occ.symbol)) {
                if caller != callee_name {
                    scip_names.insert((caller.to_string(), callee_name.to_string()));
                }
            }
        }
    }

    // Get syn edges.
    let syn_map = extract_syn_edges(&workspace_root);

    // Categorize phantoms.
    let mut phantom_enum_ctor = 0usize; // starts with uppercase → likely enum variant
    let mut phantom_macro_ctor = 0usize; // common Rust macros (vec!, format!, etc.)
    let mut phantom_generic = 0usize; // lowercase, in scip but under different caller
    let mut phantom_not_in_scip = 0usize; // genuinely not in any SCIP pair

    let all_scip_callees: std::collections::HashSet<String> = scip_names.iter().map(|(_, c)| c.clone()).collect();
    let common_macros = std::collections::HashSet::from(["vec", "format", "println", "eprintln",
        "assert", "assert_eq", "assert_ne", "panic", "todo", "unimplemented", "unreachable",
        "write", "writeln", "dbg", "concat", "include_str", "env"]);

    for (_file, edges) in &syn_map {
        for (caller, callee) in edges {
            if scip_names.contains(&(caller.clone(), callee.clone())) {
                continue; // confirmed
            }
            // Categorize this phantom.
            let first_char = callee.chars().next().unwrap_or('_');
            if first_char.is_uppercase() {
                phantom_enum_ctor += 1;
            } else if common_macros.contains(callee.as_str()) {
                phantom_macro_ctor += 1;
            } else if all_scip_callees.contains(callee) {
                // callee exists in SCIP but under a different caller-name
                phantom_generic += 1;
            } else {
                phantom_not_in_scip += 1;
            }
        }
    }

    let total_phantoms = phantom_enum_ctor + phantom_macro_ctor + phantom_generic + phantom_not_in_scip;
    println!("Phantom edge categories:");
    println!("  Uppercase callee (enum ctors / type paths): {phantom_enum_ctor}  ({:.1}%)", phantom_enum_ctor as f64 / total_phantoms as f64 * 100.0);
    println!("  Common macro names: {phantom_macro_ctor}  ({:.1}%)", phantom_macro_ctor as f64 / total_phantoms as f64 * 100.0);
    println!("  Callee in SCIP, different caller: {phantom_generic}  ({:.1}%)", phantom_generic as f64 / total_phantoms as f64 * 100.0);
    println!("  Callee not in SCIP at all: {phantom_not_in_scip}  ({:.1}%)", phantom_not_in_scip as f64 / total_phantoms as f64 * 100.0);
    println!("  Total phantoms: {total_phantoms}");
    println!();

    // Show a sample of the "not in SCIP at all" category.
    println!("Sample of 'callee not in SCIP' phantoms (first 20):");
    let mut count = 0;
    'outer: for (file, edges) in &syn_map {
        for (caller, callee) in edges {
            if scip_names.contains(&(caller.clone(), callee.clone())) { continue; }
            let first_char = callee.chars().next().unwrap_or('_');
            if !first_char.is_uppercase() && !common_macros.contains(callee.as_str()) && !all_scip_callees.contains(callee) {
                println!("  {file}  {caller} -> {callee}");
                count += 1;
                if count >= 20 { break 'outer; }
            }
        }
    }
}
