mod qoi;
mod qomf;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::time::Instant;

use image::codecs::png::PngDecoder;
use image::open;
use image::DynamicImage;
use qoi::Qoi;

use crate::qoi::decode;
use crate::qoi::encode;

fn main() {
    let path = "qoi_test_images/testcard.png";
    let t0 = Instant::now();
    let img = read_image(path);
    let t1 = Instant::now();
    let png_decode_us = t0.elapsed().as_micros();
    let encoded = encode(img);
    let qoi_encode_us = t1.elapsed().as_micros();
    save_qoi("processed.qoi", &encoded);
    // let encoded = read_qoi("qoi_test_images/qoi_logo.qoi");
    println!("###");
    let t2 = Instant::now();
    let decoded = decode(encoded);
    let qoi_decode_us = t2.elapsed().as_micros();
    let t3 = Instant::now();
    save_image("processed.png", &decoded);
    let png_encode_us = t3.elapsed().as_micros();
    println!("encoding png = {}us", png_encode_us);
    println!("decoding png = {}us", png_decode_us);
    println!(
        "encoding qoi = {}us (x{})",
        qoi_encode_us,
        qoi_encode_us as f32 / png_encode_us as f32
    );
    println!(
        "decoding qoi = {}us (x{})",
        qoi_decode_us,
        qoi_decode_us as f32 / png_decode_us as f32
    );
}

fn read_image(path: &str) -> DynamicImage {
    let buf = read_vec(path);
    if buf[42] == 42 {
        println!("what luck!")
    }
    let t0 = Instant::now();
    let img = DynamicImage::from_decoder(PngDecoder::new(buf.as_slice()).unwrap()).unwrap();
    println!("png decode = {}us", t0.elapsed().as_micros());
    return img;
    // return open(path).unwrap();
}

fn save_image(path: &str, image: &DynamicImage) {
    image.save(path).unwrap();
}

fn read_qoi(path: &str) -> Qoi {
    return Qoi {
        buf: read_vec(path),
    };
}

fn read_vec(path: &str) -> Vec<u8> {
    let mut file = match File::open(path) {
        Err(why) => panic!("{}", why),
        Ok(file) => file,
    };

    let mut buf = Vec::<u8>::new();
    match file.read_to_end(&mut buf) {
        Err(why) => panic!("{}", why),
        Ok(_) => (),
    }
    return buf;
}

fn save_qoi(path: &str, encoded: &Qoi) {
    save_vec(path, &encoded.buf);
}

fn save_vec(path: &str, buf: &Vec<u8>) {
    let mut file = match File::create(path) {
        Err(why) => panic!("{}", why),
        Ok(file) => file,
    };

    match file.write_all(buf) {
        Err(why) => panic!("{}", why),
        Ok(_) => (),
    }
}
