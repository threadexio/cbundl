use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use eyre::{Context, Result};
use serde::Deserialize;

use crate::display::display_path;

use super::Config;

const DEFAULT_FILES: &[&str] = &[".cbundl.toml", "cbundl.toml"];

#[derive(Debug, Clone, Deserialize)]
pub struct File {
    #[serde(rename = "no-format")]
    no_format: Option<bool>,

    formatter: Option<PathBuf>,

    deterministic: Option<bool>,

    #[serde(rename = "output")]
    output_file: Option<PathBuf>,
}

impl File {
    fn read() -> Option<Result<Self>> {
        for f in DEFAULT_FILES {
            let x = match read_file(Path::new(f)) {
                Some(Ok(x)) => x,
                Some(Err(e)) => return Some(Err(e)),
                None => continue,
            };

            match toml::from_str(&x) {
                Ok(x) => return Some(Ok(x)),
                Err(e) => return Some(Err(e.into())),
            }
        }

        None
    }

    pub fn merge(config: &mut Config) -> Result<()> {
        let file = match Self::read() {
            Some(x) => x?,
            None => return Ok(()),
        };

        if let Some(x) = file.no_format {
            config.no_format = x;
        }

        if let Some(x) = file.formatter {
            config.formatter = x;
        }

        if let Some(x) = file.deterministic {
            config.deterministic = x;
        }

        if let Some(x) = file.output_file {
            config.output_file = Some(x);
        }

        Ok(())
    }
}

fn read_file(path: &Path) -> Option<Result<String>> {
    match fs::read_to_string(path) {
        Ok(x) => Some(Ok(x)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        r @ Err(_) => Some(r.with_context(|| format!("failed to read `{}`", display_path(path)))),
    }
}
