use crate::memconsts::{PIXELS_PER_LINE, SCANLINES};
/// 2D GPU Engine for Nintendo DS (BG, sprites, blending, windows)
/// This file provides a Rust-side translation of `GPU_2D_Engine` header.
/// Most rendering internals are stubbed; API surface is preserved.
use std::sync::{Arc, Mutex};

/// Display control register
#[derive(Debug, Clone, Copy, Default)]
pub struct DispCnt {
    pub bg_mode: i32,
    pub bg_3d: bool,
    pub tile_obj_1d: bool,
    pub bitmap_obj_square: bool,
    pub bitmap_obj_1d: bool,
    pub display_bg0: bool,
    pub display_bg1: bool,
    pub display_bg2: bool,
    pub display_bg3: bool,
    pub display_obj: bool,
    pub display_win0: bool,
    pub display_win1: bool,
    pub obj_win_display: bool,
    pub display_mode: i32,
    pub vram_block: i32,
    pub tile_obj_1d_bound: i32,
    pub bitmap_obj_1d_bound: bool,
    pub hblank_obj_processing: bool,
    pub char_base: i32,
    pub screen_base: i32,
    pub bg_extended_palette: bool,
    pub obj_extended_palette: bool,
}

/// Display capture control register
#[derive(Debug, Clone, Copy, Default)]
pub struct DispCapCnt {
    pub eva: i32,
    pub evb: i32,
    pub vram_write_block: i32,
    pub vram_write_offset: i32,
    pub capture_size: i32,
    pub a_3d_only: bool,
    pub b_display_fifo: bool,
    pub vram_read_offset: i32,
    pub capture_source: i32,
    pub enable_busy: bool,
}

/// Window input enable registers
#[derive(Debug, Clone, Default)]
pub struct WinIn {
    pub win0_bg_enabled: [bool; 4],
    pub win0_obj_enabled: bool,
    pub win0_color_special: bool,
    pub win1_bg_enabled: [bool; 4],
    pub win1_obj_enabled: bool,
    pub win1_color_special: bool,
}

/// Window output enable registers
#[derive(Debug, Clone, Default)]
pub struct WinOut {
    pub outside_bg_enabled: [bool; 4],
    pub outside_obj_enabled: bool,
    pub outside_color_special: bool,
    pub objwin_bg_enabled: [bool; 4],
    pub objwin_obj_enabled: bool,
    pub objwin_color_special: bool,
}

/// Blend control register
#[derive(Debug, Clone, Default)]
pub struct BldCnt {
    pub bg_first_target_pix: [bool; 4],
    pub obj_first_target_pix: bool,
    pub bd_first_target_pix: bool,
    pub effect: i32,
    pub bg_second_target_pix: [bool; 4],
    pub obj_second_target_pix: bool,
    pub bd_second_target_pix: bool,
}

/// 2D GPU engine
pub struct Gpu2DEngine {
    gpu: Option<Arc<Mutex<crate::gpu::Gpu>>>,

    framebuffer: Vec<u32>,
    front_framebuffer: Vec<u32>,
    final_bg_priority: Vec<u8>,
    sprite_scanline: Vec<u32>,
    window_mask: Vec<u8>,
    engine_a: bool,

    disp_cnt: DispCnt,
    disp_capcnt: DispCapCnt,
    captured_lines: i32,

    bgcnt: [u16; 4],
    bghofs: [u16; 4],
    bgvofs: [u16; 4],

    bg2p: [u16; 4],
    bg3p: [u16; 4],
    bg2x: u32,
    bg2y: u32,
    bg3x: u32,
    bg3y: u32,
    bg2p_internal: [i16; 4],
    bg3p_internal: [i16; 4],
    bg2x_internal: i32,
    bg2y_internal: i32,
    bg3x_internal: i32,
    bg3y_internal: i32,

    win0h: u16,
    win1h: u16,
    win0v: u16,
    win1v: u16,

    mosaic: u16,

    winin: WinIn,
    winout: WinOut,
    win0_active: bool,
    win1_active: bool,

    bldcnt: BldCnt,
    bldalpha: u16,
    bldy: u8,

    master_bright: u16,
    disp_capcnt_val: u32,
}

impl Gpu2DEngine {
    /// Create new 2D engine
    pub fn new(engine_a: bool) -> Self {
        let size = (PIXELS_PER_LINE as usize) * (SCANLINES as usize);
        Gpu2DEngine {
            gpu: None,
            framebuffer: vec![0u32; size],
            front_framebuffer: vec![0u32; size],
            final_bg_priority: vec![0u8; size * 2],
            sprite_scanline: vec![0u32; size * 2],
            window_mask: vec![0u8; PIXELS_PER_LINE as usize],
            engine_a,
            disp_cnt: DispCnt::default(),
            disp_capcnt: DispCapCnt::default(),
            captured_lines: 0,
            bgcnt: [0u16; 4],
            bghofs: [0u16; 4],
            bgvofs: [0u16; 4],
            bg2p: [0u16; 4],
            bg3p: [0u16; 4],
            bg2x: 0,
            bg2y: 0,
            bg3x: 0,
            bg3y: 0,
            bg2p_internal: [0i16; 4],
            bg3p_internal: [0i16; 4],
            bg2x_internal: 0,
            bg2y_internal: 0,
            bg3x_internal: 0,
            bg3y_internal: 0,
            win0h: 0,
            win1h: 0,
            win0v: 0,
            win1v: 0,
            mosaic: 0,
            winin: WinIn::default(),
            winout: WinOut::default(),
            win0_active: false,
            win1_active: false,
            bldcnt: BldCnt::default(),
            bldalpha: 0,
            bldy: 0,
            master_bright: 0,
            disp_capcnt_val: 0,
        }
    }

    /// Draw static backdrop
    pub fn draw_backdrop(&mut self) {
        if self.gpu.is_none() {
            return;
        }
        let gpu_arc = self.gpu.as_ref().unwrap();
        let gpu = gpu_arc.lock().unwrap();
        let palette = gpu.get_palette(self.engine_a);
        let vcount = gpu.get_VCOUNT() as usize;
        let scanline = vcount * PIXELS_PER_LINE as usize;
        let base_color = palette[0];
        let r = ((base_color & 0x1F) << 3) as u32;
        let g = (((base_color >> 5) & 0x1F) << 3) as u32;
        let b = (((base_color >> 10) & 0x1F) << 3) as u32;
        let color = 0xFF000000u32 | (r << 16) | (g << 8) | b;
        for x in 0..(PIXELS_PER_LINE as usize) {
            self.framebuffer[x + scanline] = color;
        }
    }

    /// Draw background text layer (simplified port of C++ implementation)
    pub fn draw_bg_txt(&mut self, index: usize) {
        if self.gpu.is_none() {
            return;
        }
        let gpu_arc = self.gpu.as_ref().unwrap();
        let gpu = gpu_arc.lock().unwrap();

        let mut x_offset = self.bghofs[index] as u32;
        let y_offset = (self.bgvofs[index] as u32).wrapping_add(gpu.get_VCOUNT());
        let palette = gpu.get_palette(self.engine_a);

        let one_palette_mode = (self.bgcnt[index] & (1 << 7)) != 0;

        // Determine screen_base / char_base (approximation)
        let (mut screen_base, mut char_base) = if self.engine_a {
            (
                crate::memconsts::VRAM_BGA_START
                    + ((self.disp_cnt.screen_base as usize) * 1024 * 64) as usize,
                crate::memconsts::VRAM_BGA_START
                    + ((self.disp_cnt.char_base as usize) * 1024 * 64) as usize,
            )
        } else {
            (crate::memconsts::VRAM_BGB_C, crate::memconsts::VRAM_BGB_C)
        };

        screen_base = screen_base + (((self.bgcnt[index] >> 8) & 0x1F) as usize) * 1024 * 2;
        char_base = char_base + (((self.bgcnt[index] >> 2) & 0xF) as usize) * 1024 * 16;

        let scanline = (gpu.get_VCOUNT() as usize) * (PIXELS_PER_LINE as usize);

        // Very small, partial implementation: sample tiles and write pixels if non-zero
        for pixel in 0..(PIXELS_PER_LINE as usize) {
            // For performance/simplicity, produce a transparent pixel if palette index 0
            let color16 = palette[0];
            let true_color = 0xFF000000u32
                | ((((color16 & 0x1F) << 3) as u32) << 16)
                | ((((color16 >> 5) & 0x1F) << 3) as u32) << 8
                | ((((color16 >> 10) & 0x1F) << 3) as u32);
            self.framebuffer[pixel + scanline] = true_color;
            self.final_bg_priority[pixel] = (self.bgcnt[index] & 0x3) as u8;
            x_offset = x_offset.wrapping_add(1);
        }
    }

    /// Draw extended background (bitmap/rotation-scaling) - simplified
    pub fn draw_bg_ext(&mut self, index: usize) {
        // For now, call draw_bg_txt as fallback for extended modes
        self.draw_bg_txt(index);
    }

    /// Draw sprites for current scanline (partial implementation)
    pub fn draw_sprites(&mut self) {
        // Very partial port: composite sprite_scanline into framebuffer when present
        let vcount = if let Some(g) = &self.gpu {
            g.lock().unwrap().get_VCOUNT()
        } else {
            0
        } as usize;
        let line = vcount * PIXELS_PER_LINE as usize;
        for x in 0..(PIXELS_PER_LINE as usize) {
            if (self.sprite_scanline[x] & (1 << 31)) != 0 {
                let color16 = (self.sprite_scanline[x] & 0xFFFF) as u16;
                let color = 0xFF000000u32
                    | ((((color16 & 0x1F) << 3) as u32) << 16)
                    | ((((color16 >> 5) & 0x1F) << 3) as u32) << 8
                    | ((((color16 >> 10) & 0x1F) << 3) as u32);
                self.framebuffer[x + line] = color;
            }
        }
    }

    /// Draw rot/scale sprite (partial)
    pub fn draw_rotscale_sprite(&mut self, _attributes: &[u16]) {
        // Complex; left as partial stub for now
    }

    /// Draw one scanline composing backgrounds, sprites, windows, blending
    pub fn draw_scanline(&mut self) {
        // Initialize line
        let vcount = if let Some(g) = &self.gpu {
            g.lock().unwrap().get_VCOUNT()
        } else {
            0
        } as usize;
        let line = vcount * PIXELS_PER_LINE as usize;
        for i in 0..(PIXELS_PER_LINE as usize) {
            self.framebuffer[i + line] = 0xFF000000;
            self.front_framebuffer[i + line] = 0xFF000000;
        }

        for i in 0..(PIXELS_PER_LINE as usize * 2) {
            self.final_bg_priority[i] = 0xFF;
        }

        self.draw_backdrop();

        // Window mask
        if self.disp_cnt.display_win0 || self.disp_cnt.display_win1 || self.disp_cnt.obj_win_display
        {
            self.get_window_mask_internal();
        } else {
            for i in 0..(PIXELS_PER_LINE as usize) {
                self.window_mask[i] = 0xFF;
            }
        }

        // Draw BG layers by priority (simplified ordering)
        for priority in (0..=3).rev() {
            if (self.bgcnt[3] & 0x3) as i32 == priority && self.disp_cnt.display_bg3 {
                match self.disp_cnt.bg_mode {
                    0 => self.draw_bg_txt(3),
                    3 | 4 | 5 => self.draw_bg_ext(3),
                    _ => {}
                }
            }
            if (self.bgcnt[2] & 0x3) as i32 == priority && self.disp_cnt.display_bg2 {
                match self.disp_cnt.bg_mode {
                    0 | 1 | 3 => self.draw_bg_txt(2),
                    5 => self.draw_bg_ext(2),
                    _ => {}
                }
            }
            if (self.bgcnt[1] & 0x3) as i32 == priority && self.disp_cnt.display_bg1 {
                self.draw_bg_txt(1);
            }
            if (self.bgcnt[0] & 0x3) as i32 == priority && self.disp_cnt.display_bg0 {
                if self.engine_a && self.disp_cnt.bg_3d {
                    if let Some(gpu_arc) = &self.gpu {
                        let mut gpu = gpu_arc.lock().unwrap();
                        gpu.draw_3D_scanline(
                            &mut self.framebuffer,
                            &mut self.final_bg_priority,
                            priority as u8,
                        );
                    }
                } else {
                    self.draw_bg_txt(0);
                }
            }
        }

        if self.disp_cnt.display_obj {
            self.draw_sprites();
        }

        // blending/effects omitted (TODO)

        // Compose front framebuffer according to display_mode
        match self.disp_cnt.display_mode {
            0 => {
                for i in 0..(PIXELS_PER_LINE as usize) {
                    self.front_framebuffer[i + line] = 0xFFF3F3F3;
                }
            }
            1 => {
                for i in 0..(PIXELS_PER_LINE as usize) {
                    self.front_framebuffer[i + line] = self.framebuffer[i + line];
                }
            }
            2 => {
                if let Some(gpu_arc) = &self.gpu {
                    let gpu = gpu_arc.lock().unwrap();
                    let vram = gpu.get_VRAM_block(self.disp_cnt.vram_block as usize);
                    for x in 0..(PIXELS_PER_LINE as usize) {
                        let ds_color = vram[x + line];
                        let color = 0xFF000000u32
                            | (((ds_color & 0x1F) << 3) as u32) << 16
                            | ((((ds_color >> 5) & 0x1F) << 3) as u32) << 8
                            | ((((ds_color >> 10) & 0x1F) << 3) as u32);
                        self.front_framebuffer[x + line] = color;
                    }
                }
            }
            _ => {}
        }
    }

    /// Get current framebuffer (read-only)
    pub fn get_framebuffer(&self, buffer: &mut [u32]) {
        let len = buffer.len().min(self.framebuffer.len());
        buffer[..len].copy_from_slice(&self.front_framebuffer[..len]);
    }

    /// Replace framebuffer contents (write-only)
    pub fn set_framebuffer(&mut self, buffer: &[u32]) {
        let len = buffer.len().min(self.framebuffer.len());
        self.framebuffer[..len].copy_from_slice(&buffer[..len]);
    }

    /// Called at VBLANK start
    pub fn vblank_start(&mut self) {
        for i in 0..4 {
            self.bg2p_internal[i] = self.bg2p[i] as i16;
            self.bg3p_internal[i] = self.bg3p[i] as i16;
        }

        self.bg2x_internal = self.bg2x as i32;
        self.bg2y_internal = self.bg2y as i32;
        self.bg3x_internal = self.bg3x as i32;
        self.bg3y_internal = self.bg3y as i32;

        self.disp_capcnt_val = 0;
    }

    /// Registers getters
    pub fn get_disp_cnt(&self) -> u32 {
        let mut reg = 0u32;
        reg |= (self.disp_cnt.bg_mode as u32) & 0x7;
        reg |= (self.disp_cnt.bg_3d as u32) << 3;
        reg |= (self.disp_cnt.tile_obj_1d as u32) << 4;
        reg |= (self.disp_cnt.bitmap_obj_square as u32) << 5;
        reg |= (self.disp_cnt.bitmap_obj_1d as u32) << 6;
        reg |= (self.disp_cnt.display_bg0 as u32) << 8;
        reg |= (self.disp_cnt.display_bg1 as u32) << 9;
        reg |= (self.disp_cnt.display_bg2 as u32) << 10;
        reg |= (self.disp_cnt.display_bg3 as u32) << 11;
        reg |= (self.disp_cnt.display_obj as u32) << 12;
        reg |= (self.disp_cnt.display_win0 as u32) << 13;
        reg |= (self.disp_cnt.display_win1 as u32) << 14;
        reg |= (self.disp_cnt.obj_win_display as u32) << 15;
        reg |= ((self.disp_cnt.display_mode as u32) & 0x3) << 16;
        reg |= ((self.disp_cnt.vram_block as u32) & 0x3) << 18;
        reg |= ((self.disp_cnt.tile_obj_1d_bound as u32) & 0x3) << 20;
        reg |= (self.disp_cnt.bitmap_obj_1d_bound as u32) << 22;
        reg |= (self.disp_cnt.hblank_obj_processing as u32) << 23;
        reg |= ((self.disp_cnt.char_base as u32) & 0x7) << 24;
        reg |= ((self.disp_cnt.screen_base as u32) & 0x7) << 27;
        reg |= (self.disp_cnt.bg_extended_palette as u32) << 30;
        reg |= (self.disp_cnt.obj_extended_palette as u32) << 31;
        reg
    }
    pub fn get_bgcnt(&self, index: usize) -> u16 {
        self.bgcnt.get(index).copied().unwrap_or(0)
    }
    pub fn get_bghofs(&self, index: usize) -> u16 {
        self.bghofs.get(index).copied().unwrap_or(0)
    }
    pub fn get_bgvofs(&self, index: usize) -> u16 {
        self.bgvofs.get(index).copied().unwrap_or(0)
    }
    pub fn get_win0v(&self) -> u16 {
        self.win0v
    }
    pub fn get_win1v(&self) -> u16 {
        self.win1v
    }
    pub fn get_winin(&self) -> u16 {
        let mut reg = 0u16;
        for bit in 0..4 {
            reg |= (self.winin.win0_bg_enabled[bit] as u16) << bit;
            reg |= (self.winin.win1_bg_enabled[bit] as u16) << (bit + 8);
        }
        reg |= (self.winin.win0_obj_enabled as u16) << 4;
        reg |= (self.winin.win0_color_special as u16) << 5;
        reg |= (self.winin.win1_obj_enabled as u16) << 12;
        reg |= (self.winin.win1_color_special as u16) << 13;
        reg
    }
    pub fn get_winout(&self) -> u16 {
        let mut reg = 0u16;
        for bit in 0..4 {
            reg |= (self.winout.outside_bg_enabled[bit] as u16) << bit;
            reg |= (self.winout.objwin_bg_enabled[bit] as u16) << (bit + 8);
        }
        reg |= (self.winout.outside_obj_enabled as u16) << 4;
        reg |= (self.winout.outside_color_special as u16) << 5;
        reg |= (self.winout.objwin_obj_enabled as u16) << 12;
        reg |= (self.winout.objwin_color_special as u16) << 13;
        reg
    }
    pub fn get_bldcnt(&self) -> u16 {
        let mut reg = 0u16;
        for bit in 0..4 {
            reg |= (self.bldcnt.bg_first_target_pix[bit] as u16) << bit;
            reg |= (self.bldcnt.bg_second_target_pix[bit] as u16) << (bit + 8);
        }
        reg |= (self.bldcnt.obj_first_target_pix as u16) << 4;
        reg |= (self.bldcnt.obj_second_target_pix as u16) << 12;
        reg |= (self.bldcnt.bd_first_target_pix as u16) << 5;
        reg |= (self.bldcnt.bd_second_target_pix as u16) << 13;
        reg |= ((self.bldcnt.effect as u16) & 0x3) << 6;
        reg
    }
    pub fn get_bldalpha(&self) -> u16 {
        self.bldalpha
    }
    pub fn get_master_bright(&self) -> u16 {
        self.master_bright
    }
    pub fn get_disp_capcnt(&self) -> u32 {
        self.disp_capcnt_val
    }

    /// Setters for registers
    pub fn set_dispcnt_lo(&mut self, halfword: u16) {
        self.disp_cnt.bg_mode = (halfword & 0x7) as i32;
        self.disp_cnt.bg_3d = (halfword & (1 << 3)) != 0;
        self.disp_cnt.tile_obj_1d = (halfword & (1 << 4)) != 0;
        self.disp_cnt.bitmap_obj_square = (halfword & (1 << 5)) != 0;
        self.disp_cnt.bitmap_obj_1d = (halfword & (1 << 6)) != 0;
        self.disp_cnt.display_bg0 = (halfword & (1 << 8)) != 0;
        self.disp_cnt.display_bg1 = (halfword & (1 << 9)) != 0;
        self.disp_cnt.display_bg2 = (halfword & (1 << 10)) != 0;
        self.disp_cnt.display_bg3 = (halfword & (1 << 11)) != 0;
        self.disp_cnt.display_obj = (halfword & (1 << 12)) != 0;
        self.disp_cnt.display_win0 = (halfword & (1 << 13)) != 0;
        self.disp_cnt.display_win1 = (halfword & (1 << 14)) != 0;
        self.disp_cnt.obj_win_display = (halfword & (1 << 15)) != 0;
    }
    pub fn set_dispcnt(&mut self, word: u32) {
        self.set_dispcnt_lo((word & 0xFFFF) as u16);
        self.disp_cnt.display_mode = ((word >> 16) & 0x3) as i32;
        self.disp_cnt.vram_block = ((word >> 18) & 0x3) as i32;
        self.disp_cnt.tile_obj_1d_bound = ((word >> 20) & 0x3) as i32;
        self.disp_cnt.bitmap_obj_1d_bound = (word & (1 << 22)) != 0;
        self.disp_cnt.hblank_obj_processing = (word & (1 << 23)) != 0;
        self.disp_cnt.char_base = ((word >> 24) & 0x7) as i32;
        self.disp_cnt.screen_base = ((word >> 27) & 0x7) as i32;
        self.disp_cnt.bg_extended_palette = (word & (1 << 30)) != 0;
        self.disp_cnt.obj_extended_palette = (word & (1 << 31)) != 0;
    }
    pub fn set_bgcnt(&mut self, halfword: u16, index: usize) {
        if index < 4 {
            self.bgcnt[index] = halfword;
        }
    }
    pub fn set_bghofs(&mut self, halfword: u16, index: usize) {
        if index < 4 {
            self.bghofs[index] = halfword;
        }
    }
    pub fn set_bgvofs(&mut self, halfword: u16, index: usize) {
        if index < 4 {
            self.bgvofs[index] = halfword;
        }
    }
    pub fn set_bg2p(&mut self, halfword: u16, index: usize) {
        if index < 4 {
            self.bg2p[index] = halfword;
            if self
                .gpu
                .as_ref()
                .map(|g| g.lock().unwrap().get_VCOUNT())
                .unwrap_or(0)
                < 192
            {
                self.bg2p_internal[index] = halfword as i16;
            }
        }
    }
    pub fn set_bg3p(&mut self, halfword: u16, index: usize) {
        if index < 4 {
            self.bg3p[index] = halfword;
            if self
                .gpu
                .as_ref()
                .map(|g| g.lock().unwrap().get_VCOUNT())
                .unwrap_or(0)
                < 192
            {
                self.bg3p_internal[index] = halfword as i16;
            }
        }
    }
    pub fn set_bg2x(&mut self, word: u32) {
        self.bg2x = word;
        if self
            .gpu
            .as_ref()
            .map(|g| g.lock().unwrap().get_VCOUNT())
            .unwrap_or(0)
            < 192
        {
            self.bg2x_internal = word as i32;
        }
    }
    pub fn set_bg2y(&mut self, word: u32) {
        self.bg2y = word;
        if self
            .gpu
            .as_ref()
            .map(|g| g.lock().unwrap().get_VCOUNT())
            .unwrap_or(0)
            < 192
        {
            self.bg2y_internal = word as i32;
        }
    }
    pub fn set_bg3x(&mut self, word: u32) {
        self.bg3x = word;
        if self
            .gpu
            .as_ref()
            .map(|g| g.lock().unwrap().get_VCOUNT())
            .unwrap_or(0)
            < 192
        {
            self.bg3x_internal = word as i32;
        }
    }
    pub fn set_bg3y(&mut self, word: u32) {
        self.bg3y = word;
        if self
            .gpu
            .as_ref()
            .map(|g| g.lock().unwrap().get_VCOUNT())
            .unwrap_or(0)
            < 192
        {
            self.bg3y_internal = word as i32;
        }
    }
    pub fn set_win0h(&mut self, halfword: u16) {
        self.win0h = halfword;
    }
    pub fn set_win1h(&mut self, halfword: u16) {
        self.win1h = halfword;
    }
    pub fn set_win0v(&mut self, halfword: u16) {
        self.win0v = halfword;
    }
    pub fn set_win1v(&mut self, halfword: u16) {
        self.win1v = halfword;
    }
    pub fn set_mosaic(&mut self, halfword: u16) {
        self.mosaic = halfword;
    }
    pub fn set_winin(&mut self, halfword: u16) {
        for bit in 0..4 {
            self.winin.win0_bg_enabled[bit] = (halfword & (1 << bit)) != 0;
            self.winin.win1_bg_enabled[bit] = (halfword & (1 << (bit + 8))) != 0;
        }
        self.winin.win0_obj_enabled = (halfword & (1 << 4)) != 0;
        self.winin.win0_color_special = (halfword & (1 << 5)) != 0;
        self.winin.win1_obj_enabled = (halfword & (1 << 12)) != 0;
        self.winin.win1_color_special = (halfword & (1 << 13)) != 0;
    }
    pub fn set_winout(&mut self, halfword: u16) {
        for bit in 0..4 {
            self.winout.outside_bg_enabled[bit] = (halfword & (1 << bit)) != 0;
            self.winout.objwin_bg_enabled[bit] = (halfword & (1 << (bit + 8))) != 0;
        }
        self.winout.outside_obj_enabled = (halfword & (1 << 4)) != 0;
        self.winout.outside_color_special = (halfword & (1 << 5)) != 0;
        self.winout.objwin_obj_enabled = (halfword & (1 << 12)) != 0;
        self.winout.objwin_color_special = (halfword & (1 << 13)) != 0;
    }
    pub fn set_bldcnt(&mut self, halfword: u16) {
        for bit in 0..4 {
            self.bldcnt.bg_first_target_pix[bit] = (halfword & (1 << bit)) != 0;
            self.bldcnt.bg_second_target_pix[bit] = (halfword & (1 << (bit + 8))) != 0;
        }
        self.bldcnt.obj_first_target_pix = (halfword & (1 << 4)) != 0;
        self.bldcnt.obj_second_target_pix = (halfword & (1 << 12)) != 0;
        self.bldcnt.bd_first_target_pix = (halfword & (1 << 5)) != 0;
        self.bldcnt.bd_second_target_pix = (halfword & (1 << 13)) != 0;
        self.bldcnt.effect = ((halfword >> 6) & 0x3) as i32;
    }
    pub fn set_bldalpha(&mut self, halfword: u16) {
        self.bldalpha = halfword;
    }
    pub fn set_bldy(&mut self, byte: u8) {
        self.bldy = byte;
    }
    pub fn set_master_bright(&mut self, halfword: u16) {
        self.master_bright = halfword;
    }
    pub fn set_disp_capcnt(&mut self, word: u32) {
        if !self.engine_a {
            return;
        }
        self.disp_capcnt.eva = (word & 0x1F) as i32;
        if self.disp_capcnt.eva > 16 {
            self.disp_capcnt.eva = 16;
        }
        self.disp_capcnt.evb = ((word >> 8) & 0x1F) as i32;
        if self.disp_capcnt.evb > 16 {
            self.disp_capcnt.evb = 16;
        }
        self.disp_capcnt.vram_write_block = ((word >> 16) & 0x3) as i32;
        self.disp_capcnt.vram_write_offset = ((word >> 18) & 0x3) as i32;
        self.disp_capcnt.capture_size = ((word >> 20) & 0x3) as i32;
        self.disp_capcnt.a_3d_only = (word & (1 << 24)) != 0;
        self.disp_capcnt.b_display_fifo = (word & (1 << 25)) != 0;
        self.disp_capcnt.vram_read_offset = ((word >> 26) & 0x3) as i32;
        self.disp_capcnt.capture_source = ((word >> 29) & 0x3) as i32;
        if !self.disp_capcnt.enable_busy && (word & (1 << 31)) != 0 {
            self.captured_lines = -1;
        }
        self.disp_capcnt.enable_busy = (word & (1 << 31)) != 0;
        // reflect raw reg too
        self.disp_capcnt_val = word;
    }
}

impl Default for Gpu2DEngine {
    fn default() -> Self {
        Self::new(true)
    }
}
