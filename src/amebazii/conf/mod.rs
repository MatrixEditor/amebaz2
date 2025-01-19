use serde::{Deserialize, Serialize};
use std::fs;

pub mod pt;
pub use pt::{PartitionItemCfg, PartitionTableCfg};

pub mod sysctrl;
pub use sysctrl::SystemDataCfg;

#[macro_export]
macro_rules! expect_length {
    ($value:expr, $expected:literal) => {
        if $value.len() != $expected {
            return Err(crate::error::Error::InvalidState(format!(
                "Expected {} characters in hex string, got {}",
                $expected,
                $value.len()
            )));
        }
    };
}

/// A struct representing an array of bytes of a fixed size, which can be either
/// loaded from a file or encoded as a hexadecimal string.
#[derive(Debug)]
pub struct DataArray<const N: usize> {
    /// The byte array that holds the data.
    pub data: [u8; N],

    /// The optional file path from which the data was loaded (if applicable).
    pub path: Option<String>,
}

impl<const N: usize> DataArray<N> {
    /// Creates a new `DataArray` from a string that is either a file path or a hexadecimal string.
    /// If the path exists, it loads the data from the file. Otherwise, it treats the string as a hex string.
    ///
    /// # Parameters
    /// - `data`: A string which could either be a file path or a hex string.
    ///
    /// # Returns
    /// - A `Result` containing the `DataArray` object or an error if the data is invalid.
    pub fn new(data: String) -> Result<Self, crate::error::Error> {
        if let Ok(exists) = fs::exists(&data) {
            if exists {
                // If the path exists, load data from the file
                return DataArray::load(data);
            }
        }
        // Otherwise, treat the data as a hex string
        DataArray::from_string(data)
    }

    /// Creates a `DataArray` from a hexadecimal string.
    ///
    /// # Parameters
    /// - `data`: A hexadecimal string to decode into the byte array.
    ///
    /// # Returns
    /// - A `Result` containing the `DataArray` object or an error if the data length is invalid.
    pub fn from_string(data: String) -> Result<Self, crate::error::Error> {
        let mut obj = DataArray {
            data: [0; N], // Initialize with an array of zeros
            path: None,   // No file path initially
        };

        // Decode the hexadecimal string into a byte array
        let decoded = hex::decode(data)?;

        // Ensure the decoded data matches the expected length
        if decoded.len() != N {
            return Err(crate::error::Error::InvalidState(format!(
                "Expected {} bytes in hex string, got {}",
                N,
                decoded.len()
            )));
        }

        // Copy the decoded bytes into the `data` field
        obj.data.copy_from_slice(&decoded);
        Ok(obj)
    }

    /// Loads the data from a file at the given path.
    ///
    /// # Parameters
    /// - `path`: The file path to load the data from.
    ///
    /// # Returns
    /// - A `Result` containing the `DataArray` object or an error if the data length is invalid.
    pub fn load(path: String) -> Result<Self, crate::error::Error> {
        // Read the file content into a buffer
        let buffer = fs::read(&path)?;
        let mut obj = DataArray {
            data: [0; N],     // Initialize with an array of zeros
            path: Some(path), // Store the file path
        };

        // Ensure the file contains the expected number of bytes
        if buffer.len() != N {
            return Err(crate::error::Error::InvalidState(format!(
                "Expected {} bytes in file, got {}",
                N,
                buffer.len()
            )));
        }

        // Copy the file data into the `data` field
        obj.data.copy_from_slice(&buffer);
        Ok(obj)
    }
}

impl<'de, const N: usize> Deserialize<'de> for DataArray<N> {
    /// Custom deserialization logic for `DataArray`. It reads the input as a string, which can either be a file path or a hex string.
    ///
    /// # Parameters
    /// - `deserializer`: The deserializer used to read the input.
    ///
    /// # Returns
    /// - A `Result` containing the `DataArray` object or an error.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize the input string (either a path or hex string)
        let s = String::deserialize(deserializer)?;
        // Use `DataArray::new` to create the object from the string
        DataArray::new(s).map_err(serde::de::Error::custom)
    }
}

impl<const N: usize> Serialize for DataArray<N> {
    /// Custom serialization logic for `DataArray`. If the data was loaded from a file, it writes the data to the file and serializes the path.
    /// If the data was provided as a hex string, it serializes the hex string.
    ///
    /// # Parameters
    /// - `serializer`: The serializer used to write the output.
    ///
    /// # Returns
    /// - A `Result` containing the serialized object or an error.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.path {
            Some(path) => {
                // If the data has a path, write the data to the file and serialize the path
                fs::write(path, &self.data).map_err(serde::ser::Error::custom)?;
                serializer.serialize_str(path)
            }
            None => {
                // If there's no path, serialize the data as a hex string
                serializer.serialize_str(&hex::encode(&self.data))
            }
        }
    }
}
