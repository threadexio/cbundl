use std::fmt::Write;

use eyre::Result;

use crate::source::Sources;

#[derive(Debug, Clone)]
pub struct Bundler {}

impl Bundler {
    pub fn bundle(&self, sources: &Sources) -> String {
        let mut out = String::new();

        sources
            .dependency_order()
            .try_for_each(|source| -> Result<()> {
                let file_name = source
                    .path
                    .file_name()
                    .expect("source file paths should always have a last component");

                let header = format!("bundled from \"{}\"", file_name.to_string_lossy());

                writeln!(out, "/**")?;
                writeln!(out, " * {}", header)?;
                writeln!(out, " */")?;
                writeln!(out)?;

                out.write_str(&source.content)?;
                if !source.content.ends_with("\n\n") {
                    writeln!(out)?;
                }

                Ok(())
            })
            .expect("writing to String should never fail");

        out
    }
}
