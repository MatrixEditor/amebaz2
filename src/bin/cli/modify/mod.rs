use clap::Parser;
use clap_num::maybe_hex;

use super::{Cli, ModSubCommand};

mod headings {
    pub const RECORD_OPTIONS: &str = "Record Options";
    pub const OTA_SWITCH_OPTIONS: &str = "OTA Switch Options";
}

mod parttab;
mod sysdata;

/// Modify partition tables
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
#[command(arg_required_else_help = true)]
pub struct ModParttabOptions {
    #[command(flatten)]
    pub input: super::InputOptions,

    #[command(flatten)]
    pub output: super::OutputOptionsInPlace,

    /// Specifies if the input partition table contains calibration data.
    #[arg(long = "has-calib", action = clap::ArgAction::SetTrue)]
    pub has_calibration: bool,

    /// Determines if calibration data should be added to the new partition table.
    #[arg(long = "add-calib", action = clap::ArgAction::SetTrue)]
    pub add_calibration: bool,

    /// Sets the index of the first firmware partition. Must be a valid index value.
    #[arg(
        long = "set-fw1",
        value_name = "IDX",
        help_heading = super::headings::PARTTAB_OPTIONS
    )]
    pub fw1_idx: Option<u8>,

    /// Sets the index of the second firmware partition.
    #[arg(
        long = "set-fw2",
        value_name = "IDX",
        help_heading = super::headings::PARTTAB_OPTIONS
    )]
    pub fw2_idx: Option<u8>,

    #[arg(
        long = "set-efwv",
        value_name = "EFWV",
        help_heading = super::headings::PARTTAB_OPTIONS
    )]
    pub efwv: Option<u8>,

    #[arg(
        long = "set-rma-wstate",
        value_name = "STATE",
        help_heading = super::headings::PARTTAB_OPTIONS
    )]
    pub rma_wstate: Option<u8>,

    #[arg(
        long = "set-rma-ovstate",
        value_name = "STATE",
        help_heading = super::headings::PARTTAB_OPTIONS
    )]
    pub rma_ovstate: Option<u8>,

    /// Defines a partition type to be removed from the table.
    ///
    /// Conflicts with `add` and `modify` options.
    #[arg(short = 'R', conflicts_with = "add", group = "r", value_name = "TYPE")]
    pub remove: Option<PartitionType>,

    /// Specifies a partition type to be added.
    ///
    /// This option cannot be used with `remove` or `modify`.
    #[arg(
        short = 'A',
        conflicts_with = "remove",
        conflicts_with = "modify",
        group = "a",
        value_name = "TYPE",
        name = "add"
    )]
    pub add: Option<PartitionType>,

    /// Specifies a partition type to modify in the table.
    ///
    /// Cannot be used with `remove` or `add`.
    #[arg(
        short = 'M',
        conflicts_with = "remove",
        conflicts_with = "add",
        group = "m",
        value_name = "TYPE",
        name = "modify"
    )]
    pub alter: Option<PartitionType>,

    /// Sets the starting address for the partition record.
    ///
    /// The address should be provided as a decimal or hexadecimal value.
    #[arg(
        long,
        value_name = "ADDR",
        help_heading = headings::RECORD_OPTIONS,
        value_parser = maybe_hex::<u32>
    )]
    pub start: Option<u32>,

    /// Defines the length of the partition record in bytes, given as a decimal or hexadecimal value.
    #[arg(
        long,
        value_name = "ADDR",
        help_heading = headings::RECORD_OPTIONS,
        value_parser = maybe_hex::<u32>
    )]
    pub length: Option<u32>,

    /// Skips the debugging check for the record..
    #[arg(long, action = clap::ArgAction::SetTrue, help_heading = headings::RECORD_OPTIONS)]
    pub debug_skip: bool,

    /// Provides a hash key or file path used to sign the partition.
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

/// Modify system control settings, like OTA switch configurations.
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
#[command(arg_required_else_help = true)]
pub struct ModSysctrlOptions {
    #[command(flatten)]
    pub input: super::InputOptions,

    #[command(flatten)]
    pub output: super::OutputOptionsInPlace,

    /// Sets the address for the OTA2 switch.
    #[arg(
        long = "set-ota2-addr",
        value_name = "ADDR",
        help_heading = headings::OTA_SWITCH_OPTIONS,
        value_parser = maybe_hex::<u32>,
    )]
    pub ota2_addr: Option<u32>,

    /// Specifies the size of the OTA2 section in the firmware.
    #[arg(
        long = "set-ota2-size",
        value_name = "SIZE",
        help_heading = headings::OTA_SWITCH_OPTIONS,
        value_parser = maybe_hex::<u32>,
    )]
    pub ota2_size: Option<u32>,

    /// Disables the OTA2 functionality, preventing it from being activated in the firmware.
    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        conflicts_with = "ota2_addr",
        conflicts_with = "ota2_size",
    )]
    pub ota2_disable: bool,
    // other options not supported yet
}

pub fn main(cli: &Cli, subcommand: Option<&ModSubCommand>) -> Result<(), amebazii::error::Error> {
    match subcommand {
        Some(ModSubCommand::Parttab { options }) => parttab::modify_parttab(cli, options)?,
        Some(ModSubCommand::Sysdata { options }) => sysdata::modify_sysdata(cli, options)?,
        _ => {}
    }
    Ok(())
}
