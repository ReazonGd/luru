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
    pub fn new(start_location: &PathBuf) -> App {
        let mut decs_label = format!("type ctrl + c, or :q to exit");
        let mut path = start_location.clone();
        let mut config = Config::new();
        config.load();

        if path.eq(&PathBuf::new()) {
            // path = env::current_dir().unwrap();
            path = config.working_path.clone();
        }

        if !path.exists() {
            decs_label = format!("cannot open {} directing to /", start_location.display());
            path =
                pathmanager::resolve_path(&start_location, &pathmanager::NavigationCommand::Root)
                    .unwrap();
        }

        // config.set_working_path(&path);

        App {
            current_path: path.clone(),
            temp_path: PathBuf::from("/"),
            app_ui: UI::new(),
            app_term: Termin::new(),
            app_mode: AppMode::Normal,
            decs_label,
            command: String::new(),
            content: Vec::<ReadDirItems>::new(),
            command_history: Vec::<String>::new(),
            content_to_read: Vec::<String>::new(),
            config,
            x_cursor: 0,
            re_read: true,
            quit: false,
            sugest: String::new(),
        }
    }

    // scan, organize,
    pub fn do_a_scan(&mut self) {
        let mut res: Vec<ReadDirItems> = Vec::new();
        let mut dirs: Vec<ReadDirItems> = Vec::new();
        let mut files: Vec<ReadDirItems> = Vec::new();

        let r = filemanager::read_dir(&self.current_path);

        if let Err(e) = r {
            self.decs_label = format!("got an error! kind of:{}", e.kind());
            res.clear();
            res.push(ReadDirItems {
                path: self.temp_path.clone(),
                navigation_type: pathmanager::NavigationCommand::Absolute(self.temp_path.clone()),
                file_name: String::from("/"),
                label: String::from(self.temp_path.to_str().unwrap()),
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
                    navigation_type: pathmanager::convert_path_to_nav("../").unwrap(),
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
            self.config.set_working_path(&self.current_path);
            self.config.save();
        }

        self.content = res;
        self.content_to_read = self.content.iter().map(|f| f.label.clone()).collect();

        // Ok(res)
    }

    pub fn open_dir(&mut self) {
        let path_selected = &self.content[self.app_ui.content_cursor];
        if path_selected.path.is_dir() {
            let new_path_nav = &path_selected.navigation_type;
            self.current_path =
                pathmanager::resolve_path(&self.current_path, &new_path_nav).unwrap();

            self.re_read = true;
        }
        if path_selected.path.is_file() {
            let meta = filemanager::metadata(&path_selected.path).unwrap();
            // set desc to file information
            self.decs_label = format!(
                "file: {} size: {} modified: {}",
                meta.display_name, meta.size, meta.modified
            );
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.app_ui.begin();

        ctrlc::set_handler(move || {
            print!("^C");
        })
        .expect("error setting ctrl+c handler");

        while !self.quit {
            if self.app_mode == AppMode::TerminalOnly {
                self.termin_run();
            }

            if self.re_read {
                let is_cd: bool = !self.current_path.eq(&self.temp_path);
                self.do_a_scan();

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
                self.re_read = false;
            }

            // self.counter = self.counter + 1;

            self.display_ui();

            // wait until event
            let event: event::Event = keyboard::wait_for_keyboard().unwrap();
            self.event_handler(event);
        }

        Ok(())
    }

    fn find_sugest(&mut self) {
        // mencari kata yang ada posisi x
        let words = self.command.split(" ");
        let mut word: String = String::new();
        let mut pos_x2 = self.x_cursor as isize;

        for (_, w) in words.enumerate() {
            pos_x2 -= w.len() as isize + 1;

            if pos_x2 <= 0 {
                word = w.to_string();
                break;
            }
        }

        if word.starts_with("./") {
            word = word.get(2..).unwrap().to_string();
        }

        let mut filename_selected = String::new();
        for i in self.content.iter() {
            if i.file_name.starts_with(&word) && !i.file_name.eq(&word) {
                filename_selected = i.file_name.clone();
                break;
            }
        }

        let s = word.len();
        let n = filename_selected.len();
        if s >= 1 && n >= 1 {
            self.sugest = filename_selected.get(s..n).unwrap().to_string();
        }

        let mut test = word.clone();
        test.push_str(&self.sugest.as_str());

        if !test.eq(&filename_selected) {
            self.sugest = String::new();
        }

        if self.sugest.len() > 0 {
            self.decs_label = format!("sugest : {}\x1b[035m{}\x1b[0m", word, self.sugest);
        }
        // self.decs_label = format!("sugest : ", word, self.sugest);
    }

    fn display_ui(&mut self) {
        self.decs_label = format!("type 'exit' to exit");
        match self.app_mode {
            AppMode::Normal => {
                self.find_sugest();
                self.app_ui
                    .set_frame_content(self.current_path.clone(), self.decs_label.clone());

                self.app_ui.render_content(&self.content_to_read);

                // self.move_cursor(0, self.window_size.1.wrapping_sub(1));
                self.app_ui.move_cursor(2, self.app_ui.window_size.1);
                // self.app_ui.print(content);
                self.app_ui.print(&self.command);
                self.app_ui
                    .move_cursor(self.x_cursor + 2, self.app_ui.window_size.1);
            }
            _ => {}
        }
    }

    fn termin_run(&mut self) {
        self.app_ui.end();
        print!("Attention: This not terminal. first word wil be command and other will be args. (&& and other keys is not implemented yet)");
        while self.app_mode == AppMode::TerminalOnly {
            self.app_ui
                .print_term_start(&format!("{}", self.config.working_path.display()));
            io::stdin().read_line(&mut self.command).unwrap();
            // print!("execute : {}\n", &self.command);
            // let mut d = String::new();
            self.command_handler();
        }
        self.app_ui.begin();
        self.current_path = env::current_dir().unwrap();
        self.re_read = true;
    }

    fn event_handler(&mut self, event: event::Event) {
        match event {
            Event::Key(key) => self.key_event_handler(key),
            Event::Resize(width, height) => {
                self.app_ui.set_window_size(width, height);
                self.re_read = true;
            }
            _ => {}
        }
    }
    fn key_event_handler(&mut self, key_event: KeyEvent) {
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
                    self.app_ui.clear_screen();
                } else {
                    // self.command.push(ch);
                    self.command.insert(self.x_cursor as usize, ch);
                    self.x_cursor += 1;
                }
            }
            KeyCode::Up => {
                if self.app_ui.content_cursor > 0 {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.app_ui.content_cursor = (self.app_ui.content_cursor as usize)
                            .wrapping_sub(self.app_ui.content_cursor.min(5))
                    } else {
                        self.app_ui.content_cursor = self.app_ui.content_cursor.wrapping_sub(1);
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
                let max_cursor = self.content.len().wrapping_sub(1);
                if self.app_ui.content_cursor < max_cursor {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.app_ui.content_cursor = (self.app_ui.content_cursor as usize)
                            .wrapping_add(
                                max_cursor.wrapping_sub(self.app_ui.content_cursor).min(5),
                            )
                    } else {
                        self.app_ui.content_cursor = self.app_ui.content_cursor.wrapping_add(1);
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

                    self.command.push_str(&ns.as_str());
                    self.x_cursor += ns.len() as u16;
                }
            }

            KeyCode::Enter => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.open_dir();
                } else {
                    self.command_history.push(self.command.clone());

                    self.command_handler();
                    self.x_cursor = 0;
                }
            }
            _ => {}
        }
    }

    fn command_handler(&mut self) {
        //
        match self.command.as_str() {
            "exit" | "quit" | ":q" => {
                self.quit = true;
            }

            "back" | ":b" => {
                if !self.current_path.eq(&PathBuf::from("/")) {
                    // let new_path_nav = N;
                    self.current_path = pathmanager::resolve_path(
                        &self.current_path,
                        &pathmanager::convert_path_to_nav("../").unwrap(),
                    )
                    .unwrap();

                    self.re_read = true;
                }
            }

            s if s.starts_with("luru") | s.starts_with("sudo luru") => {
                // self.termin_run();
                self.app_mode = AppMode::Normal;
            }
            "" | ":o" | "open" => {
                // let path_selected = &self.content[self.app_ui.content_cursor];
                self.open_dir();
            }

            s if s.starts_with("cd ") => {
                let cd_args: Vec<&str> = s.split(" ").collect();
                if cd_args.len() < 1 {
                    self.decs_label = String::from("invalid argument");
                } else {
                    let nav_cmd = pathmanager::convert_path_to_nav(cd_args[1]).unwrap();
                    self.current_path =
                        pathmanager::resolve_path(&self.current_path, &nav_cmd).unwrap();

                    self.re_read = true;
                }
            }
            s if s.starts_with(":nf") => {
                let cd_args: Vec<&str> = s.split(" ").collect();
                if cd_args.len() < 1 {
                    self.decs_label = String::from("invalid argument");
                } else {
                    let nav_cmd = pathmanager::convert_path_to_nav(cd_args[1]).unwrap();
                    let path = pathmanager::resolve_path(&self.current_path, &nav_cmd).unwrap();

                    filemanager::make_empty_file(path);

                    self.re_read = true;
                }
            }
            _ => {
                if self.app_mode == AppMode::TerminalOnly {
                    self.app_term.run(self.command.clone()).unwrap();
                } else {
                    self.app_ui.end();
                    self.app_ui.print_term_start(
                        &format!("{}", self.current_path.display()),
                        // &self.command,
                    );
                    print!("{} \n", &self.command);
                    self.app_term.run(self.command.clone()).unwrap();
                    self.app_ui.print_term_end();
                    let mut d = String::new();
                    io::stdin().read_line(&mut d).unwrap();
                    self.app_ui.begin();
                    self.current_path = env::current_dir().unwrap();

                    // self.current_path = self.app_term.running_path.clone();
                }

                self.re_read = true;
            }
        }

        self.command = String::new();
    }

    pub fn end(&mut self) {
        self.app_ui.end();
        println!(
            "JOURNAL:\n\nlast path :\n{}",
            self.current_path.display(),
            // self.temp_path.display()
        );
    }
}
