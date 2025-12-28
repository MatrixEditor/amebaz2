use amebazii::types::{AsImage, key_from_hex};
use colored::Colorize;
use openssl::memcmp::eq;
use std::io::Read;
use std::{fs, io::Write};

use amebazii::{
    is_valid_data,
    keys::HASH_KEY,
    read_valid_data,
    types::{
        from_stream, set_default_signature, DataType, EncryptedOr, HashAlgo, OTAImage, ToStream,
    },
};

use crate::cli::{util, Cli};

use super::ReSignOptions;

pub fn re_sign(cli: &Cli, options: &ReSignOptions) -> Result<(), amebazii::error::Error> {
    if let Some(input_file) = &options.input.file {
        let input = util::open_file(cli, input_file.clone(), None);
        if input.is_err() {
            return Ok(());
        }

        let mut input = input.unwrap();
        let mut ota: OTAImage = from_stream(&mut input)?;

        let mut hash_key: DataType<32> = None;
        if let Some(key) = &options.key {
            if fs::exists(key)? {
                let mut r = fs::File::open(key)?;
                read_valid_data!(hash_key, 32, &mut r);
            } else {
                hash_key = key_from_hex(key);
            }
        }

        let hash_key = match hash_key {
            Option::Some(hash_key) => hash_key,
            Option::None => HASH_KEY.clone(),
        };
        let algo = if options.use_md5 {
            HashAlgo::Md5
        } else {
            HashAlgo::Sha256
        };

        let first_signature = ota.build_first_signature(Some(&hash_key))?;
        for (idx, subimage) in ota.get_subimages_mut().iter_mut().enumerate() {
            if let EncryptedOr::Plain(fst) = &mut subimage.fst {
                if !options.same_algo {
                    fst.hash_algo = Some(algo);
                }
            }
            let old_signature = subimage.get_hash().to_vec();
            let new_signature;
            if idx == 0 {
                // first subimage differs
                new_signature = first_signature.clone();
                subimage.set_signature(&first_signature);
            } else {
                set_default_signature(subimage, Some(&hash_key))?;
                new_signature = subimage.get_hash().to_vec();
            }

            if old_signature != new_signature {
                println!("[{}] Subimage: {:?}", idx, subimage.header.img_type);
                println!(" - Old signature: {}", hex::encode(old_signature));
                println!(" - New signature: {}\n", hex::encode(new_signature));
            }
        }

        let new_ota_signature = ota.build_ota_signature(Some(&hash_key))?;
        let old_ota_signature = ota.get_ota_signature();

        if !eq(&new_ota_signature, old_ota_signature) {
            println!(
                "[{}] Old signature: {}",
                "OTA".bold(),
                hex::encode(old_ota_signature)
            );
            println!(
                "[{}] New signature: {}",
                "OTA".bold(),
                hex::encode(&new_ota_signature)
            );
        }
        ota.set_ota_signature(&new_ota_signature);

        let output = util::open_output_file(cli, Some(&options.input), &options.output)?;
        ota.update_checksum()?;
        let mut writer = std::io::BufWriter::new(output);
        ota.write_to(&mut writer)?;
        writer.flush()?;
    }

    Ok(())
}
