use super::pathmanager;
use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

pub struct Config {
    pub working_path: PathBuf,
    pub config_file_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let temp_folder = env::temp_dir();
        let nav_to_app_temp = pathmanager::convert_path_to_nav("luru").unwrap();
        let des_path = pathmanager::resolve_path(&temp_folder, &nav_to_app_temp).unwrap();

        let file = File::open(&des_path);
        if let Err(_) = file {
            let content = format!("WORKING_PATH=");
            fs::write(&des_path, content).unwrap();
        }

        Config {
            working_path: PathBuf::from("/"),
            config_file_path: des_path,
        }
    }

    pub fn load(&mut self) {
        let file = File::open(&self.config_file_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let vv: Vec<&str> = line.split("=").collect();

            match vv[0] {
                "WORKING_PATH" => {
                    let val = vv[1];
                    if val.is_empty() {
                        self.set_working_path(&env::current_dir().unwrap());
                    } else {
                        self.working_path = PathBuf::from(val.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    pub fn save(&mut self) {
        let content = format!("WORKING_PATH={}", self.working_path.display());

        fs::write(&self.config_file_path, content).unwrap();
    }

    pub fn set_working_path<P: AsRef<Path>>(&mut self, wp: &P) {
        self.working_path = wp.as_ref().to_path_buf();
    }
}
