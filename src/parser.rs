#![allow(dead_code)]

use crate::either::Either;
use std::{fmt::Debug, ops::RangeBounds};

#[derive(PartialEq, Eq)]
pub(crate) enum ParseResult<'i, T> {
    Found { subject: T, rest: &'i [u8] },
    Missed { rest: &'i [u8] },
}

impl<'i, T> ParseResult<'i, T> {
    pub(crate) fn map<U, F>(self, op: F) -> ParseResult<'i, U>
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

    pub(crate) fn then<U, F>(self, op: F) -> ParseResult<'i, U>
    where
        F: FnOnce(T, &'i [u8]) -> ParseResult<'i, U>,
    {
        match self {
            Self::Found { subject, rest } => op(subject, rest),
            Self::Missed { rest } => ParseResult::Missed { rest },
        }
    }

    pub(crate) fn or_else<U, F>(self, op: F) -> ParseResult<'i, Either<T, U>>
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

    pub(crate) fn optional(self) -> ParseResult<'i, Option<T>> {
        match self {
            Self::Found { subject, rest } => ParseResult::Found {
                subject: Some(subject),
                rest,
            },
            Self::Missed { rest } => ParseResult::Found {
                subject: None,
                rest,
            },
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

    fn then<P: Parser, F>(self: Self, op: F) -> ThenParser<Self, F>
    where
        F: FnOnce(Self::Out) -> P + Clone + Copy,
        Self: Sized,
    {
        ThenParser { parser: self, op }
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

    fn optional(self: Self) -> OptionalParser<Self>
    where
        Self: Sized,
    {
        OptionalParser(self)
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

    fn select<F>(self: Self, op: F) -> SelectParser<Self, F>
    where
        F: FnOnce(&Self::Out) -> bool + Clone + Copy,
        Self: Sized,
    {
        SelectParser { parser: self, op }
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

pub(crate) struct SelectParser<P, F> {
    parser: P,
    op: F,
}

impl<P: Parser, F: FnOnce(&P::Out) -> bool + Clone + Copy> Parser for SelectParser<P, F> {
    type Out = P::Out;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        match self.parser.parse(input) {
            ParseResult::Found { subject, rest } => {
                if (self.op)(&subject) {
                    ParseResult::Found { subject, rest }
                } else {
                    ParseResult::Missed { rest: input }
                }
            }
            ParseResult::Missed { rest } => ParseResult::Missed { rest },
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
            .then(|a, a_rest| match self.b.parse(a_rest) {
                ParseResult::Found { subject, rest } => ParseResult::Found {
                    subject: (a, subject),
                    rest,
                },
                ParseResult::Missed { .. } => ParseResult::Missed { rest: input },
            })
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

pub(crate) struct AnyParser<A> {
    ps: A,
}

impl<A: Parser, const N: usize> AnyParser<[A; N]> {
    pub(crate) fn new(ps: [A; N]) -> Self {
        AnyParser { ps }
    }
}

impl<A: Parser, const N: usize> Parser for AnyParser<[A; N]> {
    type Out = A::Out;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        self.ps
            .iter()
            .find_map(|p| {
                let result = p.parse(input);
                if matches!(result, ParseResult::Found { .. }) {
                    Some(result)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| ParseResult::Missed { rest: input })
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

pub(crate) struct ThenParser<P, F> {
    parser: P,
    op: F,
}

impl<P: Parser, U: Parser, F: FnOnce(P::Out) -> U + Clone + Copy> Parser for ThenParser<P, F> {
    type Out = U::Out;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        self.parser
            .parse(input)
            .then(|a, a_rest| (self.op)(a).parse(a_rest))
    }
}

pub(crate) struct OptionalParser<P>(P);

impl<P: Parser> Parser for OptionalParser<P> {
    type Out = Option<P::Out>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        self.0.parse(input).optional()
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

pub(crate) struct InsensitiveTermParser {
    term: Vec<u8>,
}

impl InsensitiveTermParser {
    pub(crate) fn new(term: impl Into<Vec<u8>>) -> Self {
        Self {
            term: term.into().to_ascii_lowercase(),
        }
    }
}

impl Parser for InsensitiveTermParser {
    type Out = Vec<u8>;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Vec<u8>> {
        if input.get(..self.term.len()).is_some_and(|input| {
            input.iter().enumerate().all(|(i, b)| {
                self.term
                    .get(i)
                    .is_some_and(|t| *t == b.to_ascii_lowercase())
            })
        }) {
            ParseResult::Found {
                subject: self.term.clone(),
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

pub(crate) struct RangeParser<R> {
    range: R,
}

impl<R: RangeBounds<u8>> RangeParser<R> {
    pub(crate) fn new(range: R) -> Self {
        Self { range }
    }
}

impl<R: RangeBounds<u8>> Parser for RangeParser<R> {
    type Out = u8;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        if let Some(a) = input.first()
            && self.range.contains(a)
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

pub(crate) struct CharParser {
    char: u8,
}

impl CharParser {
    pub(crate) fn new(char: u8) -> Self {
        Self { char }
    }
}

impl Parser for CharParser {
    type Out = u8;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        if let Some(a) = input.first()
            && *a == self.char
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
