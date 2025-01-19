//! Utility functions for creating signatures and hashes.

use openssl::md::Md;
use openssl::md_ctx::MdCtx;
use openssl::pkey::PKey;
use std::io;

use crate::error;

/// Skips bytes in the provided reader to ensure that the next read operation aligns
/// with the specified alignment.
///
/// This function calculates the number of bytes to skip in order to make the current
/// position in the stream a multiple of the given `align` value. If the current position
/// is already aligned, no bytes are skipped.
///
/// # Parameters
/// - `reader`: A mutable reference to a reader that implements the `Seek` trait. This
///   is typically a stream or file from which you want to seek.
/// - `align`: The alignment boundary (in bytes) to which the current stream position
///   should be aligned. Must be a power of two.
///
/// # Returns
/// - An `Err(io::Error)` if an I/O error occurs while seeking.
///
/// # Example
/// ```
/// use std::io::{Cursor, Seek, SeekFrom};
/// use amebazii::util::skip_aligned;
///
/// let mut cursor = Cursor::new(vec![0u8; 100]);
/// let align = 16;
/// skip_aligned(&mut cursor, align).unwrap();
/// assert_eq!(cursor.position() % align, 0); // The position should now be aligned to 16.
/// ```
///
/// # Errors
/// This function may return errors related to seeking if the underlying reader does not
/// support seeking or encounters an I/O issue.
pub fn skip_aligned<S>(reader: &mut S, align: u64) -> Result<(), io::Error>
where
    S: std::io::Seek,
{
    let skip = reader.stream_position()? % align;
    if skip > 0 {
        reader.seek(io::SeekFrom::Current(align as i64 - skip as i64))?;
    }
    Ok(())
}

/// Computes an HMAC-MD5 signature for the provided key and data.
///
/// This function generates an HMAC (Hash-based Message Authentication Code) using the MD5
/// hashing algorithm. The key and data are processed, and the resulting 128-bit (16-byte)
/// signature is returned.
///
/// # Parameters
/// - `key`: A byte slice representing the secret key used for the HMAC computation.
/// - `data`: A byte slice containing the data to be authenticated.
///
/// # Returns
/// - `[u8; 16]`: A 16-byte array containing the HMAC-MD5 signature.
/// - `Err(error::Error)`: An error if there is a failure during the HMAC computation (e.g., key or data issues, cryptographic errors).
///
/// # Example
/// ```
/// use amebazii::util::hmac_md5;
///
/// let key = b"secret";
/// let data = b"message";
/// let signature = hmac_md5(key, data).unwrap();
/// assert_eq!(signature.len(), 16); // The HMAC-MD5 signature should be 16 bytes.
/// ```
///
/// # Errors
/// This function may return an error if any step in the HMAC-MD5 computation fails.
pub fn hmac_md5(key: &[u8], data: &[u8]) -> Result<[u8; 16], error::Error> {
    let mut signature = [0xFF; 16];
    let mut ctx = MdCtx::new()?;
    let pkey = PKey::hmac(key)?;

    ctx.digest_sign_init(Some(Md::md5()), &pkey)?;
    ctx.digest_update(data)?;
    ctx.digest_sign_final(Some(&mut signature))?;
    return Ok(signature);
}

/// Computes an HMAC-SHA256 signature for the provided key and data.
///
/// This function generates an HMAC (Hash-based Message Authentication Code) using the SHA-256
/// hashing algorithm. The key and data are processed, and the resulting 256-bit (32-byte)
/// signature is returned.
///
/// # Parameters
/// - `key`: A byte slice representing the secret key used for the HMAC computation.
/// - `data`: A byte slice containing the data to be authenticated.
///
/// # Returns
/// - `[u8; 32]`: A 32-byte array containing the HMAC-SHA256 signature.
/// - `Err(error::Error)`: An error if there is a failure during the HMAC computation (e.g., key or data issues, cryptographic errors).
///
/// # Example
/// ```
/// use amebazii::util::hmac_sha256;
///
/// let key = b"secret";
/// let data = b"message";
/// let signature = hmac_sha256(key, data).unwrap();
/// assert_eq!(signature.len(), 32); // The HMAC-SHA256 signature should be 32 bytes.
/// ```
///
/// # Errors
/// This function may return an error if any step in the HMAC-SHA256 computation fails.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<[u8; 32], error::Error> {
    let mut signature = [0xFF; 32];
    let mut ctx = MdCtx::new()?;
    let pkey = PKey::hmac(key)?;

    ctx.digest_sign_init(Some(Md::sha256()), &pkey)?;
    ctx.digest_update(data)?;
    ctx.digest_sign_final(Some(&mut signature))?;
    return Ok(signature);
}

/// Computes a SHA-256 hash of the provided data.
///
/// This function computes the SHA-256 hash of the input data. The result is the 256-bit
/// (32-byte) hash.
///
/// # Parameters
/// - `data`: A byte slice containing the data to be hashed.
///
/// # Returns
/// - `[u8; 32]`: A 32-byte array containing the computed SHA-256 hash.
/// - `Err(error::Error)`: An error if there is a failure during the hashing process.
///
/// # Example
/// ```
/// use my_crate::sha256;
/// let data = b"message";
/// let hash = sha256(data).unwrap();
/// assert_eq!(hash.len(), 32); // The SHA-256 hash should be 32 bytes.
/// ```
///
/// # Errors
/// This function may return an error if there is a failure during the SHA-256 hashing process.
pub fn sha256(data: &[u8]) -> Result<[u8; 32], error::Error> {
    let mut signature = [0xFF; 32];
    let mut ctx = MdCtx::new()?;

    ctx.digest_init(Md::sha256())?;
    ctx.digest_update(data)?;
    ctx.digest_final(&mut signature)?;
    return Ok(signature);
}

/// Computes an MD5 hash of the provided data.
///
/// This function computes the MD5 hash of the input data. The result is a 128-bit
/// (16-byte) hash.
///
/// # Parameters
/// - `data`: A byte slice containing the data to be hashed.
///
/// # Returns
/// - `Ok([u8; 16])`: A 16-byte array containing the computed MD5 hash.
/// - `Err(error::Error)`: An error if there is a failure during the hashing process.
///
/// # Example
/// ```
/// use my_crate::md5;
/// let data = b"message";
/// let hash = md5(data).unwrap();
/// assert_eq!(hash.len(), 16); // The MD5 hash should be 16 bytes.
/// ```
///
/// # Errors
/// This function may return an error if there is a failure during the MD5 hashing process.
pub fn md5(data: &[u8]) -> Result<[u8; 16], error::Error> {
    let mut signature = [0xFF; 16];
    let mut ctx = MdCtx::new()?;

    ctx.digest_init(Md::md5())?;
    ctx.digest_update(data)?;
    ctx.digest_final(&mut signature)?;
    return Ok(signature);
}

/// Writes the specified byte `fill` repeatedly to the writer for the given length.
///
/// This function writes the byte `fill` to the writer in chunks, filling the writer
/// with `length` bytes of the specified value. It is more memory-efficient as it avoids
/// allocating a large buffer in memory by writing in smaller chunks.
///
/// # Parameters
/// - `writer`: A mutable reference to a writer that implements the `Write` trait. This can
///   be a file, buffer, or network stream to which the data will be written.
/// - `fill`: The byte value to fill the output with. It will be written repeatedly to the
///   writer.
/// - `length`: The number of times the byte `fill` will be written to the writer.
///
/// # Returns
/// - `Err(io::Error)`: If an I/O error occurs while writing.
///
/// # Example
/// ```
/// use std::io::{Cursor, Write};
/// use my_crate::write_fill;
///
/// let mut cursor = Cursor::new(vec![]);
/// write_fill(&mut cursor, b'A', 10).unwrap();
/// assert_eq!(cursor.into_inner(), vec![b'A'; 10]); // Writes 10 'A' bytes.
/// ```
///
/// # Performance
/// This implementation writes the byte `fill` in smaller chunks, reducing memory usage
/// and making it more efficient for large data sizes.
pub fn write_fill<W>(writer: &mut W, fill: u8, length: u64) -> Result<(), io::Error>
where
    W: std::io::Write,
{
    const CHUNK_SIZE: usize = 4096;
    let mut remaining = length;
    let buf = [fill; CHUNK_SIZE];

    while remaining > 0 {
        let chunk_size = remaining.min(CHUNK_SIZE as u64) as usize;
        writer.write_all(&buf[..chunk_size])?;
        remaining -= chunk_size as u64;
    }

    Ok(())
}
