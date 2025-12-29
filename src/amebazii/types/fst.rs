use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;

use super::{enums::*, BinarySize, DataRefType, DataType, FromStream, ToStream};
use crate::{
    error::Error, is_valid_data, keys::DEFAULT_VALID_PATTERN, read_padding, util::write_fill,
    write_data, write_padding,
};

/// # Firmware Security Table (FST)
///
/// The `FST` struct represents the firmware security table (FST) of a sub-image within a
/// firmware image. This table holds information about the encryption algorithm, hash
/// algorithm, security keys, and other configuration for firmware partitions.
///
/// ## Layout
/// ```text
///          +-------+-------+--------+-------+-----------+---------------+---+---+---+---+----+----+----+----+----+----+
///          | 0     | 1     | 2      | 3     | 4         | 5             | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +========+=======+=======+========+=======+===========+===============+===+===+===+===+====+====+====+====+====+====+
/// | 0x00   | enc_algo: u16 | hash_algo: u16 |          part_size: u32           |          valipat: bytes[8]          |
/// +--------+--------------------------------+-----------+---------------+---------------------------------------------+
/// | 0x10   |                                | flags: u8 | key_flags: u8 |                                             |
/// +--------+--------------------------------+-----------+---------------+---------------------------------------------+
/// | 0x20   |                                          cipher_key: bytes[32]                                           |
/// +--------+----------------------------------------------------------------------------------------------------------+
/// | 0x40   |                                           cipher_iv: bytes[16]                                           |
/// +--------+----------------------------------------------------------------------------------------------------------+
/// | 0x50   |                                                                                                          |
/// +--------+----------------------------------------------------------------------------------------------------------+
/// ```
/// - Size = 0x60 = 96 bytes
///
/// **Note:** Encryption and cipher-related fields are placeholders, as encryption is not
/// currently supported. These fields are implemented as `Option<T>` to allow future extension
/// if encryption support is added later.
///
/// # Example:
/// ```rust
/// let mut fst = FST::default();
///
/// // set hash algorithm
/// fst.hash_algo = Some(HashAlgo::Sha256);
///
/// // clear encryption algorithm
/// fst.enc_algo = None;
/// ```
#[derive(Debug)]
pub struct FST {
    /// encryption algorithm (not supported)
    pub enc_algo: Option<EncryptionAlgo>,

    /// The hash algorithm used for hashing. Default is `Sha256`.
    pub hash_algo: Option<HashAlgo>,

    pub partition_size: u32,
    valid_pattern: [u8; 8],

    cipher_key: DataType<32>,
    cipher_iv: DataType<16>,
}

impl Default for FST {
    fn default() -> FST {
        return FST {
            enc_algo: None, // currently encryption is not supported
            hash_algo: Some(HashAlgo::Sha256),
            partition_size: 0, // default is zero
            valid_pattern: DEFAULT_VALID_PATTERN.clone(),
            cipher_key: None,
            cipher_iv: None,
        };
    }
}

impl BinarySize for FST {
    /// Returns the binary size of the `FST` structure in bytes.
    ///
    /// # Returns:
    /// The size of the `FST` struct in bytes, which is `0x60` (96 bytes).
    #[inline]
    fn binary_size() -> usize {
        return 0x60;
    }
}

impl FST {
    // ------------------------------------------------------------------------------------
    // instance methods
    // ------------------------------------------------------------------------------------

    /// Checks if the cipher key and IV are valid.
    ///
    /// # Returns:
    /// - `true`: If both the cipher key and IV are valid.
    /// - `false`: If either the cipher key or IV is not set or invalid.
    pub fn is_cipher_key_iv_valid(&self) -> bool {
        match (&self.cipher_key, &self.cipher_iv) {
            (Some(key), Some(iv)) => is_valid_data!(key) && is_valid_data!(iv),
            _ => false,
        }
    }

    /// Returns a reference to the validation pattern used for the FST structure.
    ///
    /// # Returns:
    /// A reference to the 8-byte validation pattern.
    pub fn get_pattern(&self) -> &[u8; 8] {
        return &self.valid_pattern;
    }

    /// Returns a reference to the cipher key if it is set.
    ///
    /// # Returns:
    /// An `Option` containing a reference to the 32-byte cipher key.
    ///
    /// ```rust
    /// let fst = FST::default();
    /// if let Some(cipher_key) = fst.get_cipher_key() {
    ///     // Handle valid cipher key
    /// }
    /// ```
    pub fn get_cipher_key(&self) -> DataRefType<'_, 32> {
        return self.cipher_key.as_ref();
    }

    /// Returns a reference to the cipher IV if it is set.
    ///
    /// # Returns:
    /// An `Option` containing a reference to the 16-byte cipher IV.
    pub fn get_cipher_iv(&self) -> DataRefType<'_, 16> {
        return self.cipher_iv.as_ref();
    }

    pub fn set_cipher_iv(&mut self, iv: DataType<16>) {
        self.cipher_iv = iv;
    }

    pub fn set_cipher_key(&mut self, key: DataType<32>) {
        self.cipher_key = key;
    }

    pub fn set_valid_pattern(&mut self, pattern: [u8; 8]) {
        self.valid_pattern = pattern;
    }
}

impl FromStream for FST {
    /// Reads the `FST` structure from a stream and parses its data.
    ///
    /// # Parameters:
    /// - `reader`: A mutable reference to a reader that implements `std::io::Read` and
    ///   `std::io::Seek` traits. This could be a file, buffer, or network stream.
    ///
    /// # Returns:
    /// - On failure, an `Error` is returned.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: std::io::Read + std::io::Seek,
    {
        // Read the encryption algorithm (u16 to EncryptionAlgo), even though it may
        // be unset later on
        self.enc_algo = Some(EncryptionAlgo::try_from(
            reader.read_u16::<LittleEndian>()?,
        )?);

        self.hash_algo = Some(HashAlgo::try_from(reader.read_u16::<LittleEndian>()?)?);
        self.partition_size = reader.read_u32::<LittleEndian>()?;
        reader.read_exact(&mut self.valid_pattern)?; // 8 bytes

        // 4 bytes padding
        read_padding!(reader, 4);

        let flags = reader.read_u8()? & 0b11;
        let enc_enabled = flags & 0b01 == 0x01;
        let hash_enabled = flags & 0b10 != 0;
        // REVISIT: necessary?
        if !enc_enabled {
            self.enc_algo = None;
        }
        if !hash_enabled {
            self.hash_algo = None;
        }

        if reader.read_u8()? & 0b1 == 1 {
            // keys are valid
            reader.seek(std::io::SeekFrom::Current(10))?;
            let mut key = [0; 32];
            let mut iv = [0; 16];
            reader.read_exact(&mut key)?; // 32 bytes
            reader.read_exact(&mut iv)?; // 16 bytes

            self.cipher_key = Some(key);
            self.cipher_iv = Some(iv);
            // align to 96
            read_padding!(reader, 16);
        } else {
            // align to 96
            read_padding!(reader, 74); // 16 + 32 + 16 + 10
        }
        return Ok(());
    }
}

impl ToStream for FST {
    /// Writes the `FST` structure to a stream
    ///
    /// # Parameters:
    /// - `writer`: A mutable reference to a writer that implements the `std::io::Write` trait.
    ///   This could be a file, buffer, or network stream where the `FST` will be written.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        // Write the encryption algorithm and hash algorithm (u16) or a default value
        writer.write_u16::<LittleEndian>(self.enc_algo.unwrap_or_default() as u16)?;
        writer.write_u16::<LittleEndian>(self.hash_algo.unwrap_or_default() as u16)?;
        writer.write_u32::<LittleEndian>(self.partition_size)?;
        writer.write_all(&self.valid_pattern)?; // 8 bytes

        // padding
        write_padding!(writer, 4);

        let flags = if self.enc_algo.is_some() { 0b01 } else { 0 }
            | if self.hash_algo.is_some() { 0b10 } else { 0 };
        writer.write_u8(flags & 0b11)?; // 2 bits
        writer.write_u8(self.is_cipher_key_iv_valid() as u8)?;

        // padding
        write_padding!(writer, 10);
        write_data!(writer, self.cipher_key, 32);
        write_data!(writer, self.cipher_iv, 16);
        // align to 96
        write_padding!(writer, 16);
        Ok(())
    }
}
