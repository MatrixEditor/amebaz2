use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod flash;
pub mod ota;
pub mod util;

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
        #[arg(long, value_name = "ADDR")]
        ram_vector_start: Option<u64>,

        /// End address of the RAM vector table.
        #[arg(long, value_name = "ADDR")]
        ram_vector_end: Option<u64>,

        /// Start address of the RAM function entry table.
        #[arg(long, value_name = "ADDR")]
        ram_func_table_start: Option<u64>,

        /// End address of the RAM function entry table.
        #[arg(long, value_name = "ADDR")]
        ram_func_table_end: Option<u64>,

        /// Start address of the RAM image signature.
        #[arg(long, value_name = "ADDR")]
        ram_img_signature: Option<u64>,

        /// End address of the RAM image signature.
        #[arg(long, value_name = "ADDR")]
        ram_img_signature_end: Option<u64>,

        /// Start address of the program code and text sections in RAM.
        #[arg(long, value_name = "ADDR")]
        ram_code_text: Option<u64>,

        /// End address of the program code and text sections in RAM.
        #[arg(long, value_name = "ADDR")]
        ram_code_text_end: Option<u64>,

        /// Start address of the DTCM (Data Tightly Coupled Memory) RAM region.
        #[arg(long, value_name = "ADDR")]
        dtcm_ram: Option<u64>,

        /// End address of the DTCM RAM region.
        #[arg(long, value_name = "ADDR")]
        dtcm_ram_end: Option<u64>,

        /// Start address of the XIP encrypted section in flash.
        #[arg(long, value_name = "ADDR")]
        xip_c_start: Option<u64>,

        /// End address of the XIP encrypted section in flash.
        #[arg(long, value_name = "ADDR")]
        xip_c_end: Option<u64>,

        /// Start address of the XIP plaintext section in flash.
        #[arg(long, value_name = "ADDR")]
        xip_p_start: Option<u64>,

        /// End address of the XIP plaintext section in flash.
        #[arg(long, value_name = "ADDR")]
        xip_p_end: Option<u64>,
    },
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
