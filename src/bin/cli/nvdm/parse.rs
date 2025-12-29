use colored::Colorize;
use itertools::Itertools;
use pretty_hex::*;

use crate::cli::{nvdm::ParseOptions, util, Cli};
use amebazii::types::{DataItemStatus, FromStream, NVDM, NVDM_PORT_PEB_SIZE};

pub fn parse(cli: &Cli, options: &ParseOptions) -> Result<(), amebazii::error::Error> {
    let cfg = HexConfig {
        title: false,
        ..HexConfig::default()
    };

    if let Some(input_file) = &options.file {
        let file_reader = util::open_file(cli, input_file.clone(), None);
        if file_reader.is_err() {
            return Ok(());
        }

        let mut fp = file_reader.unwrap();
        let mut nvdm = NVDM::from_peb_size(options.block_size.unwrap_or(NVDM_PORT_PEB_SIZE));
        nvdm.read_from(&mut fp)?;

        let groups = nvdm.get_groups();
        if options.groups {
            println!("Groups:");
            for (idx, group) in groups.iter().sorted().enumerate() {
                println!("  [{}] {}", idx, group.bold());
            }
            return Ok(());
        }

        for group in groups.iter().sorted() {
            if let Some(target) = &options.group {
                if target != group {
                    continue;
                }
            }

            let mut group_items =
                nvdm.get_items_by_group(group, amebazii::types::DataItemStatus::Valid);
            if !options.only_valid {
                group_items.extend(
                    nvdm.get_items_by_group(*group, amebazii::types::DataItemStatus::Delete),
                );
            }

            group_items.sort_by(|x, y| {
                x.item_header()
                    .index
                    .partial_cmp(&y.item_header().index)
                    .unwrap()
            });
            println!("Group {}:", group.bold());
            for item in group_items {
                if let Some(target) = &options.item_name {
                    if item.name() != target {
                        continue;
                    }
                }

                print!(
                    "  - [{}] {} ({})",
                    item.item_header().index,
                    item.name().bold(),
                    match item.item_header().status {
                        DataItemStatus::Delete => "deleted".red(),
                        DataItemStatus::Valid => "valid".green(),
                        DataItemStatus::Writing => "writing".yellow(),
                        _ => "invalid".bright_black(),
                    }
                );
                let item_data = item.data();
                if item_data.len() >= 1 && item_data[0] != 0 {
                    let hex_str = format!("{:?}\n", &item.data().hex_conf(cfg));
                    println!("\n{}", textwrap::indent(&hex_str, "    "));
                } else {
                    println!(" (empty)")
                }
            }
        }
    }
    Ok(())
}
