use std::path::PathBuf;

use thiserror::Error;

use super::{consume_until_whitespace, consume_whitespace, consume_word};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Directive {
    Bundle,
    ImplFile(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ParseDirectiveError {
    #[error("missing directive keyword")]
    MissingKeyword,
    #[error("invalid directive keyword (expected: 'bundle', 'impl')")]
    InvalidKeyword,
    #[error("invalid directive syntax: {0}")]
    InvalidSyntax(&'static str),
}

impl Directive {
    pub fn try_parse(s: &str) -> Option<Result<Self, ParseDirectiveError>> {
        let s = s.trim_start();
        let s = s.strip_prefix("// cbundl:")?;

        Some(parse_body(s))
    }
}

fn parse_body(s: &str) -> Result<Directive, ParseDirectiveError> {
    let mut iter = s.chars().peekable();

    consume_whitespace(&mut iter, false);
    let keyword = consume_word(&mut iter).ok_or(ParseDirectiveError::MissingKeyword)?;

    match keyword.as_str() {
        "bundle" => Ok(Directive::Bundle),
        "impl" => {
            consume_whitespace(&mut iter, false);
            let x = iter.next().ok_or(ParseDirectiveError::InvalidSyntax(
                "missing '=' for impl directive",
            ))?;
            if x != '=' {
                return Err(ParseDirectiveError::InvalidSyntax(
                    "expected '=' after 'impl' keyword",
                ));
            }

            consume_whitespace(&mut iter, false);
            let path = consume_until_whitespace(&mut iter).ok_or(
                ParseDirectiveError::InvalidSyntax("missing implementation file path"),
            )?;

            Ok(Directive::ImplFile(PathBuf::from(path)))
        }
        _ => Err(ParseDirectiveError::InvalidKeyword),
    }
}
