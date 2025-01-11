use colored::{Color, Colorize};
use std::{fs, io, io::Read, io::Seek, path::PathBuf};

use amebazii::{
    error::Error,
    keys::FLASH_PATTERN,
    read_padding,
    types::{
        from_stream, transfer_to, Flash, FromStream, Partition, PartitionTableImage, PartitionType,
        SystemData,
    },
};

use crate::cli::{debug, error, util, Cli};

use super::CombineOptions;

pub fn combine_images(cli: &Cli, options: &CombineOptions) -> Result<(), amebazii::error::Error> {
    let ptfile = util::open_file(
        cli,
        options.parttab.clone().unwrap(),
        Some("Partition Table"),
    );
    if ptfile.is_err() {
        return Ok(());
    }

    let mut flash = Flash::default();
    let mut ptfp = ptfile.unwrap();

    if options.pt_has_calibpat {
        ptfp.read_exact(flash.get_calibration_pattern_mut())?;
        read_padding!(ptfp, 16);
    } else {
        flash
            .get_calibration_pattern_mut()
            .copy_from_slice(FLASH_PATTERN);
    }

    let ptimage: PartitionTableImage = from_stream(&mut ptfp)?;
    // setup partition table
    flash.set_partition(PartitionType::PartTab, Partition::PartitionTable(ptimage));

    if let Some(src_dir) = &options.srcdir {
        load_from_dir(cli, &mut flash, src_dir)?;
    }

    // todo: support encrypted partition tables
    if let Some(sys_data_file) = &options.system {
        set_system_partition(cli, &mut flash, sys_data_file)?;
    }

    if let Some(fw1_file) = &options.fw1 {
        set_fw1_partition(cli, &mut flash, fw1_file)?;
    }

    if let Some(fw2_file) = &options.fw2 {
        set_fw2_partition(cli, &mut flash, fw2_file)?;
    }

    if let Some(user_file) = &options.user {
        set_user_partition(cli, &mut flash, user_file)?;
    }

    if let Some(boot_file) = &options.boot {
        set_boot_partition(cli, &mut flash, boot_file)?;
    }

    // make sure the system partition is always present
    if !flash.has_partition(PartitionType::Sys) {
        debug!(cli, "No system data provided, using default");
        flash.set_system_partition(SystemData::default());
    }

    // sanity checks
    if !flash.has_partition(PartitionType::Boot) {
        error!(
            "{}",
            "Boot partition not found! This partition is required to build a valid"
        );
        error!(
            "{}",
            "flash image. Please provide a boot image with the --boot option."
        );
        return Ok(());
    }

    let out_path = options.file.clone().unwrap();
    if out_path.exists() && options.no_overwrite {
        error!("{} already exists", out_path.display());
        return Ok(());
    }

    if out_path.is_dir() {
        error!("{} is a directory", out_path.display());
        return Ok(());
    }

    let mut out_file = fs::File::create(out_path.clone())?;
    transfer_to(&flash, &mut out_file)?;

    Ok(())
}

fn read_image<T: FromStream + Default>(
    cli: &Cli,
    file: &PathBuf,
    file_type: Option<&str>,
) -> Result<T, amebazii::error::Error> {
    let file = util::open_file(cli, file.clone(), file_type);
    if file.is_err() {
        return Err(amebazii::error::Error::InvalidState(
            "File not found".to_string(),
        ));
    }
    let mut img = file.unwrap();
    debug!(
        cli,
        "Parsing {} with {} bytes",
        file_type.unwrap(),
        img.metadata().unwrap().len()
    );
    let image: T = match from_stream(&mut img) {
        Ok(i) => i,
        Err(e) => {
            error!("--- Failed to parse {} ---", file_type.unwrap());
            return Err(e);
        }
    };
    debug!(
        cli,
        "Successfully parsed {}",
        file_type.or(Some("image")).unwrap()
    );
    Ok(image)
}

fn read_raw_image(
    cli: &Cli,
    file: &PathBuf,
    file_type: Option<&str>,
) -> Result<Vec<u8>, amebazii::error::Error> {
    let file = util::open_file(cli, file.clone(), file_type);
    if file.is_err() {
        return Err(amebazii::error::Error::InvalidState(
            "File not found".to_string(),
        ));
    }
    let mut img = file.unwrap();
    let mut image = Vec::with_capacity(img.metadata().unwrap().len() as usize);
    img.read_to_end(&mut image)?;
    Ok(image)
}

fn set_partition<T, F>(
    cli: &Cli,
    flash: &mut Flash,
    path: &PathBuf,
    part_type: PartitionType,
    fn_create_part: F,
    file_type: &str,
) -> Result<(), amebazii::error::Error>
where
    T: FromStream + Default,
    F: FnOnce(T) -> Partition,
{
    let image: T = read_image(cli, path, Some(file_type))?;
    print!("[{part_type:?}] => Status: ");
    flash.set_partition(part_type, fn_create_part(image));
    println!("{}", "Ok".green());
    Ok(())
}

fn set_system_partition(cli: &Cli, flash: &mut Flash, path: &PathBuf) -> Result<(), Error> {
    set_partition(
        cli,
        flash,
        path,
        PartitionType::Sys,
        |i| Partition::System(i),
        "System Data",
    )
}

fn set_boot_partition(cli: &Cli, flash: &mut Flash, path: &PathBuf) -> Result<(), Error> {
    set_partition(
        cli,
        flash,
        path,
        PartitionType::Boot,
        |i| Partition::Bootloader(i),
        "Boot Image",
    )
}

fn set_fw1_partition(cli: &Cli, flash: &mut Flash, path: &PathBuf) -> Result<(), Error> {
    set_partition(
        cli,
        flash,
        path,
        PartitionType::Fw1,
        |i| Partition::Fw1(i),
        "Firmware 1",
    )
}

fn set_fw2_partition(cli: &Cli, flash: &mut Flash, path: &PathBuf) -> Result<(), Error> {
    set_partition(
        cli,
        flash,
        path,
        PartitionType::Fw2,
        |i| Partition::Fw2(i),
        "Firmware 2",
    )
}

fn set_user_partition(cli: &Cli, flash: &mut Flash, path: &PathBuf) -> Result<(), Error> {
    set_raw_partition(
        cli,
        flash,
        path,
        PartitionType::User,
        |i| Partition::User(i),
        "User Data",
    )
}

fn set_raw_partition<F>(
    cli: &Cli,
    flash: &mut Flash,
    path: &PathBuf,
    part_type: PartitionType,
    fn_create_part: F,
    file_type: &str,
) -> Result<(), amebazii::error::Error>
where
    F: FnOnce(Vec<u8>) -> Partition,
{
    let image = read_raw_image(cli, path, Some(file_type))?;
    print!("[{part_type:?}] => Status: ");
    flash.set_partition(part_type, fn_create_part(image));
    println!("{}", "Ok".green());
    Ok(())
}

fn load_from_dir(
    cli: &Cli,
    flash: &mut Flash,
    src_dir: &PathBuf,
) -> Result<(), amebazii::error::Error> {
    debug!(cli, "Loading from directory {}", src_dir.display());
    for entry in std::fs::read_dir(src_dir)? {
        let path = entry?.path();

        if let Some(file_type) = path.file_name() {
            match file_type.to_str() {
                Some("sys") => {
                    set_system_partition(cli, flash, &path)?;
                }
                Some("fw1") => {
                    set_fw1_partition(cli, flash, &path)?;
                }
                Some("fw2") => {
                    set_fw2_partition(cli, flash, &path)?;
                }
                Some("user") => {
                    set_user_partition(cli, flash, &path)?;
                }
                Some("boot") => {
                    set_boot_partition(cli, flash, &path)?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
