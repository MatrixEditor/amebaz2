/// Represents a pair of cryptographic keys: a private key and a public key.
pub struct ConstKeyPair {
    priv_key: &'static [u8; 32],  // The private key (32 bytes)
    pub_key: &'static [u8; 32],   // The public key (32 bytes)
}

impl ConstKeyPair {
    /// Creates a new `KeyPair` instance with the provided private and public keys.
    ///
    /// # Parameters
    /// - `priv_key`: A reference to a 32-byte array representing the private key.
    /// - `pub_key`: A reference to a 32-byte array representing the public key.
    ///
    /// # Returns
    /// Returns a new `KeyPair` instance containing the provided keys.
    pub const fn new(priv_key: &'static [u8; 32], pub_key: &'static [u8; 32]) -> Self {
        ConstKeyPair { priv_key, pub_key }
    }

    /// Returns a reference to the private key.
    ///
    /// # Returns
    /// Returns a reference to the 32-byte private key.
    pub const fn get_priv_key(&self) -> &'static [u8; 32] {
        self.priv_key
    }

    /// Returns a reference to the public key.
    ///
    /// # Returns
    /// Returns a reference to the 32-byte public key.
    pub const fn get_pub_key(&self) -> &'static [u8; 32] {
        self.pub_key
    }
}

// Default values for cryptographic keys and patterns used throughout the system.

// The default hash key used to generate signatures for the partition table.
pub const HASH_KEY: &[u8; 32] =
    b"\x47\xe5\x66\x13\x35\xa4\xc5\xe0\xa9\x4d\x69\xf3\xc7\x37\xd5\x4f\x23\x83\x79\x13\x32\x93\x97\x53\xef\x24\x27\x96\x08\xf6\xd7\x2b";

// The default Initialization Vector (IV) used for encryption/decryption operations.
pub const DEFAULT_IV: &[u8; 16] =
    b"\xe7\x91\x9e\xe6\x98\xb1\xe5\x8d\x8a\xe5\xb0\x8e\xe9\xab\x94\x38";

// Default second key used in the application subimage for encryption.
pub const APP_DEFAULT_USER_KEY2: &[u8; 32] =
    b"\xbb\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0b\x0c\r\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f";

// Default first key used in the boot image for encryption.
pub const BOOT_DEFAULT_USER_KEY1: &[u8; 32] =
    b"\xaa\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0b\x0c\r\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f";

// Default AES encryption key used for encrypting/decrypting XIP section data.
pub const XIP_KEY: &[u8; 16] = b"\xa0\xd6\xda\xe7\xe0b\xca\x94\xcb\xb2\x94\xbf\x89k\x9fh";

// Default IV used for encrypting/decrypting XIP section data.
pub const XIP_IV: &[u8; 16] = b"\x94\x87\x94\x87\x94\x87\x94\x87\x94\x87\x94\x87\x94\x87\x94\x87";

// Default flash calibration pattern used in flash
pub const FLASH_PATTERN: &[u8; 16] =
    b"\x99\x99\x96\x96\x3f\xcc\x66\xfc\xc0\x33\xcc\x03\xe5\xdc\x31\x62";

// Default cryptographic key pair for general use in encryption/decryption images.
pub const KEY_PAIR_000: ConstKeyPair = ConstKeyPair::new(
    b"\xa0\xd6\xda\xe7\xe0b\xca\x94\xcb\xb2\x94\xbf\x89k\x9fh\xcf\x848wBV\xact\x03\xcaO\xd9\xa1\xc9VO",
    b"hQ>\xf8>9k\x12\xba\x05\x9a\x90\x0f6\xb6\xd3\x1d\x11\xfe\x1c]%\xeb\x8a\xa7\xc5P0\x7f\x9c$\x05",
);

// Default hash key pair used for hashing operations (not seen).
pub const KEY_PAIR_001: ConstKeyPair = ConstKeyPair::new(
    b"\x88*\xa1l\x8cD\xa7v\n\xa8\xc9\xab\"\xe3V\x8co\xa1l*\xfaO\x0c\xea)\xa1\n\xbc\xdf`\xe4O",
    b"H\xad#\xdd\xbd\xac\x9eeq\x9d\xb7\xd3\x94\xd4Mb\x82\r\x19\xe5\rh7gt#~\x98\xd20^j",
);

// Unused key pair; not seen
pub const KEY_PAIR_002: ConstKeyPair = ConstKeyPair::new(
    b"X\xa3\xd9\x15ph5!\"`\xc2-b\x8b3m\x13\x19\x0bS\x97\x14\xe3\xdb$\x9d\x82<\xa5wDS",
    b"\xfd\x8d?>Qm\x96\x18n\x10\xf0zd\xb2L}\xe76\x82j$\xfa\xfe6~y\xf1\xfb\xb2\xf1\xc82",
);

// Default key pair used for firmware signature generation (used in OTA operations).
pub const KEY_PAIR_003: ConstKeyPair = ConstKeyPair::new(
    b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0b\x0c\r\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e_",
    b"\x8f@\xc5\xad\xb6\x8f%bJ\xe5\xb2\x14\xeavzn\xc9M\x82\x9d={^\x1a\xd1\xbao>!8(_"
);

