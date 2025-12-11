use crate::memconsts::{PIXELS_PER_LINE, SCANLINES};
/// Emulation thread wrapper
/// Provides a thread loop that runs the `Emulator` instance, handles pause/shutdown,
/// input events, and emits frame updates via optional callbacks.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

/// Pause event types (ported from C++ PAUSE_EVENT)
#[derive(Debug, Clone, Copy)]
pub enum PauseEvent {
    GameNotStarted = 0,
    LoadingRom = 1,
    OutOfFocus = 2,
    Manual = 3,
}

/// DS button keys (ported from C++ DS_KEYS)
#[derive(Debug, Clone, Copy)]
pub enum DsKeys {
    A,
    B,
    X,
    Y,
    L,
    R,
    Start,
    Select,
    Left,
    Right,
    Up,
    Down,
    Debugging,
}

/// Emulation thread structure
pub struct EmuThread {
    emulator: Arc<Mutex<crate::emulator::Emulator>>,

    load_mutex: Mutex<()>,
    pause_mutex: Mutex<()>,
    key_mutex: Mutex<()>,
    screen_mutex: Mutex<()>,

    pause_status: Arc<Mutex<u32>>,
    abort: Arc<AtomicBool>,

    upper_buffer: Vec<u32>,
    lower_buffer: Vec<u32>,

    /// Optional callbacks to receive finished frames and FPS updates
    pub finished_frame_cb: Option<Box<dyn Fn(&[u32], &[u32]) + Send + Sync>>,
    pub update_fps_cb: Option<Box<dyn Fn(i32) + Send + Sync>>,
}

impl EmuThread {
    /// Create a new EmuThread with its own Emulator instance
    pub fn new() -> Self {
        EmuThread {
            emulator: Arc::new(Mutex::new(crate::emulator::Emulator::new())),
            load_mutex: Mutex::new(()),
            pause_mutex: Mutex::new(()),
            key_mutex: Mutex::new(()),
            screen_mutex: Mutex::new(()),
            pause_status: Arc::new(Mutex::new(1)), // start paused
            abort: Arc::new(AtomicBool::new(false)),
            upper_buffer: vec![0u32; (PIXELS_PER_LINE as usize) * (SCANLINES as usize)],
            lower_buffer: vec![0u32; (PIXELS_PER_LINE as usize) * (SCANLINES as usize)],
            finished_frame_cb: None,
            update_fps_cb: None,
        }
    }

    /// Initialize emulator (wrapper)
    pub fn init(&mut self) -> Result<(), String> {
        let mut emu = self.emulator.lock().unwrap();
        emu.init()
    }

    /// Load firmware (thread-safe)
    pub fn load_firmware(&mut self) -> Result<(), String> {
        let _g = self.load_mutex.lock().unwrap();
        let mut emu = self.emulator.lock().unwrap();
        emu.load_firmware()
    }

    /// Load save database
    pub fn load_save_database(&mut self, path: &str) {
        let _g = self.load_mutex.lock().unwrap();
        let mut emu = self.emulator.lock().unwrap();
        let _ = emu.load_save_database(path); // ignore result for now
    }

    /// Load ROM file
    pub fn load_game(&mut self, rom_path: &str) -> Result<(), String> {
        let _g = self.load_mutex.lock().unwrap();
        let mut emu = self.emulator.lock().unwrap();
        emu.load_rom(rom_path)
    }

    /// Blocking run loop: mirrors C++ EmuThread::run
    /// This will run until `shutdown` is called which sets `abort`.
    pub fn run(&mut self) {
        self.abort.store(false, Ordering::SeqCst);

        let max_us_count = 1_000_000u64 / 60u64; // microseconds per frame target
        let second_count = 1_000_000u64;
        let mut frames = 0i32;
        let mut fps_update = Instant::now();

        loop {
            // check abort
            if self.abort.load(Ordering::SeqCst) {
                return;
            }

            // check pause
            let paused = {
                let ps = self.pause_status.lock().unwrap();
                *ps != 0
            };

            if paused {
                // Sleep a short time while paused
                thread::sleep(Duration::from_millis(1));
                continue;
            }

            // Run one emulation step/frame
            let start = Instant::now();
            {
                let mut emu = self.emulator.lock().unwrap();
                let _ = emu.run();
                let ub = emu.get_upper_frame();
                let lb = emu.get_lower_frame();
                // copy into buffers
                let len = self.upper_buffer.len().min(ub.len());
                self.upper_buffer[..len].copy_from_slice(&ub[..len]);
                let len2 = self.lower_buffer.len().min(lb.len());
                self.lower_buffer[..len2].copy_from_slice(&lb[..len2]);
            }

            // callback emit finished_frame
            if let Some(cb) = &self.finished_frame_cb {
                cb(&self.upper_buffer, &self.lower_buffer);
            }

            frames += 1;

            // frame timing
            let elapsed = start.elapsed();
            let us = (elapsed.as_micros()) as u64;
            if crate::config::Config::enable_framelimiter && us < max_us_count {
                let sleep_us = max_us_count - us;
                thread::sleep(Duration::from_micros(sleep_us));
            }

            let diff = fps_update.elapsed();
            let us_since = diff.as_micros() as u64;
            if us_since >= second_count {
                if let Some(cb) = &self.update_fps_cb {
                    cb(frames);
                }
                fps_update = Instant::now();
                frames = 0;
            }
        }
    }

    /// Shutdown the run loop
    pub fn shutdown(&mut self) {
        let _g = self.load_mutex.lock().unwrap();
        self.abort.store(true, Ordering::SeqCst);
    }

    /// Toggle manual pause bit
    pub fn manual_pause(&mut self) {
        let _g = self.pause_mutex.lock().unwrap();
        let mut ps = self.pause_status.lock().unwrap();
        let bit = 1 << (PauseEvent::Manual as u32);
        if (*ps & bit) != 0 {
            *ps &= !bit;
        } else {
            *ps |= bit;
        }
    }

    /// Pause with specific event
    pub fn pause(&mut self, event: PauseEvent) {
        let _g = self.pause_mutex.lock().unwrap();
        let mut ps = self.pause_status.lock().unwrap();
        *ps |= 1 << (event as u32);
    }

    /// Unpause for specific event
    pub fn unpause(&mut self, event: PauseEvent) {
        let _g = self.pause_mutex.lock().unwrap();
        let mut ps = self.pause_status.lock().unwrap();
        *ps &= !(1 << (event as u32));
    }

    /// Handle key press events
    pub fn press_key(&mut self, key: DsKeys) {
        let _g = self.key_mutex.lock().unwrap();
        let mut emu = self.emulator.lock().unwrap();
        match key {
            DsKeys::Left => emu.button_left_pressed(),
            DsKeys::Right => emu.button_right_pressed(),
            DsKeys::Up => emu.button_up_pressed(),
            DsKeys::Down => emu.button_down_pressed(),
            DsKeys::A => emu.button_a_pressed(),
            DsKeys::B => emu.button_b_pressed(),
            DsKeys::X => emu.button_x_pressed(),
            DsKeys::Y => emu.button_y_pressed(),
            DsKeys::L => emu.button_l_pressed(),
            DsKeys::R => emu.button_r_pressed(),
            DsKeys::Start => emu.button_start_pressed(),
            DsKeys::Select => emu.button_select_pressed(),
            DsKeys::Debugging => {
                let _ = emu.debug();
            }
        }
    }

    /// Handle key release events
    pub fn release_key(&mut self, key: DsKeys) {
        let _g = self.key_mutex.lock().unwrap();
        let mut emu = self.emulator.lock().unwrap();
        match key {
            DsKeys::Left => emu.button_left_released(),
            DsKeys::Right => emu.button_right_released(),
            DsKeys::Up => emu.button_up_released(),
            DsKeys::Down => emu.button_down_released(),
            DsKeys::A => emu.button_a_released(),
            DsKeys::B => emu.button_b_released(),
            DsKeys::X => emu.button_x_released(),
            DsKeys::Y => emu.button_y_released(),
            DsKeys::L => emu.button_l_released(),
            DsKeys::R => emu.button_r_released(),
            DsKeys::Start => emu.button_start_released(),
            DsKeys::Select => emu.button_select_released(),
            _ => {}
        }
    }

    /// Touchscreen event
    pub fn touchscreen_event(&mut self, x: i32, y: i32) {
        let _g = self.screen_mutex.lock().unwrap();
        let mut emu = self.emulator.lock().unwrap();
        let _ = emu.touchscreen_press(x, y);
    }
}

impl Default for EmuThread {
    fn default() -> Self {
        Self::new()
    }
}
