use amebazii::{
    keys::HASH_KEY,
    types::{
        set_default_segment_size, set_default_signature, transfer_to, BootImage, Flash,
        PartitionTableImage, SystemData,
    },
};

fn main() {
    // create empty flash image
    let mut flash = Flash::default();

    // The flash stores its partitions within a map, indexed by their
    // PartitionType. Each partition can be represented by either raw
    // bytes or a corresponding image type.

    // For example, lets create a boot image
    let mut boot = BootImage::default();
    // specify load address and entry address. The length will be set +
    // automatically later on
    boot.entry.load_address = 0x8000_0000;

    // executable code should be copied from object file
    let boot_text = Vec::new();
    boot.set_text(boot_text);

    // make sure signature is set
    set_default_segment_size(&mut boot);
    set_default_signature(&mut boot, Some(HASH_KEY)).unwrap();

    // now, add the boot image to the flash
    flash.set_boot_partition(boot);
    // or
    // flash.set_partition(PartitionType::Boot, Partition::Bootloader(boot));

    // the only required partitions are partition table and system data
    // see build_pt.rs for details on how to build a partition table
    let mut pt_image = PartitionTableImage::default();
    set_default_segment_size(&mut pt_image);
    set_default_signature(&mut pt_image, Some(HASH_KEY)).unwrap();
    flash.set_partition_table(pt_image);
    flash.set_system_partition(SystemData::default());

    // now, write the flash image to a file
    let mut f = std::fs::File::create("assets/flash.bin").unwrap();
    transfer_to(&flash, &mut f).unwrap();
}
