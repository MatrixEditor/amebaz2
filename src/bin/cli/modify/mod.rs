use clap::Parser;
use clap_num::maybe_hex;

use super::{Cli, ModSubCommand};

mod headings {
    pub const RECORD_OPTIONS: &str = "Record Options";
}

mod parttab;

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
#[command(arg_required_else_help = true)]
pub struct ModParttabOptions {
    #[command(flatten)]
    pub input: super::InputOptions,

    #[command(flatten)]
    pub output: super::OutputOptions,

    #[arg(short, action = clap::ArgAction::SetTrue)]
    pub in_place: bool,

    #[arg(long = "has-calib", action = clap::ArgAction::SetTrue)]
    pub has_calibration: bool,

    #[arg(long = "add-calib", action = clap::ArgAction::SetTrue)]
    pub add_calibration: bool,

    #[arg(long = "set-fw1", value_name = "IDX", help_heading = super::headings::PARTTAB_OPTIONS)]
    pub fw1_idx: Option<u8>,

    #[arg(long = "set-fw2", value_name = "IDX", help_heading = super::headings::PARTTAB_OPTIONS)]
    pub fw2_idx: Option<u8>,

    #[arg(long = "set-efwv", value_name = "EFWV", help_heading = super::headings::PARTTAB_OPTIONS)]
    pub efwv: Option<u8>,

    #[arg(long = "set-rma-wstate", value_name = "STATE", help_heading = super::headings::PARTTAB_OPTIONS)]
    pub rma_wstate: Option<u8>,

    #[arg(long = "set-rma-ovstate", value_name = "STATE", help_heading = super::headings::PARTTAB_OPTIONS)]
    pub rma_ovstate: Option<u8>,

    #[arg(short = 'R', conflicts_with = "add", group = "r", value_name = "TYPE")]
    pub remove: Option<PartitionType>,

    #[arg(
        short = 'A',
        conflicts_with = "remove",
        conflicts_with = "modify",
        group = "a",
        value_name = "TYPE",
        name = "add"
    )]
    pub add: Option<PartitionType>,

    #[arg(
        short = 'M',
        conflicts_with = "remove",
        conflicts_with = "add",
        group = "m",
        value_name = "TYPE",
        name = "modify"
    )]
    pub alter: Option<PartitionType>,

    #[arg(long, value_name = "ADDR", help_heading = headings::RECORD_OPTIONS, value_parser = maybe_hex::<u32>)]
    pub start: Option<u32>,

    #[arg(long, value_name = "SIZE", help_heading = headings::RECORD_OPTIONS, value_parser = maybe_hex::<u32>)]
    pub length: Option<u32>,

    #[arg(long, action = clap::ArgAction::SetTrue, help_heading = headings::RECORD_OPTIONS)]
    pub debug_skip: bool,

    #[arg(long, value_name = "FILE/KEY", help_heading = headings::RECORD_OPTIONS)]
    pub hash_key: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum PartitionType {
    Boot,
    Fw1,
    Fw2,
    User,
    Sys,
    // other types not supported yet
}


pub fn main(cli: &Cli, subcommand: Option<&ModSubCommand>) -> Result<(), amebazii::error::Error> {
    match subcommand {
        Some(ModSubCommand::Parttab { options }) => parttab::modify_parttab(cli, options)?,
        _ => {}
    }
    Ok(())
}
