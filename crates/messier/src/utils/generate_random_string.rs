use std::collections::HashSet;

use rand::Rng;



pub fn generate_random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut code: HashSet<char> = HashSet::new();

    while code.len() < len {
        let c = match rng.gen_range(0..=2) {
            0 => rng.gen_range(b'0'..=b'9') as char,
            1 => rng.gen_range(b'A'..=b'Z') as char,
            _ => rng.gen_range(b'a'..=b'z') as char,
        };
        code.insert(c);
    }

    let code_str: String = code.into_iter().collect();
    code_str
}