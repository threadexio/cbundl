use std::fs::File;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use eyre::{Context, Result};

use crate::banner::Banner;
use crate::bundler::Bundler;
use crate::consts::{CRATE_DESCRIPTION, LONG_VERSION, SHORT_VERSION};
use crate::display::display_path;
use crate::formatter::Formatter;
use crate::pipeline::Pipeline;
use crate::quotes::Quotes;
use crate::source::Sources;

#[derive(Debug, Clone, Parser)]
#[command(
    version = SHORT_VERSION,
    long_version = LONG_VERSION,
    about = CRATE_DESCRIPTION,
    long_about = None
)]
pub struct GlobalArgs {
    #[arg(
        long,
        help = "Don't pass the resulting bundle through the formatter.",
        default_value_t = false
    )]
    pub no_format: bool,

    #[arg(
        long,
        help = "Code formatter executable.",
        long_help = "Code formatter. Must format the code from stdin and write it to stdout.",
        default_value = "clang-format",
        value_name = "exe"
    )]
    pub formatter: PathBuf,

    #[arg(
        long = "deterministic",
        help = "Output a deterministic bundle.",
        default_value_t = false
    )]
    pub deterministic: bool,

    #[arg(
        short = 'o',
        long = "output",
        help = "Specify where to write the resulting bundle.",
        default_value = "-",
        value_name = "path"
    )]
    pub output_file: PathBuf,

    #[arg(help = "Path to the entry source file.", value_name = "path")]
    pub entry: PathBuf,
}

pub fn run() -> Result<()> {
    let args = GlobalArgs::parse();
    trace!("args = {args:#?}");

    let sources = Sources::new(args.entry)?;

    let mut pipeline = Pipeline {
        bundler: Bundler {},
        banner: Banner {
            quotes: Quotes {},
            deterministic: args.deterministic,
        },
        formatter: some_if(!args.no_format, || Formatter {
            exe: args.formatter,
        }),
    };

    let bundle = pipeline.process(&sources)?;

    write_bundle(bundle, &args.output_file).with_context(|| {
        format!(
            "failed to write bundle to `{}`",
            display_path(&args.output_file)
        )
    })?;

    Ok(())
}

fn write_bundle(bundle: String, path: &Path) -> Result<()> {
    let mut writer: Box<dyn Write> = if path_is_stdio(path) {
        Box::new(stdout().lock())
    } else {
        Box::new(
            File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?,
        )
    };

    writer.write_all(bundle.as_bytes())?;
    writer.flush()?;
    Ok(())
}

fn path_is_stdio(path: &Path) -> bool {
    path.as_os_str().as_encoded_bytes().eq(b"-")
}

fn some_if<T, F>(cond: bool, f: F) -> Option<T>
where
    F: FnOnce() -> T,
{
    if cond {
        Some(f())
    } else {
        None
    }
}
