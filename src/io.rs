use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::Result;

pub fn find_all_files_by_extension(
    base_directory: &Path,
    file_extension: &str,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(base_directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            paths.append(&mut find_all_files_by_extension(&path, file_extension)?);
        } else if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.to_lowercase().ends_with(file_extension) {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

pub fn get_all_filenames(path: PathBuf) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        Ok(vec![path])
    } else if path.is_dir() {
        Ok(find_all_files_by_extension(&path, ".anki.md").unwrap())
    } else {
        Err(anyhow::anyhow!("Path is not a file or directory"))
    }
}

pub fn read_file_to_string(file: &Path) -> Result<String> {
    let mut file = File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
