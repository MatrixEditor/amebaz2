use super::{FromStream, ToStream};
use crate::error::Error;
use std::io;

pub mod boot;
pub use boot::BootImage;

pub mod ota;
pub use ota::{OTAImage, SubImage};

pub mod pt;
pub use pt::{PartTab, PartitionTableImage, Record, TrapConfig};

pub type RawImage = Vec<u8>;

/// A generic enum representing either encrypted or plain data.
///
/// The `EncryptedOr` enum is used to differentiate between encrypted data and unencrypted (plain) data.
/// It allows to store either type of data in the same structure while providing methods to safely access
/// or mutate the contents, depending on whether the data is encrypted or not.
#[derive(Debug)]
pub enum EncryptedOr<T> {
    /// Contains encrypted data as a vector of bytes.
    Encrypted(Vec<u8>),

    /// Contains plain, unencrypted data of type `T`.
    Plain(T),
}

impl<T> AsRef<T> for EncryptedOr<T> {
    /// Returns a reference to the plain data if available.
    ///
    /// # Panics
    /// Panics if the data is encrypted, as the method expects plain data.
    fn as_ref(&self) -> &T {
        match self {
            EncryptedOr::Encrypted(_) => {
                panic!("Cannot get reference to encrypted data when plain is expected")
            }
            EncryptedOr::Plain(t) => t,
        }
    }
}

impl<T> AsRef<[u8]> for EncryptedOr<T> {
    /// Returns a reference to the encrypted data if available.
    ///
    /// # Panics
    /// Panics if the data is plain, as the method expects encrypted data.
    fn as_ref(&self) -> &[u8] {
        match self {
            EncryptedOr::Encrypted(v) => v,
            EncryptedOr::Plain(_) => {
                panic!("Cannot get reference to plain data when encrypted is expected")
            }
        }
    }
}

impl<T> EncryptedOr<T> {
    /// Returns `true` if the data is encrypted.
    ///
    /// This method allows checking if the current instance of `EncryptedOr` contains encrypted data.
    pub fn is_encrypted(&self) -> bool {
        match self {
            EncryptedOr::Encrypted(_) => true,
            EncryptedOr::Plain(_) => false,
        }
    }

    /// Returns `true` if the data is plain.
    ///
    /// This method allows checking if the current instance of `EncryptedOr` contains plain (unencrypted) data.
    pub fn is_plain(&self) -> bool {
        match self {
            EncryptedOr::Encrypted(_) => false,
            EncryptedOr::Plain(_) => true,
        }
    }
}

impl<T> AsMut<T> for EncryptedOr<T> {
    /// Returns a mutable reference to the plain data if available.
    ///
    /// # Panics
    /// Panics if the data is encrypted, as the method expects plain data.
    fn as_mut(&mut self) -> &mut T {
        match self {
            EncryptedOr::Encrypted(_) => {
                panic!("Cannot get mutable reference to encrypted data when plain is expected")
            }
            EncryptedOr::Plain(t) => t,
        }
    }
}

impl<T> AsMut<[u8]> for EncryptedOr<T> {
    /// Returns a mutable reference to the encrypted data if available.
    ///
    /// # Panics
    /// Panics if the data is plain, as the method expects encrypted data.
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            EncryptedOr::Encrypted(v) => v,
            EncryptedOr::Plain(_) => {
                panic!("Cannot get mutable reference to encrypted data when plain is expected")
            }
        }
    }
}

impl<T: ToStream> ToStream for EncryptedOr<T> {
    /// Writes the data to a stream, either encrypted or plain.
    ///
    /// This method is used to serialize the data into a stream. If the data is encrypted, it writes the encrypted byte vector,
    /// otherwise it serializes the plain data of type `T`.
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        match self {
            EncryptedOr::Encrypted(v) => writer.write_all(v)?,
            EncryptedOr::Plain(t) => return t.write_to(writer),
        }
        Ok(())
    }
}

impl<T: ToStream> ToStream for EncryptedOr<Vec<T>> {
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek,
    {
        match self {
            EncryptedOr::Encrypted(v) => writer.write_all(v)?,
            EncryptedOr::Plain(t) => {
                for item in t {
                    item.write_to(writer)?;
                }
            }
        }
        Ok(())
    }
}

impl<T: FromStream> FromStream for EncryptedOr<T> {
    /// Reads the data from a stream, either encrypted or plain.
    ///
    /// This method deserializes the data from a stream. If the data is encrypted, it reads the encrypted byte vector,
    /// otherwise it reads the plain data of type `T`.
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: io::Read + io::Seek,
    {
        match self {
            EncryptedOr::Encrypted(v) => reader.read_exact(v)?,
            EncryptedOr::Plain(t) => return t.read_from(reader),
        }
        Ok(())
    }
}

/// A trait that provides common functionality for image-like objects,
/// such as computing and setting the segment size and signature.
pub trait AsImage {
    /// Computes the segment size for the image.
    ///
    /// The segment size typically represents the total size of the image segment,
    /// including all of its components (e.g., header, records, user data, etc.).
    ///
    /// # Returns:
    /// - `u32` representing the segment size.
    ///
    /// # Example:
    /// ```rust
    /// let segment_size = image.build_segment_size();
    /// ```
    fn build_segment_size(&self) -> u32;

    /// Sets the segment size for the image.
    ///
    /// This method allows setting the segment size, typically after calculating
    /// it or modifying the image in some way.
    ///
    /// # Arguments:
    /// - `size`: The new segment size to set.
    ///
    /// # Example:
    /// ```rust
    /// image.set_segment_size(1024);
    /// ```
    fn set_segment_size(&mut self, size: u32);

    /// Computes the signature for the image using the provided key.
    ///
    /// The signature is usually a hash or HMAC generated from the image data and
    /// a secret key, often used for verification or authentication purposes.
    ///
    /// # Arguments:
    /// - `key`: The key used to compute the signature.
    ///
    /// # Returns:
    /// - `Result<Vec<u8>, crate::error::Error>`: The signature as a `Vec<u8>`, or an error.
    ///
    /// # Example:
    /// ```rust
    /// let signature = image.build_signature(&key);
    /// ```
    fn build_signature(&self, key: Option<&[u8]>) -> Result<Vec<u8>, crate::error::Error>;

    /// Sets the signature for the image.
    ///
    /// This method allows setting the signature after computing it or for some
    /// validation process.
    ///
    /// # Arguments:
    /// - `signature`: The computed signature to set.
    ///
    /// # Example:
    /// ```rust
    /// image.set_signature(&signature);
    /// ```
    fn set_signature(&mut self, signature: &[u8]);
}

/// Builds the signature for a given image.
///
/// This function uses the `build_signature` method from the `AsImage` trait to generate
/// the signature for the image using the provided key.
///
/// # Arguments:
/// - `image`: The image-like object that implements `AsImage`.
/// - `key`: The key used to compute the signature.
///
/// # Returns:
/// - `Result<Vec<u8>, crate::error::Error>`: The computed signature.
pub fn build_default_signature<I>(
    image: &I,
    key: Option<&[u8]>,
) -> Result<Vec<u8>, crate::error::Error>
where
    I: AsImage,
{
    image.build_signature(key)
}

/// Sets the signature for a given image.
///
/// This function computes the signature using `build_default_signature` and then sets
/// the signature for the image using `set_signature`.
///
/// # Arguments:
/// - `image`: The image-like object that implements `AsImage`.
/// - `key`: The key used to compute the signature.
///
/// # Returns:
/// - `Result<(), crate::error::Error>`: An empty result on success, or an error.
pub fn set_default_signature<I>(
    image: &mut I,
    key: Option<&[u8]>,
) -> Result<(), crate::error::Error>
where
    I: AsImage,
{
    let signature = build_default_signature(image, key)?;
    image.set_signature(&signature);
    Ok(())
}

/// Builds the segment size for a given image.
///
/// # Arguments:
/// - `image`: The image-like object that implements `AsImage`.
///
/// # Returns:
/// - `u32`: The computed segment size.
pub fn build_segment_size<I>(image: &I) -> u32
where
    I: AsImage,
{
    image.build_segment_size()
}

/// Sets the segment size for a given image.
///
/// # Arguments:
/// - `image`: The image-like object that implements `AsImage`.
///
/// # Returns:
/// - `()`: An empty result on success.
pub fn set_default_segment_size<I>(image: &mut I)
where
    I: AsImage,
{
    image.set_segment_size(image.build_segment_size())
}
