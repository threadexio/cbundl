use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use eyre::{bail, Context, Result};

use crate::display::display_path;
use crate::pipeline::Stage;

#[derive(Debug, Clone)]
pub struct Formatter {
    pub exe: PathBuf,
    pub args: Vec<String>,
}

impl Stage for Formatter {
    fn name() -> &'static str {
        "format"
    }

    fn process(&mut self, code: String) -> Result<String> {
        let mut p = Command::new(&self.exe)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("failed to run formatter `{}`", display_path(&self.exe)))?;

        p.stdin
            .as_mut()
            .expect("stdin was captured but was also None")
            .write_all(code.as_bytes())
            .context("failed to write bundle to formatter's stdin")?;

        let p = p.wait_with_output()?;

        if !p.status.success() {
            bail!("formatter exited with non-zero code");
        }

        let formatted_code =
            String::from_utf8(p.stdout).context("formatter stdout contains invalid UTF8")?;

        Ok(formatted_code)
    }
}
