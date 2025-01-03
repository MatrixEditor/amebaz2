use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod flash;
pub mod ota;
pub mod util;

/// CLI (Command Line Interface) structure that holds verbosity level and subcommands.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Verbosity level for logging/debugging, specified by repeated `-v` flags.
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
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum OtaSubCommand {
    /// Parse an OTA image file.
    #[command(arg_required_else_help = true)]
    Parse {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
}

/// Macro for printing debug messages with formatting.
#[macro_export]
macro_rules! debug {
    ($msg:literal) => {
        println!("{}{}", "D : ".bold().color(Color::BrightBlack), $msg.color(Color::BrightBlack));
    };
    ($argmsg:literal, $($arg:tt)*) => {
        println!("{}{}", "D : ".bold().color(Color::BrightBlack), format!($argmsg, $($arg)*).color(Color::BrightBlack));
    }
}

/// Macro for printing error messages with formatting.
#[macro_export]
macro_rules! error {
    ($msg:literal, $($arg:tt)*) => {
        println!("{}{}", "E : ".bold().color(Color::Red),  format!($msg, $($arg)*).color(Color::Red));
    };
}

pub(crate) use debug;
pub(crate) use error;
