//! Cargo subcommand for antigen.
//!
//! Provides `cargo antigen scan` (and future `new`, `vaccinate`, `audit`)
//! subcommands for working with antigen declarations in a Rust workspace.
//!
//! ## Status (v0.0.1)
//!
//! Initial functional release of `scan`. Other subcommands are stubs printing
//! design-phase notices.

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
    New {
        /// kebab-case name for the new antigen
        name: String,
    },
    /// Apply known immunity pattern across a structural family (design phase).
    Vaccinate {
        /// Antigen type to apply
        antigen: String,
        /// Pattern (glob or path) describing target sites
        pattern: String,
    },
    /// Comprehensive immunity coverage report (design phase).
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
}

#[derive(Debug, Parser)]
struct AuditArgs {
    /// Workspace root (default: current directory)
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// Exit with non-zero status if audit finds problematic immunity claims
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
    eprintln!("Scanning workspace: {}", args.root.display());

    let report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        }
    };

    let unaddressed = report.unaddressed_presentations();

    match args.format {
        OutputFormat::Human => {
            print_human_report(&report, &unaddressed);
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&JsonReport {
            report: &report,
            unaddressed: &unaddressed,
        }) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            }
        },
    }

    if args.strict && !unaddressed.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_human_report(report: &scan::ScanReport, unaddressed: &[scan::UnaddressedPresentation]) {
    println!();
    println!(
        "Scanned {} files, found {} antigen-related declarations:",
        report.files_scanned,
        report.total_declarations()
    );
    println!("  - {} antigen declarations", report.antigens.len());
    println!("  - {} presentations", report.presentations.len());
    println!("  - {} immunity claims", report.immunities.len());
    if !report.parse_failures.is_empty() {
        println!(
            "  - {} files failed to parse (see --format json for details)",
            report.parse_failures.len()
        );
    }
    println!();

    if unaddressed.is_empty() {
        println!("✓ No unaddressed presentations.");
        if !report.presentations.is_empty() {
            println!(
                "  All {} presentations have nearby immunity declarations.",
                report.presentations.len()
            );
        }
    } else {
        println!("⚠ {} unaddressed presentation(s):", unaddressed.len());
        println!();
        for u in unaddressed {
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
                    "    note: antigen `{}` was not declared in the scanned workspace; \
                     may be imported from an external crate or undeclared",
                    p.antigen_type
                );
            }
        }
        println!();
        println!("To address: add #[immune({}, witness = ...)] on the same item, OR mark with #[antigen_tolerance(...)]", unaddressed[0].presentation.antigen_type);
    }
}

#[derive(serde::Serialize)]
struct JsonReport<'a> {
    report: &'a scan::ScanReport,
    unaddressed: &'a [scan::UnaddressedPresentation],
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

    if args.strict && !audit_report.all_valid() {
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
    // Note: "resolved" means the witness identifier was found in the workspace.
    // It does NOT mean the witness was executed or that it asserts immunity to
    // this specific failure class. Full semantic validation requires fingerprint-
    // aware reasoning (ADR-010, planned for Sweep A3+).
    println!("Audited {} immunity claim(s):", audit_report.audits.len());
    println!(
        "  - {} declared (witness identifier found in workspace — not yet semantically verified)",
        audit_report.resolved_count
    );
    println!(
        "  - {} external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)",
        audit_report.external_count
    );
    println!(
        "  - {} broken (witness identifier not found)",
        audit_report.broken_count
    );
    println!(
        "  - {} missing (no witness identifier)",
        audit_report.missing_count
    );
    println!();

    let problematic = audit_report.problematic_audits();

    if problematic.is_empty() {
        println!("✓ All immunity claims are structurally well-formed (witness identifiers exist).");
        println!(
            "  Note: semantic verification (does the witness actually test this failure class?)"
        );
        println!("  requires fingerprint-aware audit, planned for Sweep A3+.");
        if scan_report.immunities.is_empty() {
            println!("  (No immunity declarations found in the workspace.)");
        }
    } else {
        println!("⚠ {} problematic immunity claim(s):", problematic.len());
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
                _ => {}
            }
        }
        println!();
        println!(
            "Resolve broken witnesses by either:\n  \
             a) Adding the witness function to the workspace\n  \
             b) Updating the witness reference to point at an existing function\n  \
             c) Removing the immunity claim if it's premature"
        );
    }
}
