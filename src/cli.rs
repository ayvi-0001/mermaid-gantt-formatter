use std::io::{IsTerminal, Write};

use clap::{ArgAction, Command, arg};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};
use strum_macros::AsRefStr;

use crate::format::{GanttChart, MermaidDiagramFormatter};

/// Options must be lowercase.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, EnumIter, AsRefStr, IntoStaticStr, Default)]
pub enum FormatOptions {
    #[default]
    #[strum(serialize = "gantt")]
    gantt,
}

impl FormatOptions {
    pub fn get(opt: &str) -> Result<impl MermaidDiagramFormatter, &str> {
        if let Some(formatter) = Self::iter().find(|f| opt.trim().eq_ignore_ascii_case(f.as_ref()))
        {
            match formatter {
                Self::gantt => Ok(GanttChart::new()),
            }
        } else {
            Err("Could not find formatter type.")
        }
    }
}

pub fn build() -> Command {
    let type_parser = clap::builder::PossibleValuesParser::new(
        FormatOptions::iter()
            .map(|s| <&FormatOptions as Into<&str>>::into(&s).to_string())
            .collect::<Vec<String>>(),
    );

    Command::new(clap::crate_name!())
        .version(Version::short())
        .long_version(Version::long())
        .args(&[
            arg!(-i --input "Path to file, '-' for stdin.")
                .default_value("-")
                .action(ArgAction::Set),
            arg!(-o --output "Path to write to. Prints to stdout if none.")
                .required(false)
                .action(ArgAction::Set),
            arg!(-I --"in-place" "Format the input file in-place.")
                .requires("input")
                .action(ArgAction::SetTrue),
            arg!(-t --"type" "Formatter type.")
                .default_value(<FormatOptions as Into<&str>>::into(FormatOptions::default()))
                .value_parser(type_parser)
                .action(ArgAction::Set),
        ])
}

pub enum InputSource {
    Stdin(std::io::Stdin),
    File(std::fs::File),
}

impl std::io::Read for InputSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            InputSource::Stdin(s) => s.read(buf),
            InputSource::File(f) => f.read(buf),
        }
    }
}

pub fn create_or_replace_file(file_name: &String, contents: String) -> std::io::Result<()> {
    let mut file = std::fs::File::create(file_name)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn read_stdin(cmd: &Command) -> std::io::Stdin {
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        let err_msg = "No input file specified, and nothing read from stdin.\
             \u{20}If you want to specify an input file, please use \
             \u{20}`-i <input>.`, or `-i-` to read from stdin (default).\n";
        let err = clap::Error::raw(clap::error::ErrorKind::Io, err_msg).with_cmd(cmd);
        err.exit()
    }
    stdin
}

pub fn fail(err_msg: &str, cmd: &mut Command) -> ! {
    clap::Error::raw(clap::error::ErrorKind::Io, format!("{}\n", err_msg))
        .with_cmd(cmd)
        .exit()
}

pub fn fail_input(arg_context: &str, value_context: &str, cmd: &mut Command) -> ! {
    let mut err = clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
    err.insert(
        clap::error::ContextKind::InvalidArg,
        clap::error::ContextValue::String(arg_context.to_string()),
    );
    err.insert(
        clap::error::ContextKind::InvalidValue,
        clap::error::ContextValue::String(value_context.to_string()),
    );
    err.exit()
}

#[derive(Debug)]
struct Version {}

impl Version {
    pub fn short() -> String { clap::crate_version!().to_string() }

    pub fn long() -> String {
        format!(
            "{} ({} {})\ntriple: {}\nrustc: {}",
            clap::crate_version!(),
            env!("VERGEN_GIT_SHA"),
            env!("VERGEN_GIT_COMMIT_DATE"),
            env!("VERGEN_RUSTC_HOST_TRIPLE"),
            env!("VERGEN_RUSTC_SEMVER"),
        )
    }
}
