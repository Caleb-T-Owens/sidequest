use crate::either::Either;
use crate::http::primatives::{AlphaP, DigitP, HexP, hex_chars_to_nibble};
use crate::parser::{CharParser, MatchParser, ParseResult, Parser};

pub(crate) struct EscapedP;
impl Parser for EscapedP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'%')
            .then(|_| HexP.and(HexP))
            .map(hex_chars_to_nibble)
            .parse(input)
    }
}

pub(crate) struct MarkP;
impl Parser for MarkP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(|u| b"-_.!~*'()".contains(&u)).parse(input)
    }
}

pub(crate) struct AlphanumP;
impl Parser for AlphanumP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AlphaP.or(DigitP).map(Either::unify).parse(input)
    }
}

pub(crate) struct UnreservedP;
impl Parser for UnreservedP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AlphanumP.or(MarkP).map(Either::unify).parse(input)
    }
}

pub(crate) struct ReservedP;
impl Parser for ReservedP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(|u| b";/?:@&=+$,".contains(&u)).parse(input)
    }
}

pub(crate) struct UricP;
impl Parser for UricP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        ReservedP
            .or(UnreservedP)
            .map(Either::unify)
            .or(EscapedP)
            .map(Either::unify)
            .parse(input)
    }
}

pub(crate) struct FragmentP;
impl Parser for FragmentP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UricP.span().parse(input)
    }
}

pub(crate) struct QueryP;
impl Parser for QueryP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UricP.span().parse(input)
    }
}
