use std::{
    io::{self, Stdout, Write},
    path::PathBuf,
};

use crossterm::{
    cursor,
    execute,
    // queue,
    style::{self, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};

fn str_slince(arg: &str, from: usize, to: usize) -> String {
    arg.chars().collect::<Vec<_>>()[from..to]
        .iter()
        .cloned()
        .collect::<String>()
}

fn get_char_len(arg: &str) -> usize {
    arg.chars().collect::<Vec<_>>().len()
}

// #[derive(Clone, Copy)]
pub struct UI {
    pub stdout: Stdout,
    pub window_size: (u16, u16), // - w, h
    pub safe_height: (u16, u16), // -> h-min, h-max

    pub content_cursor: usize,
    pub content_render_from: usize,
    pub content_render_items: u16,

    // command_label: String,
    path_label: PathBuf,
    desc_label: String,
}

impl UI {
    pub fn new() -> UI {
        UI {
            stdout: io::stdout(),
            window_size: (0, 0),
            safe_height: (0, 0),
            content_cursor: 0,
            content_render_from: 0,
            content_render_items: 0,
            // command_label: String::from("cmd label"),
            path_label: PathBuf::new(),
            desc_label: String::from("description label"),
        }
    }
    pub fn begin(&mut self) -> io::Result<()> {
        let _ = terminal::enable_raw_mode()?;
        self.raw_mode(true)?;
        self.window_size = terminal::size()?;
        self.safe_height = (2, self.window_size.1 - 3);
        self.content_cursor = 0;
        self.content_render_from = 0;
        self.content_render_items = self.safe_height.1 - 2;

        if self.content_render_items < 5 {
            self.end()?;
            panic!("UI: unsupport terminal size!\nterminal heigth unsupported.")
        }

        self.clear_screen()?;

        Ok(())
    }

    pub fn raw_mode(&mut self, mode: bool) -> io::Result<()> {
        if mode {
            terminal::enable_raw_mode()?;
        } else {
            terminal::disable_raw_mode()?;
        }

        Ok(())
    }

    pub fn end(&mut self) -> io::Result<()> {
        self.clear_screen()?;
        self.move_cursor(0, 0)?;
        terminal::disable_raw_mode()?;
        self.raw_mode(false)?;
        self.stdout.flush()?;

        Ok(())
    }

    // pub fn set_safe_height(&mut self, min_height: u16, max_height: u16) {
    //     self.safe_height = (min_height, max_height);
    // }

    pub fn print(&mut self, content: &String) -> io::Result<()> {
        execute!(self.stdout, style::Print(&content))?;
        Ok(())
    }

    pub fn set_window_size(&mut self, width: u16, heigh: u16) {
        self.window_size = (width, heigh);
        self.safe_height = (2, heigh - 3);
    }

    pub fn clear_screen(&mut self) -> io::Result<()> {
        execute!(
            self.stdout,
            ResetColor,
            terminal::Clear(terminal::ClearType::All)
        )?;

        Ok(())

        // replace a screen with whitespace

        // let mut s = String::from(" ");
        //for _ in 0..self.window_size.1 {
        // s.push();
        //   s.push('\n');
        // }
        // s = s.repeat(self.window_size.0 as usize);
        // s.push_str("\n");
        //s = s.repeat(self.window_size.1 as usize);
        // self.move_cursor(0, 0);
        // execute!(self.stdout, ResetColor, style::Print(s))?;
    }
    pub fn move_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        execute!(self.stdout, cursor::MoveTo(x, y))?;
        Ok(())
    }

    pub fn set_frame_content(
        &mut self,
        path_label: PathBuf,
        desc_label: String,
        // command_label: String,
    ) -> io::Result<()> {
        self.path_label = path_label;
        self.desc_label = desc_label;
        // self.command_label = command_label;

        self.render_frame()?;
        Ok(())
    }

    fn trim_to_window_width(&self, s: &str) -> String {
        let max_width = self.window_size.0 as usize;
        if s.len() <= max_width {
            s.to_string()
        } else {
            s[s.len() - max_width..].to_string()
        }
    }

    pub fn render_frame(&mut self) -> io::Result<()> {
        // self.clear_screen()?;
        self.move_cursor(0, 0)?;

        // let mut path_label_coolor = style::Color::DarkGreen;
        // if !self.path_label.exists() {
        //     path_label_coolor = style::Color::DarkRed;
        // }

        //
        // let path_display = self.path_label.to_str().unwrap();
        // let mut display_from = 0;

        // if path_display.len() > self.window_size.0 as usize {
        //     display_from = get_char_len(&path_display) - (self.window_size.0 as usize)
        // }

        let path_color = if self.path_label.exists() {
            style::Color::DarkGreen
        } else {
            style::Color::DarkRed
        };

        let path_str = self.path_label.to_string_lossy();
        let trimmed_path = self.trim_to_window_width(&path_str);

        execute!(
            self.stdout,
            SetForegroundColor(style::Color::Black),
            SetBackgroundColor(path_color),
            style::Print(trimmed_path),
            style::ResetColor,
        )?;

        // self.stdout.write(self.path_label.as_bytes()).unwrap();
        // execute!(
        //     self.stdout,
        //     SetForegroundColor(style::Color::Black),
        //     SetBackgroundColor(path_label_coolor),
        //     style::Print(format!(
        //         "{}",
        //         // display_from,
        //         // path_display
        //         str_slince(&path_display, display_from, get_char_len(&path_display)) // &(path_display[display_from..path_display.len()])
        //     )),
        //     style::ResetColor,
        // )
        // .unwrap();

        self.move_cursor(0, self.window_size.1 - 2)?;
        // self.stdout.write(self.desc_label.as_bytes()).unwrap();
        execute!(
            self.stdout,
            style::ResetColor,
            style::Print(&self.desc_label)
        )?;

        self.move_cursor(0, self.window_size.1 - 1)?;
        execute!(
            self.stdout,
            SetForegroundColor(style::Color::DarkMagenta),
            style::Print("$ "),
            style::ResetColor,
        )?;
        // self.stdout.write(self.command_label.as_bytes()).unwrap();

        self.stdout.flush()?;
        Ok(())
    }

    pub fn render_content(&mut self, content: &Vec<String>) -> io::Result<()> {
        // self.clear_screen();
        self.render_frame()?;

        if self.content_cursor > content.len() {
            self.content_cursor = 0;
        }

        // if self
        //     .content_render_from
        //     .wrapping_add(self.content_render_items as usize)
        //     >= self.content_cursor
        if self.content_cursor <= self.content_render_from {
            if self.content_cursor > 0 {
                self.content_render_from = self.content_cursor - 1;
            } else if self.content_cursor == 0 {
                self.content_render_from = 0;
            }
            // (fr + 2)
        }

        if self.content_cursor >= self.content_render_from + (self.content_render_items as usize)
            && self.content_cursor < content.len()
        {
            self.content_render_from =
                self.content_cursor - (self.content_render_items as usize) + 1;
        }

        for (i, val_r) in content
            .iter()
            // .skip(self.content_render_from as usize)
            // .take(self.content_render_items as usize)
            .enumerate()
        {
            if i < self.content_render_from as usize {
                continue;
            };

            let p = get_char_len(val_r).min(self.window_size.0 as usize);

            let mut val = format!(
                "{}",
                // i,
                str_slince(val_r, 0, p) // &(val_r[0..val_r.len().min(self.window_size.0.wrapping_sub(3) as usize)])
            );

            if p < self.window_size.0 as usize {
                val.push_str(
                    String::from(" ")
                        .repeat(self.window_size.0 as usize - p)
                        .as_str(),
                );
            }

            let x = 0;
            let y = (1 + i - self.content_render_from) as u16;

            if y > self.content_render_items.wrapping_add(1) {
                continue;
            };

            self.move_cursor(x, y)?;
            if y == 1 && self.content_render_from > 0 {
                execute!(
                    self.stdout,
                    // SetBackgroundColor(style::Colo)
                    SetForegroundColor(style::Color::DarkGrey),
                    style::Print(format!("...{} items", self.content_render_from)),
                    style::ResetColor,
                )?;
            } else if y == self.content_render_items.wrapping_add(1)
                && content.len() - (self.content_render_from + self.content_render_items as usize)
                    != 0
            {
                execute!(
                    self.stdout,
                    SetForegroundColor(style::Color::DarkGrey),
                    style::Print(format!(
                        "...{} items",
                        content.len().wrapping_sub(
                            self.content_render_from + self.content_render_items as usize
                        )
                    )),
                    style::ResetColor,
                )?;
            } else if i == self.content_cursor as usize {
                execute!(
                    self.stdout,
                    // SetForegroundColor(style::Color::White),
                    style::Print("\x1b[95m\x1b[1m"),
                    style::Print(val),
                    style::ResetColor,
                )?;
            } else {
                execute!(
                    self.stdout,
                    style::ResetColor,
                    style::Print(format!("\x1b[2m{}\x1b[0m", val))
                )?;
            }
        }

        execute!(self.stdout, style::ResetColor)?;

        self.stdout.flush()?;

        Ok(())
    }

    pub fn print_term_start(&mut self, path_label: &String) -> io::Result<()> {
        execute!(
            self.stdout,
            style::ResetColor,
            SetForegroundColor(style::Color::DarkGreen),
            style::Print("\n[Luru]"),
            SetForegroundColor(style::Color::DarkBlue),
            style::Print(format!(" {}>", &path_label)),
            SetForegroundColor(style::Color::DarkMagenta),
            style::Print("\n$ "),
            style::ResetColor,
            // style::Print(format!("{}\n", &command_label)),
        )?;
        self.stdout.flush()?;
        Ok(())
    }
    pub fn print_term_end(&mut self) -> io::Result<()> {
        execute!(
            self.stdout,
            style::ResetColor,
            SetForegroundColor(style::Color::DarkGreen),
            style::Print("\n[Program Ended]"),
            SetForegroundColor(style::Color::DarkBlue),
            style::Print("Press Enter to close"),
            style::ResetColor,
        )?;
        self.stdout.flush()?;

        Ok(())
    }
}
