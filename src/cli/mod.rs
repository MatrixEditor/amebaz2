use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod builder;
pub mod flash;
pub mod ota;
pub mod util;

mod headings {
    pub const PARTTAB_OPTIONS: &str = "PartitionTable Options";
    pub const CONFIG_GEN_OPTIONS: &str = "Config Generation Options";
    pub const BUILD_OPTIONS: &str = "Build Options";
    pub const ADDRESS_OPTIONS: &str = "Address Options";
}

/// AmebaZ2 Tools to work with OTA and flash images
#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(about, long_about)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Verbosity level for logging/debugging
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Main commands in the CLI.
#[derive(Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    Ota {
        #[command(subcommand)]
        subcommand: Option<OtaSubCommand>,
    },

    #[command(arg_required_else_help = true)]
    Flash {
        #[command(subcommand)]
        subcommand: Option<FlashSubCommand>,
    },

    #[command(arg_required_else_help = true)]
    Build {
        #[command(subcommand)]
        subcommand: Option<BuildSubCommand>,
    },
}

/// Flash-related operations.
#[derive(Subcommand)]
pub enum FlashSubCommand {
    /// Parse a flash image file.
    #[command(arg_required_else_help = true)]
    Parse {
        /// The flash image file to parse
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Parse only the partition table
        #[arg(long, action = clap::ArgAction::SetTrue)]
        pt_only: bool,
    },

    /// Split a flash image file into partitions and store them in a directory.
    #[command(arg_required_else_help = true)]
    Split {
        /// The flash image file to split (must start with calibration pattern)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// The directory to store the partitions
        #[arg(value_name = "DIR")]
        outdir: Option<PathBuf>,
    },
}

/// OTA-related operations
#[derive(Subcommand)]
pub enum OtaSubCommand {
    /// Parse an OTA image file.
    #[command(arg_required_else_help = true)]
    Parse {
        /// The OTA image file to parse
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Extracts sections from subimages within an OTA image
    #[command(arg_required_else_help = true)]
    Dump {
        /// Only dump the specified section
        #[arg(short, long)]
        section: Option<u32>,

        /// The OTA image file to dump from
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Index of the subimage to dump (start with 0)
        #[arg(short = 'I', value_name = "SUBIMAGE", required = true)]
        subimage: Option<u32>,

        /// The directory to store the sections or the target file to store the section
        #[arg(value_name = "DIR/FILE")]
        outdir: Option<PathBuf>,
    },

    /// Relink a firmware binary (OTA image)
    ///
    /// Example:
    ///     - amebazii ota relink --cap-length ./ota.bin ./ota.elf
    #[clap(verbatim_doc_comment)]
    #[command(arg_required_else_help = true, about, long_about)]
    Relink {
        /// The input firmware file to be relinked.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// The output firmware file after relinking.
        #[arg(value_name = "OUTFILE")]
        outfile: Option<PathBuf>,

        /// The directory where intermediate files will be saved (section data).
        #[arg(short, long, value_name = "DIR", value_hint = clap::ValueHint::DirPath)]
        save_intermediate: Option<PathBuf>,

        /// Cap the length of the output sections based on available data (i.e. ignores errors)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        cap_length: bool,

        /// Start address of the RAM vector table.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        ram_vector_start: Option<u64>,

        /// End address of the RAM vector table.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        ram_vector_end: Option<u64>,

        /// Start address of the RAM function entry table.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        ram_func_table_start: Option<u64>,

        /// End address of the RAM function entry table.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        ram_func_table_end: Option<u64>,

        /// Start address of the RAM image signature.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        ram_img_signature: Option<u64>,

        /// End address of the RAM image signature.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        ram_img_signature_end: Option<u64>,

        /// Start address of the program code and text sections in RAM.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        ram_code_text: Option<u64>,

        /// End address of the program code and text sections in RAM.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        ram_code_text_end: Option<u64>,

        /// Start address of the DTCM (Data Tightly Coupled Memory) RAM region.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        dtcm_ram: Option<u64>,

        /// End address of the DTCM RAM region.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        dtcm_ram_end: Option<u64>,

        /// Start address of the XIP encrypted section in flash.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        xip_c_start: Option<u64>,

        /// End address of the XIP encrypted section in flash.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        xip_c_end: Option<u64>,

        /// Start address of the XIP plaintext section in flash.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        xip_p_start: Option<u64>,

        /// End address of the XIP plaintext section in flash.
        #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
        xip_p_end: Option<u64>,
    },
}

#[derive(Subcommand)]
pub enum BuildSubCommand {
    /// Command to build partition tables.
    #[command(arg_required_else_help = true)]
    Parttab {
        #[command(flatten)]
        options: Option<BuildPartitionTableOptions>,
    },
}

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

/// Macro for printing debug messages with formatting.
macro_rules! debug {
    ($cli: expr, $msg:literal) => {
        if $cli.verbose > 2 {
            println!("{}{}", "D : ".bold().color(Color::BrightBlack), $msg.color(Color::BrightBlack));
        }
    };
    ($cli:expr, $argmsg:literal, $($arg:tt)*) => {
        if $cli.verbose > 2 {
            println!("{}{}", "D : ".bold().color(Color::BrightBlack), format!($argmsg, $($arg)*).color(Color::BrightBlack));
        }
    }
}

/// Macro for printing error messages with formatting.
macro_rules! error {
    ($msg:literal, $($arg:tt)*) => {
        println!("{}{}", "E : ".bold().red(),  format!($msg, $($arg)*).red());
    };
}

pub(crate) use debug;
pub(crate) use error;
