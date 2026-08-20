use crate::compositor::{Component, Context, EventResult, Event};
use helix_view::input::KeyEvent;
use helix_view::keyboard::KeyCode;
use helix_view::graphics::{CursorKind, Rect};
use helix_view::theme::Modifier;
use helix_view::Editor;
use helix_core::Position;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout},
};

use crate::compositor::Compositor;

pub struct Dashboard {}

impl Dashboard {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for Dashboard {
    fn render(&mut self, area: Rect, surface: &mut Buffer, cx: &mut Context) {
        let bg = cx.editor.theme.get("ui.background");
        surface.clear_with(area, bg);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);

        let ascii_logo = vec![
            "                           =            ",
            "                          +:            ",
            "                         -#             ",
            "         .-==-. .       +#              ",
            "              :+%%*=+#%@@+              ",
            "                -@@@@@@@@#              ",
            "                *@@@@@@@@@#+-           ",
            "            ...=@@@@@@@@@@@@@%+=:..     ",
            "       :--#@@#*#@@@@@@@@@@@#.           ",
            "      :=%#*=:    #@@@@@@@@@*            ",
            "   .*+=-.        =@@*+--+#@@#.          ",
            " .=+.           .%@-        .++         ",
            " ..             .@@           :=        ",
            "                =@#                     ",
            "                *@-                     ",
            "                :@                      ",
            "                 +                      ",
        ];

        let logo_height = ascii_logo.len() as u16;
        let logo_width = ascii_logo.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        
        let logo_x = layout[0].x + layout[0].width.saturating_sub(logo_width) / 2;
        let logo_y = layout[0].y + layout[0].height.saturating_sub(logo_height) / 2;

        let logo_style = cx.editor.theme.get("ui.text");
        for (i, line) in ascii_logo.iter().enumerate() {
            surface.set_string(logo_x, logo_y + i as u16, line, logo_style);
        }

        let is_git = std::process::Command::new("git")
            .args(&["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let mut menu_items = vec![
            (" Find file", "f"),
            ("󰈭 Find word", "w"),
        ];
        if is_git {
            menu_items.push(("󰈚 Recent files", "r"));
        }
        menu_items.push((" New file", "n"));
        menu_items.push((" Quit Helix", "q"));

        let y_start = layout[1].y + 2;
        let total_width = 40;

        for (i, (text, key)) in menu_items.iter().enumerate() {
            let offset_x = (layout[1].x + layout[1].width / 2).saturating_sub(total_width / 2);
            let y = y_start + (i as u16) * 2;
            
            let text_style = cx.editor.theme.get("ui.text");
            let key_style = cx.editor.theme.get("ui.text.focus").add_modifier(Modifier::BOLD);

            surface.set_string(offset_x, y, text, text_style);
            
            let padding = 38u16.saturating_sub(tui::text::Span::raw(*text).width() as u16);
            surface.set_string(offset_x + (tui::text::Span::raw(*text).width() as u16) + padding, y, key, key_style);
        }

        let footer_prefix = "♥ Support me: ";
        let footer_link = "siood.my/donate";
        let footer_width = (tui::text::Span::raw(footer_prefix).width() + tui::text::Span::raw(footer_link).width()) as u16;
        let footer_x = area.x + area.width.saturating_sub(footer_width) / 2;
        let footer_y = area.bottom().saturating_sub(2);

        let text_style = cx.editor.theme.get("ui.text").add_modifier(Modifier::DIM);
        let link_style = cx.editor.theme.get("ui.text").link("https://siood.my/donate").underline_style(helix_view::graphics::UnderlineStyle::Line);

        surface.set_string(footer_x, footer_y, footer_prefix, text_style);
        surface.set_string(footer_x + tui::text::Span::raw(footer_prefix).width() as u16, footer_y, footer_link, link_style);
    }

    fn handle_event(&mut self, event: &Event, cx: &mut Context) -> EventResult {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event {
            match code {
                KeyCode::Char('f') if modifiers.is_empty() => {
                    return EventResult::Consumed(Some(Box::new(|compositor: &mut Compositor, cx: &mut Context| {
                        compositor.pop();
                        let picker = crate::ui::file_picker(
                            cx.editor,
                            std::env::current_dir().unwrap_or_else(|_| ".".into())
                        );
                        compositor.push(Box::new(crate::ui::overlay::overlaid(picker)));
                    })));
                }
                KeyCode::Char('w') if modifiers.is_empty() => {
                    return EventResult::Consumed(Some(Box::new(|compositor: &mut Compositor, cx: &mut Context| {
                        compositor.pop();
                        let mut cmd_cx = crate::commands::Context {
                            editor: cx.editor,
                            count: None,
                            register: None,
                            jobs: cx.jobs,
                            callback: Vec::new(),
                            on_next_key_callback: None,
                        };
                        crate::commands::global_search(&mut cmd_cx);
                        for cb in cmd_cx.callback {
                            cb(compositor, cx);
                        }
                    })));
                }
                KeyCode::Char('r') if modifiers.is_empty() => {
                    return EventResult::Consumed(Some(Box::new(|compositor: &mut Compositor, cx: &mut Context| {
                        compositor.pop();
                        let mut cmd_cx = crate::commands::Context {
                            editor: cx.editor,
                            count: None,
                            register: None,
                            jobs: cx.jobs,
                            callback: Vec::new(),
                            on_next_key_callback: None,
                        };
                        crate::commands::git_recent_files(&mut cmd_cx);
                        for cb in cmd_cx.callback {
                            cb(compositor, cx);
                        }
                    })));
                }
                KeyCode::Char('n') if modifiers.is_empty() => {
                    return EventResult::Consumed(Some(Box::new(|compositor: &mut Compositor, cx: &mut Context| {
                        compositor.pop();
                        cx.editor.new_file(helix_view::editor::Action::Replace);
                    })));
                }
                KeyCode::Char('q') if modifiers.is_empty() => {
                    return EventResult::Consumed(Some(Box::new(|_compositor: &mut Compositor, cx: &mut Context| {
                        let view_id = cx.editor.tree.focus;
                        let _ = cx.editor.close(view_id);
                    })));
                }
                KeyCode::Char(c) if modifiers.is_empty() || *modifiers == helix_view::keyboard::KeyModifiers::SHIFT => {
                    if ![':', '/', '?', ' '].contains(c) {
                        return EventResult::Consumed(None);
                    }
                }
                _ => {}
            }
        }
        EventResult::Ignored(None)
    }

    fn cursor(&self, _area: Rect, _editor: &Editor) -> (Option<Position>, CursorKind) {
        (Some(Position::new(0, 0)), CursorKind::Hidden)
    }
}
