use clap::Parser;
use std::path::PathBuf;

use super::{headings, Cli, OtaSubCommand};
use amebazii::error::Error;
use amebazii::map::{
    AddressRange, DTCM_RAM, PSRAM, RAM_FUN_TABLE, RAM_IMG_SIGN, VECTORS_RAM, XIP_FLASH_C,
    XIP_FLASH_P,
};

mod dump;
mod parse;
mod relink;
mod resign;

#[derive(Parser)]
pub struct RelinkOptions {
    /// The input firmware file to be relinked.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// The output firmware file after relinking.
    #[arg(value_name = "OUTFILE")]
    outfile: Option<PathBuf>,

    /// The directory where intermediate files will be saved (section data).
    #[arg(short, long, value_name = "DIR", value_hint = clap::ValueHint::DirPath)]
    save_intermediate: Option<PathBuf>,

    /// Cap the length of the output sections based on available data (i.e. ignores errors)
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    cap_length: bool,

    /// Start address of the RAM vector table.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    ram_vector_start: Option<u64>,

    /// End address of the RAM vector table.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    ram_vector_end: Option<u64>,

    /// Start address of the RAM function entry table.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    ram_func_table_start: Option<u64>,

    /// End address of the RAM function entry table.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    ram_func_table_end: Option<u64>,

    /// Start address of the RAM image signature.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    ram_img_signature: Option<u64>,

    /// End address of the RAM image signature.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    ram_img_signature_end: Option<u64>,

    /// Start address of the program code and text sections in RAM.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    ram_code_text: Option<u64>,

    /// End address of the program code and text sections in RAM.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    ram_code_text_end: Option<u64>,

    /// Start address of the DTCM (Data Tightly Coupled Memory) RAM region.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    dtcm_ram: Option<u64>,

    /// End address of the DTCM RAM region.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    dtcm_ram_end: Option<u64>,

    /// Start address of the XIP encrypted section in flash.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    xip_c_start: Option<u64>,

    /// End address of the XIP encrypted section in flash.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    xip_c_end: Option<u64>,

    /// Start address of the XIP plaintext section in flash.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    xip_p_start: Option<u64>,

    /// End address of the XIP plaintext section in flash.
    #[arg(long, value_name = "ADDR", help_heading = headings::ADDRESS_OPTIONS)]
    xip_p_end: Option<u64>,
}

#[derive(Parser)]
pub struct ParseOptions {
    /// The input firmware file to be parsed.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// specifies whether the input file stores a bootloader image
    #[arg(long, action = clap::ArgAction::SetTrue)]
    boot: bool,
}

#[derive(Parser)]
pub struct ReSignOptions {
    #[command(flatten)]
    pub input: super::InputOptions,

    #[command(flatten)]
    pub output: super::OutputOptionsInPlace,

    /// The key to be used for the signing operation. (hex or file)
    #[arg(short, long, value_name = "KEY")]
    pub key: Option<String>,

    /// Flag indicating whether to use MD5 for hashing.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub use_md5: bool,

    /// Flag to indicate whether the same algorithm should be used.
    ///
    /// This flag ensures that the signing operation uses the same algorithm as the previous one.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub same_algo: bool,
}

fn get_address_range(
    start: &Option<u64>,
    end: &Option<u64>,
    default: &AddressRange,
) -> AddressRange {
    match (start, end) {
        (Some(s), Some(e)) => AddressRange::new(*s, *e),
        (Some(s), None) => AddressRange::new(*s, default.end()),
        (None, Some(e)) => AddressRange::new(default.start(), *e),
        (None, None) => default.clone(),
    }
}

pub fn main(cli: &Cli, command: Option<&OtaSubCommand>) -> Result<(), Error> {
    match command {
        Some(OtaSubCommand::Parse { options }) => {
            if let Some(options) = options {
                parse::parse(cli, options)?;
            }
        }
        Some(OtaSubCommand::Dump {
            file,
            subimage,
            outdir,
            section,
        }) => dump::dump_sections(
            cli,
            file.clone().unwrap(),
            subimage.unwrap(),
            outdir.clone().unwrap(),
            *section,
        )?,
        Some(OtaSubCommand::Resign { options }) => {
            resign::re_sign(cli, options)?;
        }
        Some(OtaSubCommand::Relink { options }) => {
            // wrapping these options is somewhat ugly
            if let Some(options) = options {
                let relink_options = relink::Options {
                    infile: options.file.clone().unwrap(),
                    outfile: options.outfile.clone().unwrap(),
                    save_intermediate: options.save_intermediate.clone(),
                    cap_length: *&options.cap_length,
                    ram_vector: get_address_range(
                        &options.ram_vector_start,
                        &options.ram_vector_end,
                        &VECTORS_RAM,
                    ),
                    ram_func_table: get_address_range(
                        &options.ram_func_table_start,
                        &options.ram_func_table_end,
                        &RAM_FUN_TABLE,
                    ),
                    ram_img_signature: get_address_range(
                        &options.ram_img_signature,
                        &options.ram_img_signature_end,
                        &RAM_IMG_SIGN,
                    ),
                    ram_text: get_address_range(
                        &options.ram_code_text,
                        &options.ram_code_text_end,
                        &DTCM_RAM,
                    ),
                    psram_text: get_address_range(&options.dtcm_ram, &options.dtcm_ram_end, &PSRAM),
                    xip_c_text: get_address_range(
                        &options.xip_c_start,
                        &options.xip_c_end,
                        &XIP_FLASH_C,
                    ),
                    xip_p_text: get_address_range(
                        &options.xip_p_start,
                        &options.xip_p_end,
                        &XIP_FLASH_P,
                    ),
                };
                relink::relink(&cli, &relink_options)?;
            }
        }
        _ => (),
    }
    Ok(())
}
