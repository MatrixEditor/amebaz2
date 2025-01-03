use std::io;

use crate::error::Error;

pub mod enums;
pub mod fst;
pub mod header;
pub mod section;
pub mod image;

/// `KeyType` is a type alias for an optional fixed-size array of `u8` bytes.
///
/// This type represents an optional key where the key is an array of `u8` of a fixed size,
/// defined by the constant generic parameter `N`. If the key is not present, the type
/// will be `None`, otherwise, it will contain the key as an array of bytes.
///
/// The fixed size of the key is determined at compile time by the `N` constant, allowing
/// different key sizes to be handled dynamically with a single type alias.
///
/// # Example:
/// ```rust
/// // KeyType with a key of length 4 bytes
/// let key: KeyType<4> = Some([1, 2, 3, 4]);
/// ```
///
/// ## Fields:
/// - `Some([u8; N])`: Contains a fixed-size array of `u8` bytes representing the key.
/// - `None`: Indicates that the key is not present.
///
/// # Type Parameters:
/// - `N`: The fixed length of the key array (i.e., the number of `u8` bytes in the key).
///
/// # Traits Implemented:
/// - Implements `Option`, meaning it can be `Some` containing a key or `None` for a missing key.
pub type KeyType<const N: usize> = Option<[u8; N]>;

/// `KeyRefType` is a type alias for an optional reference to a fixed-size array of `u8` bytes.
///
/// This type is similar to `KeyType`, but instead of owning the key, it holds a reference
/// to a fixed-size array of `u8` bytes, which is useful when the key data is borrowed
/// rather than owned. It is also parameterized by a constant generic `N`, allowing different
/// sizes of keys to be used with a single type.
///
/// `KeyRefType` is typically used when the key is stored elsewhere in memory, and you want
/// to reference it without copying or taking ownership of the data. This is useful for
/// scenarios where the key data already exists and you need to work with it without
/// transferring ownership.
///
/// # Example:
/// ```rust
/// // KeyRefType with a reference to a 4-byte key
/// let key_ref: KeyRefType<4> = Some(&[1, 2, 3, 4]);
/// ```
///
/// ## Fields:
/// - `Some(&[u8; N])`: A reference to a fixed-size array of `u8` bytes representing the key.
/// - `None`: Indicates that the key is not present.
///
/// # Type Parameters:
/// - `N`: The fixed length of the key array (i.e., the number of `u8` bytes in the key).
///
/// # Traits Implemented:
/// - Implements `Option`, meaning it can be `Some` containing a reference to a key or `None` for a missing key
pub type KeyRefType<'a, const N: usize> = Option<&'a [u8; N]>;


/// Checks if a given key is valid.
///
/// # Parameters
/// - `$key`: The key to check, which is expected to be an iterable collection (e.g., a slice or array).
///
/// # Returns
/// - `true` if all bytes in the key are not equal to `0xFF`.
/// - `false` if any byte in the key is equal to `0xFF`.
///
/// # Example
/// ```rust
/// let key = [0x01, 0x02, 0x03, 0x04, 0x05];
/// assert!(is_valid_key!(key)); // All bytes are non-0xFF, so it's valid.
///
/// let invalid_key = [0xFF, 0xFF, 0xFF, 0xFF];
/// assert!(!is_valid_key!(invalid_key)); // Contains only 0xFF bytes, so it's invalid.
/// ```
#[macro_export]
macro_rules! is_valid_key {
    ($key:expr) => {
        // Iterates over the key and checks if any byte is equal to 0xFF.
        $key.iter().all(|&x| x != 0xFF)
    };
}

/// `write_padding!` - A macro to write padding bytes to a writer.
///
/// This macro writes a series of padding bytes (either filled with `0xFF` or a custom byte)
/// to a writer, ensuring that the stream is correctly aligned or that the desired padding size
/// is achieved. It provides two variants:
///
/// 1. **Default fill (`0xFF`)**: The first variant writes padding filled with the byte `0xFF`.
/// 2. **Custom fill**: The second variant allows for specifying a custom byte for padding.
///
/// The macro handles the error propagation automatically, returning the result of the `write_all` method.
///
/// # Parameters:
/// - `$writer`: The writer to which padding bytes should be written. This must implement the
///   `std::io::Write` trait.
/// - `$size`: The size (in bytes) of the padding to be written.
/// - `$fill` (optional): The byte value to fill the padding. Defaults to `0xFF` if not provided.
///
/// # Example 1: Default padding (filled with `0xFF`):
/// ```rust
/// use std::io::Cursor;
/// let mut buffer = Cursor::new(Vec::new());
/// write_padding!(buffer, 16); // Writes 16 bytes of `0xFF` to the buffer
/// ```
///
/// # Example 2: Custom padding byte:
/// ```rust
/// use std::io::Cursor;
/// let mut buffer = Cursor::new(Vec::new());
/// write_padding!(buffer, 8, 0x00); // Writes 8 bytes of `0x00` to the buffer
/// ```
#[macro_export]
macro_rules! write_padding {
    // Variant 1: Default padding (filled with `0xFF`)
    ($writer:expr, $size:literal) => {
        $writer.write_all(&[0xFF; $size])?;
    };

    // Variant 2: Custom padding byte
    ($writer:expr, $size:literal, $fill:literal) => {
        $writer.write_all(&[$fill; $size])?;
    };
}

/// Write a key to a stream with padding support.
///
/// This macro writes the key to the stream if it is present. If the key is `None`,
/// the macro writes padding instead, ensuring the correct number of bytes is always written.
///
/// # Parameters:
/// - `$writer`: The writer where the key (or padding) should be written. Must implement the `std::io::Write` trait.
/// - `$key`: The key to write. This can be `Some([u8; N])` where `N` is the length of the key, or `None` to indicate that the key is missing.
/// - `$len`: The length of the key (or the padding) in bytes. This will determine how many bytes to write if the key is absent.
///
/// # Example:
/// ```rust
/// let mut buffer = Vec::new();
/// let key: Option<[u8; 10]> = Some([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
/// write_key!(buffer, key, 10);
/// ```
#[macro_export]
macro_rules! write_key {
    // Case when the key is present: writes the key to the writer
    ($writer:expr, $key:expr, $len:literal) => {
        if let Some(key) = &$key {
            $writer.write_all(key)?;
        } else {
            // If the key is None, write padding instead
            write_padding!($writer, $len);
        }
    };
}

/// Read (skip) padding bytes in a reader.
///
/// This macro skips a specified number of bytes in the stream, commonly used when there is
/// padding between fields in a binary format.
///
/// # Parameters:
/// - `$reader`: The reader to skip the padding in. This must implement the `std::io::Read`
///   and `std::io::Seek` traits.
/// - `$size`: The number of bytes to skip. This will be passed to the `seek` function in the
///   form of `SeekFrom::Current`.
///
/// # Example:
/// ```rust
/// use std::io::Cursor;
/// let mut buffer = Cursor::new(Vec::new());
/// read_padding!(buffer, 16); // Skips 16 bytes in the buffer
/// ```
#[macro_export]
macro_rules! read_padding {
    // Variant to skip a number of bytes by using SeekFrom::Current
    ($reader:expr, $size:expr) => {
        $reader.seek(io::SeekFrom::Current(($size) as i64))?;
    };
}

/// A trait for types that can be deserialized from a stream.
///
/// This trait provides a method `read_from` that allows a type to implement how it reads
/// data from a stream. Types that implement this trait can be read from a reader (e.g., a
/// file or buffer) using the `from_stream` function.
///
/// # Method
/// - `read_from`: Reads data from a stream into the implementing type.
///
/// # Example
/// ```
/// use amebazii:types::FromStream;
/// struct MyStruct {
///     field1: u32,
///     field2: String,
/// }
///
/// impl FromStream for MyStruct {
///     fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
///     where
///         R: std::io::Read + std::io::Seek,
///     {
///         // Implement logic to read from the reader and populate `self`
///         Ok(())
///     }
/// }
/// ```
pub trait FromStream {
    /// Reads data from a stream and populates the fields of the type.
    ///
    /// This method is called by the `from_stream` function to read data from a provided
    /// reader and deserialize it into the implementing type.
    ///
    /// # Parameters
    /// - `reader`: A mutable reference to the reader from which the data will be read.
    ///
    /// # Returns
    /// - `Ok(())`: If the data is successfully read and the type is populated.
    /// - `Err(Error)`: If an error occurs while reading from the stream.
    ///
    /// # Example
    /// ```rust
    /// use amebazii:types::FromStream;
    ///
    /// let mut reader = std::io::Cursor::new(vec![1, 2, 3, 4]);
    /// let mut my_struct = MyStruct::default();
    /// my_struct.read_from(&mut reader).unwrap();
    /// ```
    fn read_from<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: io::Read + io::Seek;
}

/// Reads a type from a stream.
///
/// This function attempts to read a value of type `T` from a reader that implements both the
/// `io::Read` and `io::Seek` traits. The type `T` must implement the `FromStream` trait to
/// define how it can be read from the stream, and it must also implement `Default` to create
/// an instance to populate.
///
/// # Parameters
/// - `reader`: A mutable reference to the reader from which data will be read.
///
/// # Returns
/// - `Ok(T)`: The deserialized value of type `T`.
/// - `Err(Error)`: If an error occurs while reading from the stream.
///
/// # Example
/// ```rust
/// use amebazii::types::{from_stream, FromStream, Error};
///
/// let mut reader = std::io::Cursor::new(vec![1, 2, 3, 4]);
/// let my_struct: MyStruct = from_stream(&mut reader).unwrap();
/// ```
pub fn from_stream<R, T>(reader: &mut R) -> Result<T, Error>
where
    R: io::Read + io::Seek,
    T: FromStream + Default,
{
    let mut obj = T::default();
    obj.read_from(reader)?;
    Ok(obj)
}

/// A trait for types that can provide their binary size.
///
/// This trait allows types to specify the size, in bytes, of their serialized binary representation.
/// Types that implement this trait must define the `binary_size` method to return the size of the type's binary form.
///
/// # Example
/// ```rust
/// use amebazii::types::BinarySize;
/// struct MyStruct {
///     field1: u32,
///     field2: String,
/// }
///
/// impl BinarySize for MyStruct {
///     fn binary_size() -> usize {
///         // Return the size of `MyStruct`'s binary representation
///         std::mem::size_of::<u32>() + field2.len()
///     }
/// }
/// ```
pub trait BinarySize {
    /// Returns the binary size of the type in bytes.
    ///
    /// # Returns
    /// - `usize`: The number of bytes required to serialize the type.
    ///
    fn binary_size() -> usize;
}

/// A trait for types that can be serialized to a stream.
///
/// This trait defines the `write_to` method, which allows a type to be serialized (written) to a stream, such as a file or buffer.
///
/// # Example
/// ```rust
/// use amebazii::types::{ToStream, Error};
/// struct MyStruct {
///     field1: u32,
///     field2: String,
/// }
///
/// impl ToStream for MyStruct {
///     fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
///     where
///         W: std::io::Write + std::io::Seek,
///     {
///         // Implement the logic to write the struct's fields to the stream
///         Ok(())
///     }
/// }
/// ```
pub trait ToStream {
    /// Writes the type's data to a stream.
    ///
    /// This method serializes the implementing type into a provided stream (writer). It is used by the `transfer_to`
    /// and `to_bytes` functions to write the data to various output formats.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a writer that implements `io::Write` and `io::Seek`.
    ///
    /// # Returns
    /// - `Ok(())`: If the data is successfully written to the stream.
    /// - `Err(Error)`: If an error occurs while writing to the stream.
    ///
    fn write_to<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: io::Write + io::Seek;
}

/// Transfers a type's data to a stream.
///
/// This function serializes the data of the given object (`obj`) and writes it to the provided writer. The object
/// must implement the `ToStream` trait, which specifies how to serialize the data to the stream.
///
/// # Parameters
/// - `obj`: A reference to the object that will be written to the stream.
/// - `writer`: A mutable reference to the writer that will receive the serialized data.
///
/// # Returns
/// - `Ok(())`: If the object was successfully written to the stream.
/// - `Err(Error)`: If an error occurred while writing the object to the stream.
///
/// # Example
/// ```rust
/// use amebazii::types::{transfer_to, ToStream};
///
/// let my_struct = MyStruct { field1: 42, field2: String::from("Hello") };
/// let mut buf = Vec::new();
/// transfer_to(&my_struct, &mut buf).unwrap();
/// ```
pub fn transfer_to<W, T>(obj: &T, writer: &mut W) -> Result<(), Error>
where
    W: io::Write + io::Seek,
    T: ToStream,
{
    obj.write_to(writer)
}

/// Serializes an object into a vector of bytes.
///
/// This function serializes the object (`obj`) into a `Vec<u8>`.
///
/// # Parameters
/// - `obj`: A reference to the object to be serialized.
///
/// # Returns
/// - `Ok(Vec<u8>)`: A vector of bytes representing the serialized object.
/// - `Err(Error)`: If an error occurs while writing the object to the byte vector.
///
/// # Example
/// ```rust
/// use amebazii::types::{to_bytes, ToStream};
///
/// let my_struct = MyStruct { field1: 42, field2: String::from("Hello") };
/// let bytes = to_bytes(&my_struct).unwrap();
/// ```
pub fn to_bytes<T>(obj: &T) -> Result<Vec<u8>, Error>
where
    T: ToStream,
{
    let mut buf = Vec::new();
    let mut cursor = io::Cursor::new(&mut buf);
    obj.write_to(&mut cursor)?;
    Ok(buf)
}

/// Serializes an object into a vector of bytes with an optimized capacity.
///
/// This function serializes the object (`obj`) into a `Vec<u8>`, ensuring that the vector is allocated with
/// the minimum required capacity to hold the serialized data. The object must implement both `ToStream` and
/// `BinarySize` to allow calculating the exact binary size beforehand.
///
/// # Parameters
/// - `obj`: A reference to the object to be serialized.
///
/// # Returns
/// - `Ok(Vec<u8>)`: A vector of bytes with the minimum required capacity for the serialized object.
/// - `Err(Error)`: If an error occurs while writing the object to the byte vector.
pub fn to_bytes_with_capacity<T>(obj: &T) -> Result<Vec<u8>, Error>
where
    T: ToStream + BinarySize,
{
    let mut buf = Vec::with_capacity(T::binary_size());
    let mut cursor = io::Cursor::new(&mut buf);
    obj.write_to(&mut cursor)?;
    Ok(buf)
}
