use crate::{
    error::Error,
    util::{hmac_md5, hmac_sha256, md5, sha256},
};

/// Enum representing different image types.
///
/// This enum defines various image types used within the system. The image types
/// are associated with different identifiers, and the enum also includes a fallback
/// type for unknown image types.
///
/// However, note that *image type* refers to the `SubImage` of the firmware image.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ImageType {
    Parttab,
    Boot,
    FHWSS,
    FHWSNS,
    FWLS,
    Isp,
    Voe,   // Video output encoder
    Wln,   // Wireless network ?
    Xip,   // Executable in place
    Wowln, //  Wake-on-Wireless-LAN ?
    Cinit, // Custom initialization ?
    Cpfw,
    Unknown = 0x3F,
}

impl TryFrom<u8> for ImageType {
    type Error = Error;

    /// Attempts to convert a `u8` value to an `ImageType` variant.
    ///
    /// This method tries to map a `u8` value to the corresponding `ImageType` enum variant.
    /// If the value is not valid, it returns an error of type `Error::UnknownImageType`.
    ///
    /// # Parameters
    /// - `value`: The `u8` value representing the image type.
    ///
    /// # Returns
    /// - `ImageType`: A valid `ImageType` variant if the value matches.
    /// - `Error::UnknownImageType`: An error if the value does not match any known image type.
    ///
    /// # Example
    /// ```
    /// use amebazii::types::enums::ImageType;
    /// let image_type = ImageType::try_from(1).unwrap();
    /// assert_eq!(image_type, ImageType::Boot); // Valid conversion.
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ImageType::Parttab),
            1 => Ok(ImageType::Boot),
            2 => Ok(ImageType::FHWSS),
            3 => Ok(ImageType::FHWSNS),
            4 => Ok(ImageType::FWLS),
            // REVISIT: mark unsupported images
            5 => Ok(ImageType::Isp),
            6 => Ok(ImageType::Voe),
            7 => Ok(ImageType::Wln),
            8 => Ok(ImageType::Xip),
            9 => Ok(ImageType::Wowln),
            10 => Ok(ImageType::Cinit),
            11 => Ok(ImageType::Cpfw),
            0x3F => Ok(ImageType::Unknown),
            _ => Err(Error::UnknownImageType(format!(
                "Invalid image type: {}",
                value
            ))),
        }
    }
}

/// Enum representing different section types in memory.
///
/// This enum defines the types of memory sections that can exist, each represented
/// by a specific identifier (u8 value). These sections correspond to various types of
/// memory regions, such as data cache memory (DTCM), instruction cache memory (ITCM),
/// and other specialized memory regions.
///
/// # Variants
/// - `DTCM`: Data tightly coupled memory (0x80).
/// - `ITCM`: Instruction tightly coupled memory (0x81).
/// - `SRAM`: Static RAM (0x82).
/// - `PSRAM`: Pseudo-static RAM (0x83).
/// - `LPDDR`: Low power DDR memory (0x84).
/// - `XIP`: Execute-In-Place memory (0x85), containing raw binary with compiled code.
///
/// The `XIP` variant refers to memory regions that can execute code directly from the memory,
/// without the need to copy the code into RAM.
///
/// # Example
/// ```
/// use amebazii::types::enums::SectionType;
///
/// let section = SectionType::try_from(0x80).unwrap();
/// assert_eq!(section, SectionType::DTCM); // Successfully converts to DTCM.
/// ```
///
/// # Error Handling
/// If the provided value doesn't correspond to a known section type, an error
/// (`Error::UnknownSectionType`) will be returned.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SectionType {
    DTCM = 0x80,
    ITCM,
    SRAM,
    PSRAM,
    LPDDR,
    /// Execute-In-Place (XIP) contains the raw binary with all
    /// compiled code.
    XIP,
}

impl TryFrom<u8> for SectionType {
    type Error = Error;

    /// Tries to convert a `u8` value into a corresponding `SectionType` variant.
    ///
    /// This function maps a `u8` value to its corresponding `SectionType` enum variant.
    /// If the value does not match a valid section type, it returns an error with
    /// the message indicating an invalid section type.
    ///
    /// # Parameters
    /// - `value`: The `u8` value representing the section type.
    ///
    /// # Returns
    /// - `SectionType`: A valid `SectionType` variant if the value matches.
    /// - `Error::UnknownSectionType`: An error if the value doesn't match a known section type.
    ///
    /// # Example
    /// ```
    /// use amebazii::types::enums::SectionType;
    ///
    /// let section = SectionType::try_from(0x84).unwrap();
    /// assert_eq!(section, SectionType::LPDDR); // Successfully converts to LPDDR.
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x80 => Ok(SectionType::DTCM),
            0x81 => Ok(SectionType::ITCM),
            0x82 => Ok(SectionType::SRAM),
            0x83 => Ok(SectionType::PSRAM),
            0x84 => Ok(SectionType::LPDDR),
            0x85 => Ok(SectionType::XIP),
            _ => Err(Error::UnknownSectionType(format!(
                "Invalid section type: {}",
                value
            ))),
        }
    }
}

/// Available sizes for XIP (Execute-In-Place) page remapping.
///
/// This enum defines different page sizes used in XIP remapping, with each variant
/// representing a specific page size in kilobytes.
///
/// # Variants
/// - `_16K`: Represents a 16 KB page size (0).
/// - `_32K`: Represents a 32 KB page size (1).
/// - `_64K`: Represents a 64 KB page size (2).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum XipPageRemapSize {
    _16K = 0,
    _32K,
    _64K,
}

impl TryFrom<u8> for XipPageRemapSize {
    type Error = Error;

    /// Attempts to convert a `u8` value to an `XipPageRemapSize` variant.
    ///
    /// This method maps a `u8` value to the corresponding `XipPageRemapSize` variant.
    /// If the value is not valid, it returns an error with a message indicating the
    /// invalid page remap size.
    ///
    /// # Parameters
    /// - `value`: The `u8` value representing the XIP page remap size.
    ///
    /// # Returns
    /// - `XipPageRemapSize`: The corresponding `XipPageRemapSize` variant if the value matches.
    /// - `Error::InvalidEnumValue`: An error if the value doesn't match a valid remap size.
    ///
    /// # Example
    /// ```
    /// use amebazii::types::enums::XipPageRemapSize;
    ///
    /// let size = XipPageRemapSize::try_from(2).unwrap();
    /// assert_eq!(size, XipPageRemapSize::_64K); // Successfully converts to 64 KB.
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(XipPageRemapSize::_16K),
            1 => Ok(XipPageRemapSize::_32K),
            2 => Ok(XipPageRemapSize::_64K),
            _ => Err(Error::InvalidEnumValue(format!(
                "Invalid XIP page remap size: {}",
                value
            ))),
        }
    }
}

impl XipPageRemapSize {
    /// Returns the size of the page in bytes for the given `XipPageRemapSize` variant.
    ///
    /// This function returns the page size corresponding to the variant in bytes.
    /// The page sizes are predefined as 16 KB, 32 KB, and 64 KB.
    ///
    /// # Returns
    /// - `u32`: The page size in bytes.
    pub fn page_size(&self) -> u32 {
        match self {
            XipPageRemapSize::_16K => 0x4000,
            XipPageRemapSize::_32K => 0x8000,
            XipPageRemapSize::_64K => 0x10000,
        }
    }
}

// Defined in parse_json_config
/// Supported encryption algorithms.
///
/// This enum defines the supported encryption algorithms, each represented by a specific
/// identifier (u16 value). The available algorithms include `Ecb` (Electronic Codebook),
/// `Cbc` (Cipher Block Chaining), and `Other` for any unspecified or custom algorithms.
///
/// # Variants
/// - `Ecb`: Electronic Codebook mode encryption (0).
/// - `Cbc`: Cipher Block Chaining mode encryption (1).
/// - `Other`: Represents other custom or unsupported encryption algorithms (0xFF).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum EncryptionAlgo {
    Ecb,
    Cbc,
    Other = 0xFF,
}

impl TryFrom<u16> for EncryptionAlgo {
    type Error = Error;

    /// Tries to convert a `u16` value to an `EncryptionAlgo` variant.
    ///
    /// # Parameters
    /// - `value`: The `u16` value representing the encryption algorithm.
    ///
    /// # Returns
    /// - `Ok(EncryptionAlgo)`: The corresponding `EncryptionAlgo` variant if the value matches.
    /// - `Err(Error::InvalidEnumValue)`: An error if the value doesn't match a valid encryption algorithm.
    ///
    /// # Example
    /// ```
    /// use amebazii::types::enums::EncryptionAlgo;
    ///
    /// let algo = EncryptionAlgo::try_from(0).unwrap();
    /// assert_eq!(algo, EncryptionAlgo::Ecb); // Successfully converts to Ecb.
    /// ```
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EncryptionAlgo::Ecb),
            1 => Ok(EncryptionAlgo::Cbc),
            0xFF => Ok(EncryptionAlgo::Other),
            _ => Err(Error::InvalidEnumValue(format!(
                "Invalid encryption algorithm: {}",
                value
            ))),
        }
    }
}

// --- Hash Algorithms ---

// Defined in parse_json_config
/// Supported various hash algorithms.
///
/// This enum defines the supported hash algorithms, each represented by a specific
/// identifier (u16 value). The available algorithms include `Md5`, `Sha256`, and `Other`
/// for unspecified or custom algorithms.
///
/// # Variants
/// - `Md5`: MD5 hash algorithm (0x00).
/// - `Sha256`: SHA-256 hash algorithm (0x01).
/// - `Other`: Represents other custom or unsupported hash algorithms (0xFF).
///
/// # Example
/// ```
/// use amebazii::types::enums::HashAlgo;
///
/// let algo = HashAlgo::try_from(1).unwrap();
/// assert_eq!(algo, HashAlgo::Sha256); // Successfully converts to Sha256.
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum HashAlgo {
    Md5 = 0x00,
    Sha256,
    Other = 0xFF,
}

impl TryFrom<u16> for HashAlgo {
    type Error = Error;

    /// Tries to convert a `u16` value to a corresponding `HashAlgo` variant.
    ///
    /// # Parameters
    /// - `value`: The `u16` value representing the hash algorithm.
    ///
    /// # Returns
    /// - `Ok(HashAlgo)`: The corresponding `HashAlgo` variant if the value matches.
    /// - `Err(Error::InvalidEnumValue)`: An error if the value doesn't match a valid hash algorithm.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(HashAlgo::Md5),
            1 => Ok(HashAlgo::Sha256),
            0xFF => Ok(HashAlgo::Other),
            _ => Err(Error::InvalidEnumValue(format!(
                "Invalid hash algorithm: {}",
                value
            ))),
        }
    }
}

impl HashAlgo {
    /// Computes the hash of the provided buffer using the specified algorithm.
    ///
    /// If a key is provided, HMAC (Hash-based Message Authentication Code) is used.
    ///
    /// # Parameters
    /// - `buffer`: A byte slice containing the data to be hashed.
    /// - `key`: An optional byte slice containing the key for HMAC. If `None`, the raw hash is computed.
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)`: The computed hash as a vector of bytes.
    /// - `Err(Error::UnsupportedHashAlgo)`: An error if an unsupported hash algorithm is chosen.
    ///
    /// # Example
    /// ```
    /// use amebazii::types::enums::HashAlgo;
    ///
    /// let data = b"some data to hash";
    /// let algo = HashAlgo::Md5;
    /// let result = algo.compute_hash(data, None).unwrap();
    /// assert_eq!(result.len(), 16); // MD5 produces a 16-byte hash.
    /// ```
    pub fn compute_hash(&self, buffer: &[u8], key: Option<&[u8]>) -> Result<Vec<u8>, Error> {
        match self {
            HashAlgo::Sha256 => match key {
                Some(key_data) => {
                    return Ok(hmac_sha256(&key_data, &buffer)?.to_vec());
                }
                None => {
                    return Ok(sha256(&buffer)?.to_vec());
                }
            },
            HashAlgo::Md5 => match key {
                Some(key_data) => {
                    return Ok(hmac_md5(&key_data, &buffer)?.to_vec());
                }
                None => {
                    return Ok(md5(&buffer)?.to_vec());
                }
            },
            _ => {
                return Err(Error::UnsupportedHashAlgo(*self as u8));
            }
        }
    }
}

/// Enum representing all different types of partitions. (as per _convert_pt_type)
///
/// # Variants
/// - `PartTab`: Partition table (0).
/// - `Boot`: Boot partition (1).
/// - `Fw1`: Firmware partition 1 (2).
/// - `Fw2`: Firmware partition 2 (3).
/// - `Sys`: System partition (4).
/// - `Cal`: Calibration partition (5).
/// - `User`: User data partition (6).
/// - `Var`: Variable partition (7).
/// - `MP`: Main partition (8).
/// - `Rdp`: Reserved partition (9).
///
/// # Example
/// ```
/// use amebazii::types::enums::PartitionType;
///
/// let part = PartitionType::try_from(1).unwrap();
/// assert_eq!(part, PartitionType::Boot); // Successfully converts to Boot partition.
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PartitionType {
    /// Partition table (0).
    PartTab = 0,
    /// Boot partition (1).
    Boot,
    /// Firmware partition 1 (2).
    Fw1,
    /// Firmware partition 2 (3).
    Fw2,
    /// System partition (4).
    Sys,
    /// Calibration partition (5).
    Cal,
    /// User data partition (6).
    User,
    /// Variable partition (7).
    Var,
    /// Main partition (8).
    MP,
    /// Reserved partition (9).
    Rdp,
}

impl TryFrom<u8> for PartitionType {
    type Error = Error;

    /// Attempts to convert a `u8` value to the corresponding `PartitionType` variant.
    ///
    /// This method maps a `u8` value to the appropriate `PartitionType` variant.
    /// If the value is not valid, it returns an error indicating that the partition type
    /// is invalid.
    ///
    /// # Parameters
    /// - `value`: The `u8` value representing the partition type.
    ///
    /// # Returns
    /// - `Ok(PartitionType)`: The corresponding `PartitionType` variant if the value matches.
    /// - `Err(Error::InvalidEnumValue)`: An error if the value doesn't match a valid partition type.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PartitionType::PartTab),
            1 => Ok(PartitionType::Boot),
            2 => Ok(PartitionType::Fw1),
            3 => Ok(PartitionType::Fw2),
            4 => Ok(PartitionType::Sys),
            5 => Ok(PartitionType::Cal),
            6 => Ok(PartitionType::User),
            7 => Ok(PartitionType::Var),
            8 => Ok(PartitionType::MP),
            9 => Ok(PartitionType::Rdp),
            _ => Err(Error::InvalidEnumValue(format!(
                "Invalid partition type: {}",
                value
            ))),
        }
    }
}

/// Enum representing different key export operations.
///
/// This enum defines the operations for key export, represented by a specific `u8` value.
/// The available operations include:
/// - `None`: No key export operation (0).
/// - `Latest`: Export the latest key (1).
/// - `Both`: Export both keys (2).
///
/// # Variants
/// - `None`: No key export operation (0).
/// - `Latest`: Only export the latest key (1).
/// - `Both`: Export both the latest and previous keys (2).
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum KeyExportOp {
    /// No key export operation (0).
    None = 0,
    /// Export the latest key (1).
    Latest,
    /// Export both keys (2).
    Both,
}

impl TryFrom<u8> for KeyExportOp {
    type Error = Error;

    /// Tries to convert a `u8` value to the corresponding `KeyExportOp` variant.
    ///
    /// # Parameters
    /// - `value`: The `u8` value representing the key export operation.
    ///
    /// # Returns
    /// - `Ok(KeyExportOp)`: The corresponding `KeyExportOp` variant if the value matches.
    /// - `Err(Error::InvalidEnumValue)`: An error if the value doesn't match a valid key export operation.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyExportOp::None),
            1 => Ok(KeyExportOp::Latest),
            2 => Ok(KeyExportOp::Both),
            _ => Err(Error::InvalidEnumValue(format!(
                "Invalid key export type: {}",
                value
            ))),
        }
    }
}
