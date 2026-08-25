// Dedicated headless CLI binary (Phase 18 P2 epic).
//
// Unlike the main `tyny-pulse` desktop binary, this target carries no
// `windows_subsystem` attribute, so stdout/stderr attach correctly when
// invoked from Windows terminals in release builds (AGENTS.md tech-debt
// item #7). It reuses the exact same application layer through the library
// crate for 100% execution parity with the desktop app.

use tyny_pulse_lib::presentation::cli;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    if cli::is_cli_mode(&argv) {
        std::process::exit(cli::run_headless(&argv));
    }

    eprintln!("usage: tyny-cli run <collection.json> [OPTIONS]");
    eprintln!();
    eprintln!("Try 'tyny-cli --help' for more information.");
    std::process::exit(2);
}
