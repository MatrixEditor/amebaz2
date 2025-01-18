use clap::Parser;
use colored::Colorize;

mod cli;
use cli::{error, Cli, Commands};

use amebazii::error::Error;

fn cli_entry() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        None => {}
        Some(Commands::Ota { subcommand }) => {
            cli::ota::main(&cli, subcommand.as_ref())?;
        }
        Some(Commands::Flash { subcommand }) => cli::flash::main(&cli, subcommand.as_ref())?,
        Some(Commands::Build { subcommand }) => cli::builder::main(&cli, subcommand.as_ref())?,
        Some(Commands::Mod { subcommand }) => cli::modify::main(&cli, subcommand.as_ref())?,
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    match cli_entry() {
        Ok(()) => (),
        Err(e) => match e {
            Error::InvalidState(msg) => {
                error!("Encountered an invalid state: {}", msg);
            }
            Error::NotImplemented(msg) => {
                error!("The following feature is not implemented: {}", msg);
            }

            Error::UnknownImageType(msg) => {
                error!("Could not translate image type {} to an enum!", msg);
            }
            _ => {
                error!("{}", e);
            }
        },
    }
    Ok(())
}
