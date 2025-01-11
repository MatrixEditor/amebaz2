use clap::Parser;

mod cli;

use cli::{Commands, Cli};

use amebazii::error::Error;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        None => {}
        Some(Commands::Ota { subcommand }) => {
            cli::ota::main(&cli, subcommand.as_ref())?;
        }
        Some(Commands::Flash { subcommand }) => cli::flash::main(&cli, subcommand.as_ref())?,
        Some(Commands::Build { subcommand }) => cli::builder::main(&cli, subcommand.as_ref())?,
    }

    Ok(())
}