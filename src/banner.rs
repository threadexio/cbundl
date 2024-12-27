use std::fmt::{self, Write};

use chrono::{DateTime, Local};
use const_format::formatcp;

use crate::consts::{CRATE_NAME, CRATE_REPOSITORY, SHORT_VERSION};
use crate::display::format_date;
use crate::pipeline::Stage;
use crate::quotes::Quotes;

#[derive(Debug, Clone)]
pub struct Banner {
    pub quotes: Quotes,
    pub deterministic: bool,
}

impl Banner {
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
}

impl Stage for Banner {
    fn name() -> &'static str {
        "banner"
    }

    fn process(&mut self, code: String) -> eyre::Result<String> {
        const ESTIMATED_BANNER_SIZE: usize = 1024;

        let mut out = String::with_capacity(ESTIMATED_BANNER_SIZE + code.len());

        self.write_banner(&mut out)
            .expect("writing to String should never fail");

        out.push_str(&code);
        Ok(out)
    }
}
