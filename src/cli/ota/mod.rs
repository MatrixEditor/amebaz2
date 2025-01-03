use crate::error::Error;

use super::{Cli, OTASubCommand};

pub mod parse;

pub fn main(cli: &Cli, command: Option<&OTASubCommand>) -> Result<(), Error> {
    match command {
        Some(OTASubCommand::Parse { file }) => parse::parse(cli, file.clone().unwrap())?,
        _ => (),
    }
    Ok(())
}
