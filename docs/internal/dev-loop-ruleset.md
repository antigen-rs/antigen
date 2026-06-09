# Antigen Dev-Loop Ruleset — target + staged adoption

Distilled from the June-2026 opinionated research pass. This is the **target** set of
development-time checks (lints / rustfmt / tooling), plus the constraint that governs how
we adopt it. Concern tags: `[AGENT]` agentic-determinism · `[CONATIVE]` co-native
readability · `[REUSE]` anti-redevelopment · `[PUBLIC]` external-contributor hygiene.

## The adoption constraint — read first

The antigen gate (on-save hook + CI) runs clippy/check/doc with **`-D warnings`**, which
escalates every `warn`-level lint in `[lints]` to a hard error *at the gate*. So **adding a
strict lint that FIRES turns the gate red.** Adoption is therefore **per-lint**:

> add the lint → run the relevant gate (`cargo clippy`/`cargo doc`) → fix the code or
> `#[expect(clippy::x, reason = "…")]` the justified cases → commit.

This is a real cleanup workstream, not a config-drop. (`rtk` masks `-D warnings` — use plain
`cargo`/`command cargo` for these gates, per project memory.)

---

## Tier 1 — apply now (zero new fires; landed)

- **`.cargo/config.toml`** → `[build] rustdocflags = ["-D","warnings"]` — automates CI's doc
  gate cross-platform; closes the intra-doc-link gate-drift. `[AGENT][PUBLIC]`
  (Do **not** add `build.rustflags = -D warnings` — denies in deps + breaks incremental dev.)
- **`clippy.toml`** → `allow-*-in-tests`, complexity thresholds (pinned defaults),
  `doc-valid-idents`. Relaxes/pins/reduces-FP only. `[CONATIVE][AGENT]`

## Tier 2 — strictness-adoption workstream (each lint needs a cleanup pass)

### `Cargo.toml [workspace.lints.rust]` (all stable)
```toml
unsafe_code = "forbid"                  # keep
missing_docs = "warn"                   # keep
unreachable_pub = "warn"                # [REUSE][PUBLIC] shrink the real public API
missing_debug_implementations = "warn"  # [CONATIVE][AGENT] agents debug by printing
elided_lifetimes_in_paths = "warn"      # [CONATIVE] make borrows visible
unused_qualifications = "warn"          # [CONATIVE]
meta_variable_misuse = "warn"           # [AGENT] macro_rules! bug-catch (we ship macros)
rust_2018_idioms = { level = "warn", priority = -1 }   # [PUBLIC]
# trivial_casts / trivial_numeric_casts / unused_lifetimes / single_use_lifetimes / redundant_lifetimes = "warn"
# DECIDE-noisy: missing_copy_implementations, single_use_lifetimes (keep warn, never deny)
# DELIBERATELY allow: unused_results (noisy), unused_crate_dependencies (FPs → use cargo-machete)
```

### `Cargo.toml [workspace.lints.rustdoc]` — NEW table, all stable
```toml
broken_intra_doc_links = "deny"   # [AGENT][PUBLIC] THE lint we keep tripping — hard-deny
private_intra_doc_links = "warn"
invalid_codeblock_attributes = "warn"
invalid_rust_codeblocks = "warn"
invalid_html_tags = "warn"
bare_urls = "warn"
redundant_explicit_links = "warn"
unescaped_backticks = "warn"      # [AGENT] allow-by-default → enable (agents miscount backticks)
```
> Caveat: rustdoc lints fire only under `cargo doc` — which is NOT in the pre-commit hook yet.
> Add the doc step (below) or this is "CI-only-but-locally-runnable," not dev-time-enforced.

### `Cargo.toml [workspace.lints.clippy]` — keep pedantic+nursery+allows; add curated restriction
```toml
# no-panic / no-silent-failure (the agentic core):
unwrap_used = "warn"   expect_used = "warn"   panic = "warn"
todo = "warn"   unimplemented = "warn"   unreachable = "warn"
get_unwrap = "warn"   indexing_slicing = "warn"   # noisy in parser → per-line #[expect]
panic_in_result_fn = "warn"   unwrap_in_result = "warn"
let_underscore_must_use = "warn"   let_underscore_future = "warn"   unused_result_ok = "warn"
# no dev cruft:
dbg_macro = "warn"   mem_forget = "warn"   # print_stdout/print_stderr = allow (CLI) — deny in lib crates
# co-native standouts:
allow_attributes = "warn"
allow_attributes_without_reason = "warn"   # every #[allow]/#[expect] must carry reason="…"
# cargo group:
cargo = { level = "warn", priority = -1 }   multiple_crate_versions = "allow"
```
> NOT recommended (noise): arithmetic_side_effects (~85% FP), cast_* restriction, string_slice,
> shadow_*, single_call_fn, min_ident_chars, implicit_return, missing_docs_in_private_items.

### `rustfmt.toml` — stable committed + optional nightly profile
```toml
# STABLE (every contributor's `cargo fmt` works):
edition = "2021"   max_width = 100   newline_style = "Unix"   # kills CRLF churn (Win↔Linux)
reorder_imports = true   reorder_modules = true   use_field_init_shorthand = true
use_try_shorthand = true   match_block_trailing_comma = true   remove_nested_parens = true
```
```toml
# NIGHTLY-ONLY (.rustfmt-nightly.toml, run via `cargo +nightly fmt --config-path`):
unstable_features = true
imports_granularity = "Crate"        # [AGENT][REUSE] merge use-trees → fewer parallel-agent conflicts
group_imports = "StdExternalCrate"   wrap_comments = true   format_code_in_doc_comments = true
```
> Stable customization changes ~0 files today. The nightly import options would touch **142
> hunks** (one-time), and commit us to `cargo +nightly fmt` (a nightly fmt CI gate).

## Tooling — genuinely dev-time (add to pre-commit)
- **`cargo machete`** — unused deps, <1s. `[REUSE][PUBLIC]` (replaces flaky `unused_crate_dependencies`).
- **`typos`** — spelling in code+docs, sub-second. `[CONATIVE][PUBLIC]` (ship `typos.toml` allow-list).
- **`cargo doc` step** — `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --document-private-items`
  (the `--document-private-items` matches CI). Closes the rustdoc enforcement gap above.
- **CI, not dev-time:** `cargo deny` (dep-graph policy), `cargo hack` (feature powerset). Worth having; not write-loop checks.

## Decisions (with leans)
1. `expect_used` in src — **lean warn + allow-in-tests** (the message documents intent; debatable).
2. `indexing_slicing` in the parser — **per-line `#[expect]`**, don't blanket-disable.
3. `build.rustflags = -D warnings` — **NO** (deps + incremental).
4. Nightly rustfmt — **ADOPTED** (2026-06-06): `style_edition = "2027"` + structural opts; fmt gate moved to nightly. `imports_granularity` held at **Preserve**, NOT "Crate" — Crate caused `label-drift-from-silent-merge` (see § Config-level failure-classes). `wrap_comments` OFF.
5. `print_stdout/stderr` — **deny in the 4 lib crates, allow in `cargo-antigen`**.
6. `cargo machete` + `typos` — **yes**, fast pre-commit adds.
7. `disallowed-methods` seam — **verify-then-enable** in the adoption pass (check `process::exit` usage first).

## Config-level failure-classes (Beyond-Code seeds)

Two failure-classes surfaced while adopting the rustfmt profile (2026-06-06). They live at the
*config / tooling* layer, so they are not yet enforceable Rust-code antigens — they are **declared
here as Beyond-Code charter seeds** (antigen for md/toml/json/yaml, in-file or sidecar). Recorded as
MEMORY, not observed claims (ADR-029 observe-don't-declare). `rustfmt.toml` `#[orient]`s them by name.

```rust
#[antigen(
    name = "label-drift-from-silent-merge",
    family = "comment-asserted-property-drift",   // sibling of ParallelStateTrackersDiverge
    summary = "A formatter/codegen rule silently merges a documented unit into a sibling block, \
               so a comment labelling it (\"seven primitives\") ends up over different content \
               (14 items) — the label drifts from reality, uncaught.",
)]

#[antigen(
    name = "tooling-choice-spawns-unenforced-invariant",
    family = "implicit-discipline-debt",
    summary = "A config/tool option's side effect creates an invariant no compiler or lint \
               enforces ('every import block needs a comment or it silently merges') — \
               arbitrary discipline demanded for no structural reason. Stdlib-worthy (general).",
)]
```

**Instance:** `imports_granularity = "Crate"` presents *both*; we hold `Preserve` (see `rustfmt.toml`).

**Marker semantics worked out alongside (substrate for the learning-core / effector charters):**
- A `#[presents(X)]` is a *claim*, never self-certifying — the **fingerprint** adjudicates it
  (confirmed / dubious / suggested). A careless human mark and a scanner-inferred match are graded
  the same way: by structure, not by who marked it. Abuse-resistant by construction.
- Scan/audit may **materialize** findings into the source (so the codebase carries them, not just
  the scanner) — but as **ratifiable suggestions carrying provenance**, never silent assertions
  (effector-arm SUGGEST floor; AUTO-APPLY behind sub-clause-F, ADR-005).
- A felt-but-*unnamed* worry ("breaks under team-expansion / public PRs, can't say why") is the
  **marked-unknown plane** (`#[aura]` / `#[dread]` / `#[red_flag]`, kept honest by a required
  `trigger`), which **crystallizes** into `#[antigen]` + `#[presents]` once named.
