//! Would body-span containment (the CORRECT caller attribution) actually cover our references?
//!
//! `enclosing_range` on a DEFINITION occurrence is that definition's full body span (settled by
//! `probe_enclosing`: it never names a different symbol). So the exact caller of a reference at
//! line L is the fn whose body-span CONTAINS L — no line-proximity guessing. This probe measures
//! whether the index carries enough definition body-spans to make that attribution real.

#![allow(clippy::cast_precision_loss)]

use std::path::Path;

use protobuf::Message;
use scip::types::{Index, SymbolRole};

fn is_fn_symbol(s: &str) -> bool {
    !s.starts_with("local ") && s.ends_with(").")
}

fn start_line(r: &[i32]) -> i32 {
    r.first().copied().unwrap_or(0)
}

/// `[sl, sc, ec]` (single-line) or `[sl, sc, el, ec]` (multi-line).
fn end_line(r: &[i32]) -> i32 {
    if r.len() >= 4 {
        r[2]
    } else {
        r.first().copied().unwrap_or(0)
    }
}

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

    let mut fn_defs = 0usize;
    let mut fn_defs_with_span = 0usize;
    let mut multiline = 0usize;
    let mut refs = 0usize;
    let mut covered = 0usize;
    let mut uncovered = 0usize;

    for doc in &index.documents {
        // Body spans (start, end, symbol) from DEFINITION occurrences.
        let mut spans: Vec<(i32, i32, &str)> = Vec::new();
        for o in &doc.occurrences {
            if o.symbol_roles & (SymbolRole::Definition as i32) != 0 && is_fn_symbol(&o.symbol) {
                fn_defs += 1;
                if !o.enclosing_range.is_empty() {
                    fn_defs_with_span += 1;
                    let (s, e) = (start_line(&o.enclosing_range), end_line(&o.enclosing_range));
                    if e > s {
                        multiline += 1;
                    }
                    spans.push((s, e, o.symbol.as_str()));
                }
            }
        }
        for o in &doc.occurrences {
            if o.symbol_roles & (SymbolRole::Definition as i32) != 0 || !is_fn_symbol(&o.symbol) {
                continue;
            }
            if o.symbol.is_empty() || o.symbol.starts_with("local ") {
                continue;
            }
            refs += 1;
            let l = start_line(&o.range);
            // The innermost containing body span = the true caller.
            let best = spans
                .iter()
                .filter(|&&(s, e, _)| s <= l && l <= e)
                .min_by_key(|&&(s, e, _)| e - s);
            if best.is_some() {
                covered += 1;
            } else {
                uncovered += 1;
            }
        }
    }

    let p = |n: usize, d: usize| {
        if d == 0 {
            0.0
        } else {
            100.0 * n as f64 / d as f64
        }
    };
    println!("fn DEFINITION occurrences:            {fn_defs}");
    println!(
        "  ... carrying a body span:           {fn_defs_with_span}  ({:.1}%)",
        p(fn_defs_with_span, fn_defs)
    );
    println!(
        "  ... whose span is multi-line:       {multiline}  ({:.1}%)",
        p(multiline, fn_defs_with_span)
    );
    println!("\nfn reference occurrences:             {refs}");
    println!(
        "  CONTAINED in a body span (EXACT caller): {covered}  ({:.1}%)",
        p(covered, refs)
    );
    println!(
        "  not contained (no caller recoverable):   {uncovered}  ({:.1}%)",
        p(uncovered, refs)
    );
}
