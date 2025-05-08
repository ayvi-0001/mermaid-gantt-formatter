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
mod cli;
mod format;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cli::build();
    let args = cmd.get_matches_mut();

    let input = args
        .get_one::<String>("input")
        .expect("input is required.");

    let output = args.get_one::<String>("output");

    let input_source = if input.eq("-") {
        format::InputSource::Stdin(cli::read_stdin(&cmd))
    } else {
        format::InputSource::File(std::fs::File::open(input)?)
    };

    let mut input_text = String::new();
    format::read_input_to_string(input_source, &mut input_text)?;

    let mut gantt_chart = format::GanttChart::new();
    gantt_chart.parse_text(&input_text);

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
