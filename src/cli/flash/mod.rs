use super::{Cli, FlashSubCommand};

pub mod parse;

pub fn main(cli: &Cli, subcommand: Option<&FlashSubCommand>) -> Result<(), crate::error::Error> {
    match subcommand {
        Some(FlashSubCommand::Parse { file }) => {
            parse::parse(cli, file.clone().expect("File is required"))?;
        }
        _ => {}
    }
    Ok(())
}
