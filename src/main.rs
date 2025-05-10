mod cli;
mod format;

use std::io::{Read, Write};

use format::MermaidDiagramFormatter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cli::build();
    let args = cmd.get_matches_mut();

    let input = args
        .get_one::<String>("input")
        .expect("input is required.");
    let mut output = args.get_one::<String>("output");

    let diagram_type = args
        .get_one::<String>("type")
        .expect("type is required.");

    let mut input_source = if input.eq("-") {
        cli::InputSource::Stdin(cli::read_stdin(&cmd))
    } else {
        cli::InputSource::File(
            std::fs::File::open(input)
                .map_err(|e| cli::fail(&e.to_string(), &mut cmd))
                .unwrap(),
        )
    };

    if output.is_none() && args.get_flag("in-place") {
        if let cli::InputSource::File(_) = input_source {
            output = Some(input);
        }
    };

    let mut input_text = String::new();
    input_source.read_to_string(&mut input_text)?;

    if input_text.trim_ascii_end().chars().count() == 0 {
        cli::fail("No input detected.", &mut cmd)
    }

    match cli::FormatOptions::get(diagram_type) {
        Ok(mut formatter) => match formatter.format_diagram(&input_text) {
            Ok(diagram) => {
                if let Some(file_name) = output {
                    Ok(cli::create_or_replace_file(file_name, diagram)?)
                } else {
                    std::io::stdout().write_all(diagram.as_bytes())?;
                    Ok(())
                }
            }
            Err(e) => cli::fail(e, &mut cmd),
        },
        Err(e) => cli::fail_input("--type", e, &mut cmd),
    }
}
