use colored::Colorize;
use std::{fs, path::PathBuf};

use super::{Cli, OutputOptionsExt};
use crate::cli::{debug, error};

pub fn open_file(cli: &Cli, file: PathBuf, file_type: Option<&str>) -> Result<fs::File, ()> {
    debug!(cli, "Reading file: {:#?}", file.display());

    if !file.exists() {
        error!(
            "{} file does not exist: {:#?}",
            file_type.unwrap_or("Target"),
            file.display()
        );
        return Err(());
    }

    if file.is_dir() {
        error!(
            "Cloud not open file because {:#?} is a directory",
            file.display()
        );
        return Err(());
    }

    if let Ok(fp) = fs::File::open(file.clone()) {
        Ok(fp)
    } else {
        error!("Could not open file: {:#?}", file.display());
        Err(())
    }
}

pub fn open_output_file<O: OutputOptionsExt>(
    cli: &Cli,
    input_options: Option<&super::InputOptions>,
    output_options: &O,
) -> Result<fs::File, amebazii::error::Error> {
    let mut out_path = None;
    if let Some(file) = output_options.file() {
        if file.exists() && !output_options.force() {
            return Err(amebazii::error::Error::InvalidState(format!(
                "Output file {} already exists",
                file.display()
            )));
        }
        out_path = Some(file);
    } else {
        if let Some(input) = input_options {
            if let Some(input_file) = &input.file {
                if output_options.in_place() {
                    out_path = Some(input_file.clone());
                }
            }
        }
    }

    match out_path {
        Option::Some(out_path) => {
            debug!(cli, "Writing file: {:#?}", out_path.display());
            return Ok(fs::File::options()
                .write(true)
                .create(true)
                .open(out_path)?);
        }
        Option::None => Err(amebazii::error::Error::InvalidState(
            "Output file not specified".to_string(),
        )),
    }
}
