pub mod pt;
pub mod boot;
pub mod ota;

pub type RawImage = Vec<u8>;

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
pub fn build_default_signature<I>(image: &I, key: Option<&[u8]>) -> Result<Vec<u8>, crate::error::Error>
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
pub fn set_default_signature<I>(image: &mut I, key: Option<&[u8]>) -> Result<(), crate::error::Error>
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
