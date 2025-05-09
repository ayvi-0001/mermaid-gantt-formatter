/// Formatter for Mermaid Gantt Charts.
/// Mermaid Gantt diagram Documentation https://mermaid.js.org/syntax/gantt.html#gantt-diagrams
mod cli;
mod format;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cli::build();
    let args = cmd.get_matches_mut();

    let input = args
        .get_one::<String>("input")
        .expect("input is required.");

    let mut output = args.get_one::<String>("output");

    let input_source = if input.eq("-") {
        format::InputSource::Stdin(cli::read_stdin(&cmd))
    } else {
        format::InputSource::File(std::fs::File::open(input)?)
    };

    if let format::InputSource::File(_) = input_source {
        if output.is_none() && args.get_flag("in-place") {
            output = Some(input);
        }
    }

    let mut input_text = String::new();
    format::read_input_to_string(input_source, &mut input_text)?;

    let mut gantt_chart = format::GanttChart::new();

    match gantt_chart.parse_text(&input_text) {
        Err(e) => cli::fail("--input", e, &cmd),
        Ok(_) => {
            if let Some(file_name) = output {
                Ok(format::create_or_replace_file(
                    file_name,
                    gantt_chart.to_string(),
                )?)
            } else {
                println!("{}", gantt_chart);
                Ok(())
            }
        }
    }
}
