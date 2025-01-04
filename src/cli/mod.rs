use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod flash;
pub mod ota;
pub mod util;

/// AmebaZ2 Tools to work with OTA and flash images
#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(about, long_about = None)]
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
