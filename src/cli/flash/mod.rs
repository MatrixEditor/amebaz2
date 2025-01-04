use super::{Cli, FlashSubCommand};

mod parse;

pub fn main(cli: &Cli, subcommand: Option<&FlashSubCommand>) -> Result<(), crate::error::Error> {
    match subcommand {
        Some(FlashSubCommand::Parse { file, pt_only }) => {
            parse::parse(cli, file.clone().expect("File is required"), *pt_only)?;
        }
        _ => {}
    }
    Ok(())
}
