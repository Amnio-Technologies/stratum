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
    /// point `output_dir` at where you want your PNGs to land on disk
    pub fn new(ctx: egui::Context, output_dir: impl Into<PathBuf>) -> Self {
        let dir = output_dir.into();
        fs::create_dir_all(&dir).expect("failed to create icon output dir");
        IconManager {
            ctx,
            cache: HashMap::new(),
            output_dir: dir,
        }
    }

    /// start a new request; call `.size(w,h)` or `.square(s)` to finish it
    pub fn icon(&mut self, svg_bytes: impl Into<Vec<u8>>) -> IconRequest {
        IconRequest {
            manager: self,
            svg_bytes: svg_bytes.into(),
        }
    }
}

pub struct IconRequest<'a> {
    manager: &'a mut IconManager,
    svg_bytes: Vec<u8>,
}

impl<'a> IconRequest<'a> {
    /// rasterize to exactly `width×height`
    pub fn size(self, width: u32, height: u32) -> TextureHandle {
        // 1) hash svg+size → key
        let mut hasher = Sha256::new();
        hasher.update(&self.svg_bytes);
        hasher.update(&width.to_le_bytes());
        hasher.update(&height.to_le_bytes());
        let key = format!("{:x}", hasher.finalize());

        // 2) if already in-memory → return that
        if let Some(tex) = self.manager.cache.get(&key) {
            return tex.clone();
        }

        // 3) parse & compute scales
        let opts = usvg::Options::default();
        let tree = usvg::Tree::from_data(&self.svg_bytes, &opts).expect("invalid SVG data");
        let bb = tree.root().bounding_box();
        let orig_w = bb.width();
        let orig_h = bb.height();
        let sx = width as f32 / orig_w;
        let sy = height as f32 / orig_h;

        // 4) rasterize into a tiny-skia Pixmap
        let mut pixmap = Pixmap::new(width, height).expect("failed to allocate pixmap");
        let tx = SkiaTransform::from_scale(sx.into(), sy.into());
        render(&tree, tx, &mut pixmap.as_mut());

        // 5) write to disk
        let filename = format!("{}-{}x{}.png", key, width, height);
        let path = self.manager.output_dir.join(&filename);
        pixmap.save_png(&path).expect("failed to save icon PNG");

        // 6) upload to egui
        let image =
            ColorImage::from_rgba_unmultiplied([width as usize, height as usize], pixmap.data());
        let tex = self.manager.ctx.load_texture(
            &key,
            image,
            egui::TextureOptions {
                minification: TextureFilter::Nearest,
                magnification: TextureFilter::Nearest,
                ..Default::default()
            },
        );

        // 7) cache & return
        self.manager.cache.insert(key.clone(), tex.clone());
        tex
    }

    /// convenience for `.size(s,s)`
    pub fn square(self, size: u32) -> TextureHandle {
        self.size(size, size)
    }
}
