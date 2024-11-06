use std::time::{SystemTime, UNIX_EPOCH};

pub fn random_range(min: i32, max: i32) -> i32 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
    let seed = (now % 1000) as u64;
    let random_value = (seed % ((max - min) as u64)) + (min as u64);
    random_value as i32
}
