mod cli;
mod format;

use std::io::Read;

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
        cli::InputSource::File(std::fs::File::open(input)?)
    };

    if output.is_none() && args.get_flag("in-place") {
        if let cli::InputSource::File(_) = input_source {
            output = Some(input);
        }
    };

    let mut input_text = String::new();
    input_source.read_to_string(&mut input_text)?;

    match cli::FormatOptions::get(diagram_type) {
        Some(mut formatter) => {
            if let Ok(diagram) = formatter.format_diagram(&input_text) {
                if let Some(file_name) = output {
                    Ok(cli::create_or_replace_file(file_name, diagram)?)
                } else {
                    println!("{}", diagram);
                    Ok(())
                }
            } else {
                panic!("Something went wrong, failed to format diagram.")
            }
        }
        _ => cli::fail(
            "--type",
            "Could not determine formatter type.",
            &cmd,
        ),
    }
}
