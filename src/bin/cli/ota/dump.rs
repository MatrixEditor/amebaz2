use colored::Colorize;
use std::{io::Write, path::PathBuf};

use crate::cli::{debug, error, util, Cli};
use amebazii::types::{from_stream, image::ota::OTAImage, section};

pub fn dump_sections(
    cli: &Cli,
    file: PathBuf,
    img_idx: u32,
    outfile: PathBuf,
    section: Option<u32>,
) -> Result<(), amebazii::error::Error> {
    let fp = util::open_file(cli, file.clone(), None);
    if fp.is_err() {
        return Ok(());
    }

    if section.is_none() && outfile.is_file() {
        error!(
            "Output directory {:?} is a file! Exporting multiple sections requires a directory!",
            outfile.display()
        );
        return Ok(());
    }

    let mut reader = fp.unwrap();
    let image: OTAImage = from_stream(&mut reader)?;
    debug!(cli, "Finished parsing file: {}", file.display());

    let subimages = image.get_subimages();
    if subimages.len() <= img_idx as usize {
        error!(
            "Invalid subimage index: {} (maximum is {})",
            img_idx,
            subimages.len() - 1
        );
        return Ok(());
    }

    let subimage = &subimages[img_idx as usize];
    println!(
        "[{}] {}: {:?}",
        img_idx,
        "Subimage".bold(),
        subimage.header.img_type
    );

    // REVISIT: encryption not supported here
    if subimage.header.is_encrypt {
        error!("{}", "Encryption not supported");
        return Ok(());
    }

    let sections = subimage.get_sections();
    if let Some(section_idx) = section {
        if section_idx >= sections.len() as u32 {
            error!(
                "Invalid section index: {} (maximum is {})",
                section_idx,
                sections.len() - 1
            );
            return Ok(());
        }

        let file_path = if outfile.is_dir() {
            debug!(cli, "Outputting to directory: {}", outfile.display());
            if !outfile.exists() {
                debug!(cli, "Creating directory: {}", outfile.display());
                std::fs::create_dir(&outfile)?;
            }
            outfile.join(format!("{}_section_{}.bin", img_idx, section_idx))
        } else {
            outfile.clone()
        };

        let section = &sections[section_idx as usize];
        print!("    [{}] ", section_idx);

        dump_section(cli, section, &file_path)?;
    } else {
        for (i, section) in sections.iter().enumerate() {
            print!("    [{}] ", i);
            if !outfile.exists() {
                debug!(cli, "Creating directory: {}", outfile.display());
                std::fs::create_dir(&outfile)?;
            }
            let file_path = outfile.join(format!("{}_section_{}.bin", img_idx, i));
            dump_section(cli, section, &file_path)?;
        }
    }
    Ok(())
}

fn dump_section(
    cli: &Cli,
    section: &section::Section,
    outfile: &PathBuf,
) -> Result<(), amebazii::error::Error> {
    println!(
        "{}: {:?} (Length: 0x{:08x}, LoadAddress: 0x{:08x}, EntryAddress: 0x{:08x})",
        "Section".bold(),
        section.header.sect_type,
        section.header.length,
        section.entry_header.load_address,
        section.entry_header.entry_address.unwrap_or(0xFFFF_FFFF)
    );

    let mut writer = std::fs::File::create(outfile)?;
    writer.write_all(section.get_data())?;
    debug!(cli, "Wrote section to file: {}", outfile.display());
    Ok(())
}
