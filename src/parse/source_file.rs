use std::path::PathBuf;

use eyre::{bail, Context, ContextCompat, Result};

use super::directive::Directive;
use super::include::{Include, IncludeKind};

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub content: String,
    pub impl_files: Vec<PathBuf>,
    pub includes: Vec<PathBuf>,
}

impl SourceFile {
    pub fn try_parse(s: &str) -> Result<Self> {
        let mut content = String::with_capacity(s.len());
        let mut includes = Vec::new();
        let mut impl_files = Vec::new();

        let mut lines = s.lines().enumerate().peekable();

        while let Some((line_no, line)) = lines.next() {
            match Directive::try_parse(line) {
                Some(Ok(Directive::Bundle)) => {
                    let include = lines
                        .next()
                        .and_then(|(_, next_line)| Include::try_parse(next_line))
                        .context("missing include directive after bundle directive")?
                        .with_context(|| {
                            format!("failed to parse include directive at line {}", line_no + 1)
                        })?;

                    if include.kind != IncludeKind::Local {
                        bail!("cbundl supports only local includes");
                    }

                    includes.push(include.path);
                }
                Some(Ok(Directive::ImplFile(impl_file_path))) => {
                    impl_files.push(impl_file_path);
                }
                Some(Err(e)) => {
                    return Err(e).with_context(|| {
                        format!("failed to parse directive at line {}", line_no + 1)
                    })
                }
                None => {
                    content.push_str(line);
                    content.push('\n');
                }
            }
        }

        Ok(Self {
            content,
            impl_files,
            includes,
        })
    }
}
