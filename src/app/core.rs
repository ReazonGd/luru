use std::{env, io, path::PathBuf};

use super::{
    config::Config,
    filemanager::{self, ReadDirItems},
    keyboard, pathmanager,
    termin::Termin,
    ui::UI,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ctrlc;
#[derive(PartialEq)]
enum AppMode {
    Normal,
    CommandHistory,
    TerminalOnly,
}

pub struct App {
    current_path: PathBuf,
    temp_path: PathBuf,

    app_ui: UI,
    app_term: Termin,
    decs_label: String,
    command: String,
    sugest: String,
    content: Vec<ReadDirItems>,
    content_to_read: Vec<String>,
    command_history: Vec<String>,

    config: Config,

    app_mode: AppMode,

    x_cursor: u16,
    re_read: bool,
    quit: bool,
}

impl App {
    pub fn new(start_location: &PathBuf) -> io::Result<App> {
        let mut decs_label = format!("\x1b[1m\x1b[035m{}\x1b[0m exit \x1b[1m\x1b[035m{}\x1b[0m terminal only \x1b[1m\x1b[035m{}\x1b[0m command historys", "^c", "^t", "^h");
        let mut path = start_location.clone();
        let mut config = Config::new()?;
        config.load()?;

        if path.eq(&PathBuf::new()) {
            // path = env::current_dir().unwrap();
            path = config.working_path.clone();
        }

        if !path.exists() {
            decs_label = format!(
                "\x1b[97m\x1b[41mcannot open {} directing to /\x1b[37m",
                start_location.display()
            );
            path =
                pathmanager::resolve_path(&start_location, &pathmanager::NavigationCommand::Root)?;
        }

        // config.set_working_path(&path);

        Ok(App {
            current_path: path.clone(),
            temp_path: PathBuf::from("/"),
            app_ui: UI::new(),
            app_term: Termin::new(),
            app_mode: AppMode::Normal,
            decs_label,
            command: String::new(),
            content: Vec::<ReadDirItems>::new(),
            command_history: config.command_history.clone(),
            content_to_read: Vec::<String>::new(),
            config,
            x_cursor: 0,
            re_read: true,
            quit: false,
            sugest: String::new(),
        })
    }

    // scan, organize,
    pub fn do_a_scan(&mut self) -> io::Result<()> {
        let mut res: Vec<ReadDirItems> = Vec::new();
        let mut dirs: Vec<ReadDirItems> = Vec::new();
        let mut files: Vec<ReadDirItems> = Vec::new();

        let r = filemanager::read_dir(&self.current_path);

        if let Err(e) = r {
            self.decs_label = format!("\x1b[97m\x1b[41mgot an error! kind of:{}\x1b[0m", e.kind());
            res.clear();
            res.push(ReadDirItems {
                path: self.temp_path.clone(),
                navigation_type: pathmanager::NavigationCommand::Absolute(self.temp_path.clone()),
                file_name: String::from("/"),
                label: String::from(self.temp_path.to_string_lossy()),
            });
        } else if let Ok(o) = r {
            for v in o {
                if v.path.is_dir() {
                    dirs.push(v);
                } else if v.path.is_file() {
                    files.push(v);
                }
            }

            if !self.current_path.eq(&PathBuf::from("/")) {
                res.push(ReadDirItems {
                    path: PathBuf::from("../"),
                    navigation_type: pathmanager::convert_path_to_nav("../")?,
                    file_name: String::from("../"),
                    label: String::from("../"),
                });
            }

            dirs.sort_by(|a, b| a.label.cmp(&b.label.clone()));
            files.sort_by(|a, b| a.label.cmp(&b.label.clone()));

            res.append(&mut dirs);
            res.append(&mut files);

            self.temp_path = self.current_path.clone();
            let _ = env::set_current_dir(&self.current_path);
            //self.config.set_working_path(&self.current_path);
            //self.config.save();
        }

        self.content = res;
        self.content_to_read = self.content.iter().map(|f| f.label.clone()).collect();

        Ok(())
    }

    pub fn open_dir(&mut self) -> io::Result<()> {
        let path_selected = &self.content[self.app_ui.content_cursor];
        if path_selected.path.is_dir() {
            let new_path_nav = &path_selected.navigation_type;
            self.current_path = pathmanager::resolve_path(&self.current_path, &new_path_nav)?;

            self.re_read = true;
        }
        if path_selected.path.is_file() {
            let meta = filemanager::metadata(&path_selected.path)?;
            // set desc to file information
            self.decs_label = format!(
                "\x1b[34mfile: {} size: {} modified: {}\x1b[0m",
                meta.display_name, meta.size, meta.modified
            );
        }

        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.app_ui.begin()?;

        ctrlc::set_handler(move || {
            print!("^C");
        })
        .expect("error setting ctrl+c handler");

        while !self.quit {
            if self.app_mode == AppMode::TerminalOnly {
                self.termin_run()?;
            }

            if self.re_read {
                let is_cd: bool = !self.current_path.eq(&self.temp_path);

                match self.app_mode {
                    AppMode::Normal => {
                        self.do_a_scan()?;
                        if is_cd {
                            if self.content.len() > 2 {
                                self.app_ui.content_cursor = 1;
                            } else {
                                self.app_ui.content_cursor = 0;
                            }
                            self.app_ui.content_render_from = 0;
                        } else {
                            // self.app_ui.content_cursor = 0;
                            // self.app_ui.content_render_from = 0;
                        }

                        self.content_to_read =
                            self.content.iter().map(|f| f.label.clone()).collect();
                    }
                    AppMode::CommandHistory => {
                        self.content_to_read = self.command_history.clone();
                        self.app_ui.content_render_from = 0;
                        self.app_ui.content_cursor = 0;
                    }
                    _ => {}
                }

                self.app_ui.clear_screen()?;
                self.re_read = false;
            }

            // self.counter = self.counter + 1;

            self.display_ui()?;

            // wait until event
            let event: event::Event = keyboard::wait_for_keyboard()?;
            self.event_handler(event)?;
        }

        Ok(())
    }

    fn find_sugest(&mut self) -> io::Result<()> {
        if (self.x_cursor as usize) < self.command.len() {
            return Ok(());
        }

        let words = self.command.split(" ");
        let mut word: String = String::new();
        let mut pos_x2 = self.x_cursor as isize;

        for (_, w) in words.enumerate() {
            pos_x2 -= w.len() as isize + 1;

            if pos_x2 < 0 {
                word = w.to_string();
                break;
            }
        }

        if word.starts_with("./") {
            word = word.get(2..).unwrap_or("").to_string();
        } else if word.starts_with("/") {
            word = word.get(1..).unwrap_or("").to_string();
        }

        let mut sugested_word = String::new();
        for i in self.content.iter() {
            if i.file_name.starts_with(&word) && !i.file_name.eq(&word) {
                sugested_word = i.file_name.clone();
                break;
            }
        }

        // find from cmd history
        let mut is_cmd = false;
        if sugested_word.is_empty() {
            for i in self.command_history.iter() {
                if i.starts_with(&self.command) && !i.eq(&self.command) {
                    sugested_word = i.clone();
                    is_cmd = true;
                    break;
                }
            }
        }

        let mut test = if is_cmd {
            self.command.clone()
        } else {
            word.clone()
        };

        let s = test.len();
        let n = sugested_word.len();

        if s >= 1 && n >= 1 {
            self.sugest = sugested_word.get(s..n).unwrap_or("").to_string();
        }

        test.push_str(&self.sugest.as_str());

        if !test.eq(&sugested_word) {
            self.sugest = String::new();
        }

        if self.sugest.len() > 0 {
            self.app_ui
                .move_cursor(self.x_cursor + 2, self.app_ui.window_size.1)?;
            self.app_ui
                .print(&format!("\x1b[035m{}\x1b[0m", self.sugest))?;
        }

        Ok(())
        // self.decs_label = format!("sugest : ", word, self.sugest);
    }

    fn display_ui(&mut self) -> io::Result<()> {
        // self.decs_label = format!("type 'exit' to exit");

        self.app_ui
            .set_frame_content(self.current_path.clone(), self.decs_label.clone())?;

        self.app_ui.render_content(&self.content_to_read)?;

        // self.move_cursor(0, self.window_size.1.wrapping_sub(1));
        self.app_ui.move_cursor(2, self.app_ui.window_size.1)?;
        // self.app_ui.print(content);
        self.app_ui.print(&format!(
            "{}\x1b[2m{}\x1b[0m",
            &self.command,
            String::from("/")
                .repeat(self.app_ui.window_size.0 as usize - (self.command.len() + 3))
                .as_str()
        ))?;

        self.find_sugest()?;
        self.app_ui
            .move_cursor(self.x_cursor + 2, self.app_ui.window_size.1)?;
        Ok(())
    }

    /* push to cmd history and check duplicate. if duplicate, move to top */
    fn push_cmd_to_history(&mut self) {
        if self.command.trim().is_empty() {
            return;
        }

        let cmd = self.command.trim().to_string();
        if self.command_history.contains(&cmd) {
            self.command_history.retain(|x| x != &cmd);
        }
        self.command_history.insert(0, cmd);
    }

    fn termin_run(&mut self) -> io::Result<()> {
        self.app_ui.clear_screen()?;
        self.app_ui.move_cursor(0, 0)?;
        self.app_ui.raw_mode(false)?;
        // self.app_ui.end()?;
        print!("Any command will run with \x1b[1m\x1b[093m\"sh -c [cmd]\"\x1b[0m you can type \x1b[1m\x1b[035mluru\x1b[0m or \x1b[1m\x1b[035mexit\x1b[0m to back");
        while self.app_mode == AppMode::TerminalOnly {
            self.app_ui
                .print_term_start(&format!("{}", self.current_path.display()))?;
            io::stdin().read_line(&mut self.command)?;

            self.command_handler()?;
        }
        self.app_ui.raw_mode(true)?;
        // self.app_ui.begin()?;
        self.current_path = env::current_dir()?;
        self.re_read = true;
        Ok(())
    }

    fn event_handler(&mut self, event: event::Event) -> io::Result<()> {
        match event {
            Event::Key(key) => self.key_event_handler(key)?,
            Event::Resize(width, height) => {
                self.app_ui.set_window_size(width, height);
                self.re_read = true;
            }
            _ => {}
        }

        Ok(())
    }
    fn key_event_handler(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char(ch) => {
                let is_control_pressed: bool = key_event.modifiers.contains(KeyModifiers::CONTROL);
                if is_control_pressed {
                    match ch {
                        'c' => self.quit = true,
                        'h' => self.app_mode = AppMode::CommandHistory,
                        'f' => self.app_mode = AppMode::Normal,
                        't' => self.app_mode = AppMode::TerminalOnly,
                        _ => {}
                    }
                    self.re_read = true;
                    // self.app_ui.clear_screen()?;
                } else {
                    // self.command.push(ch);
                    self.command.insert(self.x_cursor as usize, ch);
                    self.x_cursor += 1;
                }
            }
            KeyCode::Esc => {
                self.app_mode = AppMode::Normal;
                self.re_read = true;
            }
            KeyCode::Up => {
                if self.app_ui.content_cursor > 0 {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.app_ui.content_cursor = (self.app_ui.content_cursor as usize)
                            - (self.app_ui.content_cursor.min(5))
                    } else {
                        self.app_ui.content_cursor = self.app_ui.content_cursor - (1);
                    }
                }
            }

            KeyCode::Left => {
                if self.x_cursor > 0 {
                    self.x_cursor -= 1;
                }
            }

            KeyCode::Right => {
                if (self.x_cursor as usize) < self.command.chars().count().min(u16::MAX as usize) {
                    self.x_cursor += 1;
                }
            }

            KeyCode::Down => {
                let max_cursor = self.content_to_read.len() - 1;
                if self.app_ui.content_cursor < max_cursor {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.app_ui.content_cursor = (self.app_ui.content_cursor as usize)
                            + (max_cursor - (self.app_ui.content_cursor)).min(5)
                    } else {
                        self.app_ui.content_cursor = self.app_ui.content_cursor + (1);
                    }
                }
            }
            KeyCode::Backspace => {
                if self.x_cursor > 0 {
                    // self.command.pop();
                    self.command.remove((self.x_cursor - 1) as usize);
                    if self.x_cursor > 0 {
                        self.x_cursor -= 1;
                    }
                }
            }
            KeyCode::Tab => {
                if self.app_mode == AppMode::CommandHistory {
                    let selected_cmd = &self.command_history[self.app_ui.content_cursor];
                    self.command = selected_cmd.clone();
                } else {
                    if self.sugest.len() > 0 {
                        self.command
                            .insert_str(self.x_cursor as usize, &self.sugest);
                        self.x_cursor += self.sugest.len() as u16;
                    } else {
                        let path_selected = &self.content[self.app_ui.content_cursor];
                        let mut ns = String::new();

                        if !path_selected.file_name.eq("../") {
                            ns.push_str("./");
                        }

                        ns.push_str(&path_selected.file_name);
                        ns = ns.replace(" ", "\\ ");

                        self.command.push_str(&ns.as_str());
                        self.x_cursor += ns.len() as u16;
                    }
                }
            }

            KeyCode::Enter => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.open_dir()?;
                } else {
                    // self.command_history.push(self.command.clone());
                    self.push_cmd_to_history();

                    self.command_handler()?;
                    self.x_cursor = 0;
                }
            }
            _ => {}
        }

        // if self.app_mode == AppMode::CommandHistory && self.command.is_empty() {

        // }

        Ok(())
    }

    fn command_handler(&mut self) -> io::Result<()> {
        //
        match self.command.trim() {
            "exit" | "quit" | ":q" => {
                if self.app_mode == AppMode::Normal {
                    self.quit = true;
                } else {
                    self.app_mode = AppMode::Normal;
                }
            }

            "back" | ":b" => {
                if !self.current_path.eq(&PathBuf::from("/")) {
                    // let new_path_nav = N;
                    self.current_path = pathmanager::resolve_path(
                        &self.current_path,
                        &pathmanager::convert_path_to_nav("../")?,
                    )?;

                    self.re_read = true;
                }
            }

            s if s.starts_with("luru") | s.starts_with("sudo luru") => {
                // self.termin_run();
                self.app_mode = AppMode::Normal;
            }
            "" | ":o" | "open" => {
                // let path_selected = &self.content[self.app_ui.content_cursor];
                self.open_dir()?;
            }

            s if s.starts_with("cd ") => {
                let cd_args: Vec<&str> = s.split(" ").collect();
                if cd_args.len() < 1 {
                    self.decs_label = String::from("invalid argument");
                } else {
                    let nav_cmd = pathmanager::convert_path_to_nav(cd_args[1])?;
                    self.current_path = pathmanager::resolve_path(&self.current_path, &nav_cmd)?;

                    self.re_read = true;
                }
            }
            s if s.starts_with(":nf") => {
                let cd_args: Vec<&str> = s.split(" ").collect();
                if cd_args.len() < 1 {
                    self.decs_label = String::from("invalid argument");
                } else {
                    let nav_cmd = pathmanager::convert_path_to_nav(cd_args[1])?;
                    let path = pathmanager::resolve_path(&self.current_path, &nav_cmd)?;

                    filemanager::make_empty_file(path);

                    self.re_read = true;
                }
            }
            _ => {
                if self.app_mode == AppMode::TerminalOnly {
                    self.app_term.run(self.command.clone())?;
                } else {
                    // self.app_ui.end()?;
                    self.app_ui.clear_screen()?;
                    self.app_ui.move_cursor(0, 0)?;
                    self.app_ui.raw_mode(false)?;
                    self.app_ui.print_term_start(
                        &format!("{}", self.current_path.display()),
                        // &self.command,
                    )?;
                    print!("{} \n", &self.command);
                    self.app_term.run(self.command.clone())?;
                    self.app_ui.print_term_end()?;
                    let mut d = String::new();
                    io::stdin().read_line(&mut d)?;
                    // self.app_ui.begin()?;
                    self.app_ui.raw_mode(true)?;
                    self.current_path = env::current_dir()?;

                    // self.current_path = self.app_term.running_path.clone();
                }

                self.re_read = true;
            }
        }

        self.command = String::new();
        Ok(())
    }

    pub fn end(&mut self) -> io::Result<()> {
        self.app_ui.end()?;
        self.config.set_working_path(&self.current_path);
        self.config.command_history = self.command_history.clone();
        self.config.save()?;
        println!(
            "JOURNAL:\n\nlast path :\n{}",
            self.current_path.display(),
            // self.temp_path.display()
        );
        Ok(())
    }
}
