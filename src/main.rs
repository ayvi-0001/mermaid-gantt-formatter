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
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Map;
use std::ops::Add;
use std::str::Split;
use std::vec::IntoIter;

/// Required/optional keywords that may appear at the top of a mermaid gantt file or elsewhere.
/// Note: this is not an exhaustive list. This script doesn't currently account for YAML frontmatter. https://mermaid.js.org/config/configuration.html#frontmatter-config
const MMD_GANTT_KWS: [&str; 23] = [
    "axisFormat",
    "barGap",
    "barHeight",
    "bottomMarginAdj",
    "dateFormat",
    "displayMode",
    "excludes",
    "fontSize",
    "gantt",
    "gridLineStartPadding",
    "leftPadding",
    "mirrorActor",
    "numberSectionStyles",
    "rightPadding",
    "sectionFontSize",
    "tickInterval",
    "title",
    "titleTopMargin",
    "todayMarker",
    "topAxis",
    "topPadding",
    "weekday",
    "weekend",
];

/// Optional metadata tags available.
const TASK_TAGS: [&str; 4] = ["done", "active", "crit", "milestone"];

/// udis = user defined items.
/// These are referring to metadata tags outside of the default tags [active, done, crit, and milestone].
/// Possible items are taskd, startdate, & enddate.
///
/// Tasks are by default sequential. A task start date defaults to the end date of the preceding task.
///
/// dateFormat defines the format of the date input of your gantt elements. How these dates are represented in the rendered chart output are defined by axisFormat.
///
/// If a single item is specified, it determines when the task ends. It can either be a specific date/time or a duration.
/// If a duration is specified, it is added to the start date of the task to determine the end date of the task, taking into account any exclusions.
/// If two items are specified, the last item is interpreted as in the previous case.
/// The first item can either specify an explicit start date/time (in the format specified by dateFormat) or reference another task using after <otherTaskID> [[otherTaskID2 [otherTaskID3]]...].
/// In the latter case, the start date of the task will be set according to the latest end date of any referenced task.
/// If three items are specified, the last two will be interpreted as in the previous case.
/// The first item will denote the ID of the task, which can be referenced using the later <taskID> syntax.
///
/// Max lengths for start and end paramaters are used instead of assuming the default dateFormat to account for the possibility
/// that the keywords `after <otherTaskId>` or `until <otherTaskId>` may be used, and these lengths may differ from the default dateFormat config.
fn push_udis_to_task_line(
    mut task_line: String, task_udis: &Vec<&str>, map_item_lenths: HashMap<&str, usize>,
) -> String {
    let max_len_task_id: &usize = map_item_lenths.get("max_len_task_id").unwrap();
    let max_len_start: &usize = map_item_lenths.get("max_len_start").unwrap();
    let max_len_end: &usize = map_item_lenths.get("max_len_end").unwrap();

    match task_udis.len() {
        3 => {
            let mut task_id: String = String::from(task_udis[0]);
            let mut start: String = String::from(task_udis[1]);
            let mut end: String = String::from(task_udis[2]);
            task_id = pad_string(task_id, *max_len_task_id);
            start = pad_string(start, *max_len_start);
            end = pad_string(end, *max_len_end);
            task_line.push_str(&format!("{task_id},  {start},  {end}"));
        }
        2 => {
            let mut start: String = String::from(task_udis[0]);
            let mut end: String = String::from(task_udis[1]);
            start = pad_string(start, *max_len_start);
            end = pad_string(end, *max_len_end);
            task_line.push_str(&format!(
                "{task_id}   {start},  {end}",
                task_id = " ".repeat(*max_len_task_id),
                start = start,
                end = end
            ));
        }
        1 => {
            let mut end: String = String::from(task_udis[0]);
            end = pad_string(end, *max_len_end);
            task_line.push_str(&format!(
                "{task_id}   {start}   {end}",
                task_id = " ".repeat(*max_len_task_id),
                start = " ".repeat(*max_len_start),
                end = end
            ));
        }
        _ => {}
    }

    return task_line;
}

/// A colon (`:`) separates the task title from its metadata.
/// Metadata items are separated by a comma. Valid tags are active, done, crit, and milestone.
/// Tags are optional, but if used, they must be specified first. After processing the tags,
/// the remaining metadata items are interpreted following the defintions under fn push_udis_to_task_line
/// All final else statements add padding for when the tag is not provided. E.g. `&" ".repeat(x)`.
fn push_tags_to_task_line(mut task_line: String, task_tags: &Vec<&str>) -> String {
    // active and done tags both take the first column, as only one should be included at a time.
    if task_tags.contains(&"active") {
        task_line.push_str("active,  ");
    } else if task_tags.contains(&"done") {
        task_line.push_str("done  ,  ");
    } else {
        task_line.push_str(&" ".repeat(9));
    }
    if task_tags.contains(&"crit") {
        task_line.push_str("crit,  ");
    } else {
        task_line.push_str(&" ".repeat(7));
    }
    if task_tags.contains(&"milestone") {
        task_line.push_str("milestone,  ");
    } else {
        task_line.push_str(&" ".repeat(12));
    }
    return task_line;
}

fn get_task_lines(lines: Vec<&str>) -> Vec<&str> {
    let mut task_lines: Vec<&str> = vec![];
    for line in lines
        .iter()
        .cloned()
        .map(str::trim)
        .filter(|&line| {
            !MMD_GANTT_KWS
                .iter()
                .any(|&tag| line.contains(tag))
                && !line.starts_with("%%")
                && line.contains(":")
        })
    {
        task_lines.push(line);
    }
    return task_lines;
}

fn push_task_line(
    line: &str, map_item_lenths: &HashMap<&str, usize>, mut new_lines: Vec<String>,
) -> Vec<String> {
    let task_split: Vec<&str> = line.split(":").map(str::trim).collect();
    let mut task_line: String = String::from(task_split[0]);
    let max_len_title: &usize = map_item_lenths.get("max_len_title").unwrap();

    task_line = pad_string(task_line, *max_len_title);
    task_line.push_str("  : ");

    let meta_items: HashMap<String, Vec<&str>> = split_meta_tags(TASK_TAGS, task_split[1]);
    task_line = push_tags_to_task_line(task_line, meta_items.get("tags").unwrap());
    task_line = push_udis_to_task_line(
        task_line,
        meta_items.get("udis").unwrap(),
        map_item_lenths.clone(),
    );
    // Indent task titles 2 levels.
    // Format commented out tasks so that they align with other tasks, but keep them as comments.
    if task_line.starts_with("%%") {
        let comment_task_split: Vec<&str> = task_line
            .strip_prefix("%%")
            .unwrap()
            .split(":")
            .map(str::trim_start)
            .collect();

        let mut first_half: String = String::from(comment_task_split[0]);
        first_half = pad_string(first_half, *max_len_title);
        let second_half: String = String::from(comment_task_split[1]);

        new_lines.push(format!("%%  {}  : {}", first_half, second_half));
    } else {
        new_lines.push(format!("    {}", task_line));
    }
    return new_lines;
}

fn push_section_line(line: &str, mut new_lines: Vec<String>) -> Vec<String> {
    // Add section titles indented 1 level with 1 leading newline.
    let section_title: &str = line.strip_prefix("section").unwrap().trim();
    new_lines.push(String::from(format!(
        "\n  section {}",
        section_title
    )));
    return new_lines;
}

/// Returns a mapping of the max length for tasks names and metadata tags.
/// Used for padding whitespace chars and aligning columns.
fn get_max_item_lengths<'a>(tags: [&str; 4], lines: Vec<&'a str>) -> HashMap<&'a str, usize> {
    let mut titles: Vec<&str> = vec![];
    let mut ids: Vec<&str> = vec![];
    let mut starts: Vec<&str> = vec![];
    let mut ends: Vec<&str> = vec![];

    for line in get_task_lines(lines.clone()).iter() {
        let metadata: Vec<&str> = line.split(":").map(str::trim).collect();
        titles.push(metadata[0]);

        let meta_items: HashMap<String, Vec<&str>> = split_meta_tags(tags, metadata[1]);
        let task_udis: &Vec<&str> = meta_items.get("udis").unwrap();

        match task_udis.len() {
            3 => {
                ids.push(task_udis[0]);
                starts.push(task_udis[1]);
                ends.push(task_udis[2]);
            }
            2 => {
                starts.push(task_udis[0]);
                ends.push(task_udis[1]);
            }
            1 => {
                ends.push(task_udis[0]);
            }
            _ => {}
        }
    }

    fn _get_count(option: Option<&str>) -> usize {
        return option.unwrap().chars().count();
    }

    let mut map_item_lenths: HashMap<&str, usize> = HashMap::new();
    map_item_lenths.insert(
        "max_len_title",
        _get_count(find_longest_string(&titles)),
    );
    map_item_lenths.insert(
        "max_len_task_id",
        _get_count(find_longest_string(&ids)),
    );
    map_item_lenths.insert(
        "max_len_start",
        _get_count(find_longest_string(&starts)),
    );
    map_item_lenths.insert(
        "max_len_end",
        _get_count(find_longest_string(&ends)),
    );

    return map_item_lenths;
}

fn split_meta_tags<'a>(tags: [&str; 4], metadata: &'a str) -> HashMap<String, Vec<&'a str>> {
    let meta_items: Map<Split<'_, &str>, fn(&str) -> &str> = metadata.split(",").map(str::trim);
    let task_tags: Vec<&str> = meta_items
        .clone()
        .filter(|&x| !x.is_empty())
        .collect();
    let task_udis: Vec<&str> = meta_items
        .clone()
        .filter(|&line| !line.is_empty() && !tags.contains(&line))
        .collect();

    let mut meta: HashMap<String, Vec<&str>> = HashMap::new();
    meta.insert(String::from("tags"), task_tags);
    meta.insert(String::from("udis"), task_udis);
    return meta;
}

fn find_longest_string<'a>(strings: &'a [&'a str]) -> Option<&'a str> {
    if strings.is_empty() {
        return None;
    }

    let mut longest: &str = strings[0];
    for &string in strings.iter() {
        if string.len() > longest.len() {
            longest = string;
        }
    }
    return Some(longest);
}

fn pad_string(mut string: String, max_length: usize) -> String {
    let len_diff: Option<usize> = max_length.checked_sub(string.chars().count());
    if !len_diff.is_none() && len_diff < Some(max_length) {
        let padding: String = " ".repeat(len_diff.unwrap());
        string.push_str(&padding)
    }
    return string;
}

fn create_or_replace_file(file_name: &String, contents: String) -> std::io::Result<()> {
    let mut file: File = File::create(file_name)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

type MapIntoIter<'l> = Map<IntoIter<&'l str>, for<'a> fn(&'a str) -> &'a str>;

fn line_is_empty_before_section(line: &str, idx: usize, c_map_lines: &MapIntoIter) -> bool {
    let next_line: Vec<&str> = Some(c_map_lines.clone().collect::<Vec<_>>()).unwrap_or_default();
    if (idx + 1).lt(&next_line.len()) {
        return !next_line.is_empty()
            && line.is_empty()
            && next_line[idx.add(1)].contains("section");
    } else {
        return false;
    }
}

fn line_is_keyword(line: &str) -> bool {
    return MMD_GANTT_KWS
        .iter()
        .any(|&tag| line.starts_with(tag));
}

fn line_is_task(line: &str) -> bool {
    return line.contains(":");
}

fn line_is_commented_section(line: &str) -> bool {
    line.starts_with("%%") && line.contains("section")
}

fn line_is_comment(line: &str) -> bool {
    line.starts_with("%%") && !line.contains(":")
}

fn line_is_title(line: &str) -> bool {
    return line.starts_with("gantt");
}

fn generate_new_lines(lines: Vec<&str>) -> Vec<String> {
    let mut new_lines: Vec<String> = vec![];
    let map_item_lenths: HashMap<&str, usize> = get_max_item_lengths(TASK_TAGS, lines.clone());
    let map_lines: MapIntoIter = lines.into_iter().map(str::trim);
    let c_map_lines: MapIntoIter = map_lines.clone();

    for (_idx, line) in map_lines.enumerate() {
        if line_is_title(line) {
            new_lines.push(String::from(line)); // Add gantt title at top with no indent/any commented, non-task lines.
        } else if line_is_keyword(line) {
            new_lines.push(format!("  {}", line)); // Add any remaining keyword lines idented 1 level.
        } else if line_is_commented_section(line) {
            new_lines.push(String::new());
            new_lines.push(String::from(line));
        } else if line_is_comment(line) {
            new_lines.push(String::from(line));
        } else if line_is_empty_before_section(line, _idx, &c_map_lines) {
            continue; // Ignore all empty lines if before section.
        } else if line.contains("section") {
            new_lines = push_section_line(line, new_lines);
        } else if line_is_task(line) {
            new_lines = push_task_line(line, &map_item_lenths, new_lines);
        } else if !line.is_empty() {
            new_lines.push(String::from(line)); // Add remaining lines as is.
        }
    }
    new_lines.push(String::new()); // Add 1 empty line at end of file.
    return new_lines;
}

/// First arg = file to read.
/// Second arg = file to write to.
/// If only the first arg is provided, then file is edited in-place.
fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let file_name: &String = &args[1];
    let file_text: String =
        std::fs::read_to_string(file_name).expect(&format!("Could not read file {}", file_name));

    match &args.len() {
        3 => {
            let destination: &String = &args[2];
            create_or_replace_file(
                destination,
                generate_new_lines(file_text.lines().collect()).join("\n"),
            )
        }
        _ => create_or_replace_file(
            file_name,
            generate_new_lines(file_text.lines().collect()).join("\n"),
        ),
    }
}
