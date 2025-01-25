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

/// Groups files by their size.
fn group_files_by_size(folder_path: &Path, file_map: &mut HashMap<u64, Vec<PathBuf>>) {
    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = metadata(&path) {
                        let file_size = metadata.len();
                        file_map.entry(file_size).or_insert_with(Vec::new).push(path);
                    }
                } else if path.is_dir() {
                    group_files_by_size(&path, file_map);
                }
            }
        }
    } else {
        eprintln!("Failed to read directory: {}", folder_path.display());
    }
}

/// Finds and deletes duplicate files within each group of files with the same size.
fn find_and_delete_duplicates(file_map: HashMap<u64, Vec<PathBuf>>) {
    let mut hash_map: HashMap<String, PathBuf> = HashMap::new(); // Tracks hashed files

    for (_, paths) in file_map {
        if paths.len() > 1 {
            for path in paths {
                match calculate_hash(&path) {
                    Ok(hash) => {
                        if let Some(original_path) = hash_map.get(&hash) {
                            println!("Duplicate found: {:#?} (original: {:#?})", path, original_path);
                            delete_file(&path);
                        } else {
                            hash_map.insert(hash, path);
                        }
                    }
                    Err(e) => eprintln!("Failed to calculate hash for {}: {}", path.display(), e),
                }
            }
        }
    }
}

/// Top-level function to group files and delete duplicates.
pub fn start_delete_duplicates(folder_path: PathBuf) {
    let mut file_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    group_files_by_size(&folder_path, &mut file_map);
    find_and_delete_duplicates(file_map);
}
