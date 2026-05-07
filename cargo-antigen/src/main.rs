//! Cargo subcommand for antigen — design phase.
//!
//! This binary reserves the `cargo-antigen` namespace. Real subcommands
//! (`scan`, `new`, `vaccinate`, `audit`) are under design at
//! <https://github.com/antigen-rs/antigen>.

fn main() {
    eprintln!(
        "cargo-antigen 0.0.1 — design phase\n\
         \n\
         The antigen project is in active design. Reserved subcommands:\n\
           cargo antigen scan        scan codebase for unaddressed antigen presentations\n\
           cargo antigen new <name>  scaffold a new antigen declaration\n\
           cargo antigen vaccinate   apply immunity pattern across a structural family\n\
           cargo antigen audit       comprehensive immunity coverage report\n\
         \n\
         See https://github.com/antigen-rs/antigen for design progress.\n"
    );
    std::process::exit(0);
}
