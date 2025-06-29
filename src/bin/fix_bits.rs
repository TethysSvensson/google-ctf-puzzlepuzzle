use tools::write_map;

fn main() {
    let mut map = tools::read_map();
    for x in (8426..17235).step_by(24).rev() {
        map[5][x] = tools::UNPROCESSED;
    }
    write_map(&map);
}
