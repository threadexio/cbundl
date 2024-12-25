use std::marker::PhantomData;
use std::slice::Iter as SliceIter;

use rand::seq::SliceRandom;

#[derive(Clone)]
pub struct Quote<'a> {
    _marker: PhantomData<&'a ()>,
    quote: &'static BuiltInQuote,
}

impl Quote<'_> {
    fn new(quote: &'static BuiltInQuote) -> Self {
        Self {
            _marker: PhantomData,
            quote,
        }
    }

    pub fn lines(&self) -> QuoteLinesIter<'_> {
        QuoteLinesIter {
            inner: self.quote.text.iter(),
        }
    }

    pub fn author(&self) -> &str {
        self.quote.author
    }
}

#[derive(Clone)]
pub struct QuoteLinesIter<'a> {
    inner: SliceIter<'a, &'a str>,
}

impl<'a> Iterator for QuoteLinesIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Debug, Clone)]
pub struct Quotes {}

impl Quotes {
    pub fn get(&self, i: usize) -> Option<Quote<'_>> {
        BUILT_IN_QUOTES.get(i).map(Quote::new)
    }

    pub fn random(&self) -> Quote<'_> {
        Quote::new(choose_random_quote())
    }
}

include!(concat!(env!("OUT_DIR"), "/quotes.rs"));

fn choose_random_quote() -> &'static BuiltInQuote {
    let mut rng = rand::thread_rng();

    BUILT_IN_QUOTES
        .choose(&mut rng)
        .expect("we have no quotes :(")
}
