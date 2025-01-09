use colored::{Color, Colorize};
use object::build::elf::{Builder, SectionData, SectionId, Segment};
use std::fs;
use std::path::PathBuf;

use crate::map::AddressRange;
use crate::cli::{debug, error, util, Cli};
use crate::types::enums::{ImageType, SectionType};
use crate::types::image::ota::{OTAImage, SubImage};
use crate::types::{from_stream, section};

pub(super) struct Options {
    pub infile: PathBuf,
    pub outfile: PathBuf,
    pub save_intermediate: Option<PathBuf>,
    pub cap_length: bool,

    // linker options
    pub ram_vector: AddressRange,
    pub ram_func_table: AddressRange,
    pub ram_img_signature: AddressRange,
    pub ram_text: AddressRange,
    pub psram_text: AddressRange,
    pub xip_c_text: AddressRange,
    pub xip_p_text: AddressRange,
}

struct ElfData<'d> {
    pub builder: Builder<'d>,

    pub std_sections: Vec<SectionId>,
    pub ram_func_table: Option<SectionId>,
    pub ram_img_sign: Option<SectionId>,
    pub ram_text: Option<SectionId>,
    pub psram_text: Option<SectionId>,
    pub xip_c_text: Option<SectionId>,
    pub xip_p_text: Option<SectionId>,
}

pub fn relink(cli: &Cli, options: &Options) -> Result<(), crate::error::Error> {
    let fp = util::open_file(cli, options.infile.clone(), None);
    if fp.is_err() {
        return Ok(());
    }

    let mut reader = fp.unwrap();
    let image: OTAImage = from_stream(&mut reader)?;
    debug!(
        cli,
        "Parsed OTA image with {} subimages",
        image.get_subimages().len()
    );

    if let Some(outdir) = &options.save_intermediate {
        debug!(cli, "Creating directory: {}", outdir.display());
        fs::create_dir(outdir)?;
    }

    // the first subimage should be the SRAM which contains the RAM code
    let subimages = image.get_subimages();
    if subimages.len() == 0 {
        error!("{}", "No subimages found in OTA image");
        return Ok(());
    }

    let mut data = ElfData::new();
    for (i, subimage) in subimages.iter().enumerate() {
        match &subimage.header.img_type {
            ImageType::FHWSS => {
                extract_ram_from_fhwss(cli, options, &mut data, &subimage)?;
            }
            ImageType::Xip => {
                if i < 2 {
                    println!("{}: ", "XIP".bold().underline());
                }
                if let Some(section) = subimage.get_section(0) {
                    print!(
                        "  [{}] {} {:?}",
                        i - 1,
                        "Secion:".bold(),
                        section.header.sect_type
                    );

                    let elf_section = data.builder.sections.add();
                    elf_section.sh_type = object::elf::SHT_PROGBITS;
                    elf_section.sh_addralign = 1;
                    elf_section.sh_size = section.get_data().len() as u64;
                    elf_section.data = SectionData::Data(section.get_data().to_vec().into());
                    elf_section.sh_addr = section.entry_header.load_address as u64;
                    data.std_sections.push(elf_section.id());

                    if options
                        .xip_c_text
                        .contains(section.entry_header.load_address as u64)
                    {
                        // XIP Chiper section: TEXT/RODATA in this section can be encrypted (decrypt by SCE)
                        println!("_C");
                        write_section(
                            cli,
                            options,
                            ".xip.code_c",
                            &section.get_data(),
                            "XIP code cipher section",
                        )?;

                        elf_section.sh_flags =
                            (object::elf::SHF_ALLOC | object::elf::SHF_EXECINSTR) as u64;
                        elf_section.name = b".xip.code_c"[..].into();
                        data.xip_c_text = Some(elf_section.id());
                    } else {
                        /* XIP Plantext section: RODATA in this section will not be encrypted */
                        println!("_P");
                        write_section(
                            cli,
                            options,
                            ".xip.code_p",
                            &section.get_data(),
                            "XIP code plaintext section (rodata)",
                        )?;

                        elf_section.sh_flags = (object::elf::SHF_WRITE) as u64;
                        elf_section.name = b".xip.code_p"[..].into();
                        data.xip_p_text = Some(elf_section.id());
                    }
                } else {
                    error!("{}", "No sections found in XIP subimage");
                    return Ok(());
                }
            }
            _ => {
                debug!(
                    cli,
                    "Ignoring subimage type: {:?}", &subimage.header.img_type
                );
            }
        }
    }

    create_obj(cli, options, data)?;
    Ok(())
}

fn extract_ram_from_fhwss(
    cli: &Cli,
    options: &Options,
    data: &mut ElfData<'_>,
    ram_subimage: &SubImage,
) -> Result<(), crate::error::Error> {
    // sections should at least contain SRAM
    let ram_sections = ram_subimage.get_sections();
    if ram_sections.len() == 0 {
        error!("{}", "No sections found in RAM subimage");
        return Ok(());
    }

    debug!(
        cli,
        "Found {} section(s) in RAM subimage",
        ram_sections.len()
    );
    println!("{}: ", "RAM/FHWS".bold().underline());
    for (i, section) in ram_sections.iter().enumerate() {
        println!(
            "  [{}] {} {:?}",
            i,
            "Secion:".bold(),
            section.header.sect_type
        );

        match &section.header.sect_type {
            SectionType::SRAM => {
                add_sram_section(cli, &options, data, &section)?;
            }
            SectionType::PSRAM => {
                data.psram_text = Some(build_ram_section(
                    cli,
                    options,
                    data,
                    section.get_data(),
                    0,
                    &options.psram_text,
                    ".psram.code_text",
                    "PSRAM code text",
                    section.entry_header.load_address as u64,
                    (object::elf::SHF_WRITE | object::elf::SHF_EXECINSTR) as u64,
                    "__psram_code_text_start__"
                )?);
            }
            _ => {
                debug!(
                    cli,
                    "Ignoring section type: {:?}", &section.header.sect_type
                );
            }
        }
    }

    Ok(())
}

fn add_sram_section(
    cli: &Cli,
    options: &Options,
    data: &mut ElfData<'_>,
    ram_section: &section::Section,
) -> Result<(), crate::error::Error> {
    // SRAM defines the following section in our target ELF file:
    // .ram.img.signature -> __ram_img_signature__
    // .ram.func.table -> __ram_start_table_start__
    // .data -> __data_start__
    // .ram.code_text -> __ram_code_text_start__
    // .ram.code_rodata -> __ram_code_rodata_start__
    let ram = ram_section.get_data();

    // The head of the SRAM section should look something like this:
    // 00000000: f004 0010 c111 0010 8928 0010 0000 0000  .........(......
    // 00000010: 0000 0000 3830 0010 a02e 0010 c011 0010  ....80..........
    // 00000020: 1c16 0010 602f 0010 0000 0000 0000 0000  ....`/..........
    // 00000030: 0000 0000 0000 0000 0000 0000 0000 0000  ................
    // 00000040: 00fa 0310 00ea 0310 0000 0000 802f 0010  ............./..
    // 00000050: 0000 0000 0000 0000 0000 0000 0000 0000  ................
    // 00000060: 2030 0010 0000 0000 0000 0000 0000 0000   0..............
    // 00000070: 416d 6562 615a 4949 ff00 0000 0000 0000  AmebaZII........
    // 00000080: f152 809b cf09 009b 0000 0000 0000 0000  .R..............
    // 00000090: f652 809b 3d0d 009b 0000 0000 0000 0000  .R..=...........
    //
    // The first 0x60 bytes define the RAM function table (RAM_FUN_TABLE)
    // as defined in the linker script:
    //
    // RAM_FUN_TABLE (rwx)   : ORIGIN = 0x10000480, LENGTH = 0x100004F0 - 0x10000480
    let base_address = ram_section.entry_header.load_address as u64;
    let mut offset = 0;

    data.ram_func_table = Some(build_ram_section(
        cli,
        options,
        data,
        ram,
        offset,
        &options.ram_func_table,
        ".ram.func.table",
        "RAM function table",
        base_address,
        (object::elf::SHF_WRITE | object::elf::SHF_ALLOC) as u64,
        "__ram_start_table_start__"
    )?);
    offset += &options.ram_func_table.len();

    // RAM_IMG_SIGN (rwx)    : ORIGIN = 0x100004F0, LENGTH = 0x10000500 - 0x100004F0
    data.ram_img_sign = Some(build_ram_section(
        cli,
        options,
        data,
        ram,
        offset,
        &options.ram_img_signature,
        ".ram.img.signature",
        "RAM image signature",
        base_address,
        object::elf::SHF_ALLOC as u64,
        "__ram_img_signature__"
    )?);
    offset += &options.ram_img_signature.len();

    // DTCM_RAM (wrx) 		: ORIGIN = 0x10000500, LENGTH = 0x1003FA00 - 0x10000500
    data.ram_text = Some(build_ram_section(
        cli,
        options,
        data,
        ram,
        offset,
        &options.ram_text,
        ".ram.code_text",
        "DTCM RAM",
        base_address,
        (object::elf::SHF_ALLOC | object::elf::SHF_EXECINSTR) as u64,
        "__ram_code_text_start__"
    )?);
    Ok(())
}

fn build_ram_section(
    cli: &Cli,
    options: &Options,
    data: &mut ElfData<'_>,
    ram: &[u8],
    offset: u64,
    target_range: &AddressRange,
    name: &'static str,
    display_name: &str,
    base_address: u64,
    flags: u64,
    label: &'static str,
) -> Result<SectionId, crate::error::Error> {
    let mut length = target_range.len();
    if offset as usize + length as usize > ram.len() {
        if !options.cap_length {
            error!("The specified {} length is too big!", display_name);
            error!("- RAM input length: {}", ram.len());
            error!(
                "- Requested {} length: {} from offset {}",
                display_name, length, offset
            );
            return Err(crate::error::Error::InvalidState(format!(
                "{} too big!",
                display_name
            )));
        }
        length = (ram.len() - offset as usize) as u64;
    }
    let section_data = &ram[offset as usize..(offset + length) as usize];
    write_section(cli, options, name, section_data, display_name)?;
    let section_id = {
        let s = data.builder.sections.add();
        s.name = name[..].into();
        s.sh_type = object::elf::SHT_PROGBITS;
        s.sh_flags = flags;
        s.sh_size = length;
        s.sh_addr = base_address + offset as u64;
        s.sh_offset = base_address + offset as u64;
        s.data = SectionData::Data(section_data.to_vec().into());
        s.sh_addralign = 1;
        s.id()
    };

    let symbol = data.builder.symbols.add();
    symbol.name = label[..].into();
    symbol.st_value = base_address + offset as u64;
    symbol.section = Some(section_id);
    symbol.set_st_info(object::elf::STB_GLOBAL, object::elf::STT_SECTION);
    Ok(section_id)
}

fn write_section(
    cli: &Cli,
    options: &Options,
    name: &str,
    data: &[u8],
    display_name: &str,
) -> Result<(), crate::error::Error> {
    print!("{}- {}... ", " ".repeat(6), display_name.italic());
    if let Some(outdir) = &options.save_intermediate {
        let section_file = outdir.join(name);

        fs::write(&section_file, &data)?;
        println!("{}", "OK".green());
        debug!(
            cli,
            "Wrote section {} to: {:?}",
            display_name,
            section_file.display()
        );
    } else {
        println!("{}", "OK".green());
    }
    Ok(())
}

fn _print_segment_info(segment: &Segment<'_>, flags: &str, info: &str) {
    println!(
        "  {:<6}0x{:06x} 0x{:08x} 0x{:08x} 0x{:05x} 0x{:05x} {:<3} 0x{:05x} {}",
        "LOAD".italic(),
        segment.p_offset,
        segment.p_vaddr,
        segment.p_paddr,
        segment.p_filesz,
        segment.p_memsz,
        flags,
        segment.p_align,
        info
    );
}

// TODO: rewrite code below and make it readable
fn create_obj(
    cli: &Cli,
    options: &Options,
    mut data: ElfData<'_>,
) -> Result<(), crate::error::Error> {
    debug!(cli, "Creating ELF file");

    let ram_vector_table = build_ram_section(
        cli,
        options,
        &mut data,
        &vec![0x00; options.ram_vector.len() as usize],
        0,
        &options.ram_vector,
        ".ram.vector_table",
        "RAM vector table",
        options.ram_vector.start(),
        0,
        "__ram_vector_table_start__"
    )?;

    data.builder.set_section_sizes();

    println!("\n{} ", "Program Headers:".bold().underline());
    println!(
        "{}",
        "  Type  Offset   VirtAddr   PhysAddr   FileSiz MemSiz  Flg Align   Info".bold()
    );

    let std_segment = data
        .builder
        .segments
        .add_load_segment(object::elf::PF_R, data.builder.load_align);
    std_segment.p_align = 0x10000;
    for section_id in &data.std_sections {
        std_segment.append_section(data.builder.sections.get_mut(*section_id));
    }
    _print_segment_info(&std_segment, "R", "Standard sections");

    let mut offset = 0x10000;
    // insert ram vector table
    let segment = data
        .builder
        .segments
        .add_load_segment(object::elf::PF_R, data.builder.load_align);
    segment.p_filesz = 0;
    segment.p_memsz = 0;
    segment.p_align = 0x10000;
    segment.p_vaddr = options.ram_vector.start();
    segment.p_paddr = options.ram_vector.start();
    segment.p_offset = offset;
    segment.append_section(data.builder.sections.get_mut(ram_vector_table));
    _print_segment_info(&segment, "RW", "RAM vector table");

    offset += 0x480;

    if let Some(ram_func_table) = &data.ram_func_table {
        let segment = data
            .builder
            .segments
            .add_load_segment(object::elf::PF_R | object::elf::PF_W, 0x10000);
        segment.p_vaddr = options.ram_func_table.start();
        segment.p_paddr = options.ram_func_table.start();
        segment.p_offset = offset;
        segment.append_section(data.builder.sections.get_mut(*ram_func_table));
        _print_segment_info(segment, "RW", "RAM function table");

        offset += &options.ram_func_table.len();
    }

    if let Some(ram_img_sign) = &data.ram_img_sign {
        let segment = data
            .builder
            .segments
            .add_load_segment(object::elf::PF_R, 0x10000);
        segment.p_vaddr = options.ram_img_signature.start();
        segment.p_paddr = options.ram_img_signature.start();
        segment.p_offset = offset;
        segment.append_section(data.builder.sections.get_mut(*ram_img_sign));
        _print_segment_info(&segment, "R", "RAM image signature");
        offset += &options.ram_img_signature.len();
    }

    if let Some(ram_code_text) = &data.ram_text {
        let segment = data.builder.segments.add_load_segment(
            object::elf::PF_R | object::elf::PF_W | object::elf::PF_X,
            0x10000,
        );
        segment.p_vaddr = options.ram_text.start();
        segment.p_paddr = options.ram_text.start();
        segment.p_offset = offset;
        segment.append_section(data.builder.sections.get_mut(*ram_code_text));
        _print_segment_info(&segment, "RWE", "RAM text and rodata");
        offset += &options.ram_text.len();
    }

    if let Some(psram_code_text) = &data.psram_text {
        let segment = data.builder.segments.add_load_segment(
            object::elf::PF_R | object::elf::PF_W | object::elf::PF_X,
            0x10000,
        );
        segment.p_vaddr = options.psram_text.start();
        segment.p_paddr = options.psram_text.start();
        segment.p_offset = offset;
        segment.append_section(data.builder.sections.get_mut(*psram_code_text));
        _print_segment_info(&segment, "RWE", "PSRAM data, code text and rodata");
        offset += &options.psram_text.len();
    }

    if let Some(xip_c_text) = &data.xip_c_text {
        let segment = data.builder.segments.add_load_segment(
            object::elf::PF_R | object::elf::PF_W | object::elf::PF_X,
            0x10000,
        );
        segment.p_vaddr = options.xip_c_text.start();
        segment.p_paddr = options.xip_c_text.start();
        segment.p_offset = offset;
        segment.append_section(data.builder.sections.get_mut(*xip_c_text));
        _print_segment_info(&segment, "RWE", "XIP.C rodata, text");
        offset += &options.xip_c_text.len();
    }

    if let Some(xip_p_text) = &data.xip_p_text {
        let segment = data
            .builder
            .segments
            .add_load_segment(object::elf::PF_R | object::elf::PF_W, 0x10000);
        segment.p_vaddr = options.xip_p_text.start();
        segment.p_paddr = options.xip_p_text.start();
        segment.p_offset = offset;
        segment.append_section(data.builder.sections.get_mut(*xip_p_text));
        _print_segment_info(&segment, "RW", "XIP.P rodata, text");
        // not necessary
        // offset += &options.xip_p_text.len();
    }

    let mut buffer = Vec::new();
    data.builder.write(&mut buffer).unwrap();
    debug!(cli, "Writing ELF file to: {}", options.outfile.display());
    fs::write(&options.outfile, &buffer)?;

    Ok(())
}

impl ElfData<'_> {
    pub fn new() -> Self {
        let mut data = ElfData {
            builder: Builder::new(object::Endianness::Little, false),
            std_sections: Vec::new(),
            ram_func_table: None,
            ram_img_sign: None,
            ram_text: None,
            psram_text: None,
            xip_c_text: None,
            xip_p_text: None,
        };

        data.builder.header.e_type = object::elf::ET_EXEC;
        data.builder.header.e_machine = object::elf::EM_ARM;
        data.builder.header.e_phoff = 52;
        data.builder.header.e_flags =
            object::elf::EF_ARM_EABI_VER5 | object::elf::EF_ARM_SOFT_FLOAT;

        // build standard sections
        let section = data.builder.sections.add();
        section.name = b".shstrtab"[..].into();
        section.sh_type = object::elf::SHT_STRTAB;
        section.data = SectionData::SectionString;
        section.sh_addralign = 1;
        data.std_sections.push(section.id());

        data.std_sections.push(section.id());
        let section = data.builder.sections.add();
        section.name = b".symtab"[..].into();
        section.sh_type = object::elf::SHT_SYMTAB;
        section.sh_addralign = 8;
        section.data = SectionData::Symbol;
        data.std_sections.push(section.id());

        let section = data.builder.sections.add();
        section.name = b".strtab"[..].into();
        section.sh_type = object::elf::SHT_STRTAB;
        section.sh_addralign = 1;
        section.data = SectionData::String;
        data.std_sections.push(section.id());

        return data;
    }
}
