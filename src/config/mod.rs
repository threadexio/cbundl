use std::path::PathBuf;

use eyre::Result;

mod args;
mod file;

use self::args::Args;
use self::file::File;

#[derive(Debug, Clone)]
pub struct Config {
    pub no_format: bool,
    pub formatter: PathBuf,
    pub deterministic: bool,
    pub output_file: Option<PathBuf>,
    pub entry: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        let mut config = Self {
            no_format: false,
            formatter: PathBuf::from("clang-format"),
            deterministic: false,
            output_file: None,
            entry: PathBuf::new(),
        };

        trace!("default config: {config:#?}");

        File::merge(&mut config)?;
        trace!("with config file: {config:#?}");

        Args::merge(&mut config)?;
        trace!("with args: {config:#?}");

        Ok(config)
    }
}
