//! Does SCIP EMIT constructor references, or does OUR gate drop them?
//!
//! `ingest_scip` gates reference occurrences through `is_fn_symbol` (symbol must end in `").`), so
//! no `EdgeFact` can target a constructor (`EnumMember` / tuple-`Struct`). The question: is that a SCIP
//! blindness (it never records a reference to `Some` at a callsite) or our own filter throwing away
//! data SCIP already has? This probe counts SCIP reference-occurrences whose symbol is a
//! constructor-kind — the construction callsites we are currently discarding.

#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::path::Path;

use antigen_stroma::base::facts::ScipKind;
use protobuf::Message;
use scip::types::{Index, SymbolRole};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let Ok(bytes) = std::fs::read(Path::new(&path)) else {
        println!("could not read {path}");
        return;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        println!("could not parse {path}");
        return;
    };

    // symbol string -> SCIP kind (from the symbol table, both feeders).
    let mut kind: HashMap<&str, ScipKind> = HashMap::new();
    for si in index
        .documents
        .iter()
        .flat_map(|d| d.symbols.iter())
        .chain(index.external_symbols.iter())
    {
        kind.insert(si.symbol.as_str(), ScipKind::from_raw(si.kind.value()));
    }

    let mut ref_total = 0usize; // all non-definition occurrences with a known symbol
    let mut ref_ctor = 0usize; // ... whose symbol is a constructor kind
    let mut ref_ctor_with_encloser = 0usize; // ... AND sitting inside a fn body span (reconstructable)
    let mut samples: Vec<(&str, String)> = Vec::new();

    for doc in &index.documents {
        // fn body spans (definition enclosing_range) — the caller a construction edge would attach to.
        let mut spans: Vec<(i32, i32)> = Vec::new();
        for o in &doc.occurrences {
            if o.symbol_roles & (SymbolRole::Definition as i32) != 0
                && o.symbol.ends_with(").")
                && !o.enclosing_range.is_empty()
            {
                let r = &o.enclosing_range;
                let (s, e) = (
                    r.first().copied().unwrap_or(0),
                    if r.len() >= 4 { r[2] } else { 0 },
                );
                spans.push((s, e));
            }
        }
        for o in &doc.occurrences {
            if o.symbol_roles & (SymbolRole::Definition as i32) != 0 || o.symbol.is_empty() {
                continue;
            }
            let Some(&k) = kind.get(o.symbol.as_str()) else {
                continue;
            };
            ref_total += 1;
            if k.is_constructor() {
                ref_ctor += 1;
                let line = o.range.first().copied().unwrap_or(0);
                if spans.iter().any(|&(s, e)| s <= line && line <= e) {
                    ref_ctor_with_encloser += 1;
                }
                if samples.len() < 8 {
                    samples.push((o.symbol.as_str(), format!("{k:?}")));
                }
            }
        }
    }

    let pct = |n: usize, d: usize| {
        if d == 0 {
            0.0
        } else {
            100.0 * n as f64 / d as f64
        }
    };
    println!("== Does SCIP emit constructor references we are dropping? ==\n");
    println!("  non-definition references with a known symbol:   {ref_total}");
    println!(
        "  ... whose symbol is a CONSTRUCTOR kind:          {ref_ctor}  ({:.1}%)",
        pct(ref_ctor, ref_total)
    );
    println!(
        "  ... AND inside a fn body span (reconstructable): {ref_ctor_with_encloser}  ({:.1}% of ctor refs)",
        pct(ref_ctor_with_encloser, ref_ctor)
    );
    println!("\n  sample dropped constructor references:");
    for (sym, k) in &samples {
        let short = sym.rsplit(' ').next().unwrap_or(sym);
        println!("    [{k:>10}] {short}");
    }
    println!("\n== Verdict ==");
    if ref_ctor_with_encloser > 100 {
        println!(
            "  SCIP EMITS them — {ref_ctor_with_encloser} reconstructable construction callsites are\n  \
             being DROPPED by our `is_fn_symbol` `).`-suffix gate. This is OUR gate, not SCIP blindness:\n  \
             relax the gate + tag these as a Construct edge-kind. No new tool needed."
        );
    } else {
        println!(
            "  SCIP does NOT usefully emit reconstructable constructor references ({ref_ctor_with_encloser}).\n  \
             A construction edge needs another source (syn already sees them; MIR is exact)."
        );
    }
}
