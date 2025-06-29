use clap::Parser;
use tools::read_map;

#[derive(clap::Parser)]
struct Args {
    x: usize,
    y: usize,
    value: u8,
}

fn main() {
    let args = Args::parse();
    let mut map = read_map();
    map[args.y][args.x] = args.value;
    tools::write_map(&map);
}
