use amebazii::{
    keys::{HASH_KEY, KEY_PAIR_003},
    types::{
        set_default_segment_size, set_default_signature, transfer_to, EncryptedOr, KeyExportOp,
        PartTab, PartitionTableImage, PartitionType, ToStream,
    },
};

fn main() {
    // create empty partition table
    let mut pt = PartTab::default();

    // configure public fields
    pt.key_exp_op = KeyExportOp::None;
    pt.eFWV = 255;

    // user data must be set using setter methods
    pt.set_user_bin(&[0x00, 0x01]);

    // now, create records
    let record = pt.new_record(PartitionType::Boot);
    record.start_addr = 0x4000;
    record.length = 0x8000;
    record.dbg_skip = false; // default is false

    // add fw record to partition table
    let record = pt.new_record(PartitionType::Fw1);
    record.start_addr = 0xC000;
    record.length = 0xF8000;
    // hash key will be used to sign firmware image
    record.set_hash_key(Some(KEY_PAIR_003.get_priv_key().clone()));

    // to write the partition table, we must create an image first
    let mut image = PartitionTableImage::default();
    image.pt = EncryptedOr::Plain(pt);

    // configure header values (except segment size)
    image.header.serial = 0x10000001;

    // before writing the image, we must set the segment size
    set_default_segment_size(&mut image);

    // partition table must be signed
    set_default_signature(&mut image, Some(HASH_KEY)).unwrap();

    // use ToStream to write the image to a file
    let mut f = std::fs::File::create("assets/partition-new.bin").unwrap();
    transfer_to(&image, &mut f).unwrap();

    // or directly
    image.write_to(&mut f).unwrap();
}
