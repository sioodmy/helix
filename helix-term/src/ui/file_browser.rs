use crate::{
    compositor::{Component, Context, Event, EventResult},
    ctrl,
    job::Callback,
    ui::{
        overlay::{self, overlaid},
        Picker, PickerColumn, Prompt, PromptEvent,
    },
};
use helix_core::Position;
use helix_view::{
    graphics::Rect,
    theme::Style,
    Editor,
};
use std::path::PathBuf;
use tui::buffer::Buffer as Surface;
use tui::text::Span;
use std::error::Error;

fn get_file_icon(fname: &str) -> &'static str {
    if let Some(ext) = fname.rsplit('.').next() {
        match ext {
            "rs" => "󱘎",
            "js" => "󰌧",
            "ts" | "tsx" => "󰛦",
            "json" => "󰘦",
            "md" | "markdown" => "",
            "toml" => "󰅩",
            "yaml" | "yml" => "󰆧",
            "html" => "󰌝",
            "css" => "󰌜",
            "py" => "󰌠",
            "go" => "󰟓",
            "c" | "cpp" | "h" | "hpp" => "󰙲",
            "sh" | "bash" => "󰆍",
            _ => "󰈙",
        }
    } else {
        "󰈙"
    }
}

pub struct FileBrowser {
    picker: Picker<(PathBuf, bool), (PathBuf, Style)>,
    root: PathBuf,
}

impl FileBrowser {
    pub fn new(root: PathBuf, editor: &Editor) -> Result<Self, std::io::Error> {
        let directory_style = editor.theme.get("ui.text.directory");
        let directory_content = crate::ui::directory_content(&root, editor)?;

        let columns = [PickerColumn::new(
            "path",
            |(path, is_dir): &(PathBuf, bool), (root, directory_style): &(PathBuf, Style)| {
                let name = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
                if *is_dir {
                    Span::styled(format!("󰉋 {}/", name), *directory_style).into()
                } else {
                    let icon = get_file_icon(&name);
                    format!("{} {}", icon, name).into()
                }
            },
        )];
        
        let root_clone = root.clone();
        let picker = Picker::new(
            columns,
            0,
            directory_content,
            (root.clone(), directory_style),
            move |cx, (path, is_dir): &(PathBuf, bool), action| {
                if *is_dir {
                    let new_root = helix_stdx::path::normalize(path);
                    let callback = Box::pin(async move {
                        let call: Callback =
                            Callback::EditorCompositor(Box::new(move |editor, compositor| {
                                if let Ok(picker) = file_browser(new_root, editor) {
                                    compositor.push(Box::new(overlay::overlaid(picker)));
                                }
                            }));
                        Ok(call)
                    });
                    cx.jobs.callback(callback);
                } else if let Err(e) = cx.editor.open(path, action) {
                    let err = if let Some(err) = e.source() {
                        format!("{}", err)
                    } else {
                        format!("unable to open \"{}\"", path.display())
                    };
                    cx.editor.set_error(err);
                }
            },
        )
        .with_preview(|_editor, (path, _is_dir)| Some((path.as_path().into(), None)));

        Ok(Self { picker, root })
    }
}

pub fn file_browser(root: PathBuf, editor: &Editor) -> Result<FileBrowser, std::io::Error> {
    FileBrowser::new(root, editor)
}

fn refresh_picker(root: PathBuf, cx: &mut Context) {
    let callback = Box::pin(async move {
        let call: Callback =
            Callback::EditorCompositor(Box::new(move |editor, compositor| {
                if let Ok(picker) = file_browser(root, editor) {
                    // Pop prompt
                    compositor.pop();
                    // Pop the old file browser
                    compositor.pop();
                    compositor.push(Box::new(overlay::overlaid(picker)));
                }
            }));
        Ok(call)
    });
    cx.jobs.callback(callback);
}

impl Component for FileBrowser {
    fn handle_event(&mut self, event: &Event, cx: &mut Context) -> EventResult {
        if let Event::Key(key) = event {
            match *key {
                ctrl!('a') => {
                    let root = self.root.clone();
                    let prompt = Prompt::new(
                        "Create (ends with / for dir): ".into(),
                        None,
                        crate::ui::completers::none,
                        move |cx: &mut Context, input: &str, event: PromptEvent| {
                            if event == PromptEvent::Validate {
                                let path = root.join(input);
                                if input.ends_with('/') {
                                    let _ = std::fs::create_dir_all(&path);
                                } else {
                                    if let Some(parent) = path.parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    let _ = std::fs::File::create(&path);
                                }
                                refresh_picker(root.clone(), cx);
                            }
                        },
                    );
                    return EventResult::Consumed(Some(Box::new(|compositor, _| {
                        compositor.push(Box::new(prompt));
                    })));
                }
                ctrl!('r') => {
                    if let Some((path, _)) = self.picker.selection() {
                        let path = path.clone();
                        let root = self.root.clone();
                        let default_name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                        
                        let prompt = Prompt::new(
                            "Rename to: ".into(),
                            None,
                            crate::ui::completers::none,
                            move |cx: &mut Context, input: &str, event: PromptEvent| {
                                if event == PromptEvent::Validate {
                                    let new_path = root.join(input);
                                    let _ = std::fs::rename(&path, &new_path);
                                    refresh_picker(root.clone(), cx);
                                }
                            },
                        ).with_line(default_name, cx.editor);
                        return EventResult::Consumed(Some(Box::new(|compositor, _| {
                            compositor.push(Box::new(prompt));
                        })));
                    }
                    return EventResult::Consumed(None);
                }
                ctrl!('x') => {
                    if let Some((path, is_dir)) = self.picker.selection() {
                        let path = path.clone();
                        let is_dir = *is_dir;
                        let root = self.root.clone();
                        
                        let prompt = Prompt::new(
                            format!("Delete {} (y/n)? ", path.file_name().unwrap_or_default().to_string_lossy()).into(),
                            None,
                            crate::ui::completers::none,
                            move |cx: &mut Context, input: &str, event: PromptEvent| {
                                if event == PromptEvent::Validate {
                                    if input.to_lowercase() == "y" {
                                        if is_dir {
                                            let _ = std::fs::remove_dir_all(&path);
                                        } else {
                                            let _ = std::fs::remove_file(&path);
                                        }
                                    }
                                    refresh_picker(root.clone(), cx);
                                }
                            },
                        );
                        return EventResult::Consumed(Some(Box::new(|compositor, _| {
                            compositor.push(Box::new(prompt));
                        })));
                    }
                    return EventResult::Consumed(None);
                }
                _ => {}
            }
        }
        self.picker.handle_event(event, cx)
    }

    fn render(&mut self, mut area: Rect, surface: &mut Surface, cx: &mut Context) {
        if area.height > 2 {
            let footer = " [Ctrl-A] Create  [Ctrl-R] Rename  [Ctrl-X] Delete ";
            let style = cx.editor.theme.get("ui.statusline");
            let y = area.y + area.height - 1;
            surface.set_stringn(area.x, y, footer, area.width as usize, style);
            area.height -= 1;
        }
        self.picker.render(area, surface, cx);
    }

    fn required_size(&mut self, viewport: (u16, u16)) -> Option<(u16, u16)> {
        let (width, height) = self.picker.required_size(viewport)?;
        Some((width, (height + 1).min(viewport.1)))
    }

    fn cursor(&self, area: Rect, editor: &Editor) -> (Option<Position>, helix_view::graphics::CursorKind) {
        let mut p_area = area;
        if p_area.height > 2 {
            p_area.height -= 1;
        }
        self.picker.cursor(p_area, editor)
    }
}
