use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};

pub struct FontRenderer {
    font: Font,
}

impl FontRenderer {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let font = Font::from_bytes(bytes, FontSettings::default())
            .map_err(|e| format!("Failed to parse TTF font: {}", e))?;
        Ok(Self { font })
    }

    /// Render a single line or multi-line text into an RGBA image buffer with alpha blending.
    pub fn draw_text(
        &self,
        target: &mut RgbaImage,
        text: &str,
        start_x: i32,
        start_y: i32,
        size_px: f32,
        color: [u8; 4],
        max_width: Option<u32>,
    ) {
        let mut cur_x = start_x as f32;
        let mut cur_y = start_y as f32;
        let line_height = size_px * 1.35;

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = start_x as f32;
                cur_y += line_height;
                continue;
            }

            // Word-wrap check
            if let Some(max_w) = max_width {
                if (cur_x - start_x as f32) > max_w as f32 && ch.is_whitespace() {
                    cur_x = start_x as f32;
                    cur_y += line_height;
                    continue;
                }
            }

            let (metrics, bitmap) = self.font.rasterize(ch, size_px);

            let gx = cur_x as i32 + metrics.xmin;
            let gy = cur_y as i32 + (size_px as i32 - metrics.ymin - metrics.height as i32);

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let alpha_val = bitmap[row * metrics.width + col];
                    if alpha_val == 0 {
                        continue;
                    }

                    let px = gx + col as i32;
                    let py = gy + row as i32;

                    if px >= 0 && px < target.width() as i32 && py >= 0 && py < target.height() as i32 {
                        let base = target.get_pixel_mut(px as u32, py as u32);
                        // Alpha blend
                        let src_a = (color[3] as u32 * alpha_val as u32) / 255;
                        let inv_a = 255 - src_a;

                        let r = (color[0] as u32 * src_a + base[0] as u32 * inv_a) / 255;
                        let g = (color[1] as u32 * src_a + base[1] as u32 * inv_a) / 255;
                        let b = (color[2] as u32 * src_a + base[2] as u32 * inv_a) / 255;
                        let a = (src_a + (base[3] as u32 * inv_a) / 255).min(255);

                        *base = Rgba([r as u8, g as u8, b as u8, a as u8]);
                    }
                }
            }

            cur_x += metrics.advance_width;
        }
    }
}

/// Draw dialogue box overlay onto the scene frame.
pub fn draw_dialogue_box(
    frame: &mut RgbaImage,
    font: Option<&FontRenderer>,
    speaker: Option<&str>,
    dialogue: &str,
    show_indicator: bool,
) {
    let screen_w = frame.width();
    let screen_h = frame.height();

    // Box dimensions
    let box_margin_x = (screen_w as f32 * 0.05) as u32;
    let box_w = screen_w.saturating_sub(box_margin_x * 2);
    let box_h = (screen_h as f32 * 0.28) as u32;
    let box_y = screen_h.saturating_sub(box_h + (screen_h as f32 * 0.04) as u32);
    let box_x = box_margin_x;

    // 1. Draw semi-transparent dark background for dialogue box
    for y in box_y..box_y + box_h {
        for x in box_x..box_x + box_w {
            if x < screen_w && y < screen_h {
                let pixel = frame.get_pixel_mut(x, y);
                // 70% dark tint
                pixel[0] = ((pixel[0] as u32 * 30 + 15 * 70) / 100) as u8;
                pixel[1] = ((pixel[1] as u32 * 30 + 18 * 70) / 100) as u8;
                pixel[2] = ((pixel[2] as u32 * 30 + 25 * 70) / 100) as u8;
            }
        }
    }

    // 2. Draw subtle border
    let border_color = [100, 120, 160, 200];
    for x in box_x..box_x + box_w {
        if x < screen_w && box_y < screen_h {
            *frame.get_pixel_mut(x, box_y) = Rgba(border_color);
            *frame.get_pixel_mut(x, (box_y + box_h).min(screen_h - 1)) = Rgba(border_color);
        }
    }
    for y in box_y..box_y + box_h {
        if box_x < screen_w && y < screen_h {
            *frame.get_pixel_mut(box_x, y) = Rgba(border_color);
            *frame.get_pixel_mut((box_x + box_w).min(screen_w - 1), y) = Rgba(border_color);
        }
    }

    if let Some(renderer) = font {
        // 3. Draw Speaker Name
        let font_size_speaker = (screen_h as f32 * 0.05).max(18.0);
        let font_size_dialogue = (screen_h as f32 * 0.045).max(16.0);

        if let Some(name) = speaker {
            renderer.draw_text(
                frame,
                name,
                (box_x + 25) as i32,
                (box_y.saturating_sub(font_size_speaker as u32 + 8)) as i32,
                font_size_speaker,
                [255, 220, 120, 255], // Gold / Yellow
                Some(box_w),
            );
        }

        // 4. Draw Dialogue Text
        renderer.draw_text(
            frame,
            dialogue,
            (box_x + 25) as i32,
            (box_y + 20) as i32,
            font_size_dialogue,
            [255, 255, 255, 255], // White
            Some(box_w.saturating_sub(50)),
        );

        // 5. Draw prompt indicator (▼)
        if show_indicator {
            renderer.draw_text(
                frame,
                "▼",
                (box_x + box_w - 35) as i32,
                (box_y + box_h - 30) as i32,
                font_size_dialogue,
                [255, 200, 80, 255],
                None,
            );
        }
    }
}

/// Draw choice selection menu options overlaid on the center screen.
pub fn draw_choice_menu(
    frame: &mut RgbaImage,
    font: Option<&FontRenderer>,
    options: &[String],
    selected_idx: usize,
) {
    let screen_w = frame.width();
    let screen_h = frame.height();
    let font_size = (screen_h as f32 * 0.045).max(18.0);
    let item_h = (font_size * 2.2) as u32;
    let spacing = 12u32;
    let total_h = options.len() as u32 * (item_h + spacing);
    let start_y = (screen_h.saturating_sub(total_h)) / 2;
    let box_w = (screen_w as f32 * 0.70) as u32;
    let box_x = (screen_w.saturating_sub(box_w)) / 2;

    for (idx, opt_text) in options.iter().enumerate() {
        let cur_y = start_y + idx as u32 * (item_h + spacing);
        let is_selected = idx == selected_idx;

        // Background box
        for y in cur_y..cur_y + item_h {
            for x in box_x..box_x + box_w {
                if x < screen_w && y < screen_h {
                    let pixel = frame.get_pixel_mut(x, y);
                    if is_selected {
                        // Highlighted golden/blue tint
                        pixel[0] = ((pixel[0] as u32 * 20 + 200 * 80) / 100) as u8;
                        pixel[1] = ((pixel[1] as u32 * 20 + 160 * 80) / 100) as u8;
                        pixel[2] = ((pixel[2] as u32 * 20 + 50 * 80) / 100) as u8;
                    } else {
                        // Dark translucent tint
                        pixel[0] = ((pixel[0] as u32 * 30 + 15 * 70) / 100) as u8;
                        pixel[1] = ((pixel[1] as u32 * 30 + 18 * 70) / 100) as u8;
                        pixel[2] = ((pixel[2] as u32 * 30 + 25 * 70) / 100) as u8;
                    }
                }
            }
        }

        // Border
        let border_color = if is_selected {
            [255, 220, 100, 255]
        } else {
            [120, 140, 180, 180]
        };
        for x in box_x..box_x + box_w {
            if x < screen_w && cur_y < screen_h {
                *frame.get_pixel_mut(x, cur_y) = Rgba(border_color);
                *frame.get_pixel_mut(x, (cur_y + item_h).min(screen_h - 1)) = Rgba(border_color);
            }
        }
        for y in cur_y..cur_y + item_h {
            if box_x < screen_w && y < screen_h {
                *frame.get_pixel_mut(box_x, y) = Rgba(border_color);
                *frame.get_pixel_mut((box_x + box_w).min(screen_w - 1), y) = Rgba(border_color);
            }
        }

        // Text
        if let Some(renderer) = font {
            let text_color = if is_selected {
                [20, 20, 20, 255] // Dark text on bright highlight
            } else {
                [245, 245, 245, 255] // White text
            };
            let text_y = cur_y as i32 + ((item_h as f32 - font_size) / 2.0) as i32;
            let text_x = (box_x + 30) as i32;

            renderer.draw_text(
                frame,
                opt_text,
                text_x,
                text_y,
                font_size,
                text_color,
                Some(box_w.saturating_sub(60)),
            );
        }
    }
}

/// Draw in-game status HUD (Auto indicator, Skip indicator, HUD status).
pub fn draw_hud(
    frame: &mut RgbaImage,
    font: Option<&FontRenderer>,
    auto_mode: bool,
    skip_mode: bool,
) {
    if !auto_mode && !skip_mode {
        return;
    }

    let screen_w = frame.width();
    let font_size = 20.0;
    let label = if skip_mode {
        ">> SKIP >>"
    } else {
        ">> AUTO >>"
    };

    if let Some(renderer) = font {
        renderer.draw_text(
            frame,
            label,
            (screen_w.saturating_sub(160)) as i32,
            24,
            font_size,
            [255, 200, 50, 255],
            None,
        );
    }
}
