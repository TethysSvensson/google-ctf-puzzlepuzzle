fn main() {
    let data = std::fs::read("puzzlepuzzle.dat").expect("Failed to read puzzlepuzzle.dat");

    // Read width and height from little-endian bytes
    let width = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let height = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;

    println!("Width: {}, Height: {}", width, height);

    // Adjust data to skip the first 8 bytes (header)
    let tile_data = &data[8..];

    assert_eq!(width, 17268);
    assert_eq!(height, 90300);

    assert_eq!(
        tile_data.len(),
        width * height / 2,
        "Tile data length mismatch"
    );

    // Closure to get a tile value
    let get_tile = |r: usize, c: usize| -> u8 {
        if r >= height || c >= width {
            return 0;
        }
        let idx = r * width + c;
        let byte = tile_data[idx / 2];
        if idx % 2 == 0 { byte >> 4 } else { byte & 0x0F }
    };

    let mut out = Vec::with_capacity(width * height);
    for r in 0..height {
        for c in 0..width {
            let tile = get_tile(r, c);
            out.push(tile);
        }
    }
    // Write the output to a file
    std::fs::write("puzzlepuzzle.raw", out).expect("Failed to write puzzlepuzzle.raw");
}
