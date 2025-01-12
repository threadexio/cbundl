use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use eyre::{Context, Result};

use crate::{display::display_path, pipeline::Stage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderSource {
    Text(String),
    File(PathBuf),
}

impl HeaderSource {
    fn get(&self) -> Result<Cow<'_, str>> {
        match self {
            Self::Text(x) => Ok(Cow::Borrowed(x)),
            Self::File(ref x) => fs::read_to_string(x)
                .with_context(|| format!("failed to read banner source `{}`", display_path(x)))
                .map(Cow::Owned),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub source: HeaderSource,
}

impl Stage for Header {
    fn name() -> &'static str {
        "header"
    }

    fn process(&mut self, code: String) -> Result<String> {
        let banner = self.source.get()?;
        let banner = banner.trim_end();

        let mut out = String::with_capacity(banner.len() + 2 + code.len());

        out.push_str(banner);
        out.push('\n');
        out.push('\n');
        out.push_str(&code);

        Ok(out)
    }
}
