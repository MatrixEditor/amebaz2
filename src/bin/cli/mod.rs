use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod builder;
pub mod flash;
pub mod modify;
pub mod ota;
pub mod util;

mod headings {
    pub const PARTTAB_OPTIONS: &str = "PartitionTable Options";
    pub const CONFIG_GEN_OPTIONS: &str = "Config Generation Options";
    pub const BUILD_OPTIONS: &str = "Build Options";
    pub const ADDRESS_OPTIONS: &str = "Address Options";
    pub const SYSDATA_OPTIONS: &str = "SystemData Options";
    pub const PART_OPTIONS: &str = "Partition Options";
    pub const OUTPUT_OPTIONS: &str = "Output Options";
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

#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct InputOptions {
    /// Input file path.
    #[clap(verbatim_doc_comment)]
    #[arg(value_name = "INFILE", name = "input_file", required = true)]
    pub file: Option<PathBuf>,
}

pub trait OutputOptionsExt {
    fn file(&self) -> Option<PathBuf>;

    fn force(&self) -> bool {
        false
    }

    fn in_place(&self) -> bool {
        false
    }
}

#[derive(Parser)]
pub struct OutputOptions {
    /// Output file path where the binary data will be saved.
    ///
    /// This argument specifies the path to the file where the generated data
    /// will be written.
    #[clap(verbatim_doc_comment)]
    #[arg(value_name = "OUTFILE", name = "output_file")]
    pub file: Option<PathBuf>,

    /// Overwrite the output file if it already exists.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub force: bool,
}

impl OutputOptionsExt for OutputOptions {
    fn file(&self) -> Option<PathBuf> {
        self.file.clone()
    }

    fn force(&self) -> bool {
        self.force
    }
}

#[derive(Parser)]
pub struct OutputOptionsInPlace {
    /// Output file path where the binary data will be saved.
    ///
    /// This argument specifies the path to the file where the generated data
    /// will be written.
    #[clap(verbatim_doc_comment)]
    #[arg(
        value_name = "OUTFILE",
        name = "output_file",
        conflicts_with = "in_place",
        required_unless_present = "in_place"
    )]
    pub file: Option<PathBuf>,

    /// Overwrite the output file if it already exists.
    #[arg(long, action = clap::ArgAction::SetTrue, help_heading = headings::OUTPUT_OPTIONS)]
    pub force: bool,

    /// Edit the input file in-place.
    #[arg(
        short,
        long,
        action = clap::ArgAction::SetTrue,
        conflicts_with = "output_file",
        help_heading = headings::OUTPUT_OPTIONS
    )]
    pub in_place: bool,
}

impl OutputOptionsExt for OutputOptionsInPlace {
    fn file(&self) -> Option<PathBuf> {
        self.file.clone()
    }

    fn force(&self) -> bool {
        self.force
    }

    fn in_place(&self) -> bool {
        self.in_place
    }
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

    #[command(arg_required_else_help = true)]
    Mod {
        #[command(subcommand)]
        subcommand: Option<ModSubCommand>,
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
        #[command(flatten)]
        options: flash::SplitOptions,
    },

    #[command(arg_required_else_help = true)]
    Combine {
        #[command(flatten)]
        options: Option<flash::CombineOptions>,
    },
}

/// OTA-related operations
#[derive(Subcommand)]
pub enum OtaSubCommand {
    /// Parse an OTA image file.
    #[command(arg_required_else_help = true)]
    Parse {
        /// The OTA image file to parse
        #[command(flatten)]
        options: Option<ota::ParseOptions>,
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

    /// Resign a firmware binary (OTA image)
    ///
    /// Example:
    ///     - amebazii ota resign --key ./key.bin ./ota.bin ./new_ota.bin
    #[clap(verbatim_doc_comment)]
    #[command(arg_required_else_help = true)]
    Resign {
        #[command(flatten)]
        options: ota::ReSignOptions,
    },
}

/// Builder for Partition tables and mroe
#[derive(Subcommand)]
pub enum BuildSubCommand {
    /// Command to build partition tables.
    #[command(arg_required_else_help = true)]
    Parttab {
        #[command(flatten)]
        options: builder::BuildPartitionTableOptions,
    },

    /// Builds the system data partition
    #[command(arg_required_else_help = true)]
    Sysdata {
        #[command(flatten)]
        options: builder::BuildSystemDataOptions,
    },
}

/// Modification operations
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub enum ModSubCommand {
    #[command(arg_required_else_help = true)]
    Parttab {
        #[command(flatten)]
        options: modify::ModParttabOptions,
    },

    #[command(arg_required_else_help = true)]
    Sysdata {
        #[command(flatten)]
        options: modify::ModSysctrlOptions,
    },
}

/// Macro for printing debug messages with formatting.
macro_rules! debug {
    ($cli: expr, $msg:literal) => {
        if $cli.verbose > 2 {
            println!("{}{}", "D : ".bold().bright_black(), $msg.bright_black());
        }
    };
    ($cli:expr, $argmsg:literal, $($arg:tt)*) => {
        if $cli.verbose > 2 {
            println!("{}{}", "D : ".bold().bright_black(), format!($argmsg, $($arg)*).bright_black());
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
