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

pub(crate) trait Parser<Out> {
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Out>;

    #[allow(unused)]
    fn inspect(self: Self) -> InspectParser<Out>
    where
        Self: Sized + 'static,
    {
        InspectParser(Box::new(self))
    }

    #[allow(unused)]
    fn map<U, F>(self: Self, op: F) -> MapParser<Out, U, F>
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
    fn or<OutB, B>(self: Self, b: B) -> OrParser<Out, OutB>
    where
        B: Parser<OutB> + 'static,
        Self: Sized + 'static,
    {
        OrParser {
            a: Box::new(self),
            b: Box::new(b),
        }
    }

    #[allow(unused)]
    fn and<OutB, B>(self: Self, b: B) -> AndParser<Out, OutB>
    where
        B: Parser<OutB> + 'static,
        Self: Sized + 'static,
    {
        AndParser {
            a: Box::new(self),
            b: Box::new(b),
        }
    }

    #[allow(unused)]
    fn span(self: Self) -> SpanParser<Out>
    where
        Self: Sized + 'static,
    {
        SpanParser(Box::new(self))
    }
}


#[allow(unused)]
pub(crate) struct SpanParser<Out>(Box<dyn Parser<Out>>);

impl<Out> Parser<Vec<Out>> for SpanParser<Out> {
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Vec<Out>> {
        let mut result = vec![];
        let mut rest = input;

        loop {
            match self.0.parse(rest) {
                ParseResult::Found { subject, rest: new_rest } => {
                    result.push(subject);
                    rest = new_rest;
                },
                ParseResult::Missed { .. } => break
            }
        }

        ParseResult::Found { subject: result, rest }
    }
}

#[allow(unused)]
pub(crate) struct AndParser<OutA, OutB> {
    a: Box<dyn Parser<OutA>>,
    b: Box<dyn Parser<OutB>>,
}

impl<OutA, OutB> Parser<(OutA, OutB)> for AndParser<OutA, OutB> {
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, (OutA, OutB)> {
        self.a
            .parse(input)
            .then(|a, a_rest| self.b.parse(a_rest).map(|b| (a, b)))
    }
}

#[allow(unused)]
pub(crate) struct OrParser<OutA, OutB> {
    a: Box<dyn Parser<OutA>>,
    b: Box<dyn Parser<OutB>>,
}

impl<OutA, OutB> Parser<Either<OutA, OutB>> for OrParser<OutA, OutB> {
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Either<OutA, OutB>> {
        self.a.parse(input).or_else(|| self.b.parse(input))
    }
}

#[allow(unused)]
pub(crate) struct MapParser<Out, U, F: FnOnce(Out) -> U> {
    parser: Box<dyn Parser<Out>>,
    op: F,
}

impl<Out, U, F: FnOnce(Out) -> U + Clone + Copy> Parser<U> for MapParser<Out, U, F> {
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, U> {
        self.parser.parse(input).map(self.op)
    }
}

#[allow(unused)]
pub(crate) struct InspectParser<Out>(Box<dyn Parser<Out>>);

impl<Out: Debug> Parser<Out> for InspectParser<Out> {
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Out> {
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

impl<'t> Parser<&'t [u8]> for TermParser<'t> {
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
