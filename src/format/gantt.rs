use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::ops::{Div, Mul};
use std::rc::Rc;
use std::{fmt, iter};

use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Default)]
pub struct GanttChart {
    attributes: Vec<GanttAttr>,
    sections: Vec<Section>,
    comment_map: HashMap<String, VecDeque<Comment>>,
    unmapped_comments: VecDeque<Comment>,
}

impl GanttChart {
    pub fn new() -> GanttChart {
        let mut comment_map: HashMap<String, VecDeque<Comment>> = HashMap::new();
        comment_map.insert("comments".to_string(), VecDeque::new());
        GanttChart { comment_map, ..Self::default() }
    }

    pub fn parse_text(&mut self, text: &str) -> Result<(), &str> {
        self.parse_lines(
            text.lines()
                .filter(|l| !l.is_empty())
                .map(ParsedLine::new)
                .enumerate()
                .peekable(),
        )
    }

    fn flush_comments(&mut self) -> VecDeque<Comment> {
        let mut comments: VecDeque<Comment> = VecDeque::new();
        while let Some(comment) = self.get_comments().pop_front() {
            comments.push_back(comment);
        }
        comments
    }

    fn get_comments(&mut self) -> &mut VecDeque<Comment> {
        if !self.comment_map.contains_key("comments") {
            let default: VecDeque<Comment> = VecDeque::new();
            self.comment_map
                .insert("comments".to_string(), default);
        }

        self.comment_map
            .get_mut("comments")
            .expect("comment_map should contain the key `comments`.")
    }

    fn push_comment(&mut self, comment: Comment) {
        self.get_comments().push_front(comment)
    }

    fn push_attr(&mut self, attr: GanttAttr) {
        self.attributes.push(attr)
    }

    fn push_section(&mut self, section: Section) {
        self.sections.push(section)
    }

    fn push_task(&mut self, task: Task) {
        if let Some(section) = self.get_current_section() {
            section.push_task(task);
        } else {
            let mut top_section = Section::hidden();
            top_section.push_task(task);
            self.push_section(top_section);
        }
    }

    fn get_current_section(&mut self) -> Option<&mut Section> {
        self.sections.last_mut()
    }

    fn get_latest_task(&mut self) -> Option<&mut Rc<RefCell<Task>>> {
        let current_section = self.get_current_section();
        if let Some(section) = current_section {
            section.tasks.last_mut()
        } else {
            None
        }
    }

    fn iterate_tasks(&self) -> impl Iterator<Item = &Rc<RefCell<Task>>> {
        self.sections.iter().flat_map(|s| &s.tasks)
    }

    /// Get the length of the longest string for task attributes.
    fn get_task_lengths(&self) -> TaskPadding {
        let mut longest_desc: usize = 0;
        let mut longest_id: usize = 0;
        let mut longest_start_date: usize = 0;

        for task in self.iterate_tasks() {
            let task_ref = task.borrow();

            if task_ref.description.len() > longest_desc {
                longest_desc = task_ref.description.len()
            }
            if task_ref.id.len() > longest_id {
                longest_id = task_ref.id.len()
            }
            if task_ref.start_date.len() > longest_start_date {
                longest_start_date = task_ref.start_date.len()
            }
        }

        TaskPadding {
            len_desc: longest_desc,
            len_id: longest_id,
            len_start_date: longest_start_date,
        }
    }

    fn map_section_comments(&mut self, key: String) {
        let mut latest_comments = self.flush_comments();
        let mut section_comments = self.comment_map.remove(&key).unwrap_or_default();

        while let Some(comment) = latest_comments.pop_front() {
            section_comments.push_back(comment)
        }

        self.comment_map.insert(key, section_comments);
    }

    fn push_section_comments(&mut self) {
        for section in self.sections.iter_mut() {
            if let Some(mut section_comments) = self.comment_map.remove(&section.name) {
                while let Some(comment) = section_comments.pop_back() {
                    section.push_comment(comment);
                }
            }
        }
    }

    fn push_comments_to_latest_task(&mut self) {
        let mut comments = self.flush_comments();
        let task_ref = self.get_latest_task();
        if let Some(task) = task_ref {
            while let Some(comment) = comments.pop_back() {
                task.borrow_mut().comments.push(comment);
            }
        }
    }

    fn parse_lines<I>(&mut self, mut input_lines: iter::Peekable<I>) -> Result<(), &str>
    where
        I: Iterator<Item = (usize, ParsedLine)>,
    {
        while let Some((idx, current_line)) = input_lines.next() {
            let next_line = input_lines.peek();

            if idx.eq(&0) && !current_line.text.trim().eq("gantt") {
                return Err("Invalid diagram");
            };

            if current_line.is_attr(&self.sections, next_line) {
                self.push_attr(GanttAttr::new(
                    &current_line.text,
                    current_line.is_comment,
                ));
                continue;
            } else if current_line.is_section() {
                self.push_section(Section::new(
                    String::from(&current_line.text),
                    current_line.is_comment,
                ));
            } else if current_line.is_task() {
                self.push_task(Task::new(
                    &current_line.text,
                    current_line.is_comment,
                ));
            } else if current_line.is_comment {
                // Check for comment line must be after first checking section/task.
                self.push_comment(Comment { text: String::from(&current_line.text) });
                continue;
            }

            if !self.get_comments().is_empty() {
                if current_line.is_section() {
                    self.map_section_comments(current_line.text.to_string());
                } else if next_line.is_some_and(|(_, l)| l.is_section()) {
                    self.map_section_comments(next_line.unwrap().1.text.to_string());
                } else if current_line.is_task() && next_line.is_some_and(|(_, l)| l.is_comment)
                    || self.get_latest_task().is_some()
                {
                    self.push_comments_to_latest_task();
                }
            }
        }

        self.push_section_comments();

        // Save any remaining comments at end of file.
        if let Some(mut remaining_comments) = self.comment_map.remove("comments") {
            while let Some(comment) = remaining_comments.pop_front() {
                self.unmapped_comments.push_front(comment);
            }
        };

        Ok(())
    }
}

impl fmt::Display for GanttChart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut display = "gantt\n".to_string();

        for attr in &self.attributes {
            display.push_str(&attr.format());
            display.push('\n');
        }

        display.push('\n');

        let task_lengths = self.get_task_lengths();

        for section in &self.sections {
            display.push_str(&section.format(&task_lengths));
            display.push('\n');
        }

        if !self.unmapped_comments.is_empty() {
            for comment in &self.unmapped_comments {
                display.push_str(&comment.format(None));
            }
        }

        display = display.trim_end_matches("\n").to_string();
        display.push('\n');

        write!(f, "{}", display)
    }
}

fn consume<'a, T>(i: &mut impl Iterator<Item = &'a mut T>, v: &T) -> Option<&'a mut T>
where
    T: ?Sized + PartialOrd<T> + PartialEq<T>,
{
    i.find(|x| *x == v)
}

fn pad_string(string: &str, max_length: &usize) -> String {
    let mut padding = String::default();
    if let Some(c) = max_length.checked_sub(string.chars().count()) {
        padding.push_str(" ".repeat(c).as_str())
    }
    padding
}

#[derive(Default)]
struct ParsedLine {
    text: String,
    is_comment: bool,
}

impl ParsedLine {
    fn new(line: &str) -> ParsedLine {
        let mut parsed_line = Self::default();

        let trimmed_text = String::from(line).trim_ascii().to_string();

        if let Some(stripped_text) = trimmed_text.strip_prefix(Comment::TOKEN) {
            parsed_line.is_comment = true;
            parsed_line.text = String::from(stripped_text)
                .trim_ascii()
                .to_string();
        } else {
            parsed_line.text = String::from(&trimmed_text).to_string();
        }

        parsed_line
    }

    fn is_attr(&self, sections: &[Section], next: Option<&(usize, ParsedLine)>) -> bool {
        let starts_with_attr = OptionalAttr::iter().any(|a| {
            self.text
                .starts_with(<&OptionalAttr as Into<&str>>::into(&a))
        });
        let next_line_is_section = next.is_some_and(|(_, l)| l.is_section());

        if (next_line_is_section || self.is_section()) && !starts_with_attr {
            false
        // Chart attributes must be before sections.
        } else if sections.is_empty() {
            self.is_comment || starts_with_attr
        } else {
            false
        }
    }

    fn is_task(&self) -> bool {
        self.text.contains(":")
    }

    fn is_section(&self) -> bool {
        self.text.starts_with("section")
    }
}

enum Indent {
    Half,
    Full,
    Ratio(f32),
}

impl Indent {
    const SPACES: i32 = 2;
}

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Half => {
                write!(f, "{}", " ".repeat(Self::SPACES.div(2) as usize))
            }
            Self::Full => {
                write!(f, "{}", " ".repeat(Self::SPACES as usize))
            }
            Self::Ratio(ratio) => {
                let base = Mul::mul(Self::SPACES as f32, ratio);
                write!(f, "{}", " ".repeat(base as usize))
            }
        }
    }
}

/// Required/optional keywords that may appear at the top of a mermaid gantt file.
/// Note: this is not an exhaustive list. This script doesn't currently account for YAML frontmatter.
/// https://mermaid.js.org/config/configuration.html#frontmatter-config
#[allow(non_camel_case_types)]
#[derive(EnumIter, IntoStaticStr)]
enum OptionalAttr {
    axisFormat,
    barGap,
    barHeight,
    bottomMarginAdj,
    dateFormat,
    displayMode,
    excludes,
    fontSize,
    gridLineStartPadding,
    leftPadding,
    mirrorActor,
    numberSectionStyles,
    rightPadding,
    sectionFontSize,
    tickInterval,
    title,
    titleTopMargin,
    todayMarker,
    topAxis,
    topPadding,
    weekday,
    weekend,
}

#[derive(Debug)]
struct GanttAttr {
    attr: String,
    text: String,
    is_comment: bool,
}

impl GanttAttr {
    fn new(line: &str, is_comment: bool) -> GanttAttr {
        let mut text = line.to_string();

        let attr = if let Some(a) = OptionalAttr::iter()
            .map(|a| <&OptionalAttr as Into<&str>>::into(&a))
            .filter(|a| line.starts_with(a))
            .map(|a| format!("{} ", a))
            .collect::<Vec<String>>()
            .first()
        {
            text = text.strip_prefix(a).unwrap().trim().to_string();
            a.to_string()
        } else {
            String::default()
        };

        GanttAttr { attr, text, is_comment }
    }

    fn format(&self) -> String {
        if self.is_comment {
            format!(
                "{}{} {}{}",
                Indent::Full,
                Comment::TOKEN,
                self.attr,
                self.text
            )
        } else {
            format!("{}{}{}", Indent::Full, self.attr, self.text)
        }
    }
}

#[derive(Debug)]
struct Comment {
    text: String,
}

impl Comment {
    const TOKEN: &str = "%%";

    fn format(&self, whitespaces: Option<usize>) -> String {
        format!(
            "{}{}{}\n",
            Self::TOKEN,
            " ".repeat(whitespaces.unwrap_or(1)),
            self.text
        )
    }
}

/// Struct to store max lengths of task attributes.
/// Don't need length of end date as it'll always be the right most attr,
/// and won't determine the column length for any other task.
#[derive(Debug, Default)]
struct TaskPadding {
    len_desc: usize,
    len_id: usize,
    len_start_date: usize,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Default, IntoStaticStr)]
enum TaskStatus {
    active,
    done,
    #[default]
    empty,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = <&TaskStatus as Into<&str>>::into(self);
        match self {
            Self::done => {
                write!(f, "{}  , ", display)
            }
            Self::active => {
                write!(f, "{}, ", display)
            }
            Self::empty => {
                write!(f, "{}", " ".repeat(8))
            }
        }
    }
}

/// Optional task metadata tags available.
#[allow(non_camel_case_types)]
#[derive(EnumIter, IntoStaticStr)]
enum TaskTags {
    done,
    active,
    crit,
    milestone,
}

#[derive(Debug, Default)]
struct Task {
    id: String,
    description: String,
    status: TaskStatus,
    crit: bool,
    milestone: bool,
    start_date: String,
    end_date: String,
    is_comment: bool,
    comments: Vec<Comment>,
}

impl Task {
    fn new(text: &str, is_comment: bool) -> Task {
        let mut task = Task { is_comment, ..Self::default() };

        let text = String::from(text);

        // A colon (`:`) separates the task title from its metadata.
        let task_split = text
            .split_once(":")
            .expect("Check for `:` happens in parsed line.");

        task.description
            .push_str(String::from(task_split.0).trim());

        // Metadata items are separated by a comma. Valid tags are active, done, crit, and milestone.
        // Tags are optional, but if used, they must be specified first before any ids and dates.
        let mut task_meta = task_split
            .1
            .split(",")
            .map(|s| {
                // inner split whitespace is to trim extra characters
                // in dates using keywords `after` or `until`
                s.split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ")
            })
            .collect::<VecDeque<String>>();

        if consume(&mut task_meta.iter_mut(), &"active".to_string()).is_some() {
            task.status = TaskStatus::active;
        } else if consume(&mut task_meta.iter_mut(), &"done".to_string()).is_some() {
            task.status = TaskStatus::done;
        };

        if consume(&mut task_meta.iter_mut(), &"crit".to_string()).is_some() {
            task.crit = !task.crit
        };
        if consume(
            &mut task_meta.iter_mut(),
            &"milestone".to_string(),
        )
        .is_some()
        {
            task.milestone = !task.milestone
        };

        task_meta.retain(|item| {
            !TaskTags::iter().any(|tag| item.eq(<&TaskTags as Into<&str>>::into(&tag)))
        });

        // After processing the tags, the remaining metadata items are interpreted as follows:
        while let Some(p) = task_meta.pop_front() {
            match task_meta.len() + 1 {
                // If a single item is specified, it determines when the task ends.
                // It can either be a specific date/time or a duration.
                // If a duration is specified, it is added to the start date of the task to determine
                // the end date of the task, taking into account any exclusions.
                1_usize => task.end_date.push_str(&p),
                // If two items are specified, the last item is interpreted as in the previous case.
                // The first item can either specify an explicit start date/time (in the format specified by dateFormat)
                // or reference another task using after <otherTaskID> [[otherTaskID2 [otherTaskID3]]...].
                // In the latter case, the start date of the task will be set according to the latest end date of any referenced task.
                2_usize => task.start_date.push_str(&p),
                // If three items are specified, the last two will be interpreted as in the previous case.
                // The first item will denote the ID of the task, which can be referenced using the later <taskID> syntax.
                3_usize => task.id.push_str(&p),
                4_usize.. => panic!("Too many items for: {}", &task.description),
                _ => break,
            }
        }

        task
    }

    fn format(&self, task_lengths: &TaskPadding, indent_ratio: Option<f32>) -> String {
        let mut display = String::default();

        if !self.comments.is_empty() {
            for comment in &self.comments {
                display.push_str(&comment.format(Some(2)));
            }
        }

        let ratio = indent_ratio.unwrap_or(1.0);
        let padding =
            &(task_lengths.len_desc + ((if ratio != 1.0 { ratio.mul(4.0) } else { 0.0 }) as usize));

        if !self.is_comment {
            display.push_str(&format!(
                "{}{}{} : ",
                Indent::Ratio(ratio.mul(2.0)),
                self.description,
                pad_string(&self.description, padding),
            ));
        } else {
            display.push_str(&format!(
                "{}{}{}{} : ",
                Comment::TOKEN,
                Indent::Ratio(ratio),
                self.description,
                pad_string(&self.description, padding),
            ));
        }

        display.push_str(&self.status.to_string());

        if self.crit {
            display.push_str("crit, ");
        } else {
            display.push_str(&" ".repeat(6));
        }
        if self.milestone {
            display.push_str("milestone, ");
        } else {
            display.push_str(&" ".repeat(11));
        }
        if !self.id.is_empty() {
            display.push_str(&format!(
                "{}{}, ",
                self.id,
                pad_string(&self.id, &task_lengths.len_id)
            ));
        } else {
            display.push_str(&format!(
                "{}  ",
                pad_string(&self.id, &task_lengths.len_id)
            ));
        }
        if !self.start_date.is_empty() {
            display.push_str(&format!(
                "{}{}, ",
                self.start_date,
                pad_string(&self.start_date, &task_lengths.len_start_date)
            ));
        }
        if !self.end_date.is_empty() {
            display.push_str(&self.end_date.to_string());
        }

        display = display.trim_end().to_string();

        if display.ends_with(":") {
            // Even without any attributes/dates, need at least 1 empty space after the colon.
            display.push(' ');
            display
        } else {
            display
        }
    }
}

#[derive(Debug, Default)]
struct Section {
    name: String,
    tasks: Vec<Rc<RefCell<Task>>>,
    is_comment: bool,
    is_hidden: bool,
    comments: Vec<Comment>,
}

impl Section {
    fn new(name: String, is_comment: bool) -> Section {
        Section { name, is_comment, ..Self::default() }
    }

    /// This constructor is used for when a diagram has tasks at the top of the file that are not under a section.
    /// When formatted, this section will not display a "section: {title}" line, and tasks under it will be indented once,
    /// instead of indented twice like tasks under normal sections are.
    fn hidden() -> Section {
        Section { name: "".to_owned(), is_hidden: true, ..Self::default() }
    }

    fn push_task(&mut self, task: Task) {
        self.tasks.push(Rc::new(RefCell::new(task)));
    }

    fn push_comment(&mut self, comment: Comment) {
        self.comments.push(comment);
    }

    fn format(&self, task_lengths: &TaskPadding) -> String {
        let mut display = String::default();

        if !self.comments.is_empty() {
            for comment in &self.comments {
                display.push_str(&format!(
                    "{}{}{}\n",
                    Comment::TOKEN,
                    Indent::Half,
                    comment.text
                ));
            }
        };

        if !self.is_hidden {
            if !self.is_comment {
                display.push_str(&format!("{}{}\n", Indent::Full, self.name));
            } else {
                display.push_str(&format!(
                    "{}{}{}\n",
                    Comment::TOKEN,
                    Indent::Half,
                    self.name
                ));
            }
        };

        let indent_ratio = if self.is_hidden { Some(0.5) } else { None };
        for task in &self.tasks {
            let task_ref = task.borrow();
            display.push_str(&task_ref.format(task_lengths, indent_ratio));
            display.push('\n');
        }

        display
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn task_comments() {
        let input_text = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD
              section One
               %% Comment for the first task.
                A task       : done  , crit, id-01, 2025-01-01                 , 30d
                 %%    Comment for a following task.
                %%     Multiple comments for the same task.
                A milestone  : done  ,       milestone,              id-02, after id-01, 1d
                %% A commented task :                               30d
                 %%Comment for the last task.
                Another task : active,                              after id-02                , 20d
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD

              section One
            %%  Comment for the first task.
                A task           : done  , crit,            id-01, 2025-01-01 , 30d
            %%  Comment for a following task.
            %%  Multiple comments for the same task.
                A milestone      : done  ,       milestone, id-02, after id-01, 1d
            %%  A commented task :                                 30d
            %%  Comment for the last task.
                Another task     : active,                         after id-02, 20d
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn task_attributes() {
        let input_text = indoc! {"\
            gantt
            dateFormat YYYY-MM-DD
            section One
            %% this task should have at least 1 white space after the colon
            a task with no attributes:
            %% this task should not end in a comma, and have no trailing whitespace
            a task with 1 attribute : done , ,,

            a task with 'section' in the name':
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD

              section One
            %%  this task should have at least 1 white space after the colon
                a task with no attributes          : 
            %%  this task should not end in a comma, and have no trailing whitespace
                a task with 1 attribute            : done  ,
                a task with 'section' in the name' : 
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn section_with_no_tasks() {
        let input_text = indoc! {"\
            gantt
            dateFormat YYYY-MM-DD
            section One
            A task:done
            section NoTasks
            section Last
            final task: done
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD

              section One
                A task     : done  ,

              section NoTasks

              section Last
                final task : done  ,
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    /// No characters other than whitespace and newlines should be stripped.
    #[test]
    fn all_characters() {
        let input_text = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD
              title Test Gantt Diagram

              section One
            %%  A comment
                A task       : done  , crit, id-01, 2025-01-01                 , 30d
                A milestone  : done  ,       milestone,              id-02, after id-01, 1d
                Another task : active,                              after id-02                , 20d

              section Next
                Next task :                                      2025-05-04                 , 12d
                A task with only an end date : active,                                     24d
                %% Another comment
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");

        assert_eq!(
            gantt_chart
                .to_string()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("")
                .chars()
                .count(),
            input_text
                .to_string()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("")
                .chars()
                .count()
        )
    }

    #[test]
    fn comments_end_of_file() {
        let input_text = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD

              section One
                A task       : done  , crit, id-01, 2025-01-01                 , 30d
                %% Comment at end of file #1
              %% Comment at end of file #2


                 %% Comment at end of file #3
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD

              section One
                A task : done  , crit,            id-01, 2025-01-01, 30d

            %% Comment at end of file #1
            %% Comment at end of file #2
            %% Comment at end of file #3
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn section_with_no_name() {
        let input_text = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD
              section
                A task       : done,  30d
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD

              section
                A task : done  ,                    30d
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn section_comments() {
        let input_text = indoc! {"\
            gantt
            %% A comment for an attribute.
            dateFormat YYYY-MM-DD
            %% A comment for a commented section.
            %% section Commented
            %% a commented task in a commented section :

            %% A comment for the next section.
            section Next
            a task :
            %% A comment for the last section.
            %% Another comment for the last section.
            section Last
            another task :
            "
        };
        let expected_output = indoc! {"\
            gantt
              %% A comment for an attribute.
              dateFormat YYYY-MM-DD

            %% A comment for a commented section.
            %% section Commented
            %%  a commented task in a commented section : 

            %% A comment for the next section.
              section Next
                a task                                  : 

            %% A comment for the last section.
            %% Another comment for the last section.
              section Last
                another task                            : 
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn attribute_comments() {
        let input_text = indoc! {"\
            gantt
            %% A comment for an attribute.
            dateFormat YYYY-MM-DD
            %% Another comment for an attribute.
            excludes weekends
            weekend friday
            "
        };
        let expected_output = indoc! {"\
            gantt
              %% A comment for an attribute.
              dateFormat YYYY-MM-DD
              %% Another comment for an attribute.
              excludes weekends
              weekend friday
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn leading_and_trailing_newlines() {
        let input_text: &str = indoc! {"\n\n\n\n
                 gantt      
            dateFormat YYYY-MM-DD
            section One
            a task :
            \n\n\n\n\n
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD

              section One
                a task : 
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn invalid_diagram() {
        let input_text: &str = "some random text";
        let mut gantt_chart = GanttChart::new();
        assert!(gantt_chart.parse_text(input_text).is_err());
    }

    /// Tasks without a section at the top should line up with any additional tasks in following sections.
    #[test]
    fn top_tasks_no_section() {
        let input_text = indoc! {"\
            gantt
            dateFormat YYYY-MM-DD

            a task under no section : 

            section 1
            a task under section 1 :
            section 2
            %% a commented task under section 2 :

            section 3
            a task under section 3 with the longest title :
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat YYYY-MM-DD

              a task under no section                         : 

              section 1
                a task under section 1                        : 

              section 2
            %%  a commented task under section 2              : 

              section 3
                a task under section 3 with the longest title : 
            "
        };
        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn tasks_with_time() {
        let input_text = indoc! {"\
            gantt
                dateFormat HH:mm
                axisFormat %H:%M
                section Main
                Initial milestone : milestone, m1, 17:49, 2m
                Task A : 10m
                Task B : 5m
                Final milestone : milestone, m2, 18:08, 4m
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat HH:mm
              axisFormat %H:%M

              section Main
                Initial milestone :               milestone, m1, 17:49, 2m
                Task A            :                              10m
                Task B            :                              5m
                Final milestone   :               milestone, m2, 18:08, 4m
            "
        };
        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }

    #[test]
    fn top_section_and_tasks_commented() {
        let input_text = indoc! {"\
            gantt
              dateFormat YY-MM-DD
              axisFormat %y-%W
              tickInterval 1week
            %% section One
            %%  a task : done,  t1, 25-10-25        ,  1d
            section Two
            another task: active, after t1, 365d
            "
        };
        let expected_output = indoc! {"\
            gantt
              dateFormat YY-MM-DD
              axisFormat %y-%W
              tickInterval 1week

            %% section One
            %%  a task       : done  ,                  t1, 25-10-25, 1d

              section Two
                another task : active,                      after t1, 365d
            "
        };

        let mut gantt_chart = GanttChart::new();
        gantt_chart
            .parse_text(input_text)
            .expect("input_text should be a valid diagram.");
        assert_eq!(gantt_chart.to_string(), expected_output)
    }
}
