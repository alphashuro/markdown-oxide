use std::{fs::File, path::PathBuf};

use chrono::Local;

pub fn find_unique_file_path(folder: &PathBuf, format: &str) -> PathBuf {
    let now = Local::now().naive_local();

    let filename = now.format(format).to_string();
    let extension = "md";

    find_unique_file_path_rec(folder.into(), filename, extension, None)
}

fn find_unique_file_path_rec(
    path: PathBuf,
    filename: String,
    extension: &str,
    postfix: Option<i32>,
) -> PathBuf {
    let curr_filename = format!(
        "{}{}",
        filename,
        postfix.map_or("".to_string(), |n| n.to_string())
    );

    let filepath = path.join(&curr_filename).with_extension(extension);

    if filepath.exists() {
        find_unique_file_path_rec(
            path,
            filename,
            extension,
            postfix.map_or(Some(0), |n| Some(n + 1)),
        )
    } else {
        filepath
    }
}

pub fn create_unique_note(folder: &PathBuf, format: &str) -> PathBuf {
    let file_path = find_unique_file_path(folder, format);

    // file creation can fail and return an Err, ignore this and try
    // to open the file on the off chance the client knows what to do
    // TODO: log failure to create file
    let _ = file_path.parent().map(std::fs::create_dir_all).unwrap();

    File::create_new(file_path.as_path()).unwrap();

    file_path
}
