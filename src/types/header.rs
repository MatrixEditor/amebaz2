use std::io;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{error::Error, is_valid_key};

use super::{
    enums::{ImageType, SectionType, XipPageRemapSize},
    BinarySize, FromStream, ToStream,
};

/// A struct representing the key block with two public keys:
/// * an encryption public key and
/// * a hash public key.
#[derive(Debug)]
pub struct KeyBlock {
    enc_pubkey: [u8; 32],  // Encryption public key (32 bytes)
    hash_pubkey: [u8; 32], // Hash public key (32 bytes)
}

impl Default for KeyBlock {
    /// Creates a default `KeyBlock` with both public keys initialized to `[0xFF; 32]`.
    ///
    /// # Returns
    /// - A `KeyBlock` instance with both the encryption and hash public keys set to `0xFF`.
    fn default() -> Self {
        KeyBlock {
            enc_pubkey: [0xFF; 32],
            hash_pubkey: [0xFF; 32],
        }
    }
}

impl BinarySize for KeyBlock {
    /// Returns the binary size of the `KeyBlock` in bytes.
    ///
    /// Since the struct contains two 32-byte fields (`enc_pubkey` and `hash_pubkey`), the total size is 64 bytes.
    ///
    /// # Returns
    /// - `64`: The size in bytes required to serialize a `KeyBlock`.
    fn binary_size() -> usize {
        64
    }
}

impl FromStream for KeyBlock {
    /// Deserializes a `KeyBlock` from a stream.
    ///
    /// This method reads 64 bytes (32 bytes for each of the `enc_pubkey` and `hash_pubkey`) from the provided
    /// reader and populates the `KeyBlock` instance fields accordingly.
    ///
    /// # Parameters
    /// - `reader`: A mutable reference to a reader implementing `io::Read` and `io::Seek`.
    ///
    /// # Returns
    /// - `Ok(())`: If the `KeyBlock` was successfully deserialized.
    /// - `Err(Error)`: If an error occurs while reading from the stream.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        reader.read_exact(&mut self.enc_pubkey)?;
        reader.read_exact(&mut self.hash_pubkey)?;
        Ok(())
    }
}

impl ToStream for KeyBlock {
    /// Serializes a `KeyBlock` to a stream.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a writer implementing `io::Write` and `io::Seek`.
    ///
    /// # Returns
    /// - `Ok(())`: If the `KeyBlock` was successfully written to the stream.
    /// - `Err(Error)`: If an error occurs while writing to the stream.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write + std::io::Seek,
    {
        writer.write_all(&self.enc_pubkey)?;
        writer.write_all(&self.hash_pubkey)?;
        Ok(())
    }
}

impl KeyBlock {
    /// Checks if the encryption public key is valid.
    ///
    /// # Returns
    /// - `true` if the encryption public key is valid.
    /// - `false` if the encryption public key is invalid.
    pub fn is_enc_pubkey_valid(&self) -> bool {
        is_valid_key!(&self.enc_pubkey)
    }

    /// Checks if the hash public key is valid.
    ///
    /// # Returns
    /// - `true` if the hash public key is valid.
    /// - `false` if the hash public key is invalid.
    pub fn is_hash_pubkey_valid(&self) -> bool {
        is_valid_key!(&self.hash_pubkey)
    }

    /// Retrieves the encryption public key.
    ///
    /// # Returns
    /// - A reference to the encryption public key (32 bytes).
    pub fn get_enc_pubkey(&self) -> &[u8; 32] {
        &self.enc_pubkey
    }

    /// Retrieves the hash public key.
    ///
    /// # Returns
    /// - A reference to the hash public key (32 bytes).
    pub fn get_hash_pubkey(&self) -> &[u8; 32] {
        &self.hash_pubkey
    }
}

// --- Generic Header ---
// According to _create_img_header
//          +---+---+---+---+----+----+----+---+----------+----------------+--------------+---------------+----+----+----+----+
//          | 0 | 1 | 2 | 3 | 4  | 5  | 6  | 7 | 8        | 9              | 10           | 11            | 12 | 13 | 14 | 15 |
// +========+===+===+===+===+====+====+====+===+==========+================+==============+===============+====+====+====+====+
// | 0x00   |  length: u32  | next_offset: u32 | type: u8 | is_encrypt: u8 | pkey_idx: u8 | key_valid: u8 |                   |
// +--------+---------------+------------------+----------+----------------+--------------+---------------+-------------------+
// | 0x10   |               |   serial: u32    |                                                                              |
// +--------+---------------+------------------+------------------------------------------------------------------------------+
// | 0x20   |                                                 key1: bytes[32]                                                 |
// +--------+-----------------------------------------------------------------------------------------------------------------+
// | 0x40   |                                                 key2: bytes[32]                                                 |
// +--------+-----------------------------------------------------------------------------------------------------------------+
// size: 0x60 = 96 bytes

/// Generic image header.
///
/// This struct contains metadata for an image, such as the segment size, offset to the next image header,
/// the image type, encryption flag, and user keys.
#[derive(Debug)]
pub struct ImageHeader {
    /// The size of the image segment in bytes.
    ///
    /// This field specifies the size of the image's data segment, which can be used to determine how much
    /// data is associated with the current image header.
    pub segment_size: u32,

    /// Offset to the next image header.
    ///
    /// If there is no next image header, this field is set to `0xFFFF_FFFF`. Otherwise, it holds the
    /// byte offset to the next image header.
    pub next_offset: u32,

    /// The type of the image.
    ///
    /// This field stores the image type, which can represent different image types such as boot images,
    /// partition tables, etc.
    pub img_type: ImageType,

    /// Flag indicating whether the image is encrypted.
    ///
    /// This boolean flag indicates whether the image is encrypted (`true`) or not (`false`).
    pub is_encrypt: bool,

    /// The serial number associated with the image. (version number)
    ///
    /// This field stores the image's serial number. It is initialized to `0xFFFF_FFFF` by default.
    pub serial: u32,

    /// User key 1, used for encryption
    pub user_key1: [u8; 32],

    /// User key 2, used for encryption
    pub user_key2: [u8; 32],
}

impl Default for ImageHeader {
    /// Creates a default `ImageHeader` instance with the following default values:
    /// - `segment_size`: `0` (default size is zero)
    /// - `next_offset`: `0xFFFF_FFFF` (indicating no next header)
    /// - `img_type`: `ImageType::Parttab` (default is partition table)
    /// - `is_encrypt`: `false` (default is no encryption)
    /// - `serial`: `0xFFFF_FFFF` (default invalid serial number)
    /// - `user_key1`: `[0xFF; 32]` (default invalid key)
    /// - `user_key2`: `[0xFF; 32]` (default invalid key)
    ///
    /// # Returns
    /// - A new `ImageHeader` with the default values.
    fn default() -> Self {
        ImageHeader {
            segment_size: 0,
            next_offset: 0xFFFF_FFFF,     // invalid by default
            img_type: ImageType::Parttab, // partition table by default
            is_encrypt: false,
            serial: 0xFFFF_FFFF,
            user_key1: [0xFF; 32], // invalid by default
            user_key2: [0xFF; 32],
        }
    }
}

impl BinarySize for ImageHeader {
    /// Returns the binary size of the `ImageHeader` in bytes.
    fn binary_size() -> usize {
        0x60 // 96 bytes total size
    }
}

impl FromStream for ImageHeader {
    /// Reads an `ImageHeader` from a stream, populating its fields.
    ///
    /// # Parameters
    /// - `reader`: A mutable reference to a reader that implements both `io::Read` and `io::Seek`. This is used
    ///   to read the raw byte data representing the `ImageHeader`.
    ///
    /// # Returns
    /// - `Ok(())`: If the header is successfully read from the stream and all fields are populated.
    /// - `Err(Error)`: If any errors occur while reading the data from the stream.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        self.segment_size = reader.read_u32::<LittleEndian>()?;
        self.next_offset = reader.read_u32::<LittleEndian>()?;
        self.img_type = ImageType::try_from(reader.read_u8()?)?;
        self.is_encrypt = reader.read_u8()? != 0;

        // Skip the next byte (reserved - seems to be pkey_index)
        reader.seek(io::SeekFrom::Current(1))?;

        let flags = reader.read_u8()?;
        let key1_valid = flags & 0b01 == 0x01;
        let key2_valid = flags & 0b10 == 0x02;

        reader.seek(io::SeekFrom::Current(8))?;
        self.serial = reader.read_u32::<LittleEndian>()?;
        reader.seek(io::SeekFrom::Current(8))?;

        if key1_valid {
            reader.read_exact(&mut self.user_key1)?;
        } else {
            reader.seek(io::SeekFrom::Current(32))?;
        }

        if key2_valid {
            reader.read_exact(&mut self.user_key2)?;
        } else {
            reader.seek(io::SeekFrom::Current(32))?;
        }

        Ok(())
    }
}

impl ToStream for ImageHeader {
    /// Serializes the `ImageHeader` struct to a stream.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a type that implements the `std::io::Write` trait. The data will
    ///   be written to this stream.
    ///
    /// # Returns
    /// - `Ok(())`: If the data is written successfully.
    /// - `Err(Error)`: If there is an issue writing the data to the stream.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        writer.write_u32::<LittleEndian>(self.segment_size)?;
        writer.write_u32::<LittleEndian>(self.next_offset)?;
        writer.write_u8(self.img_type as u8)?;
        writer.write_u8(self.is_encrypt as u8)?;
        writer.write_u8(0x00)?;

        // Write the key validity flags (1 byte), combining key1_valid and key2_valid
        let key1_valid = self.is_key1_valid();
        let key2_valid = self.is_key2_valid();
        writer.write_u8((key1_valid as u8) | ((key2_valid as u8) << 1))?;

        writer.write_all(&[0xFF; 8])?;
        writer.write_u32::<LittleEndian>(self.serial)?;
        writer.write_all(&[0xFF; 8])?;

        // If key1 is valid, write user_key1 (32 bytes), otherwise write padding
        writer.write_all(&self.user_key1)?;
        writer.write_all(&self.user_key2)?;
        Ok(())
    }
}

impl ImageHeader {
    // ------------------------------------------------------------------------------------
    // instance methods
    // ------------------------------------------------------------------------------------

    /// Checks if the first user key (`user_key1`) is valid.
    ///
    /// # Returns
    /// - `true` if `user_key1` is valid (i.e., not all bytes are `0xFF`).
    /// - `false` if `user_key1` is invalid (i.e., all bytes are `0xFF`).
    pub fn is_key1_valid(&self) -> bool {
        is_valid_key!(self.user_key1)
    }

    /// Checks if the second user key (`user_key2`) is valid.
    ///
    /// # Returns
    /// - `true` if `user_key2` is valid (i.e., not all bytes are `0xFF`).
    /// - `false` if `user_key2` is invalid (i.e., all bytes are `0xFF`).
    pub fn is_key2_valid(&self) -> bool {
        is_valid_key!(self.user_key2)
    }

    /// Checks if there is a next image header.
    ///
    /// The `next_offset` field indicates the offset to the next image header. If the value
    /// is `0xFFFF_FFFF`, it means there is no next image. This method returns `true` if the
    /// `next_offset` is not `0xFFFF_FFFF`, indicating there is a next image header.
    ///
    /// # Returns
    /// - `true` if there is a next image (i.e., `next_offset` is not `0xFFFF_FFFF`).
    /// - `false` if there is no next image (i.e., `next_offset` is `0xFFFF_FFFF`).
    pub fn has_next(&self) -> bool {
        self.next_offset != 0xFFFF_FFFF
    }

    /// Gets the first user key (`user_key1`).
    ///
    /// # Returns
    /// A reference to the `user_key1` array (32 bytes).
    pub fn get_user_key1(&self) -> &[u8; 32] {
        &self.user_key1
    }

    /// Gets the second user key (`user_key2`).
    ///
    /// # Returns
    /// A reference to the `user_key2` array (32 bytes).
    pub fn get_user_key2(&self) -> &[u8; 32] {
        &self.user_key2
    }
}

// --- Sub-Image Header ---
// Layout
//          +---+---+---+---+----+----+----+---+----------------------+-----------------+-----------------+------------------+----+----+----+----+
//          | 0 | 1 | 2 | 3 | 4  | 5  | 6  | 7 | 8                    | 9               | 10              | 11               | 12 | 13 | 14 | 15 |
// +========+===+===+===+===+====+====+====+===+======================+=================+=================+==================+====+====+====+====+
// | 0x00   |  length: u32  | next_offset: u32 |     sec_type: u8     | sce_enabled: u8 | xip_pg_size: u8 | xip_sec_size: u8 |                   |
// +--------+---------------+------------------+----------------------+-----------------+-----------------+------------------+-------------------+
// | 0x10   |        validpat: bytes[8]        | xip_key_iv_valid: u8 |                                                                          |
// +--------+----------------------------------+----------------------+--------------------------------------------------------------------------+
// | 0x20   |                                                         xip_key: bytes[16]                                                         |
// +--------+------------------------------------------------------------------------------------------------------------------------------------+
// | 0x30   |                                                         xip_iv: bytes[16]                                                          |
// +--------+------------------------------------------------------------------------------------------------------------------------------------+
// | 0x40   |                                                        alignment: bytes[32]                                                        |
// +--------+------------------------------------------------------------------------------------------------------------------------------------+
// size: 0x60 = 96 bytes

/// Represents the header of a section in a binary image.
///
/// The `SectionHeader` struct contains information about the section's length, type,
/// offset to the next section, as well as details about encryption, remapping, and
/// validation. The struct is designed to handle sections in a memory or file layout
/// where the sections could represent different types of data, such as executable code
/// (XIP), memory regions (DTCM, ITCM), and more. Fields like `xip_page_size`, `xip_key`,
/// and `xip_iv` are especially relevant when dealing with Execute-In-Place (XIP) sections.
#[derive(Debug)]
pub struct SectionHeader {
    /// The length of the section in bytes.
    ///
    /// This field stores the total size of the section, which may include data and metadata.
    /// The exact meaning and content depend on the section type (`sect_type`).
    pub length: u32,

    /// Offset to the next section.
    ///
    /// This field indicates the position of the next section in memory. A value of `0xFFFF_FFFF`
    /// indicates that this is the last section, and no further sections exist.
    pub next_offset: u32,

    /// The type of the current section.
    pub sect_type: SectionType,

    /// Indicates whether Secure Copy Engine (SCE) is enabled for this section.
    pub sce_enabled: bool,

    /// XIP (Execute-In-Place) page size and remapping setting.
    ///
    /// This field indicates the page size used for remapping during XIP operations. The value is an
    /// integer representing one of three possible values: 0 (16K), 1 (32K), and 2 (64K).
    pub xip_page_size: XipPageRemapSize,

    /// Block size for XIP remapping.
    ///
    /// This field defines the block size used for XIP remapping, typically represented in bytes.
    /// The default value is `0`, which means that remapping block size is not defined.
    pub xip_block_size: u8,

    /// A valid pattern used to verify the integrity of the section header.
    valid_pattern: [u8; 8],

    /// The encryption key used for XIP operations.
    ///
    /// This is a 16-byte key used during the XIP process. If encryption is enabled for the section,
    /// this key is used for decryption. The default value is an array of `0xFF` bytes, indicating an
    /// invalid key by default.
    xip_key: [u8; 16],

    /// The initialization vector (IV) used for XIP encryption.
    ///
    /// This is a 16-byte initialization vector (IV) used in conjunction with the `xip_key` during XIP
    /// encryption operations. As with the key, it is initialized to an invalid value of `0xFF` bytes.
    xip_iv: [u8; 16],
}

impl BinarySize for SectionHeader {
    /// Returns the binary size (in bytes) of the `SectionHeader` struct.
    ///
    /// # Returns
    /// Returns the binary size as a constant `usize` value, which is `0x60` (96 bytes).
    fn binary_size() -> usize {
        return 0x60;
    }
}

impl Default for SectionHeader {
    /// Returns the default `SectionHeader` instance with predefined values.
    ///
    /// - `length` set to `0`, representing an empty section.
    /// - `next_offset` set to `0xFFFF_FFFF`, which indicates the absence of a next section.
    /// - `sect_type` set to `SectionType::XIP`, as a default section type.
    /// - `sce_enabled` set to `false` (assuming encryption is not enabled by default).
    /// - `xip_page_size` set to `XipPageRemapSize::_16K`, the smallest page size for XIP remapping.
    /// - `xip_block_size` set to `0`, indicating no block size set.
    /// - `valid_pattern` set to a default validation pattern of increasing values.
    /// - `xip_key` and `xip_iv` both set to `0xFF`, representing uninitialized keys/IV.
    ///
    /// # Returns
    /// Returns a `SectionHeader` with the default values.
    fn default() -> SectionHeader {
        return SectionHeader {
            length: 0,
            next_offset: 0xFFFF_FFFF, // make last as default
            sect_type: SectionType::XIP,
            sce_enabled: false, // encrypted currently not supported
            xip_page_size: XipPageRemapSize::_16K,
            xip_block_size: 0,
            valid_pattern: [0, 1, 2, 3, 4, 5, 6, 7],
            xip_key: [0xFF; 16],
            xip_iv: [0xFF; 16],
        };
    }
}

impl FromStream for SectionHeader {
    /// Reads the `SectionHeader` struct from a stream.
    ///
    /// # Error Handling
    /// This method may return an error if there is an issue with reading from the stream or if
    /// an invalid value is encountered during deserialization (e.g., an invalid enum value for `sect_type`
    /// or `xip_page_size`). Any errors encountered during reading from the stream will be propagated as an `Error`.
    ///
    /// # Returns
    /// - `Ok(())` if the `SectionHeader` was successfully read from the stream.
    /// - `Err(Error)` if there was an issue reading from the stream or an invalid value was encountered.
    ///
    /// # Panics
    /// This function assumes that the binary data is well-formed and follows the expected format. If
    /// the format is incorrect or the stream is malformed, this function will return an error instead of
    /// panicking. However, it is still important to handle errors appropriately in calling code.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        self.length = reader.read_u32::<LittleEndian>()?;
        self.next_offset = reader.read_u32::<LittleEndian>()?;
        self.sect_type = SectionType::try_from(reader.read_u8()?)?;
        self.sce_enabled = reader.read_u8()? != 0;
        self.xip_page_size = XipPageRemapSize::try_from(reader.read_u8()?)?;
        self.xip_block_size = reader.read_u8()?;

        reader.seek(io::SeekFrom::Current(4))?;
        reader.read_exact(&mut self.valid_pattern)?;

        let sce_key_iv_valid = reader.read_u8()? & 0b01 == 1;

        reader.seek(io::SeekFrom::Current(7))?;

        if sce_key_iv_valid {
            reader.read_exact(&mut self.xip_key)?;
            reader.read_exact(&mut self.xip_iv)?;
            // Align to 96 bytes (by skipping 32 bytes)
            reader.seek(io::SeekFrom::Current(32))?;
        } else {
            // Align to 96 bytes by skipping 64 bytes if no key/IV is present
            reader.seek(io::SeekFrom::Current(64))?;
        }

        Ok(())
    }
}

impl ToStream for SectionHeader {
    /// Serializes the `SectionHeader` struct to a stream.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a type that implements the `std::io::Write` trait. The data will
    ///   be written to this stream.
    ///
    /// # Returns
    /// - `Ok(())`: If the data is written successfully.
    /// - `Err(Error)`: If there is an issue writing the data to the stream.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        writer.write_u32::<LittleEndian>(self.length)?;
        writer.write_u32::<LittleEndian>(self.next_offset)?;
        writer.write_u8(self.sect_type as u8)?;
        writer.write_u8(self.sce_enabled as u8)?;
        writer.write_u8(self.xip_page_size as u8)?;
        writer.write_u8(self.xip_block_size)?;

        writer.write_all(&[0xFF; 4])?;
        writer.write_all(&self.valid_pattern)?;
        writer.write_u8(self.xip_key_iv_valid() as u8)?;
        writer.write_all(&[0xFF; 7])?;

        writer.write_all(&self.xip_key)?;
        writer.write_all(&self.xip_iv)?;
        Ok(())
    }
}

impl SectionHeader {
    // ------------------------------------------------------------------------------------
    // instance methods
    // ------------------------------------------------------------------------------------

    /// Checks if the section has a next section.
    ///
    /// This method checks if the `next_offset` field of the `SectionHeader` is set to the special
    /// value `0xFFFF_FFFF`, which indicates that there is no next section. If `next_offset` is
    /// different from this value, it implies that there is a subsequent section, and the method
    /// returns `true`. Otherwise, it returns `false`, indicating this is the last section.
    ///
    /// # Returns
    /// - `true` if the section has a next section (i.e., `next_offset` is not `0xFFFF_FFFF`).
    /// - `false` if this is the last section (i.e., `next_offset` is `0xFFFF_FFFF`).
    pub fn has_next(&self) -> bool {
        return self.next_offset != 0xFFFF_FFFF;
    }

    /// Checks if both the `xip_key` and `xip_iv` fields are valid.
    ///
    /// This method uses the `is_valid_key!` macro to check the validity of both the `xip_key`
    /// and `xip_iv` fields. It returns `true` if both fields are valid (i.e., they do not consist
    /// entirely of `0xFF` bytes). If either field is invalid, it returns `false`.
    ///
    /// This method is typically used to determine if the section is encrypted and can be used
    /// for cryptographic operations.
    ///
    /// # Returns
    /// - `true` if both `xip_key` and `xip_iv` are valid.
    /// - `false` if either `xip_key` or `xip_iv` is invalid.
    pub fn xip_key_iv_valid(&self) -> bool {
        return is_valid_key!(self.xip_key) && is_valid_key!(self.xip_iv);
    }

    /// Retrieves the XIP key.
    pub fn get_xip_key(&self) -> &[u8; 16] {
        &self.xip_key
    }

    /// Retrieves the XIP IV.
    pub fn get_xip_iv(&self) -> &[u8; 16] {
        &self.xip_iv
    }

    /// Retrieves the valid pattern.
    pub fn get_valid_pattern(&self) -> &[u8; 8] {
        &self.valid_pattern
    }
}

// -- entry header --
// Layout recovered from
//  - entry_header_t *_create_entry_header(uint length, uint load_address, uint entry_address)
//
//          +---+---+---+---+----+----+----+----+----+----+-----+----+----+----+----+----+
//          | 0 | 1 | 2 | 3 | 4  | 5  | 6  | 7  | 8  | 9  | 10  | 11 | 12 | 13 | 14 | 15 |
// +========+===+===+===+===+====+====+====+====+====+====+=====+====+====+====+====+====+
// | 0x00   |  length: u32  | load_address: u32 | entry_address: u32 |                   |
// +--------+---------------+-------------------+--------------------+-------------------+
// | 0x10   |                                                                            |
// +--------+----------------------------------------------------------------------------+
// size = 0x20

/// `EntryHeader` represents the header of a specific entry within a binary image.
/// It includes metadata about the entry, such as its length, load address, and entry point.
///
/// This structure is used to describe a segment or block of data that is loaded into memory
/// at a specified address and contains an entry point that the system can use to jump to
/// the start of execution.
///
/// Fields:
/// - `length`: The length of the entry in bytes. This defines how much memory the entry occupies.
/// - `load_address`: The memory address at which the entry is loaded into the system's memory space.
/// - `entry_address`: The address to which control is transferred when the entry is executed.
///   By default, it's set to `0xFFFF_FFFF` (None), which indicates an invalid entry address.
#[derive(Debug)]
pub struct EntryHeader {
    /// The length of the entry in bytes.
    pub length: u32,

    /// The load address in memory where the entry will be loaded.
    pub load_address: u32,

    /// The entry address, the address to which the system will jump to start execution.
    /// Defaults to `0xFFFF_FFFF` (None), indicating an invalid address.
    pub entry_address: Option<u32>,
}

impl BinarySize for EntryHeader {
    /// Returns the binary size of the `EntryHeader` struct in bytes.
    ///
    /// # Returns
    /// - `0x20`: The size of the `EntryHeader` struct in bytes.
    fn binary_size() -> usize {
        return 0x20;
    }
}

impl Default for EntryHeader {
    /// Returns the default values for the `EntryHeader` struct.
    ///
    /// The default values are:
    /// - `length`: `0` (indicating no data or length of the entry).
    /// - `load_address`: `0` (indicating the entry is not loaded anywhere).
    /// - `entry_address`: `0xFFFF_FFFF` (None) (invalid entry address, typically indicating no valid entry).
    ///
    /// # Returns
    /// - `EntryHeader`: A struct with default values.
    fn default() -> EntryHeader {
        return EntryHeader {
            length: 0,
            load_address: 0,
            entry_address: None,
        };
    }
}

impl FromStream for EntryHeader {
    /// Reads an `EntryHeader` from the provided reader (e.g., a file or memory buffer).
    ///
    /// # Arguments
    /// - `reader`: A mutable reference to an object that implements `Read` and `Seek` (e.g., a file or buffer).
    ///
    /// # Returns
    /// - `Ok(())`: If the deserialization is successful.
    /// - `Err(Error)`: If there is an error reading from the stream, such as an unexpected end of data.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        self.length = reader.read_u32::<LittleEndian>()?;
        self.load_address = reader.read_u32::<LittleEndian>()?;
        self.entry_address = match reader.read_u32::<LittleEndian>()? {
            0xFFFF_FFFF => None,
            address => Some(address),
        };
        reader.seek(io::SeekFrom::Current(20))?;
        Ok(())
    }
}

impl ToStream for EntryHeader {
    /// Serializes the `EntryHeader` struct to a stream.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a type that implements the `std::io::Write` trait. The data will
    ///   be written to this stream.
    ///
    /// # Returns
    /// - `Ok(())`: If the data is written successfully.
    /// - `Err(Error)`: If there is an issue writing the data to the stream.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        writer.write_u32::<LittleEndian>(self.length)?;
        writer.write_u32::<LittleEndian>(self.load_address)?;
        writer.write_u32::<LittleEndian>(self.entry_address.unwrap_or(0xFFFF_FFFF))?;
        writer.write_all(&[0xFF; 20])?;
        Ok(())
    }
}
