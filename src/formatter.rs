use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::from_utf8;

use eyre::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct Formatter {
    pub exe: PathBuf,
}

impl Formatter {
    pub fn format(&self, code: String) -> Result<String> {
        let mut p = Command::new(&self.exe)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        p.stdin
            .as_mut()
            .expect("stdin was captured but was also None")
            .write_all(code.as_bytes())?;

        let p = p.wait_with_output()?;

        if !p.status.success() {
            bail!("formatter exited with non-zero code");
        }

        let formatterd_code = from_utf8(&p.stdout)
            .context("formatter stdout contains invalid UTF8")?
            .trim()
            .to_owned();

        Ok(formatterd_code)
    }
}
