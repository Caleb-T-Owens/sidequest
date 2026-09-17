use crate::either::Either;
use crate::http::primatives::{AlphaP, DigitP, HexP, hex_chars_to_nibble};
use crate::parser::{CharParser, InsensitiveTermParser, MatchParser, ParseResult, Parser};

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
        UricNoSlashP.and(UricP.span()).map(|_| Opaque).parse(input)
    }
}

pub(crate) struct AbsPath(pub Vec<Segment>);
pub(crate) struct AbsPathP;
impl Parser for AbsPathP {
    type Out = AbsPath;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        CharParser::new(b'/')
            .then(|_| PathSegmentsP)
            .map(AbsPath)
            .parse(input)
    }
}

pub(crate) struct PathP;
impl Parser for PathP {
    type Out = Either<AbsPath, Opaque>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AbsPathP.or(OpaquePartP).parse(input)
    }
}

pub(crate) struct PortP;
impl Parser for PortP {
    type Out = Option<u32>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        DigitP
            .span()
            .map(|r| {
                if r.is_empty() {
                    None
                } else {
                    Some(
                        r.into_iter()
                            .fold(0, |acc, d| acc * 10 + ((d - b'0') as u32)),
                    )
                }
            })
            .parse(input)
    }
}

pub(crate) struct IPv4Address(u8, u8, u8, u8);
pub(crate) struct IPv4AddressP;
impl Parser for IPv4AddressP {
    type Out = IPv4Address;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        let part = || {
            DigitP.bounded_span(1, usize::MAX).map(|r| {
                r.into_iter()
                    .fold(0, |acc, d| acc * 10 + ((d - b'0') as u8))
            })
        };
        let char = || CharParser::new(b'.');

        part()
            .and(char())
            .and(part())
            .and(char())
            .and(part())
            .and(char())
            .and(part())
            .map(|((((((a, _), b), _), c), _), d)| IPv4Address(a, b, c, d))
            .parse(input)
    }
}

pub(crate) struct TopLabelP;
impl Parser for TopLabelP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AlphaP
            .map(|a| vec![a])
            .or(AlphaP
                .and(
                    AlphanumP
                        .or(CharParser::new(b'-'))
                        .map(Either::unify)
                        .span(),
                )
                .and(AlphanumP)
                .map(|((a, b), c)| [a].into_iter().chain(b).chain([c]).collect()))
            .map(Either::unify)
            .parse(input)
    }
}

pub(crate) struct DomainLabelP;
impl Parser for DomainLabelP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AlphanumP
            .map(|a| vec![a])
            .or(AlphanumP
                .and(
                    AlphanumP
                        .or(CharParser::new(b'-'))
                        .map(Either::unify)
                        .span(),
                )
                .and(AlphanumP)
                .map(|((a, b), c)| [a].into_iter().chain(b).chain([c]).collect()))
            .map(Either::unify)
            .parse(input)
    }
}

pub(crate) struct Hostname {
    segments: Vec<Vec<u8>>,
    trailing_dot: bool,
}
pub(crate) struct HostnameP;
impl Parser for HostnameP {
    type Out = Hostname;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        DomainLabelP
            .and(CharParser::new(b'.'))
            .map(|(a, _)| a)
            .span()
            .and(TopLabelP)
            .and(CharParser::new(b'.').optional())
            .map(|((a, b), c)| Hostname {
                segments: a.into_iter().chain([b]).collect(),
                trailing_dot: c.is_some(),
            })
            .parse(input)
    }
}

pub(crate) enum Host {
    Hostname(Hostname),
    Ip(IPv4Address),
}
pub(crate) struct HostP;
impl Parser for HostP {
    type Out = Host;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        HostnameP
            .or(IPv4AddressP)
            .map(|a| match a {
                Either::Left(a) => Host::Hostname(a),
                Either::Right(b) => Host::Ip(b),
            })
            .parse(input)
    }
}

pub(crate) struct HostPort(Host, Option<u32>);
pub(crate) struct HostPortP;
impl Parser for HostPortP {
    type Out = HostPort;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        HostP
            .and(CharParser::new(b':').then(|_| PortP).optional())
            .map(|(hn, p)| HostPort(hn, p.flatten()))
            .parse(input)
    }
}

pub(crate) struct UserInfoP;
impl Parser for UserInfoP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UnreservedP
            .or(EscapedP)
            .map(Either::unify)
            .or(MatchParser::new(|u| b";:&=+$,".contains(&u)))
            .map(Either::unify)
            .span()
            .parse(input)
    }
}

pub(crate) struct Server {
    user_info: Option<Vec<u8>>,
    location: HostPort,
}
pub(crate) struct ServerP;
impl Parser for ServerP {
    type Out = Option<Server>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UserInfoP
            .and(CharParser::new(b'@'))
            .map(|(u, _)| u)
            .optional()
            .and(HostPortP)
            .map(|(user_info, location)| Server {
                user_info,
                location,
            })
            .optional()
            .parse(input)
    }
}

pub(crate) struct RegNameP;
impl Parser for RegNameP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UnreservedP
            .or(EscapedP)
            .map(Either::unify)
            .or(MatchParser::new(|u| b"$,;:@&=+".contains(&u)))
            .map(Either::unify)
            .bounded_span(1, usize::MAX)
            .parse(input)
    }
}

pub(crate) struct AuthorityP;
impl Parser for AuthorityP {
    type Out = Either<Option<Server>, Vec<u8>>;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        ServerP.or(RegNameP).parse(input)
    }
}

pub(crate) enum Scheme {
    Http,
    Other(Vec<u8>),
}
pub(crate) struct SchemeP;
impl Parser for SchemeP {
    type Out = Scheme;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        InsensitiveTermParser::new("http")
            .map(|_| Scheme::Http)
            .or(AlphaP
                .and(
                    AlphanumP
                        .or(MatchParser::new(|u| b"+-.".contains(&u)))
                        .map(Either::unify)
                        .span(),
                )
                .map(|(a, b)| Scheme::Other([a].into_iter().chain(b).collect())))
            .map(Either::unify)
            .parse(input)
    }
}

pub(crate) struct RelSegmentP;
impl Parser for RelSegmentP {
    type Out = Segment;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UnreservedP
            .or(EscapedP)
            .map(Either::unify)
            .or(MatchParser::new(|u| b";@&=+$,".contains(&u)))
            .map(Either::unify)
            .bounded_span(1, usize::MAX)
            .map(|u| Segment(vec![u]))
            .parse(input)
    }
}
