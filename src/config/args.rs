use std::path::{Path, PathBuf};

use clap::parser::ValueSource;
use clap::{CommandFactory, Parser};
use eyre::Result;

use crate::consts::{CRATE_DESCRIPTION, LONG_VERSION, SHORT_VERSION};

use super::Config;

#[derive(Debug, Clone, Parser)]
#[command(
    version = SHORT_VERSION,
    long_version = LONG_VERSION,
    about = CRATE_DESCRIPTION,
    long_about = None
)]
pub struct Args {
    #[arg(long, help = "Don't pass the resulting bundle through the formatter.")]
    no_format: bool,

    #[arg(
        long,
        help = "Code formatter executable.",
        long_help = "Code formatter. Must format the code from stdin and write it to stdout.",
        value_name = "exe",
        default_value = "clang-format"
    )]
    formatter: PathBuf,

    #[arg(long = "deterministic", help = "Output a deterministic bundle.")]
    deterministic: bool,

    #[arg(
        short = 'o',
        long = "output",
        help = "Specify where to write the resulting bundle.",
        value_name = "path",
        default_value = "-"
    )]
    output_file: PathBuf,

    #[arg(help = "Path to the entry source file.", value_name = "path")]
    entry: PathBuf,
}

impl Args {
    pub fn merge(config: &mut Config) -> Result<()> {
        let matches = Args::command().get_matches();

        // SAFETY: The following `unwrap`s are safe because the these arguments have default values.

        if matches.contains_id("no_format") {
            config.no_format = matches.get_flag("no_format");
        }

        if matches.value_source("formatter").unwrap() != ValueSource::DefaultValue {
            config.formatter = matches.get_one::<PathBuf>("formatter").unwrap().to_owned();
        }

        if matches.contains_id("deterministic") {
            config.deterministic = matches.get_flag("deterministic");
        }

        if matches.value_source("output_file").unwrap() != ValueSource::DefaultValue {
            let output_file = matches.get_one::<PathBuf>("output_file").unwrap();
            config.output_file = path_is_stdio(output_file).then(|| output_file.to_owned());
        }

        if let Some(x) = matches.get_one::<PathBuf>("entry").cloned() {
            config.entry = x;
        }

        Ok(())
    }
}

fn path_is_stdio(path: &Path) -> bool {
    path.as_os_str().as_encoded_bytes().eq(b"-")
}
