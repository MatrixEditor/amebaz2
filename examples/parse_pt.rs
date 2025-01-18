use std::io::Seek;

use amebazii::types::{from_stream, EncryptedOr, PartitionTableImage, PartitionType};

#[allow(unused_variables)]
fn main() {
    // 1. open file or create input stream
    let mut f = std::fs::File::open("assets/partition.bin").unwrap();

    // NOTE: most flash images and generated partition tables store the
    // flash calibration pattern - we have to skip that first.
    f.seek(std::io::SeekFrom::Start(0x20)).unwrap();

    // 2. parse using FromStream trait
    let image: PartitionTableImage = from_stream(&mut f).unwrap();

    // because the partition table can be empty, we need to check if it
    // is actually encrypted
    if let EncryptedOr::Plain(pt) = image.pt {
        // partition table info can be used
        // ...

        // e.g. iterate over all records
        for record in pt.get_records() {
            // ...
        }

        // e.g. get specific record
        if let Some(record) = pt.get_record(PartitionType::Boot) {
            // ...
        }
    }
}
