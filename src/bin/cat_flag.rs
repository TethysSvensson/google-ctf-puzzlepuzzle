fn main() {
    let map = tools::read_map();
    for x in (8426..17235).step_by(24).rev() {
        if map[5][x] == tools::ACTIVE {
            print!("1");
        } else if map[5][x] == tools::NOT_ACTIVE {
            print!("0");
        } else {
            print!("?");
        }
    }
    println!()
}
