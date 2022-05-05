mod qoi;

use image::open;
use image::DynamicImage;

use crate::qoi::decode;
use crate::qoi::encode;

fn main() {
    println!("Hello, world!");
    let img = read_image("qoi_test_images/testcard.png");
    let encoded = encode(img);
    let decoded = decode(encoded);
    save_image("testcard.processed.png", &decoded)
}

fn read_image(path: &str) -> DynamicImage {
    return open(path).unwrap();
}

fn save_image(path: &str, image: &DynamicImage) {
    image.save(path).unwrap();
}
