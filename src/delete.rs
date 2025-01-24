use std::{fs, path::PathBuf};

pub fn delete_file(file_path: &PathBuf) {
    match fs::remove_file(file_path) {
        Ok(_) => println!("File deleted successfully: {}", file_path.display()),
        Err(e) => eprintln!("Failed to delete file {}: {}", file_path.display(), e),
    }
}
