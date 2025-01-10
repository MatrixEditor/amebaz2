use super::{BuildSubCommand, Cli};

mod parttab;
mod sysdata;

pub fn main(cli: &Cli, subcommand: Option<&BuildSubCommand>) -> Result<(), crate::error::Error> {

    match subcommand {
        Some(BuildSubCommand::Parttab{options}) => {
            parttab::build_parttab(cli, options.as_ref().unwrap())?
        },
        Some(BuildSubCommand::Sysdata{options}) => {
            sysdata::build_sysdata(cli, options.as_ref().unwrap())?
        },
        _ => {}
    }

    Ok(())
}
