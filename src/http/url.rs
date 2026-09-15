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

pub(crate) struct UricNoSlashP;
impl Parser for UricNoSlashP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UnreservedP
            .or(EscapedP)
            .map(Either::unify)
            .or(MatchParser::new(|u| b";?:@&=+$,".contains(&u)))
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

pub(crate) struct PCharP;
impl Parser for PCharP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UnreservedP
            .or(EscapedP)
            .map(Either::unify)
            .or(MatchParser::new(|b| b":@&=+$,".contains(&b)))
            .map(Either::unify)
            .parse(input)
    }
}

pub(crate) struct ParamP;
impl Parser for ParamP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        PCharP.span().parse(input)
    }
}

pub(crate) struct Segment(pub Vec<Vec<u8>>);
pub(crate) struct SegmentP;
impl Parser for SegmentP {
    type Out = Segment;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        ParamP
            .and(CharParser::new(b';').then(|_| ParamP).span())
            .map(|(a, b)| Segment([a].into_iter().chain(b).collect()))
            .parse(input)
    }
}

pub(crate) struct PathSegmentsP;
impl Parser for PathSegmentsP {
    type Out = Vec<Segment>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        SegmentP
            .and(CharParser::new(b'/').then(|_| SegmentP).span())
            .map(|(a, b)| [a].into_iter().chain(b).collect())
            .parse(input)
    }
}

pub(crate) struct Opaque;
pub(crate) struct OpaquePartP;
impl Parser for OpaquePartP {
    type Out = Opaque;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
       UricNoSlashP 
            .and(UricP.span())
            .map(|_| Opaque)
            .parse(input)
    }
}

pub(crate) struct AbsolutePath(pub Vec<Segment>);
pub(crate) struct AbsolutePathP;
impl Parser for AbsolutePathP {
    type Out = AbsolutePath;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'/').then(|_| PathSegmentsP).map(AbsolutePath).parse(input)
    }
}

pub(crate) struct PathP;
impl Parser for PathP {
    type Out = Either<AbsolutePath, Opaque>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AbsolutePathP.or(OpaquePartP).parse(input)
    }
}
