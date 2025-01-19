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

use crate::{
    error::Error,
    read_padding,
    types::{image::EncryptedOr, PartTab},
    util::write_fill,
    write_aligned, write_padding,
};

use super::{
    enums::PartitionType,
    from_stream,
    image::{
        boot,
        ota::{self},
        pt::{self, Record},
        RawImage,
    },
    sysctrl::SystemData,
    FromStream, ToStream,
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
    System(SystemData),
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
            PartitionType::Sys => Ok(Partition::System(from_stream(reader)?)),
            PartitionType::User => Ok(Partition::User(Self::read_raw_image(
                reader,
                record.length,
            )?)),
            PartitionType::Var => Ok(Partition::Var(Self::read_raw_image(reader, record.length)?)),
            PartitionType::MP => Ok(Partition::Mp(Self::read_raw_image(reader, record.length)?)),
            PartitionType::Rdp => Ok(Partition::Reserved),
            PartitionType::Unknown => Err(Error::InvalidEnumValue(format!(
                "Invalid partition type: {:?}",
                record.part_type
            ))),
        }
    }
}

impl ToStream for Partition {
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        match self {
            Partition::PartitionTable(pt) => pt.write_to(writer)?,
            Partition::Bootloader(bt) => bt.write_to(writer)?,
            Partition::Calibration => (),
            Partition::Fw1(fw1) => fw1.write_to(writer)?,
            Partition::Fw2(fw2) => fw2.write_to(writer)?,
            Partition::Reserved => (),
            Partition::Var(var) => writer.write_all(var)?,
            Partition::System(sys) => sys.write_to(writer)?,
            Partition::User(user) => writer.write_all(user)?,
            Partition::Mp(mp) => writer.write_all(mp)?,
        }

        Ok(())
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

    /// Sets the system partition with the provided system data.
    ///
    /// # Arguments
    ///
    /// * `system_data` - The data to be written to the system partition, represented as `SystemData`.
    pub fn set_system_partition(&mut self, system_data: SystemData) {
        self.set_partition(PartitionType::Sys, Partition::System(system_data));
    }

    /// Sets the boot partition with the given boot image.
    ///
    /// # Arguments
    ///
    /// * `boot_image` - A `BootImage` object containing the bootloader image data.
    pub fn set_boot_partition(&mut self, boot_image: boot::BootImage) {
        self.set_partition(PartitionType::Boot, Partition::Bootloader(boot_image));
    }

    /// Sets the first firmware partition with the provided firmware image.
    ///
    /// This method configures the first firmware partition (`Fw1`) by passing an `OTAImage` object
    /// representing the firmware image to the internal `set_partition` method.
    ///
    /// # Arguments
    ///
    /// * `fw1_image` - An `OTAImage` object containing the first firmware image to be stored.
    pub fn set_fw1(&mut self, fw1_image: ota::OTAImage) {
        self.set_partition(PartitionType::Fw1, Partition::Fw1(fw1_image));
    }

    /// Sets the second firmware partition with the given firmware image.
    ///
    /// This method configures the second firmware partition (`Fw2`) by passing an `OTAImage` object
    /// representing the firmware image. It ensures that the firmware data is correctly placed in the
    /// `Fw2` partition.
    ///
    /// # Arguments
    ///
    /// * `fw2_image` - An `OTAImage` object containing the second firmware image to be stored.
    pub fn set_fw2(&mut self, fw2_image: ota::OTAImage) {
        self.set_partition(PartitionType::Fw2, Partition::Fw2(fw2_image));
    }

    /// Sets the partition table with the provided partition table image.
    ///
    /// # Arguments
    ///
    /// * `pt_image` - A `PartitionTableImage` object representing the partition table to be stored.
    pub fn set_partition_table(&mut self, pt_image: pt::PartitionTableImage) {
        self.set_partition(PartitionType::PartTab, Partition::PartitionTable(pt_image));
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
        if let EncryptedOr::Plain(pt) = &pt_image.pt {
            for record in pt.get_records() {
                reader.seek(io::SeekFrom::Start(record.start_addr as u64))?;
                self.set_partition(record.part_type, Partition::from_record(record, reader)?);
            }
        }
        self.set_partition(PartitionType::PartTab, Partition::PartitionTable(pt_image));
        Ok(())
    }
}

impl ToStream for Flash {
    /// Writes the flash image to the provided writer.
    ///
    /// This function writes the entire flash image, including the calibration pattern, partitions,
    /// and partition records. It populates the `Flash` struct with the data read from the stream.
    ///
    /// # Parameters:
    /// - `writer`: The output stream to which the flash image is written.
    ///
    /// # Returns:
    /// - `Ok(())` if the flash image was successfully written.
    /// - `Err(Error)` if there was an issue writing the flash image.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        writer.write_all(&self.calibration_pattern)?;
        write_padding!(writer, 16);

        let pt_image = self.partitions.get(&PartitionType::PartTab);
        if pt_image.is_none() {
            return Err(Error::InvalidState("Partition table not found".to_string()));
        }

        let pt_image = pt_image.unwrap();
        pt_image.write_to(writer)?;
        write_aligned!(writer, 0x1000);

        // system partition is mandatory
        let system = self.partitions.get(&PartitionType::Sys);
        if system.is_none() {
            return Err(Error::InvalidState(
                "System partition not found".to_string(),
            ));
        }
        system.unwrap().write_to(writer)?;
        // we don't have to align here, because the system partition already fills up
        // the space

        // calibration data: reserved
        // reserved (backup sector for write operations)
        write_padding!(writer, 0x2000);

        // even though the next sections are mandatory, we use the records within the
        // partition table to populate the flash image
        if let Partition::PartitionTable(pt_image) = pt_image {
            let pt = &pt_image.pt;
            if pt.is_encrypted() {
                return Err(Error::NotImplemented(
                    "Encrypted partition table is not supported".to_string(),
                ));
            }

            // the order must be preserved (BUT it is not checked here)
            let pt: &PartTab = pt.as_ref();
            self.write_partition(writer, pt, PartitionType::Boot)?;
            self.write_partition(writer, pt, PartitionType::Fw1)?;
            self.write_partition(writer, pt, PartitionType::Fw2)?;
            self.write_partition(writer, pt, PartitionType::User)?;
        }
        Ok(())
    }
}

impl Flash {
    /// Fills the stream with padding up to the specified offset.
    ///
    /// # Arguments:
    /// - `writer`: A mutable reference to a writer that implements the `std::io::Write` and
    ///   `std::io::Seek` traits.
    /// - `offset`: The offset to fill up to.
    ///
    /// # Returns:
    /// - `Ok(())` if the write operation is successful.
    /// - `Err(Error)` if there is an error during the write operation.
    fn fill_to_offset<W>(&self, writer: &mut W, offset: u64) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        let pos = writer.stream_position()?;
        if pos > offset {
            return Err(Error::InvalidState(format!(
                "Cannot fill to offset {}, current position is {}",
                offset, pos
            )));
        }

        write_padding!(writer, offset - pos);
        Ok(())
    }

    /// Writes a partition to the stream (if present).
    ///
    /// # Arguments:
    /// - `writer`: A mutable reference to a writer that implements the `std::io::Write` and
    ///   `std::io::Seek` traits.
    /// - `pt`: A reference to the partition table.
    /// - `part_type`: The type of the partition to write.
    ///
    /// # Returns:
    /// - `Ok(())` if the write operation is successful.
    /// - `Err(Error)` if there is an error during the write operation.
    fn write_partition<W>(
        &self,
        writer: &mut W,
        pt: &PartTab,
        part_type: PartitionType,
    ) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        if let Some(record) = pt.get_record(part_type) {
            self.fill_to_offset(writer, record.start_addr as u64)?;
            if let Some(partition) = self.partitions.get(&record.part_type) {
                partition.write_to(writer)?;
            } else {
                return Err(Error::InvalidState(format!(
                    "Partition with type '{:?}' not found (is mandatory)",
                    record.part_type
                )));
            }
        }
        Ok(())
    }
}
