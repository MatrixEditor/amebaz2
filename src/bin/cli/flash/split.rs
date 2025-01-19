use colored::Colorize;
use std::{
    fs,
    io::{self, Read, Seek},
    path::PathBuf,
};

use crate::cli::{debug, error, util, Cli};

use amebazii::types::{from_stream, EncryptedOr, PartitionTableImage, PartitionType, Record};

pub fn split_flash(cli: &Cli, options: &super::SplitOptions) -> Result<(), amebazii::error::Error> {
    if let Some(input_file) = &options.input.file {
        let input = util::open_file(cli, input_file.clone(), None);
        if input.is_err() {
            return Ok(());
        }

        let mut input = input.unwrap();
        input.seek(io::SeekFrom::Start(32))?;
        let pt_image: PartitionTableImage = from_stream(&mut input)?;

        if let Some(outdir) = &options.outdir {
            if !outdir.is_dir() {
                debug!(cli, "Creating directory {}", outdir.display());
                std::fs::create_dir_all(&outdir)?;
            }

            if let EncryptedOr::Plain(pt) = pt_image.pt {
                write_pt(cli, &mut input, &outdir)?;

                let file_size = input.metadata().unwrap().len() as u32;
                for (i, record) in pt.get_records().iter().enumerate() {
                    println!("[{}] {} ({:?})", i, "Record".bold(), record.part_type);
                    if record.start_addr >= file_size {
                        println!(
                            "  - {}",
                            format!(
                                "Start address 0x{:06x} is larger than file size 0x{:06x} - skipping...",
                                record.start_addr, file_size
                            )
                            .yellow()
                        );
                        continue;
                    }

                    println!(
                        "  - Offset: 0x{:06x}, Length: 0x{:06x})",
                        record.start_addr, record.length
                    );
                    write_record(cli, &record, &i, &mut input, &outdir)?;
                }
            } else {
                error!(
                    "{}",
                    "Partition table is encrypted (decryption not supported)"
                );
            }

            if options.include_common {
                write_common(cli, &mut input, &outdir)?;
            }
        }
    }

    Ok(())
}

fn write_pt(
    cli: &Cli,
    fp: &mut std::fs::File,
    outdir: &PathBuf,
) -> Result<(), amebazii::error::Error> {
    let pt_file_path = outdir.join("partition.bin");
    debug!(
        cli,
        "Writing partition table to {:?}",
        pt_file_path.display()
    );

    let mut writer = fs::File::create(outdir.join("partition.bin"))?;
    let size = fp.stream_position()?;

    fp.seek(std::io::SeekFrom::Start(0))?;
    let mut reader = fp.take(size);

    io::copy(&mut reader, &mut writer)?;
    Ok(())
}

fn write_record(
    cli: &Cli,
    record: &Record,
    index: &usize,
    fp: &mut std::fs::File,
    outdir: &PathBuf,
) -> Result<(), amebazii::error::Error> {
    fp.seek(io::SeekFrom::Start(record.start_addr as u64))?;
    let mut reader = fp.take(record.length as u64);

    let file_name = match record.part_type {
        PartitionType::Boot => "boot.bin".to_string(),
        PartitionType::Fw1 => "fw1.bin".to_string(),
        PartitionType::Fw2 => "fw2.bin".to_string(),
        PartitionType::PartTab => "partition.bin".to_string(),
        PartitionType::Cal => "calibration.bin".to_string(),
        PartitionType::User => "user.bin".to_string(),
        PartitionType::MP => "mp.bin".to_string(),
        PartitionType::Rdp => "reserved.bin".to_string(),
        PartitionType::Var => "var.bin".to_string(),
        _ => format!("record_{}.bin", index),
    };

    let file_path = outdir.join(file_name);
    debug!(cli, "Writing record to {}", file_path.display());
    let mut writer = fs::File::create(file_path)?;
    io::copy(&mut reader, &mut writer)?;
    Ok(())
}

fn write_common(
    cli: &Cli,
    fp: &mut std::fs::File,
    outdir: &PathBuf,
) -> Result<(), amebazii::error::Error> {

    // we assume system is at 0x1000
    fp.seek(io::SeekFrom::Start(0x1000))?;
    let mut reader = fp.take(0x1000);

    let file_path = outdir.join("sysdata.bin");
    debug!(cli, "Writing system data to {}", file_path.display());
    let mut writer = fs::File::create(file_path)?;
    io::copy(&mut reader, &mut writer)?;

    // REVISIT: include calibration and reserved regions?

    Ok(())
}
