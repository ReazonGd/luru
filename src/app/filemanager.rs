extern crate chrono;

use chrono::{DateTime, Local};
use std::{fs, io, path::PathBuf, time::SystemTime};

use super::pathmanager::{convert_path_to_nav, NavigationCommand};

// pub fn read_dir
pub struct ReadDirItems {
    pub path: PathBuf,
    pub navigation_type: NavigationCommand,
    pub label: String,
    pub file_name: String,
}

pub fn read_dir(path: &PathBuf) -> io::Result<Vec<ReadDirItems>> {
    let mut res = Vec::new();

    if path.is_dir() && path.exists() {
        let entries = fs::read_dir(path);

        if let Err(_) = entries {

            // return Err(e);
        } else if let Ok(val) = entries {
            // if path.eq(&PathBuf::from("/")) {
            //     res.push(items {
            //         path:
            //     });
            // }

            for entry in val {
                let entry = entry.unwrap();
                let path = entry.path();

                let mut name = String::new();

                let metadata = path.metadata()?;

                if let Ok(time) = metadata.modified() {
                    let date_time: DateTime<Local> = time.into();
                    name.push_str(format!("{}  ", date_time.format("%d-%m-%Y")).as_str());
                } else {
                    name.push_str("-  ");
                }

                if path.is_dir() {
                    name.push_str("dir  /");
                    // name.push('/');
                } else {
                    name.push_str("file  ");
                }

                name.push_str(path.file_name().and_then(|s| s.to_str()).unwrap());
                // name.push_str(" ".repeat().as_str());

                res.push(ReadDirItems {
                    navigation_type: convert_path_to_nav(path.clone().to_str().unwrap()).unwrap(),
                    path: path.clone(),
                    label: name.clone(),
                    file_name: path.file_name().unwrap().to_str().unwrap().to_string(),
                });
            }
        }
    } else {
        // Err(io::Error::new(io::ErrorKind::NotFound, "Not a directory"))?
        // res.push(String::from("/"));
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
