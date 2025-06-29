use core::arch;

use clap::Parser;
use tools::{SHAPE_ALPHABET, show_shape};

#[derive(clap::Parser)]
struct Args {
    shape_id: Option<usize>,
}

pub fn main() {
    let args = Args::parse();
    let shape_id = args.shape_id;
    let shape_db: tools::ShapeDb = serde_json::from_str(
        &std::fs::read_to_string("shape_db.json").expect("Failed to read shape_db.json"),
    )
    .expect("Failed to parse shape_db.json");

    let shape_id = if let Some(shape_id) = shape_id {
        shape_id
    } else {
        if let Some(id) = shape_db.iter().position(|shape| shape.solutions.is_none()) {
            id
        } else {
            eprintln!("No shape without solutions found");
            return;
        }
    };
    if shape_id >= shape_db.len() {
        eprintln!(
            "Shape ID {} out of bounds (max {})",
            shape_id,
            shape_db.len() - 1
        );
        return;
    }
    let shape = &shape_db[shape_id];
    println!("Shape ID: {}", shape_id);
    show_shape(shape);
}
