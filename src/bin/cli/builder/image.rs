use std::{fs, io::Read, path::PathBuf, str::FromStr};

use amebazii::{
    conf::image::{FSTCfg, ImageCfg},
    types::{transfer_to, BootImage, HashAlgo, ToStream},
};
use colored::Colorize;
use object::{
    elf,
    read::elf::{ElfFile, ElfFile32, ElfFile64, FileHeader},
    Endianness, ReadRef,
};

use crate::cli::{debug, error, util, Cli};

use super::{BuildImageOptions, OutputOptions};

pub fn build_image(cli: &Cli, options: &BuildImageOptions) -> Result<(), amebazii::error::Error> {
    if let Some(default_config_file) = &options.gen_defaults.generate_config {
        generate_config(cli, default_config_file)
    } else {
        let config = config_from_options(cli, options)?;
        if options.boot {
            build_boot_image(cli, options, &config)
        } else {
            build_ota_image(cli, options, &config)
        }
    }
}

fn generate_config(cli: &Cli, config_file: &PathBuf) -> Result<(), amebazii::error::Error> {
    let mut cfgout = std::fs::File::create(config_file)?;
    let config = ImageCfg::default();
    serde_json::to_writer_pretty(&mut cfgout, &config)?;
    Ok(())
}

fn config_from_options(
    cli: &Cli,
    options: &BuildImageOptions,
) -> Result<ImageCfg, amebazii::error::Error> {
    let mut config: ImageCfg;
    if let Some(config_file) = &options.config.file {
        let cfgin = util::open_file(cli, config_file.clone(), Some("Config File"));
        if cfgin.is_err() {
            return Err(amebazii::error::Error::InvalidState("".to_string()));
        }
        let mut cfgin = cfgin.unwrap();
        config = serde_json::from_reader(&mut cfgin)?;
    } else {
        config = ImageCfg::default();
    }

    if let Some(serial) = options.serial {
        config.imghdr.serial = serial;
    }

    if let Some(hash_algo) = &options.hash_algo {
        let hash_algo = HashAlgo::from_str(&hash_algo)?;
        if let Some(fst) = &mut config.fstcfg {
            fst.hash_algo = Some(hash_algo);
        } else {
            config.fstcfg = Some(FSTCfg {
                hash_algo: Some(hash_algo),
                ..FSTCfg::default()
            });
        }
    }

    Ok(config)
}

fn build_boot_image(
    cli: &Cli,
    options: &BuildImageOptions,
    config: &ImageCfg,
) -> Result<(), amebazii::error::Error> {
    if let Some(source) = &options.source {
        // revisit: we don't want to use unsafe here but we need a reader that can be used to parse the ELF
        let elf_file = util::open_file(cli, source.clone(), Some("ELF"));
        if elf_file.is_err() {
            return Err(amebazii::error::Error::InvalidState("".to_string()));
        }
        let mut elf_file = elf_file.unwrap();
        let mut data = Vec::with_capacity(elf_file.metadata().unwrap().len() as usize);
        elf_file.read_to_end(&mut data)?;

        let in_elf = elf::FileHeader32::<Endianness>::parse(data.as_slice());
        if in_elf.is_err() {
            error!("Failed to parse ELF file: {}", source.display());
            return Ok(());
        }
        debug!(
            cli,
            "Parsed ELF file: {:#?} successfully!",
            source.display()
        );
        build_boot_image_impl(cli, options, &in_elf, config)?;
    } else {
        error!("{}", "No source file specified.");
        return Ok(());
    }
    Ok(())
}

fn build_boot_image_impl<Elf>(
    cli: &Cli,
    options: &BuildImageOptions,
    source: &Elf,
    config: &ImageCfg,
) -> Result<(), amebazii::error::Error> {
    let mut image = BootImage::default();

    if config.secfg.is_empty() {
        error!(
            "{}",
            "Please specify at least one section (SRAM) in config file."
        );
    }

    let section = config.secfg.first().unwrap();
    image.entry.entry_address = section.entry_addr;
    image.entry.load_address = section.load_addr;

    image.header = config.imghdr.clone().try_into()?;

    Ok(())
}

fn build_ota_image(
    cli: &Cli,
    options: &BuildImageOptions,
    config: &ImageCfg,
) -> Result<(), amebazii::error::Error> {
    Ok(())
}

fn save_image<T>(
    cli: &Cli,
    image: &T,
    options: &OutputOptions,
) -> Result<(), amebazii::error::Error>
where
    T: ToStream,
{
    if let Some(outfile_path) = &options.file {
        if fs::exists(outfile_path)? && !options.force {
            error!(
                "Output file already exists. Use {} to overwrite.",
                "--force".bold()
            );
        }
        debug!(cli, "Saving image to: {:#?}", outfile_path.display());
        let mut outfp = std::fs::File::create(outfile_path.clone())?;
        transfer_to(image, &mut outfp)?;
    }

    Ok(())
}
