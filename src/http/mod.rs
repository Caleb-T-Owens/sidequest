#![allow(dead_code)]

use crate::http::primatives::DigitP;
use crate::parser::{CharParser, InsensitiveTermParser, ParseResult, Parser};

mod primatives;
mod url;

pub(crate) struct USizeP;
impl Parser for USizeP {
    type Out = usize;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        DigitP
            .bounded_span(1, usize::MAX)
            .map(|r| {
                r.into_iter()
                    .fold(0, |acc, d| acc * 10 + ((d - b'0') as usize))
            })
            .parse(input)
    }
}

#[derive(Debug)]
pub(crate) struct HttpVersion {
    pub(crate) major: usize,
    pub(crate) minor: usize,
}
pub(crate) struct HttpVersionP;
impl Parser for HttpVersionP {
    type Out = HttpVersion;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        InsensitiveTermParser::new("http/")
            .then(|_| USizeP)
            .and(CharParser::new(b'.').then(|_| USizeP))
            .map(|(major, minor)| HttpVersion { major, minor })
            .parse(input)
    }
}
