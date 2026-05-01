use rand::distr::Alphanumeric;
use rand::Rng;

pub fn random_str() -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    return s;
}

pub fn char_code_at(s: &str, idx: usize) -> Option<u32> {
    s.chars().nth(idx).map(|c| c as u32)
}