//! LineAnnotation implementation for inline images.
//! Reserves virtual lines in the document layout for image rendering.

use helix_core::doc_formatter::FormattedGrapheme;
use helix_core::text_annotations::LineAnnotation;
use helix_core::Position;

use crate::image::{ImageAnchor, ImageManager, ImageSource};

/// LineAnnotation that reserves virtual lines for inline images.
pub struct InlineImageAnnotation<'a> {
    anchors: &'a [ImageAnchor],
    current: usize,
    image_manager: &'a ImageManager,
    max_cols: u16,
}

impl<'a> InlineImageAnnotation<'a> {
    pub fn new(
        anchors: &'a [ImageAnchor],
        image_manager: &'a ImageManager,
        max_cols: u16,
    ) -> Box<dyn LineAnnotation + 'a> {
        Box::new(Self {
            anchors,
            current: 0,
            image_manager,
            max_cols,
        })
    }

    fn next_anchor_char_idx(&self) -> usize {
        self.anchors
            .get(self.current)
            .map_or(usize::MAX, |a| a.char_idx)
    }
}

impl LineAnnotation for InlineImageAnnotation<'_> {
    fn reset_pos(&mut self, char_idx: usize) -> usize {
        self.current = self.anchors.partition_point(|a| a.char_idx < char_idx);
        self.next_anchor_char_idx()
    }

    fn skip_concealed_anchors(&mut self, conceal_end_char_idx: usize) -> usize {
        self.reset_pos(conceal_end_char_idx)
    }

    fn process_anchor(&mut self, _grapheme: &FormattedGrapheme) -> usize {
        self.current += 1;
        self.next_anchor_char_idx()
    }

    fn insert_virtual_lines(
        &mut self,
        _line_end_char_idx: usize,
        _line_end_visual_pos: Position,
        doc_line: usize,
    ) -> Position {
        let mut total_rows = 0usize;
        for anchor in self.anchors.iter() {
            if anchor.doc_line != doc_line {
                if anchor.doc_line > doc_line {
                    break;
                }
                continue;
            }

            let rows = match &anchor.source {
                ImageSource::File(path) => {
                    let key = crate::image::ImageSource::File(path.clone());
                    // We can only check the cache here (immutable borrow)
                    // The actual loading happens in the Decoration
                    self.image_manager
                        .peek_cached(&key)
                        .as_ref()
                        .map(|data| self.image_manager.image_rows(data) as usize)
                }
                ImageSource::Math(expr) => {
                    let key = crate::image::ImageSource::Math(expr.clone());
                    self.image_manager
                        .peek_cached(&key)
                        .as_ref()
                        .map(|data| self.image_manager.image_rows(data) as usize)
                }
            };

            if let Some(rows) = rows {
                total_rows += rows;
            }
        }

        Position::new(total_rows, 0)
    }
}
