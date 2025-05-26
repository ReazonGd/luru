use std::{
    io,
    process::{Command, Stdio},
};

use super::filemanager::MetadataInfo;
use base64::{engine::general_purpose::STANDARD, Engine};

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

    pub fn convert_run_option(&mut self, meta: &MetadataInfo, cmd: String) -> String {
        //
        let base64_filename = STANDARD.encode(meta.display_name.clone());
        let mut cmd = cmd.replace("{{path}}", format!("\"{}\"", meta.path.display()).as_str());
        cmd = cmd.replace("{{filename}}", &meta.display_name);
        cmd = cmd.replace("{{base64}}", &base64_filename);

        return cmd;
    }
}
