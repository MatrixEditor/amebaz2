use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::{Cli, FlashSubCommand, headings};

mod parse;
mod split;

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct CombineOptions {

    #[arg(value_name = "DEST")]
    pub file: Option<PathBuf>,

    #[arg(short, long, value_name = "PT", required = true)]
    pub parttab: Option<PathBuf>,

    #[arg(short, long, value_name = "DIR")]
    pub srcdir: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    pub fw1: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    pub fw2: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    pub sysdata: Option<PathBuf>,


}

pub fn main(cli: &Cli, subcommand: Option<&FlashSubCommand>) -> Result<(), amebazii::error::Error> {
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
        },
        Some(FlashSubCommand::Combine { options }) => {
            match options {
                Some(options) => {
                    // combine::combine_images(cli, options)?;
                }
                None => {}
            }
        }
        _ => {}
    }
    Ok(())
}
