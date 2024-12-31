use std::fs::File;
use std::io::{stdout, Write};
use std::path::PathBuf;

use eyre::{Context, Result};

use crate::banner::Banner;
use crate::bundler::Bundler;
use crate::config::Config;
use crate::display::display_path;
use crate::formatter::Formatter;
use crate::pipeline::Pipeline;
use crate::quotes::Quotes;
use crate::source::Sources;

pub fn run() -> Result<()> {
    let config = Config::new()?;
    trace!("config = {config:#?}");

    let sources = Sources::new(config.entry)?;

    let mut pipeline = Pipeline {
        bundler: Bundler {},
        banner: Banner {
            quotes: Quotes {},
            deterministic: config.deterministic,
        },
        formatter: (!config.no_format).then_some(Formatter {
            exe: config.formatter,
        }),
    };

    let bundle = pipeline.process(&sources)?;

    write_bundle(bundle, config.output_file.as_ref()).with_context(|| {
        if let Some(path) = config.output_file.as_ref() {
            format!("failed to write bundle to `{}`", display_path(path))
        } else {
            "failed to write bundle to stdout".to_owned()
        }
    })?;

    Ok(())
}

fn write_bundle(bundle: String, path: Option<&PathBuf>) -> Result<()> {
    let mut writer: Box<dyn Write> = if let Some(path) = path {
        Box::new(
            File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?,
        )
    } else {
        Box::new(stdout().lock())
    };

    writer.write_all(bundle.as_bytes())?;
    writer.flush()?;
    Ok(())
}
