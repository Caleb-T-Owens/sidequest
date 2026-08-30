#![allow(dead_code)]

use crate::either::Either;
use crate::parser::{MatchParser, Parser};

fn char_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| u < 128)
}

fn up_alpha_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| (b'A'..=b'Z').contains(&u))
}

fn lo_alpha_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| (b'a'..=b'z').contains(&u))
}

fn alpha_p() -> impl Parser<Out = u8> {
    up_alpha_p().or(lo_alpha_p()).map(Either::unify)
}

fn digit_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| (b'0'..=b'9').contains(&u))
}

fn ctl_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| (0..=31).contains(&u) || u == 127)
}

fn cr_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| u == 13)
}

fn lf_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| u == 10)
}

fn sp_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| u == 32)
}

fn ht_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| u == 9)
}

fn dq_p() -> impl Parser<Out = u8> {
    MatchParser::new(|u| u == 34)
}
