//! `substrate` — the PERSISTENT content-addressed manifold (the water-holding one).
//!
//! Unlike `throw_at_addresses.rs` (which rebuilds the whole pile in memory every run and lets it
//! evaporate on exit), this is a manifold that **holds its water**: each firehose is an incremental
//! writer that APPENDS records to a store on disk and STAYS; fusion happens on *read* (fold by
//! address); organs read the accumulation. Fire one valve, save; fire the next later, it lands on the
//! same addresses and joins. "All at once IN THE SUBSTRATE, one valve at a time."
//!
//! This is the `stromix-core` shape (`Record` + `CellAddress` + fusion) with a persistence layer — the
//! append-only-log-is-truth, projection-is-a-rebuildable-fold model. It graduates to `stromix-core`.
//!
//! Usage:
//!   cargo run -p antigen-stroma --example substrate -- <store> write <repo> <scip> <hose>  # fire ONE hose
//!   cargo run -p antigen-stroma --example substrate -- <store> fold                        # read the accumulation
//! Hoses: scip-defs, scip-refs, git-commits, clone-shape (more added one at a time).

use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::Command;

use antigen_stroma::syn_ingest::bare_fn_name;
use protobuf::Message;
use scip::types::{Index, SymbolRole};

/// What a firehose pours at an address. `(kind, key)` = a candidate address; `source` = which hose;
/// `content` = the observation. No "kind"-as-partition — kind is just part of the address key.
struct Record {
    kind: &'static str,
    key: String,
    source: &'static str,
    content: String,
}

fn sanitize(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

/// APPEND records to the persistent store (append-only log; the log is the source of truth). Fusion is
/// NOT done here — it happens on read (`fold`), so a later hose's records simply join earlier ones.
fn append(store: &Path, records: &[Record]) -> std::io::Result<usize> {
    let f = OpenOptions::new().create(true).append(true).open(store)?;
    let mut w = BufWriter::new(f);
    for r in records {
        writeln!(
            w,
            "{}\t{}\t{}\t{}",
            r.kind,
            sanitize(&r.key),
            r.source,
            sanitize(&r.content)
        )?;
    }
    w.flush()?;
    Ok(records.len())
}

/// Fold the whole log into the `address -> multi-content` view. This is the projection; it is
/// rebuildable from the log at any time, and it is where fusion (same address, many records) appears.
fn fold(store: &Path) -> HashMap<(String, String), Vec<(String, String)>> {
    let mut m: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    let Ok(f) = std::fs::File::open(store) else {
        return m;
    };
    for line in BufReader::new(f).lines().map_while(Result::ok) {
        let mut it = line.splitn(4, '\t');
        if let (Some(k), Some(key), Some(src), Some(con)) =
            (it.next(), it.next(), it.next(), it.next())
        {
            m.entry((k.to_string(), key.to_string()))
                .or_default()
                .push((src.to_string(), con.to_string()));
        }
    }
    m
}

fn read_index(scip: &Path) -> Option<Index> {
    let bytes = std::fs::read(scip).ok()?;
    Index::parse_from_bytes(&bytes).ok()
}

/// `(symbol, bare, path, start_line, end_line)` for every fn-def. Shared by the SCIP-based hoses.
type FnDef = (String, String, String, i32, i32);

/// `path -> [(bare_fn_name, start_line, end_line)]` — every fn-def's span, grouped by file. Used
/// by `hose_git_churn`'s single-fn/multi-fn split and its budgeted hunk-overlap pass.
type FileSpans = HashMap<String, Vec<(String, i32, i32)>>;
fn fn_defs(scip: &Path) -> Vec<FnDef> {
    let mut out = Vec::new();
    let Some(index) = read_index(scip) else {
        return out;
    };
    for doc in &index.documents {
        let path = doc.relative_path.replace('\\', "/");
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) == 0 {
                continue;
            }
            let Some(bare) = bare_fn_name(&occ.symbol) else {
                continue;
            };
            let r = if occ.enclosing_range.len() >= 3 {
                &occ.enclosing_range
            } else {
                &occ.range
            };
            if r.len() < 3 {
                continue;
            }
            out.push((
                occ.symbol.clone(),
                bare.to_string(),
                path.clone(),
                r[0],
                r[2],
            ));
        }
    }
    out
}

/// Firehose: SCIP function definitions -> records at THREE candidate addresses (precise symbol,
/// bare name, `(file, span)`). The three granularities we learned behave differently.
fn hose_scip_defs(scip: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    for (symbol, bare, path, s, e) in fn_defs(scip) {
        let content = format!("fn-def:{bare}");
        recs.push(Record {
            kind: "Precise",
            key: symbol,
            source: "scip-defs",
            content: content.clone(),
        });
        recs.push(Record {
            kind: "Bare",
            key: bare,
            source: "scip-defs",
            content: content.clone(),
        });
        recs.push(Record {
            kind: "Span",
            key: format!("{path}@{s}-{e}"),
            source: "scip-defs",
            content,
        });
    }
    recs
}

/// Firehose: SCIP references (non-def occurrences) -> records at the referenced symbol + bare name.
/// Its records land on the SAME addresses the defs did, so folding shows a function's address holding
/// both `scip-defs` AND `scip-refs` content: fusion on disk.
fn hose_scip_refs(scip: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    let Some(index) = read_index(scip) else {
        return recs;
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) != 0 {
                continue; // references only
            }
            let Some(bare) = bare_fn_name(&occ.symbol) else {
                continue;
            };
            recs.push(Record {
                kind: "Precise",
                key: occ.symbol.clone(),
                source: "scip-refs",
                content: "ref".into(),
            });
            recs.push(Record {
                kind: "Bare",
                key: bare.to_string(),
                source: "scip-refs",
                content: "ref".into(),
            });
        }
    }
    recs
}

/// Firehose: git commits -> records at commit-SHA and author-email addresses (a NEW address space —
/// the temporal / authorship manifold). One `git log` pass. Opens the time axis of the substrate.
fn hose_git_commits(repo: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    let Ok(out) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--all",
            "--no-merges",
            "--format=%H%x00%ae",
        ])
        .output()
    else {
        return recs;
    };
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut it = line.splitn(2, '\0');
        if let (Some(sha), Some(email)) = (it.next(), it.next()) {
            let short = &sha[..sha.len().min(10)];
            recs.push(Record {
                kind: "Sha",
                key: sha.to_string(),
                source: "git-log",
                content: "commit".into(),
            });
            recs.push(Record {
                kind: "Author",
                key: email.to_string(),
                source: "git-log",
                content: format!("commit:{short}"),
            });
        }
    }
    recs
}

/// Firehose: code-clone shape -> records at each function's bare name AND at a `CloneShape` address
/// keyed by a hash of its identifier-token-multiset. Functions with the same shape collide on the same
/// `CloneShape` address — that collision IS a clone family, the failure-propagation signal. Opens the
/// structural-similarity axis and fuses on the bare-name addresses with the SCIP hoses.
fn hose_clone_shape(repo: &Path, scip: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    let mut file_cache: HashMap<String, Vec<String>> = HashMap::new();
    for (_symbol, bare, path, s, e) in fn_defs(scip) {
        let lines = file_cache.entry(path.clone()).or_insert_with(|| {
            std::fs::read_to_string(repo.join(&path))
                .map(|t| t.lines().map(String::from).collect())
                .unwrap_or_default()
        });
        let a = usize::try_from(s.max(1)).unwrap_or(1) - 1;
        let b = usize::try_from(e).unwrap_or(0).min(lines.len());
        if a >= b {
            continue;
        }
        let mut toks: Vec<&str> = Vec::new();
        for line in &lines[a..b] {
            toks.extend(
                line.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .filter(|t| !t.is_empty()),
            );
        }
        toks.sort_unstable();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for t in &toks {
            t.hash(&mut h);
        }
        let shape = format!("{:016x}", h.finish());
        recs.push(Record {
            kind: "Bare",
            key: bare,
            source: "clone-shape",
            content: format!("shape:{shape}"),
        });
        recs.push(Record {
            kind: "CloneShape",
            key: shape,
            source: "clone-shape",
            content: "fn".into(),
        });
    }
    recs
}

/// Firehose: intent/quality markers (TODO/FIXME/HACK/SAFETY/unsafe) in source -> records at the FILE
/// address (a new axis: the intent manifold). Naive substring — catches prose too (a known instrument
/// hole); we HOLD it, we don't judge it.
fn hose_markers(repo: &Path) -> Vec<Record> {
    const MARKERS: [&str; 5] = ["TODO", "FIXME", "HACK", "SAFETY", "unsafe"];
    let mut recs = Vec::new();
    let mut stack = vec![repo.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for ent in rd.flatten() {
            let p = ent.path();
            let name = ent.file_name();
            let name = name.to_string_lossy();
            if p.is_dir() {
                if name == "target" || name.starts_with('.') {
                    continue;
                }
                stack.push(p);
            } else if name.ends_with(".rs") {
                let Ok(text) = std::fs::read_to_string(&p) else {
                    continue;
                };
                let rel = p
                    .strip_prefix(repo)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .replace('\\', "/");
                for (i, line) in text.lines().enumerate() {
                    for kw in MARKERS {
                        if line.contains(kw) {
                            recs.push(Record {
                                kind: "File",
                                key: rel.clone(),
                                source: "markers",
                                content: format!("{kw}@{}", i + 1),
                            });
                        }
                    }
                }
            }
        }
    }
    recs
}

/// The wall-clock budget for `hunk_churn_budgeted`'s content-diff subprocess. Every one of these
/// corpora except `antigen` is a PARTIAL CLONE (`--filter=blob:none`, promisor remote — confirmed
/// via `git config remote.origin.promisor` on rhai/ripgrep/serde/syn/tokio, live-traced in
/// `throw_at_addresses.rs`'s `commit_hunks_budgeted`), so `git log --unified=0` over full history
/// transparently spawns its OWN `git fetch` child to pull missing blob content on demand — that's
/// network-round-trip-bound, not CPU/disk-bound, can stall for minutes on a first touch, and is
/// NOT bounded by corpus size (a crude per-hose `timeout N` wrapper caps the damage from outside
/// but doesn't give an honest partial result the way a budget-and-salvage does).
const CHURN_HUNK_BUDGET: std::time::Duration = std::time::Duration::from_secs(45);

/// Kill `child` and (on Windows) its whole descendant tree — see `throw_at_addresses.rs`'s
/// `kill_tree` for the live-traced reason a plain `Child::kill` isn't enough: a promisor-remote
/// `git log --unified=0` may have spawned its own `git fetch` grandchild to satisfy a missing
/// blob, and `Child::kill` on Windows only reaches the immediate child, orphaning the fetch.
#[cfg_attr(windows, allow(clippy::needless_pass_by_ref_mut))]
fn kill_tree(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = child.kill();
    }
}

/// The CHEAP pass: a `--name-only` tree-diff, per commit (SHA comparison only — blob CONTENT is
/// never read, so a partial clone answers this with ZERO network fetches; measured 0.13s on cold
/// `rhai`, 0.24s on cold `tokio`'s full history). Returns `file -> total touch count`.
fn commit_touch_counts(repo: &Path) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let Ok(out) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--all",
            "--no-merges",
            "--name-only",
            "--format=@@%H",
        ])
        .output()
    else {
        return counts;
    };
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if line.len() == 42
            && line.starts_with("@@")
            && line[2..].bytes().all(|c| c.is_ascii_hexdigit())
        {
            continue; // a commit-header line, not a path
        }
        if line.is_empty() {
            continue;
        }
        *counts.entry(line.replace('\\', "/")).or_default() += 1;
    }
    counts
}

/// The EXPENSIVE pass, TIME-BOXED: hunk-line-overlap churn for files holding 2+ functions, where
/// only content-diff hunks can say WHICH function a touch belongs to (a single-fn file doesn't
/// need this — see `hose_git_churn`). Spawns `git log --unified=0` with a piped-stdout read on a
/// background thread; if `budget` elapses before the child exits, `kill_tree`s it and parses
/// whatever bytes were captured — same "skipped-and-reported honestly beats hanging" idiom as
/// `throw_at_addresses.rs`'s `commit_hunks_budgeted`. Returns `(churn counts, commits whose diff
/// was captured, timed_out)`.
fn hunk_churn_budgeted(
    repo: &Path,
    multi_fn_files: &FileSpans,
    budget: std::time::Duration,
) -> (HashMap<(String, String), usize>, usize, bool) {
    use std::io::Read;
    use std::process::Stdio;
    use std::sync::mpsc;
    use std::thread;

    let mut churn: HashMap<(String, String), usize> = HashMap::new();
    let Ok(mut child) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--all",
            "--no-merges",
            "--unified=0",
            "--format=@@%H",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return (churn, 0, false);
    };
    let Some(mut stdout) = child.stdout.take() else {
        return (churn, 0, false);
    };
    let (tx, rx) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    let (bytes, timed_out) = rx.recv_timeout(budget).map_or_else(
        |_| {
            // Budget hit — kill the whole tree (see `kill_tree`), which closes the reader
            // thread's pipe and lets its blocked `read_to_end` unblock with whatever was
            // captured before the cutoff.
            kill_tree(&mut child);
            (
                rx.recv_timeout(std::time::Duration::from_secs(10))
                    .unwrap_or_default(),
                true,
            )
        },
        |buf| (buf, false),
    );
    let _ = reader.join();
    let _ = child.wait(); // reap regardless of the kill/finish path — no orphaned git.exe

    let text = String::from_utf8_lossy(&bytes);
    let mut cur_file: Option<String> = None;
    let mut commits_seen = 0usize;
    for line in text.lines() {
        if line.len() == 42 && line.starts_with("@@") && !line.starts_with("@@ ") {
            cur_file = None; // a commit header, not a hunk
            commits_seen += 1;
        } else if let Some(f) = line.strip_prefix("+++ b/") {
            cur_file = Some(f.replace('\\', "/"));
        } else if let Some(h) = line.strip_prefix("@@ ") {
            let Some(ref f) = cur_file else { continue };
            let Some(fns) = multi_fn_files.get(f) else {
                continue;
            };
            if let Some(plus) = h.split('+').nth(1) {
                let nums = plus.split('@').next().unwrap_or("").trim();
                let mut it = nums.split(',');
                if let Some(c) = it.next().and_then(|s| s.trim().parse::<i32>().ok()) {
                    let d = it
                        .next()
                        .and_then(|s| s.trim().parse::<i32>().ok())
                        .unwrap_or(1);
                    let (hs, he) = (c, c + d.max(1));
                    for (bare, fs, fe) in fns {
                        if *fs <= he && *fe >= hs {
                            *churn
                                .entry((bare.clone(), format!("{f}@{fs}-{fe}")))
                                .or_default() += 1;
                        }
                    }
                }
            }
        }
    }
    (churn, commits_seen, timed_out)
}

/// Firehose: per-function git churn -> records at each function's Bare + Span address (deepens the
/// temporal axis ON the function-addresses, fusing with the SCIP hoses).
///
/// TWO-PASS (ported from `throw_at_addresses.rs`'s `commit_files_all` / `commit_hunks_budgeted`
/// split, after the live-traced finding there — see `CHURN_HUNK_BUDGET`'s docs): a naive single
/// `git log --unified=0` over full history is network-fetch-bound on a partial clone, not
/// CPU/disk-bound, and isn't bounded by corpus size, so a fixed `timeout` wrapper only caps
/// damage rather than giving an honest result.
///
/// - CHEAP pass (`commit_touch_counts`): for a file holding EXACTLY ONE function, "this commit
///   touched the file" and "this commit touched the function" are the SAME fact — no hunk-level
///   disambiguation needed, so a `--name-only` tree-diff (zero blob fetches) gives a COMPLETE,
///   always-fast churn count for every single-fn file, immune to the budget below ever cutting
///   it off.
/// - BUDGETED pass (`hunk_churn_budgeted`): only for files holding 2+ functions, where
///   hunk-line-overlap is the only way to say which function a touch belongs to. Reports honest
///   partial coverage to stderr on a timeout — "hunks captured for N commits" — rather than
///   going silently incomplete.
///
/// Record shape UNCHANGED: `Bare` + `Span` `churn:{n}`, whichever pass produced the count.
fn hose_git_churn(repo: &Path, scip: &Path) -> Vec<Record> {
    let mut spans: FileSpans = HashMap::new();
    for (_sym, bare, path, s, e) in fn_defs(scip) {
        spans.entry(path).or_default().push((bare, s, e));
    }
    let (single_fn_files, multi_fn_files): (FileSpans, FileSpans) =
        spans.into_iter().partition(|(_, fns)| fns.len() == 1);

    let mut churn: HashMap<(String, String), usize> = HashMap::new();

    // -- CHEAP: single-fn-file churn, complete, unconditional --
    if !single_fn_files.is_empty() {
        let touch_counts = commit_touch_counts(repo);
        for (file, fns) in &single_fn_files {
            if let Some(&n) = touch_counts.get(file) {
                let (bare, s, e) = &fns[0];
                churn.insert((bare.clone(), format!("{file}@{s}-{e}")), n);
            }
        }
    }

    // -- BUDGETED: multi-fn-file churn, hunk-overlap disambiguation, time-boxed --
    if !multi_fn_files.is_empty() {
        let (hunk_churn, commits_seen, timed_out) =
            hunk_churn_budgeted(repo, &multi_fn_files, CHURN_HUNK_BUDGET);
        churn.extend(hunk_churn);
        if timed_out {
            eprintln!(
                "  [git-churn] hunk pass hit the {:.0}s budget after {commits_seen} commits — honest partial coverage for {} multi-fn files (single-fn-file churn above is complete, unaffected)",
                CHURN_HUNK_BUDGET.as_secs_f64(),
                multi_fn_files.len()
            );
        }
    }

    let mut recs = Vec::new();
    for ((bare, span), n) in churn {
        recs.push(Record {
            kind: "Bare",
            key: bare,
            source: "git-churn",
            content: format!("churn:{n}"),
        });
        recs.push(Record {
            kind: "Span",
            key: span,
            source: "git-churn",
            content: format!("churn:{n}"),
        });
    }
    recs
}

// ===== NEW ADDRESS KINDS (tier-1 signal hoses) — each holds a different coordinate fixed =====

/// Every `.rs` source under `repo` as `(relative_path, text)`, skipping `target/` and dotdirs.
/// Shared by the source-scanning hoses below (fail-site, attr, dep-use, impl-trait, type-sig).
fn rs_sources(repo: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![repo.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for ent in rd.flatten() {
            let p = ent.path();
            let name = ent.file_name();
            let name = name.to_string_lossy();
            if p.is_dir() {
                if name == "target" || name.starts_with('.') {
                    continue;
                }
                stack.push(p);
            } else if name.ends_with(".rs") {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    let rel = p
                        .strip_prefix(repo)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/");
                    out.push((rel, text));
                }
            }
        }
    }
    out
}

/// `path -> [(bare, start, end)]`, for mapping a source line to its enclosing function (fusing a
/// line-based signal onto the function manifold). Reuses the SCIP fn-def spans.
fn fn_span_index(scip: &Path) -> FileSpans {
    let mut idx: FileSpans = HashMap::new();
    for (_sym, bare, path, s, e) in fn_defs(scip) {
        idx.entry(path).or_default().push((bare, s, e));
    }
    idx
}

/// Bare name of the innermost fn whose span contains `line` (1-based), if any.
fn enclosing_fn(spans: &[(String, i32, i32)], line: i32) -> Option<&str> {
    spans
        .iter()
        .filter(|(_, s, e)| *s <= line && line <= *e)
        .min_by_key(|(_, s, e)| e - s)
        .map(|(b, ..)| b.as_str())
}

/// Drop `<...>` generic blocks (depth-aware) so the head identifier of a type-ish path survives.
fn strip_generics(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = (depth - 1).max(0),
            _ if depth == 0 => out.push(c),
            _ => {},
        }
    }
    out
}

/// The last identifier run in a (possibly-generic, possibly-pathed) type expression, e.g.
/// `core::fmt::Display` -> `Display`, `Foo<T>` -> `Foo`, `&mut [u8]` -> `u8`.
fn last_type_ident(s: &str) -> Option<String> {
    let stripped = strip_generics(s);
    let cleaned = stripped.trim().trim_end_matches('{').trim();
    cleaned
        .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
        .find(|p| !p.is_empty())
        .map(str::to_string)
}

/// Firehose: FAILURE-CLASS sites (unwrap/expect/panic/unsafe/todo/unimplemented/unreachable) ->
/// records at a `FailClass` address (the class BECOMES an address; its pool is EVERY site — antigen's
/// own antigens, as coordinates in the stroma), plus the File address and the enclosing fn's Bare
/// (fusing failure-density onto the file + function manifolds). Substring scan — a known instrument
/// hole (catches the words in prose/strings too); HELD, not judged.
fn hose_fail_site(repo: &Path, scip: &Path) -> Vec<Record> {
    const CLASSES: [&str; 7] = [
        ".unwrap(",
        ".expect(",
        "panic!",
        "unsafe",
        "todo!",
        "unimplemented!",
        "unreachable!",
    ];
    let idx = fn_span_index(scip);
    let empty: Vec<(String, i32, i32)> = Vec::new();
    let mut recs = Vec::new();
    for (rel, text) in rs_sources(repo) {
        let spans = idx.get(&rel).unwrap_or(&empty);
        for (i, line) in text.lines().enumerate() {
            let ln = i32::try_from(i + 1).unwrap_or(i32::MAX);
            for pat in CLASSES {
                if line.contains(pat) {
                    let class = pat.trim_start_matches('.').trim_end_matches('(');
                    recs.push(Record {
                        kind: "FailClass",
                        key: class.to_string(),
                        source: "fail-site",
                        content: format!("{rel}@{ln}"),
                    });
                    recs.push(Record {
                        kind: "File",
                        key: rel.clone(),
                        source: "fail-site",
                        content: format!("{class}@{ln}"),
                    });
                    if let Some(bare) = enclosing_fn(spans, ln) {
                        recs.push(Record {
                            kind: "Bare",
                            key: bare.to_string(),
                            source: "fail-site",
                            content: format!("{class}@{ln}"),
                        });
                    }
                }
            }
        }
    }
    recs
}

/// Firehose: ATTRIBUTE surface. `#[cfg(..)]` -> a platform/config cohort address (the pool at
/// `cfg(target_os = "windows")` is EVERY item gated to it); `#[derive(A, B)]` -> one `derive:A` per
/// trait; `#[deprecated]` -> an ATTESTED deprecation (witness-plural with the INFERRED dead-surface
/// signal); plus test/inline/etc. Records at an `Attr` address + the File address. Manual line parse.
fn hose_attr(repo: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    for (rel, text) in rs_sources(repo) {
        for (i, line) in text.lines().enumerate() {
            let ln = i32::try_from(i + 1).unwrap_or(i32::MAX);
            let t = line.trim_start();
            let Some(rest) = t.strip_prefix("#[").or_else(|| t.strip_prefix("#![")) else {
                continue;
            };
            let inner = rest.split_once(']').map_or(rest, |(a, _)| a);
            let head = inner.split('(').next().unwrap_or(inner).trim();
            if head.is_empty() || !head.chars().all(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let mut keys: Vec<String> = Vec::new();
            if head == "derive" {
                if let Some(open) = inner.find('(') {
                    let close = inner.rfind(')').unwrap_or(inner.len());
                    for tr in inner[open + 1..close].split(',') {
                        let tr = tr.trim();
                        if !tr.is_empty() {
                            keys.push(format!("derive:{tr}"));
                        }
                    }
                }
            } else if head == "cfg" || head == "cfg_attr" {
                keys.push(inner.trim().to_string()); // keep cfg(...) whole = the cohort key
            } else {
                keys.push(head.to_string());
            }
            for key in keys {
                recs.push(Record {
                    kind: "Attr",
                    key: key.clone(),
                    source: "attr",
                    content: format!("{rel}@{ln}"),
                });
                recs.push(Record {
                    kind: "File",
                    key: rel.clone(),
                    source: "attr",
                    content: format!("{key}@{ln}"),
                });
            }
        }
    }
    recs
}

/// Classify a commit subject (conventional-commit prefix, else keyword) into an intent class.
fn classify_subject(subject: &str) -> &'static str {
    let s = subject.trim();
    let head = s
        .split([':', '(', ' '])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let by_prefix = match head.as_str() {
        "fix" | "bugfix" | "hotfix" => Some("fix"),
        "feat" | "feature" => Some("feat"),
        "refactor" => Some("refactor"),
        "revert" => Some("revert"),
        "docs" | "doc" => Some("docs"),
        "test" | "tests" => Some("test"),
        "perf" => Some("perf"),
        "chore" => Some("chore"),
        "style" => Some("style"),
        "build" | "ci" => Some("build"),
        _ => None,
    };
    if let Some(c) = by_prefix {
        return c;
    }
    let l = s.to_ascii_lowercase();
    if l.contains("revert") {
        "revert"
    } else if l.contains("fix") || l.contains("bug") {
        "fix"
    } else if l.contains("add") || l.contains("implement") {
        "feat"
    } else if l.contains("refactor") || l.contains("clean") {
        "refactor"
    } else {
        "other"
    }
}

/// Firehose: COMMIT INTENT. Classify each commit subject -> records at a `CommitIntent` address (the
/// pool at `fix` is the defect-touching-commit map — antigen's home turf; the revert-signal
/// generalized) AND at the `Sha` address (fusing intent onto the temporal manifold git-commits opened).
fn hose_commit_intent(repo: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    let Ok(out) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--all",
            "--no-merges",
            "--format=%H%x00%s",
        ])
        .output()
    else {
        return recs;
    };
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut it = line.splitn(2, '\0');
        if let (Some(sha), Some(subj)) = (it.next(), it.next()) {
            let class = classify_subject(subj);
            recs.push(Record {
                kind: "CommitIntent",
                key: class.to_string(),
                source: "commit-intent",
                content: sha.to_string(),
            });
            recs.push(Record {
                kind: "Sha",
                key: sha.to_string(),
                source: "commit-intent",
                content: format!("intent:{class}"),
            });
        }
    }
    recs
}

/// Firehose: DEPENDENCY use-graph. Each `use <root>::...` -> a `Dep` address keyed by the crate ROOT
/// (the pool at `serde` is every use-site — the dependency-fingerprint) + the File address. `crate` /
/// `self` / `super` roots are internal; anything else is an external-dependency coordinate.
fn hose_dep_use(repo: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    for (rel, text) in rs_sources(repo) {
        for (i, line) in text.lines().enumerate() {
            let ln = i32::try_from(i + 1).unwrap_or(i32::MAX);
            let t = line.trim_start();
            let t = t.strip_prefix("pub ").unwrap_or(t);
            let Some(rest) = t.strip_prefix("use ") else {
                continue;
            };
            let root = rest
                .trim_start_matches("::")
                .split([':', ' ', ';', '{', '<'])
                .next()
                .unwrap_or("")
                .trim();
            if root.is_empty() || !root.starts_with(|c: char| c.is_alphabetic() || c == '_') {
                continue;
            }
            recs.push(Record {
                kind: "Dep",
                key: root.to_string(),
                source: "dep-use",
                content: format!("{rel}@{ln}"),
            });
            recs.push(Record {
                kind: "File",
                key: rel.clone(),
                source: "dep-use",
                content: format!("dep:{root}@{ln}"),
            });
        }
    }
    recs
}

/// Firehose: TRAIT participation. `impl <Trait> for <Type>` -> a `Trait` address (the pool at
/// `Display` is every type participating in that protocol — the protocol-participation axis) + a
/// `Type` address for the implementing type. Manual parse; inherent `impl Type {` (no `for`) skipped.
fn hose_impl_trait(repo: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    for (rel, text) in rs_sources(repo) {
        for (i, line) in text.lines().enumerate() {
            let ln = i32::try_from(i + 1).unwrap_or(i32::MAX);
            let t = line.trim_start();
            let Some(rest) = t.strip_prefix("impl") else {
                continue;
            };
            if !rest.starts_with(' ') && !rest.starts_with('<') {
                continue;
            }
            let Some((traitp, typep)) = rest.split_once(" for ") else {
                continue;
            };
            let (Some(tr), Some(ty)) = (last_type_ident(traitp), last_type_ident(typep)) else {
                continue;
            };
            recs.push(Record {
                kind: "Trait",
                key: tr.clone(),
                source: "impl-trait",
                content: format!("for:{ty}@{rel}:{ln}"),
            });
            recs.push(Record {
                kind: "Type",
                key: ty,
                source: "impl-trait",
                content: format!("impl:{tr}@{rel}:{ln}"),
            });
        }
    }
    recs
}

/// Firehose: RETURN-TYPE flow. `fn f(..) -> T` -> a `Type` address keyed by the return type's head
/// identifier (the pool at `Result` is every fn that yields it) + the enclosing fn's Bare. Crude
/// manual parse (drops generics, keeps the head ident; misses multi-line signatures) — a known
/// instrument hole, HELD not judged. Fuses with impl-trait's Type addresses.
fn hose_type_sig(repo: &Path, scip: &Path) -> Vec<Record> {
    let idx = fn_span_index(scip);
    let empty: Vec<(String, i32, i32)> = Vec::new();
    let mut recs = Vec::new();
    for (rel, text) in rs_sources(repo) {
        let spans = idx.get(&rel).unwrap_or(&empty);
        for (i, line) in text.lines().enumerate() {
            let ln = i32::try_from(i + 1).unwrap_or(i32::MAX);
            if !line.contains("fn ") {
                continue;
            }
            let Some((_, after)) = line.split_once("->") else {
                continue;
            };
            let rt = after.split('{').next().unwrap_or(after);
            let rt = rt.split(" where").next().unwrap_or(rt);
            let Some(ty) = last_type_ident(rt) else {
                continue;
            };
            recs.push(Record {
                kind: "Type",
                key: ty.clone(),
                source: "type-sig",
                content: format!("ret@{rel}:{ln}"),
            });
            if let Some(bare) = enclosing_fn(spans, ln) {
                recs.push(Record {
                    kind: "Bare",
                    key: bare.to_string(),
                    source: "type-sig",
                    content: format!("ret:{ty}"),
                });
            }
        }
    }
    recs
}

/// Firehose: CODE VINTAGE / cohort. Each file's BIRTH commit (`--diff-filter=A`, earliest add) -> a
/// `Cohort` address keyed by birth YEAR (the pool at `2021` is every file of that vintage; an old era
/// falls out as a cohort with no path-name needed) + a `File` address stamped with its birth year.
/// `--name-only` tree-diff — zero blob fetches on a partial clone.
fn hose_vintage(repo: &Path) -> Vec<Record> {
    let mut recs = Vec::new();
    let Ok(out) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--all",
            "--no-merges",
            "--diff-filter=A",
            "--name-only",
            "--format=@@%at",
        ])
        .output()
    else {
        return recs;
    };
    // Log is newest-first; keep the MIN timestamp per file -> the earliest (birth) add.
    let mut birth: HashMap<String, i64> = HashMap::new();
    let mut cur_ts: i64 = 0;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some(ts) = line.strip_prefix("@@") {
            cur_ts = ts.trim().parse::<i64>().unwrap_or(cur_ts);
        } else if !line.is_empty() {
            let f = line.replace('\\', "/");
            birth
                .entry(f)
                .and_modify(|t| *t = (*t).min(cur_ts))
                .or_insert(cur_ts);
        }
    }
    for (file, ts) in birth {
        let year = 1970 + ts / 31_557_600; // 365.25d; coarse cohort bucket, no date lib
        recs.push(Record {
            kind: "Cohort",
            key: year.to_string(),
            source: "vintage",
            content: file.clone(),
        });
        recs.push(Record {
            kind: "File",
            key: file,
            source: "vintage",
            content: format!("born:{year}"),
        });
    }
    recs
}

#[allow(clippy::cast_precision_loss)] // counts fit in f64's mantissa; this is display-only.
fn summarize(store: &Path) {
    let m = fold(store);
    let total_records: usize = m.values().map(Vec::len).sum();
    let mut multi = 0usize;
    let mut multi_source = 0usize;
    let mut by_kind: HashMap<&str, usize> = HashMap::new();
    let mut by_source: HashMap<String, usize> = HashMap::new();
    let mut max_addr: Option<(usize, String)> = None;
    for ((kind, key), contents) in &m {
        *by_kind.entry(kind.as_str()).or_default() += 1;
        let srcs: HashSet<&str> = contents.iter().map(|(s, _)| s.as_str()).collect();
        for s in &srcs {
            *by_source.entry((*s).to_string()).or_default() += 1;
        }
        if contents.len() > 1 {
            multi += 1;
        }
        if srcs.len() >= 2 {
            multi_source += 1;
        }
        if max_addr.as_ref().is_none_or(|(n, _)| contents.len() > *n) {
            max_addr = Some((contents.len(), format!("{kind}:{key}")));
        }
    }
    let pct = |n: usize| 100.0 * n as f64 / m.len().max(1) as f64;
    println!("== substrate fold: {} ==\n", store.display());
    println!("  distinct addresses:      {}", m.len());
    println!("  content records (total): {total_records}");
    println!(
        "  addresses w/ >1 record (multi-content):        {multi}  ({:.1}%)",
        pct(multi)
    );
    println!(
        "  addresses w/ content from >=2 HOSES (FUSION):  {multi_source}  ({:.1}%)  <- happened ON DISK, across separate runs",
        pct(multi_source)
    );
    let mut kinds: Vec<_> = by_kind.into_iter().collect();
    kinds.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    println!("  by address-granularity:  {kinds:?}");
    let mut srcs: Vec<_> = by_source.into_iter().collect();
    srcs.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    println!("  hoses present (addrs each): {srcs:?}");
    if let Some((n, a)) = max_addr {
        println!("  deepest pool:            {n} records at {a}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: substrate <store> write <repo> <scip> <hose>  |  substrate <store> fold\n  hoses: scip-defs, scip-refs, git-commits, clone-shape, markers, git-churn,\n         fail-site, attr, commit-intent, dep-use, impl-trait, type-sig, vintage";
    let Some(store) = args.first() else {
        eprintln!("{usage}");
        std::process::exit(2);
    };
    let store = Path::new(store);
    match args.get(1).map(String::as_str) {
        Some("write") => {
            let (Some(repo), Some(scip), Some(hose)) = (args.get(2), args.get(3), args.get(4))
            else {
                eprintln!("{usage}");
                std::process::exit(2);
            };
            let (repo, scip) = (Path::new(repo), Path::new(scip));
            let recs = match hose.as_str() {
                "scip-defs" => hose_scip_defs(scip),
                "scip-refs" => hose_scip_refs(scip),
                "git-commits" => hose_git_commits(repo),
                "clone-shape" => hose_clone_shape(repo, scip),
                "markers" => hose_markers(repo),
                "git-churn" => hose_git_churn(repo, scip),
                "fail-site" => hose_fail_site(repo, scip),
                "attr" => hose_attr(repo),
                "commit-intent" => hose_commit_intent(repo),
                "dep-use" => hose_dep_use(repo),
                "impl-trait" => hose_impl_trait(repo),
                "type-sig" => hose_type_sig(repo, scip),
                "vintage" => hose_vintage(repo),
                other => {
                    eprintln!(
                        "unknown hose '{other}' — try scip-defs | scip-refs | git-commits | clone-shape | markers | git-churn | fail-site | attr | commit-intent | dep-use | impl-trait | type-sig | vintage"
                    );
                    std::process::exit(2);
                },
            };
            if recs.is_empty() {
                eprintln!(
                    "  WARNING: hose '{hose}' produced 0 records — rule out the harness (empty/bad scip, wrong repo) before trusting the zero."
                );
            }
            match append(store, &recs) {
                Ok(n) => println!(
                    "  fired hose '{hose}' -> appended {n} records to {} (it STAYS; fire another hose to accumulate).",
                    store.display()
                ),
                Err(e) => {
                    eprintln!("  append failed: {e}");
                    std::process::exit(1);
                },
            }
        },
        Some("fold") => summarize(store),
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        },
    }
}
