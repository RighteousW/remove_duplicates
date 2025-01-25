use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::delete::delete_file;

/// Computes the SHA-256 hash of a file.
fn calculate_hash(file_path: &Path) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Retrieves the creation time or last modification time of a file.
fn get_creation_time(meta: &Metadata) -> io::Result<SystemTime> {
    meta.created().or_else(|_| meta.modified())
}

/// Handles duplicate detection and deletion based on file hashes and creation times.
fn handle_duplicate(
    hash: String,
    path: PathBuf,
    creation_time: SystemTime,
    hash_map: &Mutex<HashMap<String, (PathBuf, SystemTime)>>,
) {
    let mut hash_map = hash_map.lock().unwrap();
    if let Some((original_path, original_time)) = hash_map.get(&hash) {
        if creation_time > *original_time {
            println!("Duplicate found: {:?} (keeping: {:?})", path, original_path);
            delete_file(&path);
        } else {
            println!("Duplicate found: {:?} (keeping: {:?})", original_path, path);
            delete_file(original_path);
            hash_map.insert(hash, (path, creation_time));
        }
    } else {
        hash_map.insert(hash, (path, creation_time));
    }
}

/// Processes a file by calculating its hash, getting its creation time, and handling duplicates.
fn process_file(path: PathBuf, hash_map: Arc<Mutex<HashMap<String, (PathBuf, SystemTime)>>>) {
    if let Ok(meta) = path.metadata() {
        if let Ok(hash) = calculate_hash(&path) {
            if let Ok(creation_time) = get_creation_time(&meta) {
                handle_duplicate(hash, path, creation_time, &hash_map);
            } else {
                eprintln!("Failed to get creation time for {}", path.display());
            }
        } else {
            eprintln!("Failed to calculate hash for {}", path.display());
        }
    } else {
        eprintln!("Failed to retrieve metadata for {}", path.display());
    }
}

/// Recursively scans a folder for files and processes them for duplicates using parallelism.
fn scan_folder(folder_path: &Path, hash_map: Arc<Mutex<HashMap<String, (PathBuf, SystemTime)>>>) {
    if let Ok(entries) = fs::read_dir(folder_path) {
        entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>()
            .par_iter()
            .for_each(|path| {
                if path.is_file() {
                    process_file(path.clone(), Arc::clone(&hash_map));
                } else if path.is_dir() {
                    scan_folder(path, Arc::clone(&hash_map));
                }
            });
    } else {
        eprintln!("Failed to read directory: {}", folder_path.display());
    }
}

/// Entry point for scanning and deleting duplicate files.
pub fn start_delete_duplicates(folder_path: PathBuf) {
    let hash_map = Arc::new(Mutex::new(HashMap::new()));
    scan_folder(&folder_path, Arc::clone(&hash_map));
}
