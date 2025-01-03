use colored::{Color, Colorize};
use openssl::memcmp::eq;
use std::{io::Seek, path::PathBuf};

use crate::{
    cli::{debug, util, Cli},
    read_padding,
    types::{
        enums::PartitionType,
        flash::{Flash, Partition},
        from_stream,
        image::pt::PartitionTableImage,
        HASH_KEY,
    },
};

pub fn parse(cli: &Cli, file: PathBuf) -> Result<(), crate::error::Error> {
    if let Ok(mut fp) = util::open_file(cli, file.clone()) {
        let flash: Flash = from_stream(&mut fp)?;

        if cli.verbose > 2 {
            debug!("Finished parsing file: {}", file.display());
        }

        println!("{} {} {}", "*".repeat(42), "Flash".bold(), "*".repeat(42));

        println!("{}:", "Calibration Pattern".bold());
        println!("  - {:?}\n", hex::encode(&flash.get_calibration_pattern()));

        if let Some(Partition::PartitionTable(partition_table)) =
            flash.get_partition(PartitionType::PartTab)
        {
            dump_partition_table(partition_table, &mut fp)?;
        }
    }

    Ok(())
}

fn dump_partition_table(
    pt_image: &PartitionTableImage,
    fp: &mut std::fs::File,
) -> Result<(), crate::error::Error> {
    println!(
        "{} {} {}",
        "=".repeat(37),
        "Partition Table".bold(),
        "=".repeat(37)
    );

    let enc_pubkey = pt_image.keyblock.get_enc_pubkey();
    let hash_pubkey = pt_image.keyblock.get_hash_pubkey();

    println!("{}:", "Public Keys".bold());
    println!("  [0] - {:?}", hex::encode(enc_pubkey));
    println!("  [1] - {:?}", hex::encode(hash_pubkey));

    println!(
        "\n{}: ({})",
        "Signature".bold(),
        "using default hash key".italic()
    );

    fp.seek(std::io::SeekFrom::Start(32))?;
    let signature = pt_image.create_signature(fp, HASH_KEY)?;
    let pt_hash = pt_image.get_hash();
    print!("  - {:?} ", hex::encode(pt_hash));
    if eq(&signature, pt_hash) {
        println!("{}", "OK".green());
    } else {
        println!("{}", "invalid/encrypted/wrong key".red().italic());
    }

    println!("\n{}: ", "User Data".bold());
    println!(
        "  - {}: {:?} ",
        "UserExt",
        hex::encode(pt_image.pt.get_user_ext())
    );

    print!("  - {}: ", "UserBin");
    let user_bin = pt_image.pt.get_user_bin();
    if user_bin.len() > 0 {
        println!("{}, length={}", "valid".color(Color::Green), user_bin.len());
    } else {
        println!("{}", "<not set>".color(Color::Yellow).italic());
    }

    println!("\n{}: ", "Records".bold());
    let records = pt_image.pt.get_records();
    for (i, record) in records.iter().enumerate() {
        println!(
            "  [{}] - Type: {:?} (offset: 0x{:06x}, 0xlength: {:06x})",
            i, record.part_type, record.start_addr, record.length
        );
        print!("      - HashKey: ");
        if let Some(key) = record.get_hash_key() {
            println!("{:?}", hex::encode(key));
        } else {
            println!("{}", "<not set>".italic().red());
        }
    }
    println!("{}\n", "=".repeat(91));

    Ok(())
}
