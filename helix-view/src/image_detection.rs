//! Scan documents for inline images and math expressions.

use crate::image::{ImageAnchor, ImageSource};
use helix_core::Rope;
use std::path::{Path, PathBuf};

/// Scan a document for inline images and math expressions.
pub fn detect_images(
    text: &Rope,
    language_id: Option<&str>,
    doc_dir: Option<&Path>,
) -> Vec<ImageAnchor> {
    let mut anchors = Vec::new();
    let text_str = text.to_string();

    match language_id {
        Some("markdown") => {
            detect_markdown_images(&text_str, text, doc_dir, &mut anchors);
            detect_math_expressions(&text_str, text, &mut anchors);
        }
        Some("org") => {
            detect_org_images(&text_str, text, doc_dir, &mut anchors);
            detect_markdown_images(&text_str, text, doc_dir, &mut anchors);
            detect_math_expressions(&text_str, text, &mut anchors);
        }
        _ => {}
    }

    anchors.sort_by_key(|a| a.char_idx);
    anchors
}

fn detect_markdown_images(
    text_str: &str,
    rope: &Rope,
    doc_dir: Option<&Path>,
    anchors: &mut Vec<ImageAnchor>,
) {
    let re = regex::Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
    for cap in re.captures_iter(text_str) {
        let path_str = cap.get(2).unwrap().as_str();
        if is_image_extension(path_str) {
            let full_match = cap.get(0).unwrap();
            let byte_start = full_match.start();
            let char_idx = rope.byte_to_char(byte_start);
            let doc_line = rope.char_to_line(char_idx);
            let resolved = resolve_path(path_str, doc_dir);
            anchors.push(ImageAnchor {
                doc_line,
                char_idx,
                source: ImageSource::File(resolved),
            });
        }
    }
}

fn detect_org_images(
    text_str: &str,
    rope: &Rope,
    doc_dir: Option<&Path>,
    anchors: &mut Vec<ImageAnchor>,
) {
    let re = regex::Regex::new(r"\[\[(?:file:)?([^\]]+)\]\]").unwrap();
    for cap in re.captures_iter(text_str) {
        let path_str = cap.get(1).unwrap().as_str();
        if is_image_extension(path_str) {
            let full_match = cap.get(0).unwrap();
            let byte_start = full_match.start();
            let char_idx = rope.byte_to_char(byte_start);
            let doc_line = rope.char_to_line(char_idx);
            let resolved = resolve_path(path_str, doc_dir);
            anchors.push(ImageAnchor {
                doc_line,
                char_idx,
                source: ImageSource::File(resolved),
            });
        }
    }
}

fn detect_math_expressions(text_str: &str, rope: &Rope, anchors: &mut Vec<ImageAnchor>) {
    let re = regex::Regex::new(r"\$\$([^$]+)\$\$").unwrap();
    for cap in re.captures_iter(text_str) {
        let expr = cap.get(1).unwrap().as_str().trim();
        let full_match = cap.get(0).unwrap();
        let byte_start = full_match.start();
        let char_idx = rope.byte_to_char(byte_start);
        let doc_line = rope.char_to_line(char_idx);
        anchors.push(ImageAnchor {
            doc_line,
            char_idx,
            source: ImageSource::Math(expr.to_string()),
        });
    }
}

fn is_image_extension(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".webp")
        || lower.ends_with(".gif")
        || lower.ends_with(".bmp")
        || lower.ends_with(".svg")
}

fn resolve_path(path_str: &str, doc_dir: Option<&Path>) -> PathBuf {
    let p = PathBuf::from(path_str);
    if p.is_absolute() {
        p
    } else if let Some(dir) = doc_dir {
        dir.join(p)
    } else {
        p
    }
}
