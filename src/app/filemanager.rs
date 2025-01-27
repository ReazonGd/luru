extern crate chrono;

// use chrono::{DateTime, Local};
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
    time::SystemTime,
};

use super::pathmanager::{convert_path_to_nav, NavigationCommand};

// pub fn read_dir
pub struct ReadDirItems {
    pub path: PathBuf,
    pub navigation_type: NavigationCommand,
    pub label: String,
    pub file_name: String,
}

pub fn read_dir(path: &PathBuf, hide_hidden_file: &bool) -> io::Result<Vec<ReadDirItems>> {
    let mut res = Vec::new();

    if path.is_dir() && path.exists() {
        let entries = fs::read_dir(path);

        if let Err(_) = entries {
        } else if let Ok(val) = entries {
            for entry in val {
                let entry = entry.unwrap();
                let path = entry.path();

                let mut name = String::new();
                let file_name = path.file_name().unwrap();
                let file_name = file_name.to_str().unwrap();

                if *hide_hidden_file && file_name.starts_with('.') {
                    continue;
                }
                if path.is_dir() {
                    name.push_str("🖿 ");
                } else {
                    name.push_str("📑 ");
                }

                name.push_str(&file_name);

                res.push(ReadDirItems {
                    navigation_type: convert_path_to_nav(path.clone().to_str().unwrap())?,
                    path: path.clone(),
                    label: name.clone(),
                    file_name: file_name.to_string(),
                });
            }
        }
    } else {
    }
    Ok(res)
}

pub fn make_empty_file(path: PathBuf) {
    fs::write(path, "").unwrap();
}

pub struct MetadataInfo {
    pub display_name: String,
    pub size: u64,
    pub modified: String,
}
pub fn metadata(path: &PathBuf) -> io::Result<MetadataInfo> {
    let metadata = fs::metadata(path)?;
    let display_name = path.file_name().unwrap().to_str().unwrap().to_string();
    let size = metadata.len();
    let modified = metadata
        .modified()
        .map(|time| {
            time.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        })
        .unwrap_or(0);

    Ok(MetadataInfo {
        display_name,
        size,
        modified: format!("{:?}", modified),
    })
}

pub fn read_file(path: &PathBuf) -> io::Result<String> {
    // read file. if file not found, create it;
    if path.exists() {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    } else {
        fs::write(path, "")?;
        Ok(String::new())
    }
}
