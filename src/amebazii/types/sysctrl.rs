use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io;

use crate::{
    error::Error, is_valid_data, read_padding, read_valid_data, util::write_fill, write_data,
    write_padding,
};

use super::{
    enums::{FlashSize, SpiIOMode, SpiSpeed},
    DataRefType, DataType, FromStream, ToStream,
};

/// Represents the configuration for forcing the use of an old image.
///
/// This struct is used to store and manipulate the settings for whether an old image
/// (e.g., firmware or partition) should be used, and how it should be configured.
/// The configuration consists of three components:
/// - `pin`: A pin number (0-31) used in the configuration.
/// - `port`: A port number (0 or 1), which determines whether the configuration uses a specific port.
/// - `active`: A boolean flag indicating whether the old image configuration is active or not.
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ForceOldImage {
    /// The pin number (0-31) used in the configuration.
    pin: u8,

    /// The port number (0 or 1), which determines the port configuration.
    port: u8,

    /// A flag that determines if the old image configuration is active.
    active: bool,
}

impl ForceOldImage {
    /// Creates a new instance of `ForceOldImage` with the specified pin, port, and active status.
    ///
    /// # Returns:
    /// - A new `ForceOldImage` struct.
    pub fn new(pin: u8, port: u8, active: bool) -> Self {
        Self { pin, port, active }
    }

    /// Retrieves the pin number of the configuration.
    ///
    /// # Returns:
    /// - The pin number (0-31) used in the configuration.
    pub fn pin(&self) -> u8 {
        self.pin
    }

    /// Retrieves the port number of the configuration.
    ///
    /// # Returns:
    /// - The port number (0 or 1) used in the configuration.
    pub fn port(&self) -> u8 {
        self.port
    }

    /// Checks whether the old image configuration is active.
    ///
    /// # Returns:
    /// - `true` if the configuration is active, `false` otherwise.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl From<u32> for ForceOldImage {
    /// Converts a 32-bit unsigned integer into a `ForceOldImage` instance.
    ///
    /// # Parameters:
    /// - `value`: A `u32` integer representing the configuration.
    ///
    /// # Returns:
    /// - A `ForceOldImage` struct with the appropriate values extracted from the `u32`.
    fn from(value: u32) -> Self {
        Self {
            pin: (value & 0b11111) as u8,
            port: ((value >> 5) & 0b1) as u8,
            active: ((value >> 7) & 0b1) == 1,
        }
    }
}

impl Default for ForceOldImage {
    /// Returns a default `ForceOldImage` configuration with all fields set to their default values.
    ///
    /// The default values are:
    /// - `pin`: 0
    /// - `port`: 0
    /// - `active`: false
    ///
    /// # Returns:
    /// - A `ForceOldImage` instance with default values.
    fn default() -> Self {
        Self {
            pin: 0,
            port: 0,
            active: false,
        }
    }
}

impl Into<u32> for ForceOldImage {
    /// Converts a `ForceOldImage` instance into a 32-bit unsigned integer.
    ///
    /// # Returns:b
    /// - A `u32` integer representing the `ForceOldImage` configuration.
    fn into(self) -> u32 {
        (self.pin as u32) | ((self.port as u32) << 5) | ((self.active as u32) << 7)
    }
}

/// Represents the SPI (Serial Peripheral Interface) configuration.
///
/// Example:
/// ```rust
/// // Create a new SPI configuration instance.
/// let spi_config = SpiConfig {
///     io_mode: SpiIOMode::Quad_IO,
///     io_speed: SpiSpeed::_50MHz,
/// };
///
/// // Convert the SPI config into a 32-bit integer.
/// let packed_value: u32 = spi_config.into();
///
/// // Convert the 32-bit integer back into an SPI config.
/// let unpacked_value = SpiConfig::from(packed_value);
///
/// // Print the values from the unpacked config.
/// println!("SPI IO Mode: {:?}", unpacked_value.io_mode);
/// println!("SPI IO Speed: {:?}", unpacked_value.io_speed);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpiConfig {
    /// The SPI I/O mode (defines the number of I/O lines and mode of operation).
    io_mode: SpiIOMode,

    /// The SPI communication speed (defines the frequency at which data is transferred).
    io_speed: SpiSpeed,
}

impl Default for SpiConfig {
    /// Returns the default `SpiConfig` with default I/O mode and I/O speed.
    ///
    /// # Returns:
    /// - A `SpiConfig` instance with default values for `io_mode` and `io_speed`.
    fn default() -> Self {
        Self {
            io_mode: SpiIOMode::default(),
            io_speed: SpiSpeed::default(),
        }
    }
}

impl Into<u32> for SpiConfig {
    /// Converts a `SpiConfig` instance into a 32-bit unsigned integer.
    ///
    /// # Parameters:
    /// - `self`: The `SpiConfig` instance to convert.
    ///
    /// # Returns:
    /// - A `u32` integer that represents the `SpiConfig` instance in a packed format.
    fn into(self) -> u32 {
        (self.io_mode as u32) | ((self.io_speed as u32) << 16)
    }
}

impl From<u32> for SpiConfig {
    /// Converts a 32-bit unsigned integer into a `SpiConfig` instance.
    ///
    /// # Parameters:
    /// - `value`: The `u32` integer to convert.
    ///
    /// # Returns:
    /// - A `SpiConfig` instance with the `io_mode` and `io_speed` fields populated.
    fn from(value: u32) -> Self {
        Self {
            io_mode: ((value & 0xFFFF) as u16).into(),
            io_speed: (((value >> 16) & 0xFFFF) as u16).into(),
        }
    }
}

/// Represents information about the flash memory configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FlashInfo {
    /// The flash memory's unique identifier (e.g., a manufacturer ID or model number).
    flash_id: u16,

    /// The size of the flash memory, represented by the `FlashSize` enum.
    flash_size: FlashSize,
}

impl Default for FlashInfo {
    /// Returns the default `FlashInfo` instance.
    ///
    /// # Returns:
    /// - A `FlashInfo` instance with a `flash_id` of 0 and the default `flash_size`.
    fn default() -> Self {
        Self {
            flash_id: 0,
            flash_size: FlashSize::default(),
        }
    }
}

impl Into<u32> for FlashInfo {
    /// Converts a `FlashInfo` instance into a 32-bit unsigned integer.
    ///
    /// # Parameters:
    /// - `self`: The `FlashInfo` instance to convert.
    ///
    /// # Returns:
    /// - A `u32` integer that represents the `FlashInfo` instance in a packed format.
    fn into(self) -> u32 {
        (self.flash_id as u32) | ((self.flash_size as u32) << 16)
    }
}

impl From<u32> for FlashInfo {
    /// Converts a 32-bit unsigned integer into a `FlashInfo` instance.
    ///
    /// # Parameters:
    /// - `value`: The `u32` integer to convert.
    ///
    /// # Returns:
    /// - A `FlashInfo` instance with the `flash_id` and `flash_size` fields populated.
    fn from(value: u32) -> Self {
        Self {
            flash_id: (value & 0xFFFF) as u16,  // Extract the first 16 bits for flash_id
            flash_size: (((value >> 16) & 0xFFFF) as u16).into(),  // Extract the next 16 bits for flash_size
        }
    }
}


// Although hal_sys_ctrl.h only defines the first two fields as reserved, there are actually more:
//
//  1. OTA_Change(int current) in ota_8710c_ram_lib_soc.o implements the behaviour by
//     changing the current OTA address
//
// Layout:
//          +-------+------+-------+-------+--------+------+----------+---------+------+------+-------+------+----+----+----+----+
//          | 0     | 1    | 2     | 3     | 4      | 5    | 6        | 7       | 8    | 9    | 10    | 11   | 12 | 13 | 14 | 15 |
// +========+=======+======+=======+=======+========+======+==========+=========+======+======+=======+======+====+====+====+====+
// | 0x00   |      ota2_address: u32       |           ota2_size: u32           | force_old_ota: ForceOldOTA |                   |
// +--------+------------------------------+------------------------------------+----------------------------+-------------------+
// | 0x10   |                                                                                                                    |
// +--------+--------------+---------------+---------------+--------------------+------------------------------------------------+
// | 0x20   | io_mode: u16 | io_speed: u16 | flash_id: u16 | flash_size_mb: u16 |                                                |
// +--------+--------------+---------------+---------------+--------------------+------------------------------------------------+
// | 0x30   |        ulog_baud: u32        |                                                                                     |
// +--------+------------------------------+-------------------------------------------------------------------------------------+
// | 0x40   |                                        spic_calibration_setting: bytes[0x30]                                       |
// +--------+--------------------------------------------------------------------------------------------------------------------+
// | 0x70   |                                                                                                                    |
// +--------+--------------------------------------------------------------------------------------------------------------------+
// | 0xFE0  |                                            bt_parameter_data: bytes[0x20]                                          |
// +--------+--------------------------------------------------------------------------------------------------------------------+

/// Represents system data related to the device, including configuration for OTA, SPI, and other hardware settings.
///
/// The `SystemData` struct holds various fields representing the system's configuration and settings:
/// - **OTA**: Configurations for over-the-air (OTA) updates, including address and size of OTA2.
/// - **Force Old Image**: Settings related to forcing the usage of an older firmware image.
/// - **SPI Configuration**: Configuration for the Serial Peripheral Interface (SPI) bus.
/// - **Flash Information**: Information about the flash memory, including ID and size.
/// - **Logging Baud Rate**: Baud rate for UART logging.
/// - **Calibration Data**: SPI calibration data and Bluetooth parameters.
///
/// This struct is designed to be serialized and deserialized from a stream, allowing for easy reading and writing
/// of system settings, especially in embedded systems or firmware configuration.
#[derive(Debug)]
pub struct SystemData {
    /// Address of the OTA2 image, or `None` if OTA1 is active.
    pub ota2_addr: Option<u32>,

    /// Size of the OTA2 image, or `None` if OTA1 is active.
    pub ota2_size: Option<u32>,

    /// Configuration for forcing the usage of an older image, including the pin, port, and active status.
    pub old_img_trap: ForceOldImage,

    /// Configuration for the SPI bus, including IO mode and speed.
    pub spi_cfg: SpiConfig,

    /// Information about the flash memory, including ID and size.
    pub flash_info: FlashInfo,

    /// Baud rate for the UART logging interface.
    pub ulog_baud: u32,

    /// SPI calibration configuration, stored as raw data.
    spic_calibcfg: DataType<0x30>,

    /// Bluetooth parameter data, stored as raw data.
    bt_parameter_data: DataType<0x20>,
}

impl Default for SystemData {
    /// Returns the default `SystemData` instance with preset values.
    ///
    /// Default values:
    /// - `ota2_addr`: `None` (indicating OTA1 is active).
    /// - `ota2_size`: `None`.
    /// - `old_img_trap`: Default `ForceOldImage` values.
    /// - `spi_cfg`: Default `SpiConfig` values.
    /// - `flash_info`: Default `FlashInfo` values.
    /// - `ulog_baud`: `0xFFFF_FFFF`.
    /// - `spic_calibcfg`: `None`.
    /// - `bt_parameter_data`: `None`.
    fn default() -> Self {
        Self {
            ota2_addr: None,
            ota2_size: None,
            old_img_trap: ForceOldImage::default(),
            spi_cfg: SpiConfig::default(),
            flash_info: FlashInfo::default(),
            ulog_baud: 0xFFFF_FFFF,
            spic_calibcfg: None,
            bt_parameter_data: None,
        }
    }
}

impl FromStream for SystemData {
    /// Reads `SystemData` from a stream (e.g., file, memory, or network).
    ///
    /// # Parameters:
    /// - `_reader`: The reader from which data will be read (must implement `Read` and `Seek`).
    ///
    /// # Returns:
    /// - A result indicating success or failure of the reading operation.
    fn read_from<R>(&mut self, _reader: &mut R) -> Result<(), Error>
    where
        R: io::Read + io::Seek,
    {
        self.ota2_size = match _reader.read_u32::<LittleEndian>()? {
            0xFFFF_FFFF => None,
            value => Some(value),
        };
        self.ota2_addr = match _reader.read_u32::<LittleEndian>()? {
            0xFFFF_FFFF => None,
            value => Some(value),
        };
        self.old_img_trap = _reader.read_u32::<LittleEndian>()?.into();
        read_padding!(_reader, 20);

        self.spi_cfg = _reader.read_u32::<LittleEndian>()?.into();
        self.flash_info = _reader.read_u32::<LittleEndian>()?.into();
        read_padding!(_reader, 8);

        self.ulog_baud = _reader.read_u32::<LittleEndian>()?;
        read_padding!(_reader, 12);

        read_valid_data!(self.spic_calibcfg, 0x30, _reader);
        read_padding!(_reader, 0xf70);
        read_valid_data!(self.bt_parameter_data, 0x20, _reader);
        Ok(())
    }
}

impl ToStream for SystemData {
    /// Writes `SystemData` to a stream (e.g., file, memory, or network).
    ///
    /// # Parameters:
    /// - `_writer`: The writer to which data will be written (must implement `Write` and `Seek`).
    ///
    /// # Returns:
    /// - A result indicating success or failure of the writing operation.
    fn write_to<W>(&self, _writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        _writer.write_u32::<LittleEndian>(self.ota2_size.unwrap_or(0xFFFF_FFFF))?;
        _writer.write_u32::<LittleEndian>(self.ota2_addr.unwrap_or(0xFFFF_FFFF))?;
        _writer.write_u32::<LittleEndian>(self.old_img_trap.into())?;
        write_padding!(_writer, 20);

        _writer.write_u32::<LittleEndian>(self.spi_cfg.into())?;
        _writer.write_u32::<LittleEndian>(self.flash_info.into())?;
        write_padding!(_writer, 8);

        _writer.write_u32::<LittleEndian>(self.ulog_baud)?;
        write_padding!(_writer, 12);

        write_data!(_writer, self.spic_calibcfg, 0x30);
        write_padding!(_writer, 0xf70);
        write_data!(_writer, self.bt_parameter_data, 0x20);
        Ok(())
    }
}

impl SystemData {
    /// Retrieves the Bluetooth parameter data as a reference.
    ///
    /// # Returns:
    /// - A reference to the Bluetooth parameter data (if available).
    pub fn get_bt_paramdata(&self) -> DataRefType<'_, 32> {
        self.bt_parameter_data.as_ref()
    }

    /// Retrieves the SPI calibration configuration as a reference.
    ///
    /// # Returns:
    /// - A reference to the SPI calibration data (if available).
    pub fn get_spic_calibcfg(&self) -> DataRefType<'_, 48> {
        self.spic_calibcfg.as_ref()
    }

    /// Sets the Bluetooth parameter data.
    ///
    /// # Parameters:
    /// - `data`: The new Bluetooth parameter data to set.
    pub fn set_pt_paramdata(&mut self, data: DataType<32>) {
        self.bt_parameter_data = data;
    }

    /// Sets the SPI calibration configuration.
    ///
    /// # Parameters:
    /// - `data`: The new SPI calibration data to set.
    pub fn set_spic_calibcfg(&mut self, data: DataType<48>) {
        self.spic_calibcfg = data;
    }
}
