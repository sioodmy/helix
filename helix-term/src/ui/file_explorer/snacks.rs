use std::path::PathBuf;
use std::time::SystemTime;
use crate::ui::prompt::{Prompt, PromptEvent};
use crate::compositor::{Component, Context, EventResult, Event};
use helix_view::input::KeyEvent;
use helix_view::keyboard::KeyCode;
use tui::buffer::Buffer as Surface;
use helix_view::graphics::Rect;
use crate::ui::glyph::file_icon;

pub struct SnacksExplorer {
    root: PathBuf,
    entries: Vec<(String, bool)>,
    selected: usize,
    last_modified: std::time::SystemTime,
}

impl SnacksExplorer {
    pub fn new(root: PathBuf) -> Self {
        let mut explorer = Self {
            root,
            entries: Vec::new(),
            selected: 0,
            last_modified: SystemTime::UNIX_EPOCH,
        };
        explorer.reload();
        explorer
    }

    pub fn reload(&mut self) {
        let mut entries = Vec::new();
        if let Ok(dir) = std::fs::read_dir(&self.root) {
            for entry in dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                entries.push((name, is_dir));
            }
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        self.entries = entries;
        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
        self.last_modified = std::fs::metadata(&self.root).and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
    }
}

impl Component for SnacksExplorer {
    fn handle_event(&mut self, event: &Event, cx: &mut Context) -> EventResult {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if !self.entries.is_empty() {
                        self.selected = (self.selected + 1).min(self.entries.len() - 1);
                    }
                    return EventResult::Consumed(None);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.selected = self.selected.saturating_sub(1);
                    return EventResult::Consumed(None);
                }
                KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
                    if let Some(parent) = self.root.parent() {
                        self.root = parent.to_path_buf();
                        self.reload();
                    }
                    return EventResult::Consumed(None);
                }
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                    if let Some((name, is_dir)) = self.entries.get(self.selected) {
                        if *is_dir {
                            self.root = self.root.join(name);
                            self.reload();
                            return EventResult::Consumed(None);
                        } else if matches!(code, KeyCode::Enter) {
                            let path = self.root.join(name);
                            let _ = cx.editor.open(&path, helix_view::editor::Action::Replace);
                            cx.editor.file_explorer_active = false;
                            return EventResult::Consumed(Some(Box::new(|compositor: &mut crate::compositor::Compositor, _cx| {
                                compositor.pop();
                            })));
                        }
                    }
                    return EventResult::Consumed(None);
                }
                KeyCode::Char('r') => {
                    self.reload();
                    return EventResult::Consumed(None);
                }
                KeyCode::Char('d') => {
                    if let Some((name, is_dir)) = self.entries.get(self.selected) {
                        let path = self.root.join(name);
                        if *is_dir {
                            let _ = std::fs::remove_dir_all(&path);
                        } else {
                            let _ = std::fs::remove_file(&path);
                        }
                        self.reload();
                    }
                    return EventResult::Consumed(None);
                }
                KeyCode::Char('c') | KeyCode::Char('a') => {
                    let root = self.root.clone();
                    let prompt = Prompt::new(
                        "Create file/dir (ends with / for dir): ".into(),
                        None,
                        |_, _| Vec::new(),
                        move |_cx: &mut Context, input: &str, event: PromptEvent| {
                            if event == PromptEvent::Validate {
                                let new_path = root.join(input);
                                if input.ends_with('/') {
                                    let _ = std::fs::create_dir_all(&new_path);
                                } else {
                                    if let Some(parent) = new_path.parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    let _ = std::fs::File::create(&new_path);
                                }
                            }
                        }
                    );
                    return EventResult::Consumed(Some(Box::new(move |compositor: &mut crate::compositor::Compositor, _cx| {
                        compositor.push(Box::new(prompt));
                    })));
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    cx.editor.file_explorer_active = false;
                    return EventResult::Consumed(Some(Box::new(|compositor: &mut crate::compositor::Compositor, _cx| {
                        compositor.pop();
                    })));
                }
                _ => {}
            }
        }
        EventResult::Ignored(None)
    }

    fn render(&mut self, area: Rect, surface: &mut Surface, cx: &mut Context) {
        let current_modified = std::fs::metadata(&self.root).and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        if current_modified != self.last_modified {
            self.reload();
        }

        let style = cx.editor.theme.get("ui.text");
        let selected_style = cx.editor.theme.get("ui.selection");

        let config = cx.editor.config();
        let mut explorer_area = area;
        explorer_area.width = config.file_explorer.width;
        if config.file_explorer.side == helix_view::editor::FileExplorerSide::Right {
            explorer_area.x = area.x + area.width.saturating_sub(config.file_explorer.width);
        }

        surface.clear_with(explorer_area, cx.editor.theme.get("ui.background"));

        for (i, (name, is_dir)) in self.entries.iter().enumerate() {
            let row = explorer_area.y + i as u16;
            if row >= explorer_area.y + explorer_area.height {
                break;
            }

            let style_to_use = if i == self.selected {
                selected_style
            } else {
                style
            };

            let glyph = file_icon(name);
            let icon_str = if *is_dir {
                "📁"
            } else {
                glyph.icon
            };

            let line = format!("{} {}", icon_str, name);
            surface.set_stringn(explorer_area.x, row, &line, explorer_area.width as usize, style_to_use);
        }
    }
}
