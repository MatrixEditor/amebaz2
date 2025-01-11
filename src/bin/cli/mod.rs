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
    pub const SYSDATA_OPTIONS: &str = "SystemData Options";
    pub const PART_OPTIONS: &str = "Partition Options";
}

/// AmebaZ2 Tools to work with OTA and flash images
#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(name = "amebazii")]
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

    #[command(arg_required_else_help = true)]
    Combine {
        #[command(flatten)]
        options: Option<flash::CombineOptions>,
    }
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
        #[command(flatten)]
        options: Option<ota::RelinkOptions>,
    },
}

#[derive(Subcommand)]
pub enum BuildSubCommand {
    /// Command to build partition tables.
    #[command(arg_required_else_help = true)]
    Parttab {
        #[command(flatten)]
        options: Option<builder::BuildPartitionTableOptions>,
    },

    /// Builds the system data partition
    #[command(arg_required_else_help = true)]
    Sysdata {
        #[command(flatten)]
        options: Option<builder::BuildSystemDataOptions>,
    }
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
