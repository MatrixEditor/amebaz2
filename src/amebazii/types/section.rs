use std::vec;

use super::{
    from_stream,
    header::{EntryHeader, SectionHeader},
    BinarySize, FromStream, ToStream,
};
use crate::{error::Error, util::skip_aligned, util::write_fill, write_aligned};

/// Represents a section in a sub-image.
///
/// This struct encapsulates a section within the image, consisting of the following components:
///
/// - `header`: The metadata and configuration for the section (represented by `SectionHeader`).
/// - `entry_header`: The header that defines the entry point and loading address of the section (represented by `EntryHeader`).
/// - `data`: A `Vec<u8>` containing the raw data for the section, which can be processed or manipulated as needed.
///
/// # Default Values:
/// - The `header` and `entry_header` are initialized with their default values.
/// - The `data` is an empty vector by default.
#[derive(Debug)]
pub struct Section {
    /// The metadata and configuration for the section.
    pub header: SectionHeader,

    /// The header that defines the entry point and loading address of the section.
    pub entry_header: EntryHeader,

    /// The raw data of the section.
    data: Vec<u8>,
}

impl Default for Section {
    /// Returns a default `Section` with default headers and an empty data vector.
    fn default() -> Section {
        Section {
            header: SectionHeader::default(),
            entry_header: EntryHeader::default(),
            data: Vec::new(),
        }
    }
}

impl Section {
    // ------------------------------------------------------------------------------------
    // Static Methods
    // ------------------------------------------------------------------------------------

    /// Creates a new `Section` with a specified data capacity.
    ///
    /// This static method allows you to create a new `Section` with a predefined data capacity.
    /// The `data` field will be initialized as a vector of zeroed bytes with the given size.
    ///
    /// # Parameters:
    /// - `capacity`: The size (in bytes) to which the `data` vector should be initialized.
    ///
    /// # Returns:
    /// A new `Section` instance with the specified `capacity` for its `data` field.
    ///
    /// # Example:
    /// ```rust
    /// let section = Section::new_with_size(1024);
    /// ```
    pub fn new_with_size(capacity: usize) -> Section {
        Section {
            header: SectionHeader::default(),
            entry_header: EntryHeader::default(),
            data: vec![0; capacity],
        }
    }

    // ------------------------------------------------------------------------------------
    // Instance Methods
    // ------------------------------------------------------------------------------------

    /// Returns a reference to the section's data.
    ///
    /// # Returns:
    /// A slice of the section's data (`&[u8]`).
    pub fn get_data(&self) -> &[u8] {
        return &self.data;
    }

    /// Computes the aligned length of the section data, ensuring it is padded to a 0x20-byte boundary.
    ///
    /// This function calculates the total length of the section (including the `EntryHeader` and the
    /// section's data) and ensures that the result is aligned to a 0x20-byte boundary.
    ///
    /// # Returns:
    /// - `u32`: The aligned length of the section, including the `EntryHeader` and section data.
    pub fn build_aligned_length(&self) -> u32 {
        let length = EntryHeader::binary_size() + self.data.len();
        let alignment = length % 0x20;
        if alignment == 0 {
            length as u32
        } else {
            (length + (0x20 - alignment)) as u32
        }
    }

    /// Computes the aligned size of the section, including the `SectionHeader`, `EntryHeader`, and section data.
    ///
    /// # Returns:
    /// - `u32`: The aligned size of the section, including headers and data.
    pub fn build_aligned_size(&self) -> u32 {
        SectionHeader::binary_size() as u32 + self.build_aligned_length()
    }

    /// Replaces the current content of the `data` field with a new byte vector.
    ///
    /// # Arguments
    ///
    /// * `data` - The new byte data (`Vec<u8>`) to be assigned to the section.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}

impl FromStream for Section {
    /// Reads a `Section` from a stream.
    ///
    /// # Parameters:
    /// - `reader`: The stream (`Read + Seek`) from which the `Section` will be read.
    ///
    /// # Returns:
    /// A result containing `Ok(())` if the section was successfully read, or an `Error` if something
    /// went wrong during the process.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        self.header = from_stream(reader)?;
        self.entry_header = from_stream(reader)?;
        // length includes entry header size
        self.data.resize(self.header.length as usize - 0x20, 0x00);
        // Read the actual data for the section into the data buffer
        reader.read_exact(&mut self.data)?;
        skip_aligned(reader, 0x20)?;
        Ok(())
    }
}

impl ToStream for Section {
    /// Writes a `Section` to a stream.
    ///
    /// # Parameters:
    /// - `writer`: The stream (`Write + Seek`) to which the `Section` will be written.
    ///
    /// # Returns:
    /// A result containing `Ok(())` if the section was successfully written, or an `Error` if something
    /// went wrong during the process.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write + std::io::Seek,
    {
        self.header.write_to(writer)?;
        self.entry_header.write_to(writer)?;
        writer.write_all(&self.data)?;

        // align the stream
        write_aligned!(writer, 0x20, optional);
        Ok(())
    }
}
