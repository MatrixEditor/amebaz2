use std::path::PathBuf;

use clap::Parser;

use super::{headings, Cli, FlashSubCommand};

mod combine;
mod parse;
mod split;

/// Combine partitions to a usable flash image.
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct CombineOptions {
    /// The destination file where combined partitions will be saved.
    #[arg(value_name = "DEST", required = true)]
    pub file: Option<PathBuf>,

    /// Path to the parttab file containing partition information. (partition table)
    #[arg(short, long, value_name = "PT", required = true)]
    pub parttab: Option<PathBuf>,

    /// Directory path from which all partition files are read.
    #[arg(short, long, value_name = "DIR")]
    pub srcdir: Option<PathBuf>,

    /// Path to the first firmware file (fw1).
    #[arg(long, value_name = "FILE", help_heading = headings::PART_OPTIONS)]
    pub fw1: Option<PathBuf>,

    /// Path to the second firmware file (fw2).
    #[arg(long, value_name = "FILE", help_heading = headings::PART_OPTIONS)]
    pub fw2: Option<PathBuf>,

    /// Path to the system-data partition file.
    #[arg(long, value_name = "FILE", help_heading = headings::PART_OPTIONS)]
    pub system: Option<PathBuf>,

    /// Path to the user partition file.
    #[arg(long, value_name = "FILE", help_heading = headings::PART_OPTIONS)]
    pub user: Option<PathBuf>,

    /// Path to the boot partition file.
    #[arg(long, value_name = "FILE", help_heading = headings::PART_OPTIONS)]
    pub boot: Option<PathBuf>,

    // Other partitions are subject to future implementation
    /// Indicating that the calibration patternis within the parttab file.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub pt_has_calibpat: bool,

    /// Do not overwrite destination files
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub no_overwrite: bool,
}

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct SplitOptions {
    #[command(flatten)]
    pub input: super::InputOptions,

    /// The directory to store the partitions
    #[arg(value_name = "DIR")]
    pub outdir: Option<PathBuf>,

    #[arg(short = 'c', long, action = clap::ArgAction::SetTrue)]
    pub include_common: bool,
}

pub fn main(cli: &Cli, subcommand: Option<&FlashSubCommand>) -> Result<(), amebazii::error::Error> {
    match subcommand {
        Some(FlashSubCommand::Parse { file, pt_only }) => {
            parse::parse(cli, file.clone().expect("File is required"), *pt_only)?;
        }
        Some(FlashSubCommand::Split { options }) => {
            split::split_flash(cli, options)?;
        }
        Some(FlashSubCommand::Combine { options }) => match options {
            Some(options) => {
                combine::combine_images(cli, options)?;
            }
            None => {}
        },
        _ => {}
    }
    Ok(())
}
