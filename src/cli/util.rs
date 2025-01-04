use colored::{Color, Colorize};
use std::{fs, path::PathBuf};

use crate::cli::{debug, error};

use super::Cli;

pub fn open_file(cli: &Cli, file: PathBuf) -> Result<fs::File, ()> {
    if cli.verbose > 2 {
        debug!(cli, "Reading file: {:#?}", file.display());
    }

    if !file.exists() {
        error!("Target file does not exist: {:#?}", file.display());
        return Err(());
    }

    if file.is_dir() {
        error!(
            "Cloud not start parsing, because {:#?} is a directory",
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
