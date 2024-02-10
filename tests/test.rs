use serde_construct_classic::from_bytes;
use serde_construct_classic::to_bytes;
use serde_json::Value;
use std::cmp::min;
use std::{env, fs};

#[test]
fn test() {
    let dir_path = env::var("SAMPLES_DIR").unwrap();
    for entry in std::fs::read_dir(dir_path).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() { continue; }
        let name = path.file_name().unwrap().to_string_lossy();
        let bytes = std::fs::read(&path).unwrap();
        eprintln!("Deserializing {name}");
        let value = match from_bytes::<Value>(&bytes) {
            Ok(value) => value,
            Err(err) => panic!("{}", err),
        };
        eprintln!("Reserializing");
        let bytes_out = match to_bytes(&value) {
            Ok(bytes) => bytes,
            Err(err) => panic!("{}", err),
        };
        for i in 0..min(bytes.len(), bytes_out.len()) {
            if bytes[i] != bytes_out[i] {
                fs::write(format!("{}.0", name), &bytes).unwrap();
                fs::write(format!("{}.1", name), &bytes_out).unwrap();
                panic!("Difference at offset {i}: 0x{:02x} != 0x{:02x}", bytes[i], bytes_out[i])
            }
        }
        if bytes.len() != bytes_out.len() {
            panic!("Input is length {}, output is length {}", bytes.len(), bytes_out.len());
        }
    }
}