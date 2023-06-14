use async_std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, Read};
use std::process::Command;

pub fn generate_hash(file_path: &str) -> String {
    let mut file = File::open(file_path).expect("Unable to open file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("Unable to read file");

    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    let hash_value = hasher.finish();

    format!("{:x}", hash_value)
}

pub async fn hash_file(path: PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;
    // Read Exact bytes, save us a lot of time
    let mut contents = [0; 4096];
    file.read_exact(&mut contents)?;
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

pub fn file_locate_exploer(file_path: &str) -> Result<std::process::Output, io::Error> {
    Command::new("explorer")
        .arg("/select,")
        .arg(file_path)
        .output()
    // .expect("Failed to execute process");
}
