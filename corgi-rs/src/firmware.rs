/// Nintendo DS Firmware controller
/// Handles firmware data loading, CRC verification, and SPI data transfer
use std::sync::{Arc, Mutex};

/// Firmware commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareCommand {
    /// No command
    None = 0,
    /// Read status register
    ReadStatusReg = 1,
    /// Read data stream
    ReadStream = 2,
}

impl FirmwareCommand {
    /// Convert numeric value to FirmwareCommand
    pub fn from_value(val: u32) -> Self {
        match val {
            0 => FirmwareCommand::None,
            1 => FirmwareCommand::ReadStatusReg,
            2 => FirmwareCommand::ReadStream,
            _ => FirmwareCommand::None,
        }
    }
}

/// Nintendo DS Firmware
/// Stores firmware data and manages SPI communication
pub struct Firmware {
    /// Emulator reference
    emulator: Option<Arc<Mutex<crate::emulator::Emulator>>>,

    /// Firmware data (262 KB)
    firmware: Vec<u8>,
    /// Status register
    status_reg: u8,
    /// User data section
    user_data: i32,

    /// Current command
    command_id: FirmwareCommand,
    /// Current address
    address: u32,
    /// Total arguments for command
    total_args: i32,
}

impl Firmware {
    /// Firmware size in bytes (256 KB)
    pub const SIZE: usize = 1024 * 256;

    /// Create new Firmware controller
    pub fn new() -> Self {
        Firmware {
            emulator: None,
            firmware: vec![0u8; Self::SIZE],
            status_reg: 0,
            user_data: 0,
            command_id: FirmwareCommand::None,
            address: 0,
            total_args: 0,
        }
    }

    /// Load firmware from file
    /// Returns number of bytes loaded or error
    pub fn load_firmware(&mut self, _file_name: &str) -> Result<usize, String> {
        // In a real implementation, this would read from a file
        // For now, initialize with default values
        self.status_reg = 0x00;
        self.address = 0;
        Ok(Self::SIZE)
    }

    /// Direct boot - initialize firmware for direct boot mode
    pub fn direct_boot(&mut self) -> Result<(), String> {
        self.status_reg = 0x00;
        self.command_id = FirmwareCommand::None;
        self.address = 0;
        Ok(())
    }

    /// Transfer data byte via SPI
    /// Input: byte to send to firmware
    /// Returns: byte received from firmware
    pub fn transfer_data(&mut self, input: u8) -> u8 {
        match self.command_id {
            FirmwareCommand::None => {
                // Parse command byte
                self.command_id = FirmwareCommand::from_value(input as u32);
                self.total_args = 0;
                self.address = 0;
                0x00
            }
            FirmwareCommand::ReadStatusReg => {
                // Return status register
                self.command_id = FirmwareCommand::None;
                self.status_reg
            }
            FirmwareCommand::ReadStream => {
                // Return firmware data byte
                if (self.address as usize) < self.firmware.len() {
                    let byte = self.firmware[self.address as usize];
                    self.address = self.address.wrapping_add(1);
                    byte
                } else {
                    0x00
                }
            }
        }
    }

    /// Deselect firmware (end SPI transfer)
    pub fn deselect(&mut self) {
        self.command_id = FirmwareCommand::None;
        self.address = 0;
        self.total_args = 0;
    }

    /// Get current firmware byte at address
    pub fn get_byte(&self, address: usize) -> u8 {
        if address < self.firmware.len() {
            self.firmware[address]
        } else {
            0x00
        }
    }

    /// Set firmware byte at address
    pub fn set_byte(&mut self, address: usize, value: u8) {
        if address < self.firmware.len() {
            self.firmware[address] = value;
        }
    }

    /// Get status register value
    pub fn get_status(&self) -> u8 {
        self.status_reg
    }

    /// Set status register value
    pub fn set_status(&mut self, value: u8) {
        self.status_reg = value;
    }

    /// Get firmware data
    pub fn get_firmware(&self) -> &[u8] {
        &self.firmware
    }

    /// Get mutable firmware data
    pub fn get_firmware_mut(&mut self) -> &mut [u8] {
        &mut self.firmware
    }

    // CRC helper functions

    /// Create CRC16 for firmware data
    ///
    /// # Arguments
    /// * `data` - Data to calculate CRC for
    /// * `length` - Length of data
    /// * `start` - Starting offset
    ///
    /// # Returns
    /// CRC16 value
    fn create_crc(data: &[u8], length: usize, start: usize) -> u16 {
        let mut crc = 0xFFFFu16;

        for i in start..(start + length) {
            if i < data.len() {
                let byte = data[i];
                crc = Self::crc_update(crc, byte);
            }
        }

        crc
    }

    /// Update CRC16 with new byte
    fn crc_update(mut crc: u16, byte: u8) -> u16 {
        for _ in 0..8 {
            let carry = (crc ^ (byte as u16)) & 1;
            crc >>= 1;
            if carry != 0 {
                crc ^= 0xA001;
            }
        }
        crc
    }

    /// Verify CRC16 in firmware data
    ///
    /// # Arguments
    /// * `start` - Start offset
    /// * `offset` - Data offset
    /// * `length` - Data length
    /// * `crc_offset` - CRC offset
    ///
    /// # Returns
    /// true if CRC matches, false otherwise
    pub fn verify_crc(
        &self,
        start: usize,
        offset: usize,
        length: usize,
        crc_offset: usize,
    ) -> bool {
        if crc_offset + 2 > self.firmware.len() {
            return false;
        }

        let stored_crc =
            (self.firmware[crc_offset] as u16) | ((self.firmware[crc_offset + 1] as u16) << 8);

        let calculated_crc = Self::create_crc(&self.firmware, length, offset);

        calculated_crc == stored_crc
    }

    /// Get user data section
    pub fn get_user_data(&self) -> i32 {
        self.user_data
    }

    /// Set user data section
    pub fn set_user_data(&mut self, value: i32) {
        self.user_data = value;
    }
}

impl Default for Firmware {
    fn default() -> Self {
        Self::new()
    }
}
