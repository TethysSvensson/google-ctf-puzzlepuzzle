use core::panic;

fn main() {
    let map = tools::read_map();
    let mut bits = Vec::new();
    for x in (8427..17236).step_by(24).rev() {
        if map[5][x] == tools::ACTIVE {
            bits.push(true)
        } else if map[5][x] == tools::NOT_ACTIVE {
            bits.push(false);
        } else {
            panic!("Unexpected tile value at (5, {}): {}", x, map[5][x]);
        }
    }
    assert!(
        bits.len() % 8 == 0,
        "Bit length is not a multiple of 8: {}",
        bits.len()
    );
    print!("CTF{{");
    for chunk in bits.chunks(8).rev() {
        let byte = chunk
            .iter()
            .rev()
            .enumerate()
            .fold(0u8, |acc, (i, &bit)| acc | ((bit as u8) << (7 - i)));
        print!("{}", byte as char);
    }
    println!("}}");
}
