use eyre::{Context, Result};

use crate::banner::Banner;
use crate::bundler::Bundler;
use crate::formatter::Formatter;
use crate::header::Header;
use crate::source::Sources;

pub trait Stage: Sized {
    fn name() -> &'static str;
    fn process(&mut self, code: String) -> Result<String>;
}

impl<S: Stage> Stage for Option<S> {
    fn name() -> &'static str {
        S::name()
    }

    fn process(&mut self, code: String) -> Result<String> {
        match self.as_mut() {
            None => Ok(code),
            Some(stage) => stage.process(code),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub bundler: Bundler,
    pub header: Option<Header>,
    pub banner: Option<Banner>,
    pub formatter: Option<Formatter>,
}

impl Pipeline {
    pub fn process(&mut self, sources: &Sources) -> Result<String> {
        let mut out = self.bundler.bundle(sources);

        out = run_stage(&mut self.banner, out)?;
        out = run_stage(&mut self.header, out)?;
        out = run_stage(&mut self.formatter, out)?;

        Ok(out)
    }
}

fn run_stage<S: Stage>(stage: &mut S, code: String) -> Result<String> {
    stage
        .process(code)
        .with_context(|| format!("stage '{}' failed", S::name()))
}
