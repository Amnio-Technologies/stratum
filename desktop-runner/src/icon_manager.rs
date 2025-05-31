use egui::{ColorImage, TextureFilter, TextureHandle};
use resvg::{render, usvg};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, path::PathBuf};
use tiny_skia::{Pixmap, Transform as SkiaTransform};

pub struct IconManager {
    ctx: egui::Context,
    cache: HashMap<String, TextureHandle>,
    output_dir: PathBuf,
}

impl IconManager {
    /// Create a new IconManager, pointing `output_dir` at where PNGs will be written.
    pub fn new(ctx: egui::Context, output_dir: impl Into<PathBuf>) -> Self {
        let dir = output_dir.into();
        fs::create_dir_all(&dir).expect("failed to create icon output dir");
        IconManager {
            ctx,
            cache: HashMap::new(),
            output_dir: dir,
        }
    }

    /// Start a new request; call `.size(w, h)` or `.square(s)` to finish it.
    pub fn icon(&mut self, svg_bytes: impl Into<Vec<u8>>) -> IconRequest {
        IconRequest {
            manager: self,
            svg_bytes: svg_bytes.into(),
        }
    }

    /// Try to retrieve a cached texture by its key.
    fn get_cached(&self, key: &str) -> Option<TextureHandle> {
        self.cache.get(key).cloned()
    }

    /// Insert a newly created texture into the cache.
    fn insert_into_cache(&mut self, key: String, texture: TextureHandle) {
        self.cache.insert(key, texture);
    }

    /// Write out the pixmap as a PNG file for inspection/debugging.
    fn write_png_to_disk(&self, key: &str, width: u32, height: u32, pixmap: &Pixmap) {
        let filename = format!("{}-{}x{}.png", key, width, height);
        let path = self.output_dir.join(&filename);
        pixmap
            .save_png(&path)
            .unwrap_or_else(|_| panic!("failed to save icon PNG to {:?}", path));
    }

    /// Upload the RGBA data to egui and return a TextureHandle.
    fn upload_to_egui(&self, key: &str, width: u32, height: u32, pixmap: &Pixmap) -> TextureHandle {
        let image =
            ColorImage::from_rgba_unmultiplied([width as usize, height as usize], pixmap.data());
        self.ctx.load_texture(
            key,
            image,
            egui::TextureOptions {
                minification: TextureFilter::Nearest,
                magnification: TextureFilter::Nearest,
                ..Default::default()
            },
        )
    }
}

pub struct IconRequest<'a> {
    manager: &'a mut IconManager,
    svg_bytes: Vec<u8>,
}

impl<'a> IconRequest<'a> {
    /// Rasterize to exactly `width x height`, returning an `egui::TextureHandle`.
    pub fn size(self, width: u32, height: u32) -> TextureHandle {
        let key = self.compute_key(width, height);

        if let Some(tex) = self.manager.get_cached(&key) {
            return tex;
        }

        let (tree, sx, sy) = self.parse_and_compute_scale(width, height);

        let mut pixmap = Pixmap::new(width, height).expect("failed to allocate pixmap");
        let tx = SkiaTransform::from_scale(sx, sy);
        render(&tree, tx, &mut pixmap.as_mut());

        self.manager.write_png_to_disk(&key, width, height, &pixmap);

        let texture = self.manager.upload_to_egui(&key, width, height, &pixmap);

        self.manager.insert_into_cache(key.clone(), texture.clone());
        texture
    }

    /// Convenience for `.size(s, s)`.
    pub fn square(self, size: u32) -> TextureHandle {
        self.size(size, size)
    }

    /// Hash the SVG bytes + width + height into a unique key.
    fn compute_key(&self, width: u32, height: u32) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.svg_bytes);
        hasher.update(&width.to_le_bytes());
        hasher.update(&height.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Step 3: Parse the SVG into a `usvg::Tree`, then extract the canvas size and compute `sx`/`sy` so that `orig_w x orig_h` -> `width x height`.
    fn parse_and_compute_scale(&self, width: u32, height: u32) -> (usvg::Tree, f32, f32) {
        let opts = usvg::Options::default();
        let tree = usvg::Tree::from_data(&self.svg_bytes, &opts).expect("invalid SVG data");

        // extract viewBox dimensions
        let canvas = tree.size();
        let orig_w = canvas.width();
        let orig_h = canvas.height();

        // Compute uniform scale
        let sx = (width as f32) / orig_w;
        let sy = (height as f32) / orig_h;

        (tree, sx, sy)
    }
}
