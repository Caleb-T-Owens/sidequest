#![allow(dead_code)]

use crate::http::primatives::DigitP;
use crate::parser::{InsensitiveTermParser, ParseResult, Parser};

mod primatives;

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

