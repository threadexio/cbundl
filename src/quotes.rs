use std::marker::PhantomData;
use std::vec::IntoIter as VecIter;

use rand::seq::IteratorRandom;
use serde::Deserialize;

#[derive(Clone)]
enum QuoteInner<'a> {
    Builtin(&'static BuiltInQuote),
    Custom(&'a CustomQuote),
}

#[derive(Clone)]
pub struct Quote<'a> {
    inner: QuoteInner<'a>,
}

impl Quote<'_> {
    pub fn lines(&self) -> QuoteLinesIter<'_> {
        let inner = match self.inner {
            // Don't listen to clippy, this is very much necessary so we can obtain `VecIter`.
            #[allow(clippy::unnecessary_to_owned)]
            QuoteInner::Builtin(quote) => quote.text.to_vec().into_iter(),

            QuoteInner::Custom(quote) => quote
                .text
                .lines()
                .map(|x| x.trim_end())
                .collect::<Vec<&str>>()
                .into_iter(),
        };

        QuoteLinesIter {
            _marker: PhantomData,
            inner,
        }
    }

    pub fn author(&self) -> &str {
        match self.inner {
            QuoteInner::Builtin(x) => x.author,
            QuoteInner::Custom(x) => &x.author,
        }
    }
}

#[derive(Clone)]
pub struct QuoteLinesIter<'a> {
    _marker: PhantomData<&'a ()>,
    inner: VecIter<&'a str>,
}

impl<'a> Iterator for QuoteLinesIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotePicker {
    All,
    Custom,
    Builtin,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomQuote {
    pub text: String,
    pub author: String,
}

#[derive(Debug, Clone)]
pub struct Quotes {
    pub deterministic: bool,
    pub picker: QuotePicker,
    pub custom_quotes: Vec<CustomQuote>,
}

impl Quotes {
    fn get_builtin_quote(&self) -> Quote<'_> {
        let inner = if self.deterministic {
            &BUILT_IN_QUOTES[0]
        } else {
            choose_random(BUILT_IN_QUOTES.iter()).unwrap()
        };

        Quote {
            inner: QuoteInner::Builtin(inner),
        }
    }

    fn get_custom_quote(&self) -> Option<Quote<'_>> {
        let inner = if self.deterministic {
            self.custom_quotes.first()
        } else {
            choose_random(self.custom_quotes.iter())
        }?;

        Some(Quote {
            inner: QuoteInner::Custom(inner),
        })
    }

    fn get_any_quote(&self) -> Quote<'_> {
        if self.deterministic {
            Quote {
                inner: QuoteInner::Builtin(&BUILT_IN_QUOTES[0]),
            }
        } else {
            let builtin = BUILT_IN_QUOTES.iter().map(|x| Quote {
                inner: QuoteInner::Builtin(x),
            });

            let custom = self.custom_quotes.iter().map(|x| Quote {
                inner: QuoteInner::Custom(x),
            });

            let quotes = builtin.chain(custom);

            // SAFETY: `builtin` has at least one element, so `quotes` must also have
            //         at least one element.
            choose_random(quotes).unwrap().clone()
        }
    }

    pub fn random(&self) -> Quote<'_> {
        match self.picker {
            QuotePicker::All => self.get_any_quote(),
            QuotePicker::Builtin => self.get_builtin_quote(),
            QuotePicker::Custom => self
                .get_custom_quote()
                .unwrap_or_else(|| self.get_builtin_quote()),
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/quotes.rs"));

fn choose_random<T, I>(iter: I) -> Option<T>
where
    I: Iterator<Item = T>,
{
    iter.choose(&mut rand::thread_rng())
}
