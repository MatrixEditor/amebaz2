use std::io;

use crate::error::Error;

pub mod enums;
pub use enums::*; // revisit

pub mod flash;
pub use flash::{Flash, Partition};

pub mod fst;
pub use fst::FST;

pub mod header;
pub use header::{EntryHeader, ImageHeader, KeyBlock, SectionHeader};

pub mod image;
pub use image::*; // revisit

pub mod section;
pub use section::Section;

pub mod sysctrl;
pub use sysctrl::{FlashInfo, ForceOldImage, SpiConfig, SystemData};

/// `DataType` is a type alias for an optional fixed-size array of `u8` bytes.
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
/// // DataType with a key of length 4 bytes
/// let key: DataType<4> = Some([1, 2, 3, 4]);
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
pub type DataType<const N: usize> = Option<[u8; N]>;

/// Converts a hexadecimal string into a `DataType` array.
///
/// This function takes a hexadecimal string (`hexstr`), decodes it into bytes,
/// and then attempts to convert the bytes into a `DataType` of a specific size.
///
/// The size of the resulting `DataType` is determined by the constant `N`. This function
/// will panic if the length of the decoded byte array does not match the expected size `N`.
///
/// # Type Parameters:
/// - `N`: The size of the `DataType` array. This is a constant array length that the decoded
///   hexadecimal string must match. It is passed at compile-time to ensure type safety.
///
/// # Arguments:
/// - `hexstr`: A string containing the hexadecimal representation of the key. The string must
///   contain an even number of characters (each representing a byte).
///
/// # Returns:
/// - `Some(DataType<N>)`: A `DataType` array of size `N`, constructed from the decoded bytes.
///   Returns `None` if the hexadecimal string is empty or the decoded bytes do not match the expected length.
///
/// # Panics:
/// - This function will panic if the length of the decoded byte array is not equal to `N`.
///
/// # Example:
/// ```
/// let hexstr = "a1b2c3d4e5f67890";
/// let key = key_from_hex::<8>(hexstr);
/// assert_eq!(key, Some([0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x78, 0x90]));
/// ```
pub fn key_from_hex<const N: usize>(hexstr: &str) -> DataType<N> {
    let bytes = hex::decode(hexstr).unwrap();
    assert!(bytes.len() == N);
    Some(bytes.try_into().unwrap())
}

/// Converts a `DataType` array into a hexadecimal string.
///
/// This function takes a `DataType` array (`key`) and converts it into its corresponding
/// hexadecimal string representation. If the key is `None`, it returns `None`.
///
/// # Type Parameters:
/// - `N`: The size of the `DataType` array. This is a constant array length that ensures type safety.
///
/// # Arguments:
/// - `key`: A reference to a `DataType` array of size `N`. This is the key to be encoded into a
///   hexadecimal string. It must be a valid key array of the appropriate size.
///
/// # Returns:
/// - `Some(String)`: The hexadecimal string representation of the key if the key is `Some`.
///   Returns `None` if the key is `None`.
///
/// # Example:
/// ```
/// let key = Some([0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x78, 0x90]);
/// let hexstr = key_to_hex(key);
/// assert_eq!(hexstr, Some("a1b2c3d4e5f67890".to_string()));
/// ```
pub fn key_to_hex<const N: usize>(key: DataRefType<N>) -> Option<String> {
    match key {
        None => None,                        // If the key is None, return None.
        Some(key) => Some(hex::encode(key)), // Otherwise, convert the key to a hex string and return.
    }
}

/// `DataRefType` is a type alias for an optional reference to a fixed-size array of `u8` bytes.
///
/// This type is similar to `DataType`, but instead of owning the key, it holds a reference
/// to a fixed-size array of `u8` bytes, which is useful when the key data is borrowed
/// rather than owned. It is also parameterized by a constant generic `N`, allowing different
/// sizes of keys to be used with a single type.
///
/// `DataRefType` is typically used when the key is stored elsewhere in memory, and you want
/// to reference it without copying or taking ownership of the data. This is useful for
/// scenarios where the key data already exists and you need to work with it without
/// transferring ownership.
///
/// # Example:
/// ```rust
/// let key_ref: DataRefType<4> = Some(&[1, 2, 3, 4]);
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
pub type DataRefType<'a, const N: usize> = Option<&'a [u8; N]>;

/// Checks if the given data is valid by ensuring none of the bytes are equal to `0xFF`.
///
/// This macro checks if all elements in the provided data are non-`0xFF`.
///
/// # Parameters
/// - `$key`: The key or data collection to check, which must be an iterable type (e.g., a slice or array).
///
/// # Returns
/// - `true` if no byte in the data is `0xFF`.
/// - `false` if any byte in the data is `0xFF`.
///
/// # Example
/// ```rust
/// let key = [0x01, 0x02, 0x03, 0x04, 0x05];
/// assert!(is_valid_data!(key)); // All bytes are non-0xFF, so it's valid.
///
/// let invalid_key = [0xFF, 0xFF, 0xFF, 0xFF];
/// assert!(!is_valid_data!(invalid_key)); // Contains only 0xFF bytes, so it's invalid.
/// ```
#[macro_export]
macro_rules! is_valid_data {
    ($key:expr) => {
        $key.iter().any(|&x| x != 0xFF)
    };
}

/// Reads valid data from the reader into the target, ensuring that the data does not contain any `0xFF` bytes.
///
/// This macro attempts to read a specific amount of data from a reader, checks if the data is valid
/// (i.e., it does not contain any `0xFF` bytes), and if valid, assigns the data to the provided target.
///
/// # Parameters
/// - `$target`: The target variable where the data will be stored (of type `Option<[u8; $length]>`).
/// - `$length`: The length of the data to read (must match the expected size of the data).
/// - `$reader`: The reader from which the data will be read. The reader must implement the `Read` trait.
///
/// # Example
/// ```rust
/// let mut reader: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05];
/// let mut target: Option<[u8; 5]> = None;
/// read_valid_data!(target, 5, reader);
/// assert!(target.is_some()); // The data read is valid, so target should be Some([0x01, 0x02, 0x03, 0x04, 0x05]).
/// ```
///
/// # Error Handling
/// - If the data contains any `0xFF` byte, it will not be assigned to the target.
/// - The macro expects the reader to support reading the exact number of bytes as specified by `$length`.
/// - This macro will return an error if the reader cannot fulfill the request.
#[macro_export]
macro_rules! read_valid_data {
    ($target:expr, $length:expr, $reader:expr) => {
        let mut buf = [0u8; $length];
        $reader.read_exact(&mut buf)?;
        if !is_valid_data!(buf) {
            $target = Some(buf);
        }
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
        if $size > 4096 {
            write_fill($writer, 0xFF, $size as u64)?;
        } else {
            $writer.write_all(&vec![0xFF; $size as usize])?;
        }
    };

    // Variant 2: Custom padding byte
    ($writer:expr, $size:literal, $fill:literal) => {
        if $size > 4096 {
            write_fill($writer, $fill, $size as u64)?;
        } else {
            $writer.write_all(&vec![$fill; $size as usize])?;
        }
    };
}

/// Writes data to a stream.
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
/// write_data!(buffer, key, 10);
/// ```
#[macro_export]
macro_rules! write_data {
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

/// Writes padding to a binary stream to ensure that the next write operation aligns to a specified size.
///
/// This macro writes padding bytes to the stream in order to align the current write position to the specified size.
/// The padding is done with a specified fill byte and can optionally be skipped if the alignment is already met.
///
/// The macro can be used in different forms depending on whether you need to specify a fill byte and whether the padding
/// is optional. The following variants are available:
///
/// 1. **Default Padding with 0x00 Fill (non-optional):**
///   Aligns the current stream position to the next boundary of the specified size and fills with `0x00`.
///
///   ```rust
///   write_aligned!(writer, 16);
///   ```
///   This will ensure the stream is aligned to a 16-byte boundary, and `0x00` is used for padding.
///
/// 2. **Default Padding with Custom Fill (non-optional):**
///   Aligns the current stream position to the next boundary of the specified size and fills with a custom byte value.
///
///   ```rust
///   write_aligned!(writer, 16, 0xFF);
///   ```
///   This will align to a 16-byte boundary and use `0xFF` as the padding byte.
///
/// 3. **Optional Padding with 0x00 Fill:**
///   Optionally applies padding if necessary to align the stream position to the specified size. If the stream is already
///   aligned, no padding is written.
///
///   ```rust
///   write_aligned!(writer, 16, optional);
///   ```
///   This will only write padding if needed to align to a 16-byte boundary and will use `0x00` as the fill byte.
///
/// 4. **Optional Padding with Custom Fill:**
///   Optionally applies padding with a custom fill byte if the stream is not already aligned to the specified size.
///
///   ```rust
///   write_aligned!(writer, 16, 0xFF, optional);
///   ```
///   This will apply padding with `0xFF` only if needed to align to a 16-byte boundary.

#[macro_export]
macro_rules! write_aligned {
    // 1. Default padding with 0x00 fill and optional padding
    ($writer:expr, $size:expr, optional) => {
        write_aligned!($writer, $size, 0x00, optional);
    };

    // 2. Default padding with 0x00 fill (non-optional)
    ($writer:expr, $size:expr) => {
        write_aligned!($writer, $size, 0x00);
    };

    // 3. Custom padding with optional fill byte
    ($writer:expr, $size:expr, $fill:expr, optional) => {
        let pos = $writer.stream_position()?;
        let padding = (pos % $size);
        if padding > 0 {
            if padding > 4096 {
                write_fill($writer, $fill, ($size - padding) as u64)?;
            } else {
                $writer.write_all(&vec![$fill; ($size - padding) as usize])?;
            }
        }
    };

    // 4. Custom padding with specified fill byte (non-optional)
    ($writer:expr, $size:expr, $fill:expr) => {
        let pos = $writer.stream_position()?;
        let padding = $size - (pos % $size);
        if padding > 4096 {
            write_fill($writer, $fill, padding as u64)?;
        } else {
            $writer.write_all(&vec![$fill; padding as usize])?;
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
