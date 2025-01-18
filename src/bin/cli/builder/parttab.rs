use std::io::{Cursor, Seek, Write};

use colored::Colorize;

use crate::cli::{debug, error, util, Cli};
use amebazii::{
    conf::{DataArray, PartitionItemCfg, PartitionTableCfg},
    keys::{FLASH_PATTERN, HASH_KEY, KEY_PAIR_003},
    types::{
        key_to_hex, set_default_segment_size, set_default_signature, AsImage, EncryptedOr,
        PartitionTableImage, PartitionType, ToStream,
    },
    util::write_fill,
    write_aligned,
};

use super::BuildPartitionTableOptions;

pub fn build_parttab(
    cli: &Cli,
    options: &BuildPartitionTableOptions,
) -> Result<(), amebazii::error::Error> {
    if let Some(default_config_file) = &options.gen_defaults.generate_config {
        debug!(
            cli,
            "Generating default config file: {:#?}", default_config_file
        );
        let mut config = PartitionTableCfg::default();

        if options.generate_default_entries {
            config.items.push(PartitionItemCfg {
                start_addr: 0x4000,
                length: 0x8000,
                part_type: PartitionType::Boot,
                debug_skip: false,
                hash_key: key_to_hex(Some(&[0xFF; 32])).unwrap(),
            });
            config.items.push(PartitionItemCfg {
                start_addr: 0xC000,
                length: 0xF8000,
                part_type: PartitionType::Fw1,
                debug_skip: false,
                hash_key: key_to_hex(Some(KEY_PAIR_003.get_priv_key())).unwrap(),
            });
        }

        let mut cfgout = std::fs::File::create(default_config_file.clone())?;
        serde_json::to_writer_pretty(&mut cfgout, &config)?;
        return Ok(());
    }

    let mut config: PartitionTableCfg;
    if let Some(config_file) = &options.config.file {
        let cfgfp = util::open_file(cli, config_file.clone(), Some("Config"));
        if cfgfp.is_err() {
            return Ok(());
        }
        let mut cfgin = cfgfp.unwrap();
        config = serde_json::from_reader(&mut cfgin)?;
    } else {
        config = PartitionTableCfg::default();
    }

    // add all custom options
    if let Some(efwv) = options.efwv {
        config.eFWV = efwv;
    }
    if let Some(rma_wstate) = options.rma_wstate {
        config.rma_w_state = rma_wstate;
    }
    if let Some(rma_ovstate) = options.rma_ovstate {
        config.rma_ov_state = rma_ovstate;
    }
    if let Some(fw1_idx) = options.fw1_idx {
        config.fw1_idx = fw1_idx;
    }
    if let Some(fw2_idx) = options.fw2_idx {
        config.fw2_idx = fw2_idx;
    }
    if let Some(key_exp_op) = options.key_exp_op {
        config.key_exp_op = key_exp_op.try_into()?;
    }

    if let Some(user_ext) = &options.user_ext {
        config.user_ext = Some(DataArray::new(user_ext.clone())?);
    }
    if let Some(user_bin) = &options.user_bin {
        config.user_bin = Some(DataArray::new(user_bin.clone())?);
    }

    let mut image = PartitionTableImage::default();
    image.pt = EncryptedOr::Plain(config.try_into()?);
    image.header.segment_size = image.build_segment_size();

    set_default_segment_size(&mut image);
    set_default_signature(&mut image, Some(HASH_KEY))?;

    let mut out = Vec::new();
    let mut writer = Cursor::new(&mut out);
    image.write_to(&mut writer).unwrap();
    debug!(cli, "Created partition table!");

    if out.len() > 0x1000 {
        error!(
            "Partition table size is too large! ({} bytes > 4096 bytes)",
            out.len()
        );
        return Ok(());
    }

    if let Some(output_file) = &options.output.file {
        let mut outfp = std::fs::File::create(output_file)?;
        if !options.no_calibpat {
            outfp.write_all(FLASH_PATTERN)?;
            outfp.write_all(&[0xFF; 16])?;
        }
        outfp.write_all(&out)?;

        if options.fill_sector {
            write_aligned!(&mut outfp, 0x1000, 0xFF);
        }
        debug!(
            cli,
            "Partition table written to: {:#?}",
            output_file.display()
        );
    }

    Ok(())
}
