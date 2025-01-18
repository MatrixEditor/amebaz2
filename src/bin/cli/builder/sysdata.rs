use crate::cli::{debug, error, Cli};
use amebazii::{
    conf::{DataArray, SystemDataCfg},
    types::{sysctrl::SystemData, transfer_to},
};

use colored::Colorize;

use super::BuildSystemDataOptions;

pub fn build_sysdata(
    cli: &Cli,
    options: &BuildSystemDataOptions,
) -> Result<(), amebazii::error::Error> {
    if let Some(default_config_file) = &options.gen_defaults.generate_config {
        debug!(
            cli,
            "Generating default config file: {:#?}", default_config_file
        );
        let config = SystemDataCfg::default();
        let mut cfgout = std::fs::File::create(default_config_file.clone())?;
        serde_json::to_writer_pretty(&mut cfgout, &config)?;
        return Ok(());
    }

    let mut config: SystemDataCfg;
    if let Some(config_file) = &options.config.file {
        let mut cfgin = std::fs::File::open(config_file.clone())?;
        config = serde_json::from_reader(&mut cfgin).unwrap();
    } else {
        config = SystemDataCfg::default();
    }

    if let Some(ota2_address) = options.ota2_address {
        config.ota2_addr = Some(ota2_address);
    }
    if let Some(size) = options.ota2_size {
        config.ota2_size = Some(size);
    }
    if let Some(ulog_baud) = options.ulog_baud {
        config.ulog_baud = Some(ulog_baud);
    }

    if let Some(spic_calibcfg) = &options.spic_setting {
        config.spic_calibcfg = Some(DataArray::new(spic_calibcfg.clone())?);
    }

    if let Some(bt_parameter_data) = &options.bt_parameter_data {
        config.bt_parameter_data = Some(DataArray::new(bt_parameter_data.clone())?);
    }

    if let Some(outfile_path) = &options.output.file {
        let mut outfp = std::fs::File::create(outfile_path)?;
        let sysdata: SystemData = config.try_into()?;
        transfer_to(&sysdata, &mut outfp)?;
        debug!(cli, "System data written to: {:#?}", outfile_path.display());
    } else {
        error!("{}", "No output file specified.");
    }
    Ok(())
}
