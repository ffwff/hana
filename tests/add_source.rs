pub fn add_source(s: &str) {
    use std::time::{Duration, SystemTime};
    std::fs::write(format!("./fuzzing/in-raw/{}.hana", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros()), s);
}