use crate::memconsts::{PIXELS_PER_LINE, SCANLINES};
/// 3D GPU (GX) subsystem for Nintendo DS
/// This module provides a Rust-side representation of the GPU_3D class
/// from the original emulator. Many functions are stubbed to match
/// the original API surface; detailed implementation can be filled in later.
use std::sync::{Arc, Mutex};

/// Display 3D control registe
#[derive(Debug, Clone, Copy, Default)]
pub struct Disp3dCnt {
    pub texture_mapping: bool,
    pub highlight_shading: bool,
    pub alpha_test: bool,
    pub alpha_blending: bool,
    pub anti_aliasing: bool,
    pub edge_marking: bool,
    pub fog_color_mode: bool,
    pub fog_enable: bool,
    pub fog_depth_shift: i32,
    pub color_buffer_underflow: bool,
    pub ram_overflow: bool,
    pub rear_plane_mode: bool,
}

/// Texture image parameters
#[derive(Debug, Clone, Copy, Default)]
pub struct TexImageParam {
    pub vram_offset: i32,
    pub repeat_s: bool,
    pub repeat_t: bool,
    pub flip_s: bool,
    pub flip_t: bool,
    pub s_size: i32,
    pub t_size: i32,
    pub format: i32,
    pub color0_transparent: bool,
    pub transformation_mode: i32,
}

/// Polygon attributes register
#[derive(Debug, Clone, Copy, Default)]
pub struct PolygonAttr {
    pub light_enable: i32,
    pub polygon_mode: i32,
    pub render_back: bool,
    pub render_front: bool,
    pub set_new_trans_depth: bool,
    pub render_1dot: bool,
    pub render_far_intersect: bool,
    pub depth_test_equal: bool,
    pub fog_enable: bool,
    pub alpha: i32,
    pub id: i32,
}

/// Viewport rectangle
#[derive(Debug, Clone, Copy, Default)]
pub struct Viewport {
    pub x1: u8,
    pub y1: u8,
    pub x2: u8,
    pub y2: u8,
}

/// GX status register
#[derive(Debug, Clone, Copy, Default)]
pub struct GxStat {
    pub box_pos_vec_busy: bool,
    pub boxtest_result: bool,
    pub mtx_stack_busy: bool,
    pub mtx_overflow: bool,
    pub geo_busy: bool,
    pub gx_fifo_irq_stat: i32,
}

/// 4x4 matrix
#[derive(Debug, Clone)]
pub struct Mtx {
    pub m: [[i32; 4]; 4],
}

impl Default for Mtx {
    fn default() -> Self {
        Mtx { m: [[0; 4]; 4] }
    }
}

impl Mtx {
    /// Set matrix values from another matrix
    pub fn set(&mut self, other: &Mtx) {
        self.m = other.m;
    }
}

/// Vertex structure used in geometry pipeline
#[derive(Debug, Clone)]
pub struct Vertex {
    pub coords: [i32; 4],
    pub colors: [i32; 3],
    pub final_colors: [i32; 3],
    pub clipped: bool,
    pub texcoords: [i32; 2],
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            coords: [0; 4],
            colors: [0; 3],
            final_colors: [0; 3],
            clipped: false,
            texcoords: [0; 2],
        }
    }
}

/// Polygon representation
#[derive(Debug, Clone)]
pub struct Polygon {
    pub vert_index: u16,
    pub vertices: u8,

    pub top_y: u16,
    pub bottom_y: u16,

    pub attributes: PolygonAttr,
    pub texparams: TexImageParam,
    pub palette_base: u32,

    pub translucent: bool,
}

impl Default for Polygon {
    fn default() -> Self {
        Polygon {
            vert_index: 0,
            vertices: 0,
            top_y: 0,
            bottom_y: 0,
            attributes: PolygonAttr::default(),
            texparams: TexImageParam::default(),
            palette_base: 0,
            translucent: false,
        }
    }
}

/// Simple GX command (command byte + parameter)
#[derive(Debug, Clone, Copy, Default)]
pub struct GxCommand {
    pub command: u8,
    pub param: u32,
}

/// 3D GPU core
pub struct Gpu3D {
    emulator: Option<Arc<Mutex<crate::emulator::Emulator>>>,
    gpu: Option<Arc<Mutex<crate::gpu::Gpu>>>,

    cycles: i32,

    disp3dcnt: Disp3dCnt,
    polygon_attr: PolygonAttr,
    teximage_param: TexImageParam,

    toon_table: [u16; 32],
    pltt_base: u32,
    viewport: Viewport,
    gxstat: GxStat,
    polygon_type: u32,
    clear_depth: u32,
    clear_color: u32,
    flush_mode: i32,

    gx_fifo: std::collections::VecDeque<GxCommand>,
    gx_pipe: std::collections::VecDeque<GxCommand>,

    cmd_params: [u32; 32],
    param_count: u8,
    cmd_param_count: u8,
    cmd_count: u8,
    total_params: u8,
    current_cmd: u32,
    current_poly_attr: PolygonAttr,

    current_color: u32,
    current_vertex: [i16; 3],
    current_texcoords: [i16; 2],

    z_buffer: Vec<Vec<u32>>,
    trans_poly_ids: Vec<u8>,

    swap_buffers: bool,

    // Command tables (static in C++)
    // We'll expose as functions returning defaults for now
    geo_vert: Vec<Vertex>,
    rend_vert: Vec<Vertex>,
    geo_poly: Vec<Polygon>,
    rend_poly: Vec<Polygon>,
    last_poly_strip: Option<usize>,

    vertex_list: [Vertex; 10],
    vertex_list_count: i32,

    geo_vert_count: i32,
    rend_vert_count: i32,
    geo_poly_count: i32,
    rend_poly_count: i32,

    consecutive_polygons: i32,

    vtx_16_index: i32,

    mtx_mode: u8,

    projection_mtx: Mtx,
    vector_mtx: Mtx,
    modelview_mtx: Mtx,
    texture_mtx: Mtx,
    projection_stack: Mtx,
    texture_stack: Mtx,
    modelview_stack: Vec<Mtx>,
    vector_stack: Vec<Mtx>,
    clip_mtx: Mtx,
    clip_dirty: bool,
    modelview_sp: u8,

    emission_color: u16,
    ambient_color: u16,
    diffuse_color: u16,
    specular_color: u16,
    light_color: [u16; 4],
    light_direction: [[i16; 3]; 4],
    normal_vector: [i16; 3],
    shine_table: [u8; 128],
    using_shine_table: bool,

    vec_test_result: [i16; 3],

    mult_params: Mtx,
    mult_params_index: i32,
}

impl Gpu3D {
    /// Create a new 3D GPU instance
    pub fn new() -> Self {
        // initialize z_buffer as SCANLINES x PIXELS_PER_LINE filled with zeros
        let mut zbuf = Vec::with_capacity(SCANLINES as usize);
        for _ in 0..(SCANLINES as usize) {
            zbuf.push(vec![0u32; PIXELS_PER_LINE as usize]);
        }

        Gpu3D {
            emulator: None,
            gpu: None,
            cycles: 0,
            disp3dcnt: Disp3dCnt::default(),
            polygon_attr: PolygonAttr::default(),
            teximage_param: TexImageParam::default(),
            toon_table: [0u16; 32],
            pltt_base: 0,
            viewport: Viewport::default(),
            gxstat: GxStat::default(),
            polygon_type: 0,
            clear_depth: 0,
            clear_color: 0,
            flush_mode: 0,
            gx_fifo: std::collections::VecDeque::new(),
            gx_pipe: std::collections::VecDeque::new(),
            cmd_params: [0u32; 32],
            param_count: 0,
            cmd_param_count: 0,
            cmd_count: 0,
            total_params: 0,
            current_cmd: 0,
            current_poly_attr: PolygonAttr::default(),
            current_color: 0,
            current_vertex: [0i16; 3],
            current_texcoords: [0i16; 2],
            z_buffer: zbuf,
            trans_poly_ids: vec![0u8; PIXELS_PER_LINE as usize],
            swap_buffers: false,
            geo_vert: vec![Vertex::default(); 6188],
            rend_vert: vec![Vertex::default(); 6188],
            geo_poly: vec![Polygon::default(); 2048],
            rend_poly: vec![Polygon::default(); 2048],
            last_poly_strip: None,
            vertex_list: [
                Vertex::default(),
                Vertex::default(),
                Vertex::default(),
                Vertex::default(),
                Vertex::default(),
                Vertex::default(),
                Vertex::default(),
                Vertex::default(),
                Vertex::default(),
                Vertex::default(),
            ],
            vertex_list_count: 0,
            geo_vert_count: 0,
            rend_vert_count: 0,
            geo_poly_count: 0,
            rend_poly_count: 0,
            consecutive_polygons: 0,
            vtx_16_index: 0,
            mtx_mode: 0,
            projection_mtx: Mtx::default(),
            vector_mtx: Mtx::default(),
            modelview_mtx: Mtx::default(),
            texture_mtx: Mtx::default(),
            projection_stack: Mtx::default(),
            texture_stack: Mtx::default(),
            modelview_stack: vec![Mtx::default(); 0x20],
            vector_stack: vec![Mtx::default(); 0x20],
            clip_mtx: Mtx::default(),
            clip_dirty: false,
            modelview_sp: 0,
            emission_color: 0,
            ambient_color: 0,
            diffuse_color: 0,
            specular_color: 0,
            light_color: [0u16; 4],
            light_direction: [[0i16; 3]; 4],
            normal_vector: [0i16; 3],
            shine_table: [0u8; 128],
            using_shine_table: false,
            vec_test_result: [0i16; 3],
            mult_params: Mtx::default(),
            mult_params_index: 0,
        }
    }

    /// Power on GPU 3D unit
    pub fn power_on(&mut self) {
        self.cycles = 0;
        self.gx_fifo.clear();
        self.gx_pipe.clear();
        // reset other state as needed
    }

    /// Render a single scanline into the provided framebuffer
    pub fn render_scanline(
        &mut self,
        _framebuffer: &mut [u32],
        _bg_priorities: &[u8],
        _bg0_priority: u8,
    ) {
        // Stubbed: detailed rasterization not implemented
    }

    /// Run the 3D engine for given cycles
    pub fn run(&mut self, _cycles_to_run: u64) {
        // Process commands from GXFIFO
        while let Some(cmd) = self.gx_fifo.pop_front() {
            // execute or queue
            self.exec_command(cmd);
        }
    }

    /// Called at end of frame
    pub fn end_of_frame(&mut self) {
        // swap buffers if requested
        if self.swap_buffers {
            self.swap_buffers = false;
        }
    }

    /// Check FIFO DMA (stub)
    pub fn check_fifo_dma(&mut self) {}

    /// Check FIFO IRQ (stub)
    pub fn check_fifo_irq(&mut self) {}

    /// Write a 32-bit word into GXFIFO (incoming command)
    pub fn write_gxfifo(&mut self, word: u32) {
        // rudimentary command push: low byte = command, param = word
        let cmd = GxCommand {
            command: (word & 0xFF) as u8,
            param: word,
        };
        self.gx_fifo.push_back(cmd);
    }

    /// Direct FIFO write (addressed)
    pub fn write_fifo_direct(&mut self, _address: u32, word: u32) {
        self.write_gxfifo(word);
    }

    /// Get raw DISP3DCNT register as 16-bit value
    pub fn get_disp3dcnt(&self) -> u16 {
        // pack bits (stub)
        0
    }

    /// Get GXSTAT register value
    pub fn get_gxstat(&self) -> u32 {
        0
    }

    /// Get vertex count
    pub fn get_vert_count(&self) -> u16 {
        self.rend_vert_count as u16
    }

    /// Get polygon count
    pub fn get_poly_count(&self) -> u16 {
        self.rend_poly_count as u16
    }

    /// Read clip matrix value at address (stub)
    pub fn read_clip_mtx(&self, _address: u32) -> u32 {
        0
    }
    pub fn read_vec_mtx(&self, _address: u32) -> u32 {
        0
    }
    pub fn read_vec_test(&self, _address: u32) -> u16 {
        0
    }

    /// Set DISP3DCNT register from halfword
    pub fn set_disp3dcnt(&mut self, _halfword: u16) {}

    /// Set clear color/depth
    pub fn set_clear_color(&mut self, word: u32) {
        self.clear_color = word;
    }
    pub fn set_clear_depth(&mut self, word: u32) {
        self.clear_depth = word;
    }

    /// Matrix mode and stack operations (stubs)
    pub fn set_mtx_mode(&mut self, _word: u32) {}
    pub fn mtx_push(&mut self) {}
    pub fn mtx_pop(&mut self, _word: u32) {}
    pub fn mtx_identity(&mut self) {}
    pub fn mtx_mult_4x4(&mut self, _word: u32) {}
    pub fn mtx_mult_4x3(&mut self, _word: u32) {}
    pub fn mtx_mult_3x3(&mut self, _word: u32) {}
    pub fn mtx_trans(&mut self, _word: u32) {}

    /// Color and normal commands
    pub fn color(&mut self, _word: u32) {}
    pub fn normal(&mut self) {}

    /// Polygon / texture setup
    pub fn set_polygon_attr(&mut self, _word: u32) {}
    pub fn set_teximage_param(&mut self, _word: u32) {}
    pub fn set_toon_table(&mut self, _address: u32, _color: u16) {}

    /// Begin vertices, swap buffers, viewport, tests
    pub fn begin_vtxs(&mut self, _word: u32) {}
    pub fn swap_buffers(&mut self, _word: u32) {
        self.swap_buffers = true;
    }
    pub fn viewport_cmd(&mut self, _word: u32) {}
    pub fn box_test(&mut self) {}
    pub fn vec_test(&mut self) {}
    pub fn set_gxstat(&mut self, _word: u32) {}

    // Internal helpers
    fn read_command(&mut self) -> Option<GxCommand> {
        self.gx_fifo.pop_front()
    }

    fn write_command(&mut self, cmd: GxCommand) {
        self.gx_pipe.push_back(cmd);
    }

    fn exec_command(&mut self, cmd: GxCommand) {
        // Very small dispatcher based on command byte
        match cmd.command {
            // 0x00 - example: clear buffers
            0x00 => {
                // handle clear
            }
            _ => {
                // unimplemented
            }
        }
    }

    fn add_mult_param(&mut self, _word: u32) {}
    fn mtx_mult(&mut self, _update_vector: bool) {}
    fn update_clip_mtx(&mut self) {}

    fn clip(
        &mut self,
        _v_list: &mut [Vertex],
        _v_len: i32,
        _clip_start: i32,
        _add_attributes: bool,
    ) -> i32 {
        0
    }
    fn clip_plane(
        &mut self,
        _plane: i32,
        _v_list: &mut [Vertex],
        _v_len: i32,
        _clip_start: i32,
        _add_attributes: bool,
    ) -> i32 {
        0
    }
    fn clip_vertex(
        &mut self,
        _plane: i32,
        _v_list: &mut Vertex,
        _v_out: &mut Vertex,
        _v_in: &mut Vertex,
        _side: i32,
        _add_attributes: bool,
    ) {
    }
    fn add_vertex(&mut self) {}
    fn add_polygon(&mut self) {}

    fn request_fifo_dma(&mut self) {}
}

impl Default for Gpu3D {
    fn default() -> Self {
        Self::new()
    }
}
