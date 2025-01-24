use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File, metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::delete::delete_file;

/// Computes the SHA-256 hash of a file.
fn calculate_hash(file_path: &Path) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Finds and deletes duplicate files in a folder using file size and hashing.
fn delete_duplicates_in(folder_path: &Path, file_map: &mut HashMap<u64, Vec<PathBuf>>, hash_map: &mut HashMap<String, PathBuf>) {
    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                if path.is_file() {
                    match metadata(&path) {
                        Ok(metadata) => {
                            let file_size = metadata.len();

                            // Group files by size
                            let group = file_map.entry(file_size).or_insert_with(Vec::new);
                            group.push(path.clone());

                            // Hash files only if size matches an existing size group
                            if group.len() > 1 {
                                for candidate_path in group.iter() {
                                    if let Ok(hash) = calculate_hash(candidate_path) {
                                        if let Some(original_path) = hash_map.get(&hash) {
                                            println!("Duplicate found: {:#?} (original: {:#?})", candidate_path, original_path);
                                            delete_file(candidate_path);
                                        } else {
                                            hash_map.insert(hash, candidate_path.clone());
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => eprintln!("Failed to retrieve metadata for {}: {}", path.display(), e),
                    }
                } else if path.is_dir() {
                    delete_duplicates_in(&path, file_map, hash_map);
                } else {
                    eprintln!("Unhandled file system object: {}", path.display());
                }
            }
        }
    } else {
        eprintln!("Failed to read directory: {}", folder_path.display());
    }
}

/// Top-level function to call the recursive one
pub fn start_delete_duplicates(folder_path: &Path) {
    let mut file_map: HashMap<u64, Vec<PathBuf>> = HashMap::new(); // Groups files by size
    let mut hash_map: HashMap<String, PathBuf> = HashMap::new();   // Tracks hashed files
    delete_duplicates_in(folder_path, &mut file_map, &mut hash_map);
}
