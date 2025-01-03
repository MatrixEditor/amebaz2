use crate::error::Error;

use super::{Cli, OtaSubCommand};

pub mod parse;

pub fn main(cli: &Cli, command: Option<&OtaSubCommand>) -> Result<(), Error> {
    match command {
        Some(OtaSubCommand::Parse { file }) => parse::parse(cli, file.clone().unwrap())?,
        _ => (),
    }
    Ok(())
}
