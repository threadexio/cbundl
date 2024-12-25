use std::fmt::{self, Write};

use chrono::{DateTime, Local};
use const_format::formatcp;
use eyre::{Context, Result};

use crate::consts::{CRATE_NAME, CRATE_REPOSITORY, SHORT_VERSION};
use crate::display::format_date;
use crate::formatter::Formatter;
use crate::quotes::Quotes;
use crate::source::Source;

#[derive(Debug, Clone)]
pub struct Bundler {
    pub formatter: Option<Formatter>,
    pub deterministic: bool,
    pub quotes: Quotes,
}

impl Bundler {
    fn make_bundle<'a, I>(&self, sources: I) -> String
    where
        I: Iterator<Item = &'a Source>,
    {
        let mut out = String::new();

        self.write_banner(&mut out)
            .expect("writing to a String should never fail");

        let mut write_source = |source: &Source| -> fmt::Result {
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
        };

        sources.for_each(|x| write_source(x).expect("writing to a String should never fail"));

        out
    }

    fn write_banner<W: Write + ?Sized>(&self, out: &mut W) -> fmt::Result {
        const MIN_WIDTH: usize = 56;
        const PADDING: usize = 4;

        const ART: &[&str] = &[
            r#"         )                (    ("#,
            r#"      ( /(    (           )\ ) )\"#,
            r#"  (   )\())  ))\   (     (()/(((_)"#,
            r#"  )\ ((_)\  /((_)  )\ )   ((_))_"#,
            r#" ((_)| |(_)(_))(  _(_/(   _| || |"#,
            r#"/ _| | '_ \| || || ' \))/ _` || |"#,
            r#"\__| |_.__/ \_,_||_||_| \__,_||_|"#,
        ];

        let generated_at = if self.deterministic {
            format_date(DateTime::UNIX_EPOCH)
        } else {
            format_date(Local::now())
        };

        let quote = if self.deterministic {
            self.quotes.get(0).expect("we dont have a single quote :'(")
        } else {
            self.quotes.random()
        };

        let line1 = formatcp!("{CRATE_NAME} {SHORT_VERSION}");
        let line2 = formatcp!("{CRATE_REPOSITORY}");
        let line3 = format!("Generated at: {}", generated_at);

        let art_width = ART.iter().map(|x| x.len()).max().unwrap();
        let banner_width = MIN_WIDTH.max(art_width).max(line1.len()).max(line2.len()) + PADDING;

        writeln!(out, "/**")?;
        writeln!(out, " *")?;
        for line in ART {
            writeln!(out, " * {:^1$}", line, banner_width)?;
        }
        writeln!(out, " * ")?;

        writeln!(out, " * {:^1$}", line1, banner_width)?;
        writeln!(out, " * {:^1$}", line2, banner_width)?;
        writeln!(out, " *")?;
        writeln!(out, " * {:^1$}", line3, banner_width)?;
        writeln!(out, " *")?;
        writeln!(out, " *")?;
        quote.lines().try_for_each(|x| writeln!(out, " * {x}"))?;
        writeln!(out, " *   - {}", quote.author())?;
        writeln!(out, " *")?;
        writeln!(out, " */")?;
        writeln!(out)?;

        Ok(())
    }

    pub fn bundle<'a, I>(&self, sources: I) -> Result<String>
    where
        I: Iterator<Item = &'a Source>,
    {
        let mut out = self.make_bundle(sources);

        if let Some(formatter) = self.formatter.as_ref() {
            out = formatter.format(out).context("failed to format bundle")?;
            out.push('\n'); // clang-format removes the final newline for some reason
        }

        Ok(out)
    }
}
