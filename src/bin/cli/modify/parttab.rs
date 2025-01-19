use colored::Colorize;
use std::{
    fs,
    io::{Read, Seek, Write},
};

use amebazii::{
    keys::{FLASH_PATTERN, HASH_KEY},
    types::{
        from_stream, key_from_hex, set_default_segment_size, set_default_signature, transfer_to,
        PartTab, PartitionTableImage, Record,
    },
    util::write_fill,
    write_padding,
};

use crate::cli::{debug, util, Cli};

use super::ModParttabOptions;

pub fn modify_parttab(
    cli: &Cli,
    options: &ModParttabOptions,
) -> Result<(), amebazii::error::Error> {
    if let Some(file_path) = &options.input.file {
        let infile = util::open_file(cli, file_path.clone(), Some("Partition Table"));
        if infile.is_err() {
            return Ok(());
        }

        let mut infile = infile.unwrap();
        if options.has_calibration {
            infile.seek(std::io::SeekFrom::Start(0x20))?;
        }

        let mut pt: PartitionTableImage = from_stream(&mut infile)?;

        println!("{}", "Modified Partition Table:".bold());
        modify_parttab_info(options, &mut pt)?;
        if let Some(part_type) = &options.add {
            add_record(cli, options, &mut pt, part_type)?;
        }
        if let Some(part_type) = &options.alter {
            modify_record(cli, options, &mut pt, part_type)?;
        }
        if let Some(part_type) = &options.remove {
            rem_record(&mut pt, part_type);
        }
        save_parttab(cli, options, &mut pt)?;
    }
    Ok(())
}

fn save_parttab(
    cli: &Cli,
    options: &ModParttabOptions,
    image: &mut PartitionTableImage,
) -> Result<(), amebazii::error::Error> {
    let mut outfile = util::open_output_file(cli, Some(&options.input), &options.output)?;
    if options.add_calibration {
        outfile.write_all(FLASH_PATTERN)?;
        write_padding!(&mut outfile, 0x10);
    }

    set_default_segment_size(image);
    set_default_signature(image, Some(HASH_KEY))?;
    transfer_to(image, &mut outfile)?;
    Ok(())
}

macro_rules! update_pt_info {
    ($pt:expr, $option:expr, $name:literal) => {
        if let Some(value) = $option {
            println!("  - {}: {} -> {}", $name.italic(), $pt, value);
            $pt = value;
        }
    };
}

fn modify_parttab_info(
    options: &ModParttabOptions,
    image: &mut PartitionTableImage,
) -> Result<(), amebazii::error::Error> {
    let info: &mut PartTab = image.pt.as_mut();
    update_pt_info!(info.rma_ov_state, options.rma_ovstate, "RMA OV State");
    update_pt_info!(info.rma_w_state, options.rma_wstate, "RMA W State");
    update_pt_info!(info.fw1_idx, options.fw1_idx, "FW1 Index");
    update_pt_info!(info.fw2_idx, options.fw2_idx, "FW2 Index");
    update_pt_info!(info.eFWV, options.efwv, "eFWV");
    Ok(())
}

fn record_set_hash_key(
    record: &mut Record,
    options: &ModParttabOptions,
    cli: &Cli,
) -> Result<(), amebazii::error::Error> {
    if let Some(key_or_file) = &options.hash_key {
        if fs::exists(key_or_file)? {
            debug!(cli, "Reading hash key from file: {}", key_or_file);
            let mut key = [0u8; 32];
            let mut key_file = fs::File::open(key_or_file.clone())?;
            key_file.read_exact(&mut key)?;
            record.set_hash_key(Some(key));
        } else {
            record.set_hash_key(key_from_hex(key_or_file));
        }
    }
    if let Some(key) = record.get_hash_key() {
        println!("{:>14}: {}", "└─ Hash Key", hex::encode(key));
    }
    Ok(())
}

fn cli_part_type_to_record_type(
    part_type: &super::PartitionType,
) -> amebazii::types::PartitionType {
    match part_type {
        super::PartitionType::Boot => amebazii::types::PartitionType::Boot,
        super::PartitionType::Fw1 => amebazii::types::PartitionType::Fw1,
        super::PartitionType::Fw2 => amebazii::types::PartitionType::Fw2,
        super::PartitionType::User => amebazii::types::PartitionType::User,
        super::PartitionType::Sys => amebazii::types::PartitionType::Sys,
    }
}

fn rem_record(image: &mut PartitionTableImage, part_type: &super::PartitionType) {
    let info: &mut PartTab = image.pt.as_mut();
    let record_type = cli_part_type_to_record_type(part_type);
    println!("  - Removing record: {:?}", record_type);

    info.rem_record(record_type);
}

fn add_record(
    cli: &Cli,
    options: &ModParttabOptions,
    image: &mut PartitionTableImage,
    part_type: &super::PartitionType,
) -> Result<(), amebazii::error::Error> {
    let info: &mut PartTab = image.pt.as_mut();
    let mut record = Record::default();

    record.part_type = cli_part_type_to_record_type(part_type);
    if info.has_record(record.part_type) {
        return Err(amebazii::error::Error::InvalidState(format!(
            "Record type {:?} already exists in partition table",
            record.part_type
        )));
    }
    println!("  [{}] +{:?}", info.get_records().len(), record.part_type);

    if let Some(start_addr) = options.start {
        record.start_addr = start_addr;
    } else {
        return Err(amebazii::error::Error::InvalidState(
            "Start address not specified".to_string(),
        ));
    }

    if let Some(length) = options.length {
        record.length = length;
    } else {
        return Err(amebazii::error::Error::InvalidState(
            "Length not specified".to_string(),
        ));
    }
    println!(
        "{:>12}: start=0x{:08x}, length=0x{:08x}",
        "├─ Bounds", record.start_addr, record.length
    );

    record.dbg_skip = options.debug_skip;
    println!(
        "{:>16}: {}",
        "├─ Debug Skip",
        if record.dbg_skip {
            "yes".yellow()
        } else {
            "no".green()
        }
    );

    record_set_hash_key(&mut record, options, cli)?;
    info.add_record(record);
    Ok(())
}

fn modify_record(
    cli: &Cli,
    options: &ModParttabOptions,
    image: &mut PartitionTableImage,
    part_type: &super::PartitionType,
) -> Result<(), amebazii::error::Error> {
    let record_type = cli_part_type_to_record_type(part_type);
    let info: &mut PartTab = image.pt.as_mut();
    let record = info.get_record_mut(record_type).ok_or_else(|| {
        amebazii::error::Error::InvalidState(format!(
            "Record type {:?} not found in partition table",
            record_type
        ))
    })?;

    println!("  [/] {:?}", record.part_type);

    if options.debug_skip != record.dbg_skip {
        println!(
            "{:>16}: {} -> {}",
            "├─ Debug Skip",
            if record.dbg_skip {
                "yes".yellow()
            } else {
                "no".green()
            },
            if options.debug_skip {
                "yes".yellow()
            } else {
                "no".green()
            }
        );
        record.dbg_skip = options.debug_skip;
    }

    print!("{:>12}: start=0x{:08x}", "├─ Bounds", record.start_addr);
    if let Some(start_addr) = options.start {
        if start_addr != record.start_addr {
            print!(" -> 0x{:08x}", start_addr);
            record.start_addr = start_addr;
        }
    }

    print!(", length=0x{:08x}", record.length);
    if let Some(length) = options.length {
        if length != record.length {
            print!(" -> 0x{:08x}", length);
            record.length = length;
        }
    }
    println!();

    record_set_hash_key(record, options, cli)?;

    Ok(())
}
