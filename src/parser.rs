use crate::either::Either;
use std::fmt::Debug;

#[derive(PartialEq, Eq)]
pub(crate) enum ParseResult<'i, T> {
    Found { subject: T, rest: &'i [u8] },
    Missed { rest: &'i [u8] },
}

impl<'i, T> ParseResult<'i, T> {
    #[allow(unused)]
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

    #[allow(unused)]
    fn then<U, F>(self, op: F) -> ParseResult<'i, U>
    where
        F: FnOnce(T, &'i [u8]) -> ParseResult<'i, U>,
    {
        match self {
            Self::Found { subject, rest } => op(subject, rest),
            Self::Missed { rest } => ParseResult::Missed { rest },
        }
    }

    #[allow(unused)]
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

pub(crate) trait Parser<'i, Out> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Out>;

    #[allow(unused)]
    fn inspect(self: Self) -> InspectParser<'i, Out>
    where
        Self: Sized + 'static,
    {
        InspectParser(Box::new(self))
    }

    #[allow(unused)]
    fn map<U, F>(self: Self, op: F) -> MapParser<'i, Out, U, F>
    where
        F: FnOnce(Out) -> U + Clone + Copy,
        Self: Sized + 'static,
    {
        MapParser {
            parser: Box::new(self),
            op,
        }
    }

    #[allow(unused)]
    fn or<OutB, B>(self: Self, b: B) -> OrParser<'i, Out, OutB>
    where
        B: Parser<'i, OutB> + 'static,
        Self: Sized + 'static,
    {
        OrParser {
            a: Box::new(self),
            b: Box::new(b),
        }
    }

    #[allow(unused)]
    fn and<OutB, B>(self: Self, b: B) -> AndParser<'i, Out, OutB>
    where
        B: Parser<'i, OutB> + 'static,
        Self: Sized + 'static,
    {
        AndParser {
            a: Box::new(self),
            b: Box::new(b),
        }
    }
}

#[allow(unused)]
pub(crate) struct AndParser<'i, OutA, OutB> {
    a: Box<dyn Parser<'i, OutA>>,
    b: Box<dyn Parser<'i, OutB>>,
}

impl<'i, OutA, OutB> Parser<'i, (OutA, OutB)> for AndParser<'i, OutA, OutB> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, (OutA, OutB)> {
        self.a
            .parse(input)
            .then(|a, a_rest| self.b.parse(a_rest).map(|b| (a, b)))
    }
}

#[allow(unused)]
pub(crate) struct OrParser<'i, OutA, OutB> {
    a: Box<dyn Parser<'i, OutA>>,
    b: Box<dyn Parser<'i, OutB>>,
}

impl<'i, OutA, OutB> Parser<'i, Either<OutA, OutB>> for OrParser<'i, OutA, OutB> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Either<OutA, OutB>> {
        self.a.parse(input).or_else(|| self.b.parse(input))
    }
}

#[allow(unused)]
pub(crate) struct MapParser<'i, Out, U, F: FnOnce(Out) -> U> {
    parser: Box<dyn Parser<'i, Out>>,
    op: F,
}

impl<'i, Out, U, F: FnOnce(Out) -> U + Clone + Copy> Parser<'i, U> for MapParser<'i, Out, U, F> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, U> {
        self.parser.parse(input).map(self.op)
    }
}

#[allow(unused)]
pub(crate) struct InspectParser<'i, Out>(Box<dyn Parser<'i, Out>>);

impl<'i, Out: Debug> Parser<'i, Out> for InspectParser<'i, Out> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Out> {
        dbg!(self.0.parse(input))
    }
}

#[allow(unused)]
pub(crate) struct TermParser<'t> {
    term: &'t [u8],
}

impl<'t> TermParser<'t> {
    pub(crate) fn new(term: &'t [u8]) -> Self {
        Self { term }
    }
}

impl<'i, 't> Parser<'i, &'t [u8]> for TermParser<'t> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, &'t [u8]> {
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
