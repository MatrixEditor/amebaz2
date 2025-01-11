use colored::{Color, Colorize};
use openssl::memcmp::eq;
use std::{io::Seek, path::PathBuf};

use crate::cli::{debug, util, Cli};
use amebazii::{
    keys::{HASH_KEY, KEY_PAIR_000, KEY_PAIR_001, KEY_PAIR_003},
    types::{from_stream, EncryptedOr, Flash, Partition, PartitionTableImage, PartitionType},
};

pub fn parse(cli: &Cli, file: PathBuf, pt_only: bool) -> Result<(), amebazii::error::Error> {
    if let Ok(mut fp) = util::open_file(cli, file.clone(), None) {
        if pt_only {
            fp.seek(std::io::SeekFrom::Start(32))?;
            let pt_image: PartitionTableImage = from_stream(&mut fp)?;
            dump_partition_table(&pt_image, &mut fp)?;
        } else {
            let flash: Flash = from_stream(&mut fp)?;

            if cli.verbose > 2 {
                debug!(cli, "Finished parsing file: {}", file.display());
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
    }

    Ok(())
}

fn dump_partition_table(
    pt_image: &PartitionTableImage,
    fp: &mut std::fs::File,
) -> Result<(), amebazii::error::Error> {
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
    if eq(enc_pubkey, KEY_PAIR_000.get_pub_key()) {
        println!(
            "        {}",
            "Note: this partition table uses the default encryption key"
                .yellow()
                .italic()
        );
    }
    println!("  [1] - {:?}", hex::encode(hash_pubkey));
    if eq(hash_pubkey, KEY_PAIR_001.get_pub_key()) {
        println!(
            "        {}",
            "Note: this partition table uses the default hash key"
                .yellow()
                .italic()
        );
    }

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

    if let EncryptedOr::Plain(pt) = &pt_image.pt {
        println!("\n{}: ", "User Data".bold());
        println!("  - {}: {:?} ", "UserExt", hex::encode(pt.get_user_ext()));

        print!("  - {}: ", "UserBin");
        let user_bin = pt.get_user_bin();
        if user_bin.len() > 0 {
            println!("{}, length={}", "valid".color(Color::Green), user_bin.len());
        } else {
            println!("{}", "<not set>".color(Color::Yellow).italic());
        }

        println!("  - Fw1 Index: {}", pt.fw1_idx);
        println!("  - Fw2 Index: {}", pt.fw2_idx);

        println!("\n{}: ", "Records".bold());
        let records = pt.get_records();
        for (i, record) in records.iter().enumerate() {
            println!(
                "  [{}] - Type: {:?} (offset: 0x{:06x}, length: 0x{:06x})",
                i, record.part_type, record.start_addr, record.length
            );
            print!("      - HashKey: ");
            if let Some(key) = record.get_hash_key() {
                println!("{:?}", hex::encode(key));
                if eq(key, KEY_PAIR_003.get_priv_key()) {
                    println!(
                        "        {}",
                        "Note: this partition uses a default hash key"
                            .yellow()
                            .italic()
                    );
                }
            } else {
                println!("{}", "<not set>".italic().red());
            }
            println!()
        }
    } else {
        println!("\n{}: {}", "SegmentData".bold(), "encrypted".red().italic());
    }

    println!("{}\n", "=".repeat(91));

    Ok(())
}
