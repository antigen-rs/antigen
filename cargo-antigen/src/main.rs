//! Cargo subcommand for antigen.
//!
//! Provides the `cargo antigen <subcommand>` CLI for working with antigen
//! declarations in a Rust workspace. After `cargo install cargo-antigen`,
//! invoke as `cargo antigen scan` / `cargo antigen audit` from any directory
//! containing a `Cargo.toml`.
//!
//! ## Subcommands (v0.1.0-rc.1)
//!
//! - `cargo antigen scan` — walk the workspace, extract antigen-related
//!   attributes, report unaddressed presentations + tolerated sites + parse
//!   failures. Supports `--include-deps` for cross-crate enumeration
//!   (ADR-017) and `--format json` for machine-readable output.
//!   See `cargo antigen scan --help` for the full surface.
//! - `cargo antigen audit` — validate each immunity declaration's witness
//!   identifier against the workspace function index. Classifies witnesses
//!   by `WitnessTier` (`Reachability` / `Execution` / `FormalProof` / `None`) and
//!   emits state-7 diagnostics for inherited presentations lacking
//!   re-attestation (ADR-018). `--strict` gates CI on tier minimums +
//!   state-7 absence.
//!
//! Two additional subcommands (`new`, `vaccinate`) exist as design-phase
//! stubs but are hidden from `--help` until they ship beyond stub state
//! (per A3.5 onboarding sweep).
//!
//! ## See also
//!
//! - [`antigen`](https://docs.rs/antigen) — the library crate with the
//!   attribute macros and the `scan` + `audit` modules this binary drives.
//! - The project's
//!   [`docs/tutorial.md`](https://github.com/antigen-rs/antigen/blob/main/docs/tutorial.md)
//!   for the narrative walkthrough.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use antigen::{audit, scan};

/// Cargo subcommand for antigen.
#[derive(Debug, Parser)]
#[command(name = "cargo-antigen", bin_name = "cargo")]
struct CargoCli {
    #[command(subcommand)]
    command: CargoSubcommand,
}

#[derive(Debug, Subcommand)]
enum CargoSubcommand {
    /// The "antigen" subcommand of cargo.
    Antigen(AntigenCli),
}

#[derive(Debug, Parser)]
struct AntigenCli {
    #[command(subcommand)]
    command: AntigenSubcommand,
}

#[derive(Debug, Subcommand)]
enum AntigenSubcommand {
    /// Scan the workspace for antigen presentations and report unaddressed ones.
    Scan(ScanArgs),
    /// Scaffold a new antigen declaration (design phase).
    ///
    /// Hidden from `--help` output until the command ships beyond its
    /// design-phase stub. Stub message remains for users who discover the
    /// name via docs or history. Per the onboarding sweep (Phase 1) +
    /// Tekgy's directive: show new users the surface that works, not the
    /// surface that doesn't.
    #[command(hide = true)]
    New {
        /// kebab-case name for the new antigen
        name: String,
    },
    /// Apply known immunity pattern across a structural family (design phase).
    ///
    /// Hidden from `--help` output until the command ships beyond its
    /// design-phase stub (see `New` above for the same discipline).
    #[command(hide = true)]
    Vaccinate {
        /// Antigen type to apply
        antigen: String,
        /// Pattern (glob or path) describing target sites
        pattern: String,
    },
    /// Comprehensive immunity coverage report — witness resolution and tier validation.
    Audit(AuditArgs),
}

#[derive(Debug, Parser)]
struct ScanArgs {
    /// Workspace root (default: current directory)
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// Exit with non-zero status if unaddressed presentations are found
    #[arg(long)]
    strict: bool,
    /// Also scan dependency crates (registry/git) resolved by `cargo metadata`.
    /// Each dep is scanned independently; results appear under `dep_reports`
    /// in JSON output. Per A3 scope-lock: no cross-crate `addresses()` matching
    /// in v0.1 — each crate's report stays its own bag of antigens. Default OFF
    /// for backward compatibility.
    #[arg(long)]
    include_deps: bool,
}

#[derive(Debug, Parser)]
struct AuditArgs {
    /// Workspace root (default: current directory)
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// Exit with non-zero status if any immunity witness is unresolved
    /// (`Missing`, `NotFound`, or `Ambiguous`). v0.1: gates on `Reachability`
    /// tier minimum; `Execution`-tier gating arrives in A3 with `cargo test`
    /// integration.
    #[arg(long)]
    strict: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

fn main() -> ExitCode {
    let cli = CargoCli::parse();
    let CargoSubcommand::Antigen(antigen_cli) = cli.command;

    match antigen_cli.command {
        AntigenSubcommand::Scan(args) => run_scan(args),
        AntigenSubcommand::New { name } => run_new(name),
        AntigenSubcommand::Vaccinate { antigen, pattern } => run_vaccinate(antigen, pattern),
        AntigenSubcommand::Audit(args) => run_audit(args),
    }
}

fn run_scan(args: ScanArgs) -> ExitCode {
    if !args.root.exists() {
        eprintln!("error: path does not exist: {}", args.root.display());
        return ExitCode::from(2);
    }
    if !args.root.is_dir() {
        eprintln!(
            "error: expected a directory, got a file: {}",
            args.root.display()
        );
        return ExitCode::from(2);
    }
    eprintln!("Scanning workspace: {}", args.root.display());

    let report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        }
    };

    let unaddressed = report.unaddressed_presentations();

    // A3 D3: optional cross-crate dep enumeration + per-crate scan. Per
    // navigator's 2026-05-09 ruling, deps are scanned independently — no
    // cross-crate addresses() matching in v0.1. Each dep report stands on
    // its own; the union appears under `dep_reports` in JSON output.
    let dep_reports = if args.include_deps {
        match scan::enumerate_dep_crate_roots(&args.root, false) {
            Ok(roots) => {
                eprintln!("Scanning {} dependency crate(s)...", roots.len());
                let mut out: Vec<DepScanResult> = Vec::with_capacity(roots.len());
                for dep in roots {
                    let mut r = match scan::scan_workspace(&dep.crate_root, None) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!(
                                "  warning: scan failed for `{}` v{}: {e}",
                                dep.package_name, dep.version
                            );
                            continue;
                        }
                    };
                    // ADR-017 Option A: stamp canonical_path post-scan in
                    // the `"<crate-name>@<version>"` format. The
                    // scan_workspace function is a pure directory scanner
                    // and does not know which crate it just scanned; the
                    // driver carries the crate-graph context and stamps
                    // here.
                    let crate_id = format!("{}@{}", dep.package_name, dep.version);
                    r.stamp_canonical_path(&crate_id);
                    out.push(DepScanResult {
                        package_name: dep.package_name,
                        version: dep.version,
                        origin: dep.origin,
                        report: r,
                    });
                }
                Some(out)
            }
            Err(e) => {
                eprintln!("error: cargo metadata failed: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        None
    };

    match args.format {
        OutputFormat::Human => {
            print_human_report(&report, &unaddressed);
            if let Some(deps) = dep_reports.as_ref() {
                print_human_dep_summary(deps);
            }
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&JsonReport {
            report: &report,
            unaddressed: &unaddressed,
            dep_reports: dep_reports.as_deref(),
        }) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            }
        },
    }

    if args.strict && (!unaddressed.is_empty() || !report.orphaned_tolerances().is_empty()) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// Render a brief summary of dep scan results in human format. Per
/// navigator's ruling, dep reports are independent — we only summarize
/// counts here (full per-crate detail goes through --format json).
fn print_human_dep_summary(deps: &[DepScanResult]) {
    println!();
    println!("Cross-crate dep scan ({} crates):", deps.len());
    let mut deps_with_antigens: Vec<&DepScanResult> = deps
        .iter()
        .filter(|d| !d.report.antigens.is_empty())
        .collect();
    deps_with_antigens.sort_by_key(|d| d.package_name.clone());
    if deps_with_antigens.is_empty() {
        println!("  No antigen declarations found in any dependency.");
        println!("  (P5 finding 2026-05-09: zero `#[antigen(...)]` instances in the wild.)");
    } else {
        for d in deps_with_antigens {
            println!(
                "  {} v{}: {} antigen(s), {} presentation(s), {} immunity claim(s)",
                d.package_name,
                d.version,
                d.report.antigens.len(),
                d.report.presentations.len(),
                d.report.immunities.len()
            );
        }
    }
}

fn print_human_report(report: &scan::ScanReport, unaddressed: &[scan::UnaddressedPresentation]) {
    use antigen::scan::MatchKind;

    let explicit_count = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::ExplicitMarker)
        .count();
    let fingerprint_count = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .count();

    println!();
    println!(
        "Scanned {} files, found {} antigen-related declarations:",
        report.files_scanned,
        report.total_declarations()
    );
    println!("  - {} antigen declarations", report.antigens.len());
    println!("  - {} explicit #[presents] markers", explicit_count);
    if fingerprint_count > 0 {
        println!("  - {fingerprint_count} fingerprint matches (unmarked sites)");
    }
    if !report.tolerances.is_empty() {
        println!(
            "  - {} tolerated sites (#[antigen_tolerance])",
            report.tolerances.len()
        );
    }
    println!("  - {} immunity claims", report.immunities.len());
    if !report.parse_failures.is_empty() {
        println!(
            "  - {} parse failures (see --format json for details)",
            report.parse_failures.len()
        );
    }
    println!();

    print_fingerprint_matches(report);
    print_orphaned_tolerances(report);
    print_unaddressed(unaddressed);
}

fn print_fingerprint_matches(report: &scan::ScanReport) {
    use antigen::scan::MatchKind;
    let fp_matches: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .collect();
    if fp_matches.is_empty() {
        return;
    }
    println!(
        "{} fingerprint match(es) — structurally similar to a declared antigen:",
        fp_matches.len()
    );
    println!();
    for p in &fp_matches {
        println!(
            "  {}:{}  {} on {} [fingerprint match]",
            p.file.display(),
            p.line,
            p.antigen_type,
            p.item_kind
        );
    }
    println!();
    println!("  To acknowledge each site, use the antigen type shown above:");
    println!("    #[presents(<antigen>)] to mark explicitly,");
    println!("    #[immune(<antigen>, witness = ...)] if defended,");
    println!("    #[antigen_tolerance(<antigen>, rationale = \"...\")] to document intent.");
    println!();
}

fn print_orphaned_tolerances(report: &scan::ScanReport) {
    let orphans = report.orphaned_tolerances();
    if orphans.is_empty() {
        return;
    }
    println!(
        "{} orphaned tolerance(s) — antigen no longer declared in workspace:",
        orphans.len()
    );
    println!();
    for t in &orphans {
        println!(
            "  {}:{}  {} [tolerance for unknown antigen]",
            t.file.display(),
            t.line,
            t.antigen_type
        );
    }
    println!();
    println!("  Remove or update these tolerances — the antigen they suppress is gone.");
    println!();
}

fn print_unaddressed(unaddressed: &[scan::UnaddressedPresentation]) {
    use antigen::scan::MatchKind;
    let explicit_unaddressed: Vec<_> = unaddressed
        .iter()
        .filter(|u| u.presentation.match_kind == MatchKind::ExplicitMarker)
        .collect();
    if explicit_unaddressed.is_empty() {
        println!("All explicit presentations are addressed.");
        return;
    }
    println!(
        "{} unaddressed explicit presentation(s):",
        explicit_unaddressed.len()
    );
    println!();
    for u in &explicit_unaddressed {
        let p = &u.presentation;
        println!(
            "  {}:{}  {} on {}",
            p.file.display(),
            p.line,
            p.antigen_type,
            p.item_kind
        );
        if !u.antigen_known {
            println!(
                "    note: antigen `{}` was not declared in the scanned workspace",
                p.antigen_type
            );
        }
    }
    println!();
    println!("To address each site, use the antigen type shown above:");
    println!("  #[immune(<antigen>, witness = ...)] on the same item, OR #[antigen_tolerance(<antigen>, rationale = \"...\")]");
}

#[derive(serde::Serialize)]
struct JsonReport<'a> {
    report: &'a scan::ScanReport,
    unaddressed: &'a [scan::UnaddressedPresentation],
    /// A3 D3: when `--include-deps` is set, one entry per scanned dep
    /// crate. `None` (skipped in JSON output via `skip_serializing_if`)
    /// when the flag wasn't passed — preserves byte-identical output for
    /// existing consumers.
    #[serde(skip_serializing_if = "Option::is_none")]
    dep_reports: Option<&'a [DepScanResult]>,
}

/// Per-dependency scan result returned by the `--include-deps` mode of
/// `cargo antigen scan`. Each dep is scanned independently — per navigator's
/// 2026-05-09 ruling on cross-crate scope, no cross-crate `addresses()`
/// matching happens here (ATK-A3-005 / module-path-qualified `ItemTarget`
/// is an ADR-class decision deferred to aristotle Phase 1-8).
#[derive(serde::Serialize)]
struct DepScanResult {
    package_name: String,
    version: String,
    origin: scan::CrateOrigin,
    report: scan::ScanReport,
}

fn run_new(name: String) -> ExitCode {
    eprintln!(
        "cargo-antigen new {} — design phase\n\
         \n\
         Antigen scaffolding (interactive scaffolding for a new declaration) is\n\
         under design. The eventual command will:\n\
           - Prompt for family (one of the 8 first-principles classes or custom)\n\
           - Prompt for fingerprint (assist with structural pattern composition)\n\
           - Prompt for witness type (test, proptest, lint, formal-verification)\n\
           - Generate a starter declaration file in your project's antigen module\n\
         \n\
         For now, please write antigen declarations by hand. See the project's\n\
         docs/expedition/conventions.md for naming guidance.\n",
        name
    );
    ExitCode::SUCCESS
}

fn run_vaccinate(antigen: String, pattern: String) -> ExitCode {
    eprintln!(
        "cargo-antigen vaccinate {} {} — design phase\n\
         \n\
         Vaccination (apply known immunity pattern across a structural family) is\n\
         under design. The eventual command will:\n\
           - Search the workspace for items matching `pattern`\n\
           - For each match without existing immunity, scaffold a witness stub\n\
           - Add #[presents] and #[immune] markers atomically with confirmation\n\
         \n\
         For now, apply immunity manually per site.\n",
        antigen, pattern
    );
    ExitCode::SUCCESS
}

fn run_audit(args: AuditArgs) -> ExitCode {
    if !args.root.exists() {
        eprintln!("error: path does not exist: {}", args.root.display());
        return ExitCode::from(2);
    }
    if !args.root.is_dir() {
        eprintln!(
            "error: expected a directory, got a file: {}",
            args.root.display()
        );
        return ExitCode::from(2);
    }
    eprintln!("Auditing workspace: {}", args.root.display());

    let scan_report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        }
    };

    let audit_report = audit::audit(&scan_report, &args.root);

    match args.format {
        OutputFormat::Human => {
            print_audit_human(&scan_report, &audit_report);
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&JsonAuditReport {
            scan: &scan_report,
            audit: &audit_report,
        }) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            }
        },
    }

    // ADR-018 §"Audit diagnostic text" + §"7-state interaction matrix":
    // `--strict` promotes state 7 (inherited + unaddressed) from warn to
    // error. Without `--strict`, the audit reports state 7 as a warning
    // but still exits 0.
    let strict_state7_fails =
        args.strict && !audit_report.inherited_unaddressed.is_empty();
    let strict_witness_fails =
        args.strict && !audit_report.all_meet_tier(audit::WitnessTier::Reachability);
    if strict_state7_fails || strict_witness_fails {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

#[derive(serde::Serialize)]
struct JsonAuditReport<'a> {
    scan: &'a scan::ScanReport,
    audit: &'a audit::AuditReport,
}

fn print_audit_human(scan_report: &scan::ScanReport, audit_report: &audit::AuditReport) {
    println!();
    print_audit_summary(audit_report);
    println!();

    // ATK-A3-019 fix: confirmed-claims block, parallel to the warnings
    // block below. Above-Execution-tier claims get an explicit positive
    // confirmation; without it, a FormalProof witness is invisible in the
    // human-readable output (the warnings block only lists claims BELOW
    // Execution). The JSON output has always carried this; the human path
    // was display-incomplete.
    print_confirmed_immunity_claims(audit_report);

    let problematic = audit_report.problematic_audits();

    if problematic.is_empty() {
        println!("✓ All immunity claims meet the Execution tier or higher.");
        println!(
            "  Note: semantic verification (does the witness actually test this failure class?)"
        );
        println!("  requires fingerprint-aware audit, planned for Sweep A4-A5.");
        if scan_report.immunities.is_empty() {
            println!("  (No immunity declarations found in the workspace.)");
        }
    } else {
        println!(
            "⚠ {} immunity claim(s) below Execution tier:",
            problematic.len()
        );
        println!();
        for a in &problematic {
            let i = &a.immunity;
            println!(
                "  {}:{}  {} (witness = `{}`)",
                i.file.display(),
                i.line,
                i.antigen_type,
                i.witness
            );
            println!("    tier = {:?}, hint = {:?}", a.witness_tier, a.audit_hint,);
            match &a.witness_status {
                audit::WitnessStatus::NotFound { reason } => {
                    println!("    → broken: {reason}");
                }
                audit::WitnessStatus::Missing => {
                    println!(
                        "    → missing: declaration has no witness identifier; \
                         a marker without proof is not a claim (per ADR-005)"
                    );
                }
                audit::WitnessStatus::Ambiguous { candidates } => {
                    println!(
                        "    → ambiguous: witness name matches {} workspace functions",
                        candidates.len(),
                    );
                    for c in candidates {
                        println!("        - {}", c.display());
                    }
                    println!(
                        "      Fix: rename one of the colliding functions, or \
                         qualify the witness path"
                    );
                }
                audit::WitnessStatus::External { tool_hint } => {
                    println!(
                        "    → external ({tool_hint}): tool prefix recognized but not invoked. \
                         A3+ will run the tool to promote this witness to Execution tier."
                    );
                }
                audit::WitnessStatus::Resolved { .. } => {
                    // Resolved witnesses below Execution tier (Reachability):
                    // empty function bodies, ignored tests, or unrun tests.
                    // The hint already says which case applies.
                }
            }
        }
        println!();
        println!(
            "Resolve below-Execution claims by either:\n  \
             a) Adding test invocation that exercises the witness path (A4-A5 feature)\n  \
             b) Pointing the witness at a runnable test (#[test] without #[ignore])\n  \
             c) Renaming colliding functions or qualifying ambiguous witness paths\n  \
             d) Adding the witness function to the workspace if it's missing\n  \
             e) Tolerating the gap with `#[antigen_tolerance(...)]` if intentional"
        );
    }

    print_state7_diagnostics(audit_report);
}

/// Audit summary with per-tier sub-counts per ATK-A3-019 (A3.5 onboarding sweep).
///
/// Per ADR-005 Amendment 3: tier counts report the work the audit ACTUALLY
/// PERFORMED, never potential maximum evidence. A `#[test]` whose run was
/// not invoked sits at Reachability, not Execution.
///
/// `resolved_count` is split into per-tier sub-counts so a `FormalProof`
/// claim (phantom-type witness, type-system-encoded proof) does NOT get
/// labeled "not yet semantically verified" — that label is true for
/// `Reachability`-tier resolutions only. `FormalProof` is semantically
/// verified at compile time; `Execution` is verified by an executed test run.
fn print_audit_summary(audit_report: &audit::AuditReport) {
    let formal_proof_count = audit_report
        .audits
        .iter()
        .filter(|a| a.witness_tier == audit::WitnessTier::FormalProof)
        .count();
    let execution_count = audit_report
        .audits
        .iter()
        .filter(|a| a.witness_tier == audit::WitnessTier::Execution)
        .count();
    // `Resolved` status entries that aren't FormalProof or Execution sit at
    // Reachability — the original "declared but not semantically verified"
    // case. resolved_count is the total of Resolved entries from the
    // AuditReport bookkeeping; the remainder after subtracting the higher
    // tiers is what stays at Reachability.
    let reachability_resolved_count = audit_report
        .resolved_count
        .saturating_sub(formal_proof_count + execution_count);

    println!("Audited {} immunity claim(s):", audit_report.audits.len());
    if formal_proof_count > 0 {
        println!(
            "  - {formal_proof_count} formal-proof (phantom-type or formal-verification \
             tool — compile-time evidence)"
        );
    }
    if execution_count > 0 {
        println!(
            "  - {execution_count} execution (test/proptest run confirmed by audit)"
        );
    }
    println!(
        "  - {reachability_resolved_count} declared (witness identifier found in \
         workspace — not yet semantically verified)"
    );
    println!(
        "  - {} external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)",
        audit_report.external_count
    );
    println!(
        "  - {} ambiguous (witness name resolves to multiple workspace functions)",
        audit_report.ambiguous_count
    );
    println!(
        "  - {} broken (witness identifier not found)",
        audit_report.broken_count
    );
    println!(
        "  - {} missing (no witness identifier)",
        audit_report.missing_count
    );
}

/// Confirmed-immunity-claims block per ATK-A3-019 (A3.5 onboarding sweep).
///
/// Parallel to the warnings block but for the positive case: lists immunity
/// claims whose witness reached `Execution` or `FormalProof` tier — the
/// audit tiers that represent confirmed evidence rather than mere
/// reachability. Without this block, a `FormalProof` claim (phantom-type
/// witness) was invisible in human-readable output; the warnings block only
/// surfaces below-Execution claims.
fn print_confirmed_immunity_claims(audit_report: &audit::AuditReport) {
    let confirmed: Vec<&audit::ImmunityAudit> = audit_report
        .audits
        .iter()
        .filter(|a| a.witness_tier >= audit::WitnessTier::Execution)
        .collect();
    if confirmed.is_empty() {
        return;
    }
    println!(
        "✓ {} immunity claim(s) at Execution tier or higher:",
        confirmed.len()
    );
    println!();
    for a in &confirmed {
        let i = &a.immunity;
        println!(
            "  {}:{}  {} (witness = `{}`)",
            i.file.display(),
            i.line,
            i.antigen_type,
            i.witness
        );
        println!("    tier = {:?}, hint = {:?}", a.witness_tier, a.audit_hint);
    }
    println!();
}

fn print_state7_diagnostics(audit_report: &audit::AuditReport) {
    // ADR-018 §"Audit diagnostic text": state-7 warnings (inherited
    // Presentations lacking re-attestation on the descendant site).
    if audit_report.inherited_unaddressed.is_empty() {
        return;
    }
    println!();
    println!(
        "⚠ {} inherited presentation(s) not re-attested on the descendant \
         (state 7 of the 7-state interaction matrix):",
        audit_report.inherited_unaddressed.len()
    );
    println!();
    for iu in &audit_report.inherited_unaddressed {
        let p = &iu.presentation;
        let ancestors: Vec<String> = p
            .inherited_from
            .as_ref()
            .map(|chain| chain.iter().map(|pe| pe.antigen_type.clone()).collect())
            .unwrap_or_default();
        println!(
            "  warning: inherited presentation: `{}` flowed from {:?} \
             to `{}` via `#[descended_from]`;",
            p.antigen_type, ancestors, p.item_kind
        );
        println!(
            "  the witness inherited from the ancestor has not been \
             re-attested on the descendant."
        );
        println!(
            "  Add `#[immune({}, witness = ...)]` or \
             `#[antigen_tolerance({}, rationale = \"...\")]` on the \
             descendant.",
            p.antigen_type, p.antigen_type
        );
        println!("    --> {}:{}", p.file.display(), p.line);
        println!();
    }
    println!(
        "  Note: behavioral re-validation (does the ancestor's witness \
         apply to the descendant?) is A4-A5 work; reachability-tier \
         audit cannot perform this check."
    );
    println!(
        "  Use `cargo antigen audit --strict` to promote state-7 \
         warnings to errors for CI gating."
    );
}
