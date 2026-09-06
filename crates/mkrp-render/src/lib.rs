pub mod scaler;
pub mod sdl;
pub mod text;

pub use scaler::TextureScaler;
pub use sdl::{HandheldAction, SdlDisplay, SdlLib};
pub use text::{draw_choice_menu, draw_dialogue_box, draw_hud, FontRenderer};

pub fn info() -> &'static str {
    "mkrp-render v0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};
    use std::io::Cursor;

    #[test]
    fn test_texture_downscaling_rg40xxh() {
        // Create 1920x1080 synthetic image
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(1920, 1080, Rgba([255, 0, 0, 255]));
        let mut png_bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png).unwrap();

        // Downscale for RG40XXH (640x480)
        let scaler = TextureScaler::for_rg40xxh();
        let scaled = scaler.load_and_scale(&png_bytes).expect("Failed to scale");

        // 1920x1080 scaled down to fit 640x480 should be 640x360
        assert_eq!(scaled.width(), 640);
        assert_eq!(scaled.height(), 360);
        assert_eq!(scaled.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
    }
}
