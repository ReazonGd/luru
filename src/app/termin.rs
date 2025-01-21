use std::{
    io,
    process::{Child, Command, Stdio},
};

pub struct Termin {}

impl Termin {
    pub fn new() -> Termin {
        Termin {}
    }

    pub fn run(&mut self, input: String) -> io::Result<bool> {
        // get command from user
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            if command.is_empty() {
                continue;
            }
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                Stdio::from(output.stdout.unwrap())
            });

            let stdout = if commands.peek().is_some() {
                // there is another command piped behind this one
                // prepare to send output to the next command
                Stdio::piped()
            } else {
                // there are no more commands piped behind this one
                // send output to shell stdout
                Stdio::inherit()
            };

            let output = Command::new(command)
                .args(args)
                .stdin(stdin)
                .stdout(stdout)
                .spawn();

            match output {
                Ok(output) => {
                    previous_command = Some(output);
                }
                Err(e) => {
                    previous_command = None;
                    eprintln!("{}", e);
                }
            };
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            let _ = final_command.wait();
        }

        Ok(true)
    }
}
