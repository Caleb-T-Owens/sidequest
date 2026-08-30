#![allow(dead_code)]

use crate::either::Either;
use crate::parser::{CharParser, MatchParser, ParseResult, Parser, RangeParser};

fn char_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| u < 128)
}

fn up_alpha_p() -> impl Parser<Out = u8> {
    RangeParser::new(b'A'..=b'Z')
}

fn lo_alpha_p() -> impl Parser<Out = u8> {
    RangeParser::new(b'a'..=b'z')
}

fn alpha_p() -> impl Parser<Out = u8> {
    up_alpha_p().or(lo_alpha_p()).map(Either::unify)
}

fn is_digit(u: u8) -> bool {
    (b'0'..=b'9').contains(&u)
}

fn digit_p() -> impl Parser<Out = u8> {
    MatchParser::new(is_digit)
}

fn is_ctl(u: u8) -> bool {
    (0..=31).contains(&u) || u == 127
}

fn ctl_p() -> impl Parser<Out = u8> {
    MatchParser::new(is_ctl)
}

fn cr_p() -> impl Parser<Out = u8> {
    CharParser::new(13)
}

fn lf_p() -> impl Parser<Out = u8> {
    CharParser::new(10)
}

fn is_sp(u: u8) -> bool {
    u == 32
}

fn sp_p() -> impl Parser<Out = u8> {
    MatchParser::new(is_sp)
}

fn is_ht(u: u8) -> bool {
    u == 32
}

fn ht_p() -> impl Parser<Out = u8> {
    CharParser::new(9)
}

fn dq_p() -> impl Parser<Out = u8> {
    CharParser::new(34)
}

fn crlf_p() -> impl Parser<Out = (u8, u8)> {
    cr_p().and(lf_p())
}

fn lws_p() -> impl Parser {
    let spaces_p = || sp_p().or(ht_p()).bounded_span(1, usize::MAX);

    spaces_p().or(crlf_p().and(spaces_p()))
}

fn is_text(u: u8) -> bool {
    !is_ctl(u) || is_sp(u)
}

fn text_p() -> impl Parser<Out = u8> {
    MatchParser::new(is_text)
}

fn hex_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| is_digit(u) || (b'a'..=b'f').contains(&u) || (b'A'..=b'F').contains(&u))
}

fn is_seperator(u: u8) -> bool {
    is_sp(u) || is_ht(u) || b"()<>@,;:\\\"/[]?={}".contains(&u)
}

fn token_p() -> impl Parser<Out = Vec<u8>> {
    MatchParser::new(|u| !is_seperator(u)).span()
}

fn quoted_pair_p() -> impl Parser<Out = u8> {
    CharParser::new(b'\\').and(char_p()).map(|(_, c)| c)
}

fn qd_text_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| is_text(u) && u != b'"')
}

fn quoted_string_p() -> impl Parser<Out = Vec<u8>> {
    dq_p()
        .and(qd_text_p().or(quoted_pair_p()).map(Either::unify).span())
        .and(dq_p())
        .map(|((_, s), _)| s)
}

fn ctext_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| is_text(u) && !b"()".contains(&u))
}

struct Comment(Vec<Either<Vec<u8>, Box<Comment>>>);

struct CommentParser;

impl Parser for CommentParser {
    type Out = Comment;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'(')
            .and(
                ctext_p()
                    .or(quoted_pair_p())
                    .map(Either::unify)
                    .span()
                    .or(CommentParser.map(Box::new))
                    .span()
                    .map(|c| Comment(c)),
            )
            .and(CharParser::new(b')'))
            .map(|((_, c), _)| c)
            .parse(input)
    }
}
