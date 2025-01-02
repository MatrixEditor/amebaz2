use std::io;

use crate::error::Error;

pub mod enums;

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
