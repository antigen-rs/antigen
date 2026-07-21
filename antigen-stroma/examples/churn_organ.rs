//! The CHURN organ — flip git history into the call graph, per FUNCTION. A first VC-strata tier-0 organ.
//!
//! Node-as-container in action: a SCIP symbol node (code) is joined with its git change-history (process) on
//! the SAME node. For each fn-def, its SCIP def-span (file + line range) is intersected with every commit's
//! changed line ranges (git diff hunks), giving **churn = how many distinct commits touched this function's
//! body**. That is the "refactor at your own risk — N people failed" meme as a measurement: high churn = a
//! function that is central, fragile, or tricky-nobody-understands.
//!
//! This is the cheap-but-real version: ONE `git log` per file (not per function), hunks attributed to the
//! functions they overlap. Joining with the SZZ fix-rate (which commits were bug-fixes) is the next layer —
//! `cursed = churn × fix-rate × co-change` — and lives in `cargo-antigen` where the git/SZZ miner already is.
//!
//! Usage: `cargo run -p antigen-stroma --example churn_organ -- [repo_root] [scip_path]`
//! Defaults to antigen's own tree + index. Needs a NON-shallow clone (a snapshot has no history to mine).

#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use antigen_stroma::syn_ingest::bare_fn_name;
use protobuf::Message;
use scip::types::{Index, SymbolRole};

/// A function's identity-plus-location: the code node, ready to receive a temporal attribute.
struct FnSpan {
    bare: String,
    start: i32,
    end: i32,
}

/// SCIP def-spans grouped by file: `relative_path -> [FnSpan]`. Uses the `enclosing_range` (full body) when
/// present, else the definition occurrence's own range (the signature line) as a proxy.
fn fn_spans_by_file(scip_path: &Path) -> HashMap<String, Vec<FnSpan>> {
    let mut out: HashMap<String, Vec<FnSpan>> = HashMap::new();
    let Ok(bytes) = std::fs::read(scip_path) else {
        return out;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return out;
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) == 0 {
                continue;
            }
            let Some(bare) = bare_fn_name(&occ.symbol) else {
                continue;
            };
            // enclosing_range or range: [start_line, start_col, end_line, end_col] (0-based lines).
            let r = if occ.enclosing_range.len() >= 3 {
                &occ.enclosing_range
            } else {
                &occ.range
            };
            if r.len() >= 3 {
                let (start, end) = (r[0], r[2]);
                out.entry(doc.relative_path.clone())
                    .or_default()
                    .push(FnSpan {
                        bare: bare.to_string(),
                        start,
                        end,
                    });
            }
        }
    }
    out
}

/// Changed line ranges per commit for one file, from `git log --unified=0`. Returns a Vec of (commit, hunks)
/// where each hunk is the (start, end) of the NEW side (`+c,d` → c..c+d).
fn file_commit_hunks(repo: &Path, file: &str) -> Vec<Vec<(i32, i32)>> {
    let out = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--no-merges",
            "--unified=0",
            "--format=%x00",
            "--",
            file,
        ])
        .output();
    let Ok(out) = out else { return Vec::new() };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut commits: Vec<Vec<(i32, i32)>> = Vec::new();
    for line in text.lines() {
        if line.starts_with('\0') {
            commits.push(Vec::new());
        } else if let Some(h) = line.strip_prefix("@@") {
            // @@ -a,b +c,d @@  — take the +c,d
            if let Some(plus) = h.split('+').nth(1) {
                let nums = plus.split('@').next().unwrap_or("").trim();
                let mut it = nums.split(',');
                if let Some(c) = it.next().and_then(|s| s.trim().parse::<i32>().ok()) {
                    let d = it
                        .next()
                        .and_then(|s| s.trim().parse::<i32>().ok())
                        .unwrap_or(1);
                    if let Some(cur) = commits.last_mut() {
                        cur.push((c, c + d.max(1)));
                    }
                }
            }
        }
    }
    commits
}

fn main() {
    let repo = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "R:/antigen".to_string());
    let scip = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let repo = Path::new(&repo);
    println!("== CHURN organ: per-function commit-touch count (VC strata on the call graph) ==\n");

    let by_file = fn_spans_by_file(Path::new(&scip));
    if by_file.is_empty() {
        println!("(no fn-def spans from {scip})");
        return;
    }

    // churn[bare] = distinct commits whose hunks overlapped this function's span.
    let mut churn: HashMap<String, usize> = HashMap::new();
    let mut files_seen = 0usize;
    for (file, fns) in &by_file {
        let commits = file_commit_hunks(repo, file);
        if commits.is_empty() {
            continue;
        }
        files_seen += 1;
        for hunks in &commits {
            for f in fns {
                if hunks
                    .iter()
                    .any(|&(cs, ce)| cs <= f.end + 1 && ce > f.start)
                {
                    *churn.entry(f.bare.clone()).or_default() += 1;
                }
            }
        }
    }

    let mut ranked: Vec<(&String, usize)> = churn.iter().map(|(k, v)| (k, *v)).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let total_fns: usize = by_file.values().map(Vec::len).sum();
    let touched = ranked.len();

    println!(
        "  files with history: {files_seen}   fn-defs: {total_fns}   functions ever touched: {touched}"
    );
    println!("\n  MOST-CHURNED functions (the 'refactor at your own risk' set):");
    for (name, c) in ranked.iter().take(15) {
        println!("    {c:3} commits  {name}");
    }
    let write_once = ranked.iter().filter(|(_, c)| *c == 1).count();
    println!(
        "\n  => {write_once} functions were touched in exactly ONE commit (write-once/stable); the long tail is\n\
         \x20    the churn distribution. High-churn functions are the temporal risk signal the CODE alone\n\
         \x20    cannot show -- a tier-0 organ over a NEW instrument (git), on the same nodes as the code organs.\n\
         \x20    Next layer: join with SZZ fix-rate -> cursed = churn x fix-rate x co-change."
    );
}
