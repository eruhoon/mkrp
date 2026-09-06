use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::{ImageReader, RgbaImage};
use std::io::Cursor;
use tracing::debug;

pub struct TextureScaler {
    target_width: u32,
    target_height: u32,
    enable_downscale: bool,
}

impl TextureScaler {
    pub fn new(target_width: u32, target_height: u32, enable_downscale: bool) -> Self {
        Self {
            target_width,
            target_height,
            enable_downscale,
        }
    }

    /// Default scaler configured for Anbernic RG40XXH (640x480 resolution)
    pub fn for_rg40xxh() -> Self {
        Self::new(640, 480, true)
    }

    /// Load image from memory bytes (supports AVIF, PNG, JPEG) and downscale if it exceeds screen dimensions
    pub fn load_and_scale(&self, image_bytes: &[u8]) -> Result<RgbaImage, String> {
        // 1. Check if image is an AVIF container (ftypavif / ftypavis)
        let is_avif = image_bytes.len() >= 12
            && &image_bytes[4..8] == b"ftyp"
            && (&image_bytes[8..12] == b"avif" || &image_bytes[8..12] == b"avis" || &image_bytes[8..12] == b"mif1");

        let rgba = if is_avif {
            let buffer = zenavif::decode(image_bytes)
                .map_err(|e| format!("Failed to decode AVIF image: {:?}", e))?;
            let width = buffer.width();
            let height = buffer.height();
            let raw_bytes = buffer.copy_to_contiguous_bytes();
            let bpp = buffer.descriptor().bytes_per_pixel();

            if bpp == 4 {
                RgbaImage::from_raw(width, height, raw_bytes)
                    .ok_or_else(|| "Failed to construct RgbaImage from 4bpp AVIF".to_string())?
            } else if bpp == 3 {
                // RGB to RGBA conversion
                let mut rgba_vec = Vec::with_capacity((width * height * 4) as usize);
                for chunk in raw_bytes.chunks_exact(3) {
                    rgba_vec.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                }
                RgbaImage::from_raw(width, height, rgba_vec)
                    .ok_or_else(|| "Failed to construct RgbaImage from 3bpp AVIF".to_string())?
            } else {
                return Err(format!("Unsupported AVIF bytes_per_pixel: {}", bpp));
            }
        } else {
            let reader = ImageReader::new(Cursor::new(image_bytes))
                .with_guessed_format()
                .map_err(|e| format!("Failed to guess image format: {}", e))?;

            let img = reader
                .decode()
                .map_err(|e| format!("Failed to decode image: {}", e))?;

            img.to_rgba8()
        };

        let (orig_w, orig_h) = (rgba.width(), rgba.height());

        if !self.enable_downscale || (orig_w <= self.target_width && orig_h <= self.target_height) {
            return Ok(rgba);
        }

        // Calculate aspect ratio preserving scaled dimensions
        let scale_x = self.target_width as f32 / orig_w as f32;
        let scale_y = self.target_height as f32 / orig_h as f32;
        let scale = scale_x.min(scale_y);

        let new_w = ((orig_w as f32 * scale).round() as u32).max(1);
        let new_h = ((orig_h as f32 * scale).round() as u32).max(1);

        debug!(
            "Downscaling texture from {}x{} to {}x{} (Target {}x{}) to prevent OOM",
            orig_w, orig_h, new_w, new_h, self.target_width, self.target_height
        );

        // Fast SIMD-accelerated resize
        let src_image = Image::from_vec_u8(
            orig_w,
            orig_h,
            rgba.into_raw(),
            PixelType::U8x4,
        ).map_err(|e| format!("Source image creation failed: {:?}", e))?;

        let mut dst_image = Image::new(new_w, new_h, PixelType::U8x4);
        let mut resizer = Resizer::new();
        let options = ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Bilinear));
        resizer
            .resize(&src_image, &mut dst_image, &options)
            .map_err(|e| format!("Fast resize failed: {:?}", e))?;

        let dst_raw = dst_image.into_vec();
        RgbaImage::from_raw(new_w, new_h, dst_raw)
            .ok_or_else(|| "Failed to construct RgbaImage from scaled buffer".into())
    }
}
