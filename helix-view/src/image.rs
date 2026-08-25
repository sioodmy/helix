//! Inline image management: loading, caching, math rendering, and state tracking.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

pub type ImageId = u32;

static NEXT_IMAGE_ID: AtomicU32 = AtomicU32::new(1000);

fn next_id() -> ImageId {
    NEXT_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
}

/// Cached image data ready for Kitty protocol transmission.
#[derive(Clone)]
pub struct ImageData {
    pub id: ImageId,
    pub png_data: std::sync::Arc<Vec<u8>>,
    pub width_px: u32,
    pub height_px: u32,
    pub transmitted: bool,
}

/// What to render at an anchor point.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageSource {
    File(PathBuf),
    Math(String),
}

/// An image's position in a document.
#[derive(Debug, Clone)]
pub struct ImageAnchor {
    pub doc_line: usize,
    pub char_idx: usize,
    pub char_length: usize,
    pub source: ImageSource,
}

/// Screen placement computed during decoration rendering.
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    pub image_id: ImageId,
    pub screen_row: u16,
    pub screen_col: u16,
    pub cols: u16,
    pub rows: u16,
    pub needs_transmit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    FilePath(PathBuf),
    MathExpr(String),
}

/// Central image manager, held in `Editor`.
pub struct ImageManager {
    cache: std::cell::RefCell<HashMap<CacheKey, ImageData>>,
    pub failed_loads: std::cell::RefCell<std::collections::HashSet<CacheKey>>,
    pub placements: std::cell::RefCell<Vec<ImagePlacement>>,
    pub pending_math: std::cell::RefCell<std::collections::HashSet<u64>>,
    pub cell_width_px: u16,
    pub cell_height_px: u16,
    pub supported: bool,
    pub math_color: String,
}

impl Default for ImageManager {
    fn default() -> Self {
        Self {
            cache: std::cell::RefCell::new(HashMap::new()),
            failed_loads: std::cell::RefCell::new(std::collections::HashSet::new()),
            placements: std::cell::RefCell::new(Vec::new()),
            pending_math: std::cell::RefCell::new(std::collections::HashSet::new()),
            cell_width_px: 8,
            cell_height_px: 16,
            supported: false,
            math_color: "white".to_string(),
        }
    }
}

impl ImageManager {
    /// Called once at startup. `supported` should be the result of
    /// `tui::kitty::is_supported()` from the terminal layer.
    pub fn init(&mut self, supported: bool) {
        self.supported = supported;
        if self.supported {
            self.query_cell_size();
        }
    }

    /// Query terminal cell size via TIOCGWINSZ ioctl.
    fn query_cell_size(&mut self) {
        #[cfg(unix)]
        {
            use std::mem::MaybeUninit;
            let mut ws = MaybeUninit::<libc::winsize>::uninit();
            let ret = unsafe {
                libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ws.as_mut_ptr())
            };
            if ret == 0 {
                let ws = unsafe { ws.assume_init() };
                if ws.ws_xpixel > 0 && ws.ws_ypixel > 0 && ws.ws_col > 0 && ws.ws_row > 0 {
                    self.cell_width_px = ws.ws_xpixel / ws.ws_col;
                    self.cell_height_px = ws.ws_ypixel / ws.ws_row;
                }
            }
        }
    }

    /// Get or load an image, returns a cached ImageData.
    pub fn get_or_load(&self, source: &ImageSource, max_cols: u16) -> Option<ImageData> {
        let key = match source {
            ImageSource::File(p) => CacheKey::FilePath(p.clone()),
            ImageSource::Math(e) => CacheKey::MathExpr(e.clone()),
        };
        if self.failed_loads.borrow().contains(&key) {
            return None;
        }

        let mut cache = self.cache.borrow_mut();
        if !cache.contains_key(&key) {
            if let Some(data) = match source {
                ImageSource::File(path) => self.load_image_file(path, max_cols),
                ImageSource::Math(expr) => self.render_math(expr, max_cols),
            } {
                cache.insert(key.clone(), data);
            } else {
                self.failed_loads.borrow_mut().insert(key.clone());
            }
        }
        cache.get(&key).cloned()
    }

    /// Check if an image is already cached (immutable access, for LineAnnotation).
    pub fn peek_cached(&self, source: &ImageSource) -> Option<ImageData> {
        let key = match source {
            ImageSource::File(p) => CacheKey::FilePath(p.clone()),
            ImageSource::Math(e) => CacheKey::MathExpr(e.clone()),
        };
        self.cache.borrow().get(&key).cloned()
    }

    /// Get image data by ID.
    pub fn get_by_id(&self, id: ImageId) -> Option<ImageData> {
        self.cache.borrow().values().find(|d| d.id == id).cloned()
    }

    /// Mark an image as transmitted.
    pub fn mark_transmitted(&self, id: ImageId) {
        for data in self.cache.borrow_mut().values_mut() {
            if data.id == id {
                data.transmitted = true;
            }
        }
    }

    /// Clear placements for next frame.
    pub fn clear_placements(&self) {
        self.placements.borrow_mut().clear();
    }

    /// How many terminal rows an image occupies.
    pub fn image_rows(&self, data: &ImageData) -> u16 {
        ((data.height_px as f64 / self.cell_height_px as f64).ceil() as u16).max(1)
    }

    /// How many terminal cols an image occupies.
    pub fn image_cols(&self, data: &ImageData) -> u16 {
        ((data.width_px as f64 / self.cell_width_px as f64).ceil() as u16).max(1)
    }

    /// Load and scale an image file.
    fn load_image_file(&self, path: &Path, max_cols: u16) -> Option<ImageData> {
        let img = image::open(path).ok()?;
        let max_width_px = max_cols as u32 * self.cell_width_px as u32;

        let (w, h) = if img.width() > max_width_px {
            let ratio = max_width_px as f64 / img.width() as f64;
            (max_width_px, (img.height() as f64 * ratio) as u32)
        } else {
            (img.width(), img.height())
        };

        let scaled = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
        let mut png_buf = Vec::new();
        scaled
            .write_to(&mut Cursor::new(&mut png_buf), image::ImageFormat::Png)
            .ok()?;

        Some(ImageData {
            id: next_id(),
            png_data: std::sync::Arc::new(png_buf),
            width_px: w,
            height_px: h,
            transmitted: false,
        })
    }

    /// Render a math expression to PNG via mathtosvg + resvg, cache to disk, and load.
    fn render_math(&self, expr: &str, max_cols: u16) -> Option<ImageData> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        expr.hash(&mut hasher);
        self.math_color.hash(&mut hasher);
        let hash = hasher.finish();

        let math_dir = helix_loader::cache_dir().join("math_renders");
        let _ = std::fs::create_dir_all(&math_dir);
        let cache_path = math_dir.join(format!("{:016x}.png", hash));

        if cache_path.exists() {
            return self.load_image_file(&cache_path, max_cols);
        }

        if !self.pending_math.borrow().contains(&hash) {
            self.pending_math.borrow_mut().insert(hash);
            let expr_clone = expr.to_string();
            let color_arg = self.math_color.clone();
            std::thread::spawn(move || {
                let color = if color_arg.is_empty() { "white" } else { &color_arg };
                if let Ok((svg_string, _)) = mathtosvg::process_query(&expr_clone, None, Some(color), None) {
                    let mut opts = usvg::Options::default();
                    opts.fontdb_mut().load_system_fonts();
                    opts.fontdb_mut().set_serif_family("Times New Roman");
                    opts.fontdb_mut().set_sans_serif_family("Arial");
                    opts.fontdb_mut().set_cursive_family("Comic Sans MS");
                    opts.fontdb_mut().set_fantasy_family("Impact");
                    opts.fontdb_mut().set_monospace_family("Courier New");
                    
                    if let Ok(tree) = usvg::Tree::from_str(&svg_string, &opts) {
                        let size = tree.size();
                        let svg_w = (size.width() / 2.0).max(1.0) as u32;
                        let svg_h = (size.height() / 2.0).max(1.0) as u32;

                        if let Some(mut pixmap) = tiny_skia::Pixmap::new(svg_w, svg_h) {
                            let transform = tiny_skia::Transform::from_scale(
                                svg_w as f32 / size.width(),
                                svg_h as f32 / size.height(),
                            );
                            resvg::render(&tree, transform, &mut pixmap.as_mut());
                            if let Ok(png_data) = pixmap.encode_png() {
                                let _ = std::fs::write(&cache_path, &png_data);
                            }
                        }
                    }
                }
            });
        }
        
        None
    }
}
