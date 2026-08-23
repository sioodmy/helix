use std::path::PathBuf;
use crate::compositor::{Component, Context, EventResult, Event};
use helix_view::editor::FileExplorerStyle;
use helix_view::graphics::Rect;
use tui::buffer::Buffer;

pub mod snacks;
pub mod mini;

pub enum Explorer {
    Snacks(snacks::SnacksExplorer),
    Mini(mini::MiniExplorer),
}

impl Component for Explorer {
    fn handle_event(&mut self, event: &Event, cx: &mut Context) -> EventResult {
        match self {
            Explorer::Snacks(e) => e.handle_event(event, cx),
            Explorer::Mini(e) => e.handle_event(event, cx),
        }
    }

    fn render(&mut self, area: Rect, surface: &mut Buffer, cx: &mut Context) {
        match self {
            Explorer::Snacks(e) => e.render(area, surface, cx),
            Explorer::Mini(e) => e.render(area, surface, cx),
        }
    }
}

pub fn file_explorer(root: PathBuf, editor: &helix_view::Editor) -> Result<Explorer, std::io::Error> {
    match editor.config().file_explorer.style {
        FileExplorerStyle::Snacks => Ok(Explorer::Snacks(snacks::SnacksExplorer::new(root))),
        FileExplorerStyle::Mini => Ok(Explorer::Mini(mini::MiniExplorer::new(root))),
    }
}
