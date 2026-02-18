pub fn _string_to_code_points(string: String) -> Vec<u32> {
    let mut code_points: Vec<u32> = vec![];
    let size = string.len();
    let mut position = 0;
    loop {
        if position >= size {
            break;
        }
        let cp = _code_point_at(&string, position);
        code_points.push(cp);
        position += 1
    }
    code_points
}

fn _code_point_at(string: &String, position: usize) -> u32 {
    let char = string
        .chars()
        .nth(position)
        .expect("Spec prevents this from occurring");
    char as u32
}
