use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::parser::ValueSource;
use clap::{CommandFactory, Parser};
use eyre::{Context, Result};
use serde::Deserialize;

use crate::consts::{
    CRATE_DESCRIPTION, DEFAULT_CONFIG_FILES, DEFAULT_FORMATTER, LONG_VERSION, SHORT_VERSION,
};
use crate::display::display_path;

#[derive(Debug, Clone, Parser)]
#[command(
    version = SHORT_VERSION,
    long_version = LONG_VERSION,
    about = CRATE_DESCRIPTION,
    long_about = None
)]
struct Args {
    #[arg(
        long,
        help = "Don't load any configuration file. (Overrides `--config`)"
    )]
    no_config: bool,

    #[arg(long, help = "Specify an alternate configuration file")]
    config: Option<PathBuf>,

    #[arg(long, help = "Don't pass the resulting bundle through the formatter.")]
    no_format: bool,

    #[arg(
        long,
        help = "Code formatter executable.",
        long_help = "Code formatter. Must format the code from stdin and write it to stdout.",
        value_name = "exe",
        default_value = DEFAULT_FORMATTER
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

#[derive(Debug, Clone, Deserialize)]
struct File {
    bundle: Option<BundleSection>,
    formatter: Option<FormatterSection>,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleSection {
    deterministic: Option<bool>,

    #[serde(rename = "output")]
    output_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
struct FormatterSection {
    enable: Option<bool>,
    path: Option<PathBuf>,
}

impl File {
    fn read(path: &Path) -> Option<Result<Self>> {
        let x = match fs::read_to_string(path) {
            Ok(x) => x,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return None,
            Err(e) => return Some(Err(e).context("failed to read file")),
        };

        let x = toml::from_str(&x).context("failed to parse file");
        Some(x)
    }

    fn read_many<'a, I>(paths: I) -> Option<Self>
    where
        I: Iterator<Item = &'a Path>,
    {
        for path in paths {
            match Self::read(path) {
                Some(r) => {
                    match r
                        .with_context(|| format!("failed to read config `{}`", display_path(path)))
                    {
                        Ok(x) => return Some(x),
                        Err(e) => {
                            warn!("{e:#}");
                            continue;
                        }
                    }
                }
                None => continue,
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub no_format: bool,
    pub formatter: PathBuf,
    pub deterministic: bool,
    pub output_file: Option<PathBuf>,
    pub entry: PathBuf,
}

// I think that discrete `if-else` blocks show the different places a value can come from
// better than using the functional-style combinators.
#[allow(
    clippy::manual_unwrap_or,
    clippy::manual_unwrap_or_default,
    clippy::manual_map
)]
impl Config {
    pub fn new() -> Result<Self> {
        let args = Args::command().get_matches();

        let file = if args.get_flag("no_config") {
            None
        } else if let Some(config) = args.get_one::<&PathBuf>("config") {
            let x = File::read(config)
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
                .with_context(|| format!("failed to read config `{}`", display_path(config)))??;

            Some(x)
        } else {
            File::read_many(DEFAULT_CONFIG_FILES.iter().copied().map(Path::new))
        };

        let no_format = if let ValueSource::CommandLine = args.value_source("no_format").unwrap() {
            args.get_flag("no_format")
        } else if let Some(x) = file
            .as_ref()
            .and_then(|x| x.formatter.as_ref())
            .and_then(|x| x.enable)
        {
            !x
        } else {
            false
        };

        let formatter = if let ValueSource::CommandLine = args.value_source("formatter").unwrap() {
            args.get_one::<PathBuf>("formatter").unwrap().clone()
        } else if let Some(x) = file
            .as_ref()
            .and_then(|x| x.formatter.as_ref())
            .and_then(|x| x.path.as_ref())
        {
            x.clone()
        } else {
            PathBuf::from(DEFAULT_FORMATTER)
        };

        let deterministic =
            if let ValueSource::CommandLine = args.value_source("deterministic").unwrap() {
                args.get_flag("deterministic")
            } else if let Some(x) = file
                .as_ref()
                .and_then(|x| x.bundle.as_ref())
                .and_then(|x| x.deterministic)
            {
                x
            } else {
                false
            };

        let output_file =
            if let ValueSource::CommandLine = args.value_source("output_file").unwrap() {
                let x = args.get_one::<PathBuf>("output_file").unwrap();
                (!path_is_stdio(x)).then(|| x.clone())
            } else if let Some(x) = file
                .as_ref()
                .and_then(|x| x.bundle.as_ref())
                .and_then(|x| x.output_file.as_ref())
            {
                Some(x.to_owned())
            } else {
                None
            };

        let entry = args.get_one::<PathBuf>("entry").unwrap().clone();

        Ok(Self {
            no_format,
            formatter,
            deterministic,
            output_file,
            entry,
        })
    }
}

fn path_is_stdio(path: &Path) -> bool {
    path.as_os_str().as_encoded_bytes().eq(b"-")
}
