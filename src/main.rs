/// Formatter for Mermaid Gantt Charts. Example:
/// gantt                                                    ->  gantt
///     title A Gantt Diagram                                ->    title A Gantt Diagram
///     dateFormat YYYY-MM-DD                                ->    dateFormat YYYY-MM-DD
///     section Section                                      ->
///         A task          :done, a1, 2014-01-01, 30d       ->    section Section
///         Another task    :active, a2, after a1, 20d       ->      A task           : done  ,                     a1     ,  2014-01-01   ,  30d
///         A milestone : milestone, after a2                ->      Another task     : active,                     a2     ,  after a1     ,  20d
///     section Another                                      ->      A milestone      :                 milestone,                            after a2
///         Task in Another :crit,taskid1,2014-01-12, 12d    ->
///         another task    :taskid2,after taskid1, 24d      ->    section Another
///                                                          ->      Task in Another  :          crit,              taskid1,  2014-01-12   ,  12d
///                                                          ->      another task     :                             taskid2,  after taskid1,  24d
///
/// Mermaid Gantt diagram Documentation https://mermaid.js.org/syntax/gantt.html#gantt-diagrams
///
mod format;

use std::fs::File;
use std::io::{self, Read, Write};

use clap::{Arg, ArgAction, ArgMatches, Command};
use format::GanttChart;

fn create_or_replace_file(file_name: &String, contents: String) -> io::Result<()> {
    let mut file: File = File::create(file_name)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

enum InputSource {
    Stdin(io::Stdin),
    File(File),
}

impl io::Read for InputSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            InputSource::Stdin(s) => s.read(buf),
            InputSource::File(f) => f.read(buf),
        }
    }
}

struct Version {}

impl Version {
    fn short() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn long() -> String {
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

fn main() -> io::Result<()> {
    let arg_matches: ArgMatches = Command::new("fmt-mmd-gantt")
        .version(Version::short())
        .long_version(Version::long())
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .required(true)
                .value_name("INPUT")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .action(ArgAction::Set),
        )
        .get_matches();

    let input: Option<&String> = arg_matches.get_one::<String>("input");
    let output: Option<&String> = arg_matches.get_one::<String>("output");

    let input_source: InputSource = if input.unwrap() == "-" {
        InputSource::Stdin(io::stdin())
    } else {
        InputSource::File(File::open(input.unwrap())?)
    };

    let mut input_text = String::new();

    io::BufReader::new(input_source).read_to_string(&mut input_text)?;

    let mut gantt_chart = GanttChart::new();

    gantt_chart.parse_text(&input_text);

    if let Some(file_path) = output {
        create_or_replace_file(file_path, gantt_chart.to_string())
    } else if input.unwrap().eq("-") {
        println!("{}", gantt_chart);
        Ok(())
    } else {
        create_or_replace_file(input.unwrap(), gantt_chart.to_string())
    }
}
