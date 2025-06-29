use clap::Parser;
use image::{ImageBuffer, ImageFormat, Rgb};
use std::fs::File;
use std::io::BufWriter;
use tools::read_map;

#[derive(clap::Parser)]
struct Args {
    x: usize,
    y: usize,
    w: u32,
    h: u32,
}

fn main() {
    let args = Args::parse();
    // Load the shape database
    let map = read_map();

    let mut img = ImageBuffer::<Rgb<u8>, _>::new(args.w, args.h);

    // Fill the image with white color

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = match map[args.y + y as usize][args.x + x as usize] {
            0 => Rgb([0, 0, 0]),
            1 => Rgb([255, 0, 0]),
            2 => Rgb([255, 255, 0]),
            3 => Rgb([0, 255, 0]),
            5 => Rgb([255, 255, 255]),
            7 => Rgb([128, 128, 128]),
            8 => Rgb([231, 141, 14]),
            10 => Rgb([255, 0, 255]),
            _ => Rgb([0, 0, 255]),
            // n => todo!("{n}"),
        }
    }

    // Save the image to a file
    let path = "output_image.png";
    let file = File::create(path).expect("Failed to create image file");
    let ref mut writer = BufWriter::new(file);

    img.write_to(writer, ImageFormat::Png)
        .expect("Failed to save image");

    println!("Image saved to {}", path);
}
