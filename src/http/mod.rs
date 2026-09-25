#![allow(dead_code)]

use crate::http::primatives::DigitP;
use crate::http::uri::{AbsPath, AbsPathP, Host, HostP, PortP, QueryP};
use crate::parser::{CharParser, InsensitiveTermParser, ParseResult, Parser};

pub(crate) mod date;
mod primatives;
pub(crate) mod uri;

pub(crate) enum USizeP {
    Unbounded,
    Bounded(usize, usize),
}
impl Parser for USizeP {
    type Out = usize;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        match self {
            Self::Unbounded => DigitP.span(),
            Self::Bounded(min, max) => DigitP.bounded_span(*min, *max),
        }
        .map(|r| {
            r.into_iter()
                .fold(0, |acc, d| acc * 10 + ((d - b'0') as usize))
        })
        .parse(input)
    }
}

pub(crate) enum U32P {
    Unbounded,
    Bounded(usize, usize),
}
impl Parser for U32P {
    type Out = u32;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        match self {
            Self::Unbounded => DigitP.span(),
            Self::Bounded(min, max) => DigitP.bounded_span(*min, *max),
        }
        .map(|r| {
            r.into_iter()
                .fold(0, |acc, d| acc * 10 + ((d - b'0') as u32))
        })
        .parse(input)
    }
}

pub(crate) enum U8P {
    Unbounded,
    Bounded(usize, usize),
}
impl Parser for U8P {
    type Out = u8;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        match self {
            Self::Unbounded => DigitP.span(),
            Self::Bounded(min, max) => DigitP.bounded_span(*min, *max),
        }
        .map(|r| {
            r.into_iter()
                .fold(0, |acc, d| acc * 10 + ((d - b'0') as u8))
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
            .then(|_| USizeP::Bounded(1, usize::MAX))
            .and(CharParser::new(b'.').then(|_| USizeP::Bounded(1, usize::MAX)))
            .map(|(major, minor)| HttpVersion { major, minor })
            .parse(input)
    }
}

#[derive(Debug)]
pub(crate) struct HttpUrl {
    host: Host,
    port: u32,
    path: Option<AbsPath>,
    query: Option<Vec<u8>>,
}
pub(crate) struct HttpUrlP;
impl Parser for HttpUrlP {
    type Out = HttpUrl;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        InsensitiveTermParser::new("http://")
            .then(|_| HostP)
            .and(CharParser::new(b':').then(|_| PortP).optional())
            .and(
                AbsPathP
                    .inspect()
                    .and(CharParser::new(b'?').then(|_| QueryP.optional()))
                    .optional()
                    .map(|x| {
                        x.map(|(path, query)| (Some(path), query))
                            .unwrap_or((None, None))
                    }),
            )
            .map(|((host, port), (path, query))| HttpUrl {
                host,
                port: port.flatten().unwrap_or(80),
                path,
                query,
            })
            .parse(input)
    }
}
