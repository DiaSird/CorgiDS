/// Touchscreen controller for Nintendo DS
/// Implements ADS7843-compatible SPI touchscreen interface

/// Touchscreen controller
/// Communicates via SPI protocol (ADS7843 compatible)
pub struct TouchScreen {
    /// Control byte for current operation
    control_byte: u8,
    /// Current output coordinate data
    output_coords: u16,
    /// Position in data transfer
    data_pos: i32,

    /// Pressed X coordinate (0-4095)
    press_x: u16,
    /// Pressed Y coordinate (0-4095)
    press_y: u16,

    /// Pen down state
    pen_down: bool,
}

impl TouchScreen {
    /// Create new touchscreen controller
    pub fn new() -> Self {
        TouchScreen {
            control_byte: 0,
            output_coords: 0,
            data_pos: 0,
            press_x: 0,
            press_y: 0,
            pen_down: false,
        }
    }

    /// Power on touchscreen
    pub fn power_on(&mut self) -> Result<(), String> {
        self.pen_down = false;
        self.press_x = 0;
        self.press_y = 0;
        Ok(())
    }

    /// Handle press event at screen coordinates
    pub fn press_event(&mut self, x: i32, y: i32) {
        // Convert screen coordinates to ADC values (0-4095)
        // Screen is 256x192 pixels, touchscreen is ~4096 steps each direction
        self.press_x = ((x as u32 * 4096) / 256) as u16;
        self.press_y = ((y as u32 * 4096) / 192) as u16;
        self.pen_down = true;
    }

    /// Release pen from touchscreen
    pub fn release(&mut self) {
        self.pen_down = false;
    }

    /// Transfer data via SPI
    /// Returns data to send to ARM7/ARM9
    pub fn transfer_data(&mut self, input: u8) -> u8 {
        // ADS7843 protocol:
        // Byte 0: Control byte (channel select, reference, ADC mode)
        // Bytes 1-2: ADC result (12-bit)

        match self.data_pos {
            0 => {
                // Receive control byte and send upper byte of X
                self.control_byte = input;
                self.data_pos = 1;

                // Extract command from control byte
                let channel = (self.control_byte >> 4) & 0x7;

                match channel {
                    0x0 => {
                        // Measure Y coordinate
                        if self.pen_down {
                            self.output_coords = self.press_y;
                        } else {
                            self.output_coords = 0xFFF; // Released
                        }
                    }
                    0x1 => {
                        // Measure X coordinate
                        if self.pen_down {
                            self.output_coords = self.press_x;
                        } else {
                            self.output_coords = 0xFFF; // Released
                        }
                    }
                    0x3 => {
                        // Measure Z1 (pressure)
                        if self.pen_down {
                            self.output_coords = 2000; // Dummy pressure value
                        } else {
                            self.output_coords = 0;
                        }
                    }
                    0x4 => {
                        // Measure Z2 (pressure continuation)
                        if self.pen_down {
                            self.output_coords = 1000; // Dummy pressure value
                        } else {
                            self.output_coords = 0;
                        }
                    }
                    0x5 => {
                        // Measure temperature 0
                        self.output_coords = 0;
                    }
                    0x6 => {
                        // Measure temperature 1
                        self.output_coords = 0;
                    }
                    _ => {
                        self.output_coords = 0;
                    }
                }

                // Return upper byte of coordinate data
                ((self.output_coords >> 4) & 0xF0) as u8
            }
            1 => {
                // Send lower byte and upper nibble
                self.data_pos = 2;
                ((self.output_coords & 0xFF) as u8) << 4
            }
            _ => {
                // After complete transfer
                self.data_pos = 0;
                0
            }
        }
    }

    /// Deselect touchscreen (end of transfer)
    pub fn deselect(&mut self) {
        self.data_pos = 0;
    }

    // Getter methods

    /// Get pressed X coordinate
    pub fn get_press_x(&self) -> u16 {
        self.press_x
    }

    /// Get pressed Y coordinate
    pub fn get_press_y(&self) -> u16 {
        self.press_y
    }

    /// Check if pen is down
    pub fn is_pen_down(&self) -> bool {
        self.pen_down
    }

    /// Set pressed X coordinate directly
    pub fn set_press_x(&mut self, x: u16) {
        self.press_x = x;
    }

    /// Set pressed Y coordinate directly
    pub fn set_press_y(&mut self, y: u16) {
        self.press_y = y;
    }

    /// Set pen down state directly
    pub fn set_pen_down(&mut self, down: bool) {
        self.pen_down = down;
    }

    /// Convert ADC value to screen pixel
    pub fn adc_to_pixel_x(adc: u16) -> i32 {
        ((adc as i32 * 256) / 4096).min(255).max(0)
    }

    /// Convert ADC value to screen pixel Y
    pub fn adc_to_pixel_y(adc: u16) -> i32 {
        ((adc as i32 * 192) / 4096).min(191).max(0)
    }

    /// Convert pixel to ADC value X
    pub fn pixel_to_adc_x(pixel: i32) -> u16 {
        ((pixel as u32 * 4096) / 256) as u16
    }

    /// Convert pixel to ADC value Y
    pub fn pixel_to_adc_y(pixel: i32) -> u16 {
        ((pixel as u32 * 4096) / 192) as u16
    }
}

impl Default for TouchScreen {
    fn default() -> Self {
        Self::new()
    }
}
