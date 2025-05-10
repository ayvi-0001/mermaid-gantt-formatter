mod gantt;

use std::fmt::Display;

pub use gantt::GanttChart;

pub trait MermaidDiagramFormatter: Display {
    fn format_diagram(&mut self, text: &str) -> Result<String, &str>;
}
