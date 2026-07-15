//! Enrichment #2 — what SCIP's symbol table says, which we had never once read.
//!
//! `syn` cannot distinguish `Some(x)` from a function call: they are syntactically identical, and
//! no amount of parsing fixes that. SCIP classifies it as an `EnumMember`. That is the second
//! instrument correcting the first's STRUCTURAL blind spot — and until now we discarded it, then
//! spent a whole measurement re-deriving it post-hoc as a "CTOR-ARTIFACT" bucket.
//!
//! Held, not enforced: this prints what SCIP observed. Nothing is filtered.

#![allow(clippy::cast_precision_loss)]

use std::collections::BTreeMap;
use std::path::Path;

use antigen_stroma::scip::ingest_scip_symbols;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let symbols = ingest_scip_symbols(Path::new(&path));
    println!("== SCIP symbol table (Document.symbols + external_symbols) ==\n");
    println!("  symbols described: {}", symbols.len());
    if symbols.is_empty() {
        println!("  (empty — generate one: rust-analyzer scip . --output target/antigen.scip)");
        return;
    }

    let mut hist: BTreeMap<String, usize> = BTreeMap::new();
    for s in &symbols {
        *hist.entry(format!("{:?}", s.kind)).or_default() += 1;
    }
    let mut v: Vec<_> = hist.into_iter().collect();
    v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    println!("\n  by SCIP kind (held UN-MAPPED — ItemTarget cannot express these):");
    for (k, n) in v.iter().take(14) {
        println!("    {n:6}  {k}");
    }

    let ctors = symbols.iter().filter(|s| s.kind.is_constructor()).count();
    let with_docs = symbols
        .iter()
        .filter(|s| !s.documentation.is_empty())
        .count();
    let with_impl = symbols
        .iter()
        .filter(|s| !s.is_implementation_of.is_empty())
        .count();
    let pct = |n: usize| 100.0 * n as f64 / symbols.len() as f64;

    println!(
        "\n  CONSTRUCTORS (a `call` to these is a CONSTRUCTION — syn structurally cannot tell):"
    );
    println!("    {ctors}  ({:.1}%)", pct(ctors));
    println!(
        "\n  documentation carried:  {with_docs}  ({:.1}%)   <- never read before",
        pct(with_docs)
    );
    println!(
        "  is_implementation_of:   {with_impl}  ({:.1}%)   <- the trait->impl dispatch map",
        pct(with_impl)
    );
    if with_impl == 0 {
        println!(
            "    NOTE: 0 is this INSTRUMENT's silence, not evidence that no impls exist.\n    \
             rust-analyzer's scip emits no relationships; the slot is held open for an index that does."
        );
    }
}
