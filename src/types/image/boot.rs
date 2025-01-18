use std::io::{Cursor, Seek, Write};

use crate::{
    types::{
        header::{EntryHeader, ImageHeader, KeyBlock},
        BinarySize, FromStream, ToStream,
    },
    util::{hmac_sha256, skip_aligned, write_fill},
    write_aligned,
};

use super::AsImage;

/// Represents a boot image, including encryption public keys, hash, and segment data.
///
/// This struct contains the details of the boot image including the encryption and hash public keys,
/// header, entry, text (payload), and a hash representing the integrity of the image.
#[derive(Debug)]
pub struct BootImage {
    pub keyblock: KeyBlock,
    /// The header of the boot image, containing general information about the image.
    pub header: ImageHeader,

    /// The entry header, typically pointing to the start of the executable code or data.
    pub entry: EntryHeader,

    /// The textual or executable payload of the boot image.
    /// This can be any binary data contained within the image, typically the code or firmware.
    text: Vec<u8>,

    /// The hash of the boot image.
    /// This is a 32-byte hash used to verify the integrity of the boot image.
    hash: [u8; 32],
}

impl Default for BootImage {
    /// Creates a new `BootImage` with default values.
    ///
    /// The `BootImage` is initialized with default values:
    /// - Encryption and hash public keys are set to all `0xFF` bytes.
    /// - The header and entry are initialized with their default values.
    /// - The `text` field is an empty vector, and the `hash` field is set to all `0xFF` bytes.
    fn default() -> Self {
        BootImage {
            keyblock: KeyBlock::default(),
            header: ImageHeader::default(),
            entry: EntryHeader::default(),
            text: Vec::new(),
            hash: [0xFF; 32],
        }
    }
}

impl BootImage {
    pub fn get_text(&self) -> &[u8] {
        &self.text
    }

    pub fn get_hash(&self) -> &[u8] {
        &self.hash
    }

    pub fn set_text(&mut self, text: Vec<u8>) {
        self.text = text;
    }
}

impl FromStream for BootImage {
    /// Reads a `BootImage` from a binary stream.
    ///
    /// # Arguments:
    /// - `reader`: The stream from which the data will be read. This must implement `std::io::Read` and `std::io::Seek`.
    ///
    /// # Returns:
    /// - `Result<(), crate::error::Error>`: A `Result` indicating success or failure. If an error occurs during reading, it returns an error.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), crate::error::Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        self.keyblock.read_from(reader)?;
        self.header.read_from(reader)?;

        // TODO: add support for encrypted boot images
        self.entry.read_from(reader)?;

        // Resize the `text` field to match the segment size in the header, then read it
        self.text.resize(
            self.header.segment_size as usize - EntryHeader::binary_size(),
            0x00,
        );
        reader.read_exact(&mut self.text)?;

        // Skip any padding (aligned to 0x20 bytes)
        skip_aligned(reader, 0x20)?;

        // Read the final hash for the boot image
        reader.read_exact(&mut self.hash)?;
        Ok(())
    }
}

impl AsImage for BootImage {
    /// Computes the segment size for the BootImage.
    ///
    /// The segment size includes the size of the `header`, `entry`, `text`, and the `hash`.
    ///
    /// # Returns:
    /// - `u32`: The computed segment size.
    fn build_segment_size(&self) -> u32 {
        // Segment size is the sum of the header size, entry size, text size, and hash size.
        // You can adjust this formula if your BootImage structure needs additional fields.
        let new_size = self.text.len() as u32 + EntryHeader::binary_size() as u32;
        new_size + (0x20 - (new_size % 0x20))
    }

    /// Sets the segment size for the BootImage.
    ///
    /// This method sets the `segment_size` field in the `header` of the `BootImage`.
    ///
    /// # Arguments:
    /// - `size`: The segment size to set.
    fn set_segment_size(&mut self, size: u32) {
        self.header.segment_size = size as u32;
        self.entry.length = size - EntryHeader::binary_size() as u32;
    }

    /// Computes the signature for the BootImage.
    ///
    /// This method computes the signature (e.g., HMAC or checksum) for the `BootImage` using the provided key.
    ///
    /// # Arguments:
    /// - `key`: The key used to compute the signature.
    ///
    /// # Returns:
    /// - `Result<Vec<u8>, crate::error::Error>`: The computed signature as a vector of bytes.
    fn build_signature(&self, key: Option<&[u8]>) -> Result<Vec<u8>, crate::error::Error> {
        let mut buffer = vec![
            0x00;
            KeyBlock::binary_size()
                + ImageHeader::binary_size()
                + self.build_segment_size() as usize
        ];
        let mut writer = Cursor::new(&mut buffer);

        // Serialize the components of the BootImage into a buffer
        self.keyblock.write_to(&mut writer)?;
        self.header.write_to(&mut writer)?;
        self.entry.write_to(&mut writer)?;
        writer.write_all(&self.text)?;
        write_aligned!(&mut writer, 0x20, 0x00, optional);

        // The signature is generated using HMAC or any other algorithm.
        Ok(hmac_sha256(key.unwrap(), &buffer)?.to_vec())
    }

    /// Sets the signature for the BootImage.
    ///
    /// This method sets the signature field of the `BootImage` (i.e., the `hash` field).
    ///
    /// # Arguments:
    /// - `signature`: The computed signature to set in the `BootImage`.
    fn set_signature(&mut self, signature: &[u8]) {
        self.hash.copy_from_slice(signature);
    }
}

impl ToStream for BootImage {
    /// Writes a `BootImage` to a binary stream.
    ///
    /// # Arguments:
    /// - `writer`: The stream to which the data will be written. This must implement `std::io::Write`.
    ///
    /// # Returns:
    /// - `Result<(), crate::error::Error>`: A `Result` indicating success or failure. If an error occurs during writing, it returns an error.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), crate::error::Error>
    where
        W: std::io::Write + std::io::Seek,
    {
        self.keyblock.write_to(writer)?;
        self.header.write_to(writer)?;
        self.entry.write_to(writer)?;
        writer.write_all(&self.text)?;

        // Pad the text to a multiple of 0x20 bytes
        let text_len = self.text.len();
        if self.header.segment_size > text_len as u32 {
            writer.write_all(&vec![
                0x00;
                (self.header.segment_size - text_len as u32) as usize
            ])?;
        }

        writer.write_all(&self.hash)?;
        Ok(())
    }
}
