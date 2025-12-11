/// CorgiDS - Emulator Window Module
/// Copyright PSISP 2017
/// Licensed under the GPLv3
/// See LICENSE.txt for details
use druid::widget::{Button, Container, Flex, Image, Label};
use druid::{
    im::Vector, AppLauncher, Color, Data, Env, Event, EventCtx, LayoutCtx, LocalizedString,
    PaintCtx, Size, UnitData, UpdateCtx, Widget, WidgetExt, WindowDesc,
};
use std::sync::{Arc, Mutex};

/// DS key codes enumeration
#[derive(Clone, Copy, Debug, PartialEq, Data)]
pub enum DSKey {
    /// D-Pad keys
    ButtonUp = 0,
    ButtonDown = 1,
    ButtonLeft = 2,
    ButtonRight = 3,
    /// Face buttons
    ButtonA = 4,
    ButtonB = 5,
    ButtonX = 6,
    ButtonY = 7,
    /// Shoulder buttons
    ButtonL = 8,
    ButtonR = 9,
    /// System buttons
    ButtonStart = 10,
    ButtonSelect = 11,
    /// Debug key
    Debugging = 12,
}

/// Screen dimensions constants
pub const PIXELS_PER_LINE: usize = 256;
pub const SCANLINES: usize = 192;

/// Pause event types
#[derive(Clone, Copy, Debug, PartialEq, Data)]
pub enum PauseEvent {
    /// Game not loaded yet
    GameNotStarted,
    /// ROM file is loading
    LoadingRom,
}

/// Frame buffer data
#[derive(Clone)]
pub struct FrameBuffer {
    /// Upper screen buffer (256x192 RGBA)
    upper: Vec<u32>,
    /// Lower screen buffer (256x192 RGBA)
    lower: Vec<u32>,
}

impl FrameBuffer {
    /// Create new empty frame buffer
    pub fn new() -> Self {
        FrameBuffer {
            upper: vec![0; PIXELS_PER_LINE * SCANLINES],
            lower: vec![0; PIXELS_PER_LINE * SCANLINES],
        }
    }

    /// Update upper screen buffer
    pub fn update_upper(&mut self, buffer: &[u32]) {
        if buffer.len() == self.upper.len() {
            self.upper.copy_from_slice(buffer);
        }
    }

    /// Update lower screen buffer
    pub fn update_lower(&mut self, buffer: &[u32]) {
        if buffer.len() == self.lower.len() {
            self.lower.copy_from_slice(buffer);
        }
    }
}

/// Main emulator window state
#[derive(Clone, Data)]
pub struct EmuWindow {
    /// Current ROM file name
    pub rom_file_name: String,
    /// Frame buffer data
    pub frame_buffer: Arc<Mutex<FrameBuffer>>,
    /// Current FPS counter
    pub fps: u32,
    /// Is emulation running
    pub is_running: bool,
    /// Is currently emulating a game
    pub is_emulating: bool,
    /// Configuration state
    pub enable_framelimiter: bool,
    pub frameskip: bool,
}

impl EmuWindow {
    /// Create new emulator window
    pub fn new() -> Self {
        EmuWindow {
            rom_file_name: String::new(),
            frame_buffer: Arc::new(Mutex::new(FrameBuffer::new())),
            fps: 0,
            is_running: false,
            is_emulating: false,
            enable_framelimiter: true,
            frameskip: false,
        }
    }

    /// Initialize the emulator window
    pub fn initialize(&self) -> Result<(), String> {
        /// Initialize emulation thread
        // TODO: Initialize EmuThread equivalent

        /// Set window title
        println!("CorgiDS initialized successfully");
        Ok(())
    }

    /// Check if emulator is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Check if game is being emulated
    pub fn is_emulating(&self) -> bool {
        self.is_emulating
    }

    /// Check if frame finished rendering
    pub fn finished_frame(&self) -> bool {
        true // TODO: Implement frame sync
    }

    /// Update frame buffers with new data
    pub fn draw_frame(&mut self, upper_buffer: &[u32], lower_buffer: &[u32]) {
        if let Ok(mut fb) = self.frame_buffer.lock() {
            fb.update_upper(upper_buffer);
            fb.update_lower(lower_buffer);
        }
    }

    /// Update FPS display
    pub fn update_fps(&mut self, fps: u32) {
        self.fps = fps;
        println!("CorgiDS - {} FPS", fps);
    }

    /// Handle key press event
    pub fn handle_key_press(&self, key_code: u32) -> Option<DSKey> {
        /// Map keyboard codes to DS keys
        match key_code {
            // Arrow keys
            38 => Some(DSKey::ButtonUp),    // Up arrow
            40 => Some(DSKey::ButtonDown),  // Down arrow
            37 => Some(DSKey::ButtonLeft),  // Left arrow
            39 => Some(DSKey::ButtonRight), // Right arrow
            // QWAS for shoulder and face buttons
            81 => Some(DSKey::ButtonL), // Q
            87 => Some(DSKey::ButtonR), // W
            65 => Some(DSKey::ButtonY), // A
            83 => Some(DSKey::ButtonX), // S
            88 => Some(DSKey::ButtonA), // X
            90 => Some(DSKey::ButtonB), // Z
            // Action buttons
            13 => Some(DSKey::ButtonStart),  // Return/Enter
            32 => Some(DSKey::ButtonSelect), // Space
            48 => Some(DSKey::Debugging),    // 0
            // Tab for framelimiter toggle
            9 => {
                // TODO: Toggle framelimiter
                None
            }
            // O for frameskip toggle
            79 => {
                // TODO: Toggle frameskip
                None
            }
            // P for manual pause
            80 => {
                // TODO: Manual pause
                None
            }
            _ => None,
        }
    }

    /// Handle touchscreen input
    pub fn handle_touchscreen(&self, x: i32, y: i32) {
        if y > SCANLINES as i32 {
            let touch_x = x;
            let touch_y = y - SCANLINES as i32;
            if touch_y >= 0 && touch_y < SCANLINES as i32 {
                println!("Touchscreen event: ({}, {})", touch_x, touch_y);
                // TODO: Send touchscreen event to emulation thread
            }
        }
    }

    /// Load ROM file
    pub fn load_rom(&mut self, path: &str) -> Result<(), String> {
        /// Check if firmware is loaded
        // TODO: Verify firmware is loaded

        /// Load the ROM file
        println!("Loading ROM: {}", path);
        self.rom_file_name = path.to_string();

        /// Start emulation
        self.is_emulating = true;
        Ok(())
    }

    /// Save screenshot to file
    pub fn save_screenshot(&self, path: &str) -> Result<(), String> {
        if let Ok(fb) = self.frame_buffer.lock() {
            println!("Saving screenshot to: {}", path);
            // TODO: Implement actual screenshot saving
            Ok(())
        } else {
            Err("Failed to acquire frame buffer lock".to_string())
        }
    }

    /// Show preferences dialog
    pub fn show_preferences(&self) {
        println!("Opening preferences dialog");
        // TODO: Implement preferences dialog
    }

    /// Show about dialog
    pub fn show_about(&self) {
        println!("CorgiDS v0.1 - Created by PSISP");
        // TODO: Implement about dialog
    }
}

impl Default for EmuWindow {
    fn default() -> Self {
        Self::new()
    }
}
