use std::collections::{HashMap, HashSet};

use clap::Parser;
use tools::{
    ACTIVE, CachedGroups, Map, NOT_ACTIVE, Shape, ShapeDb, ShapeDbIndex, ShapeId, Solution,
    UNPROCESSED, find_group, normalize_group, read_cached_groups, read_shape_db, show_shape,
    write_cached_groups, write_map, write_shape_db,
};

#[derive(clap::Parser)]
struct Args {
    x: usize,
    y: usize,
}

fn main() {
    let args = Args::parse();

    let mut shape_db = read_shape_db();
    let mut shape_db_index = shape_db
        .iter()
        .enumerate()
        .map(|(shape_id, shape)| ((shape.group.clone(), shape.parent), shape_id))
        .collect::<ShapeDbIndex>();

    let map = tools::read_map();

    if map[args.y][args.x] != UNPROCESSED {
        println!("Tile at ({}, {}) is already processed", args.x, args.y);
        return;
    }

    let mut cached_groups = read_cached_groups();
    let ((min_x, min_y), shape_id) =
        get_group(&map, &shape_db_index, &mut cached_groups, args.x, args.y);
    let shape = shape_db[shape_id].clone();

    let solutions = shape.solutions.clone().unwrap();

    let mut counts = 0;
    let mut chosen_map = None;
    let mut chosen_cached_groups = None;
    let shape_len_before = shape_db.len();
    for solution in &solutions {
        let mut map = map.clone();
        let mut cached_groups = cached_groups.clone();
        if try_solve(
            &mut shape_db,
            &mut shape_db_index,
            &mut map,
            min_x,
            min_y,
            shape_id,
            &Shape {
                solutions: Some(vec![solution.clone()]),
                ..shape.clone()
            },
            &mut cached_groups,
        )
        .is_ok()
        {
            counts += 1;
            chosen_map = Some(map);
            chosen_cached_groups = Some(cached_groups);
        }
    }
    println!("{}", counts);
    if counts == 1 {
        if shape_db.len() != shape_len_before {
            println!(
                "Shape database grew from {} to {} shapes",
                shape_len_before,
                shape_db.len()
            );
            write_shape_db(shape_db);
        }
        write_cached_groups(&chosen_cached_groups.unwrap());
        write_map(&chosen_map.unwrap());
    }
}

fn try_solve(
    shape_db: &mut Vec<Shape>,
    shape_db_index: &mut HashMap<(Vec<(usize, usize)>, Option<usize>), usize>,
    map: &mut [[u8; 17268]; 90300],
    min_x: usize,
    min_y: usize,
    shape_id: usize,
    shape: &Shape,
    cached_groups: &mut HashMap<(usize, usize), ((usize, usize), usize)>,
) -> Result<(), InconsistentError> {
    let mut todo = vec![];
    if let Some(unique_solution) = has_locally_unique_solution(
        &*map,
        shape_id,
        &shape,
        shape_db,
        shape_db_index,
        cached_groups,
        min_x,
        min_y,
    )? {
        for &((x, y), value) in &unique_solution {
            let old_value = map[y][x];
            map[y][x] = value;

            if old_value != value {
                for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                    let nx = x.wrapping_add_signed(*dx);
                    let ny = y.wrapping_add_signed(*dy);
                    if let Some(potential_number) = map.get_mut(ny).and_then(|row| row.get_mut(nx))
                    {
                        if 1 <= *potential_number && *potential_number <= 3 {
                            if value == ACTIVE {
                                assert!(*potential_number > 1);
                                *potential_number -= 1; // Decrease the neighbor tile count
                            }
                            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                                let nx = nx.wrapping_add_signed(*dx);
                                let ny = ny.wrapping_add_signed(*dy);
                                todo.push((nx, ny));
                            }
                        }
                    }
                }
            }
        }
    }

    while let Some((x, y)) = todo.pop() {
        if x >= tools::W || y >= tools::H {
            continue;
        }
        if map[y][x] != UNPROCESSED {
            continue; // Only process empty tiles
        }

        let ((min_x, min_y), shape_id) = get_group(&*map, &*shape_db_index, cached_groups, x, y);
        let shape = shape_db[shape_id].clone();

        if let Some(unique_solution) = has_locally_unique_solution(
            &*map,
            shape_id,
            &shape,
            shape_db,
            shape_db_index,
            cached_groups,
            min_x,
            min_y,
        )? {
            for &((x, y), value) in &unique_solution {
                let old_value = map[y][x];
                map[y][x] = value;

                if old_value != value {
                    for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                        let nx = x.wrapping_add_signed(*dx);
                        let ny = y.wrapping_add_signed(*dy);
                        if let Some(potential_number) =
                            map.get_mut(ny).and_then(|row| row.get_mut(nx))
                        {
                            if 1 <= *potential_number && *potential_number <= 3 {
                                if value == ACTIVE {
                                    assert!(*potential_number > 1);
                                    *potential_number -= 1; // Decrease the neighbor tile count
                                }
                                for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                                    let nx = nx.wrapping_add_signed(*dx);
                                    let ny = ny.wrapping_add_signed(*dy);
                                    todo.push((nx, ny));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
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

#[derive(Debug)]
struct InconsistentError;

impl std::fmt::Display for InconsistentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Inconsistent state detected")
    }
}

impl std::error::Error for InconsistentError {}

fn has_locally_unique_solution(
    map: &Map,
    shape_id: ShapeId,
    shape: &Shape,
    shape_db: &mut ShapeDb,
    shape_db_index: &mut ShapeDbIndex,
    cached_groups: &mut CachedGroups,
    min_x: usize,
    min_y: usize,
) -> Result<Option<Vec<Patch>>, InconsistentError> {
    let mut found_patches: Option<Vec<((usize, usize), u8)>> = None;
    let mut used_solutions = Vec::new();
    let Some(solutions) = &shape.solutions else {
        return Ok(None); // No solutions available for this shape
    };
    for (solution_id, solution) in solutions.iter().enumerate() {
        if let Some(cur_patches) = find_solution_valid_at(map, shape, solution, min_x, min_y) {
            if let Some(found_patches) = &mut found_patches {
                found_patches.retain(|kv| cur_patches.contains(kv));
            } else {
                found_patches = Some(cur_patches.into_iter().collect());
            }
            used_solutions.push(solution_id);
        }
    }
    let Some(found_patches) = found_patches else {
        return Err(InconsistentError);
    };
    if found_patches.is_empty() {
        Ok(None)
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
        Ok(Some(found_patches.into_iter().collect()))
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
            patches.push(((actual_x, actual_y), ACTIVE));
        } else {
            patches.push(((actual_x, actual_y), NOT_ACTIVE));
        }
    }

    for (neighbor, &change) in &neighbors {
        let tile = map[neighbor.1][neighbor.0];
        if tile - change < 1 {
            return None;
        }
        let other_tiles_needed = tile - 1 - change;
        let mut other_tiles_available = 0;
        for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = neighbor.0.wrapping_add_signed(*dx);
            let ny = neighbor.1.wrapping_add_signed(*dy);
            if !us.contains(&(nx, ny))
                && map.get(ny).and_then(|row| row.get(nx)) == Some(&UNPROCESSED)
            {
                other_tiles_available += 1;
            }
        }

        if other_tiles_available < other_tiles_needed {
            return None;
        }
    }

    Some(patches)
}
