use std::io::{self, Cursor};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    is_valid_data, read_padding, read_valid_data,
    types::{
        enums::{KeyExportOp, PartitionType},
        from_stream,
        header::{ImageHeader, KeyBlock},
        BinarySize, DataRefType, DataType, FromStream, ToStream,
    },
    util::{hmac_sha256, write_fill},
    write_data, write_padding,
};

use super::{AsImage, EncryptedOr};

/// Represents the configuration of a hardware trap.
///
/// A `TrapConfig` structure holds information related to a trap configuration, including
/// the pin, port, level, and validity of the trap. The fields are packed into a 16-bit
/// integer with specific bitwise encoding. The layout of the 16-bit value is as follows:
///
/// ```text
/// Layout (16-bit integer):
/// 0               8             15
/// 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |   pin   | port|l|           |v|
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// Where:
/// - `v` (bit 15) represents the validity of the trap configuration (`1` for valid, `0` for invalid).
/// - `l` (bit 8) represents the level of the trap (`0` or `1`).
/// - `port` (bits 5 to 7) represents the port number (3 bits, value range from 0 to 7).
/// - `pin` (bits 0 to 4) represents the pin number (5 bits, value range from 0 to 31).
///
/// # Type Implementation:
/// - `TrapConfig` can be created from a 16-bit integer using the `From<u16>` trait, and
///   it can be converted back to a 16-bit integer using the `Into<u16>` trait.
///
/// The packing and unpacking logic allows easy conversion between the `TrapConfig` struct
/// and a single 16-bit integer, which is useful for hardware register manipulation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrapConfig {
    /// Whether the trap configuration is valid (1 bit).
    #[serde(default)]
    pub valid: bool,

    /// The level of the trap (1 bit, 0 or 1).
    #[serde(default)]
    pub level: u8,

    /// The port number (3 bits, value range: 0-7).
    #[serde(default)]
    pub port: u8,

    /// The pin number (5 bits, value range: 0-31).
    #[serde(default)]
    pub pin: u8,
}

impl Default for TrapConfig {
    /// Returns a default `TrapConfig` with all fields set to 0 or `false`.
    fn default() -> Self {
        TrapConfig {
            valid: false,
            level: 0,
            port: 0,
            pin: 0,
        }
    }
}

impl From<u16> for TrapConfig {
    /// Converts a 16-bit integer to a `TrapConfig` by unpacking the respective bits
    /// according to the layout described above.
    ///
    /// # Arguments:
    /// - `value`: The 16-bit integer to convert into a `TrapConfig`.
    ///
    /// # Returns:
    /// A `TrapConfig` where each field is populated based on the respective bits
    /// in the 16-bit integer.
    fn from(value: u16) -> Self {
        TrapConfig {
            valid: (value >> 15) & 0x1 != 0,
            level: ((value >> 8) & 0x1) as u8,
            port: ((value >> 5) & 7) as u8,
            pin: (value & 0x1F) as u8,
        }
    }
}

impl Into<u16> for TrapConfig {
    /// Converts a `TrapConfig` back into a 16-bit integer by packing the fields into
    /// their respective bit positions.
    ///
    /// # Returns:
    /// A 16-bit integer representing the packed values of the `TrapConfig` fields.
    fn into(self) -> u16 {
        ((self.valid as u16) << 15)
            | ((self.level as u16) << 8)
            | ((self.port as u16) << 5)
            | self.pin as u16
    }
}

/// Represents a firmware partition record.
///
/// A `Record` struct encapsulates information about a partition within a firmware image.
/// This record stores a hash key for the partition, which will be later used to verify
/// the signatures of the partition contents.
///
/// # Layout (64 bytes):
/// ```text
///          +---------------+---+---+---+---+---+---+---+----------+--------------+----+----+----+----+----+----+
///          | 0             | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8        | 9            | 10 | 11 | 12 | 13 | 14 | 15 |
/// +========+===============+===+===+===+===+===+===+===+==========+==============+====+====+====+====+====+====+
/// | 0x00   |      start_addr: u32      |  length: u32  | type: u8 | dbg_skip: u8 |                             |
/// +--------+---------------------------+---------------+----------+--------------+-----------------------------+
/// | 0x10   | key_valid: u8 |                                                                                   |
/// +--------+---------------+-----------------------------------------------------------------------------------+
/// | 0x20   |                                        hash_key: bytes[32]                                        |
/// +--------+---------------------------------------------------------------------------------------------------+
/// ```
#[derive(Debug)]
pub struct Record {
    /// The starting address of the partition in the firmware image (4 bytes).
    pub start_addr: u32,

    /// The length of the partition in bytes (4 bytes).
    pub length: u32,

    /// The partition type (1 byte). This is an enum (`PartitionType`).
    pub part_type: PartitionType,

    /// A flag that indicates whether debugging should be skipped for this partition (1 byte).
    /// `true` means skip debugging, `false` means do not skip.
    pub dbg_skip: bool,

    /// A 32-byte hash key associated with this partition. By default, it's invalid
    hash_key: DataType<32>,
}

impl BinarySize for Record {
    /// Returns the size of the `Record` structure in bytes (64 bytes).
    #[inline]
    fn binary_size() -> usize {
        0x40
    }
}

impl Default for Record {
    /// Returns a default `Record` with zeroed and invalid fields.
    ///
    /// The default values are as follows:
    /// - `start_addr`: `0`
    /// - `length`: `0`
    /// - `part_type`: `PartitionType::PartTab` (default type)
    /// - `dbg_skip`: `false`
    /// - `hash_key`: `None` (invalid hash key)
    fn default() -> Self {
        Record {
            start_addr: 0,
            length: 0,
            part_type: PartitionType::PartTab, // Default to PartitionTab type
            dbg_skip: false,
            hash_key: None, // Invalid hash key by default
        }
    }
}

impl Record {
    // ------------------------------------------------------------------------------------
    // instance methods
    // ------------------------------------------------------------------------------------

    /// Checks whether the `hash_key` is valid.
    ///
    /// # Returns:
    /// - `true` if the `hash_key` is valid (non-`None` and passes the validation check).
    /// - `false` otherwise (e.g., if the key is `None` or invalid).
    pub fn hash_key_valid(&self) -> bool {
        match &self.hash_key {
            None => false,
            Some(key) => is_valid_data!(key), // checks key validity using the macro
        }
    }

    /// Returns a reference to the `hash_key`.
    pub fn get_hash_key(&self) -> DataRefType<32> {
        self.hash_key.as_ref()
    }

    /// Sets the `hash_key` to a new value.
    ///
    /// # Arguments:
    /// - `key`: A reference to a 32-byte array slice that represents the new `hash_key`.
    ///   If `None` is passed, the `hash_key` will be cleared.
    pub fn set_hash_key(&mut self, key: DataType<32>) {
        self.hash_key = key;
    }
}

impl FromStream for Record {
    /// Parses a `Record` from a binary stream.
    ///
    /// # Arguments:
    /// - `reader`: A mutable reference to a reader that implements `std::io::Read` and `std::io::Seek`.
    ///
    /// # Returns:
    /// - `Ok(())` if the record was successfully parsed.
    /// - `Err(Error)` if there was an issue reading from the stream.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: io::Read + io::Seek,
    {
        self.start_addr = reader.read_u32::<LittleEndian>()?;
        self.length = reader.read_u32::<LittleEndian>()?;

        self.part_type = PartitionType::try_from(reader.read_u8()?)?;
        self.dbg_skip = reader.read_u8()? != 0;

        // Skip 6 bytes of padding.
        read_padding!(reader, 6);

        // Check if the hash_key is valid (using a specific flag).
        if reader.read_u8()? & 0x1 != 0 {
            read_padding!(reader, 15);
            read_valid_data!(self.hash_key, 32, reader);
        } else {
            // Skip 47 bytes if the hash_key is not valid
            read_padding!(reader, 47);
        }
        Ok(())
    }
}

impl ToStream for Record {
    /// Writes a `Record` to a binary stream.
    ///
    /// # Arguments:
    /// - `writer`: A mutable reference to a writer that implements `std::io::Write`.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write,
    {
        writer.write_u32::<LittleEndian>(self.start_addr)?;
        writer.write_u32::<LittleEndian>(self.length)?;

        writer.write_u8(self.part_type as u8)?;
        writer.write_u8(self.dbg_skip as u8)?;

        write_padding!(writer, 6);
        writer.write_u8(self.hash_key_valid() as u8)?;
        write_padding!(writer, 15);
        write_data!(writer, self.hash_key, 32);
        Ok(())
    }
}

/// =====================================================================================
/// Partition Table (PartTab)
/// =====================================================================================
///
/// The `PartTab` struct represents a partition table for the flash, containing various
/// metadata and configuration related to the partitioning, as well as firmware-specific
/// data such as the state of firmware updates, trap configurations, and key export
/// operations.
///
/// # Layout:
/// ```text
///          +-----------------+------------------+---+----------+---------+-------------+-------------+---+---+---+-------+-------+-------+------+----+----------------+
///          | 0               | 1                | 2 | 3        | 4       | 5           | 6           | 7 | 8 | 9 | 10    | 11    | 12    | 13   | 14 | 15             |
/// +========+=================+==================+===+==========+=========+=============+=============+===+===+===+=======+=======+=======+======+====+================+
/// | 0x00   | rma_w_state: u8 | rma_ov_state: u8 |   | eFWV: u8 | num: u8 | fw1_idx: u8 | fw2_idx: u8 |           | ota_trap: u16 | mp_trap: u16 |    | key_exp_op: u8 |
/// +--------+-----------------+------------------+---+----------+---------+-------------+-------------+-----------+---------------+--------------+----+----------------+
/// | 0x10   |                   user_len: u32                   |                                         user_ext: bytes[12]                                          |
/// +--------+---------------------------------------------------+------------------------------------------------------------------------------------------------------+
/// | 0x20   |                                                                  records: Record * num                                                                   |
/// +--------+----------------------------------------------------------------------------------------------------------------------------------------------------------+
/// | 0x30   |                                                                                                                                                          |
/// +--------+----------------------------------------------------------------------------------------------------------------------------------------------------------+
/// | 0x40   |                                                                                                                                                          |
/// +--------+----------------------------------------------------------------------------------------------------------------------------------------------------------+
/// | 0x50   |                                                                                                                                                          |
/// +--------+----------------------------------------------------------------------------------------------------------------------------------------------------------+
/// | 0x60   |                                                                user_bin: bytes[user_len]                                                                 |
/// +--------+----------------------------------------------------------------------------------------------------------------------------------------------------------+
/// ```
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct PartTab {
    pub rma_w_state: u8,
    pub rma_ov_state: u8,
    pub eFWV: u8,
    pub fw1_idx: u8,
    pub fw2_idx: u8,
    pub ota_trap: TrapConfig,
    pub mp_trap: TrapConfig,
    pub key_exp_op: KeyExportOp,
    user_ext: [u8; 12],
    records: Vec<Record>,
    user_bin: Vec<u8>,
}

impl Default for PartTab {
    fn default() -> Self {
        PartTab {
            rma_w_state: 0xFF,
            rma_ov_state: 0xFF,
            eFWV: 0,
            fw1_idx: 0,
            fw2_idx: 0,
            ota_trap: TrapConfig::default(),
            mp_trap: TrapConfig::default(),
            key_exp_op: KeyExportOp::None,
            user_ext: [0xFF; 12],
            records: Vec::new(),
            user_bin: Vec::new(),
        }
    }
}

impl PartTab {
    /// Returns the records in the partition table.
    ///
    /// # Returns:
    /// - A slice of `Record` structs.
    pub fn get_records(&self) -> &[Record] {
        return &self.records;
    }

    /// Returns the user binary data.
    ///
    /// This method provides access to the raw user binary data in the partition table.
    ///
    /// # Returns:
    /// - A slice of bytes representing the user binary data.
    pub fn get_user_bin(&self) -> &[u8] {
        return &self.user_bin;
    }

    /// Returns the user extension data (12 bytes).
    ///
    /// This method provides access to the 12-byte user extension field, which can be used
    /// for storing additional metadata related to the partition table.
    ///
    /// # Returns:
    /// - A reference to the 12-byte array representing the user extension data.
    pub fn get_user_ext(&self) -> &[u8] {
        return &self.user_ext;
    }

    /// Sets the user binary data in the partition table.
    ///
    /// # Arguments:
    /// - `user_bin`: A slice of bytes representing the new user binary data to store.
    pub fn set_user_bin(&mut self, user_bin: &[u8]) {
        self.user_bin.extend_from_slice(user_bin);
    }

    /// Sets the user extension data in the partition table.
    ///
    /// # Arguments:
    /// - `user_ext`: A slice of bytes representing the new user extension data (12 bytes).
    pub fn set_user_ext(&mut self, user_ext: &[u8]) {
        self.user_ext.copy_from_slice(user_ext);
    }

    /// Adds a new partition record to the partition table.
    ///
    /// # Arguments:
    /// - `record`: The `Record` struct that defines the new partition entry to add.
    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    /// Creates and returns a new record for a specific partition type.
    ///
    /// # Arguments:
    /// - `part_type`: The `PartitionType` enum value representing the type of the new record.
    ///
    /// # Returns:
    /// - A mutable reference to the newly created record.
    pub fn new_record(&mut self, part_type: PartitionType) -> &mut Record {
        let mut r = Record::default();
        r.part_type = part_type;
        self.records.push(r);
        return self.records.last_mut().unwrap();
    }

    /// Returns the record for a specific partition type.
    ///
    /// # Arguments:
    /// - `part_type`: The `PartitionType` enum value representing the partition type.
    ///
    /// # Returns:
    /// - `Some(&Record)` if the record for the given partition type is found.
    /// - `None` if the record for the given partition type is not found.
    pub fn get_record(&self, part_type: PartitionType) -> Option<&Record> {
        return self.records.iter().find(|r| r.part_type == part_type);
    }

    /// Returns the record for a specific partition type.
    ///
    /// # Arguments:
    /// - `part_type`: The `PartitionType` enum value representing the partition type.
    ///
    /// # Returns:
    /// - `Some(&mut Record)` if the record for the given partition type is found.
    /// - `None` if the record for the given partition type is not found.
    pub fn get_record_mut(&mut self, part_type: PartitionType) -> Option<&mut Record> {
        return self.records.iter_mut().find(|r| r.part_type == part_type);
    }

    /// Removes a partition record from the partition table.
    ///
    /// # Arguments:
    /// - `part_type`: The `PartitionType` enum value representing the partition type to remove.
    pub fn rem_record(&mut self, part_type: PartitionType) {
        self.records.retain(|r| r.part_type != part_type);
    }

    /// Checks if a record exists for a specific partition type.
    ///
    /// # Arguments:
    /// - `part_type`: The `PartitionType` enum value representing the partition type to check.
    ///
    /// # Returns:
    /// - `true` if a record exists for the given partition type.
    /// - `false` if no record exists for the given partition type.
    pub fn has_record(&self, part_type: PartitionType) -> bool {
        self.get_record(part_type).is_some()
    }
}

impl FromStream for PartTab {
    /// Parses a `PartTab` from a binary stream.
    ///
    /// # Returns:
    /// - `Ok(())` if the `PartTab` was successfully parsed.
    /// - `Err(Error)` if there was an issue reading from the stream.
    ///
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: io::Read + io::Seek,
    {
        self.rma_w_state = reader.read_u8()?;
        self.rma_ov_state = reader.read_u8()?;
        self.eFWV = reader.read_u8()?;
        read_padding!(reader, 1);

        let num = reader.read_u8()? as u32;
        self.fw1_idx = reader.read_u8()?;
        self.fw2_idx = reader.read_u8()?;
        read_padding!(reader, 3);

        self.ota_trap = TrapConfig::from(reader.read_u16::<LittleEndian>()?);
        self.mp_trap = TrapConfig::from(reader.read_u16::<LittleEndian>()?);

        // Skip the byte set to 0xFF manually in generate_pt_table()
        read_padding!(reader, 1);
        self.key_exp_op = KeyExportOp::try_from(reader.read_u8()?)?;

        let mut user_len = reader.read_u32::<LittleEndian>()?;
        reader.read_exact(&mut self.user_ext)?;

        // Read the partition records (num + 1, including boot record).
        for _ in 0..=num {
            self.records.push(from_stream(reader)?);
        }

        // See #1 for details. Even though we parse the user data here,
        // we don't verify its length. This causes issues with malformed
        // partition tables.
        if user_len > 0x100 {
            // REVISIT: this length seems to be correct as a fallback mechanism
            // as it is the maximum length supported by the image tool.
            user_len = 0x100; // == 256
        }

        self.user_bin = vec![0xFF; user_len as usize];
        reader.read_exact(&mut self.user_bin)?;
        Ok(())
    }
}

impl ToStream for PartTab {
    /// Writes a `PartTab` to a binary stream.
    ///
    /// # Arguments:
    /// - `writer`: A mutable reference to a writer that implements `std::io::Write`.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        writer.write_u8(self.rma_w_state)?;
        writer.write_u8(self.rma_ov_state)?;
        writer.write_u8(self.eFWV)?;
        writer.write_u8(0x000)?;

        if self.records.is_empty() {
            return Err(Error::InvalidState("Empty partition table".to_string()));
        }
        writer.write_u8((self.records.len() - 1) as u8)?;
        writer.write_u8(self.fw1_idx)?;
        writer.write_u8(self.fw2_idx)?;
        write_padding!(writer, 3);

        writer.write_u16::<LittleEndian>(self.ota_trap.into())?;
        writer.write_u16::<LittleEndian>(self.mp_trap.into())?;

        // Skip the byte set to 0xFF manually in generate_pt_table()
        write_padding!(writer, 1);
        writer.write_u8(self.key_exp_op as u8)?;

        writer.write_u32::<LittleEndian>(self.user_bin.len() as u32)?;
        writer.write_all(&self.user_ext)?;

        // Write the partition records (num + 1, including boot record).
        for record in &self.records {
            record.write_to(writer)?;
        }

        // Write the user binary data.
        writer.write_all(&self.user_bin)?;
        Ok(())
    }
}

/// =====================================================================================
/// Partition Table Image (PartitionTableImage)
/// =====================================================================================
///
/// The `PartitionTableImage` struct represents a complete partition table image, including
/// the key block, header, partition table, and a hash value. It provides methods to read
/// the image from a stream, retrieve the hash, and generate its signature.
#[derive(Debug)]
pub struct PartitionTableImage {
    pub keyblock: KeyBlock,
    pub header: ImageHeader,
    pub pt: EncryptedOr<PartTab>,
    hash: [u8; 32],
}

impl FromStream for PartitionTableImage {
    /// Parses a `PartitionTableImage` from a binary stream.
    ///
    /// # Arguments:
    /// - `reader`: A mutable reference to a reader that implements `std::io::Read` and `std::io::Seek`.
    ///
    /// # Returns:
    /// - `Ok(())` if the partition table image was successfully parsed.
    /// - `Err(Error)` if there was an issue reading from the stream.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), crate::error::Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        // Read the components of the partition table image
        self.keyblock
            .read_from(reader)
            .map_err(|e| Error::MalformedKeyblock(e.to_string()))?;

        self.header
            .read_from(reader)
            .map_err(|e| Error::MalformedImageHeader(e.to_string()))?;

        // Save the current position to determine the expected size later
        let start_pos = reader.stream_position()?;
        if self.header.is_encrypt {
            self.pt = EncryptedOr::Encrypted(vec![0x00; self.header.segment_size as usize]);
            self.pt
                .read_from(reader)
                .map_err(|e| Error::MalfromedPartTab(e.to_string()))?;
        } else {
            self.pt = EncryptedOr::Plain(
                from_stream(reader).map_err(|e| Error::MalfromedPartTab(e.to_string()))?,
            );
        }
        let current_pos = reader.stream_position()?;
        let target_pos = start_pos + self.header.segment_size as u64;

        // If the stream is behind of the expected position, seek back
        if current_pos < target_pos {
            reader.seek(io::SeekFrom::Current((target_pos - current_pos) as i64))?;
        }
        reader.read_exact(&mut self.hash)?;
        Ok(())
    }
}

impl PartitionTableImage {
    /// Returns a reference to the 32-byte hash value of the partition table image.
    ///
    /// # Returns:
    /// - A reference to the 32-byte hash array.
    pub fn get_hash(&self) -> &[u8; 32] {
        return &self.hash;
    }

    /// Creates a signature for the partition table image using HMAC-SHA256.
    ///
    /// This function generates a signature by using the HMAC (Hash-based Message Authentication
    /// Code) algorithm with SHA-256. It takes a key and the partition table image data (excluding
    /// the `hash` field) as inputs, and returns the resulting signature as a vector of bytes.
    ///
    /// # Arguments:
    /// - `reader`: A mutable reference to a reader that implements `std::io::Read` and `std::io::Seek`.
    /// - `key`: The key to be used in the HMAC algorithm, which should be a byte slice.
    ///
    /// # Returns:
    /// - `Ok(Vec<u8>)` containing the cryptographic signature.
    /// - `Err(Error)` if an error occurs during reading or signature generation.
    ///
    /// # Example:
    /// ```rust
    /// let signature = pt_image.create_signature(&mut reader, &key).unwrap();
    /// ```
    pub fn create_signature<R>(&self, reader: &mut R, key: &[u8]) -> Result<Vec<u8>, Error>
    where
        R: io::Read + io::Seek,
    {
        // Buffer for reading the data, size calculated based on header and segment size
        let mut buffer =
            vec![0xFF; 64 + ImageHeader::binary_size() + self.header.segment_size as usize];

        reader.read_exact(&mut buffer)?;
        return Ok(hmac_sha256(key, &buffer)?.to_vec());
    }
}

impl AsImage for PartitionTableImage {
    /// Computes the segment size for the partition table image.
    ///
    /// The segment size includes the sizes of the keyblock, header, partition table records,
    /// and the user binary data.
    ///
    /// # Returns:
    /// - `u32`: The computed segment size.
    fn build_segment_size(&self) -> u32 {
        // segment size is partition table static size + partition table records + user data length
        let new_size = match &self.pt {
            EncryptedOr::Encrypted(data) => data.len() as u32,
            EncryptedOr::Plain(data) => {
                (0x20 + ((data.records.len() + 1) * Record::binary_size()) + data.user_bin.len())
                    as u32
            }
        };

        // align size to 0x20
        return new_size + (0x20 - (new_size % 0x20));
    }

    /// Computes the signature for the partition table image.
    ///
    /// This method generates the HMAC SHA-256 signature for the image using the provided key.
    ///
    /// # Arguments:
    /// - `key`: The key used to compute the HMAC SHA-256 signature.
    ///
    /// # Returns:
    /// - `Result<Vec<u8>, crate::error::Error>`: The computed signature as a vector of bytes.
    fn build_signature(&self, key: Option<&[u8]>) -> Result<Vec<u8>, Error> {
        let mut buffer =
            vec![0xFF; 64 + ImageHeader::binary_size() + self.build_segment_size() as usize];
        let mut writer = Cursor::new(&mut buffer);

        self.keyblock.write_to(&mut writer)?;
        self.header.write_to(&mut writer)?;
        self.pt.write_to(&mut writer)?;
        Ok(hmac_sha256(key.unwrap(), &buffer)?.to_vec())
    }

    /// Sets the signature for the partition table image.
    ///
    /// This method sets the signature in the image, typically after it has been calculated.
    ///
    /// # Arguments:
    /// - `signature`: The signature to set in the image.
    fn set_signature(&mut self, signature: &[u8]) {
        self.hash.copy_from_slice(signature);
    }

    /// Sets the segment size for the partition table image.
    ///
    /// This method allows setting the segment size manually.
    ///
    /// # Arguments:
    /// - `size`: The segment size to set.
    fn set_segment_size(&mut self, size: u32) {
        self.header.segment_size = size;
    }
}

impl Default for PartitionTableImage {
    /// Returns a default `PartitionTableImage` with default values for all fields.
    ///
    /// The `keyblock`, `header`, and `pt` are initialized with their respective defaults,
    /// and the `hash` field is set to an array of `0xFF` bytes (representing an uninitialized hash).
    ///
    /// # Returns:
    /// - A `PartitionTableImage` with all fields set to their default values.
    ///
    /// # Example:
    /// ```rust
    /// let default_pt_image = PartitionTableImage::default();
    /// ```
    fn default() -> Self {
        PartitionTableImage {
            keyblock: KeyBlock::default(),
            header: ImageHeader::default(),
            pt: EncryptedOr::Plain(PartTab::default()),
            hash: [0xFF; 32],
        }
    }
}

impl ToStream for PartitionTableImage {
    /// Writes a `PartitionTableImage` to a binary stream.
    ///
    /// Note that this method does not check for valid segment size or hash values, and the
    /// padding is applied automatically as part of the partition table write process.
    ///
    /// # Arguments:
    /// - `writer`: A mutable reference to a writer that implements the `std::io::Write` and
    ///   `std::io::Seek` traits.
    ///
    /// # Returns:
    /// - `Ok(())` if the write operation is successful.
    /// - `Err(Error)` if there is an error during the write operation.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        // Write the keyblock and header to the stream
        self.keyblock.write_to(writer)?;
        self.header.write_to(writer)?;

        // Create a buffer to hold the partition table (with padding applied)
        let mut pt_buffer = vec![0xFF; self.header.segment_size as usize];
        let mut pt_writer = Cursor::new(&mut pt_buffer);
        self.pt.write_to(&mut pt_writer)?;

        writer.write_all(&pt_buffer)?;
        writer.write_all(&self.hash)?;
        Ok(())
    }
}
