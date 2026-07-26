//! `manifold` — the DENSIFY-don't-fuse substrate (the blind-opaque-keyed rebuild).
//!
//! The reframe (superseding `substrate.rs`'s `kind`-enum + fusion%):
//! - **stromix is blind.** A Cell is a container keyed by an OPAQUE address; the engine buckets
//!   droplets by key-equality and never interprets the key. It handles any project because it has no
//!   address semantics.
//! - **Each source is a BLIND `derive`** — a pure fn `data -> [Droplet]` that never reads the
//!   manifold and never targets a correspondent. Blindness = no read-after-write between sources =
//!   parallel, order-independent, deterministically replayable.
//! - **antigen decides what an Address MEANS** (the `derive_*` fns here). Correspondence is EMERGENT:
//!   two sources co-locate iff our derivations canonicalize to the same opaque key. We densify; we
//!   never fuse/reduce.
//! - **Nodes AND edges are both cells** — edge-ness is just a convention in the opaque key
//!   (`node|...` vs `edge|...`), read only by the reporting VIEW, never by the engine.
//! - **Time lives INSIDE the cell** (the droplet facet `t`), not as a substrate layer. The store only
//!   GROWS; appearance/disappearance is a VIEW that filters droplets by `t` (timeline = re-fold).
//!
//! Usage:
//!   cargo run -p antigen-stroma --example manifold -- <store> derive <repo> <source>  # git | tree
//!   cargo run -p antigen-stroma --example manifold -- <store> fold                     # density view
//!   cargo run -p antigen-stroma --example manifold -- <store> timeline                 # watch it breathe

use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::Command;

use antigen_stroma::syn_ingest::bare_fn_name;
use protobuf::Message;
use scip::types::{Index, SymbolRole};

/// A sprayed observation. `addr` is OPAQUE to the engine; `t` is event-time (epoch secs) — a facet
/// living INSIDE the cell. antigen's `derive_*` fns decide what `addr` means.
struct Droplet {
    addr: String,
    source: &'static str,
    content: String,
    t: i64,
}

fn san(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

/// APPEND droplets to the store (append-only; the log only ever grows — deletion is a VIEW).
fn append(store: &Path, drops: &[Droplet]) -> std::io::Result<usize> {
    let f = OpenOptions::new().create(true).append(true).open(store)?;
    let mut w = BufWriter::new(f);
    for d in drops {
        // 6-field record (A6 shape): addr, source, content, t, tier, derived_from(';'-joined).
        // Raw/blind sources are tier-0 with no derived_from; organs write their own tier-N records.
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t0\t",
            san(&d.addr),
            d.source,
            san(&d.content),
            d.t
        )?;
    }
    w.flush()?;
    Ok(drops.len())
}

/// Every droplet with event-time `<= t_max` (use `i64::MAX` for all). The temporal filter that turns
/// the ever-growing log into a point-in-time view.
fn read_at(store: &Path, t_max: i64) -> Vec<(String, String, String, i64)> {
    let mut out = Vec::new();
    let Ok(f) = std::fs::File::open(store) else {
        return out;
    };
    for line in BufReader::new(f).lines().map_while(Result::ok) {
        // Take the first 4 fields; tier + derived_from (fields 5-6, A6) are read only by `cone`.
        let mut it = line.split('\t');
        if let (Some(a), Some(s), Some(c), Some(t)) = (it.next(), it.next(), it.next(), it.next()) {
            let t: i64 = t.parse().unwrap_or(0);
            if t <= t_max {
                out.push((a.to_string(), s.to_string(), c.to_string(), t));
            }
        }
    }
    out
}

/// Fold droplets (`<= t_max`) into cells keyed by the OPAQUE address. No fusion metric — the cell is
/// just the bag of everything sprayed there, from any source, at any time.
fn fold_at(store: &Path, t_max: i64) -> HashMap<String, Vec<(String, String, i64)>> {
    let mut m: HashMap<String, Vec<(String, String, i64)>> = HashMap::new();
    for (a, s, c, t) in read_at(store, t_max) {
        m.entry(a).or_default().push((s, c, t));
    }
    m
}

/// Classify a commit subject into an intent (conventional-commit prefix, else keyword). antigen's
/// semantics, applied blindly per commit.
fn classify(subject: &str) -> &'static str {
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

/// Commits touching more than this many files skip co-change edges (the quadratic guard); reported.
const COCHANGE_CAP: usize = 30;

/// Emit one commit's droplets, all stamped with the commit's event-time: per touched file a
/// `node|file` touch (the churn STREAM, not a count) and an `edge|touched` author→file; one
/// `node|author`; and `edge|cochange` between co-touched files. Then clear `files` for the next commit.
fn flush_commit(
    files: &mut Vec<String>,
    t: i64,
    author: &str,
    intent: &str,
    drops: &mut Vec<Droplet>,
    skipped: &mut usize,
) {
    for f in files.iter() {
        drops.push(Droplet {
            addr: format!("node|file|{f}"),
            source: "git",
            content: format!("touched:{intent}"),
            t,
        });
        drops.push(Droplet {
            addr: format!("edge|touched|{author}|{f}"),
            source: "git",
            content: intent.to_string(),
            t,
        });
    }
    if !author.is_empty() {
        drops.push(Droplet {
            addr: format!("node|author|{author}"),
            source: "git",
            content: "commit".into(),
            t,
        });
    }
    if files.len() <= COCHANGE_CAP {
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let (a, b) = if files[i] <= files[j] {
                    (&files[i], &files[j])
                } else {
                    (&files[j], &files[i])
                };
                drops.push(Droplet {
                    addr: format!("edge|cochange|{a}|{b}"),
                    source: "git",
                    content: "cochange".into(),
                    t,
                });
            }
        }
    } else {
        *skipped += 1;
    }
    files.clear();
}

/// BLIND source: git history. One `--name-status` pass (zero blob fetch on a partial clone). A `D`
/// status sprays a `deleted` droplet on the file NODE (which STAYS — hold-all; deletion is a facet,
/// not a removal); a rename/copy deletes the old path and touches the new. Non-deleted paths accrue
/// touch + co-change edges. Returns droplets + the count of large commits whose co-change was skipped.
fn derive_git(repo: &Path) -> (Vec<Droplet>, usize) {
    let mut drops = Vec::new();
    let mut skipped = 0usize;
    let Ok(out) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--all",
            "--no-merges",
            "--name-status",
            "--format=@@%H%x00%at%x00%ae%x00%s",
        ])
        .output()
    else {
        return (drops, skipped);
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut t: i64 = 0;
    let mut author = String::new();
    let mut intent = "other";
    let mut files: Vec<String> = Vec::new(); // non-deleted (touched) paths this commit
    let mark_deleted = |path: &str, t: i64, drops: &mut Vec<Droplet>| {
        drops.push(Droplet {
            addr: format!("node|file|{}", path.replace('\\', "/")),
            source: "git",
            content: "deleted".into(),
            t,
        });
    };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("@@") {
            flush_commit(&mut files, t, &author, intent, &mut drops, &mut skipped);
            let mut p = rest.splitn(4, '\0');
            let _sha = p.next();
            t = p.next().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
            author = p.next().unwrap_or("").to_string();
            intent = classify(p.next().unwrap_or(""));
        } else if !line.is_empty() {
            let mut parts = line.split('\t');
            let sc = parts.next().unwrap_or("").chars().next().unwrap_or('M');
            let rest: Vec<&str> = parts.collect();
            if rest.is_empty() {
                continue;
            }
            match sc {
                'D' => mark_deleted(rest[0], t, &mut drops),
                'R' | 'C' if rest.len() >= 2 => {
                    mark_deleted(rest[0], t, &mut drops); // old path dies
                    files.push(rest[1].replace('\\', "/")); // new path touched
                },
                _ => files.push(rest[rest.len() - 1].replace('\\', "/")),
            }
        }
    }
    flush_commit(&mut files, t, &author, intent, &mut drops, &mut skipped);
    (drops, skipped)
}

/// BLIND source: the current tree. Sprays a `node|file` (exists) plus, per failure-class site, a
/// `node|failclass` and an `edge|has-failsite` file→class — all stamped at HEAD-time. Its `node|file`
/// addresses canonicalize to the SAME form `derive_git` uses, so the file cells DENSIFY (git's touch
/// stream + the tree's current structure) with no fusion forced — correspondence falls out.
fn derive_tree(repo: &Path, head_t: i64) -> Vec<Droplet> {
    const FAIL: [&str; 7] = [
        ".unwrap(",
        ".expect(",
        "panic!",
        "unsafe",
        "todo!",
        "unimplemented!",
        "unreachable!",
    ];
    let mut drops = Vec::new();
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
                drops.push(Droplet {
                    addr: format!("node|file|{rel}"),
                    source: "tree",
                    content: "exists".into(),
                    t: head_t,
                });
                for (i, line) in text.lines().enumerate() {
                    let ln = i + 1;
                    for pat in FAIL {
                        if line.contains(pat) {
                            let class = pat.trim_start_matches('.').trim_end_matches('(');
                            drops.push(Droplet {
                                addr: format!("node|failclass|{class}"),
                                source: "tree",
                                content: format!("{rel}@{ln}"),
                                t: head_t,
                            });
                            drops.push(Droplet {
                                addr: format!("edge|has-failsite|{rel}|{class}"),
                                source: "tree",
                                content: format!("@{ln}"),
                                t: head_t,
                            });
                        }
                    }
                }
            }
        }
    }
    drops
}

fn read_index(scip: &Path) -> Option<Index> {
    let bytes = std::fs::read(scip).ok()?;
    Index::parse_from_bytes(&bytes).ok()
}

/// `(is_pub, is_test)` for a fn defined near line `start` (0/1-based tolerant) named `bare`: reads the
/// def line for a `pub`/`pub(` prefix and the few lines above for `#[test]`/`#[cfg(test)]`. Lets the
/// dead-code view exclude harness-called tests and externally-called `pub` API (the false-positives).
fn fn_flags(lines: &[String], start: usize, bare: &str) -> (bool, bool, bool) {
    let n = lines.len();
    if n == 0 {
        return (false, false, false);
    }
    let needle = format!("fn {bare}");
    let (lo, hi) = (start.saturating_sub(3), (start + 2).min(n));
    let di = (lo..hi)
        .find(|&i| lines[i].contains(&needle))
        .unwrap_or_else(|| start.min(n - 1));
    let is_pub = lines.get(di).is_some_and(|l| {
        let t = l.trim_start();
        t.starts_with("pub ") || t.starts_with("pub(")
    });
    let is_test = (di.saturating_sub(6)..di).any(|i| {
        let l = &lines[i];
        l.contains("#[test]") || l.contains("::test]") || l.contains("#[cfg(test")
    });
    // impl-marker: the nearest strictly-lower-indent opener above the fn is `impl <Trait> for <Type>`
    // → this is a trait-method impl (dispatch-reachable; SCIP's missing caller is EXPECTED, not death).
    let fn_indent = lines[di].len() - lines[di].trim_start().len();
    let mut is_trait_impl = false;
    for i in (di.saturating_sub(80)..di).rev() {
        let t = lines[i].trim_start();
        if t.is_empty() {
            continue;
        }
        if lines[i].len() - t.len() < fn_indent {
            is_trait_impl = t.starts_with("impl") && t.contains(" for ");
            break;
        }
    }
    (is_pub, is_test, is_trait_impl)
}

/// BLIND source: SCIP — a CLOSED-WORLD static snapshot at HEAD. Per fn-def it sprays the symbol as a
/// `node|scip` (content `bare|vis`, vis ∈ test/pub/priv), its file as `node|file` (canonicalizing with
/// git/tree → correspondence), an `edge|contains` file→symbol, a `node|bare` and an `edge|same-as`
/// symbol↔bare-name (identity WITHOUT merge — a bare cell same-as'd to N symbols IS the collision, held
/// as structure). Per reference it sprays `edge|calls` caller→callee (caller = enclosing fn). A `priv`
/// symbol with zero inbound `calls` edges is a TRUE dead-code candidate. All stamped at HEAD-time.
#[allow(clippy::too_many_lines)] // two SCIP passes (defs+refs) read clearest inline
fn derive_scip(repo: &Path, scip: &Path, t: i64) -> Vec<Droplet> {
    let mut drops = Vec::new();
    let Some(index) = read_index(scip) else {
        return drops;
    };
    let mut file_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut spans: HashMap<String, Vec<(String, i32, i32)>> = HashMap::new();
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
            let sym = &occ.symbol;
            let lines = file_cache.entry(path.clone()).or_insert_with(|| {
                std::fs::read_to_string(repo.join(&path))
                    .map(|t| t.lines().map(String::from).collect())
                    .unwrap_or_default()
            });
            // Anchor on the IDENTIFIER line (exact `fn NAME`), not enclosing_range[0] — the latter
            // includes the doc/attr block, pushing the fn line out of fn_flags's window.
            let name_line = usize::try_from(*occ.range.first().unwrap_or(&r[0])).unwrap_or(0);
            let (is_pub, is_test, is_trait_impl) = fn_flags(lines, name_line, bare);
            let path_test = path.contains("/tests/")
                || path.contains("/benches/")
                || path.starts_with("tests/")
                || path.starts_with("benches/");
            let vis = if is_test || path_test {
                "test"
            } else if is_trait_impl {
                "impl"
            } else if is_pub {
                "pub"
            } else {
                "priv"
            };
            drops.push(Droplet {
                addr: format!("node|scip|{sym}"),
                source: "scip",
                content: format!("{bare}|{vis}"),
                t,
            });
            drops.push(Droplet {
                addr: format!("node|file|{path}"),
                source: "scip",
                content: "scip".into(),
                t,
            });
            drops.push(Droplet {
                addr: format!("edge|contains|{path}|{sym}"),
                source: "scip",
                content: "contains".into(),
                t,
            });
            drops.push(Droplet {
                addr: format!("node|bare|{bare}"),
                source: "scip",
                content: "name".into(),
                t,
            });
            drops.push(Droplet {
                addr: format!("edge|same-as|{sym}|{bare}"),
                source: "scip",
                content: "sym~bare".into(),
                t,
            });
            spans
                .entry(path.clone())
                .or_default()
                .push((sym.clone(), r[0], r[2]));
        }
    }
    for doc in &index.documents {
        let path = doc.relative_path.replace('\\', "/");
        let file_spans = spans.get(&path);
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) != 0 || occ.range.is_empty() {
                continue;
            }
            let line = occ.range[0];
            let caller = file_spans
                .and_then(|v| {
                    v.iter()
                        .filter(|(_, s, e)| *s <= line && line <= *e)
                        .min_by_key(|(_, s, e)| e - s)
                })
                .map_or_else(|| format!("file:{path}"), |(sym, ..)| sym.clone());
            drops.push(Droplet {
                addr: format!("edge|calls|{caller}|{}", occ.symbol),
                source: "scip",
                content: "ref".into(),
                t,
            });
        }
    }
    drops
}

/// Dead-code / identity VIEW (SCIP closed-world at HEAD). UNCALLED = fn symbols with zero inbound
/// `calls` edges = dead-code candidates — with the honest caveat that "uncalled" means uncalled
/// WITHIN this repo (a `pub` API called only by downstream crates false-positives; the library-surface
/// caveat — and exactly why RUNTIME telemetry is the true closed world). COLLISIONS = a bare name
/// `same-as`'d to >=2 distinct symbols, the phantom-merge turned into visible structure.
#[allow(clippy::cast_precision_loss)] // display-only counts
fn dead(store: &Path) {
    let all = read_at(store, i64::MAX);
    // scip symbol -> (bare, vis) where vis ∈ test/pub/priv
    let mut fns: HashMap<String, (String, String)> = HashMap::new();
    for (addr, source, content, _t) in &all {
        if source == "scip" {
            if let Some(sym) = addr.strip_prefix("node|scip|") {
                let (bare, vis) = content.split_once('|').unwrap_or((content.as_str(), "?"));
                fns.insert(sym.to_string(), (bare.to_string(), vis.to_string()));
            }
        }
    }
    let mut inbound: HashMap<String, usize> = HashMap::new();
    let mut bare_syms: HashMap<String, HashSet<String>> = HashMap::new();
    for (addr, _s, _c, _t) in &all {
        if let Some(rest) = addr.strip_prefix("edge|calls|") {
            if let Some((_caller, callee)) = rest.split_once('|') {
                *inbound.entry(callee.to_string()).or_default() += 1;
            }
        } else if let Some(rest) = addr.strip_prefix("edge|same-as|") {
            if let Some((sym, bare)) = rest.rsplit_once('|') {
                bare_syms
                    .entry(bare.to_string())
                    .or_default()
                    .insert(sym.to_string());
            }
        }
    }
    let (mut n_test, mut n_pub, mut n_impl, mut uncalled_total) = (0usize, 0usize, 0usize, 0usize);
    let mut strong_dead: Vec<&str> = Vec::new(); // priv & uncalled = unmarked, suspected-DEAD
    for (sym, (bare, vis)) in &fns {
        if inbound.get(sym.as_str()).copied().unwrap_or(0) != 0 {
            continue;
        }
        uncalled_total += 1;
        match vis.as_str() {
            "test" => n_test += 1,
            "pub" => n_pub += 1,
            "impl" => n_impl += 1,
            _ => strong_dead.push(bare.as_str()),
        }
    }
    strong_dead.sort_unstable();
    let mut collisions: Vec<(&String, usize)> = bare_syms
        .iter()
        .filter(|(_, syms)| syms.len() >= 2)
        .map(|(b, syms)| (b, syms.len()))
        .collect();
    collisions.sort_unstable_by_key(|(_, n)| std::cmp::Reverse(*n));

    let fns_n = fns.len().max(1) as f64;
    let upct = 100.0 * uncalled_total as f64 / fns_n;
    let spct = 100.0 * strong_dead.len() as f64 / fns_n;
    println!(
        "== dead-code / identity view (SCIP closed-world at HEAD): {} ==\n",
        store.display()
    );
    println!("  scip fn symbols: {}", fns.len());
    println!(
        "  UNCALLED in-repo: {uncalled_total} ({upct:.1}%) = {n_test} test + {n_pub} pub + {n_impl} trait-impl (dynamic-dispatch) + {} priv",
        strong_dead.len()
    );
    println!(
        "  WEAK DEAD  (trait-impl & uncalled — suspected-DYNAMIC; expect runtime to light it up): {n_impl}"
    );
    println!(
        "  STRONG DEAD (priv & uncalled — unmarked, suspected-DEAD, the real find): {} ({spct:.1}%)",
        strong_dead.len()
    );
    for b in strong_dead.iter().take(20) {
        println!("      {b}");
    }
    println!(
        "\n  bare-name COLLISIONS (a bare cell same-as'd to >=2 distinct symbols = the phantom-merge as VISIBLE structure): {}",
        collisions.len()
    );
    for (b, n) in collisions.iter().take(12) {
        println!("      {b}  -> {n} distinct symbols");
    }
}

/// HEAD commit's author-timestamp (epoch secs), so tree droplets land at "now" on the timeline.
fn head_time(repo: &Path) -> i64 {
    Command::new("git")
        .args(["-C", &repo.to_string_lossy(), "log", "-1", "--format=%at"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(0)
}

/// Density VIEW: cells, droplets, and EMERGENT correspondence (cells sprayed by >=2 sources — an
/// observation, never a target). Splits node|/edge| only for the human; the engine never did.
#[allow(clippy::cast_precision_loss)] // display-only counts
fn summarize(store: &Path) {
    let m = fold_at(store, i64::MAX);
    let (mut nodes, mut edges, mut drops, mut corr) = (0usize, 0usize, 0usize, 0usize);
    let mut by_source: HashMap<String, usize> = HashMap::new();
    let mut deepest: Option<(usize, String)> = None;
    for (addr, ds) in &m {
        drops += ds.len();
        if addr.starts_with("edge|") {
            edges += 1;
        } else {
            nodes += 1;
        }
        let srcs: HashSet<&str> = ds.iter().map(|(s, ..)| s.as_str()).collect();
        if srcs.len() >= 2 {
            corr += 1;
        }
        for s in &srcs {
            *by_source.entry((*s).to_string()).or_default() += 1;
        }
        if deepest.as_ref().is_none_or(|(n, _)| ds.len() > *n) {
            deepest = Some((ds.len(), addr.clone()));
        }
    }
    println!("== manifold density: {} ==\n", store.display());
    println!("  cells:      {}  ({nodes} node | {edges} edge)", m.len());
    println!("  droplets:   {drops}");
    let corr_pct = 100.0 * corr as f64 / m.len().max(1) as f64;
    println!(
        "  EMERGENT correspondence (cells w/ >=2 sources): {corr}  ({corr_pct:.1}%)  <- fell out, never forced"
    );
    let mut srcs: Vec<_> = by_source.into_iter().collect();
    srcs.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    println!("  sources (cells each): {srcs:?}");
    if let Some((n, a)) = deepest {
        println!("  deepest cell: {n} droplets at {a}");
    }
}

/// Timeline VIEW: re-fold at 8 slices across the event-time range and watch nodes/edges APPEAR. The
/// substrate never shrank — this is the growing shadow of an ever-growing log.
fn timeline(store: &Path) {
    let all = read_at(store, i64::MAX);
    let times: Vec<i64> = all
        .iter()
        .map(|(_, _, _, t)| *t)
        .filter(|t| *t > 0)
        .collect();
    let (Some(&tmin), Some(&tmax)) = (times.iter().min(), times.iter().max()) else {
        println!("no timestamped droplets yet");
        return;
    };
    println!(
        "== manifold timeline: {} droplets over epoch {tmin}..{tmax} ==\n",
        all.len()
    );
    for k in 0..=8i64 {
        let t = tmin + (tmax - tmin) * k / 8;
        let m = fold_at(store, t);
        let (mut nodes, mut edges) = (0usize, 0usize);
        for addr in m.keys() {
            if addr.starts_with("edge|") {
                edges += 1;
            } else {
                nodes += 1;
            }
        }
        let year = 1970 + t / 31_557_600; // coarse label, no date lib
        println!("  ~{year} (t<={t}): {nodes:7} nodes, {edges:8} edges present");
    }
}

/// QUIET/FROZEN VIEW (NOT orphan detection — a corrected label). The manifold is OPEN-WORLD: absence
/// of a recent droplet asserts NOTHING, so "no recent git activity" means a file STABILIZED, not that
/// it lost anything — all its edges still stand and are still true. This view reports that stability
/// signal honestly. TRUE orphan/dead detection needs POSITIVE removal evidence — an explicit deletion
/// (git `D`, counted here) or a CLOSED-WORLD re-derivation delta of STATE edges (e.g. SCIP `calls`
/// present-then-absent), NOT event-edge silence. See the deleted count for the one real removal signal.
#[allow(clippy::cast_precision_loss)] // display-only counts
fn quiet(store: &Path) {
    let all = read_at(store, i64::MAX);
    let times: Vec<i64> = all
        .iter()
        .map(|(_, _, _, t)| *t)
        .filter(|t| *t > 0)
        .collect();
    let (Some(&tmin), Some(&tmax)) = (times.iter().min(), times.iter().max()) else {
        println!("no timestamped droplets yet");
        return;
    };
    let recent_start = tmax - ((tmax - tmin) / 5).max(1); // recency = last 1/5 of history

    // File nodes: birth + whether the latest status droplet is a delete. Nodes only ever accumulate.
    let mut last: HashMap<String, (i64, String)> = HashMap::new();
    for (addr, _s, content, t) in &all {
        if let Some(p) = addr.strip_prefix("node|file|") {
            let e = last
                .entry(p.to_string())
                .or_insert_with(|| (*t, content.clone()));
            if *t >= e.0 {
                *e = (*t, content.clone());
            }
        }
    }
    let files: HashSet<&str> = last.keys().map(String::as_str).collect();

    // git-activity edges only (co-change/touched); tree existence-edges excluded on purpose.
    let (mut recent, mut past): (HashSet<&str>, HashSet<&str>) = (HashSet::new(), HashSet::new());
    for (addr, source, _c, t) in &all {
        if source == "git" && addr.starts_with("edge|") {
            for part in addr.split('|').skip(2) {
                if let Some(f) = files.get(part) {
                    if *t >= recent_start {
                        recent.insert(f);
                    } else {
                        past.insert(f);
                    }
                }
            }
        }
    }

    let (mut alive, mut deleted, mut quiet_ct) = (0usize, 0usize, 0usize);
    let mut sample: Vec<&str> = Vec::new();
    for (p, (_, content)) in &last {
        if content == "deleted" {
            deleted += 1;
            continue;
        }
        alive += 1;
        if past.contains(p.as_str()) && !recent.contains(p.as_str()) {
            quiet_ct += 1;
            if sample.len() < 15 {
                sample.push(p.as_str());
            }
        }
    }
    let qpct = 100.0 * quiet_ct as f64 / alive.max(1) as f64;
    println!(
        "== quiet/frozen view (open-world: silence != removal): {} ==\n",
        store.display()
    );
    println!("  file nodes ever seen:        {}", last.len());
    println!(
        "  DELETED (git D — the ONE real removal signal; node PERSISTS as history): {deleted}"
    );
    println!("  alive:                       {alive}");
    println!("  recency window: t >= {recent_start} (last 1/5 of git history)");
    println!(
        "  QUIET/FROZEN (alive, no git activity in the window) = STABILIZED, edges still STAND — NOT orphaned: {quiet_ct}  ({qpct:.1}% of alive)"
    );
    for s in &sample {
        println!("      {s}");
    }
    println!(
        "\n  (true orphan = LOST state-edges — needs a closed-world re-derivation delta, e.g. SCIP calls; not computed here)"
    );
}

/// The first antigen ORGAN — recognize + write-back, closing the tier-loop. Reads each file-cell's
/// stacked risk signals from the densified pile (git churn, bug-FIX history via `touched:fix`,
/// failure-sites via `has-failsite` edges), RECOGNIZES where >=2 stack as a defect-prone nexus, and
/// WRITES the judgment BACK as `organ:nexus` droplets (a new tier-1 source — re-fold to see the loop).
/// It is a presents(risk?) QUESTION — surface-all, route-to-human, NOT a verdict; it ships the PARTIAL
/// signals that stacked (per ratified ADR-002), never a scalar. detect(stromix) read → recognize(antigen) written.
#[allow(clippy::cast_precision_loss)] // display-only counts
fn nexus(store: &Path) {
    const CHURN_CUT: usize = 10;
    const FIX_CUT: usize = 2;
    const FAIL_CUT: usize = 1;
    let all = read_at(store, i64::MAX);
    let now_t = all.iter().map(|(_, _, _, t)| *t).max().unwrap_or(0);
    let mut touch: HashMap<String, usize> = HashMap::new();
    let mut fix: HashMap<String, usize> = HashMap::new();
    let mut fail: HashMap<String, usize> = HashMap::new();
    let mut fail_edges: HashMap<String, Vec<String>> = HashMap::new(); // file -> the has-failsite edge cells (provenance)
    let mut deleted: HashSet<String> = HashSet::new();
    for (addr, source, content, _t) in &all {
        if let Some(f) = addr.strip_prefix("node|file|") {
            if source == "git" {
                if content == "deleted" {
                    deleted.insert(f.to_string());
                } else if content.starts_with("touched") {
                    *touch.entry(f.to_string()).or_default() += 1;
                    if content == "touched:fix" {
                        *fix.entry(f.to_string()).or_default() += 1;
                    }
                }
            }
        } else if let Some(rest) = addr.strip_prefix("edge|has-failsite|") {
            if let Some((f, _)) = rest.split_once('|') {
                *fail.entry(f.to_string()).or_default() += 1;
                let v = fail_edges.entry(f.to_string()).or_default();
                if !v.contains(addr) {
                    v.push(addr.clone()); // distinct failure-class edge-cells only
                }
            }
        }
    }
    // RECOGNIZE: a file where >=2 of {churn, bug-fix history, failure-sites} STACK = a defect-prone nexus.
    let mut nexi: Vec<(String, usize, usize, usize)> = Vec::new();
    let files: HashSet<&String> = touch.keys().chain(fail.keys()).collect();
    for f in files {
        if deleted.contains(f) {
            continue;
        }
        let (tc, fx, fl) = (
            touch.get(f).copied().unwrap_or(0),
            fix.get(f).copied().unwrap_or(0),
            fail.get(f).copied().unwrap_or(0),
        );
        let signals =
            usize::from(tc >= CHURN_CUT) + usize::from(fx >= FIX_CUT) + usize::from(fl >= FAIL_CUT);
        if signals >= 2 {
            nexi.push((f.clone(), tc, fx, fl));
        }
    }
    nexi.sort_by_key(|(_, tc, fx, fl)| std::cmp::Reverse((*fl, *fx, *tc)));

    // WRITE BACK: 6-field tier-1 records (A6 shape) carrying `derived_from` — the cone lineage down to
    // raw. The organ read tier-0 and writes tier-1; its output records WHICH cells it rests on
    // (the file's git/tree cell + its has-failsite edges), so `cone` can walk fat-end → raw.
    let n = nexi.len();
    if let Ok(fh) = OpenOptions::new().create(true).append(true).open(store) {
        let mut w = BufWriter::new(fh);
        for (f, tc, fx, fl) in &nexi {
            let mut sigs = Vec::new();
            if *tc >= CHURN_CUT {
                sigs.push(format!("churn:{tc}"));
            }
            if *fx >= FIX_CUT {
                sigs.push(format!("fix:{fx}"));
            }
            if *fl >= FAIL_CUT {
                sigs.push(format!("fail:{fl}"));
            }
            let mut df = vec![format!("node|file|{f}")]; // git churn/fix + tree existence live here
            if let Some(edges) = fail_edges.get(f) {
                df.extend(edges.iter().cloned()); // the failure-site edges it rests on
            }
            let _ = writeln!(
                w,
                "{}\torgan:nexus\t{}\t{}\t1\t{}",
                san(&format!("node|file|{f}")),
                san(&format!("presents:risk[{}]", sigs.join(","))),
                now_t,
                df.join(";")
            );
        }
        let _ = w.flush();
    }

    println!(
        "== organ:nexus — presents(risk?), a QUESTION not a verdict (route-to-human): {} ==\n",
        store.display()
    );
    println!(
        "  recognition: a file where >=2 of {{ churn>={CHURN_CUT}, fix-history>={FIX_CUT}, failure-sites>={FAIL_CUT} }} STACK"
    );
    println!("  defect-prone nexi surfaced (partial signals, no scalar): {n}\n");
    for (f, tc, fx, fl) in nexi.iter().take(20) {
        println!("      churn:{tc:<4} fix:{fx:<3} fail:{fl:<3}  {f}");
    }
    println!(
        "\n  wrote {n} `organ:nexus` droplets BACK into the manifold (tier-1) — re-fold to see the loop: the organ's recognition is now a source."
    );
}

/// A full 6-field record: `(source, content, t, tier, derived_from)`.
type FullRec = (String, String, i64, u32, Vec<String>);

/// Read the full A6-shaped records (incl. `tier` + `derived_from`), grouped by cell address.
fn read_full(store: &Path) -> HashMap<String, Vec<FullRec>> {
    let mut m: HashMap<String, Vec<FullRec>> = HashMap::new();
    let Ok(f) = std::fs::File::open(store) else {
        return m;
    };
    for line in BufReader::new(f).lines().map_while(Result::ok) {
        let p: Vec<&str> = line.split('\t').collect();
        if p.len() < 4 {
            continue;
        }
        let tier = p.get(4).and_then(|s| s.parse().ok()).unwrap_or(0u32);
        let df: Vec<String> = p.get(5).map_or_else(Vec::new, |s| {
            if s.is_empty() {
                Vec::new()
            } else {
                s.split(';').map(String::from).collect()
            }
        });
        m.entry(p[0].to_string()).or_default().push((
            p[1].to_string(),
            p[2].to_string(),
            p[3].parse().unwrap_or(0),
            tier,
            df,
        ));
    }
    m
}

/// Cone VIEW: walk `derived_from` from a fat-end cell down to raw — A3 provenance-tiered, the
/// substrate's own grounding chain (stromrez `chain` generalized to the whole store). Shows how a
/// tier-N recognition rests, all the way down, on tier-0 data.
fn cone(store: &Path, target: &str) {
    let full = read_full(store);
    if !full.contains_key(target) {
        println!("no cell '{target}' (try a `node|file|...` that organ:nexus flagged)");
        return;
    }
    println!("== cone: {target}  (fat end -> raw) ==\n");
    let mut seen = HashSet::new();
    walk_cone(&full, target, 0, &mut seen);
}

fn walk_cone(
    full: &HashMap<String, Vec<FullRec>>,
    addr: &str,
    depth: usize,
    seen: &mut HashSet<String>,
) {
    let indent = "  ".repeat(depth);
    if !seen.insert(addr.to_string()) {
        return; // already shown (e.g. an organ resting on its own cell)
    }
    let Some(recs) = full.get(addr) else {
        println!("{indent}[absent] {addr}");
        return;
    };
    let maxtier = recs.iter().map(|r| r.3).max().unwrap_or(0);
    let mut srcs: Vec<&str> = recs.iter().map(|r| r.0.as_str()).collect();
    srcs.sort_unstable();
    srcs.dedup();
    let rep = recs.iter().max_by_key(|r| r.3).map_or("", |r| r.1.as_str());
    println!(
        "{indent}[t{maxtier}] {addr}  <{}>  {rep}  ({} rec)",
        srcs.join(","),
        recs.len()
    );
    let mut df_all: Vec<String> = Vec::new();
    for r in recs {
        for d in &r.4 {
            if d != addr && !df_all.contains(d) {
                df_all.push(d.clone());
            }
        }
    }
    if depth < 8 {
        for d in df_all {
            walk_cone(full, &d, depth + 1, seen);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: manifold <store> derive <repo> <source> [scip]  |  manifold <store> {fold|timeline|quiet|dead|nexus}  |  manifold <store> cone <cell>\n  sources: git, tree, scip <scip-path>";
    let Some(store) = args.first() else {
        eprintln!("{usage}");
        std::process::exit(2);
    };
    let store = Path::new(store);
    match args.get(1).map(String::as_str) {
        Some("derive") => {
            let (Some(repo), Some(src)) = (args.get(2), args.get(3)) else {
                eprintln!("{usage}");
                std::process::exit(2);
            };
            let repo = Path::new(repo);
            let (drops, note) = match src.as_str() {
                "git" => {
                    let (d, sk) = derive_git(repo);
                    (
                        d,
                        format!("(co-change skipped for {sk} >{COCHANGE_CAP}-file commits)"),
                    )
                },
                "tree" => (derive_tree(repo, head_time(repo)), String::new()),
                "scip" => {
                    let Some(scip) = args.get(4) else {
                        eprintln!("scip source needs a 4th arg: the .scip index path");
                        std::process::exit(2);
                    };
                    (
                        derive_scip(repo, Path::new(scip), head_time(repo)),
                        String::new(),
                    )
                },
                other => {
                    eprintln!("unknown source '{other}' — try git | tree | scip <scip-path>");
                    std::process::exit(2);
                },
            };
            match append(store, &drops) {
                Ok(n) => println!(
                    "  sprayed {n} droplets from '{src}' into {} {note} (it STAYS; add sources to densify).",
                    store.display()
                ),
                Err(e) => {
                    eprintln!("  append failed: {e}");
                    std::process::exit(1);
                },
            }
        },
        Some("fold") => summarize(store),
        Some("timeline") => timeline(store),
        Some("quiet") => quiet(store),
        Some("dead") => dead(store),
        Some("nexus") => nexus(store),
        Some("cone") => {
            let Some(target) = args.get(2) else {
                eprintln!(
                    "cone needs a <cell> address, e.g. `node|file|antigen-macros/src/parse.rs`"
                );
                std::process::exit(2);
            };
            cone(store, target);
        },
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        },
    }
}
