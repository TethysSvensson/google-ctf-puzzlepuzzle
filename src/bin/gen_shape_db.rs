use std::{
    collections::{HashMap, HashSet},
    num::NonZero,
    sync::Mutex,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tools::{H, Map, W, normalize_group, read_map};

fn main() {
    let map = read_map();

    let normalized_group_types = Mutex::new(HashSet::new());

    (0..H)
        .into_par_iter()
        .for_each_with(HashSet::new(), |used, y| {
            if y % 5000 == 0 {
                println!("Processing row {y} of {H}");
            }
            for x in 0..W {
                if let Some((group, _neighbors)) = find_group(&map, x, y, used) {
                    let (_, _, normalized_group) = normalize_group(&group);
                    normalized_group_types
                        .lock()
                        .unwrap()
                        .insert(normalized_group);
                }
            }
        });

    let normalized_group_types = normalized_group_types.into_inner().unwrap();
    let mut normalized_group_types: Vec<Vec<(usize, usize)>> =
        normalized_group_types.into_iter().collect();
    normalized_group_types.sort_unstable();

    let shape_db: tools::ShapeDb = normalized_group_types
        .into_iter()
        .map(|group| tools::Shape {
            group,
            solutions: None, // Solutions can be added later
        })
        .collect();

    std::fs::write("shape_db.json", serde_json::to_string(&shape_db).unwrap()).unwrap();
}

pub fn find_group(
    map: &Map,
    x: usize,
    y: usize,
    used: &mut HashSet<(usize, usize)>,
) -> Option<(Vec<(usize, usize)>, HashMap<(usize, usize), u8>)> {
    let mut group = Vec::new();
    let mut stack = vec![(x, y)];
    let mut neighbors = HashMap::new();
    let tile = map[y][x];
    if tile != 5 || used.contains(&(x, y)) {
        return None;
    }

    while let Some((cx, cy)) = stack.pop() {
        for (nx, ny) in [
            (cx.wrapping_sub(1), cy), // left
            (cx + 1, cy),             // right
            (cx, cy.wrapping_sub(1)), // up
            (cx, cy + 1),             // down
        ] {
            if nx < W && ny < H {
                if map[ny][nx] == 5 {
                    if used.insert((nx, ny)) {
                        group.push((nx, ny));
                        stack.push((nx, ny));
                    }
                } else if map[ny][nx] != 0 {
                    neighbors.insert((nx, ny), map[ny][nx]);
                }
            }
        }
    }

    Some((group, neighbors))
}
