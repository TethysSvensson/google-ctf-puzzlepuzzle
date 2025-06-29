use std::collections::HashMap;

pub const W: usize = 17268;
pub const H: usize = 90300;

pub type Map = [[u8; W]; H];

pub fn read_map() -> Box<Map> {
    let data = std::fs::read("puzzlepuzzle.raw").expect("Failed to read puzzlepuzzle.raw");
    assert_eq!(data.len(), H * W, "Data length mismatch");
    let boxed: Box<[u8; W * H]> = data
        .into_boxed_slice()
        .try_into()
        .expect("Failed to convert to fixed-size array");
    let array: Box<Map> = unsafe { std::mem::transmute(boxed) };
    array
}

pub fn find_group(map: &Map, x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut group = Vec::new();
    let mut stack = vec![(x, y)];
    let tile = map[y][x];
    assert_eq!(tile, 5, "Expected tile at ({}, {}) to be 5", x, y);

    while let Some((cx, cy)) = stack.pop() {
        for (nx, ny) in [
            (cx.wrapping_sub(1), cy), // left
            (cx + 1, cy),             // right
            (cx, cy.wrapping_sub(1)), // up
            (cx, cy + 1),             // down
        ] {
            if nx < W && ny < H && map[ny][nx] == 5 && !group.contains(&(nx, ny)) {
                group.push((nx, ny));
                stack.push((nx, ny));
            }
        }
    }

    group
}

pub fn normalize_group(group: &[(usize, usize)]) -> (usize, usize, Vec<(usize, usize)>) {
    assert!(!group.is_empty());

    let min_x = group.iter().map(|(x, _)| *x).min().unwrap();
    let min_y = group.iter().map(|(_, y)| *y).min().unwrap();

    let mut normalized = group
        .into_iter()
        .map(|(x, y)| (x - min_x, y - min_y))
        .collect::<Vec<_>>();
    normalized.sort_unstable();
    (min_x, min_y, normalized)
}

pub const UNPROCESSED: u8 = 5;
pub const NOT_ACTIVE: u8 = 6;
pub const ACTIVE: u8 = 7;

pub fn show_at(map: &Map, gx: usize, gy: usize, size: usize) {
    for y in gy.saturating_sub(size)..gy.saturating_add(size).min(H) {
        for x in gx.saturating_sub(size)..gx.saturating_add(size).min(W) {
            let tile = map[y][x];
            if x == gx && y == gy {
                // Highlight the center tile
                print!("\x1b[31;1m"); // Red bold for the center
            }
            match tile {
                0 => print!(" "),
                1 => print!("0"),
                2 => print!("1"),
                3 => print!("2"),
                UNPROCESSED => print!("#"),
                NOT_ACTIVE => print!("\x1b[37;2m.\x1b[0m"),
                ACTIVE => print!("\x1b[33;1m#\x1b[0m"),
                10 => print!("?"),
                _ => panic!("Unexpected tile value: {tile}"),
            }
            if x == gx && y == gy {
                // Reset color after the center tile
                print!("\x1b[0m");
            }
        }
        println!();
    }
}

pub const SHAPE_ALPHABET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub type ShapeDbIndex = HashMap<(Vec<(usize, usize)>, Option<ShapeId>), ShapeId>;
pub type CachedGroups = HashMap<(usize, usize), ((usize, usize), ShapeId)>;
pub type ShapeDb = Vec<Shape>;
pub type ShapeId = usize;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Shape {
    pub group: Vec<(usize, usize)>,
    pub solutions: Option<Vec<Solution>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parent: Option<ShapeId>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub used_solutions: Option<Vec<usize>>,
}

pub type Solution = Vec<(usize, usize)>;

pub fn show_shape(shape: &Shape) {
    let group = &shape.group;
    if group.is_empty() {
        println!("No shape to display");
        return;
    }
    let max_x = group.iter().map(|(x, _)| *x).max().unwrap();
    let max_y = group.iter().map(|(_, y)| *y).max().unwrap();
    let mut letters = SHAPE_ALPHABET.chars();
    for y in 0..=max_y {
        for x in 0..=max_x {
            if group.contains(&(x, y)) {
                print!("{}", letters.next().unwrap());
            } else {
                print!(" ");
            }
        }
        println!();
    }

    if let Some(solutions) = &shape.solutions {
        println!("Current solutions:");
        for solution in solutions {
            println!("=========");
            for y in 0..=max_y {
                for x in 0..=max_x {
                    if shape.group.contains(&(x, y)) {
                        if solution.contains(&(x, y)) {
                            print!("#")
                        } else {
                            print!(".");
                        }
                    } else {
                        print!(" ");
                    }
                }
                println!()
            }
        }
        println!();
    }

    if let Some(parent) = shape.parent {
        println!("Parent shape ID: {}", parent);
    } else {
        println!("No parent shape");
    }
    if let Some(used_solutions) = &shape.used_solutions {
        println!("Used solutions: {:?}", used_solutions);
    }
}

pub fn read_shape_db() -> ShapeDb {
    serde_json::from_str::<ShapeDb>(
        &std::fs::read_to_string("shape_db.json").expect("Failed to read shape_db.json"),
    )
    .unwrap()
}

pub fn read_cached_groups() -> CachedGroups {
    if std::fs::exists("cached_groups.bin").unwrap() {
        serde_cbor::from_slice(
            &std::fs::read("cached_groups.bin").expect("Failed to read cached_groups.json"),
        )
        .unwrap()
    } else {
        Default::default()
    }
}

pub fn write_shape_db(shape_db: &[Shape]) {
    std::fs::write("shape_db.json", serde_json::to_string(&shape_db).unwrap()).unwrap();
    println!("Written to shape_db.json");
}

pub fn write_cached_groups(cached_groups: &CachedGroups) {
    std::fs::write(
        "cached_groups.bin",
        serde_cbor::to_vec(cached_groups).expect("Failed to serialize cached groups"),
    )
    .expect("Failed to write cached_groups.bin");
    println!("Written to cached_groups.bin");
}

pub fn write_cached_groups_named(cached_groups: &CachedGroups, name: &str) {
    std::fs::write(
        name,
        serde_cbor::to_vec(cached_groups).expect("Failed to serialize cached groups"),
    )
    .expect("Failed to write {name}");
    println!("Written cache group to {name}");
}

pub fn write_map(map: &Map) {
    let map: &[[u8; W]; H] = map;
    let map: &[u8; W * H] = unsafe { core::mem::transmute(map) };
    std::fs::write("puzzlepuzzle.raw", map).expect("Failed to  write puzzlepuzzle.raw");
    println!("Written to puzzlepuzzle.raw");
}

pub fn write_map_named(map: &Map, name: &str) {
    let map: &[[u8; W]; H] = map;
    let map: &[u8; W * H] = unsafe { core::mem::transmute(map) };
    std::fs::write(&name, map).expect("Failed to  write puzzlepuzzle.raw");
    println!("Written to map to {name}");
}
