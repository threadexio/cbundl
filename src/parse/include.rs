use std::path::PathBuf;

use thiserror::Error;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IncludeKind {
    Local,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Include {
    pub kind: IncludeKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ParseIncludeError {
    #[error("expected include path separator")]
    MissingSeparator,
    #[error("invalid include path separator")]
    InvalidSeparator,
    #[error("missing include path")]
    MissingPath,
}

impl Include {
    pub fn try_parse(s: &str) -> Option<Result<Self, ParseIncludeError>> {
        let s = s.trim_start();
        let s = s.strip_prefix("#include")?;
        Some(parse_body(s))
    }
}

fn parse_body(s: &str) -> Result<Include, ParseIncludeError> {
    let mut iter = s.chars().peekable();

    consume_whitespace(&mut iter, false);
    let sep = iter.next().ok_or(ParseIncludeError::MissingSeparator)?;

    match sep {
        '"' => {
            let path =
                consume_while(&mut iter, |c| c != '"').ok_or(ParseIncludeError::MissingPath)?;

            let sep = iter.next().ok_or(ParseIncludeError::MissingSeparator)?;
            if sep != '"' {
                return Err(ParseIncludeError::InvalidSeparator);
            }

            Ok(Include {
                kind: IncludeKind::Local,
                path: PathBuf::from(path),
            })
        }
        '<' => {
            let path =
                consume_while(&mut iter, |c| c != '>').ok_or(ParseIncludeError::MissingPath)?;

            let sep = iter.next().ok_or(ParseIncludeError::MissingSeparator)?;
            if sep != '>' {
                return Err(ParseIncludeError::InvalidSeparator);
            }

            Ok(Include {
                kind: IncludeKind::System,
                path: PathBuf::from(path),
            })
        }
        _ => Err(ParseIncludeError::InvalidSeparator),
    }
}
