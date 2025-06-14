use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, Parser, ValueEnum};
use eyre::{bail, Context, Result};
use module::types::Lines;
use module::types::{Ordered, Overridable};
use module::Merge;
use serde::Deserialize;

use crate::consts::{
    CRATE_DESCRIPTION, DEFAULT_CONFIG_FILES, DEFAULT_FORMATTER, LONG_VERSION, SHORT_VERSION,
};
use crate::header::HeaderSource;
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
            _ => {
                let x = *self.get_one::<BooleanFlag>(id).unwrap();
                Some(x.into())
            }
        }
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

impl Args {
    fn get() -> ArgMatches {
        #[cfg(debug_assertions)]
        Self::command().debug_assert();

        Self::command().get_matches()
    }
}

#[derive(Debug, Default, Clone, Deserialize, Merge)]
struct BundleSection {
    separators: Option<Overridable<bool>>,
    deterministic: Option<Overridable<bool>>,
    output: Option<Overridable<PathBuf>>,
}

#[derive(Debug, Default, Clone, Deserialize, Merge)]
#[serde(default)]
struct HeaderSection {
    enable: Option<Overridable<bool>>,
    text: Option<Lines>,
    source: Option<Overridable<PathBuf>>,
}

#[derive(Debug, Default, Clone, Deserialize, Merge)]
#[serde(default)]
struct BannerSection {
    enable: Option<Overridable<bool>>,
    quote: QuoteSection,
}

#[derive(Debug, Default, Clone, Deserialize, Merge)]
#[serde(default)]
struct QuoteSection {
    enable: Option<Overridable<bool>>,
    pick: Option<Overridable<QuotePicker>>,
}

#[derive(Debug, Default, Clone, Deserialize, Merge)]
#[serde(default)]
struct FormatterSection {
    enable: Option<Overridable<bool>>,
    path: Option<Overridable<PathBuf>>,
    args: Option<Ordered<Vec<String>>>,
}

#[derive(Debug, Default, Clone, Deserialize, Merge)]
#[serde(default)]
struct File {
    bundle: BundleSection,
    header: HeaderSection,
    banner: BannerSection,
    formatter: FormatterSection,
    quote: Vec<CustomQuote>,
}

impl File {
    fn read(path: &Path) -> Result<Self> {
        module_util::file::toml(path)
            .map_err(MergeErrorDisplay)
            .with_context(|| format!("failed to read config {}", display!(path)))
    }

    fn read_many<'a, I>(paths: I) -> Option<Result<Self>>
    where
        I: IntoIterator<Item = &'a Path>,
    {
        for path in paths {
            match fs::metadata(path) {
                Ok(_) => return Some(Self::read(path)),
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    debug!("config file {} not found. skipping...", display!(path));
                    continue;
                }
                Err(e) => {
                    return Some(
                        Err(e).with_context(|| format!("failed to read config {}", display!(path))),
                    )
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bundle_separators: bool,
    pub deterministic: bool,
    pub output_file: Option<PathBuf>,

    pub header_source: Option<HeaderSource>,

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
        let args = Args::get();

        let file = if args.get_flag("no_config") {
            None
        } else if let Some(path) = args.value::<PathBuf>("config") {
            let x = File::read(path)?;
            Some(x)
        } else {
            let default_config_files = DEFAULT_CONFIG_FILES.iter().copied().map(Path::new);

            match File::read_many(default_config_files) {
                Some(Ok(x)) => Some(x),
                Some(Err(e)) => return Err(e),
                None => None,
            }
        };

        let bundle_separators = file
            .as_ref()
            .and_then(|x| x.bundle.separators.as_deref().copied())
            .unwrap_or(true);

        let deterministic = args
            .flag("deterministic")
            .or_else(|| {
                file.as_ref()
                    .and_then(|x| x.bundle.deterministic.as_deref().copied())
            })
            .unwrap_or(false);

        let output_file = args
            .value::<PathBuf>("output_file")
            .map(|x| path_not_stdio(x).cloned())
            .or_else(|| {
                let x = file
                    .as_ref()
                    .and_then(|x| x.bundle.output.as_deref().cloned());

                Some(x)
            })
            .unwrap_or(None);

        let header_source = file.as_ref().map(|x| &x.header).map_or(Ok(None), |x| {
            if !x.enable.as_deref().copied().unwrap_or(false) {
                return Ok(None);
            }

            match (x.text.as_deref(), x.source.as_deref()) {
                (Some(x), None) => Ok(Some(HeaderSource::Text(x.clone()))),
                (None, Some(x)) => Ok(Some(HeaderSource::File(x.clone()))),
                (Some(_), Some(_)) => {
                    bail!("both `text` and `source` were specified")
                }
                (None, None) => {
                    bail!("bundle header needs at least one of `text` or `source` to be specified")
                }
            }
        })?;

        let no_banner = args
            .flag("no_banner")
            .or_else(|| {
                file.as_ref()
                    .and_then(|x| x.banner.enable.as_deref().copied())
                    .map(|x| !x)
            })
            .unwrap_or(false);

        let enable_quote = file
            .as_ref()
            .and_then(|x| x.banner.quote.enable.as_deref().copied())
            .unwrap_or(true);

        let quote_picker = file
            .as_ref()
            .and_then(|x| x.banner.quote.pick.as_deref().copied())
            .unwrap_or(QuotePicker::All);

        let custom_quotes = file.as_ref().map(|x| x.quote.clone()).unwrap_or_default();

        let no_format = args
            .flag("no_format")
            .or_else(|| {
                file.as_ref()
                    .and_then(|x| x.formatter.enable.as_deref().copied())
                    .map(|x| !x)
            })
            .unwrap_or(false);

        let formatter = args
            .value::<PathBuf>("formatter")
            .cloned()
            .or_else(|| {
                file.as_ref()
                    .and_then(|x| x.formatter.path.as_deref().cloned())
            })
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FORMATTER));

        let formatter_args = file
            .as_ref()
            .and_then(|x| x.formatter.args.as_deref().cloned())
            .unwrap_or_default();

        let entry = args.value::<PathBuf>("entry").unwrap().clone();

        Ok(Self {
            bundle_separators,
            deterministic,
            output_file,

            header_source,

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

struct MergeErrorDisplay(module::Error);

impl fmt::Debug for MergeErrorDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for MergeErrorDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(module::Error {
            ref kind,
            ref modules,
            ref value,
            ..
        }) = self;

        writeln!(f, "{kind}")?;

        if !value.is_empty() {
            writeln!(f)?;
            writeln!(f, "    …while evaluating {value}")?;
        }

        for x in modules.iter().rev() {
            writeln!(f)?;
            writeln!(f, "    …while evaluating module")?;
            writeln!(f, "      …{}", display!(x; as path))?;
        }

        Ok(())
    }
}

impl std::error::Error for MergeErrorDisplay {}
