mod qoi;

use std::fs::File;
use std::io::Write;

use image::open;
use image::DynamicImage;
use qoi::Qoi;

use crate::qoi::decode;
use crate::qoi::encode;

fn main() {
    println!("Hello, world!");
    let img = read_image("qoi_test_images/testcard.png");
    let encoded = encode(img);
    save_qoi("testcard.processed.qoi", &encoded);
    println!("###");
    let decoded = decode(encoded);
    save_image("testcard.processed.png", &decoded)
}

fn read_image(path: &str) -> DynamicImage {
    return open(path).unwrap();
}

fn save_image(path: &str, image: &DynamicImage) {
    image.save(path).unwrap();
}

fn save_qoi(path: &str, encoded: &Qoi) {
    let mut file = match File::create(path) {
        Err(why) => panic!("{}", why),
        Ok(file) => file,
    };

    match file.write_all(&encoded.buf) {
        Err(why) => panic!("{}", why),
        Ok(_) => (),
    }
}
