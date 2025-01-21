use core::time;
use crossterm::event::{poll, read, Event};
use std::io;

pub fn wait_for_keyboard() -> io::Result<Event> {
    loop {
        // thread::sleep(time::Duration::from_millis(33));
        if poll(time::Duration::from_millis(33)).unwrap() {
            return Ok(read()?);
        }
    }
}
