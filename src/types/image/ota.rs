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

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{self, Cursor},
    vec,
};

use crate::{
    error::Error,
    is_valid_data,
    types::{
        enums::HashAlgo,
        from_stream,
        fst::FST,
        header::{ImageHeader, KeyBlock},
        section::Section,
        BinarySize, FromStream, DataRefType, DataType, ToStream,
    },
    util::{skip_aligned, write_fill},
    write_aligned, write_data, write_padding,
};

use super::{AsImage, EncryptedOr};

/// Represents a sub-image, including a header, FST (Firmware Security Table), sections, and hash for integrity verification.
///
/// This struct provides methods to manipulate sections, retrieve data, and manage the sub-image’s hash.
#[derive(Debug)]
pub struct SubImage {
    /// The header of the sub-image containing general information about the sub-image.
    pub header: ImageHeader,

    // REVISIT: this struct does not cover the use-case of an encrypted sub-image!!
    /// The Firmware Security Table (FST) associated with the sub-image.
    pub fst: EncryptedOr<FST>,

    /// The collection of sections in the sub-image.
    sections: EncryptedOr<Vec<Section>>,

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
            fst: EncryptedOr::Plain(FST::default()),
            sections: EncryptedOr::Plain(Vec::new()),
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
        match &self.sections {
            EncryptedOr::Plain(sections) => sections,
            EncryptedOr::Encrypted(_) => panic!("SubImage is encrypted"),
        }
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
        match &mut self.sections {
            EncryptedOr::Plain(sections) => sections,
            EncryptedOr::Encrypted(_) => panic!("SubImage is encrypted"),
        }
    }

    /// Adds a new section to the sub-image.
    ///
    /// This method appends the provided `section` to the list of sections in the sub-image.
    ///
    /// # Arguments:
    /// - `section`: The section to add to the sub-image.
    ///
    pub fn add_section(&mut self, section: Section) {
        match &mut self.sections {
            EncryptedOr::Plain(sections) => sections.push(section),
            EncryptedOr::Encrypted(_) => panic!("SubImage is encrypted"),
        }
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
        match &mut self.sections {
            EncryptedOr::Plain(sections) => sections.remove(index),
            EncryptedOr::Encrypted(_) => panic!("SubImage is encrypted"),
        };
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
        match &self.sections {
            EncryptedOr::Plain(sections) => sections.get(index),
            EncryptedOr::Encrypted(_) => panic!("SubImage is encrypted"),
        }
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
        match &mut self.sections {
            EncryptedOr::Plain(sections) => sections.get_mut(index),
            EncryptedOr::Encrypted(_) => panic!("SubImage is encrypted"),
        }
    }

    /// Reads the signature for this `SubImage` from a binary stream and computes its hash.
    ///
    /// This function reads the header and segment data of the `SubImage` from the given reader,
    /// and then computes the hash of the data using the specified hashing algorithm (`algo`).
    /// Optionally, a key can be provided for use by certain hashing algorithms (e.g., HMAC).
    ///
    /// # Arguments:
    /// - `reader`: A mutable reference to a reader that implements both `io::Read` and `io::Seek`.
    /// - `algo`: The hash algorithm to use for computing the signature (e.g., SHA-256, HMAC).
    /// - `key`: An optional key for algorithms that require one (e.g., HMAC). If no key is needed, this can be `None`.
    ///
    /// # Returns:
    /// - `Result<Vec<u8>, Error>`: Returns the computed signature as a `Vec<u8>` if successful,
    ///   or an `Error` if there is an issue reading the data or computing the hash.
    pub fn signature_from_stream<R>(
        &self,
        reader: &mut R,
        algo: HashAlgo,
        key: Option<&[u8]>,
    ) -> Result<Vec<u8>, Error>
    where
        R: io::Read + io::Seek,
    {
        let mut buffer = vec![0x00; ImageHeader::binary_size() + self.header.segment_size as usize];
        reader.read_exact(&mut buffer)?;
        algo.compute_hash(&buffer, key)
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
            + match &self.sections {
                EncryptedOr::Plain(sections) => sections
                    .iter()
                    .map(Section::build_aligned_size)
                    .sum::<u32>(),
                EncryptedOr::Encrypted(sections_data) => sections_data.len() as u32,
            }
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
        let hash_algo = match &self.fst {
            // TODO: what should we use in case of an encrypted FST?
            EncryptedOr::Encrypted(_) => Some(HashAlgo::Sha256),
            EncryptedOr::Plain(fst) => fst.hash_algo,
        };

        let mut buffer = vec![0u8; ImageHeader::binary_size() + self.build_segment_size() as usize];
        let mut writer = Cursor::new(&mut buffer);

        // Write the header, FST, and sections to the buffer.
        self.header.write_to(&mut writer)?;
        self.fst.write_to(&mut writer)?;
        self.sections.write_to(&mut writer)?;

        // Compute the hash using the FST's hash algorithm.
        match hash_algo {
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
        if self.header.is_encrypt {
            self.fst = EncryptedOr::Encrypted(vec![0; FST::binary_size() as usize]);
            self.fst.read_from(reader)?;

            let mut sections =
                vec![0; self.header.segment_size as usize - FST::binary_size() as usize];
            reader.read_exact(&mut sections)?;
            self.sections = EncryptedOr::Encrypted(sections);
        } else {
            self.fst.read_from(reader)?;

            let mut sections = Vec::new();
            loop {
                let section: Section = from_stream(reader)?;
                let has_next = section.header.has_next();

                sections.push(section);
                if !has_next {
                    break;
                }
            }
            self.sections = EncryptedOr::Plain(sections);
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
        self.sections.write_to(writer)?;
        writer.write_all(&self.hash)?;

        let align = if self.header.has_next() { 0x4000 } else { 0x40 };
        write_aligned!(writer, align, 0x87, optional);
        Ok(())
    }
}

/// Represents an OTA (Over-The-Air) image.
///
/// An `OTAImage` is used in the context of firmware updates, where the image consists of
/// multiple subimages (representing different sections of the firmware), each of which
/// may be signed and encrypted. The `keyblock` holds encrypted keys, `public_keys` are used for
/// verifying signatures, and `checksum` ensures data integrity.
///
/// **Note**: The encryption and signature verification are currently are using the hash key
/// specified in the partition table!
#[derive(Debug)]
pub struct OTAImage {
    /// The key block containing cryptographic keys for encryption and signature verification.
    pub keyblock: KeyBlock,

    /// Public keys (up to 5) used for signature verification.
    public_keys: [DataType<32>; 5],

    /// A collection of subimages contained in the OTA image.
    subimages: Vec<SubImage>,

    /// A checksum value for verifying the integrity of the OTA image.
    pub checksum: Option<u32>,
}

impl Default for OTAImage {
    /// Creates a default `OTAImage` with an empty keyblock, no public keys, no subimages, and a checksum of -1.
    fn default() -> Self {
        OTAImage {
            keyblock: KeyBlock::default(),
            public_keys: [None; 5],
            subimages: Vec::new(),
            checksum: None,
        }
    }
}

impl OTAImage {
    /// Returns a slice of the subimages contained in the OTA image.
    ///
    /// # Returns:
    /// - A reference to a slice containing all the subimages.
    pub fn get_subimages(&self) -> &[SubImage] {
        return &self.subimages;
    }

    /// Returns a mutable slice of the subimages contained in the OTA image.
    ///
    /// # Returns:
    /// - A mutable reference to a slice containing all the subimages.
    pub fn get_subimages_mut(&mut self) -> &mut [SubImage] {
        return &mut self.subimages;
    }

    /// Retrieves a specific subimage by its index.
    ///
    /// # Arguments:
    /// - `index`: The index of the subimage to retrieve.
    ///
    /// # Returns:
    /// - `Some(SubImage)` if the subimage exists at the given index, `None` otherwise.
    pub fn get_subimage(&self, index: usize) -> Option<&SubImage> {
        return self.subimages.get(index);
    }

    /// Retrieves a mutable reference to a specific subimage by its index.
    ///
    /// # Arguments:
    /// - `index`: The index of the subimage to retrieve.
    ///
    /// # Returns:
    /// - `Some(&mut SubImage)` if the subimage exists at the given index, `None` otherwise.
    pub fn get_subimage_mut(&mut self, index: usize) -> Option<&mut SubImage> {
        return self.subimages.get_mut(index);
    }

    /// Adds a new subimage to the OTA image.
    ///
    /// # Arguments:
    /// - `subimage`: The `SubImage` to add to the OTA image.
    pub fn add_subimage(&mut self, subimage: SubImage) {
        self.subimages.push(subimage);
    }

    /// Removes a subimage from the OTA image at the specified index.
    ///
    /// # Arguments:
    /// - `index`: The index of the subimage to remove.
    pub fn rem_subimage_at(&mut self, index: usize) {
        self.subimages.remove(index);
    }

    /// Returns the encryption public key from the keyblock, which is used for OTA signature verification.
    ///
    /// # Returns:
    /// - A reference to the encryption public key (32 bytes) used for signature verification.
    pub fn get_ota_signature(&self) -> &[u8; 32] {
        return self.keyblock.get_enc_pubkey();
    }

    /// Retrieves a specific public key used for signature verification from the OTA image.
    ///
    /// # Arguments:
    /// - `index`: The index (0-4) of the public key to retrieve.
    ///
    /// # Returns:
    /// - A reference to the public key at the specified index, if it exists.
    pub fn get_public_key(&self, index: u8) -> DataRefType<32> {
        return self.public_keys[index as usize].as_ref();
    }
}

// cryptographic ops
impl OTAImage {
    /// Builds the OTA image signature, which is the hash result of the first SubImage's header.
    ///
    /// # Arguments:
    /// - `key`: An optional key to be used in the hash calculation (may be `None` if no key is provided).
    ///
    /// # Returns:
    /// - `Result<Vec<u8>, crate::error::Error>`: The computed signature as a vector of bytes on success,
    ///   or an error if the computation cannot be completed (e.g., `fst.hash_algo` is `None`).
    pub fn build_ota_signature(&self, key: Option<&[u8]>) -> Result<Vec<u8>, crate::error::Error> {
        let mut buffer = Vec::with_capacity(ImageHeader::binary_size());
        // according to spec:
        // OTA signature: The hash result of the 1st Image header “Sub FW Image 0 Header”
        // which is:
        // ├────────────────────────────────────────┤
        // │      SubImage 0 Header (96 bytes)      │
        // ├────────────────────────────────────────┤
        //
        // the key is the hash key from the partition table record for this image
        if let Some(subimage) = self.get_subimage(0) {
            if let EncryptedOr::Plain(fst) = &subimage.fst {
                if let Some(algo) = &fst.hash_algo {
                    let mut writer = Cursor::new(&mut buffer);
                    subimage.header.write_to(&mut writer)?;
                    return algo.compute_hash(&buffer, key);
                } else {
                    return Err(Error::NotImplemented(
                        "OTAImage::build_ota_signature: subimage[0].fst.hash_algo is None"
                            .to_string(),
                    ));
                }
            }
            return Err(Error::NotImplemented(
                "OTAImage::build_ota_signature: subimage[0].fst is encrypted".to_string(),
            ));
        }

        Err(Error::NotImplemented(
            "OTAImage::build_ota_signature: subimage[0] not found".to_string(),
        ))
    }

    /// Reads the OTA signature from a stream and computes its hash using a specified algorithm.
    ///
    /// This function reads the `OTAImage` signature data from the provided reader, computes
    /// its hash using the specified `HashAlgo`, and returns the computed signature.
    ///
    /// The function assumes that the data read corresponds to the "OTA signature" section of
    /// the `OTAImage` format, which is typically the first part of the image.
    ///
    /// # Arguments:
    /// - `reader`: A mutable reference to a reader that implements `io::Read` and `io::Seek`.
    ///   This will be used to read the OTA signature data.
    /// - `algo`: The hash algorithm to use for computing the signature (e.g., SHA-256).
    /// - `key`: An optional key to be used by certain hash algorithms (e.g., for HMAC).
    ///   If the algorithm does not require a key, this can be `None`.
    ///
    /// # Returns:
    /// - `Result<Vec<u8>, crate::error::Error>`: Returns the computed signature as a `Vec<u8>`,
    ///   or an error if there is an issue reading the data or computing the hash.
    pub fn ota_signature_from_stream<R>(
        reader: &mut R,
        algo: HashAlgo,
        key: Option<&[u8]>,
    ) -> Result<Vec<u8>, crate::error::Error>
    where
        R: io::Read + io::Seek,
    {
        let mut buffer = vec![0x00; ImageHeader::binary_size()];
        reader.read_exact(&mut buffer)?;
        algo.compute_hash(&buffer, key)
    }

    /// Sets the OTA image signature, specifically the public encryption key in the keyblock.
    ///
    /// # Arguments:
    /// - `signature`: The signature (encryption public key) to set, which will replace the existing public key.
    pub fn set_ota_signature(&mut self, signature: &[u8]) {
        self.keyblock
            .get_enc_pubkey_mut()
            .copy_from_slice(signature);
    }

    /// Calculates a checksum from a byte buffer by summing all the byte values and applying a bitmask.
    ///
    /// # Arguments:
    /// - `buf`: The byte buffer to compute the checksum from.
    ///
    /// # Returns:
    /// - `i32`: The computed checksum as a 32-bit signed integer.
    pub fn checksum_from_buffer(buf: &[u8]) -> u32 {
        buf.iter().map(|&byte| byte as u32).sum::<u32>()
    }

    /// Calculates a checksum from a stream by reading the content into a buffer and computing its checksum.
    ///
    /// # Arguments:
    /// - `reader`: A reader that implements `io::Read + io::Seek` from which the content will be read.
    ///
    /// # Returns:
    /// - `Result<i32, Error>`: The checksum computed from the stream as a 32-bit signed integer, or an error if the reading fails.
    pub fn checksum_from_stream<R>(reader: &mut R) -> Result<u32, Error>
    where
        R: io::Read + io::Seek,
    {
        let mut buffer = Vec::new();
        // we assume this reader is at pos 0
        reader.read_to_end(&mut buffer)?;
        Ok(OTAImage::checksum_from_buffer(&buffer[..&buffer.len() - 4]))
    }
}

impl FromStream for OTAImage {
    /// Reads an `OTAImage` from a binary stream.
    ///
    /// This function assumes that the provided reader is positioned correctly and
    /// that the stream contains the expected data format for the `OTAImage` struct.
    ///
    /// # Arguments:
    /// - `reader`: A mutable reference to a reader that implements both `io::Read` and `io::Seek`.
    ///
    /// # Returns:
    /// - `Result<(), Error>`: Returns `Ok(())` if the data is read and parsed successfully, or an `Error`
    ///   if something goes wrong (e.g., invalid format, stream read errors).
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: io::Read + io::Seek,
    {
        self.keyblock.read_from(reader)?;

        // Read 5 public keys, validate each, and store the valid ones.
        for i in 0..5 {
            let mut key = [0x00; 32];
            reader.read_exact(&mut key)?;
            if is_valid_data!(&key) {
                self.public_keys[i] = Some(key);
            }
        }

        loop {
            let subimage: SubImage = from_stream(reader)?;
            let has_next = subimage.header.has_next();
            self.subimages.push(subimage);

            // If there is no next subimage, break out of the loop.
            if !has_next {
                break;
            }
        }

        let checksum = reader.read_u32::<LittleEndian>()?;
        if checksum != 0xFFFF_FFFF {
            self.checksum = Some(checksum);
        }
        Ok(())
    }
}

impl ToStream for OTAImage {
    /// Writes an `OTAImage` to a binary stream.
    ///
    /// # Arguments:
    /// - `writer`: A mutable reference to a writer that implements both `io::Write` and `io::Seek`.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        self.keyblock.write_to(writer)?;
        for key in &self.public_keys {
            write_data!(writer, key, 32);
        }
        for subimage in &self.subimages {
            subimage.write_to(writer)?;
        }
        if let Some(checksum) = self.checksum {
            writer.write_u32::<LittleEndian>(checksum)?;
        }
        Ok(())
    }
}
