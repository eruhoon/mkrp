use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use image::RgbaImage;
use mkrp_core::{DialogueLine, SceneMode, SceneState, ScriptExtractor};
use mkrp_render::{
    draw_choice_menu, draw_dialogue_box, draw_hud, FontRenderer, HandheldAction, SdlDisplay,
    TextureScaler,
};
use mkrp_rpa::Vfs;
use mkrp_rpyc::{RenpyVersion, RpycFile};

struct Config {
    game_dir: PathBuf,
    save_dir: PathBuf,
    target_width: u32,
    target_height: u32,
    enable_downscale: bool,
    fullscreen: bool,
}

impl Config {
    fn from_args() -> Self {
        let mut game_dir = PathBuf::from("game");
        let mut save_dir = PathBuf::from("saves");
        let mut target_width = 640;
        let mut target_height = 480;
        let mut enable_downscale = true;
        let mut fullscreen = true;

        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--game-dir" if i + 1 < args.len() => {
                    game_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
                "--save-dir" if i + 1 < args.len() => {
                    save_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
                "--target-width" if i + 1 < args.len() => {
                    if let Ok(w) = args[i + 1].parse() {
                        target_width = w;
                    }
                    i += 1;
                }
                "--target-height" if i + 1 < args.len() => {
                    if let Ok(h) = args[i + 1].parse() {
                        target_height = h;
                    }
                    i += 1;
                }
                "--no-downscale" => {
                    enable_downscale = false;
                }
                "--windowed" => {
                    fullscreen = false;
                }
                _ => {}
            }
            i += 1;
        }

        Self {
            game_dir,
            save_dir,
            target_width,
            target_height,
            enable_downscale,
            fullscreen,
        }
    }
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    println!("=========================================================");
    println!("🎮 mkrp - Lightweight Native Ren'Py Runner for PortMaster");
    println!("   Version: v{} (Rust Native Engine)", env!("CARGO_PKG_VERSION"));
    println!("   Target Devices: RG40XXH, RG VITA PRO, RG DS (Linux ARM64)");
    println!("=========================================================");

    let config = Config::from_args();

    info!(
        "Configuration: game_dir={:?}, save_dir={:?}, display={}x{}, downscale={}, fullscreen={}",
        config.game_dir,
        config.save_dir,
        config.target_width,
        config.target_height,
        config.enable_downscale,
        config.fullscreen
    );

    // 1. Initialize Virtual File System
    let mut vfs = Vfs::new();

    if !config.game_dir.exists() {
        warn!(
            "Game directory {:?} does not exist yet. Please place Ren'Py game files into {:?}",
            config.game_dir, config.game_dir
        );
        return;
    }

    info!("Scanning game assets in {:?}", config.game_dir);
    if let Err(e) = vfs.mount_game_directory(&config.game_dir) {
        warn!("VFS mount warning: {}", e);
    }

    let all_files = vfs.list_all_files();
    info!("VFS ready: {} unique files indexed.", all_files.len());

    // Pre-index all image base names to exact VFS paths (e.g. "d1_1_1" -> "images/cg07_pc/d1_1_1.avif")
    let mut image_base_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut known_images: std::collections::HashSet<String> = std::collections::HashSet::new();

    for file_path in &all_files {
        let lower = file_path.to_lowercase();
        if lower.ends_with(".avif") || lower.ends_with(".png") || lower.ends_with(".jpg") {
            if let Some(filename) = std::path::Path::new(file_path).file_stem() {
                let base_name = filename.to_string_lossy().to_lowercase();
                image_base_map.insert(base_name.clone(), file_path.clone());
                known_images.insert(base_name);
            }
        }
    }
    info!("Pre-indexed {} distinct CG/background image base names.", image_base_map.len());

    // 2. Detect Ren'Py Version (Ren'Py 8 vs Ren'Py 7)
    let rpyc_files: Vec<&String> = all_files
        .iter()
        .filter(|f| f.ends_with(".rpyc"))
        .collect();

    info!("Found {} compiled Ren'Py script (.rpyc) files.", rpyc_files.len());

    let mut detected_version = RenpyVersion::Unknown;
    let mut decompressed_scripts: Vec<Vec<u8>> = Vec::new();

    // Sort rpyc files: Prioritize start/intro story (d1_1, prologue, start), then Korean events, avoiding patches
    let mut sorted_rpyc_files = rpyc_files.clone();
    sorted_rpyc_files.sort_by_key(|f| {
        let lower = f.to_lowercase();
        if lower.contains("d1_1") {
            0
        } else if lower.contains("start") || lower.contains("prologue") {
            1
        } else if lower.contains("tl/korean") && lower.contains("events") {
            2
        } else if lower.contains("tl/korean") {
            3
        } else if lower.contains("events") || lower.contains("story") {
            4
        } else if lower.contains("patches") {
            6
        } else {
            5
        }
    });

    for rpyc_path in &sorted_rpyc_files {
        if let Ok(bytes) = vfs.read(rpyc_path) {
            if let Ok(rpyc) = RpycFile::from_bytes(&bytes) {
                if detected_version == RenpyVersion::Unknown {
                    detected_version = rpyc.version();
                    info!(
                        "Analyzed {}: detected engine target {:?}",
                        rpyc_path, detected_version
                    );
                }
                // Cache script payload for dialogue parsing
                if rpyc_path.contains("events") || rpyc_path.contains("story") || rpyc_path.contains("tl/korean") {
                    decompressed_scripts.push(rpyc.decompressed_payload().to_vec());
                }
            }
        }
    }

    match detected_version {
        RenpyVersion::Renpy8 => {
            println!(">> Engine Mode: Ren'Py 8.x (Python 3 Modern AST)");
        }
        RenpyVersion::Renpy7 => {
            println!(">> Engine Mode: Ren'Py 7.x / 6.x (Python 2 Legacy AST)");
        }
        RenpyVersion::Unknown => {
            println!(">> Engine Mode: Defaulting to Ren'Py 8.x");
        }
    }

    // 3. Extract Dialogue Lines with associated image tags
    let mut script_lines = Vec::new();
    for payload in &decompressed_scripts {
        let lines = ScriptExtractor::extract_dialogue_from_payload(payload, &known_images);
        if !lines.is_empty() {
            script_lines.extend(lines);
        }
    }

    if script_lines.is_empty() {
        info!("Creating initial welcome dialogue flow.");
        script_lines.push(DialogueLine::new(
            Some("mkrp Engine".into()),
            "Welcome to mkrp Native Runner for PortMaster handheld consoles!".into(),
        ));
        script_lines.push(DialogueLine::new(
            Some("Guide".into()),
            "Press (A) button to advance dialogue, (B) button to rollback history.".into(),
        ));
        script_lines.push(DialogueLine::new(
            Some("Guide".into()),
            "Press (X) to toggle dialogue window, (Start / Select) to exit.".into(),
        ));
    }

    let mut scene_state = SceneState::new();
    scene_state.load_lines(script_lines);

    // 4. Initialize SDL2 Display
    println!("=========================================================");
    println!("Initializing SDL2 display window on PortMaster...");
    println!("=========================================================");

    let mut display = match SdlDisplay::new(
        "mkrp - Ren'Py Runner",
        config.target_width,
        config.target_height,
        config.fullscreen,
    ) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to initialize SDL2 display: {}", e);
            warn!("Running in headless diagnostic mode for 3 seconds.");
            std::thread::sleep(Duration::from_secs(3));
            return;
        }
    };

    let screen_w = display.width();
    let screen_h = display.height();
    info!("Active display resolution: {}x{}", screen_w, screen_h);

    // 5. Initialize Scaler using actual hardware screen resolution
    let scaler = TextureScaler::new(
        screen_w,
        screen_h,
        config.enable_downscale,
    );
    info!(
        "Texture Scaler active: targets hardware {}x{} for VRAM protection.",
        screen_w, screen_h
    );

    // 6. Find and load game TTF / OTF Font for dialogue rendering
    let mut font_renderer: Option<FontRenderer> = None;
    let font_candidates = [
        "tl/korean.ttf",
        "fonts/SourceHanSansJP-Medium.ttf",
        "NanumGothic.ttf",
        "NotoSansCJK-Regular.ttc",
        "NotoSansKR-Regular.otf",
        "NanumBarunGothic.ttf",
        "gui/font.ttf",
        "font.ttf",
    ];

    for font_name in &font_candidates {
        if let Ok(bytes) = vfs.read(font_name) {
            if let Ok(fr) = FontRenderer::from_bytes(&bytes) {
                info!("Loaded game font: {}", font_name);
                font_renderer = Some(fr);
                break;
            }
        }
    }

    // If game font not found in VFS, search all indexed files for any .ttf or .otf
    if font_renderer.is_none() {
        for f in &all_files {
            if f.ends_with(".ttf") || f.ends_with(".otf") {
                if let Ok(bytes) = vfs.read(f) {
                    if let Ok(fr) = FontRenderer::from_bytes(&bytes) {
                        info!("Loaded discovered font: {}", f);
                        font_renderer = Some(fr);
                        break;
                    }
                }
            }
        }
    }

    // 7. Find Splash / Title Background Image
    let possible_splash_names = [
        "presplash_background.jpg",
        "presplash_foreground.png",
        "presplash.png",
        "presplash.jpg",
        "splash.png",
        "gui/main_menu.png",
        "gui/window_icon.png",
    ];

    let mut background_image = None;
    for splash_name in &possible_splash_names {
        if let Ok(bytes) = vfs.read(splash_name) {
            if let Ok(scaled) = scaler.load_and_scale(&bytes) {
                info!(
                    "Loaded background image '{}' ({}x{}) to fit screen {}x{}",
                    splash_name,
                    scaled.width(),
                    scaled.height(),
                    screen_w,
                    screen_h
                );
                background_image = Some(scaled);
                break;
            }
        }
    }

    let default_bg = background_image.unwrap_or_else(|| {
        image::ImageBuffer::from_pixel(
            screen_w,
            screen_h,
            image::Rgba([25, 25, 40, 255]),
        )
    });

    // Flush any pending launch button events
    display.delay(100);
    display.flush_events();

    info!("mkrp Interactive Dialogue Loop started! Gamepad mapped: A=Next, B=Rollback, X=Toggle UI, Y=Menu, L1=Auto, R1=Skip, Start=Quit.");

    // Pre-cache background images mapped by scene tag
    let mut bg_cache: HashMap<String, RgbaImage> = HashMap::new();
    let mut last_bg_tag: Option<String> = None;
    let mut current_bg_image: RgbaImage = default_bg.clone();

    // 8. Main Interactive Game Loop
    let mut frame_count: u64 = 0;
    let mut dirty = true;
    let mut composite_frame = default_bg.clone();
    let mut last_auto_advance = Instant::now();

    loop {
        // Auto / Skip timer checks
        if scene_state.skip_mode {
            if scene_state.advance() {
                dirty = true;
            } else {
                scene_state.skip_mode = false;
            }
        } else if scene_state.auto_mode && last_auto_advance.elapsed() >= Duration::from_millis(1500) {
            if scene_state.advance() {
                last_auto_advance = Instant::now();
                dirty = true;
            } else {
                scene_state.auto_mode = false;
            }
        }

        // Poll handheld actions
        if let Some(action) = display.poll_action() {
            match action {
                HandheldAction::Quit => {
                    info!("Exit triggered by user.");
                    break;
                }
                HandheldAction::Next => {
                    if scene_state.mode == SceneMode::ChoiceMenu {
                        // Confirm choice option
                        info!("Choice selected: #{}", scene_state.selected_choice);
                        scene_state.advance();
                    } else if scene_state.advance() {
                        info!(
                            "Advanced to dialogue #{}: {:?}",
                            scene_state.current_index,
                            scene_state.current_line().map(|l| &l.text)
                        );
                    } else {
                        info!("Reached end of loaded script, looping back to start.");
                        scene_state.current_index = 0;
                    }
                    last_auto_advance = Instant::now();
                    dirty = true;
                }
                HandheldAction::Rollback => {
                    if scene_state.rollback() {
                        info!(
                            "Rollback to dialogue #{}: {:?}",
                            scene_state.current_index,
                            scene_state.current_line().map(|l| &l.text)
                        );
                        dirty = true;
                    }
                }
                HandheldAction::HideUi => {
                    scene_state.toggle_ui();
                    info!("Toggled UI visibility: {}", scene_state.ui_visible);
                    dirty = true;
                }
                HandheldAction::Menu => {
                    scene_state.toggle_auto();
                    info!("Toggled Auto mode: {}", scene_state.auto_mode);
                    last_auto_advance = Instant::now();
                    dirty = true;
                }
                HandheldAction::Skip => {
                    scene_state.toggle_skip();
                    info!("Toggled Skip mode: {}", scene_state.skip_mode);
                    dirty = true;
                }
                HandheldAction::Up => {
                    if scene_state.mode == SceneMode::ChoiceMenu {
                        scene_state.choice_up();
                        dirty = true;
                    }
                }
                HandheldAction::Down => {
                    if scene_state.mode == SceneMode::ChoiceMenu {
                        scene_state.choice_down();
                        dirty = true;
                    }
                }
            }
        }

        // 9. Check if Scene / Background changed
        if let Some(line) = scene_state.current_line() {
            if let Some(bg_tag) = &line.background {
                if last_bg_tag.as_ref() != Some(bg_tag) {
                    last_bg_tag = Some(bg_tag.clone());

                    // Look up or load background image from VFS
                    if let Some(cached) = bg_cache.get(bg_tag) {
                        current_bg_image = cached.clone();
                        dirty = true;
                    } else {
                        let bg_pattern = bg_tag.to_lowercase();
                        // 1. Direct O(1) lookup via pre-indexed base name map
                        let found_file = image_base_map.get(&bg_pattern).cloned().or_else(|| {
                            // 2. Fallback: fuzzy substring match in all files
                            all_files.iter().find(|f| {
                                let f_lower = f.to_lowercase();
                                (f_lower.ends_with(".avif") || f_lower.ends_with(".png") || f_lower.ends_with(".jpg"))
                                    && f_lower.contains(&bg_pattern)
                            }).cloned()
                        });

                        if let Some(image_path) = found_file {
                            if let Ok(bytes) = vfs.read(&image_path) {
                                if let Ok(scaled) = scaler.load_and_scale(&bytes) {
                                    info!(
                                        "Loaded scene background: {} -> {} ({}x{})",
                                        bg_tag, image_path, scaled.width(), scaled.height()
                                    );
                                    bg_cache.insert(bg_tag.clone(), scaled.clone());
                                    current_bg_image = scaled;
                                    dirty = true;
                                }
                            }
                        } else {
                            warn!("Scene background / CG '{}' not found in VFS archives.", bg_tag);
                        }
                    }
                }
            }
        }

        // Re-compose frame only when state changes
        if dirty {
            composite_frame = current_bg_image.clone();

            if scene_state.ui_visible {
                if scene_state.mode == SceneMode::ChoiceMenu {
                    if let Some(line) = scene_state.current_line() {
                        let options: Vec<String> = line.choices.iter().map(|c| c.text.clone()).collect();
                        draw_choice_menu(
                            &mut composite_frame,
                            font_renderer.as_ref(),
                            &options,
                            scene_state.selected_choice,
                        );
                    }
                } else if let Some(line) = scene_state.current_line() {
                    draw_dialogue_box(
                        &mut composite_frame,
                        font_renderer.as_ref(),
                        line.speaker.as_deref(),
                        &line.text,
                        true,
                    );
                }

                // Draw Auto / Skip HUD
                draw_hud(
                    &mut composite_frame,
                    font_renderer.as_ref(),
                    scene_state.auto_mode,
                    scene_state.skip_mode,
                );
            }

            dirty = false;
        }

        // Present to SDL2 display
        if let Err(e) = display.render_image(&composite_frame) {
            warn!("Render frame warning: {}", e);
        }

        display.delay(33); // ~30 FPS Handheld VSYNC
        frame_count += 1;

        if frame_count % 300 == 0 {
            info!("mkrp running smoothly ({} frames presented).", frame_count);
        }
    }

    info!("Shutting down mkrp cleanly. Total frames: {}", frame_count);
    println!("=========================================================");
    println!("mkrp engine finished cleanly.");
    println!("=========================================================");
}
