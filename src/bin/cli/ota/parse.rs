use colored::{Color, Colorize};
use openssl::memcmp::eq;
use std::io::{Read, Seek};
use std::path::PathBuf;

use crate::cli::{debug, util, Cli};
use amebazii::{
    keys::KEY_PAIR_003,
    types::{from_stream, BinarySize, EncryptedOr, ImageHeader, OTAImage, SubImage},
};

#[allow(unused_variables)]
pub fn parse(cli: &Cli, file: PathBuf) -> Result<(), amebazii::error::Error> {
    let file_reader = util::open_file(cli, file.clone(), None);
    if file_reader.is_err() {
        return Ok(());
    }

    let mut fp = file_reader.unwrap();
    let image: OTAImage = from_stream(&mut fp)?;

    if cli.verbose > 2 {
        debug!(cli, "Finished parsing file: {}", file.display());
    }

    dump_ota_image(&image, &mut fp)?;
    Ok(())
}

fn dump_ota_image(
    ota_image: &OTAImage,
    fp: &mut std::fs::File,
) -> Result<(), amebazii::error::Error> {
    println!(
        "{} {} {}",
        "=".repeat(44),
        "OTA Image".bold(),
        "=".repeat(45)
    );

    println!("{}:", "Public Keys".bold());
    println!(
        "  [Hash Public Key] - {:?}",
        hex::encode(ota_image.keyblock.get_hash_pubkey())
    );
    for i in 0..5 {
        print!("  [{}] -  ", i);
        if let Some(key) = ota_image.get_public_key(i) {
            println!("{:?}", hex::encode(key));
        } else {
            println!("{}", "<not set>".italic().yellow());
        }
    }

    println!(
        "\n{}: ({})",
        "OTA-Signature".bold(),
        "using default hash key".italic()
    );

    let first_subimage = ota_image.get_subimage(0).unwrap();
    if let EncryptedOr::Plain(fst) = &first_subimage.fst {
        // signature starts at 224
        if let Some(algo) = &fst.hash_algo {
            fp.seek(std::io::SeekFrom::Start(224))?;
            let signature =
                OTAImage::ota_signature_from_stream(fp, *algo, Some(KEY_PAIR_003.get_priv_key()))?;
            let image_signature = ota_image.get_ota_signature();
            print!("  - {:?} ", hex::encode(image_signature));
            if eq(&signature, image_signature) {
                println!("{}", "OK".green());
            } else {
                println!("{}", "invalid/encrypted/wrong key".red().italic());
            }
        }
    }

    if let Some(ota_checksum) = ota_image.checksum {
        fp.seek(std::io::SeekFrom::Start(0))?;
        let checksum = OTAImage::checksum_from_stream(fp)?;
        let suffix = if checksum == ota_checksum {
            "OK".green()
        } else {
            "invalid".red().italic()
        };
        println!(
            "  - {}: 0x{:06x} {}",
            "Checksum".italic(),
            ota_checksum,
            suffix
        );
    }

    println!("{}\n", "=".repeat(100));
    let mut offset = 224;
    for subimage in ota_image.get_subimages() {
        println!(
            "{} {} {}",
            "-".repeat(45),
            "Subimage".bold(),
            "-".repeat(45)
        );

        dump_subimage(subimage, fp, offset)?;
        println!("{}\n", "-".repeat(100));

        if let Some(next_offset) = subimage.header.next_offset {
            offset += next_offset as u64;
        }
    }

    // -- subimages --

    Ok(())
}

fn dump_subimage(
    subimage: &SubImage,
    fp: &mut std::fs::File,
    offset: u64,
) -> Result<(), amebazii::error::Error> {
    print!("{}: ", "Header".bold());
    if subimage.header.is_encrypt {
        println!("{}", "encrypted".color(Color::Red).italic());
    } else {
        println!();
    }

    println!("  - Type: {:?}", subimage.header.img_type);
    println!("  - Size: 0x{:08x}", subimage.header.segment_size);
    println!("  - Serial: {}", subimage.header.serial);

    println!("\n{}: ", "User Keys".bold());
    if let Some(key1) = subimage.header.get_user_key1() {
        println!("  [0] - {:?}", hex::encode(key1));
    } else {
        println!("  [0] - {}", "<not set>".italic().yellow());
    }

    if let Some(key2) = subimage.header.get_user_key2() {
        println!("  [1] - {:?}", hex::encode(key2));
    } else {
        println!("  [0] - {}", "<not set>".italic().yellow());
    }

    println!("\n{}:", "Security".bold());
    print!("  - Encryption: ");
    if let EncryptedOr::Plain(fst) = &subimage.fst {
        println!("{}", "disabled".yellow().italic());
        print!("  - Hashing: ");
        if let Some(hash_algo) = &fst.hash_algo {
            println!("{} ({:?})", "enabled".color(Color::Green), hash_algo);
            // REVISIT: this does not cover the first signature
            let subimage_hash = subimage.get_hash();
            print!("    - {:?} ", hex::encode(subimage_hash));

            let hash;
            if offset == 224 {
                fp.seek(std::io::SeekFrom::Start(0))?;
                let mut buffer =
                    vec![
                        0x00;
                        ImageHeader::binary_size() + subimage.header.segment_size as usize + 224
                    ];
                fp.read_exact(&mut buffer)?;
                hash = hash_algo.compute_hash(&buffer, Some(KEY_PAIR_003.get_priv_key()))?
            } else {
                fp.seek(std::io::SeekFrom::Start(offset))?;
                hash = subimage.signature_from_stream(
                    fp,
                    *hash_algo,
                    Some(KEY_PAIR_003.get_priv_key()),
                )?;
            }

            if eq(&hash, subimage_hash) {
                println!("{}", "OK".green());
            } else {
                println!("{}", "invalid/encrypted/wrong key".red().italic());
            }
        }
    } else {
        println!("{}", "enabled".color(Color::Green));
    }

    println!("\n{}:", "Sections".bold());
    let sections = subimage.get_sections();
    for i in 0..sections.len() {
        let section = &sections[i];
        println!(
            "  [{}] - {:?} (length: 0x{:08x}, load: 0x{:08x}, entry: 0x{:08x})",
            i,
            section.header.sect_type,
            section.header.length,
            section.entry_header.load_address,
            section.entry_header.entry_address.unwrap_or(0xFFFF_FFFF)
        );
    }
    Ok(())
}
