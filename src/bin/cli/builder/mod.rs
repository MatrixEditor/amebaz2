use super::{headings, BuildSubCommand, Cli};
use clap::Parser;
use std::path::PathBuf;

mod parttab;
mod sysdata;

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct BuildPartitionTableOptions {
    /// Output file path where the partition table will be saved.
    ///
    /// This argument specifies the path to the file where the generated partition table
    /// will be written.
    #[clap(verbatim_doc_comment)]
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Whether to fill the entire sector with data (default: false).
    ///
    /// If enabled, the partition table will fill the entire sector during the build process,
    /// which might be useful for certain flashing or partitioning scenarios. This option
    /// is typically used when you want to align the partition table to sector boundaries.
    #[clap(verbatim_doc_comment)]
    #[arg(short, long, action = clap::ArgAction::SetTrue, help_heading = headings::BUILD_OPTIONS)]
    pub fill_sector: bool,

    /// Configuration file to use when building the partition table.
    ///
    /// This option allows you to specify a configuration file that contains predefined
    /// settings or parameters for the partition table. The configuration file is expected
    /// to be in a format understood by the tool (JSON).
    #[clap(verbatim_doc_comment)]
    #[arg(short, long, value_name = "CFG")]
    pub config: Option<PathBuf>,

    /// Path to a configuration file to generate partition table options to.
    ///
    /// This option is used to generate a configuration file for partition table settings
    /// rather than directly building the partition table. The generated file can be used
    /// later to configure the partition table build process.
    #[clap(verbatim_doc_comment)]
    #[arg(short = 'G', long, value_name = "CFG", help_heading = headings::CONFIG_GEN_OPTIONS)]
    pub generate_config: Option<PathBuf>,

    /// Flag to generate default entries for the partition table (default: false).
    ///
    /// If this flag is set, default partition entries will be created during the build process,
    /// which might be helpful for quick tests or when the user does not want to manually specify
    /// all the entries.
    #[clap(verbatim_doc_comment)]
    #[arg(long = "default-entries", action = clap::ArgAction::SetTrue, help_heading = headings::CONFIG_GEN_OPTIONS)]
    pub generate_default_entries: bool,

    /// Skip calibration pattern generation during partition table build.
    ///
    /// If this flag is set, the flash calibration pattern will not be included in the generated
    /// partition table.
    #[clap(verbatim_doc_comment)]
    #[arg(long, action = clap::ArgAction::SetTrue, help_heading = headings::BUILD_OPTIONS)]
    pub no_calibpat: bool,

    // Default options for the partition table.
    // eFWV - naming unclear
    /// Set eFWV for the partition table.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::PARTTAB_OPTIONS)]
    pub efwv: Option<u8>,

    /// Set the RMA write state for the partition table.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::PARTTAB_OPTIONS)]
    pub rma_wstate: Option<u8>,

    /// Set the RMA overwrite state for the partition table.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::PARTTAB_OPTIONS)]
    pub rma_ovstate: Option<u8>,

    /// Set the index for firmware 1 in the partition table.
    ///
    /// This option specifies the index of the first firmware image in the partition table. The partition
    /// table uses this index to point to the location of firmware 1 on the storage medium.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::PARTTAB_OPTIONS, value_name = "IDX")]
    pub fw1_idx: Option<u8>,

    /// Set the index for firmware 2 in the partition table.
    ///
    /// This option specifies the index of the second firmware image in the partition table. The partition
    /// table uses this index to point to the location of firmware 2 on the storage medium.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::PARTTAB_OPTIONS, value_name = "IDX")]
    pub fw2_idx: Option<u8>,

    /// Set the key export operation for the partition table.
    ///
    /// This option allows you to define how keys are exported or handled within the partition table.
    #[clap(verbatim_doc_comment)]
    #[arg(value_enum, long, help_heading = headings::PARTTAB_OPTIONS, value_name = "OP")]
    pub key_exp_op: Option<u8>,

    /// Set a custom user extension field for the partition table.
    ///
    /// This field is used to store custom data related to the partition table. The data can be
    /// a hex string or a binary file.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::PARTTAB_OPTIONS)]
    pub user_ext: Option<String>,

    /// Set a custom user binary file for the partition table.
    ///
    /// This field allows you to specify a binary file that will be included as part of the partition table.
    /// The file can contain firmware, configuration data, or other relevant information that will be embedded
    /// in the partition table.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::PARTTAB_OPTIONS)]
    pub user_bin: Option<String>,
}

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct BuildSystemDataOptions {
    /// Output file path where the system data will be saved.
    ///
    /// Specifies the path to the file where the generated system data
    /// will be written.
    #[clap(verbatim_doc_comment)]
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Path to a configuration file.
    ///
    /// Specifies the path to a configuration file that will
    /// be used when building the system data.
    #[clap(verbatim_doc_comment)]
    #[arg(short, long, value_name = "CFG", value_hint = clap::ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    /// Address for OTA2 (Over-the-Air update) system.
    ///
    /// Allows specifying a custom address for OTA2 data.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::SYSDATA_OPTIONS)]
    pub ota2_address: Option<u32>,

    /// Size of the OTA2 data.
    ///
    /// Specifies the size of the OTA2 data that will be used.
    /// It is usually provided as a 32-bit unsigned integer.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::SYSDATA_OPTIONS)]
    pub ota2_size: Option<u32>,

    /// Baud rate for the ULOG (Universal Log) system.
    ///
    /// Sets the baud rate for the ULOG communication interface.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::SYSDATA_OPTIONS)]
    pub ulog_baud: Option<u32>,

    /// SPI calibration settings as a string.
    ///
    /// Provides the SPI calibration settings in string format,
    /// which may be used to configure the system for specific
    /// SPI communication requirements.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::SYSDATA_OPTIONS, value_name = "FILE/DATA")]
    pub spic_setting: Option<String>,

    /// BT parameter data.
    #[clap(verbatim_doc_comment)]
    #[arg(long, help_heading = headings::SYSDATA_OPTIONS, value_name = "FILE/DATA")]
    pub bt_parameter_data: Option<String>,

    /// Flag to generate a configuration file.
    #[clap(verbatim_doc_comment)]
    #[arg(short = 'G', long, value_name = "CFG", help_heading = headings::CONFIG_GEN_OPTIONS)]
    pub generate_config: Option<PathBuf>,
}

pub fn main(cli: &Cli, subcommand: Option<&BuildSubCommand>) -> Result<(), amebazii::error::Error> {
    match subcommand {
        Some(BuildSubCommand::Parttab { options }) => {
            parttab::build_parttab(cli, options.as_ref().unwrap())?
        }
        Some(BuildSubCommand::Sysdata { options }) => {
            sysdata::build_sysdata(cli, options.as_ref().unwrap())?
        }
        _ => {}
    }

    Ok(())
}
