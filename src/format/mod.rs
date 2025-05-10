mod gantt;

pub use gantt::GanttChart;

use std::fmt::Display;

pub trait MermaidDiagramFormatter: Display {
    fn format_diagram(&mut self, text: &str) -> Result<String, &str>;
}
