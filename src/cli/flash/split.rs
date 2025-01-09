use colored::{Color, Colorize};
use std::{
    fs,
    io::{self, Read, Seek},
    path::PathBuf,
};

use crate::{
    cli::{debug, error, util, Cli},
    types::{
        enums::PartitionType,
        from_stream,
        image::{
            pt::{PartitionTableImage, Record},
            EncryptedOr,
        },
    },
};

pub fn split_flash(cli: &Cli, file: PathBuf, outdir: PathBuf) -> Result<(), crate::error::Error> {
    let fp = util::open_file(cli, file, None);
    if fp.is_err() {
        return Ok(());
    }

    if !outdir.is_dir() {
        debug!(cli, "Creating directory {}", outdir.display());
        std::fs::create_dir_all(&outdir)?;
    }

    let mut reader = fp.unwrap();
    reader.seek(io::SeekFrom::Start(32))?;
    let pt_image: PartitionTableImage = from_stream(&mut reader)?;

    if let EncryptedOr::Plain(pt) = pt_image.pt {
        write_pt(cli, &mut reader, &outdir)?;

        let file_size = reader.metadata().unwrap().len() as u32;
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
            write_record(cli, &record, &i, &mut reader, &outdir)?;
        }
    } else {
        error!(
            "{}",
            "Partition table is encrypted (decryption not supported)"
        );
    }

    Ok(())
}

fn write_pt(
    cli: &Cli,
    fp: &mut std::fs::File,
    outdir: &PathBuf,
) -> Result<(), crate::error::Error> {
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
) -> Result<(), crate::error::Error> {
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
