use crate::{error::Error, is_valid_key};

use super::{BinarySize, FromStream, ToStream};

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