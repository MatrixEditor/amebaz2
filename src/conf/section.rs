use serde::{Deserialize, Serialize};

use crate::types::{
    enums::{SectionType, XipPageRemapSize}, header::SectionHeader, key_from_hex, key_to_hex, DataRefType, DataType
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SectionHeaderCfg {
    pub sect_type: SectionType,

    #[serde(default)]
    pub is_enc: bool,

    #[serde(default)]
    pub sce_en: bool,

    pub xip_key: Option<String>,
    pub xip_iv: Option<String>,

    #[serde(default)]
    pub xip_pg_size: XipPageRemapSize,

    #[serde(default)]
    pub xip_bk_size: u8,
}

impl Default for SectionHeaderCfg {
    fn default() -> Self {
        Self {
            sect_type: SectionType::XIP,
            is_enc: false,
            sce_en: false,
            xip_key: None,
            xip_iv: None,
            xip_pg_size: XipPageRemapSize::default(),
            xip_bk_size: 0,
        }
    }
}

impl SectionHeaderCfg {
    pub fn get_xip_key(&self) -> DataType<16> {
        match &self.xip_key {
            Some(key) => key_from_hex(key.as_str()),
            None => None,
        }
    }

    pub fn get_xip_iv(&self) -> DataType<16> {
        match &self.xip_iv {
            Some(iv) => key_from_hex(iv.as_str()),
            None => None,
        }
    }

    pub fn set_xip_key(&mut self, key: DataRefType<16>) {
        self.xip_key = key_to_hex(key);
    }

    pub fn set_xip_iv(&mut self, iv: DataRefType<16>) {
        self.xip_iv = key_to_hex(iv);
    }
}

impl TryInto<SectionHeader> for SectionHeaderCfg {
    type Error = crate::error::Error;

    fn try_into(self) -> Result<SectionHeader, Self::Error> {
        let mut header = SectionHeader::default();
        header.sect_type = self.sect_type;
        header.sce_enabled = self.sce_en;
        header.
        Ok(header)

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SectionCfg {
    #[serde(default)]
    pub ram_addr: u32,

    #[serde(default)]
    pub load_addr: u32,

    pub filename: Option<String>,
    pub sections: Vec<String>,
    pub shdrcfg: SectionHeaderCfg,
}
