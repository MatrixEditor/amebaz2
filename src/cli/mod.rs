use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod ota;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    Ota {
        #[command(subcommand)]
        subcommand: Option<OTASubCommand>,
    }
}

#[derive(Subcommand)]
pub enum OTASubCommand {
    #[command(arg_required_else_help = true)]
    Parse {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
}

macro_rules! debug {
    ($msg:literal) => {
        println!("{}{}", "D : ".bold().color(Color::BrightBlack), $msg.color(Color::BrightBlack));
    };
    ($argmsg:literal, $($arg:tt)*) => {
        println!("{}{}", "D : ".bold().color(Color::BrightBlack), format!($argmsg, $($arg)*).color(Color::BrightBlack));
    }
}

macro_rules! error {
    ($msg:literal, $($arg:tt)*) => {
        println!("{}{}", "E : ".bold().color(Color::Red),  format!($msg, $($arg)*).color(Color::Red));
    };
}

pub(crate) use debug;
pub(crate) use error;

