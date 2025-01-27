use std::{
    io,
    process::{Command, Stdio},
};

pub struct Termin {}

impl Termin {
    pub fn new() -> Termin {
        Termin {}
    }

    pub fn run(&mut self, input: String) -> io::Result<bool> {
        if input.is_empty() {
            return Ok(false);
        }

        let stdin = Stdio::inherit();
        let stdout = Stdio::inherit();

        let output = Command::new("sh")
            .arg("-c")
            .arg(input)
            .stdin(stdin)
            .stdout(stdout)
            .spawn();

        match output {
            Ok(mut output) => {
                output.wait()?;
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        };
        Ok(true)
    }
}
