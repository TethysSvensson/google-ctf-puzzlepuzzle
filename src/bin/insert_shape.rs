use std::collections::BTreeSet;

use clap::Parser;
use tools::{SHAPE_ALPHABET, Shape, Solution, read_shape_db, show_shape, write_shape_db};

#[derive(clap::Parser)]
struct Args {
    shape_id: usize,
    solutions: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let shape_id = args.shape_id;
    let mut shape_db: tools::ShapeDb = read_shape_db();

    if shape_id >= shape_db.len() {
        eprintln!(
            "Shape ID {} out of bounds (max {})",
            shape_id,
            shape_db.len() - 1
        );
        return;
    }

    let solutions = args.solutions;
    let sol_unique = solutions
        .iter()
        .map(|sol| {
            assert!(sol.chars().all(|c| tools::SHAPE_ALPHABET.contains(c)));
            let sol_unique = sol.chars().collect::<BTreeSet<char>>();
            assert_eq!(
                sol.len(),
                sol_unique.len(),
                "Duplicate characters in solution: {sol}"
            );
            sol_unique
        })
        .collect::<BTreeSet<BTreeSet<char>>>();

    assert_eq!(
        sol_unique.len(),
        solutions.len(),
        "Duplicate solutions found: {:?}",
        sol_unique
    );

    let shape = &mut shape_db[shape_id];
    show_shape(&shape);
    println!();
    println!("You entered:");

    let solutions = solutions
        .into_iter()
        .map(|sol| solution_string_to_solution(&sol, &shape))
        .collect::<Vec<Solution>>();
    shape.solutions = Some(solutions);

    println!("Correct?");
    let is_correct = std::io::stdin()
        .lines()
        .next()
        .expect("Failed to read input")
        .expect("Failed to read line")
        .trim()
        .starts_with('y');

    if is_correct {
        write_shape_db(&shape_db);
    }
}

fn solution_string_to_solution(solutions: &str, shape: &Shape) -> Solution {
    let mut solution = solutions.chars().collect::<BTreeSet<char>>();
    let max_x = shape.group.iter().map(|(x, _)| *x).max().unwrap();
    let max_y = shape.group.iter().map(|(_, y)| *y).max().unwrap();
    let mut letters = SHAPE_ALPHABET.chars();
    let mut out = vec![];
    println!("=========");
    for y in 0..=max_y {
        for x in 0..=max_x {
            if shape.group.contains(&(x, y)) {
                let letter = letters.next().unwrap();
                if solution.remove(&letter) {
                    out.push((x, y));
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
    println!();
    assert!(
        solution.is_empty(),
        "Not all letters used in solution: {:?}",
        solution
    );

    out
}
