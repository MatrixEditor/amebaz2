use colored::{Color, Colorize};
use std::{fs, path::PathBuf};

use crate::cli::{debug, error};

use super::Cli;

pub fn open_file(cli: &Cli, file: PathBuf, file_type: Option<&str>) -> Result<fs::File, ()> {
    if cli.verbose > 2 {
        debug!(cli, "Reading file: {:#?}", file.display());
    }

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
