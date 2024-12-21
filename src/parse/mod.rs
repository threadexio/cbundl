use std::iter::Peekable;
use std::str::Chars;

pub mod directive;
pub mod include;
pub mod source_file;

fn consume_whitespace(iter: &mut Peekable<Chars<'_>>, required: bool) -> Option<()> {
    if required {
        if !iter.next()?.is_whitespace() {
            return None;
        }
    }

    while let Some(c) = iter.peek() {
        if c.is_whitespace() {
            let _ = iter.next();
        } else {
            break;
        }
    }

    Some(())
}

fn consume_while<F>(iter: &mut Peekable<Chars<'_>>, mut f: F) -> Option<String>
where
    F: FnMut(char) -> bool,
{
    let mut buf = String::new();

    while let Some(c) = iter.peek() {
        if f(*c) {
            buf.push(*c);
            let _ = iter.next();
        } else {
            break;
        }
    }

    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

fn consume_word(iter: &mut Peekable<Chars<'_>>) -> Option<String> {
    consume_while(iter, |c| c.is_alphabetic())
}

fn consume_until_whitespace(iter: &mut Peekable<Chars<'_>>) -> Option<String> {
    consume_while(iter, |c| !c.is_whitespace())
}
