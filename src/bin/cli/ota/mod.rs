use super::{Cli, OtaSubCommand};
use amebazii::error::Error;
use amebazii::map::{
    AddressRange, DTCM_RAM, PSRAM, RAM_FUN_TABLE, RAM_IMG_SIGN, VECTORS_RAM, XIP_FLASH_C,
    XIP_FLASH_P,
};

mod dump;
mod parse;
mod relink;

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
        Some(OtaSubCommand::Parse { file }) => parse::parse(cli, file.clone().unwrap())?,
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
        Some(OtaSubCommand::Relink {
            file,
            outfile,
            save_intermediate,
            cap_length,
            ram_vector_start,
            ram_vector_end,
            ram_func_table_start,
            ram_func_table_end,
            ram_img_signature,
            ram_img_signature_end,
            ram_code_text,
            ram_code_text_end,
            dtcm_ram,
            dtcm_ram_end,
            xip_c_start,
            xip_c_end,
            xip_p_start,
            xip_p_end,
        }) => {
            let options = relink::Options {
                infile: file.clone().unwrap(),
                outfile: outfile.clone().unwrap(),
                save_intermediate: save_intermediate.clone(),
                cap_length: *cap_length,
                ram_vector: get_address_range(ram_vector_start, ram_vector_end, &VECTORS_RAM),
                ram_func_table: get_address_range(
                    ram_func_table_start,
                    ram_func_table_end,
                    &RAM_FUN_TABLE,
                ),
                ram_img_signature: get_address_range(
                    ram_img_signature,
                    ram_img_signature_end,
                    &RAM_IMG_SIGN,
                ),
                ram_text: get_address_range(ram_code_text, ram_code_text_end, &DTCM_RAM),
                psram_text: get_address_range(dtcm_ram, dtcm_ram_end, &PSRAM),
                xip_c_text: get_address_range(xip_c_start, xip_c_end, &XIP_FLASH_C),
                xip_p_text: get_address_range(xip_p_start, xip_p_end, &XIP_FLASH_P),
            };
            relink::relink(&cli, &options)?;
        }
        _ => (),
    }
    Ok(())
}
