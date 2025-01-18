use amebazii::types::{from_stream, FromStream, OTAImage};

#[allow(unused_variables)]
fn main() {
    // 1. open file or create input stream
    let mut f = std::fs::File::open("assets/fw1.bin").unwrap();

    // APPROACH I
    // use FromStream trait to parse OTAImage
    let ota: OTAImage = from_stream(&mut f).unwrap();

    // APPROACH II
    // otherwise, just create a new OTAImage and fill it
    let mut image = OTAImage::default();
    image.read_from(&mut f).unwrap();
}
