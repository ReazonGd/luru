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
    pub command_option_path: PathBuf,
    pub run_options: Vec<RunOption>,
}

#[derive(Clone, Debug)]
pub struct RunOptionCommand {
    pub name: String,
    pub run: String,
}
#[derive(Clone, Debug)]
pub struct RunOption {
    pub filetype_name: String,
    pub commands: Vec<RunOptionCommand>,
}

impl Config {
    pub fn new() -> io::Result<Self> {
        let nav_to_config_folder = pathmanager::convert_path_to_nav("~/.config/luru")?;
        let config_folder = pathmanager::resolve_path(&PathBuf::from("/"), &nav_to_config_folder)?;

        if !config_folder.exists() {
            fs::create_dir_all(&config_folder)?;
        }

        let nav_to_app_temp = pathmanager::convert_path_to_nav("luru")?;
        let temp_path = pathmanager::resolve_path(&config_folder, &nav_to_app_temp)?;

        let nav_to_app_command_history: pathmanager::NavigationCommand =
            pathmanager::convert_path_to_nav("luru-cmd.log")?;
        let history_path = pathmanager::resolve_path(&config_folder, &nav_to_app_command_history)?;

        let nav_to_app_command_option: pathmanager::NavigationCommand =
            pathmanager::convert_path_to_nav("luru-cmd-opt.txt")?;
        let command_option_path =
            pathmanager::resolve_path(&config_folder, &nav_to_app_command_option)?;

        Ok(Config {
            working_path: PathBuf::from("/"),
            command_history: Vec::new(),
            bookmark: Vec::new(),
            hide_hidden_file: true,
            config_file_path: temp_path,
            history_path,
            command_option_path,
            run_options: vec![],
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
        self.load_history()?;
        self.load_command_option()?;
        Ok(())
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

    pub fn load_command_option(&mut self) -> io::Result<()> {
        let file;
        if !self.config_file_path.exists() {
            let text = r#"# line start with hashtag will be ignored
# line start with @ start new type for run option. and @ again to end it
# a run option separated by "=". first one is name, and run script  

# a template surounded by {{name}} please do exact for replacement 
# {{path}} will replaced to full path  
# {{filename}} will replaced to file name, example: "note.txt"
# {{base64}} will replaced to file name encoded in base64

@ .
nano=nano {{path}}

@ txt
nano=nano {{path}}

# ends
@"#;
            let content = format!("{}", text);
            file = content.clone();
            fs::write(&self.command_option_path, content)?;
        } else {
            file = filemanager::read_file(&self.command_option_path)?;
        }

        //
        self.run_options = vec![];
        let mut curent_run_option: RunOption = RunOption {
            filetype_name: "".to_string(),
            commands: vec![],
        };

        for line in file.split('\n') {
            let line = line.to_string();
            if line.starts_with("#") {
                continue;
            } else if line.starts_with("@") {
                if !curent_run_option.filetype_name.is_empty()
                    && curent_run_option.commands.len() > 0
                {
                    self.run_options.push(curent_run_option.clone());
                    curent_run_option = RunOption {
                        filetype_name: "".to_string(),
                        commands: vec![],
                    };
                }
                let filetypename = &line[1..].trim().to_owned();
                curent_run_option.filetype_name = filetypename.to_lowercase().clone();
            } else if line.contains("=") {
                let rows: Vec<&str> = line.split("=").collect();
                curent_run_option.commands.push(RunOptionCommand {
                    name: format!("{}", rows[0]),
                    run: format!("{}", rows[1]),
                });
            }
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
