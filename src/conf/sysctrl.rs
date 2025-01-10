use serde::{Deserialize, Serialize};

use crate::types::sysctrl::{FlashInfo, ForceOldImage, SpiConfig, SystemData};

use super::DataArray;

/// Configuration for the system data, containing options for various system parameters.
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemDataCfg {
    /// The address of the second OTA partition, if available.
    pub ota2_addr: Option<u32>,
    /// The size of the second OTA partition, if available.
    pub ota2_size: Option<u32>,

    ///  Configuration for the old image trap, typically used to control whether an old image is forced into memory.
    #[serde(default)]
    pub old_img_trap: ForceOldImage,

    /// Configuration of the SPI interface, including IO mode and speed.
    #[serde(default)]
    pub spi_cfg: SpiConfig,

    /// Information related to the flash memory (e.g., flash ID, size).
    #[serde(default)]
    pub flash_info: FlashInfo,

    /// Baud rate for the UART logging interface.
    pub ulog_baud: Option<u32>,

    /// SPI calibration configuration, stored as raw data.
    pub spic_calibcfg: Option<DataArray<0x30>>,

    /// Bluetooth parameter data, stored as raw data.
    pub bt_parameter_data: Option<DataArray<0x20>>,
}

impl Default for SystemDataCfg {
    /// Returns the default configuration for `SystemDataCfg`.
    ///
    /// # Returns
    /// A `SystemDataCfg` with default values.
    fn default() -> Self {
        Self {
            ota2_addr: None,
            ota2_size: None,
            old_img_trap: ForceOldImage::default(),
            spi_cfg: SpiConfig::default(),
            flash_info: FlashInfo::default(),
            ulog_baud: None,
            spic_calibcfg: None,
            bt_parameter_data: None,
        }
    }
}

impl TryInto<SystemData> for SystemDataCfg {
    type Error = crate::error::Error;

    /// Tries to convert `SystemDataCfg` into a `SystemData` instance.
    ///
    /// This method attempts to convert the configuration (`SystemDataCfg`) into a complete
    /// `SystemData` object. It populates the `SystemData` fields based on the values in the
    /// configuration struct, ensuring that required fields are filled, and default values are used
    /// where no data is provided.
    ///
    /// # Parameters
    /// - `self`: The `SystemDataCfg` instance to be converted.
    ///
    /// # Returns
    /// A `Result<SystemData, Error>`:
    /// - `Ok(SystemData)`: The conversion succeeded, and the data is populated.
    /// - `Err(Error)`: An error occurred during conversion, for example, invalid data during
    ///   conversion (e.g., when trying to convert data that doesn't fit the expected type).
    ///
    /// # Example
    /// ```rust
    /// let config = SystemDataCfg::default();
    /// let sysdata: SystemData = config.try_into().unwrap();
    /// ```
    fn try_into(self) -> Result<SystemData, Self::Error> {
        let mut sysdata = SystemData::default();
        sysdata.ota2_addr = self.ota2_addr;
        sysdata.ota2_size = self.ota2_size;
        sysdata.old_img_trap = self.old_img_trap;
        sysdata.spi_cfg = self.spi_cfg;
        sysdata.flash_info = self.flash_info;
        sysdata.ulog_baud = self.ulog_baud.unwrap_or(0xFFFF_FFFF);

        if let Some(bt_parameter_data) = self.bt_parameter_data {
            // revisit: custom error handling
            sysdata.set_pt_paramdata(Some(bt_parameter_data.data.try_into().unwrap()));
        }

        if let Some(spic_calibcfg) = self.spic_calibcfg {
            sysdata.set_spic_calibcfg(Some(spic_calibcfg.data.try_into().unwrap()));
        }

        Ok(sysdata)
    }
}
