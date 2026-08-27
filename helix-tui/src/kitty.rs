//! Kitty Graphics Protocol encoder.
//! Ref: https://sw.kovidgoyal.net/kitty/graphics-protocol/

use std::io::Write;

pub type ImageId = u32;
const MAX_CHUNK: usize = 4000;

/// Transmit PNG data to the terminal (cache only, don't display).
pub fn transmit_png(id: ImageId, png_data: &[u8], buf: &mut Vec<u8>) {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let b64 = STANDARD.encode(png_data);
    let chunks: Vec<&[u8]> = b64.as_bytes().chunks(MAX_CHUNK).collect();

    for (i, chunk) in chunks.iter().enumerate() {
        let more = if i < chunks.len() - 1 { 1 } else { 0 };
        if i == 0 {
            let _ = write!(buf, "\x1b_Ga=t,q=2,f=100,i={id},m={more};");
        } else {
            let _ = write!(buf, "\x1b_Gm={more};");
        }
        buf.extend_from_slice(chunk);
        buf.extend_from_slice(b"\x1b\\");
    }
}

/// Place a cached image at the current cursor position.
pub fn place_image(id: ImageId, cols: u16, rows: u16, buf: &mut Vec<u8>) {
    let _ = write!(buf, "\x1b_Ga=p,q=2,i={id},c={cols},r={rows},C=1\x1b\\");
}

/// Delete a specific image's placements.
pub fn delete_image_placements(id: ImageId, buf: &mut Vec<u8>) {
    let _ = write!(buf, "\x1b_Ga=d,q=2,d=i,i={id}\x1b\\");
}

/// Delete ALL visible image placements.
pub fn delete_all_placements(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1b_Ga=d,q=2,d=a\x1b\\");
}

/// ANSI cursor movement to absolute position (0-indexed input, 1-indexed output).
pub fn cursor_goto(row: u16, col: u16, buf: &mut Vec<u8>) {
    let _ = write!(buf, "\x1b[{};{}H", row + 1, col + 1);
}

/// Save cursor position.
pub fn cursor_save(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1b7");
}

/// Restore cursor position.
pub fn cursor_restore(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1b8");
}

/// Detect if the terminal supports Kitty graphics protocol.
pub fn is_supported() -> bool {
    // Forcing true to bypass tmux/ssh env var masking issues for now.
    return true;
}

