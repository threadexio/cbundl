use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, Parser, ValueEnum};
use eyre::{Context, Result};
use serde::Deserialize;

use crate::consts::{
    CRATE_DESCRIPTION, DEFAULT_CONFIG_FILES, DEFAULT_FORMATTER, LONG_VERSION, SHORT_VERSION,
};
use crate::display::display_path;
use crate::quotes::{CustomQuote, QuotePicker};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum BooleanFlag {
    Yes,
    No,
}

impl From<BooleanFlag> for bool {
    fn from(x: BooleanFlag) -> Self {
        x == BooleanFlag::Yes
    }
}

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

    #[arg(
        long,
        help = "Specify an alternate configuration file.",
        value_name = "path",
        default_values = DEFAULT_CONFIG_FILES
    )]
    config: Option<PathBuf>,

    #[arg(
        long = "deterministic",
        help = "Output a deterministic bundle.",
        default_value = "no",
        value_name = "boolean",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "yes",
        hide_default_value = true,
    )]
    deterministic: BooleanFlag,

    #[arg(
        short = 'o',
        long = "output",
        help = "Specify where to write the resulting bundle.",
        value_name = "path",
        default_value = "-"
    )]
    output_file: PathBuf,

    #[arg(
        long,
        help = "Don't output the banner at the top of the bundle.",
        default_value = "no",
        value_name = "boolean",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "yes",
        hide_default_value = true,
    )]
    no_banner: BooleanFlag,

    #[arg(
        long,
        help = "Don't pass the resulting bundle through the formatter.",
        default_value = "no",
        value_name = "boolean",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "yes",
        hide_default_value = true,
    )]
    no_format: BooleanFlag,

    #[arg(
        long,
        help = "Code formatter executable.",
        long_help = "Code formatter. Must format the code from stdin and write it to stdout.",
        value_name = "exe",
        default_value = DEFAULT_FORMATTER
    )]
    formatter: PathBuf,

    #[arg(help = "Path to the entry source file.", value_name = "path")]
    entry: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    bundle: Option<BundleSection>,
    banner: Option<BannerSection>,
    formatter: Option<FormatterSection>,

    #[serde(rename = "quote")]
    quotes: Vec<CustomQuote>,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleSection {
    separators: Option<bool>,
    deterministic: Option<bool>,

    #[serde(rename = "output")]
    output_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
struct BannerSection {
    enable: Option<bool>,
    quote: Option<QuoteSection>,
}

#[derive(Debug, Clone, Deserialize)]
struct QuoteSection {
    enable: Option<bool>,

    #[serde(rename = "pick")]
    picker: Option<QuotePicker>,
}

#[derive(Debug, Clone, Deserialize)]
struct FormatterSection {
    enable: Option<bool>,
    path: Option<PathBuf>,
    args: Option<Vec<String>>,
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

    fn read_many<'a, I>(paths: I) -> Option<Result<Self>>
    where
        I: Iterator<Item = &'a Path>,
    {
        for path in paths {
            match Self::read(path) {
                Some(r) => {
                    return Some(r.with_context(|| {
                        format!("failed to read config `{}`", display_path(path))
                    }))
                }
                None => continue,
            }
        }

        None
    }
}

trait ArgMatchesExt: Sized {
    fn value<'a, T>(&'a self, id: &str) -> Option<&'a T>
    where
        T: Clone + Send + Sync + 'static;

    fn flag(&self, id: &str) -> Option<bool>;
}

impl ArgMatchesExt for ArgMatches {
    fn value<'a, T>(&'a self, id: &str) -> Option<&'a T>
    where
        T: Clone + Send + Sync + 'static,
    {
        match self.value_source(id)? {
            ValueSource::DefaultValue => None,
            _ => Some(self.get_one::<T>(id).unwrap()),
        }
    }

    fn flag(&self, id: &str) -> Option<bool> {
        match self.value_source(id).unwrap() {
            ValueSource::DefaultValue => None,
            _ => Some(self.get_one::<BooleanFlag>(id).unwrap().clone().into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bundle_separators: bool,
    pub deterministic: bool,
    pub output_file: Option<PathBuf>,

    pub no_banner: bool,
    pub enable_quote: bool,
    pub quote_picker: QuotePicker,
    pub custom_quotes: Vec<CustomQuote>,

    pub no_format: bool,
    pub formatter: PathBuf,
    pub formatter_args: Vec<String>,

    pub entry: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        let args = Args::command().get_matches();

        let file = if args.get_flag("no_config") {
            None
        } else if let Some(path) = args.value::<PathBuf>("config") {
            let x = File::read(path)
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
                .with_context(|| format!("failed to read config `{}`", display_path(path)))??;

            Some(x)
        } else {
            let default_config_files = DEFAULT_CONFIG_FILES.iter().copied().map(Path::new);

            match File::read_many(default_config_files) {
                Some(x) => Some(x?),
                None => None,
            }
        };

        let bundle_separators = file
            .as_ref()
            .and_then(|x| x.bundle.as_ref())
            .and_then(|x| x.separators)
            .unwrap_or(true);

        let deterministic = args
            .flag("deterministic")
            .or_else(|| {
                file.as_ref()
                    .and_then(|x| x.bundle.as_ref())
                    .and_then(|x| x.deterministic)
            })
            .unwrap_or(false);

        let output_file = args
            .value::<PathBuf>("output_file")
            .map(|x| path_not_stdio(x).cloned())
            .or_else(|| {
                let x = file
                    .as_ref()
                    .and_then(|x| x.bundle.as_ref())
                    .and_then(|x| x.output_file.as_ref())
                    .cloned();

                Some(x)
            })
            .unwrap_or(None);

        let no_banner = args
            .flag("no_banner")
            .or_else(|| {
                file.as_ref()
                    .and_then(|x| x.banner.as_ref())
                    .and_then(|x| x.enable)
                    .map(|x| !x)
            })
            .unwrap_or(false);

        let enable_quote = file
            .as_ref()
            .and_then(|x| x.banner.as_ref())
            .and_then(|x| x.quote.as_ref())
            .and_then(|x| x.enable)
            .unwrap_or(true);

        let quote_picker = file
            .as_ref()
            .and_then(|x| x.banner.as_ref())
            .and_then(|x| x.quote.as_ref())
            .and_then(|x| x.picker.as_ref())
            .cloned()
            .unwrap_or(QuotePicker::All);

        let custom_quotes = file.as_ref().map(|x| x.quotes.clone()).unwrap_or_default();

        let no_format = args
            .flag("no_format")
            .or_else(|| {
                file.as_ref()
                    .and_then(|x| x.formatter.as_ref())
                    .and_then(|x| x.enable)
                    .map(|x| !x)
            })
            .unwrap_or(false);

        let formatter = args
            .value::<PathBuf>("formatter")
            .cloned()
            .or_else(|| {
                file.as_ref()
                    .and_then(|x| x.formatter.as_ref())
                    .and_then(|x| x.path.as_ref())
                    .cloned()
            })
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FORMATTER));

        let formatter_args = file
            .as_ref()
            .and_then(|x| x.formatter.as_ref())
            .and_then(|x| x.args.clone())
            .unwrap_or_default();

        let entry = args.value::<PathBuf>("entry").unwrap().clone();

        Ok(Self {
            bundle_separators,
            deterministic,
            output_file,

            no_banner,
            enable_quote,
            quote_picker,
            custom_quotes,

            no_format,
            formatter,
            formatter_args,

            entry,
        })
    }
}

fn path_not_stdio(path: &PathBuf) -> Option<&PathBuf> {
    if path_is_stdio(path) {
        None
    } else {
        Some(path)
    }
}

fn path_is_stdio(path: &Path) -> bool {
    path.as_os_str().as_encoded_bytes().eq(b"-")
}
