use async_std::path::PathBuf;
use ring::digest;
use serde::ser::Error;
use sha2::Sha256;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::io::{self, Read};


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
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    // Ok(hasher.finish())
    Ok(format!("{:x}", hasher.finish()))
}

