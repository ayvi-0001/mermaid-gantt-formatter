use crate::format::{GanttChart, MermaidDiagramFormatter};

use std::io::{IsTerminal, Write};

use clap::{
    arg,
    error::{ContextKind, ContextValue, ErrorKind},
    ArgAction, Command,
};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};
use strum_macros::AsRefStr;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, EnumIter, AsRefStr, IntoStaticStr, Default)]
pub enum FormatOptions {
    #[default]
    #[strum(serialize = "gantt")]
    gantt,
}

impl FormatOptions {
    pub fn get(opt: &str) -> Option<impl MermaidDiagramFormatter> {
        if let Some(formatter) = Self::iter().find(|format_type| opt.eq(format_type.as_ref())) {
            match formatter {
                Self::gantt => Some(GanttChart::new()),
            }
        } else {
            None
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
                .default_value(<FormatOptions as Into<&str>>::into(
                    FormatOptions::default(),
                ))
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
        fail("--input", "", cmd)
    }
    stdin
}

pub fn fail(arg_context: &str, value_context: &str, cmd: &Command) -> ! {
    let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
    err.insert(
        ContextKind::InvalidArg,
        ContextValue::String(arg_context.to_string()),
    );
    err.insert(
        ContextKind::InvalidValue,
        ContextValue::String(value_context.to_string()),
    );
    err.exit()
}

struct Version {}

impl Version {
    pub fn short() -> String {
        clap::crate_version!().to_string()
    }

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
