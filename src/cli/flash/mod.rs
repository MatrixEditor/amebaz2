use super::{Cli, FlashSubCommand};

mod parse;
mod split;

pub fn main(cli: &Cli, subcommand: Option<&FlashSubCommand>) -> Result<(), crate::error::Error> {
    match subcommand {
        Some(FlashSubCommand::Parse { file, pt_only }) => {
            parse::parse(cli, file.clone().expect("File is required"), *pt_only)?;
        }
        Some(FlashSubCommand::Split { file, outdir }) => {
            split::split_flash(
                cli,
                file.clone().expect("File is required"),
                outdir.clone().expect("Outdir is required"),
            )?;
        }
        _ => {}
    }
    Ok(())
}
