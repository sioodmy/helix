//! Decoration implementation for inline images.
//! Records screen coordinates for image placements during rendering.
//! The actual Kitty protocol emission happens in Application::build_kitty_frame.

use helix_core::doc_formatter::FormattedGrapheme;
use helix_core::Position;
use helix_view::graphics::Rect;
use helix_view::image::{ImageAnchor, ImageManager, ImagePlacement, ImageSource};

use crate::ui::document::{LinePos, TextRenderer};

use super::Decoration;

/// Decoration that records screen coordinates for image placements.
pub struct InlineImageDecoration<'a> {
    anchors: &'a [ImageAnchor],
    image_manager: &'a ImageManager,
    max_cols: u16,
    inner_x: u16,
    inner_y: u16,
}

impl<'a> InlineImageDecoration<'a> {
    pub fn new(
        anchors: &'a [ImageAnchor],
        image_manager: &'a ImageManager,
        max_cols: u16,
        inner_x: u16,
        inner_y: u16,
    ) -> Self {
        Self {
            anchors,
            image_manager,
            max_cols,
            inner_x,
            inner_y,
        }
    }
}

impl Decoration for InlineImageDecoration<'_> {
    fn render_virt_lines(
        &mut self,
        _renderer: &mut TextRenderer,
        pos: LinePos,
        virt_off: Position,
    ) -> Position {
        let mut total_rows = 0u16;
        let cell_w = self.image_manager.cell_width_px;
        let cell_h = self.image_manager.cell_height_px;

        for anchor in self.anchors.iter() {
            if anchor.doc_line != pos.doc_line {
                if anchor.doc_line > pos.doc_line {
                    break;
                }
                continue;
            }

            // Try to get/load the image
            let source_clone = anchor.source.clone();
            if let Some(data) = self.image_manager.get_or_load(&source_clone, self.max_cols) {
                // Extract values before releasing the borrow
                let id = data.id;
                let w_px = data.width_px;
                let h_px = data.height_px;
                let needs_transmit = !data.transmitted;

                let cols = ((w_px as f64 / cell_w as f64).ceil() as u16).max(1);
                let rows = ((h_px as f64 / cell_h as f64).ceil() as u16).max(1);

                let screen_row =
                    self.inner_y + pos.visual_line + virt_off.row as u16 + total_rows;
                let screen_col = self.inner_x;

                // Record placement for Kitty emission in Application::build_kitty_frame
                self.image_manager.placements.borrow_mut().push(ImagePlacement {
                    image_id: id,
                    screen_row,
                    screen_col,
                    cols,
                    rows,
                    needs_transmit,
                });

                total_rows += rows;
            }
        }

        Position::new(total_rows as usize, 0)
    }
}
