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
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Map;
use std::str::Split;

/// Required/optional keywords that may appear at the top of a gantt .mmd file or elsewhere.
/// Note: this list is not inclusive. This script doesn't currently account for YAML frontmatter. https://mermaid.js.org/config/configuration.html#frontmatter-config
const MMD_GANTT_KWS: [&str; 22] = [
    "gantt",
    "title",
    "excludes",
    "dateFormat",
    "todayMarker",
    "titleTopMargin",
    "barHeight",
    "barGap",
    "topPadding",
    "rightPadding",
    "leftPadding",
    "gridLineStartPadding",
    "fontSize",
    "sectionFontSize",
    "numberSectionStyles",
    "axisFormat",
    "tickInterval",
    "topAxis",
    "displayMode",
    "weekday",
    "mirrorActor",
    "bottomMarginAdj",
];

/// Optional metadata tags available to mmd gantt charts.
const TASK_TAGS: [&str; 4] = ["done", "active", "crit", "milestone"];

/// A colon, :, separates the task title from its metadata.
/// Metadata items are separated by a comma, ,. Valid tags are active, done, crit, and milestone.
/// Tags are optional, but if used, they must be specified first. After processing the tags,
/// the remaining metadata items are interpreted following the defintions under fn push_udis_to_task_line
/// All final else statements add padding for when the tag is not provided. E.g. `&" ".repeat(x)`.
fn push_tags_to_task_line(mut task_line: String, task_tags: Vec<&str>) -> String {
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
    mut task_line: String,
    task_udis: &Vec<&str>,
    map_item_lenths: HashMap<&str, usize>,
) -> String {
    let max_len_task_id: &usize = map_item_lenths.get("max_len_task_id").unwrap();
    let max_len_start: &usize = map_item_lenths.get("max_len_start").unwrap();
    let max_len_end: &usize = map_item_lenths.get("max_len_end").unwrap();

    let len_udis: usize = task_udis.len();
    if len_udis == 3 {
        let mut task_id: String = String::from(task_udis[0]);
        let mut start: String = String::from(task_udis[1]);
        let mut end: String = String::from(task_udis[2]);
        task_id = pad_string(task_id, *max_len_task_id);
        start = pad_string(start, *max_len_start);
        end = pad_string(end, *max_len_end);
        task_line.push_str(&format!("{},  {},  {}", task_id, start, end));
    } else if len_udis == 2 {
        let task_id: String = " ".repeat(*max_len_task_id);
        let mut start: String = String::from(task_udis[0]);
        let mut end: String = String::from(task_udis[1]);
        start = pad_string(start, *max_len_start);
        end = pad_string(end, *max_len_end);
        task_line.push_str(&format!("{}   {},  {}", task_id, start, end));
    } else if len_udis == 1 {
        let task_id: String = " ".repeat(*max_len_task_id);
        let start: String = " ".repeat(*max_len_start);
        let mut end: String = String::from(task_udis[0]);
        end = pad_string(end, *max_len_end);
        task_line.push_str(&format!("{}   {}   {}", task_id, start, end));
    }
    return task_line;
}

fn get_task_lines(lines: Vec<&str>) -> Vec<&str> {
    let mut task_lines: Vec<&str> = vec![];

    for line in lines
        .iter()
        .cloned()
        .map(str::trim)
        .filter(|&line| !MMD_GANTT_KWS.iter().any(|&tag| line.contains(tag)))
        .filter(|&item| !item.starts_with("%%"))
    {
        if line.contains(":") {
            task_lines.push(line);
        }
    }
    return task_lines;
}

/// Returns a mapping of the max length for tasks names and metadata tags.
/// Used for padding whitespace chars and aligning columns.
fn get_max_item_lengths<'a>(tags: [&str; 4], lines: Vec<&'a str>) -> HashMap<&'a str, usize> {
    let mut task_titles: Vec<&str> = vec![];
    let mut task_ids: Vec<&str> = vec![];
    let mut task_starts: Vec<&str> = vec![];
    let mut task_ends: Vec<&str> = vec![];

    for line in get_task_lines(lines.clone()).iter() {
        let metadata: Vec<&str> = line.split(":").map(str::trim).collect();
        task_titles.push(metadata[0]);

        let meta_items: HashMap<String, Vec<&str>> = split_meta_vars(tags, metadata);
        let task_udis: &Vec<&str> = meta_items.get("defs").unwrap();

        let len_udis: usize = task_udis.len();
        if len_udis == 3 {
            task_ids.push(task_udis[0]);
            task_starts.push(task_udis[1]);
            task_ends.push(task_udis[2]);
        } else if len_udis == 2 {
            task_starts.push(task_udis[0]);
            task_ends.push(task_udis[1]);
        } else if len_udis == 1 {
            task_ends.push(task_udis[0]);
        }
    }

    let mut map_item_lenths: HashMap<&str, usize> = HashMap::new();
    let longest_title: Option<&str> = find_longest_string(&task_titles);
    let longest_id: Option<&str> = find_longest_string(&task_ids);
    let longest_start: Option<&str> = find_longest_string(&task_starts);
    let longest_end: Option<&str> = find_longest_string(&task_ends);

    fn _get_count(option: Option<&str>) -> usize {
        return option.unwrap().chars().count();
    }

    map_item_lenths.insert("max_len_title", _get_count(longest_title));
    map_item_lenths.insert("max_len_task_id", _get_count(longest_id));
    map_item_lenths.insert("max_len_start", _get_count(longest_start));
    map_item_lenths.insert("max_len_end", _get_count(longest_end));

    return map_item_lenths;
}

fn split_meta_vars<'a>(tags: [&str; 4], metadata: Vec<&'a str>) -> HashMap<String, Vec<&'a str>> {
    let meta_items: Map<Split<'_, &str>, fn(&str) -> &str> = metadata[1].split(",").map(str::trim);
    let task_tags: Vec<&str> = meta_items.clone().filter(|&x| !x.is_empty()).collect();
    let task_udis: Vec<&str> = meta_items
        .clone()
        .filter(|&x| !x.is_empty())
        .filter(|&item| !tags.contains(&item))
        .collect();

    let mut meta: HashMap<String, Vec<&str>> = HashMap::new();
    meta.insert(String::from("tags"), task_tags);
    meta.insert(String::from("defs"), task_udis);
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
    return Ok(());
}

/// Pass filename as first arg. File is edited in-place.
fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let file_name: &String = &args[1];

    let file_text: String =
        std::fs::read_to_string(file_name).expect(&format!("Could not read file {}", file_name));

    let lines: Vec<&str> = file_text.lines().collect();
    let mut new_lines: Vec<String> = vec![];
    let map_item_lenths: HashMap<&str, usize> = get_max_item_lengths(TASK_TAGS, lines.clone());

    for line in lines.into_iter().map(str::trim) {
        if line.starts_with("gantt") || line.starts_with("%%") {
            new_lines.push(String::from(line)); // Add gantt title at top of file with no indent.
        } else if MMD_GANTT_KWS.iter().any(|&tag| line.starts_with(tag)) {
            new_lines.push(format!("  {}", line)); // Add any remaining args idented 1 level.
        } else if line.contains("section") {
            if new_lines.last().unwrap() != "" {
                new_lines.push(String::new());
            }
            new_lines.push(String::from(format!("  {}", line))); // Indent sections 1 level.
        } else if line.contains(":") {
            let task_split: Vec<&str> = line.split(":").map(str::trim).collect();
            let mut task_line: String = String::from(task_split[0]);
            let max_len_title: &usize = map_item_lenths.get("max_len_title").unwrap();

            task_line = pad_string(task_line, *max_len_title);
            task_line.push_str("  : ");

            let meta_items: HashMap<String, Vec<&str>> = split_meta_vars(TASK_TAGS, task_split);
            let task_tags: &Vec<&str> = meta_items.get("tags").unwrap();
            let task_udis: &Vec<&str> = meta_items.get("defs").unwrap();

            task_line = push_tags_to_task_line(task_line, task_tags.to_vec());
            task_line = push_udis_to_task_line(task_line, task_udis, map_item_lenths.clone());
            new_lines.push(format!("    {}", task_line)); // Indent task titles 2 levels.
        } else {
            new_lines.push(String::from(line));
        }
    }
    new_lines.push(String::new()); // Add 1 empty line at end of file.

    return create_or_replace_file(file_name, new_lines.join("\n"));
}