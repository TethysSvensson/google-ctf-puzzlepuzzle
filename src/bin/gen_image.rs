use clap::Parser;
use image::{ImageEncoder, codecs::png::PngEncoder};
use std::fs::File;
use std::io::BufWriter;
use tools::{H, W, read_map};

#[derive(clap::Parser)]
struct Args {
    x1: usize,
    x2: usize,
    y1: usize,
    y2: usize,
}

fn main() {
    let args = Args::parse();
    let map = read_map();

    let width = (args.x2 - args.x1).min(W - args.x1);
    let height = (args.y2 - args.y1).min(H - args.y1);

    let mut img = vec![0; width * height * 3];

    for y in 0..height {
        for x in 0..width {
            let pixel = match map[(args.y1 + y) as usize][(args.x1 + x) as usize] {
                0 => [0, 0, 0],
                1 => [255, 0, 0],
                2 => [255, 255, 0],
                3 => [0, 255, 0],
                5 => [255, 255, 255],
                7 => [128, 128, 128],
                8 => [231, 141, 14],
                10 => [255, 0, 255],
                _ => [0, 0, 255],
                // n => todo!("{n}"),
            };
            img[(y * width + x) * 3 + 0] = pixel[0];
            img[(y * width + x) * 3 + 1] = pixel[1];
            img[(y * width + x) * 3 + 2] = pixel[2];
        }
    }

    // Save the image to a file
    let path = "output_image.png";
    let file = File::create(path).expect("Failed to create image file");
    let writer = PngEncoder::new_with_quality(
        BufWriter::new(file),
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::NoFilter,
    );

    writer
        .write_image(
            &img,
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgb8,
        )
        .expect("Failed to save image");

    println!("Image saved to {}", path);
}
