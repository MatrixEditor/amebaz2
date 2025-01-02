use std::io;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{error::Error, is_valid_key};

use super::{enums::ImageType, BinarySize, FromStream, ToStream};

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
