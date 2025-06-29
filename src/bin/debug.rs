use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tools::{H, Map, W, normalize_group, read_map, show_at};

fn main() {
    let map = read_map();
    for y in 0..map.len() {
        for x in 0..map[y].len() {
            let tile = map[y][x];
            assert!(
                [0, 1, 2, 3, 5, 10].contains(&tile),
                "Unexpected tile value: {tile} at ({x}, {y})"
            );
        }
    }

    let normalized_group_types = Mutex::new(HashMap::new());
    let all_group_locations = dashmap::DashSet::new();

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
                        .entry(normalized_group)
                        .or_insert_with(|| {
                            let xsum = group.iter().map(|(x, _)| *x).sum::<usize>();
                            let ysum = group.iter().map(|(_, y)| *y).sum::<usize>();
                            let avg_x = ((xsum as f64) / (group.len() as f64)).round() as usize;
                            let avg_y = ((ysum as f64) / (group.len() as f64)).round() as usize;
                            all_group_locations.insert((avg_x, avg_y));

                            (avg_x, avg_y)
                        });
                }
            }
        });

    let normalized_group_types = normalized_group_types.into_inner().unwrap();
    println!("Found {} unique groups", normalized_group_types.len());
    for (group, &(x, y)) in &normalized_group_types {
        println!("Group found at ({x}, {y})");
        let max_x = group.iter().map(|(x, _)| *x).max().unwrap();
        let max_y = group.iter().map(|(_, y)| *y).max().unwrap();
        for y in 0..max_y + 1 {
            for x in 0..max_x + 1 {
                if group.contains(&(x, y)) {
                    print!("#");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }
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
