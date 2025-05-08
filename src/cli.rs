use std::io::IsTerminal;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{arg, ArgAction, Command};

pub fn build() -> Command {
    Command::new("fmt-mmd-gantt")
        .version(Version::short())
        .long_version(Version::long())
        .args(&[
            arg!(-i --input "Path to file, '-' for stdin.")
                .default_value("-")
                .action(ArgAction::Set),
            arg!(-o --output "Path to write to. Prints to stdout if none.")
                .required(false)
                .action(ArgAction::Set),
        ])
}

pub fn read_stdin(cmd: &Command) -> std::io::Stdin {
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        let mut err = clap::Error::new(ErrorKind::InvalidValue).with_cmd(cmd);
        err.insert(
            ContextKind::InvalidArg,
            ContextValue::String("--input".to_string()),
        );
        err.insert(
            ContextKind::InvalidValue,
            ContextValue::String("".to_string()),
        );
        err.exit()
    }
    stdin
}

struct Version {}

impl Version {
    pub fn short() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    pub fn long() -> String {
        format!(
            "{} ({} {})\ntriple: {}\nrustc: {}",
            env!("CARGO_PKG_VERSION"),
            env!("VERGEN_GIT_SHA"),
            env!("VERGEN_GIT_COMMIT_DATE"),
            env!("VERGEN_RUSTC_HOST_TRIPLE"),
            env!("VERGEN_RUSTC_SEMVER"),
        )
    }
}
