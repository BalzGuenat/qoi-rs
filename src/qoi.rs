use std::num::Wrapping;

use image::{DynamicImage, ImageBuffer, Rgba};
use wrapping_arithmetic::wrappit;

pub struct Qoi {
    pub buf: Vec<u8>,
    // img: DynamicImage,
    // img: RgbaImage,
    // qoi_header: QoiHeader,
}

struct QoiHeader {
    magic: [u8; 4],
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
}

#[wrappit]
pub fn encode(bitmap: DynamicImage) -> Qoi {
    let buf = bitmap.into_rgba8();

    let mut encode_stream: Vec<u8> = Vec::new();

    let header = QoiHeader {
        magic: *b"qoif",
        width: buf.width(),
        height: buf.height(),
        channels: 4,
        colorspace: 0,
    };
    for ele in encode_header(&header) {
        encode_stream.push(ele);
    }

    let mut prev_pixel: Rgba<u8> = Rgba::from([0, 0, 0, 255]);
    // let previous: [[u8; 4]; 64] = [[0, 0, 0, 255]; 64];
    let mut previous: [Rgba<u8>; 64] = [Rgba::from([0, 0, 0, 255]); 64];

    let mut run = 0u8;

    let mut num_chunks: u32 = 0;

    for pixel in buf.pixels() {
        // println!("next pixel");
        let idx = index_position(pixel);

        if prev_pixel.eq(pixel) {
            // continue run
            run += 1;
            if run == 62 {
                // stop max length run
                encode_stream.push(chunk_qoi_op_run(run));
                num_chunks += 1;
                println!("{} run ({})", num_chunks, run);
                run = 0;
            }
            continue;
        } else if run > 0 {
            // stop current run
            encode_stream.push(chunk_qoi_op_run(run));
            num_chunks += 1;
            println!("{} run ({})", num_chunks, run);
            run = 0;
        }

        if pixel.eq(&previous[usize::from(idx)]) {
            // reference index
            encode_stream.push(chunk_qoi_op_index(idx));
            num_chunks += 1;
            println!("{} index", num_chunks);

            prev_pixel = *pixel;
            continue;
        }

        previous[usize::from(idx)] = pixel.clone();

        if pixel.0[3] == prev_pixel.0[3] {
            // alpha is unchanged
            let [dr, dg, db] = difference(pixel, &prev_pixel);

            prev_pixel = *pixel;

            if (dr + 2) < 4 && (dg + 2) < 4 && (db + 2) < 4 {
                // diff
                encode_stream.push(chunk_qoi_op_diff(dr, dg, db));
                num_chunks += 1;
                println!("{} diff", num_chunks);
                continue;
            }

            if (dg + 32) < 64 && (dr - dg + 8) < 16 && (db - dg + 8) < 16 {
                // luma
                for ele in chunk_qoi_op_luma(dr, dg, db) {
                    encode_stream.push(ele);
                }
                num_chunks += 1;
                println!("{} luma", num_chunks);
                continue;
            }

            // rgb
            for ele in chunk_qoi_op_rgb(pixel) {
                encode_stream.push(ele);
            }
            num_chunks += 1;
            println!("{} rgb", num_chunks);
            continue;
        }

        prev_pixel = *pixel;

        // rgba
        for ele in chunk_qoi_op_rgba(pixel) {
            encode_stream.push(ele);
        }
        num_chunks += 1;
        println!("{} rgba", num_chunks);
    }

    if run > 0 {
        // stop current run
        encode_stream.push(chunk_qoi_op_run(run));
        num_chunks += 1;
        println!("{} run ({})", num_chunks, run);
    }

    println!("num_chunks = {}", num_chunks);

    let footer: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
    for ele in footer {
        encode_stream.push(ele);
    }

    let result: Qoi = Qoi {
        buf: encode_stream,
        // img: buf,
        // qoi_header: header,
    };
    return result;
}

#[wrappit]
pub fn decode(encoded_image: Qoi) -> DynamicImage {
    let header = decode_header(&encoded_image.buf);
    // let image = RgbaImage::new(header.width, header.height);
    // let buf = Vec<Rbga<u8>
    // let mut buf: Vec<Rgba<u8>> = Vec::with_capacity((header.width * header.height).try_into().unwrap());
    let mut image = ImageBuffer::new(header.width, header.height);
    let mut processed_pixels = 0;

    // let mut prev_pixel = Rgba::from([Wrapping(0u8), Wrapping(0), Wrapping(0), Wrapping(255)]);
    let mut prev_pixel: Rgba<u8> = Rgba::from([0, 0, 0, 255]);
    let mut previous = [prev_pixel; 64];

    let mut buf_iter = encoded_image.buf[14..].iter();
    let mut pixel_iter = image.pixels_mut();

    let mut num_chunks: u32 = 0;

    while processed_pixels < header.width * header.height {
        num_chunks += 1;
        println!(
            "next chunk {} ({}/{} pixels)",
            num_chunks,
            processed_pixels,
            header.width * header.height
        );
        let b1 = *buf_iter.next().unwrap();

        if b1 >> 6 == 0b00 {
            // index
            println!("{} index", num_chunks);
            let idx = b1 & 0b0011_1111;
            let px = pixel_iter.next().unwrap();
            prev_pixel = previous[usize::from(idx)];
            *px = prev_pixel;
            processed_pixels += 1;
            continue;
        }

        if b1 >> 6 == 0b01 {
            // diff
            println!("{} diff", num_chunks);
            let dr = ((b1 >> 4) & 0b11) - 2;
            let dg = ((b1 >> 2) & 0b11) - 2;
            let db = (b1 & 0b11) - 2;

            let [pr, pg, pb, pa] = prev_pixel.0;
            prev_pixel.0 = [pr + dr, pg + dg, pb + db, pa];
            let idx = index_position(&prev_pixel);
            previous[usize::from(idx)] = prev_pixel;
            let px = pixel_iter.next().unwrap();
            *px = prev_pixel;
            processed_pixels += 1;
            continue;
        }

        if b1 >> 6 == 0b10 {
            // luma
            println!("{} luma", num_chunks);
            let dg = (b1 & 0b0011_1111) - 32;

            let b2 = *buf_iter.next().unwrap();
            let dr_dg = ((b2 >> 4) & 0b1111) - 8;
            let db_dg = (b2 & 0b1111) - 8;

            let dr = dr_dg + dg;
            let db = db_dg + dg;

            let [pr, pg, pb, pa] = prev_pixel.0;
            prev_pixel.0 = [pr + dr, pg + dg, pb + db, pa];
            let idx = index_position(&prev_pixel);
            previous[usize::from(idx)] = prev_pixel;
            let px = pixel_iter.next().unwrap();
            *px = prev_pixel;
            processed_pixels += 1;
            continue;
        }

        if (b1 & 0b1111_1110) != 0b1111_1110 {
            // run
            let run = (b1 & 0b0011_1111) + 1;
            println!("{} run ({})", num_chunks, run);
            for i in 0..run {
                let px;
                match pixel_iter.next() {
                    Some(x) => px = x,
                    None => panic!("failed on pixel {}", processed_pixels + i as u32),
                }
                // let px = pixel_iter
                //     .next()
                //     .unwrap_or_else(|| );
                *px = prev_pixel;
            }
            processed_pixels += run as u32;
            continue;
        }

        if b1 == 0b1111_1110 {
            // rgb
            println!("{} rgb", num_chunks);
            let r = *buf_iter.next().unwrap();
            let g = *buf_iter.next().unwrap();
            let b = *buf_iter.next().unwrap();

            let [_, _, _, pa] = prev_pixel.0;
            prev_pixel.0 = [r, g, b, pa];
            let idx = index_position(&prev_pixel);
            previous[usize::from(idx)] = prev_pixel;
            let px = pixel_iter.next().unwrap();
            *px = prev_pixel;
            processed_pixels += 1;
            continue;
        }

        if b1 == 0b1111_1111 {
            // rgba
            println!("{} rgba", num_chunks);
            let r = *buf_iter.next().unwrap();
            let g = *buf_iter.next().unwrap();
            let b = *buf_iter.next().unwrap();
            let a = *buf_iter.next().unwrap();

            prev_pixel.0 = [r, g, b, a];
            let idx = index_position(&prev_pixel);
            previous[usize::from(idx)] = prev_pixel;
            let px = pixel_iter.next().unwrap();
            *px = prev_pixel;
            processed_pixels += 1;
            continue;
        }
    }

    for _ in 0..7 {
        let b = *buf_iter.next().unwrap();
        if b != 0 {
            panic!("incomplete end marker")
        }
    }
    let b = *buf_iter.next().unwrap();
    if b != 1 {
        panic!("incomplete end marker")
    }

    match buf_iter.next() {
        Some(_) => panic!("more input after end marker"),
        None => (),
    }

    // let image = ImageBuffer::from_vec(header.width, header.height, buf).unwrap();
    return image::DynamicImage::ImageRgba8(image);
}

fn index_position(pixel: &Rgba<u8>) -> u8 {
    let [r, g, b, a] = pixel.0;
    let [wr, wg, wb, wa] = [Wrapping(r), Wrapping(g), Wrapping(b), Wrapping(a)];
    return ((wr * Wrapping(3) + wg * Wrapping(5) + wb * Wrapping(7) + wa * Wrapping(11))
        % Wrapping(64))
    .0;
}

#[wrappit]
fn difference(pixel: &Rgba<u8>, prev_pixel: &Rgba<u8>) -> [u8; 3] {
    let [r, g, b, ..] = pixel.0;
    let [pr, pg, pb, ..] = prev_pixel.0;
    return [r - pr, g - pg, b - pb];
}

fn encode_header(header: &QoiHeader) -> Vec<u8> {
    // let magic: [u8; 4] = [0x71, 0x6F, 0x69, 0x66];
    // let magic = header.magic;
    let width = transform_u32_to_array_of_u8(header.width);
    let height = transform_u32_to_array_of_u8(header.height);
    let foo: [&[u8]; 5] = [
        &header.magic,
        &width,
        &height,
        &[header.channels],
        &[header.colorspace],
    ];
    return foo.concat();
}

fn decode_header(buf: &Vec<u8>) -> QoiHeader {
    // let chunks = buf.chunks_exact(4);
    if buf.len() < 22 {
        panic!("buffer is too short for header and end marker");
    }
    // let (chunks, remainder) = buf.as_chunks::<4>();
    // let magicc = chunks.next().unwrap();
    let magic = &buf[0..4];
    if magic != b"qoif" {
        panic!("incorrect magic")
    }

    let width_bytes = buf[4..8].try_into().unwrap();
    let width = transform_array_of_u8_to_u32(width_bytes);

    let height_bytes = buf[4..8].try_into().unwrap();
    let height = transform_array_of_u8_to_u32(height_bytes);

    return QoiHeader {
        magic: *b"qoif",
        width,
        height,
        channels: 4,
        colorspace: 0,
    };
}

#[wrappit]
fn chunk_qoi_op_rgb(pixel: &Rgba<u8>) -> [u8; 4] {
    let [r, g, b, ..] = pixel.0;
    return [0b1111_1110, r, g, b];
}

#[wrappit]
fn chunk_qoi_op_rgba(pixel: &Rgba<u8>) -> [u8; 5] {
    let [r, g, b, a] = pixel.0;
    return [0b1111_1111, r, g, b, a];
}

#[wrappit]
fn chunk_qoi_op_index(idx: u8) -> u8 {
    if idx >= 64 {
        panic!("idx {} too large", idx);
    }
    return idx;
}

#[wrappit]
fn chunk_qoi_op_diff(dr: u8, dg: u8, db: u8) -> u8 {
    if dr + 2 >= 4 || dg + 2 >= 4 || db + 2 >= 4 {
        panic!("diff too large");
    }
    let chunk = 0b0100_0000 | ((dr + 2) << 4) | ((dg + 2) << 2) | (db + 2);
    // println!("{}", chunk);
    return chunk;
}

#[wrappit]
fn chunk_qoi_op_luma(dr: u8, dg: u8, db: u8) -> [u8; 2] {
    if dg + 32 >= 64 || dr - dg + 8 >= 16 || db - dg + 8 >= 16 {
        panic!("luma diff too large");
    }
    return [0b1000_0000 | dg + 32, ((dr - dg + 8) << 4) | (db - dg + 8)];
}

#[wrappit]
fn chunk_qoi_op_run(run: u8) -> u8 {
    if run > 62 {
        panic!("run {} too large", run);
    }
    if run == 0 {
        panic!("zero-length run");
    }
    return 0b1100_0000 | (run - 1);
}

fn transform_u32_to_array_of_u8(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4];
}

fn transform_array_of_u8_to_u32(x: &[u8; 4]) -> u32 {
    let [b1, b2, b3, b4] = x;
    return ((*b1 as u32) << 24) | ((*b2 as u32) << 16) | ((*b3 as u32) << 8) | (*b4 as u32);
}
