use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::sync::Arc;
use image::RgbaImage;
use libloading::{Library, Symbol};
use tracing::info;

pub const SDL_INIT_VIDEO: u32 = 0x00000020;
pub const SDL_INIT_GAMECONTROLLER: u32 = 0x00002000;
pub const SDL_INIT_JOYSTICK: u32 = 0x00000200;
pub const SDL_WINDOWPOS_CENTERED: c_int = 0x2FFF0000;
pub const SDL_WINDOW_FULLSCREEN_DESKTOP: u32 = 0x00001001;
pub const SDL_WINDOW_SHOWN: u32 = 0x00000004;
pub const SDL_RENDERER_ACCELERATED: u32 = 0x00000002;
pub const SDL_RENDERER_PRESENTVSYNC: u32 = 0x00000004;
pub const SDL_PIXELFORMAT_RGBA32: u32 = 0x16462004;
pub const SDL_PIXELFORMAT_ABGR8888: u32 = 0x16762004;
pub const SDL_TEXTUREACCESS_STREAMING: c_int = 1;

pub const SDL_QUIT: u32 = 0x100;
pub const SDL_KEYDOWN: u32 = 0x300;
pub const SDL_KEYUP: u32 = 0x301;
pub const SDL_MOUSEBUTTONDOWN: u32 = 0x401;
pub const SDL_MOUSEBUTTONUP: u32 = 0x402;
pub const SDL_JOYBUTTONDOWN: u32 = 0x603;
pub const SDL_CONTROLLERBUTTONDOWN: u32 = 0x653;
pub const SDL_FINGERDOWN: u32 = 0x700;
pub const SDL_FINGERUP: u32 = 0x701;
pub const SDL_FINGERMOTION: u32 = 0x702;

pub const SDLK_ESCAPE: i32 = 27;
pub const SDLK_RETURN: i32 = 13;
pub const SDLK_SPACE: i32 = 32;
pub const SDLK_PAGEUP: i32 = 1073741899;
pub const SDLK_PAGEDOWN: i32 = 1073741902;
pub const SDLK_UP: i32 = 1073741906;
pub const SDLK_DOWN: i32 = 1073741905;
pub const SDLK_LEFT: i32 = 1073741904;
pub const SDLK_RIGHT: i32 = 1073741903;
pub const SDLK_TAB: i32 = 9;
pub const SDLK_Q: i32 = 113;
pub const SDLK_H: i32 = 104;
pub const SDLK_M: i32 = 109;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandheldAction {
    Next,      // A button or Enter or Space
    Rollback,  // B button or PageUp
    Menu,      // Y button or 'm' or Escape
    HideUi,    // X button or 'h'
    Skip,      // L1 or Tab
    Up,
    Down,
    Quit,      // ESC or Q or Quit event
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SDL_Rect {
    pub x: c_int,
    pub y: c_int,
    pub w: c_int,
    pub h: c_int,
}

#[repr(C)]
pub struct SDL_Event {
    pub type_: u32,
    pub padding: [u8; 56],
}

type FnInit = unsafe extern "C" fn(flags: u32) -> c_int;
type FnQuit = unsafe extern "C" fn();
type FnGetError = unsafe extern "C" fn() -> *const c_char;
type FnCreateWindow = unsafe extern "C" fn(
    title: *const c_char,
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
    flags: u32,
) -> *mut c_void;
type FnDestroyWindow = unsafe extern "C" fn(window: *mut c_void);
type FnCreateRenderer = unsafe extern "C" fn(
    window: *mut c_void,
    index: c_int,
    flags: u32,
) -> *mut c_void;
type FnDestroyRenderer = unsafe extern "C" fn(renderer: *mut c_void);
type FnCreateTexture = unsafe extern "C" fn(
    renderer: *mut c_void,
    format: u32,
    access: c_int,
    w: c_int,
    h: c_int,
) -> *mut c_void;
type FnDestroyTexture = unsafe extern "C" fn(texture: *mut c_void);
type FnUpdateTexture = unsafe extern "C" fn(
    texture: *mut c_void,
    rect: *const SDL_Rect,
    pixels: *const c_void,
    pitch: c_int,
) -> c_int;
type FnSetRenderDrawColor = unsafe extern "C" fn(
    renderer: *mut c_void,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) -> c_int;
type FnRenderClear = unsafe extern "C" fn(renderer: *mut c_void) -> c_int;
type FnRenderCopy = unsafe extern "C" fn(
    renderer: *mut c_void,
    texture: *mut c_void,
    srcrect: *const SDL_Rect,
    dstrect: *const SDL_Rect,
) -> c_int;
type FnRenderPresent = unsafe extern "C" fn(renderer: *mut c_void);
#[repr(C)]
#[derive(Default, Debug)]
pub struct SDL_DisplayMode {
    pub format: u32,
    pub w: c_int,
    pub h: c_int,
    pub refresh_rate: c_int,
    pub driverdata: *mut c_void,
}

type FnGetCurrentDisplayMode = unsafe extern "C" fn(display_index: c_int, mode: *mut SDL_DisplayMode) -> c_int;
type FnPollEvent = unsafe extern "C" fn(event: *mut SDL_Event) -> c_int;
type FnDelay = unsafe extern "C" fn(ms: u32);
type FnShowCursor = unsafe extern "C" fn(toggle: c_int) -> c_int;

#[derive(Clone)]
pub struct SdlLib {
    _lib: Arc<Library>,
    pub init: FnInit,
    pub quit: FnQuit,
    pub get_error: FnGetError,
    pub create_window: FnCreateWindow,
    pub destroy_window: FnDestroyWindow,
    pub create_renderer: FnCreateRenderer,
    pub destroy_renderer: FnDestroyRenderer,
    pub create_texture: FnCreateTexture,
    pub destroy_texture: FnDestroyTexture,
    pub update_texture: FnUpdateTexture,
    pub set_render_draw_color: FnSetRenderDrawColor,
    pub render_clear: FnRenderClear,
    pub render_copy: FnRenderCopy,
    pub render_present: FnRenderPresent,
    pub get_current_display_mode: FnGetCurrentDisplayMode,
    pub poll_event: FnPollEvent,
    pub delay: FnDelay,
    pub show_cursor: FnShowCursor,
}

impl SdlLib {
    pub fn load() -> Result<Self, String> {
        let possible_names = [
            "libSDL2-2.0.so.0",
            "libSDL2.so",
            "/usr/lib/aarch64-linux-gnu/libSDL2-2.0.so.0",
            "/usr/lib/aarch64-linux-gnu/libSDL2.so",
            "/roms/ports/PortMaster/lib/libSDL2-2.0.so.0",
            "SDL2.dll",
        ];

        let mut loaded_lib = None;
        for name in &possible_names {
            if let Ok(lib) = unsafe { Library::new(name) } {
                info!("Successfully loaded SDL2 dynamically from: {}", name);
                loaded_lib = Some(lib);
                break;
            }
        }

        let lib = loaded_lib.ok_or_else(|| {
            "Failed to load SDL2 library (tried standard paths and names)".to_string()
        })?;

        unsafe {
            let init: Symbol<FnInit> = lib
                .get(b"SDL_Init\0")
                .map_err(|e| format!("SDL_Init: {}", e))?;
            let quit: Symbol<FnQuit> = lib
                .get(b"SDL_Quit\0")
                .map_err(|e| format!("SDL_Quit: {}", e))?;
            let get_error: Symbol<FnGetError> = lib
                .get(b"SDL_GetError\0")
                .map_err(|e| format!("SDL_GetError: {}", e))?;
            let create_window: Symbol<FnCreateWindow> = lib
                .get(b"SDL_CreateWindow\0")
                .map_err(|e| format!("SDL_CreateWindow: {}", e))?;
            let destroy_window: Symbol<FnDestroyWindow> = lib
                .get(b"SDL_DestroyWindow\0")
                .map_err(|e| format!("SDL_DestroyWindow: {}", e))?;
            let create_renderer: Symbol<FnCreateRenderer> = lib
                .get(b"SDL_CreateRenderer\0")
                .map_err(|e| format!("SDL_CreateRenderer: {}", e))?;
            let destroy_renderer: Symbol<FnDestroyRenderer> = lib
                .get(b"SDL_DestroyRenderer\0")
                .map_err(|e| format!("SDL_DestroyRenderer: {}", e))?;
            let create_texture: Symbol<FnCreateTexture> = lib
                .get(b"SDL_CreateTexture\0")
                .map_err(|e| format!("SDL_CreateTexture: {}", e))?;
            let destroy_texture: Symbol<FnDestroyTexture> = lib
                .get(b"SDL_DestroyTexture\0")
                .map_err(|e| format!("SDL_DestroyTexture: {}", e))?;
            let update_texture: Symbol<FnUpdateTexture> = lib
                .get(b"SDL_UpdateTexture\0")
                .map_err(|e| format!("SDL_UpdateTexture: {}", e))?;
            let set_render_draw_color: Symbol<FnSetRenderDrawColor> = lib
                .get(b"SDL_SetRenderDrawColor\0")
                .map_err(|e| format!("SDL_SetRenderDrawColor: {}", e))?;
            let render_clear: Symbol<FnRenderClear> = lib
                .get(b"SDL_RenderClear\0")
                .map_err(|e| format!("SDL_RenderClear: {}", e))?;
            let render_copy: Symbol<FnRenderCopy> = lib
                .get(b"SDL_RenderCopy\0")
                .map_err(|e| format!("SDL_RenderCopy: {}", e))?;
            let render_present: Symbol<FnRenderPresent> = lib
                .get(b"SDL_RenderPresent\0")
                .map_err(|e| format!("SDL_RenderPresent: {}", e))?;
            let get_current_display_mode: Symbol<FnGetCurrentDisplayMode> = lib
                .get(b"SDL_GetCurrentDisplayMode\0")
                .map_err(|e| format!("SDL_GetCurrentDisplayMode: {}", e))?;
            let poll_event: Symbol<FnPollEvent> = lib
                .get(b"SDL_PollEvent\0")
                .map_err(|e| format!("SDL_PollEvent: {}", e))?;
            let delay: Symbol<FnDelay> = lib
                .get(b"SDL_Delay\0")
                .map_err(|e| format!("SDL_Delay: {}", e))?;
            let show_cursor: Symbol<FnShowCursor> = lib
                .get(b"SDL_ShowCursor\0")
                .map_err(|e| format!("SDL_ShowCursor: {}", e))?;

            Ok(Self {
                init: *init,
                quit: *quit,
                get_error: *get_error,
                create_window: *create_window,
                destroy_window: *destroy_window,
                create_renderer: *create_renderer,
                destroy_renderer: *destroy_renderer,
                create_texture: *create_texture,
                destroy_texture: *destroy_texture,
                update_texture: *update_texture,
                set_render_draw_color: *set_render_draw_color,
                render_clear: *render_clear,
                render_copy: *render_copy,
                render_present: *render_present,
                get_current_display_mode: *get_current_display_mode,
                poll_event: *poll_event,
                delay: *delay,
                show_cursor: *show_cursor,
                _lib: Arc::new(lib),
            })
        }
    }

    pub fn last_error(&self) -> String {
        unsafe {
            let ptr = (self.get_error)();
            if ptr.is_null() {
                "No SDL error".to_string()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().to_string()
            }
        }
    }
}

pub struct SdlDisplay {
    sdl: SdlLib,
    window: *mut c_void,
    renderer: *mut c_void,
    target_width: u32,
    target_height: u32,
}

impl SdlDisplay {
    pub fn new(title: &str, mut width: u32, mut height: u32, fullscreen: bool) -> Result<Self, String> {
        let sdl = SdlLib::load()?;

        unsafe {
            let res = (sdl.init)(SDL_INIT_VIDEO | SDL_INIT_GAMECONTROLLER | SDL_INIT_JOYSTICK);
            if res != 0 {
                return Err(format!("SDL_Init failed: {}", sdl.last_error()));
            }

            // 1. Hardware native display mode query
            let mut dm = SDL_DisplayMode::default();
            if (sdl.get_current_display_mode)(0, &mut dm) == 0 && dm.w > 0 && dm.h > 0 {
                info!(
                    "Hardware native screen detected: {}x{} @ {}Hz",
                    dm.w, dm.h, dm.refresh_rate
                );
                width = dm.w as u32;
                height = dm.h as u32;
            } else if let (Ok(pw), Ok(ph)) = (
                std::env::var("DISPLAY_WIDTH"),
                std::env::var("DISPLAY_HEIGHT"),
            ) {
                if let (Ok(w), Ok(h)) = (pw.parse::<u32>(), ph.parse::<u32>()) {
                    info!("Using PortMaster environment resolution: {}x{}", w, h);
                    width = w;
                    height = h;
                }
            }

            let c_title = CString::new(title).unwrap_or_default();
            let mut flags = SDL_WINDOW_SHOWN;
            if fullscreen {
                flags |= SDL_WINDOW_FULLSCREEN_DESKTOP;
            }

            let window = (sdl.create_window)(
                c_title.as_ptr(),
                SDL_WINDOWPOS_CENTERED,
                SDL_WINDOWPOS_CENTERED,
                width as c_int,
                height as c_int,
                flags,
            );

            if window.is_null() {
                return Err(format!("SDL_CreateWindow failed: {}", sdl.last_error()));
            }

            let renderer = (sdl.create_renderer)(
                window,
                -1,
                SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC,
            );

            if renderer.is_null() {
                // Fallback to software renderer if accelerated fails
                let fallback = (sdl.create_renderer)(window, -1, 0);
                if fallback.is_null() {
                    (sdl.destroy_window)(window);
                    return Err(format!("SDL_CreateRenderer failed: {}", sdl.last_error()));
                }
            }

            (sdl.show_cursor)(0); // Hide mouse cursor for handheld

            info!(
                "Created SDL2 display window ({}x{}, fullscreen={})",
                width, height, fullscreen
            );

            Ok(Self {
                sdl,
                window,
                renderer,
                target_width: width,
                target_height: height,
            })
        }
    }

    pub fn width(&self) -> u32 {
        self.target_width
    }

    pub fn height(&self) -> u32 {
        self.target_height
    }

    /// Clear screen, upload RGBA image as texture, scale preserving aspect ratio, and present.
    pub fn render_image(&mut self, image: &RgbaImage) -> Result<(), String> {
        let (img_w, img_h) = (image.width(), image.height());

        unsafe {
            let texture = (self.sdl.create_texture)(
                self.renderer,
                SDL_PIXELFORMAT_RGBA32,
                SDL_TEXTUREACCESS_STREAMING,
                img_w as c_int,
                img_h as c_int,
            );

            if texture.is_null() {
                return Err(format!("SDL_CreateTexture failed: {}", self.sdl.last_error()));
            }

            let pitch = (img_w * 4) as c_int;
            let pixels_ptr = image.as_raw().as_ptr() as *const c_void;
            (self.sdl.update_texture)(texture, std::ptr::null(), pixels_ptr, pitch);

            // Black letterbox background
            (self.sdl.set_render_draw_color)(self.renderer, 0, 0, 0, 255);
            (self.sdl.render_clear)(self.renderer);

            // Calculate destination rect preserving aspect ratio
            let scale_x = self.target_width as f32 / img_w as f32;
            let scale_y = self.target_height as f32 / img_h as f32;
            let scale = scale_x.min(scale_y);

            let dst_w = (img_w as f32 * scale).round() as c_int;
            let dst_h = (img_h as f32 * scale).round() as c_int;
            let dst_x = ((self.target_width as c_int - dst_w) / 2).max(0);
            let dst_y = ((self.target_height as c_int - dst_h) / 2).max(0);

            let dst_rect = SDL_Rect {
                x: dst_x,
                y: dst_y,
                w: dst_w,
                h: dst_h,
            };

            (self.sdl.render_copy)(self.renderer, texture, std::ptr::null(), &dst_rect);
            (self.sdl.render_present)(self.renderer);

            (self.sdl.destroy_texture)(texture);
        }

        Ok(())
    }

    /// Flush any pending events from the queue (e.g. initial launch button press).
    pub fn flush_events(&self) {
        let mut event = SDL_Event {
            type_: 0,
            padding: [0; 56],
        };
        unsafe {
            while (self.sdl.poll_event)(&mut event) != 0 {}
        }
    }

    /// Process SDL event queue. Returns true if user requested quit (SDL_QUIT or ESC / Q).
    pub fn poll_quit(&self) -> bool {
        let mut event = SDL_Event {
            type_: 0,
            padding: [0; 56],
        };

        unsafe {
            while (self.sdl.poll_event)(&mut event) != 0 {
                if event.type_ == SDL_QUIT {
                    return true;
                }
                if event.type_ == SDL_KEYDOWN {
                    let sym = i32::from_ne_bytes([
                        event.padding[16],
                        event.padding[17],
                        event.padding[18],
                        event.padding[19],
                    ]);
                    if sym == SDLK_ESCAPE || sym == SDLK_Q {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Poll for handheld gameplay actions mapped to gamepad buttons and keyboard keys.
    pub fn poll_action(&self) -> Option<HandheldAction> {
        let mut event = SDL_Event {
            type_: 0,
            padding: [0; 56],
        };

        unsafe {
            while (self.sdl.poll_event)(&mut event) != 0 {
                if event.type_ == SDL_QUIT {
                    return Some(HandheldAction::Quit);
                }

                if event.type_ == SDL_KEYDOWN {
                    let sym = i32::from_ne_bytes([
                        event.padding[16],
                        event.padding[17],
                        event.padding[18],
                        event.padding[19],
                    ]);

                    info!("SDL KeyDown received: sym={}", sym);

                    match sym {
                        SDLK_RETURN | SDLK_SPACE => return Some(HandheldAction::Next),
                        SDLK_PAGEUP => return Some(HandheldAction::Rollback),
                        SDLK_H => return Some(HandheldAction::HideUi),
                        SDLK_M => return Some(HandheldAction::Menu),
                        SDLK_TAB => return Some(HandheldAction::Skip),
                        SDLK_UP => return Some(HandheldAction::Up),
                        SDLK_DOWN => return Some(HandheldAction::Down),
                        SDLK_ESCAPE | SDLK_Q => return Some(HandheldAction::Quit),
                        _ => {
                            // Any other key also advances dialogue for convenience
                            return Some(HandheldAction::Next);
                        }
                    }
                }

                // Handle direct Touch Screen events (SDL_FINGERDOWN)
                if event.type_ == SDL_FINGERDOWN {
                    info!("SDL Touch FingerDown received on screen!");
                    return Some(HandheldAction::Next);
                }

                // Handle Mouse Clicks (mouse emulation or touch screen mapped to mouse)
                if event.type_ == SDL_MOUSEBUTTONDOWN {
                    let button = event.padding[16];
                    info!("SDL MouseButtonDown received: button={}", button);
                    if button == 1 { // Left click / Single Tap
                        return Some(HandheldAction::Next);
                    } else if button == 3 { // Right click / Two-finger tap
                        return Some(HandheldAction::Rollback);
                    }
                }

                // Handle direct SDL Joystick / Controller button events (PortMaster without GPTK)
                if event.type_ == SDL_JOYBUTTONDOWN || event.type_ == SDL_CONTROLLERBUTTONDOWN {
                    let button = event.padding[16];
                    info!("SDL Joystick/Controller button down: id={}", button);
                    match button {
                        0 | 11 | 12 => return Some(HandheldAction::Next),     // A button or Cross or Start
                        1 | 13 => return Some(HandheldAction::Rollback),      // B button or Circle
                        2 => return Some(HandheldAction::HideUi),              // X button
                        3 => return Some(HandheldAction::Menu),                // Y button
                        4 | 9 => return Some(HandheldAction::Skip),            // L1 button
                        6 | 10 => return Some(HandheldAction::Quit),           // Select / Back
                        7 => return Some(HandheldAction::Next),                // Start
                        _ => {
                            // Fallback: any button advances dialogue
                            return Some(HandheldAction::Next);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn delay(&self, ms: u32) {
        unsafe {
            (self.sdl.delay)(ms);
        }
    }
}

impl Drop for SdlDisplay {
    fn drop(&mut self) {
        unsafe {
            if !self.renderer.is_null() {
                (self.sdl.destroy_renderer)(self.renderer);
            }
            if !self.window.is_null() {
                (self.sdl.destroy_window)(self.window);
            }
            (self.sdl.quit)();
        }
    }
}
