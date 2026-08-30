#![allow(dead_code)]

use crate::either::Either;
use crate::parser::{CharParser, MatchParser, ParseResult, Parser, RangeParser};

struct CharP;
impl Parser for CharP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(|u| u < 128).parse(input)
    }
}

struct UpAlphaP;
impl Parser for UpAlphaP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        RangeParser::new(b'A'..=b'Z').parse(input)
    }
}

struct LoAlphaP;
impl Parser for LoAlphaP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        RangeParser::new(b'a'..=b'z').parse(input)
    }
}

struct AlphaP;
impl Parser for AlphaP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UpAlphaP.or(LoAlphaP).map(Either::unify).parse(input)
    }
}

fn is_digit(u: u8) -> bool {
    (b'0'..=b'9').contains(&u)
}

struct DigitP;
impl Parser for DigitP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(is_digit).parse(input)
    }
}

fn is_ctl(u: u8) -> bool {
    (0..=31).contains(&u) || u == 127
}

struct CtlP;
impl Parser for CtlP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(is_ctl).parse(input)
    }
}

struct CrP;
impl Parser for CrP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'\r').parse(input)
    }
}

struct LfP;
impl Parser for LfP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'\n').parse(input)
    }
}

fn is_sp(u: u8) -> bool {
    u == b' '
}

struct SpP;
impl Parser for SpP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(is_sp).parse(input)
    }
}

fn is_ht(u: u8) -> bool {
    u == 9
}

struct HtP;
impl Parser for HtP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(is_ht).parse(input)
    }
}

struct DqP;
impl Parser for DqP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'"').parse(input)
    }
}

struct Crlf;
struct CrlfP;
impl Parser for CrlfP {
    type Out = Crlf;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CrP.and(LfP).map(|_| Crlf).parse(input)
    }
}

struct LwsP;
impl Parser for LwsP {
    type Out = Either<Vec<u8>, (Crlf, Vec<u8>)>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        let spaces_p = || SpP.or(HtP).map(Either::unify).bounded_span(1, usize::MAX);

        spaces_p().or(CrlfP.and(spaces_p())).parse(input)
    }
}

fn is_text(u: u8) -> bool {
    !is_ctl(u) || is_sp(u)
}

struct TextP;
impl Parser for TextP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(is_text).parse(input)
    }
}

struct HexP;
impl Parser for HexP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(|u| {
            is_digit(u) || (b'a'..=b'f').contains(&u) || (b'A'..=b'F').contains(&u)
        })
        .parse(input)
    }
}

fn is_seperator(u: u8) -> bool {
    is_sp(u) || is_ht(u) || b"()<>@,;:\\\"/[]?={}".contains(&u)
}

struct TokenP;
impl Parser for TokenP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(|u| !is_seperator(u)).span().parse(input)
    }
}

struct QuotedPairP;
impl Parser for QuotedPairP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'\\')
            .and(CharP)
            .map(|(_, c)| c)
            .parse(input)
    }
}

struct QdTextP;
impl Parser for QdTextP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(|u| is_text(u) && u != b'"').parse(input)
    }
}

struct QuotedStringP;
impl Parser for QuotedStringP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        DqP.and(QdTextP.or(QuotedPairP).map(Either::unify).span())
            .and(DqP)
            .map(|((_, s), _)| s)
            .parse(input)
    }
}

struct CTextP;
impl Parser for CTextP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(|u| is_text(u) && !b"()".contains(&u)).parse(input)
    }
}

struct Comment(Vec<Either<Vec<u8>, Box<Comment>>>);

struct CommentP;
impl Parser for CommentP {
    type Out = Comment;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'(')
            .and(
                CTextP
                    .or(QuotedPairP)
                    .map(Either::unify)
                    .span()
                    .or(CommentP.map(Box::new))
                    .span()
                    .map(|c| Comment(c)),
            )
            .and(CharParser::new(b')'))
            .map(|((_, c), _)| c)
            .parse(input)
    }
}
