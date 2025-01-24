mod delete;
mod duplicates;

use duplicates::start_delete_duplicates;
use rfd::FileDialog;

fn main() {
    // Open a dialog to select a folder
    if let Some(folder_path) = FileDialog::new().pick_folder() {
        println!("Selected folder: {:?}", folder_path);
        start_delete_duplicates(folder_path.as_path());
    } else {
        println!("No folder selected.");
    }
}
