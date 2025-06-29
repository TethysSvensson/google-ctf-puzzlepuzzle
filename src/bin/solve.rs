use std::collections::{HashMap, HashSet};

use clap::Parser;
use tools::{
    CachedGroups, Map, Shape, ShapeDb, ShapeDbIndex, ShapeId, Solution, find_group,
    normalize_group, read_cached_groups, read_shape_db, show_shape, write_cached_groups, write_map,
    write_shape_db,
};

#[derive(clap::Parser)]
struct Args {
    positions: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let mut shape_db = read_shape_db();
    let mut shape_db_index = shape_db
        .iter()
        .enumerate()
        .map(|(shape_id, shape)| ((shape.group.clone(), shape.parent), shape_id))
        .collect::<ShapeDbIndex>();
    let mut cached_groups = read_cached_groups();

    let mut map = tools::read_map();
    let mut todo: Vec<(usize, usize)> = args
        .positions
        .iter()
        .map(|arg| {
            let (x, y) = arg.split_once(",").unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect();

    let mut solved_count = 0;
    let shape_len_before = shape_db.len();

    while let Some((x, y)) = todo.pop() {
        if map[y][x] != 5 {
            continue; // Only process empty tiles
        }

        let ((min_x, min_y), shape_id) = get_group(&map, &shape_db_index, &mut cached_groups, x, y);
        let shape = shape_db[shape_id].clone();

        if let Some(unique_solution) = has_locally_unique_solution(
            &map,
            shape_id,
            &shape,
            &mut shape_db,
            &mut shape_db_index,
            &mut cached_groups,
            min_x,
            min_y,
        ) {
            solved_count += 1;
            for &((x, y), value) in &unique_solution {
                map[y][x] = value;
            }

            for ((x, y), value) in unique_solution {
                if 1 <= value && value <= 3 {
                    // If the tile is a neighbor, add it to the todo list
                    for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                        let nx = x.wrapping_add_signed(*dx);
                        let ny = y.wrapping_add_signed(*dy);
                        if map.get(ny).and_then(|row| row.get(nx)) == Some(&5) {
                            todo.push((nx, ny));
                        }
                    }
                }
            }
        }
    }
    println!("Solved {} tiles", solved_count);
    if shape_db.len() != shape_len_before {
        println!(
            "Shape database grew from {} to {} shapes",
            shape_len_before,
            shape_db.len()
        );
        write_shape_db(shape_db);
    }
    write_cached_groups(&cached_groups);
    write_map(&map);
}

fn get_group(
    map: &Map,
    shape_db_index: &ShapeDbIndex,
    cached_groups: &mut CachedGroups,
    x: usize,
    y: usize,
) -> ((usize, usize), ShapeId) {
    if let Some(&group) = cached_groups.get(&(x, y)) {
        group
    } else {
        let group = find_group(map, x, y);
        let (min_x, min_y, normalized_group) = normalize_group(&group);
        let key = (normalized_group, None);
        let shape_id = *shape_db_index.get(&key).unwrap_or_else(|| {
            panic!(
                "Shape not found in index for group at ({}, {}): {:?}",
                x, y, key.0
            )
        });
        cached_groups.insert((x, y), ((min_x, min_y), shape_id));
        ((min_x, min_y), shape_id)
    }
}

type Patch = ((usize, usize), u8);

fn has_locally_unique_solution(
    map: &Map,
    shape_id: ShapeId,
    shape: &Shape,
    shape_db: &mut ShapeDb,
    shape_db_index: &mut ShapeDbIndex,
    cached_groups: &mut CachedGroups,
    min_x: usize,
    min_y: usize,
) -> Option<Vec<Patch>> {
    let mut found_patches: Option<Vec<((usize, usize), u8)>> = None;
    let mut used_solutions = Vec::new();
    for (solution_id, solution) in shape.solutions.as_ref()?.iter().enumerate() {
        if let Some(cur_patches) = find_solution_valid_at(map, shape, solution, min_x, min_y) {
            if let Some(found_patches) = &mut found_patches {
                found_patches.retain(|kv| cur_patches.contains(kv));
            } else {
                found_patches = Some(cur_patches.into_iter().collect());
            }
            used_solutions.push(solution_id);
        }
    }
    let found_patches = found_patches?;
    if found_patches.is_empty() {
        None
    } else {
        let mut not_patched = shape
            .group
            .iter()
            .filter_map(|&(x, y)| {
                let actual_x = x + min_x;
                let actual_y = y + min_y;
                if found_patches
                    .iter()
                    .any(|(pos, _value)| *pos == (actual_x, actual_y))
                {
                    None
                } else {
                    Some((x, y))
                }
            })
            .collect::<Vec<_>>();
        if !not_patched.is_empty() {
            show_shape(shape);
            println!("Found patches: {:?}", found_patches);
            println!("Not patched: {:?}", not_patched);
            not_patched.sort_unstable();
            let key = (not_patched, Some(shape_id));
            if let Some(shape_id) = shape_db_index.get(&key) {
                for not_patched in key.0 {
                    cached_groups.insert(
                        (not_patched.0 + min_x, not_patched.1 + min_y),
                        ((min_x, min_y), *shape_id),
                    );
                }
            } else {
                let child_shape_id = shape_db.len();
                shape_db_index.insert((key.0.clone(), Some(shape_id)), shape_id);
                shape_db.push(Shape {
                    group: key.0.clone(),
                    solutions: None, // Solutions can be added later
                    parent: Some(shape_id),
                    used_solutions: Some(used_solutions),
                });
                for not_patched in key.0 {
                    cached_groups.insert(
                        (not_patched.0 + min_x, not_patched.1 + min_y),
                        ((min_x, min_y), child_shape_id),
                    );
                }
            }
        }
        Some(found_patches.into_iter().collect())
    }
}

fn find_solution_valid_at(
    map: &Map,
    shape: &Shape,
    solution: &Solution,
    min_x: usize,
    min_y: usize,
) -> Option<Vec<Patch>> {
    let mut neighbors: HashMap<(usize, usize), u8> = HashMap::new();
    let mut us = HashSet::new();

    let mut patches = Vec::new();

    for &(x, y) in &shape.group {
        let actual_x = x + min_x;
        let actual_y = y + min_y;
        us.insert((actual_x, actual_y));

        for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = actual_x.wrapping_add_signed(*dx);
            let ny = actual_y.wrapping_add_signed(*dy);
            if let Some(&tile) = map.get(ny).and_then(|row| row.get(nx)) {
                if 1 <= tile && tile <= 3 {
                    let entry = neighbors.entry((nx, ny)).or_default();
                    if solution.contains(&(x, y)) {
                        *entry += 1;
                    }
                }
            }
        }
        if solution.contains(&(x, y)) {
            patches.push(((actual_x, actual_y), 7));
        } else {
            patches.push(((actual_x, actual_y), 6));
        }
    }

    for (neighbor, &change) in &neighbors {
        let tile = map[neighbor.1][neighbor.0];
        if tile - change < 1 {
            return None;
        }
        let other_tiles_needed = tile - 1 - change;
        patches.push((*neighbor, tile - change));
        let mut other_tiles_available = 0;
        for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = neighbor.0.wrapping_add_signed(*dx);
            let ny = neighbor.1.wrapping_add_signed(*dy);
            if !us.contains(&(nx, ny)) && map.get(ny).and_then(|row| row.get(nx)) == Some(&5) {
                other_tiles_available += 1;
            }
        }

        if other_tiles_available < other_tiles_needed {
            return None;
        }
    }

    Some(patches)
}
