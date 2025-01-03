//    Section 5.2 - Layout
//    ┌──────────────────┐
//    │ Partition Table  │ 0x00000020 - 0x00001000
//    ├──────────────────┤
//    │ System Data      │ 0x00001000 - 0x00002000
//    ├──────────────────┤
//    │ Calibration Data │ 0x00002000 - 0x00003000
//    ├──────────────────┤
//    │ Reserved         │ 0x00003000 - 0x00004000
//    ├──────────────────┤
//    │                  │
//    │ Boot Image       │ 0x00004000 - 0x0000C000
//    │                  │
//    ├──────────────────┤
//    │                  │
//    │                  │
//    │ Firmware 1       │ 0x0000C000
//    │                  │
//    │                  │
//    ├──────────────────┤
//    │                  │
//    │                  │
//    │ Firmware 2       │
//    │                  │
//    │                  │
//    ├──────────────────┤
//    │                  │
//    │ User Data        │
//    │                  │
//    └──────────────────┘

use std::{collections::HashMap, io};

use crate::{error::Error, read_padding};

use super::{
    enums::PartitionType,
    from_stream,
    image::{
        boot, ota,
        pt::{self, Record},
        RawImage,
    },
    FromStream,
};

/// Represents different types of partitions in a flash image.
#[derive(Debug)]
pub enum Partition {
    PartitionTable(pt::PartitionTableImage),
    Bootloader(boot::BootImage),
    Calibration,
    Fw1(ota::OTAImage),
    Fw2(ota::OTAImage),
    Reserved,
    Var(RawImage),
    System(RawImage),
    User(RawImage),
    Mp(RawImage),
}

impl Partition {
    /// Reads the raw image data from the reader based on the provided record size.
    ///
    /// # Parameters:
    /// - `reader`: The input stream to read from.
    /// - `record_size`: The size of the record to read.
    ///
    /// # Returns:
    /// - A `RawImage` containing the raw data read from the stream.
    fn read_raw_image<R>(reader: &mut R, record_size: u32) -> Result<RawImage, Error>
    where
        R: io::Read + io::Seek,
    {
        let mut buffer = Vec::with_capacity(record_size as usize);
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// Creates a `Partition` from a `Record` and a reader stream.
    ///
    /// This function reads the appropriate partition data based on the partition type from the
    /// provided record and reader. The partition type is matched and the corresponding partition
    /// image is created from the reader.
    ///
    /// # Parameters:
    /// - `record`: The partition record containing metadata (e.g., partition type and length).
    /// - `reader`: The input stream to read the partition data from.
    ///
    /// # Returns:
    /// - A `Partition` variant matching the partition type from the `record`.
    pub fn from_record<R>(record: &Record, reader: &mut R) -> Result<Self, Error>
    where
        R: io::Read + io::Seek,
    {
        match &record.part_type {
            PartitionType::PartTab => Ok(Partition::PartitionTable(from_stream(reader)?)),
            PartitionType::Boot => Ok(Partition::Bootloader(from_stream(reader)?)),
            PartitionType::Fw1 => Ok(Partition::Fw1(from_stream(reader)?)),
            PartitionType::Fw2 => Ok(Partition::Fw2(from_stream(reader)?)),
            PartitionType::Cal => Ok(Partition::Calibration),
            PartitionType::Sys => Ok(Partition::System(Self::read_raw_image(
                reader,
                record.length,
            )?)),
            PartitionType::User => Ok(Partition::User(Self::read_raw_image(
                reader,
                record.length,
            )?)),
            PartitionType::Var => Ok(Partition::Var(Self::read_raw_image(reader, record.length)?)),
            PartitionType::MP => Ok(Partition::Mp(Self::read_raw_image(reader, record.length)?)),
            PartitionType::Rdp => Ok(Partition::Reserved),
        }
    }
}

/// Represents a flash image, including calibration data and partitions.
pub struct Flash {
    /// A 16-byte calibration pattern used for calibration data.
    calibration_pattern: [u8; 16],

    /// A `HashMap` storing partitions indexed by their `PartitionType`.
    partitions: HashMap<PartitionType, Partition>,
}

impl Default for Flash {
    fn default() -> Self {
        Self {
            calibration_pattern: [0; 16],
            partitions: HashMap::new(),
        }
    }
}

impl Flash {
    /// Returns a reference to the calibration pattern.
    ///
    /// This function is used to retrieve the 16-byte calibration pattern for the flash image.
    ///
    /// # Returns:
    /// - A reference to the 16-byte array holding the calibration pattern.
    pub fn get_calibration_pattern(&self) -> &[u8; 16] {
        &self.calibration_pattern
    }

    /// Returns a mutable reference to the calibration pattern.
    ///
    /// This function allows modification of the calibration pattern for the flash image.
    ///
    /// # Returns:
    /// - A mutable reference to the 16-byte array holding the calibration pattern.
    pub fn get_calibration_pattern_mut(&mut self) -> &mut [u8; 16] {
        &mut self.calibration_pattern
    }

    /// Retrieves a partition by its type.
    ///
    /// # Parameters:
    /// - `part_type`: The partition type to search for in the flash.
    ///
    /// # Returns:
    /// - `Some(&Partition)` if the partition exists, otherwise `None`.
    pub fn get_partition(&self, part_type: PartitionType) -> Option<&Partition> {
        self.partitions.get(&part_type)
    }

    /// Checks whether a partition of the specified type exists.
    ///
    /// # Parameters:
    /// - `part_type`: The partition type to check.
    ///
    /// # Returns:
    /// - `true` if the partition exists, otherwise `false`.
    pub fn has_partition(&self, part_type: PartitionType) -> bool {
        self.partitions.contains_key(&part_type)
    }

    /// Sets the partition for the specified type.
    ///
    /// # Parameters:
    /// - `part_type`: The type of partition to set.
    /// - `partition`: The partition data to store.
    pub fn set_partition(&mut self, part_type: PartitionType, partition: Partition) {
        self.partitions.insert(part_type, partition);
    }
}

impl FromStream for Flash {
    /// Reads the flash image from the provided reader.
    ///
    /// This function reads the entire flash image, including the calibration pattern, partitions,
    /// and partition records. It populates the `Flash` struct with the data read from the stream.
    ///
    /// # Parameters:
    /// - `reader`: The input stream from which the flash image is read.
    ///
    /// # Returns:
    /// - `Ok(())` if the flash image was successfully read and parsed.
    /// - `Err(Error)` if there was an issue reading the flash image.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: io::Read + io::Seek,
    {
        reader.read_exact(&mut self.calibration_pattern)?;
        read_padding!(reader, 16);

        let pt_image: pt::PartitionTableImage = from_stream(reader)?;
        for record in pt_image.pt.get_records() {
            reader.seek(io::SeekFrom::Start(record.start_addr as u64))?;
            self.set_partition(record.part_type, Partition::from_record(record, reader)?);
        }

        self.set_partition(PartitionType::PartTab, Partition::PartitionTable(pt_image));
        Ok(())
    }
}
