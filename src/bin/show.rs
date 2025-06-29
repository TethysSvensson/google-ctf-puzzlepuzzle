use clap::Parser;
use tools::{read_map, show_at};

#[derive(clap::Parser)]
struct Args {
    x: usize,
    y: usize,
}

fn main() {
    let args = Args::parse();
    let x = args.x;
    let y = args.y;

    let map = read_map();
    show_at(&map, x, y, 15);
}
