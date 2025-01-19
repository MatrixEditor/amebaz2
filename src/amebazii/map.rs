/// Represents a memory address range, defined by a starting address and an ending address.
///
/// This struct is used to define ranges of memory addresses, providing utilities to check
/// if a specific address falls within the range. The range is inclusive of the start address
/// and exclusive of the end address.
#[derive(Debug, Clone, Copy)]
pub struct AddressRange(u64, u64);

/// The vector table, it must start with 256 bytes aligned address.
pub const VECTORS_RAM: AddressRange = AddressRange::new(0x10000000, 0x100000A0);

/// RAM functions entry table, storing function entries in RAM.
pub const RAM_FUN_TABLE: AddressRange = AddressRange::new(0x10000480, 0x100004F0);

/// RAM image signature, used for storing the image signature in RAM.
pub const RAM_IMG_SIGN: AddressRange = AddressRange::new(0x100004F0, 0x10000500);

/// Internal RAM for program data and text (Data and Text are loaded into DTCM RAM).
pub const DTCM_RAM: AddressRange = AddressRange::new(0x10000500, 0x1003FA00);

/// Extension RAM for heap, used as dynamic memory.
pub const EXTENSION_RAM: AddressRange = AddressRange::new(0x10040000, 0x10060000);

/// External PSRAM for storing text, read-only data (RODATA), and data sections.
pub const PSRAM: AddressRange = AddressRange::new(0x60000000, 0x60400000);

/// XIP Chiper section, where TEXT and RODATA sections **can** be encrypted.
///
/// **Note**: There's some reserved data at the start of each XIP Flash section (0x140)
/// bytes. We don't include them here.
pub const XIP_FLASH_C: AddressRange = AddressRange::new(0x9B000140, 0x9B800000);

/// XIP Plaintext section, where RODATA is not encrypted.
///
/// **Note**: There's some reserved data at the start of each XIP Flash section (0x140)
/// bytes. We don't include them here.
pub const XIP_FLASH_P: AddressRange = AddressRange::new(0x9B800140, 0x9BFF0000);

impl AddressRange {
    /// Creates a new `AddressRange` instance with a given start and end address.
    ///
    /// # Parameters
    /// - `start`: The starting address of the range (inclusive).
    /// - `end`: The ending address of the range (exclusive).
    ///
    /// # Returns
    /// Returns a new `AddressRange` representing the memory range between `start` and `end`.
    pub const fn new(start: u64, end: u64) -> Self {
        AddressRange(start, end)
    }

    /// Returns the length of the address range, calculated as the difference between the end and start addresses.
    ///
    /// # Returns
    /// Returns the length of the range as a `u64`.
    pub const fn len(&self) -> u64 {
        self.1 - self.0
    }

    /// Returns the start address of the range.
    ///
    /// # Returns
    /// The start address of the range as a `u64`.
    pub const fn start(&self) -> u64 {
        self.0
    }

    /// Returns the end address of the range.
    ///
    /// # Returns
    /// The end address of the range as a `u64`.
    pub const fn end(&self) -> u64 {
        self.1
    }

    /// Checks if the provided address falls within the range (inclusive of the start, exclusive of the end).
    ///
    /// # Parameters
    /// - `addr`: The address to check.
    ///
    /// # Returns
    /// Returns `true` if the address is within the range, `false` otherwise.
    #[inline]
    pub fn contains(&self, addr: u64) -> bool {
        self.0 <= addr && addr < self.1
    }
}
