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

    fn inspect(self: Self) -> InspectorParser<'i, Out>
    where
        Self: Sized + 'static,
    {
        InspectorParser(Box::new(self))
    }

    fn map<U, F>(self: Self, op: F) -> MappedParser<'i, Out, U, F>
    where
        F: FnOnce(Out) -> U + Clone + Copy,
        Self: Sized + 'static,
    {
        MappedParser {
            parser: Box::new(self),
            op,
        }
    }
}

pub(crate) struct MappedParser<'i, Out, U, F: FnOnce(Out) -> U> {
    parser: Box<dyn Parser<'i, Out>>,
    op: F,
}

impl<'i, Out, U, F: FnOnce(Out) -> U + Clone + Copy> Parser<'i, U> for MappedParser<'i, Out, U, F> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, U> {
        self.parser.parse(input).map(self.op)
    }
}

pub(crate) struct InspectorParser<'i, Out>(Box<dyn Parser<'i, Out>>);

impl<'i, Out: Debug> Parser<'i, Out> for InspectorParser<'i, Out> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Out> {
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
