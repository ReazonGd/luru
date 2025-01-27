use super::{filemanager, pathmanager};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

pub struct Config {
    pub working_path: PathBuf,
    pub hide_hidden_file: bool,
    pub config_file_path: PathBuf,

    pub history_path: PathBuf,
    pub command_history: Vec<String>,
    pub bookmark: Vec<String>,
}

impl Config {
    pub fn new() -> io::Result<Self> {
        let temp_folder = env::temp_dir();
        let nav_to_app_temp = pathmanager::convert_path_to_nav("luru")?;
        let temp_path = pathmanager::resolve_path(&temp_folder, &nav_to_app_temp)?;

        let nav_to_app_command_history = pathmanager::convert_path_to_nav("luru-cmd.log")?;
        let history_path = pathmanager::resolve_path(&temp_folder, &nav_to_app_command_history)?;

        // let file = File::open(&temp_path);
        // if let Err(_) = file {
        //     let content = format!("WORKING_PATH=");
        //     fs::write(&temp_path, content)?;
        // }

        // let file = File::open(&history_path);
        // if let Err(_) = file {
        //     fs::write(&history_path, "")?;
        // }

        Ok(Config {
            working_path: PathBuf::from("/"),
            command_history: Vec::new(),
            bookmark: Vec::new(),
            hide_hidden_file: true,
            config_file_path: temp_path,
            history_path,
        })
    }

    pub fn load(&mut self) -> io::Result<()> {
        // let file = File::open(&self.config_file_path)?;
        // let reader = BufReader::new(file);

        let file = filemanager::read_file(&self.config_file_path)?;
        for line in file.split("\n") {
            // let line = line;
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
                "HIDE_HIDDEN_FILE" => {
                    self.hide_hidden_file = vv[1].parse().unwrap_or(false);
                }
                "BOOKMARK" => {
                    let val = vv[1];
                    self.bookmark = val.to_string().split(";").map(|s| s.to_string()).collect();
                }
                _ => {}
            }
        }

        // load history
        self.load_history()
    }

    pub fn load_history(&mut self) -> io::Result<()> {
        // let file = File::open(&self.history_path).unwrap();
        // let reader = BufReader::new(file);
        let file = filemanager::read_file(&self.history_path)?;

        for line in file.split('\n') {
            let line = line.to_string();
            self.command_history.push(line);
        }

        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        let content = format!(
            "WORKING_PATH={}\nHIDE_HIDDEN_FILE={}\nBOOKMARK={}",
            self.working_path.display(),
            self.hide_hidden_file.to_string(),
            self.bookmark.join(";")
        );
        fs::write(&self.config_file_path, content)?;

        fs::write(&self.history_path, self.command_history.join("\n"))?;

        Ok(())
    }

    pub fn set_working_path<P: AsRef<Path>>(&mut self, wp: &P) {
        self.working_path = wp.as_ref().to_path_buf();
    }
}
