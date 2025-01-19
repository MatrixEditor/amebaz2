use colored::Colorize;

use amebazii::types::{from_stream, transfer_to, SystemData};

use crate::cli::{debug, util, Cli};

use super::ModSysctrlOptions;

pub fn modify_sysdata(
    cli: &Cli,
    options: &ModSysctrlOptions,
) -> Result<(), amebazii::error::Error> {
    if let Some(input_file) = &options.input.file {
        let input = std::fs::File::open(input_file);
        if input.is_err() {
            return Ok(());
        }

        let mut input = input.unwrap();
        let mut data: SystemData = from_stream(&mut input)?;

        if options.ota2_disable {
            debug!(cli, "Disabling OTA2");
            data.ota2_size = None;
            data.ota2_addr = None;
        } else {
            if let Some(addr) = options.ota2_addr {
                debug!(cli, "Setting OTA2 address to {:#x}", addr);
                data.ota2_addr = Some(addr);
            }
            if let Some(size) = options.ota2_size {
                debug!(cli, "Setting OTA2 size to {:#x}", size);
                data.ota2_size = Some(size);
            }
        }

        let mut output = util::open_output_file(cli, Some(&options.input), &options.output)?;
        transfer_to(&data, &mut output)?;
    }

    Ok(())
}
