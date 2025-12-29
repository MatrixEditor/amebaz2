use itertools::Itertools;
// This module is roughly based on the LinkIt SDK (Public), available here:
// https://github.com/hermeszhang/linkit_sdk_public/
// and source code here:
// https://github.com/dangkhoalk95/demoMT/blob/master/middleware/MTK/nvdm_core
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    types::{from_stream, BinarySize, FromStream, ToStream},
};

/// Enum representing the type of an NVDM data item.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NvdmDataItemType {
    /// Raw binary data with no implicit encoding.
    RawData = 1,

    /// Null-terminated or length-delimited UTF-8 string data.
    String = 2,

    // any other type
    Unknown = 0,
}

impl TryFrom<u8> for NvdmDataItemType {
    type Error = Error;

    /// Attempts to convert a raw `u8` value into an [`NvdmDataItemType`].
    ///
    /// # Parameters
    /// - `value`: The raw byte value read from NVDM storage.
    ///
    /// # Returns
    /// - `Ok(NvdmDataItemType)` if the value corresponds to a known data item type.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NvdmDataItemType::RawData),
            2 => Ok(NvdmDataItemType::String),
            _ => Ok(NvdmDataItemType::Unknown),
        }
    }
}

/// Enum representing the lifecycle status of an NVDM data item.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DataItemStatus {
    /// The data item is marked for deletion.
    Delete = 248,

    /// The data item is valid and fully written.
    Valid = 252,

    /// The data item is currently being written.
    Writing = 254,

    /// The data item slot is empty and unused.
    Empty = 255,

    /// any other status
    Unknown = 0,
}

impl TryFrom<u8> for DataItemStatus {
    type Error = Error;

    /// Attempts to convert a raw `u8` value into a [`DataItemStatus`].
    ///
    /// # Parameters
    /// - `value`: The raw status byte read from flash or NVDM metadata.
    ///
    /// # Returns
    /// - `Ok(DataItemStatus)` if the value matches a known status.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            248 => Ok(DataItemStatus::Delete),
            252 => Ok(DataItemStatus::Valid),
            254 => Ok(DataItemStatus::Writing),
            255 => Ok(DataItemStatus::Empty),
            _ => Ok(DataItemStatus::Unknown),
        }
    }
}

/// Enum representing the status of a Physical Erase Block (PEB).
///
/// This enum maps directly to `peb_status_t` from the C implementation.
/// The values are encoded as raw bytes in flash metadata and therefore
/// occupy the upper range of a `u8`.
///
/// The ordering and spacing of the values reflect different phases of
/// the PEB lifecycle, including erase, activation, data transfer, and
/// reclaim operations. These values must not be renumbered.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PebStatus {
    /// The PEB is currently being erased.
    Erasing = 128,

    /// The PEB is being reclaimed after becoming obsolete.
    Reclaiming = 192,

    /// The PEB is active and contains valid data.
    Actived = 224,

    /// The PEB has finished transferring its data to another block.
    Transfered = 240,

    /// The PEB is in the process of transferring its data.
    Transfering = 248,

    /// The PEB is in the process of becoming active.
    Activing = 252,

    /// The PEB is empty and contains no valid data.
    Empty = 254,

    /// the block status is undefined, it maybe has erased or not erased completely
    Virgin = 255,

    /// any other state
    Unknown = 0,
}

impl TryFrom<u8> for PebStatus {
    type Error = Error;

    /// Attempts to convert a raw `u8` value into a [`PebStatus`].
    ///
    /// # Parameters
    /// - `value`: The raw status byte read from flash metadata.
    ///
    /// # Returns
    /// - `Ok(PebStatus)` if the value matches a known PEB status.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            128 => Ok(PebStatus::Erasing),
            192 => Ok(PebStatus::Reclaiming),
            224 => Ok(PebStatus::Actived),
            240 => Ok(PebStatus::Transfered),
            248 => Ok(PebStatus::Transfering),
            252 => Ok(PebStatus::Activing),
            254 => Ok(PebStatus::Empty),
            255 => Ok(PebStatus::Virgin),
            _ => Ok(PebStatus::Unknown),
        }
    }
}

/// Data item header.
///
/// This struct represents the on-flash header for an NVDM data item.
/// It contains metadata describing the state, identity, size, and
/// storage layout of the data item.
///
/// The layout and field meanings map directly to `struct data_item_header_t`
/// in the original C implementation and are used by the NVDM storage
/// and recovery logic.
#[derive(Debug)]
pub struct DataItemHeader {
    /// Status of the data item.
    ///
    /// This field encodes the lifecycle state of the data item, such as
    /// whether it is valid, being written, deleted, or empty.
    pub status: DataItemStatus,

    /// Physical block number where the data item is stored.
    ///
    /// This value typically identifies the PEB or page index associated
    /// with the data item.
    pub pnum: u8,

    /// Reserved field.
    ///
    /// This field is reserved for future use and should be ignored.
    /// It is preserved to maintain binary compatibility with the
    /// on-flash data layout.
    pub reserved: u16,

    /// Offset to the data item value.
    ///
    /// This field specifies the byte offset from the start of the data
    /// item header to the actual value data.
    pub offset: u16,

    /// Size of the group name, in bytes.
    ///
    /// This value indicates the length of the group name associated
    /// with the data item.
    pub group_name_size: u8,

    /// Size of the data item name, in bytes.
    ///
    /// This value indicates the length of the data item name associated
    /// with the data item.
    pub data_item_name_size: u8,

    /// Size of the data item value, in bytes.
    ///
    /// This field specifies the length of the stored value data.
    pub value_size: u16,

    /// Index of the data item.
    ///
    /// This field is typically used to distinguish between multiple
    /// instances of data items with the same name.
    pub index: u8,

    /// Type of the data item value.
    ///
    /// This field specifies how the value data should be interpreted,
    /// such as raw binary data or string data.
    pub item_type: NvdmDataItemType,

    /// Sequence number of the data item.
    ///
    /// This monotonically increasing value is used to determine the
    /// most recent version of a data item during recovery or scanning.
    pub sequence_number: u32,

    /// Hash of the data item name.
    ///
    /// This field stores a hash value derived from the group name and
    /// data item name, enabling faster lookups during NVDM operations.
    pub hash_name: u32,
}

impl Default for DataItemHeader {
    /// Creates a default `DataItemHeader` instance with safe invalid values.
    ///
    /// Default semantics:
    /// - `status`: `DataItemStatus::Empty` (unused slot)
    /// - `pnum`: `0xFF` (invalid physical block number)
    /// - `reserved`: `0`
    /// - `offset`: `0`
    /// - `group_name_size`: `0`
    /// - `data_item_name_size`: `0`
    /// - `value_size`: `0`
    /// - `index`: `0`
    /// - `item_type`: `NvdmDataItemType::RawData`
    /// - `sequence_number`: `0`
    /// - `hash_name`: `0`
    fn default() -> Self {
        DataItemHeader {
            status: DataItemStatus::Empty,
            pnum: 0xFF,
            reserved: 0,
            offset: 0,
            group_name_size: 0,
            data_item_name_size: 0,
            value_size: 0,
            index: 0,
            item_type: NvdmDataItemType::RawData,
            sequence_number: 0,
            hash_name: 0,
        }
    }
}

impl BinarySize for DataItemHeader {
    /// Returns the binary size of the `DataItemHeader` in bytes.
    ///
    /// Layout size:
    /// - Fixed-size, packed
    /// - Total: 20 bytes
    #[inline]
    fn binary_size() -> usize {
        20
    }
}

impl FromStream for DataItemHeader {
    /// Reads a `DataItemHeader` from a binary stream.
    ///
    /// # Parameters
    /// - `reader`: A mutable reference to a reader implementing `Read`.
    ///
    /// # Returns
    /// - `Ok(())` if the header was successfully parsed.
    /// - `Err(Error)` if an I/O or conversion error occurs.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read,
    {
        self.status = DataItemStatus::try_from(reader.read_u8()?)?;
        self.pnum = reader.read_u8()?;
        self.reserved = reader.read_u16::<LittleEndian>()?;
        self.offset = reader.read_u16::<LittleEndian>()?;
        self.group_name_size = reader.read_u8()?;
        self.data_item_name_size = reader.read_u8()?;
        self.value_size = reader.read_u16::<LittleEndian>()?;
        self.index = reader.read_u8()?;
        self.item_type = NvdmDataItemType::try_from(reader.read_u8()?)?;
        self.sequence_number = reader.read_u32::<LittleEndian>()?;
        self.hash_name = reader.read_u32::<LittleEndian>()?;

        Ok(())
    }
}

impl ToStream for DataItemHeader {
    /// Serializes the `DataItemHeader` to a binary stream.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a writer implementing `Write`.
    ///
    /// # Returns
    /// - `Ok(())` if serialization succeeds.
    /// - `Err(Error)` if an I/O error occurs.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        writer.write_u8(self.status as u8)?;
        writer.write_u8(self.pnum)?;
        writer.write_u16::<LittleEndian>(self.reserved)?;
        writer.write_u16::<LittleEndian>(self.offset)?;
        writer.write_u8(self.group_name_size)?;
        writer.write_u8(self.data_item_name_size)?;
        writer.write_u16::<LittleEndian>(self.value_size)?;
        writer.write_u8(self.index)?;
        writer.write_u8(self.item_type as u8)?;
        writer.write_u32::<LittleEndian>(self.sequence_number)?;
        writer.write_u32::<LittleEndian>(self.hash_name)?;

        Ok(())
    }
}
/// Represents a data item with header information, names, a value payload,
/// and a checksum for validation.
#[derive(Debug)]
pub struct DataItem {
    header: DataItemHeader,
    checksum: u16,
    group_name: String,
    item_name: String,
    value: Vec<u8>,
}

impl Default for DataItem {
    /// Creates a new `DataItem` with default values.
    ///
    /// # Returns
    /// - A `DataItem` instance where all fields are initialized to their
    ///   default states, including an empty group and item names,
    ///   zero checksum, and an empty value vector.
    fn default() -> Self {
        DataItem {
            header: DataItemHeader::default(),
            checksum: 0,
            group_name: String::new(),
            item_name: String::new(),
            value: Vec::new(),
        }
    }
}

impl DataItem {
    /// Calculates the total binary size of the `DataItem`.
    ///
    /// This includes the fixed header size, sizes of the group name,
    /// item name, value, and the checksum (2 bytes).
    ///
    /// # Returns
    /// - The full size of the item as a `u32` value.
    pub fn item_size(&self) -> u32 {
        return DataItemHeader::binary_size() as u32
            + self.header.value_size as u32
            + self.header.data_item_name_size as u32
            + self.header.group_name_size as u32
            + 0x02; // checksum size
    }

    /// Provides a reference to the `DataItemHeader` of this item.
    ///
    /// # Returns
    /// - A reference to the contained `DataItemHeader`.
    pub fn item_header(&self) -> &DataItemHeader {
        &self.header
    }

    /// Returns the name of the item as a string slice.
    ///
    /// # Returns
    /// - A string slice representing the item name.
    pub fn name(&self) -> &str {
        return &self.item_name;
    }

    /// Returns the group name of the item as a string slice.
    ///
    /// # Returns
    /// - A string slice representing the group name.
    pub fn group(&self) -> &str {
        return &self.group_name;
    }

    /// Returns a byte slice containing the item's value data.
    ///
    /// # Returns
    /// - A slice of bytes representing the value of the data item.
    pub fn data(&self) -> &[u8] {
        return &self.value;
    }
}

impl FromStream for DataItem {
    /// Reads a `DataItem` from a binary stream.
    ///
    /// This method reads the header first, then if the status is not
    /// `Empty` or `Unknown`, it reads the group name, item name,
    /// value bytes, and checksum from the stream.
    ///
    /// The group and item names are read as UTF-8 strings and exclude
    /// the trailing null byte.
    ///
    /// # Parameters
    /// - `reader`: A mutable reference to a reader implementing `Read` and `Seek`.
    ///
    /// # Returns
    /// - `Ok(())` if the item was read successfully.
    /// - `Err(Error)` if any I/O error occurs or UTF-8 conversion fails.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        self.header.read_from(reader)?;
        if self.header.status == DataItemStatus::Empty
            || self.header.status == DataItemStatus::Unknown
        {
            return Ok(());
        }

        let mut group_name_raw = vec![0; self.header.group_name_size as usize];
        reader.read_exact(&mut group_name_raw)?;
        self.group_name = String::from_utf8(group_name_raw[..group_name_raw.len() - 1].to_vec())?;

        let mut item_name_raw = vec![0; self.header.data_item_name_size as usize];
        reader.read_exact(&mut item_name_raw)?;
        self.item_name = String::from_utf8(item_name_raw[..item_name_raw.len() - 1].to_vec())?;

        self.value = vec![0; self.header.value_size as usize];
        reader.read_exact(&mut self.value)?;
        self.checksum = reader.read_u16::<LittleEndian>()?;
        Ok(())
    }
}

pub const PEB_MAGIC: &[u8; 4] = b"NVDM";

/// PEB header.
///
/// This struct represents the on-flash header of a Physical Erase Block (PEB).
/// It stores metadata required for flash management, wear leveling, and
/// recovery logic.
///
/// The layout maps directly to `struct peb_header_t` in the original C
/// implementation and must not be altered.
#[derive(Debug)]
pub struct PebHeader {
    /// Magic value identifying a valid PEB header. (NVDM)
    ///
    /// This field is used to validate that the flash block contains
    /// a properly initialized PEB header.
    pub magic: [u8; 4],

    /// Erase count of the PEB.
    ///
    /// This value tracks how many times the block has been erased and
    /// is typically used for wear-leveling decisions.
    pub erase_count: u32,

    /// Current status of the PEB.
    ///
    /// This field encodes the lifecycle state of the PEB, such as
    /// virgin, active, transferring, or reclaiming.
    pub status: PebStatus,

    /// Reserved byte specific to PEB metadata.
    ///
    /// This field is reserved for future use and should be preserved
    /// as-is to maintain binary compatibility.
    pub peb_reserved: u8,

    /// Version of the PEB header format.
    ///
    /// This field allows future extensions of the PEB header layout.
    pub version: u8,

    /// Reserved field.
    ///
    /// This field is reserved and should be written as zero.
    pub reserved: u8,
}

impl Default for PebHeader {
    /// Creates a default `PebHeader` instance with invalid / erased values.
    ///
    /// Default semantics:
    /// - `magic`: `0xFFFF_FFFF` (invalid magic)
    /// - `erase_count`: `0`
    /// - `status`: `PebStatus::Virgin`
    /// - `peb_reserved`: `0xFF`
    /// - `version`: `0`
    /// - `reserved`: `0`
    fn default() -> Self {
        PebHeader {
            magic: *PEB_MAGIC,
            erase_count: 0,
            status: PebStatus::Virgin,
            peb_reserved: 0xFF,
            version: 0,
            reserved: 0,
        }
    }
}

impl BinarySize for PebHeader {
    /// Returns the binary size of the `PebHeader` in bytes.
    ///
    /// Layout:
    /// - Fixed-size
    /// - Total: 12 bytes
    #[inline]
    fn binary_size() -> usize {
        12
    }
}

impl FromStream for PebHeader {
    /// Reads a `PebHeader` from a binary stream.
    ///
    /// # Parameters
    /// - `reader`: A mutable reference to a reader implementing `Read`.
    ///
    /// # Returns
    /// - `Ok(())` if the header was successfully parsed.
    /// - `Err(Error)` if an I/O or conversion error occurs.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read,
    {
        reader.read_exact(&mut self.magic)?;
        if self.magic != *PEB_MAGIC {
            return Err(Error::InvalidState(format!(
                "Invalid peb_header_t magic, expected NVDM, got {:?}",
                self.magic,
            )));
        }

        self.erase_count = reader.read_u32::<LittleEndian>()?;
        self.status = PebStatus::try_from(reader.read_u8()?)?;
        self.peb_reserved = reader.read_u8()?;
        self.version = reader.read_u8()?;
        self.reserved = reader.read_u8()?;

        Ok(())
    }
}

impl ToStream for PebHeader {
    /// Serializes the `PebHeader` to a binary stream.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a writer implementing `Write`.
    ///
    /// # Returns
    /// - `Ok(())` if serialization succeeds.
    /// - `Err(Error)` if an I/O error occurs.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.magic)?;
        writer.write_u32::<LittleEndian>(self.erase_count)?;
        writer.write_u8(self.status as u8)?;
        writer.write_u8(self.peb_reserved)?;
        writer.write_u8(self.version)?;
        writer.write_u8(self.reserved)?;

        Ok(())
    }
}

/// /* This macro defines size of PEB, normally it is size of flash block */
pub const NVDM_PORT_PEB_SIZE: u32 = 4096;

/// Represents the Non-Volatile Data Management (NVDM) system, which manages
/// user data storage in flash memory.
///
/// NVDM supports data retention after power off and organizes data items
/// into groups, enabling classification and orderly management of items.
pub struct NVDM {
    // config
    peb_size: u32,
    items: Vec<DataItem>,
}

impl Default for NVDM {
    /// Creates a new `NVDM` instance with default configuration.
    ///
    /// The `peb_size` is set to the default port-specific PEB size, and
    /// the items vector is initialized empty.
    ///
    /// # Returns
    /// - An `NVDM` instance with default parameters.
    fn default() -> Self {
        return NVDM {
            peb_size: NVDM_PORT_PEB_SIZE,
            items: Vec::new(),
        };
    }
}

impl NVDM {
    /// Constructs a new `NVDM` with the specified physical erase block (PEB) size.
    ///
    /// Other fields are initialized with default values.
    ///
    /// # Parameters
    /// - `peb_size`: The size of a physical erase block in bytes.
    ///
    /// # Returns
    /// - An `NVDM` instance configured with the given PEB size.
    pub fn from_peb_size(peb_size: u32) -> Self {
        NVDM {
            peb_size,
            ..Default::default()
        }
    }

    /// Computes the flash address based on PEB number and offset within the PEB.
    ///
    /// # Parameters
    /// - `pnum`: The physical erase block number.
    /// - `offset`: The offset within the PEB.
    ///
    /// # Returns
    /// - The calculated flash address as a `u32`.
    #[inline]
    pub fn nvdm_port_get_peb_address(&self, pnum: u32, offset: u32) -> u32 {
        pnum * self.peb_size + offset
    }

    /// Retrieves a reference to a data item matching the specified group,
    /// name, and status.
    ///
    /// # Parameters
    /// - `group`: The group name to match.
    /// - `name`: The item name to match.
    /// - `status`: The status of the data item to match.
    ///
    /// # Returns
    /// - `Some(&DataItem)` if an item matching all criteria exists.
    /// - `None` if no matching item is found.
    pub fn get_item(&self, group: &str, name: &str, status: DataItemStatus) -> Option<&DataItem> {
        self.items.iter().find(|&item| {
            item.group() == group && name == item.name() && item.item_header().status == status
        })
    }

    /// Retrieves all data items within a specified group and having a
    /// specified status.
    ///
    /// # Parameters
    /// - `group`: The group name to filter by.
    /// - `status`: The status of the data items to include.
    ///
    /// # Returns
    /// - A vector of references to matching `DataItem`s.
    pub fn get_items_by_group(&self, group: &str, status: DataItemStatus) -> Vec<&DataItem> {
        self.items
            .iter()
            .filter(|&item| item.group() == group && item.item_header().status == status)
            .collect()
    }

    /// Returns a list of unique group names present in the stored data items.
    ///
    /// # Returns
    /// - A vector of string slices representing distinct group names.
    pub fn get_groups(&self) -> Vec<&str> {
        self.items
            .iter()
            .map(|item| item.group())
            .unique()
            .collect()
    }
}

impl FromStream for NVDM {
    /// Reads the NVDM data from a binary stream, parsing all physical
    /// erase blocks (PEBs) and extracting valid data items.
    ///
    /// Iterates through each PEB, checking if it is active, then reads
    /// data items within, adding valid items to the internal collection.
    ///
    /// # Parameters
    /// - `reader`: A mutable reference to a reader implementing `Read` and `Seek`.
    ///
    /// # Returns
    /// - `Ok(())` if all data was successfully read.
    /// - `Err(Error)` if an I/O or parsing error occurs.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        // read all PEBs and insert all (valid) items
        let start = reader.stream_position()?;
        let end = reader.seek(std::io::SeekFrom::End(0))?;
        reader.seek(std::io::SeekFrom::Start(start))?;

        // data size we can use
        let size = end - start;
        for pnum in 0..(size / self.peb_size as u64) {
            let address = start + self.nvdm_port_get_peb_address(pnum as u32, 0) as u64;
            reader.seek(std::io::SeekFrom::Start(address))?;

            let header: PebHeader = from_stream(reader)?;
            if header.status == PebStatus::Actived {
                let mut offset = 0;
                while offset < self.peb_size - 0x20 {
                    let item: DataItem = from_stream(reader)?;
                    match item.item_header().status {
                        DataItemStatus::Delete
                        | DataItemStatus::Valid
                        | DataItemStatus::Writing => {
                            offset += item.item_size();
                            self.items.push(item);
                        }
                        _ => break,
                    }
                }
            }
        }
        Ok(())
    }
}
