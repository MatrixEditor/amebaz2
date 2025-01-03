use clap::Parser;

use amebazii::cli::{self, Cli, Commands};
use amebazii::error::Error;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        None => {}
        Some(Commands::Ota { subcommand }) => {
            cli::ota::main(&cli, subcommand.as_ref())?;
        }
        Some(Commands::Flash { subcommand }) => cli::flash::main(&cli, subcommand.as_ref())?,
    }

    Ok(())
}
