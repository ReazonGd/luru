use super::pathmanager;
use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

pub struct Config {
    pub working_path: PathBuf,
    pub config_file_path: PathBuf,
    pub history_path: PathBuf,
    pub command_history: Vec<String>,
}

impl Config {
    pub fn new() -> io::Result<Self> {
        let temp_folder = env::temp_dir();
        let nav_to_app_temp = pathmanager::convert_path_to_nav("luru")?;
        let temp_path = pathmanager::resolve_path(&temp_folder, &nav_to_app_temp)?;

        let nav_to_app_command_history = pathmanager::convert_path_to_nav("luru-cmd.log")?;
        let history_path = pathmanager::resolve_path(&temp_folder, &nav_to_app_command_history)?;

        let file = File::open(&temp_path);
        if let Err(_) = file {
            let content = format!("WORKING_PATH=");
            fs::write(&temp_path, content)?;
        }

        let file = File::open(&history_path);
        if let Err(_) = file {
            fs::write(&history_path, "")?;
        }

        Ok(Config {
            working_path: PathBuf::from("/"),
            command_history: Vec::new(),
            config_file_path: temp_path,
            history_path,
        })
    }

    pub fn load(&mut self) -> io::Result<()> {
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

        // load history
        self.load_history()
    }

    pub fn load_history(&mut self) -> io::Result<()> {
        let file = File::open(&self.history_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            self.command_history.push(line);
        }

        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        let content = format!("WORKING_PATH={}", self.working_path.display());
        fs::write(&self.config_file_path, content)?;

        fs::write(&self.history_path, self.command_history.join("\n"))?;

        Ok(())
    }

    pub fn set_working_path<P: AsRef<Path>>(&mut self, wp: &P) {
        self.working_path = wp.as_ref().to_path_buf();
    }
}
