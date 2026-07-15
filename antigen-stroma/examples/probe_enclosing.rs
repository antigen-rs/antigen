//! What does `enclosing_range` actually MEAN in our index? (settle it against the data)
//!
//! `ingest_scip` reads `occurrence.enclosing_range` as *"the span of the function that encloses this
//! reference"* (→ the caller). The SCIP field is deprecated and its doc is stripped from the
//! generated Rust, so the semantics is unverified. The competing reading: it is *"the full span of
//! the definition THIS occurrence defines"* (definition semantics) — in which case using it to name
//! a caller is a bug.
//!
//! Decisive test: for occurrences carrying `enclosing_range`, does the DEF at its start line carry
//! the occurrence's OWN symbol? If yes → definition semantics → Method 1 is misusing the field.

// A single-shot measurement probe: one linear pass + its report reads best as one unit.
#![allow(clippy::cast_precision_loss, clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::path::Path;

use protobuf::Message;
use scip::types::{Index, SymbolRole};

fn is_fn_symbol(s: &str) -> bool {
    !s.starts_with("local ") && s.ends_with(").")
}

fn start_line(r: &[i32]) -> i32 {
    r.first().copied().unwrap_or(0)
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

    let mut roles_hist: BTreeMap<i32, usize> = BTreeMap::new();
    let mut with_enc = 0usize;
    let mut enc_is_self = 0usize; // enclosing_range's DEF line carries THIS occurrence's symbol
    let mut enc_is_other = 0usize; // ... carries a DIFFERENT symbol (true "enclosing fn" reading)
    let mut enc_no_def = 0usize; // no DEF at that line at all
    let mut enc_equals_own_range = 0usize; // enclosing_range.start == this occurrence's own line

    for doc in &index.documents {
        let defs: BTreeMap<i32, &str> = doc
            .occurrences
            .iter()
            .filter(|o| {
                o.symbol_roles & (SymbolRole::Definition as i32) != 0 && is_fn_symbol(&o.symbol)
            })
            .map(|o| (start_line(&o.range), o.symbol.as_str()))
            .collect();

        for occ in &doc.occurrences {
            if occ.enclosing_range.is_empty() || !is_fn_symbol(&occ.symbol) {
                continue;
            }
            with_enc += 1;
            *roles_hist.entry(occ.symbol_roles).or_default() += 1;
            let el = start_line(&occ.enclosing_range);
            if el == start_line(&occ.range) {
                enc_equals_own_range += 1;
            }
            match defs.get(&el) {
                Some(&sym) if sym == occ.symbol.as_str() => enc_is_self += 1,
                Some(_) => enc_is_other += 1,
                None => enc_no_def += 1,
            }
        }
    }

    let pct = |n: usize| {
        if with_enc == 0 {
            0.0
        } else {
            100.0 * n as f64 / with_enc as f64
        }
    };

    println!("== Occurrences carrying `enclosing_range` (fn symbols): {with_enc} ==\n");
    println!("  symbol_roles histogram (1=Definition, 0=plain reference, 16=Generated, 64=Fwd):");
    for (roles, n) in &roles_hist {
        let mut tags = Vec::new();
        if roles & 1 != 0 {
            tags.push("Definition");
        }
        if roles & 2 != 0 {
            tags.push("Import");
        }
        if roles & 16 != 0 {
            tags.push("Generated");
        }
        if roles & 32 != 0 {
            tags.push("Test");
        }
        if roles & 64 != 0 {
            tags.push("ForwardDefinition");
        }
        if tags.is_empty() {
            tags.push("(none = plain reference)");
        }
        println!("    roles={roles:3} → {n:6}  [{}]", tags.join(", "));
    }

    println!("\n  What does enclosing_range's start line point at?");
    println!(
        "    a DEF carrying THIS occurrence's OWN symbol: {enc_is_self:6}  ({:.1}%)  ← definition semantics",
        pct(enc_is_self)
    );
    println!(
        "    a DEF carrying a DIFFERENT symbol:          {enc_is_other:6}  ({:.1}%)  ← \"enclosing fn\" semantics",
        pct(enc_is_other)
    );
    println!(
        "    no DEF at that line at all:                 {enc_no_def:6}  ({:.1}%)",
        pct(enc_no_def)
    );
    println!(
        "    (enclosing_range.start == occurrence's own line: {enc_equals_own_range}  {:.1}%)",
        pct(enc_equals_own_range)
    );

    println!("\n== Verdict ==");
    if enc_is_self > enc_is_other {
        println!(
            "  DEFINITION semantics: enclosing_range spans the definition THIS occurrence defines\n  \
             (its own body), NOT the function enclosing a reference. `ingest_scip`'s Method 1 is\n  \
             MISUSING the field — and any test that treats it as caller ground-truth is invalid."
        );
    } else {
        println!(
            "  ENCLOSING-FN semantics: enclosing_range names a DIFFERENT (enclosing) definition, so\n  \
             reading it as the caller is sound."
        );
    }
}
