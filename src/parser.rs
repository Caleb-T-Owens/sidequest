#![allow(dead_code)]

use crate::either::Either;
use std::fmt::Debug;

#[derive(PartialEq, Eq)]
pub(crate) enum ParseResult<'i, T> {
    Found { subject: T, rest: &'i [u8] },
    Missed { rest: &'i [u8] },
}

impl<'i, T> ParseResult<'i, T> {
    fn map<U, F>(self, op: F) -> ParseResult<'i, U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Found { subject, rest } => ParseResult::Found {
                subject: op(subject),
                rest,
            },
            Self::Missed { rest } => ParseResult::Missed { rest },
        }
    }

    fn then<U, F>(self, op: F) -> ParseResult<'i, U>
    where
        F: FnOnce(T, &'i [u8]) -> ParseResult<'i, U>,
    {
        match self {
            Self::Found { subject, rest } => op(subject, rest),
            Self::Missed { rest } => ParseResult::Missed { rest },
        }
    }

    fn or_else<U, F>(self, op: F) -> ParseResult<'i, Either<T, U>>
    where
        F: FnOnce() -> ParseResult<'i, U>,
    {
        match self {
            Self::Found { subject, rest } => ParseResult::Found {
                subject: Either::Left(subject),
                rest,
            },
            Self::Missed { .. } => op().map(Either::Right),
        }
    }
}

impl<'i, T: Debug> Debug for ParseResult<'i, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Found { subject, rest } => f
                .debug_struct("ParseResult::Found")
                .field("subject", subject)
                .field(
                    "rest",
                    &std::str::from_utf8(rest).unwrap_or("non-utf8 bytes"),
                )
                .finish(),
            Self::Missed { rest } => f
                .debug_struct("ParseResult::Missed")
                .field(
                    "rest",
                    &std::str::from_utf8(rest).unwrap_or("non-utf8 bytes"),
                )
                .finish(),
        }
    }
}

pub(crate) trait Parser {
    type Out;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out>;

    fn inspect(self: Self) -> InspectParser<Self>
    where
        Self: Sized,
        Self::Out: Debug,
    {
        InspectParser(self)
    }

    fn map<U, F>(self: Self, op: F) -> MapParser<Self, F>
    where
        F: FnOnce(Self::Out) -> U + Clone + Copy,
        Self: Sized,
    {
        MapParser { parser: self, op }
    }

    fn or<P: Parser>(self: Self, b: P) -> OrParser<Self, P>
    where
        Self: Sized,
    {
        OrParser { a: self, b }
    }

    fn and<P: Parser>(self: Self, b: P) -> AndParser<Self, P>
    where
        Self: Sized,
    {
        AndParser { a: self, b }
    }

    fn span(self: Self) -> SpanParser<Self>
    where
        Self: Sized,
    {
        self.bounded_span(0, usize::MAX)
    }

    fn bounded_span(self: Self, min: usize, max: usize) -> SpanParser<Self>
    where
        Self: Sized,
    {
        SpanParser {
            parser: self,
            min,
            max,
        }
    }
}

impl<P> Parser for Box<P>
where
    P: Parser + ?Sized,
{
    type Out = P::Out;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        (**self).parse(input)
    }
}

pub(crate) struct SpanParser<P> {
    parser: P,
    min: usize,
    max: usize,
}

impl<P: Parser> Parser for SpanParser<P> {
    type Out = Vec<P::Out>;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        let mut result = vec![];
        let mut rest = input;

        loop {
            if result.len() >= self.max {
                break;
            }
            match self.parser.parse(rest) {
                ParseResult::Found {
                    subject,
                    rest: new_rest,
                } => {
                    result.push(subject);
                    rest = new_rest;
                }
                ParseResult::Missed { .. } => break,
            }
        }

        if result.len() >= self.min {
            ParseResult::Found {
                subject: result,
                rest,
            }
        } else {
            ParseResult::Missed { rest: input }
        }
    }
}

pub(crate) struct AndParser<A, B> {
    a: A,
    b: B,
}

impl<A: Parser, B: Parser> Parser for AndParser<A, B> {
    type Out = (A::Out, B::Out);
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        self.a
            .parse(input)
            .then(|a, a_rest| self.b.parse(a_rest).map(|b| (a, b)))
    }
}

pub(crate) struct OrParser<A, B> {
    a: A,
    b: B,
}

impl<A: Parser, B: Parser> Parser for OrParser<A, B> {
    type Out = Either<A::Out, B::Out>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        self.a.parse(input).or_else(|| self.b.parse(input))
    }
}

pub(crate) struct MapParser<P, F> {
    parser: P,
    op: F,
}

impl<P: Parser, U, F: FnOnce(P::Out) -> U + Clone + Copy> Parser for MapParser<P, F> {
    type Out = U;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, U> {
        self.parser.parse(input).map(self.op)
    }
}

pub(crate) struct InspectParser<P>(P);

impl<P: Parser> Parser for InspectParser<P>
where
    P::Out: Debug,
{
    type Out = P::Out;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        dbg!(self.0.parse(input))
    }
}

pub(crate) struct TermParser<'t> {
    term: &'t [u8],
}

impl<'t> TermParser<'t> {
    pub(crate) fn new(term: &'t [u8]) -> Self {
        Self { term }
    }
}

impl<'t> Parser for TermParser<'t> {
    type Out = &'t [u8];

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, &'t [u8]> {
        if input.starts_with(self.term) {
            ParseResult::Found {
                subject: self.term,
                rest: &input[self.term.len()..],
            }
        } else {
            ParseResult::Missed { rest: input }
        }
    }
}

pub(crate) struct MatchParser<F: Fn(u8) -> bool> {
    matcher: F,
}

impl<F: Fn(u8) -> bool> MatchParser<F> {
    pub(crate) fn new(matcher: F) -> Self {
        Self { matcher }
    }
}

impl<F: Fn(u8) -> bool> Parser for MatchParser<F> {
    type Out = u8;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        if let Some(a) = input.first()
            && (self.matcher)(*a)
        {
            ParseResult::Found {
                subject: *a,
                rest: &input[1..],
            }
        } else {
            ParseResult::Missed { rest: input }
        }
    }
}
