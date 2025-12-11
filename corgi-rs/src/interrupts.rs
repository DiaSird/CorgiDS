/// Interrupt definitions and registers for the emulator
/// Provides IRQ constants and helper for checking pending requests

/// Interrupt sources used by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    VBlank,
    HBlank,
    VCountMatch,
    Timer0,
    Timer1,
    Timer2,
    Timer3,
    RTC,
    DMA0,
    DMA1,
    DMA2,
    DMA3,
    Keypad,
    GBASlot,
    IPCSync = 16,
    IPCFifoEmpty,
    IPCFifoNEmpty,
    CartTransfer,
    CartIReqMc,
    GeometryFifo,
    UnfoldScreen,
    SPI,
    WiFi,
}

impl From<Interrupt> for u32 {
    fn from(i: Interrupt) -> u32 {
        match i {
            Interrupt::VBlank => 0,
            Interrupt::HBlank => 1,
            Interrupt::VCountMatch => 2,
            Interrupt::Timer0 => 3,
            Interrupt::Timer1 => 4,
            Interrupt::Timer2 => 5,
            Interrupt::Timer3 => 6,
            Interrupt::RTC => 7,
            Interrupt::DMA0 => 8,
            Interrupt::DMA1 => 9,
            Interrupt::DMA2 => 10,
            Interrupt::DMA3 => 11,
            Interrupt::Keypad => 12,
            Interrupt::GBASlot => 13,
            Interrupt::IPCSync => 16,
            Interrupt::IPCFifoEmpty => 17,
            Interrupt::IPCFifoNEmpty => 18,
            Interrupt::CartTransfer => 19,
            Interrupt::CartIReqMc => 20,
            Interrupt::GeometryFifo => 21,
            Interrupt::UnfoldScreen => 22,
            Interrupt::SPI => 23,
            Interrupt::WiFi => 24,
        }
    }
}

/// Interrupt register block
#[derive(Debug, Clone, Copy)]
pub struct InterruptRegs {
    /// Interrupt Master Enable (IME)
    pub ime: u32,
    /// Interrupt Enable (IE) bitmask
    pub ie: u32,
    /// Interrupt Flags (IF) bitmask
    pub iflags: u32,
}

impl InterruptRegs {
    /// Check whether a given interrupt (bit mask) is requesting service
    /// Returns true if enabled and flagged
    pub fn is_requesting_int(&self, bit_mask: u32) -> bool {
        (self.ie & bit_mask) != 0 && (self.iflags & bit_mask) != 0
    }
}

impl Default for InterruptRegs {
    fn default() -> Self {
        InterruptRegs { ime: 0, ie: 0, iflags: 0 }
    }
}
