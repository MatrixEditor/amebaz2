// Firmware/OTA Firmware Layout
// ------------------------------------------------------------------------------------
//
// ┌────────────────────────────────────────┐  ┐
// │        OTA Signature (32 bytes)        │  │
// ├────────────────────────────────────────┤  │
// │        Public Key 0 (32 bytes)         │  │
// │                  ...                   │  │
// │        Public Key 5 (32 bytes)         │  │
// ├────────────────────────────────────────┤  │
// │      SubImage 0 Header (96 bytes)      │  │
// ├────────────────────────────────────────┤  │
// │       SubImage 0 FST (96 bytes)        │  │ SubImage 0 Hash
// ├────────────────────────────────────────┤  │
// │ SubImage 0 Section 0 Header (96 bytes) │  │
// ├────────────────────────────────────────┤  │
// │       SubImage 0 Section 0 Data        │  │
// └────────────────────────────────────────┘  │
//                   .                         │
//                   .                         │
//                   .                         │
// ┌────────────────────────────────────────┐  │
// │ SubImage 0 Section N Header (96 bytes) │  │
// ├────────────────────────────────────────┤  │
// │       SubImage 0 Section N Data        │  │
// ├────────────────────────────────────────┤  ┘
// │       SubImage 0 Hash (32 bytes)       │
// ├────────────────────────────────────────┤
// │      SubImage 1 Header (96 bytes)      │
// ├────────────────────────────────────────┤
// │       SubImage 1 FST (96 bytes)        │
// ├────────────────────────────────────────┤
// │ SubImage 1 Section 0 Header (96 bytes) │
// ├────────────────────────────────────────┤
// │       SubImage 1 Section 0 Data        │
// └────────────────────────────────────────┘
//                     .

use std::io::{self, Cursor};

use crate::{
    error::Error,
    types::{
        from_stream, fst::FST, header::ImageHeader, section::Section, BinarySize, FromStream,
        ToStream,
    },
    util::{skip_aligned, write_fill},
    write_aligned,
};

use super::AsImage;

/// Represents a sub-image, including a header, FST (Firmware Security Table), sections, and hash for integrity verification.
///
/// This struct provides methods to manipulate sections, retrieve data, and manage the sub-image’s hash.
#[derive(Debug)]
pub struct SubImage {
    /// The header of the sub-image containing general information about the sub-image.
    pub header: ImageHeader,

    // REVISIT: this struct does not cover the use-case of an encrypted sub-image!!
    /// The Firmware Security Table (FST) associated with the sub-image.
    pub fst: FST,

    /// The collection of sections in the sub-image.
    sections: Vec<Section>,

    /// The hash of the sub-image used for integrity verification.
    hash: [u8; 32],
}

impl Default for SubImage {
    /// Creates a new `SubImage` with default values.
    ///
    /// The default `SubImage` is initialized as follows:
    /// - The `header` is initialized with the default value of `ImageHeader`.
    /// - The `fst` is initialized with the default value of `FST`.
    /// - The `sections` is an empty vector.
    /// - The `hash` is set to an array of 32 `0xFF` bytes (indicating an uninitialized or invalid hash).
    fn default() -> Self {
        SubImage {
            header: ImageHeader::default(),
            fst: FST::default(),
            sections: Vec::new(),
            hash: [0xFF; 32],
        }
    }
}

impl SubImage {
    /// Returns a reference to the hash of the sub-image.
    ///
    /// # Returns:
    /// - A reference to the 32-byte hash of the sub-image.
    ///
    pub fn get_hash(&self) -> &[u8; 32] {
        &self.hash
    }

    /// Returns a reference to the sections in the sub-image.
    ///
    /// This method provides access to the sub-image's sections as an immutable slice.
    ///
    /// # Returns:
    /// - A reference to the `Vec<Section>` representing the sections in the sub-image.
    ///
    pub fn get_sections(&self) -> &[Section] {
        &self.sections
    }

    /// Returns a mutable reference to the sections in the sub-image.
    ///
    /// This method provides access to the sub-image's sections as a mutable slice,
    /// allowing for modification of the sections.
    ///
    /// # Returns:
    /// - A mutable reference to the `Vec<Section>` representing the sections in the sub-image.
    ///
    pub fn get_sections_mut(&mut self) -> &mut [Section] {
        &mut self.sections
    }

    /// Adds a new section to the sub-image.
    ///
    /// This method appends the provided `section` to the list of sections in the sub-image.
    ///
    /// # Arguments:
    /// - `section`: The section to add to the sub-image.
    ///
    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    /// Removes the section at the specified index from the sub-image.
    ///
    /// This method removes the section at the given `index` from the list of sections.
    /// If the index is out of bounds, the method will panic.
    ///
    /// # Arguments:
    /// - `index`: The index of the section to remove.
    ///
    pub fn rem_section_at(&mut self, index: usize) {
        self.sections.remove(index);
    }

    /// Returns a reference to the section at the specified index, if it exists.
    ///
    /// This method retrieves the section at the specified index. If the index is out of bounds,
    /// `None` is returned.
    ///
    /// # Arguments:
    /// - `index`: The index of the section to retrieve.
    ///
    /// # Returns:
    /// - `Option<&Section>`: `Some(section)` if the section exists, or `None` if the index is out of bounds.
    ///
    pub fn get_section(&self, index: usize) -> Option<&Section> {
        self.sections.get(index)
    }

    /// Returns a mutable reference to the section at the specified index, if it exists.
    ///
    /// This method retrieves the section at the specified index. If the index is out of bounds,
    /// `None` is returned.
    ///
    /// # Arguments:
    /// - `index`: The index of the section to retrieve.
    ///
    /// # Returns:
    /// - `Option<&mut Section>`: `Some(section)` if the section exists, or `None` if the index is out of bounds.
    ///
    pub fn get_section_mut(&mut self, index: usize) -> Option<&mut Section> {
        self.sections.get_mut(index)
    }
}

impl AsImage for SubImage {
    /// Set the signature for the SubImage.
    ///
    /// # Arguments:
    /// - `signature`: A slice containing the signature to set.
    fn set_signature(&mut self, signature: &[u8]) {
        self.hash.copy_from_slice(signature);
    }

    /// Set the segment size for the SubImage.
    ///
    /// # Arguments:
    /// - `size`: The size to set for the SubImage's segment.
    fn set_segment_size(&mut self, size: u32) {
        self.header.segment_size = size;
    }

    /// Build the segment size for the SubImage.
    ///
    /// # Returns:
    /// The total segment size, calculated by adding the size of the `ImageHeader`, the `FST`,
    /// and the aligned sizes of all the sections.
    fn build_segment_size(&self) -> u32 {
        // Segment size does not include the hash or image padding
        FST::binary_size() as u32
            + self
                .sections
                .iter()
                .map(Section::build_aligned_size)
                .sum::<u32>()
    }

    /// Build the signature for the SubImage.
    ///
    /// This function generates a signature by calculating the hash of the image's content,
    /// including the header, firmware security table (FST), and sections.
    ///
    /// # Arguments:
    /// - `key`: A byte slice containing the key used to generate the signature.
    ///
    /// # Returns:
    /// A `Result<Vec<u8>, crate::error::Error>` that contains:
    /// - `Ok(Vec<u8>)`: The signature as a byte vector.
    /// - `Err(Error)`: An error if signature calculation fails (e.g., unsupported hash algorithm).
    fn build_signature(&self, key: Option<&[u8]>) -> Result<Vec<u8>, crate::error::Error> {
        let mut buffer = vec![0u8; self.build_segment_size() as usize];
        let mut writer = Cursor::new(&mut buffer);

        // Write the header, FST, and sections to the buffer.
        self.header.write_to(&mut writer)?;
        self.fst.write_to(&mut writer)?;
        for section in &self.sections {
            section.write_to(&mut writer)?;
        }

        // Compute the hash using the FST's hash algorithm.
        match &self.fst.hash_algo {
            Some(algo) => Ok(algo.compute_hash(&buffer, key)?.to_vec()),
            None => Err(Error::NotImplemented(
                "SubImage::build_signature".to_string(),
            )),
        }
    }
}

impl FromStream for SubImage {
    /// Reads a `SubImage` from a binary stream.
    ///
    /// # Arguments:
    /// - `reader`: The stream from which the data will be read. This must implement `std::io::Read` and `std::io::Seek`.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: io::Read + io::Seek,
    {
        self.header.read_from(reader)?;
        self.fst.read_from(reader)?;

        loop {
            let section: Section = from_stream(reader)?;
            let has_next = section.header.has_next();
            self.sections.push(section);
            if !has_next {
                skip_aligned(reader, 0x20)?;
                break;
            }
        }

        reader.read_exact(&mut self.hash)?;
        skip_aligned(reader, if self.header.has_next() { 0x4000 } else { 0x40 })?;
        Ok(())
    }
}

impl ToStream for SubImage {
    /// Writes a `SubImage` to a binary stream.
    ///
    /// # Arguments:
    /// - `writer`: The stream to which the data will be written. This must implement `std::io::Write` and `std::io::Seek`.
    ///
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        self.header.write_to(writer)?;
        self.fst.write_to(writer)?;

        for section in &self.sections {
            section.write_to(writer)?;
        }
        writer.write_all(&self.hash)?;

        let align = if self.header.has_next() { 0x4000 } else { 0x40 };
        write_aligned!(writer, align, 0x87, optional);
        Ok(())
    }
}
