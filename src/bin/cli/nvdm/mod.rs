use amebazii::error::Error;
use clap::Parser;
use std::path::PathBuf;

use crate::cli::{Cli, NvdmSubCommand};

mod parse;

/// List all entries of an NVDM image.
#[derive(Parser)]
pub struct ParseOptions {
    /// The input NVDM flash image to be parsed.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// only output all configuration group names
    #[arg(long, action = clap::ArgAction::SetTrue, long = "list-groups")]
    groups: bool,

    /// only display valid entries (data items)
    #[arg(long = "only-valid", action = clap::ArgAction::SetTrue)]
    only_valid: bool,

    /// Filter data items by group
    #[arg(short, long, value_name = "NAME")]
    pub group: Option<String>,

    /// Filter data items by name
    #[arg(short, long, value_name = "NAME")]
    pub item_name: Option<String>,

    /// Flash block size (default is 4096)
    #[arg(short, long, value_name = "SIZE")]
    pub block_size: Option<u32>,
}

pub fn main(cli: &Cli, command: Option<&NvdmSubCommand>) -> Result<(), Error> {
    match command {
        Some(NvdmSubCommand::View { options }) => {
            parse::parse(cli, options)?;
        },
        _ => {}
    }

    Ok(())
}