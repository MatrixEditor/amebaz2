use std::fs;

use serde::{Deserialize, Serialize};

use crate::{
    expect_length,
    types::{
        enums::{KeyExportOp, PartitionType},
        image::pt::{PartTab, Record, TrapConfig},
        key_from_hex,
    },
};

/// Represents a single item in the partition table configuration.
///
#[derive(Serialize, Deserialize, Debug)]
pub struct PartitionItemCfg {
    pub start_addr: u32,
    pub length: u32,
    pub part_type: PartitionType,

    #[serde(default)]
    pub debug_skip: bool,
    pub hash_key: String,
}

impl TryInto<Record> for PartitionItemCfg {
    type Error = crate::error::Error;

    /// Converts a `PartitionItemCfg` instance into a `Record` instance.
    ///
    /// This conversion checks if the `hash_key` is a valid 64-character hexadecimal string.
    /// If valid, it maps the `PartitionItemCfg` attributes into the `Record` struct.
    ///
    /// # Returns:
    /// - `Ok(Record)`: A `Record` instance with values copied from `PartitionItemCfg`.
    /// - `Err(Error)`: An error if the `hash_key` is invalid.
    fn try_into(self) -> Result<Record, Self::Error> {
        expect_length!(&self.hash_key, 64); // Ensure that the hash key is 64 characters.

        let mut record = Record::default();
        record.start_addr = self.start_addr;
        record.length = self.length;
        record.part_type = self.part_type;
        record.dbg_skip = self.debug_skip;
        record.set_hash_key(key_from_hex(&self.hash_key)); // Convert the hash key from hexadecimal string to bytes.

        Ok(record)
    }
}

impl Default for PartitionItemCfg {
    /// Returns the default configuration for a `PartitionItemCfg` instance.
    ///
    /// The default values are:
    /// - `start_addr`: `0`
    /// - `length`: `0`
    /// - `part_type`: `PartitionType::PartTab` (the default partition type).
    /// - `debug_skip`: `false`
    /// - `hash_key`: An empty string.
    fn default() -> Self {
        Self {
            start_addr: 0,
            length: 0,
            part_type: PartitionType::PartTab,
            debug_skip: false,
            hash_key: String::new(),
        }
    }
}

/// Represents the partition table configuration.
///
/// This struct holds the configuration settings for a partition table.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct PartitionTableCfg {
    #[serde(default)]
    pub rma_w_state: u8,

    #[serde(default)]
    pub rma_ov_state: u8,

    #[serde(default)]
    pub eFWV: u8,

    pub fw1_idx: u8,
    pub fw2_idx: u8,

    pub ota_trap: Option<TrapConfig>,
    pub mp_trap: Option<TrapConfig>,

    #[serde(default)]
    pub key_exp_op: KeyExportOp,

    pub user_ext: Option<String>,
    pub user_bin: Option<String>,

    pub items: Vec<PartitionItemCfg>,
}

impl TryInto<PartTab> for PartitionTableCfg {
    type Error = crate::error::Error;

    /// Converts a `PartitionTableCfg` instance into a `PartTab` instance.
    ///
    /// This method converts all relevant fields from the `PartitionTableCfg` struct into
    /// a `PartTab` struct, including handling the user extension data and binary data
    /// from external files (if provided).
    ///
    /// # Returns:
    /// - `Ok(PartTab)`: A `PartTab` instance with the appropriate values set.
    /// - `Err(Error)`: An error if something goes wrong, such as if the user extension file
    ///   does not exist or cannot be read.
    fn try_into(self) -> Result<PartTab, Self::Error> {
        let mut pt = PartTab::default();
        pt.eFWV = self.eFWV;
        pt.fw1_idx = self.fw1_idx;
        pt.fw2_idx = self.fw2_idx;
        pt.key_exp_op = self.key_exp_op;
        pt.rma_ov_state = self.rma_ov_state;
        pt.rma_w_state = self.rma_w_state;

        if let Some(ota_trap) = self.ota_trap {
            pt.ota_trap = ota_trap;
        }
        if let Some(mp_trap) = self.mp_trap {
            pt.mp_trap = mp_trap;
        }

        if let Some(user_ext) = &self.user_ext {
            if fs::exists(&user_ext)? {
                pt.set_user_ext(&fs::read(&user_ext)?);
            } else {
                expect_length!(&user_ext, 24);
                if let Some(value) = key_from_hex::<12>(user_ext) {
                    pt.set_user_ext(&value);
                }
            }
        }

        if let Some(user_bin) = &self.user_bin {
            if fs::exists(&user_bin)? {
                pt.set_user_bin(&fs::read(&user_bin)?);
            }
        }

        for item in self.items {
            pt.add_record(item.try_into()?)
        }
        Ok(pt)
    }
}

impl Default for PartitionTableCfg {
    /// Returns the default configuration for a `PartitionTableCfg` instance.
    ///
    /// The default values are:
    /// - `rma_w_state`: `0xFF`
    /// - `rma_ov_state`: `0xFF`
    /// - `eFWV`: `0`
    /// - `fw1_idx`: `0`
    /// - `fw2_idx`: `0`
    /// - `ota_trap`: `None`
    /// - `mp_trap`: `None`
    /// - `key_exp_op`: `KeyExportOp::None`
    /// - `user_ext`: `None`
    /// - `user_bin`: `None`
    /// - `items`: An empty vector of `PartitionItemCfg` items.
    fn default() -> Self {
        Self {
            rma_w_state: 0xFF,
            rma_ov_state: 0xFF,
            eFWV: 0,
            fw1_idx: 0,
            fw2_idx: 0,
            ota_trap: None,
            mp_trap: None,
            key_exp_op: KeyExportOp::None,
            user_ext: None,
            user_bin: None,
            items: Vec::new(),
        }
    }
}
