use serde::{Deserialize, Serialize};

use crate::types::{
    enums::{EncryptionAlgo, HashAlgo, ImageType},
    key_from_hex, DataType,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageHeaderCfg {
    pub next_img: Option<String>,
    pub img_type: ImageType,

    #[serde(default)]
    pub is_encrypt: bool,

    #[serde(default)]
    pub serial: u32,

    #[serde(default)]
    pub user_key_valid: bool,

    pub user_key1: Option<String>,
    pub user_key2: Option<String>,
}

impl Default for ImageHeaderCfg {
    fn default() -> Self {
        Self {
            next_img: None,
            img_type: ImageType::Unknown,
            is_encrypt: false,
            serial: 0,
            user_key_valid: false,
            user_key1: None,
            user_key2: None,
        }
    }
}

impl ImageHeaderCfg {
    pub fn get_user_key1(&self) -> DataType<16> {
        match &self.user_key1 {
            Some(key) => key_from_hex(key),
            None => None,
        }
    }

    pub fn get_user_key2(&self) -> DataType<16> {
        match &self.user_key2 {
            Some(key) => key_from_hex(key),
            None => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FSTCfg {
    pub enc_algo: Option<EncryptionAlgo>, // already covers enc_en
    pub hash_algo: Option<HashAlgo>,      // already covers hash_en
    pub part_size: u32,

    pub cipher_key: Option<String>,
    pub cipher_iv: Option<String>,
    pub hash_key: Option<String>,
    pub valid_pat: Option<String>,
}
