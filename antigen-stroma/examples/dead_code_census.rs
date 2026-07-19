//! Dead-code census — the first F-axis producer, end to end (nb 028 §4 / nb 029 §5).
//!
//! Runs `cargo check --message-format=json` on a target repo (the impure subprocess — the library
//! `dead_code_absences` parser stays pure) and reports the witnessed absences (`AbsenceWitness::DeadCode`):
//! items the compiler says are declared-but-never-used — clean F.
//!
//! A disciplined, warnings-clean codebase (antigen itself) yields 0 — dead-code is a *corpus* signal, not
//! a self-dogfood one. Point it at a rougher repo (or the deadcode-probe) to watch F fire.
//!
//! Usage: `cargo run -p antigen-stroma --example dead_code_census -- [path/to/repo]` (default `.`).

use std::process::Command;

use antigen_stroma::dead_code::dead_code_absences;

fn main() {
    let target = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    println!("== Dead-code census (witnessed absence, F) on {target} ==\n");

    let out = Command::new("cargo")
        .args(["check", "--message-format=json"])
        .current_dir(&target)
        .output()
        .expect("cargo must be on PATH");
    let json = String::from_utf8_lossy(&out.stdout);

    let absences = dead_code_absences(&json);
    println!(
        "dead-code absences (AbsenceWitness::DeadCode): {}",
        absences.len()
    );
    for a in absences.iter().take(40) {
        println!("  {}", a.subject);
    }
    if absences.is_empty() {
        println!("  (none — a warnings-clean codebase, or dead_code is denied here)");
    }
    println!(
        "\n(F = a witnessed absence-of-use, held in AbsenceFacts; the join to a graph node — subject \
         file:line → the containing symbol — is an organ's projection, a later slice.)"
    );
}
