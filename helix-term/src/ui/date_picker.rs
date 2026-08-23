use crate::compositor::{Component, Context, Event, EventResult};
use helix_view::graphics::{Margin, Rect};
use helix_view::keyboard::KeyCode;
use tui::buffer::Buffer as Surface;
use tui::widgets::{Block, Borders, Widget};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, Duration, Timelike};

type CallbackFn = Box<dyn FnOnce(&mut Context, NaiveDateTime)>;

pub struct DatePicker {
    selected_date: NaiveDate,
    view_date: NaiveDate,
    time: NaiveTime,
    focus: Focus,
    callback: Option<CallbackFn>,
}

#[derive(PartialEq)]
enum Focus {
    Calendar,
    Hour,
    Minute,
}

impl DatePicker {
    pub fn new(callback: impl FnOnce(&mut Context, NaiveDateTime) + 'static) -> Self {
        let now = Local::now().naive_local();
        let date = now.date();
        let time = now.time();
        Self {
            selected_date: date,
            view_date: NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap(),
            time,
            focus: Focus::Calendar,
            callback: Some(Box::new(callback)),
        }
    }
}

impl Component for DatePicker {
    fn handle_event(&mut self, event: &Event, _cx: &mut Context) -> EventResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    return EventResult::Consumed(Some(Box::new(|c: &mut crate::compositor::Compositor, _cx: &mut Context| {
                        c.pop();
                    })));
                }
                KeyCode::Enter => {
                    if self.focus == Focus::Calendar {
                        self.focus = Focus::Hour;
                    } else if self.focus == Focus::Hour {
                        self.focus = Focus::Minute;
                    } else {
                        // Submit!
                        let cb = self.callback.take().unwrap();
                        let dt = NaiveDateTime::new(self.selected_date, self.time);
                        return EventResult::Consumed(Some(Box::new(move |c: &mut crate::compositor::Compositor, cx: &mut Context| {
                            c.pop();
                            cb(cx, dt);
                        })));
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    match self.focus {
                        Focus::Calendar => {
                            self.selected_date = self.selected_date - Duration::days(1);
                            self.view_date = NaiveDate::from_ymd_opt(self.selected_date.year(), self.selected_date.month(), 1).unwrap();
                        }
                        Focus::Hour => {
                            let (h, m, s) = (self.time.hour(), self.time.minute(), self.time.second());
                            let mut h = h as i32 - 1;
                            if h < 0 { h = 23; }
                            self.time = NaiveTime::from_hms_opt(h as u32, m, s).unwrap();
                        }
                        Focus::Minute => {
                            let (h, m, s) = (self.time.hour(), self.time.minute(), self.time.second());
                            let mut m = m as i32 - 1;
                            if m < 0 { m = 59; }
                            self.time = NaiveTime::from_hms_opt(h, m as u32, s).unwrap();
                        }
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    match self.focus {
                        Focus::Calendar => {
                            self.selected_date = self.selected_date + Duration::days(1);
                            self.view_date = NaiveDate::from_ymd_opt(self.selected_date.year(), self.selected_date.month(), 1).unwrap();
                        }
                        Focus::Hour => {
                            let (h, m, s) = (self.time.hour(), self.time.minute(), self.time.second());
                            self.time = NaiveTime::from_hms_opt((h + 1) % 24, m, s).unwrap();
                        }
                        Focus::Minute => {
                            let (h, m, s) = (self.time.hour(), self.time.minute(), self.time.second());
                            self.time = NaiveTime::from_hms_opt(h, (m + 1) % 60, s).unwrap();
                        }
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    match self.focus {
                        Focus::Calendar => {
                            self.selected_date = self.selected_date - Duration::days(7);
                            self.view_date = NaiveDate::from_ymd_opt(self.selected_date.year(), self.selected_date.month(), 1).unwrap();
                        }
                        Focus::Hour => {
                            let (h, m, s) = (self.time.hour(), self.time.minute(), self.time.second());
                            self.time = NaiveTime::from_hms_opt((h + 1) % 24, m, s).unwrap();
                        }
                        Focus::Minute => {
                            let (h, m, s) = (self.time.hour(), self.time.minute(), self.time.second());
                            self.time = NaiveTime::from_hms_opt(h, (m + 1) % 60, s).unwrap();
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    match self.focus {
                        Focus::Calendar => {
                            self.selected_date = self.selected_date + Duration::days(7);
                            self.view_date = NaiveDate::from_ymd_opt(self.selected_date.year(), self.selected_date.month(), 1).unwrap();
                        }
                        Focus::Hour => {
                            let (h, m, s) = (self.time.hour(), self.time.minute(), self.time.second());
                            let mut h = h as i32 - 1;
                            if h < 0 { h = 23; }
                            self.time = NaiveTime::from_hms_opt(h as u32, m, s).unwrap();
                        }
                        Focus::Minute => {
                            let (h, m, s) = (self.time.hour(), self.time.minute(), self.time.second());
                            let mut m = m as i32 - 1;
                            if m < 0 { m = 59; }
                            self.time = NaiveTime::from_hms_opt(h, m as u32, s).unwrap();
                        }
                    }
                }
                KeyCode::Char('\t') => {
                    self.focus = match self.focus {
                        Focus::Calendar => Focus::Hour,
                        Focus::Hour => Focus::Minute,
                        Focus::Minute => Focus::Calendar,
                    };
                }
                _ => {}
            }
            return EventResult::Consumed(None);
        }
        EventResult::Ignored(None)
    }

    fn render(&mut self, area: Rect, surface: &mut Surface, cx: &mut Context) {
        let block = Block::default()
            .title(" Date & Time Picker (Enter: Confirm, Tab/h/j/k/l: Navigate) ")
            .borders(Borders::ALL);
        
        let width = 50;
        let height = 15;
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        
        let area = Rect::new(x, y, width, height);
        surface.clear_with(area, cx.editor.theme.get("ui.window"));
        block.render(area, surface);
        
        let inner = area.inner(Margin { vertical: 1, horizontal: 2 });
        let mut y = inner.y;
        
        let theme = &cx.editor.theme;
        let normal = theme.get("ui.text");
        let selected = theme.get("ui.menu.selected");
        let header = theme.get("ui.text.focus");
        
        let header_str = self.view_date.format("%B %Y").to_string();
        surface.set_string(inner.x + (inner.width.saturating_sub(header_str.len() as u16)) / 2, y, &header_str, header);
        y += 2;
        
        let days = "Su Mo Tu We Th Fr Sa";
        surface.set_string(inner.x + (inner.width.saturating_sub(days.len() as u16)) / 2, y, days, normal);
        y += 1;
        
        let first_day = self.view_date.weekday().num_days_from_sunday();
        let start_x = inner.x + (inner.width.saturating_sub(days.len() as u16)) / 2;
        let mut current_x = start_x + (first_day as u16) * 3;
        
        let mut curr_date = self.view_date;
        while curr_date.month() == self.view_date.month() {
            let day_str = format!("{:2}", curr_date.day());
            let style = if curr_date == self.selected_date && self.focus == Focus::Calendar {
                selected
            } else if curr_date == self.selected_date {
                header // Highlight slightly when calendar not focused
            } else {
                normal
            };
            
            surface.set_string(current_x, y, &day_str, style);
            current_x += 3;
            
            if curr_date.weekday().num_days_from_sunday() == 6 {
                current_x = start_x;
                y += 1;
            }
            curr_date = curr_date + Duration::days(1);
        }
        
        // Push y to bottom for time
        y = inner.y + 9;
        
        let time_str = "Time: ";
        let h_str = format!("{:02}", self.time.hour());
        let m_str = format!("{:02}", self.time.minute());
        
        let time_x = inner.x + (inner.width.saturating_sub(12)) / 2;
        surface.set_string(time_x, y, time_str, normal);
        
        surface.set_string(time_x + 6, y, &h_str, if self.focus == Focus::Hour { selected } else { normal });
        surface.set_string(time_x + 8, y, ":", normal);
        surface.set_string(time_x + 9, y, &m_str, if self.focus == Focus::Minute { selected } else { normal });
    }
}
