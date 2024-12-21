#![allow(non_camel_case_types)]

use std::fmt::{self, Display};
use std::path::Path;

use chrono::{DateTime, Local};
use owo_colors::OwoColorize;

pub struct display_path<T>(pub T);

impl<T: AsRef<Path>> Display for display_path<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().display().bright_cyan().fmt(f)
    }
}

pub fn format_date(date: DateTime<Local>) -> String {
    date.format("%a %d %b %Y %H:%M:%S (UTC%:z)").to_string()
}
