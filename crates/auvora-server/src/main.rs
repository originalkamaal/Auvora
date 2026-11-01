//! The `auvora` binary.
//!
//! Phase 0 skeleton: it only reports its version. Roles (`serve --roles=…`,
//! ADR-0005) and CLI subcommands arrive in later P0/P1 commits.

use std::io::{self, Write};
use std::process::ExitCode;

const USAGE: &str = "\
auvora: unified identity, access and authorization platform

USAGE:
    auvora <COMMAND>

COMMANDS:
    version    Print version information
    help       Print this message
";

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    let mut stdout = io::stdout().lock();
    let result = match command.as_deref() {
        Some("version" | "--version" | "-V") => {
            writeln!(stdout, "auvora {}", env!("CARGO_PKG_VERSION"))
        }
        Some("help" | "--help" | "-h") | None => stdout.write_all(USAGE.as_bytes()),
        Some(other) => {
            let _ = writeln!(io::stderr(), "error: unknown command '{other}'\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    if result.is_err() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
