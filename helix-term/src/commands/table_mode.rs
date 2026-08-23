use crate::commands::*;
use helix_core::{Tendril, Transaction};

pub fn table_mode_toggle(cx: &mut Context) {
    let doc_table_mode = {
        let (_, doc) = current_ref!(cx.editor);
        !doc.table_mode
    };
    {
        let doc = doc_mut!(cx.editor);
        doc.table_mode = doc_table_mode;
    }
    cx.editor.set_status(format!("Table mode: {}", if doc_table_mode { "ON" } else { "OFF" }));
}

pub fn table_mode_enable(cx: &mut Context) {
    {
        let doc = doc_mut!(cx.editor);
        doc.table_mode = true;
    }
    cx.editor.set_status("Table mode: ON".to_string());
}

pub fn table_mode_disable(cx: &mut Context) {
    {
        let doc = doc_mut!(cx.editor);
        doc.table_mode = false;
    }
    cx.editor.set_status("Table mode: OFF".to_string());
}

fn is_table_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') || trimmed.starts_with('+')
}

fn is_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !is_table_line(line) { return false; }
    trimmed.chars().all(|c| c == '|' || c == '+' || c == '-' || c == '=' || c == ':' || c.is_whitespace())
}

fn split_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() { return vec![]; }
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = trimmed.chars().peekable();
    
    // skip first border
    if let Some(&c) = chars.peek() {
        if c == '|' || c == '+' {
            chars.next();
        }
    }
    
    while let Some(c) = chars.next() {
        if c == '|' || c == '+' {
            cells.push(current.trim().to_string());
            current.clear();
        } else if c == '\\' {
            if let Some(&next) = chars.peek() {
                if next == '|' || next == '+' {
                    current.push(chars.next().unwrap());
                } else {
                    current.push(c);
                }
            } else {
                current.push(c);
            }
        } else {
            current.push(c);
        }
    }
    
    cells
}

#[derive(Clone, Copy, PartialEq)]
enum Align { Left, Center, Right, Default }

fn parse_alignment(cell: &str) -> Align {
    let cell = cell.trim();
    if cell.is_empty() { return Align::Default; }
    let starts_with_colon = cell.starts_with(':');
    let ends_with_colon = cell.ends_with(':');
    if starts_with_colon && ends_with_colon {
        Align::Center
    } else if ends_with_colon {
        Align::Right
    } else if starts_with_colon {
        Align::Left
    } else {
        Align::Default
    }
}

fn align_cell(content: &str, width: usize, align: Align) -> String {
    let content_len = content.chars().count();
    let pad = if width > content_len { width - content_len } else { 0 };
    match align {
        Align::Left | Align::Default => format!(" {}{} ", content, " ".repeat(pad)),
        Align::Right => format!(" {}{} ", " ".repeat(pad), content),
        Align::Center => {
            let left_pad = pad / 2;
            let right_pad = pad - left_pad;
            format!(" {}{}{} ", " ".repeat(left_pad), content, " ".repeat(right_pad))
        }
    }
}

fn format_separator_cell(width: usize, align: Align, fill: char) -> String {
    match align {
        Align::Left => format!(":{fill}{}", fill.to_string().repeat(if width > 1 { width - 1 } else { 0 })),
        Align::Right => format!("{}{fill}:", fill.to_string().repeat(if width > 1 { width - 1 } else { 0 })),
        Align::Center => format!(":{fill}{fill}:", fill=fill.to_string().repeat(if width > 2 { width - 2 } else { 0 })),
        Align::Default => fill.to_string().repeat(width + 2),
    }
}

fn get_corner_char(cx: &Context) -> char {
    let (_, doc) = current_ref!(cx.editor);
    let lang = doc.language_name().unwrap_or("");
    if lang == "org" || lang == "rst" { '+' } else { '|' }
}

fn realign_table_in_range(cx: &mut Context, start_line: usize, end_line: usize) {
    let (view, doc) = current_ref!(cx.editor);
    let text = doc.text();
    let corner = get_corner_char(cx);
    
    let mut lines = Vec::new();
    for i in start_line..=end_line {
        lines.push(text.line(i).to_string());
    }
    
    let mut parsed_lines = Vec::new();
    let mut max_widths = Vec::new();
    let mut alignments = Vec::new();
    
    // First pass: parse lines and find max widths and alignments
    for line in &lines {
        let is_sep = is_separator_line(line);
        let cells = split_cells(line);
        
        if is_sep {
            for (i, cell) in cells.iter().enumerate() {
                if i >= alignments.len() { alignments.push(Align::Default); }
                let align = parse_alignment(cell);
                if align != Align::Default {
                    alignments[i] = align;
                }
            }
        } else {
            for (i, cell) in cells.iter().enumerate() {
                if i >= max_widths.len() { max_widths.push(0); }
                let width = cell.chars().count();
                if width > max_widths[i] { max_widths[i] = width; }
            }
        }
        parsed_lines.push((is_sep, cells));
    }
    
    // Ensure all rows have the same number of columns
    let num_cols = max_widths.len();
    
    // Second pass: format lines
    let mut new_lines = Vec::new();
    for (is_sep, mut cells) in parsed_lines {
        let mut new_line = String::new();
        new_line.push(if is_sep { corner } else { '|' });
        
        while cells.len() < num_cols { cells.push(String::new()); }
        
        for i in 0..num_cols {
            let width = *max_widths.get(i).unwrap_or(&3);
            let align = *alignments.get(i).unwrap_or(&Align::Default);
            let cell = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            
            if is_sep {
                new_line.push_str(&format_separator_cell(width, align, '-'));
                new_line.push(corner);
            } else {
                new_line.push_str(&align_cell(cell, width, align));
                new_line.push('|');
            }
        }
        new_lines.push(new_line);
    }
    
    let start_char = text.line_to_char(start_line);
    let end_char = text.line_to_char(end_line + 1);
    
    let line_ending = doc.line_ending.as_str();
    let replacement = new_lines.join(line_ending);
    
    let replacement = if end_line + 1 < text.len_lines() {
        replacement + line_ending
    } else {
        replacement
    };
    
    let transaction = Transaction::change(
        text,
        vec![(start_char, end_char, Some(Tendril::from(replacement)))].into_iter(),
    );
    
    let doc_id = doc.id();
    let view_id = view.id;
    let doc = doc_mut!(cx.editor, &doc_id);
    doc.apply(&transaction, view_id);
}

pub fn table_realign(cx: &mut Context) {
    let (view, doc) = current_ref!(cx.editor);
    let text = doc.text();
    let cursor_line = text.char_to_line(doc.selection(view.id).primary().cursor(text.slice(..)));
    
    let line_str = text.line(cursor_line).to_string();
    if !is_table_line(&line_str) {
        cx.editor.set_error("Not in a table".to_string());
        return;
    }
    
    let mut start_line = cursor_line;
    while start_line > 0 && is_table_line(&text.line(start_line - 1).to_string()) {
        start_line -= 1;
    }
    
    let mut end_line = cursor_line;
    while end_line + 1 < text.len_lines() && is_table_line(&text.line(end_line + 1).to_string()) {
        end_line += 1;
    }
    
    realign_table_in_range(cx, start_line, end_line);
}

pub fn auto_align_on_insert(cx: &mut Context, _c: char) {
    let (_, doc) = current_ref!(cx.editor);
    if !doc.table_mode {
        return;
    }
    // We can just call table_realign if we are in a table
    table_realign(cx);
}

pub fn tableize(cx: &mut Context) {
    cx.editor.set_status("Tableized (Not yet fully implemented)".to_string());
}

pub fn table_delete_row(_cx: &mut Context) {}
pub fn table_delete_column(_cx: &mut Context) {}
pub fn table_insert_column_after(_cx: &mut Context) {}
pub fn table_insert_column_before(_cx: &mut Context) {}
pub fn table_next_cell(_cx: &mut Context) {}
pub fn table_prev_cell(_cx: &mut Context) {}
pub fn table_up_cell(_cx: &mut Context) {}
pub fn table_down_cell(_cx: &mut Context) {}
pub fn table_add_formula(_cx: &mut Context) {}
pub fn table_eval_formula(_cx: &mut Context) {}
pub fn table_sort(_cx: &mut Context) {}
