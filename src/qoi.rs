use image::{DynamicImage, Rgba, RgbaImage};

pub struct Qoi {
    buf: Vec<u8>,
    // img: DynamicImage,
    img: RgbaImage,
    qoi_header: QoiHeader,
}

struct QoiHeader {
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
}

pub fn encode(bitmap: DynamicImage) -> Qoi {
    let buf = bitmap.into_rgba8();

    let mut encode_stream: Vec<u8> = Vec::new();

    let header = QoiHeader {
        width: buf.width(),
        height: buf.height(),
        channels: 4,
        colorspace: 0,
    };
    for ele in encode_header(&header) {
        encode_stream.push(ele);
    }

    let prev_pixel = Rgba::from([0, 0, 0, 255]);
    // let previous: [[u8; 4]; 64] = [[0, 0, 0, 255]; 64];
    let previous: [Rgba<u8>; 64] = [Rgba::from([0, 0, 0, 255]); 64];

    let mut run = 0;

    for pixel in buf.pixels() {
        let idx = index_position(pixel);

        if prev_pixel.eq(pixel) {
            // continue run
            run += 1;
            if run == 62 {
                // stop max length run
                encode_stream.push(chunk_qoi_op_run(run));
                run = 0;
            }
            continue;
        } else if run > 0 {
            // stop current run
            encode_stream.push(chunk_qoi_op_run(run));
            run = 0;
        }

        if pixel.eq(&previous[usize::from(idx)]) {
            // reference index
            encode_stream.push(chunk_qoi_op_index(idx));
            continue;
        }

        if pixel.0[3] == prev_pixel.0[3] {
            // alpha is unchanged
            let [dr, dg, db] = difference(pixel, &prev_pixel);
            if (dr + 2) < 4 && (dg + 2) < 4 && (db + 2) < 4 {
                // diff
                encode_stream.push(chunk_qoi_op_diff(dr, dg, db));
                continue;
            }

            if (dg + 32) < 64 && (dr - dg + 8) < 16 && (db - dg + 8) < 16 {
                // luma
                for ele in chunk_qoi_op_luma(dr, dg, db) {
                    encode_stream.push(ele);
                }
                continue;
            }

            // rgb
            for ele in chunk_qoi_op_rgb(pixel) {
                encode_stream.push(ele);
            }
            continue;
        }

        // rgba
        for ele in chunk_qoi_op_rgba(pixel) {
            encode_stream.push(ele);
        }
    }

    let footer: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
    for ele in footer {
        encode_stream.push(ele);
    }

    let result: Qoi = Qoi {
        buf: encode_stream,
        img: buf,
        qoi_header: header,
    };
    return result;
}

pub fn decode(encoded_image: Qoi) -> DynamicImage {
    // return encoded_image.img;
    return DynamicImage::from(encoded_image.img);
}

fn index_position(pixel: &Rgba<u8>) -> u8 {
    let [r, g, b, a] = pixel.0;
    return ((r * 3 + g * 5 + b * 7 + a * 11) % 64).into();
}

fn difference(pixel: &Rgba<u8>, prev_pixel: &Rgba<u8>) -> [u8; 3] {
    let [r, g, b, ..] = pixel.0;
    let [pr, pg, pb, ..] = prev_pixel.0;
    return [r - pr, g - pg, b - pb];
}

fn encode_header(header: &QoiHeader) -> Vec<u8> {
    let magic: [u8; 4] = [0x71, 0x6F, 0x69, 0x66];
    let width = transform_u32_to_array_of_u8(header.width);
    let height = transform_u32_to_array_of_u8(header.height);
    let foo: [&[u8]; 5] = [
        &magic,
        &width,
        &height,
        &[header.channels],
        &[header.colorspace],
    ];
    return foo.concat();
}

fn chunk_qoi_op_rgb(pixel: &Rgba<u8>) -> [u8; 4] {
    let [r, g, b, ..] = pixel.0;
    return [0b11111110, r, g, b];
}

fn chunk_qoi_op_rgba(pixel: &Rgba<u8>) -> [u8; 5] {
    let [r, g, b, a] = pixel.0;
    return [0b11111111, r, g, b, a];
}

fn chunk_qoi_op_index(idx: u8) -> u8 {
    return idx;
}

fn chunk_qoi_op_diff(dr: u8, dg: u8, db: u8) -> u8 {
    return 0b01000000 | ((dr + 2) << 4) | ((dg + 2) << 2) | (db + 2);
}

fn chunk_qoi_op_luma(dr: u8, dg: u8, db: u8) -> [u8; 2] {
    return [0b10000000 | dg + 32, ((dr - dg + 8) << 4) | (db - dg + 8)];
}

fn chunk_qoi_op_run(run: u8) -> u8 {
    return 0b11000000 | run - 1;
}

fn transform_u32_to_array_of_u8(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4];
}
