use std::io::Write;
use std::process::ExitCode;

use log::{Level, LevelFilter};
use owo_colors::OwoColorize;

#[macro_use]
extern crate log;

mod bundler;
mod cli;
mod consts;
mod display;
mod formatter;
mod parse;
mod quotes;
mod source;
mod minify;

fn main() -> ExitCode {
    setup();
    print_debug_info();

    if let Err(e) = cli::run() {
        error!("{e:#}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn setup() {
    use crate::consts::BUILD_IS_DEBUG;

    color_eyre::install().unwrap();

    env_logger::builder()
        .filter_level(
            const {
                if BUILD_IS_DEBUG {
                    LevelFilter::Trace
                } else {
                    LevelFilter::Info
                }
            },
        )
        .format(|out, record| {
            let level_str = match record.level() {
                Level::Error => "error".bright_red().bold().to_string(),
                Level::Warn => "warn".bright_yellow().bold().to_string(),
                Level::Info => "info".bright_green().bold().to_string(),
                Level::Debug => "debug".white().dimmed().to_string(),
                Level::Trace => "trace".bright_white().bold().to_string(),
            };

            writeln!(out, "{level_str}: {}", record.args())
        })
        .init();
}

fn print_debug_info() {
    use crate::consts::{RUSTC_VERSION, SHORT_VERSION};

    debug!("version: {SHORT_VERSION}");
    debug!("rustc version: {RUSTC_VERSION}");
}
