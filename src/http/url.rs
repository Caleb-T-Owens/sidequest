use crate::either::Either;
use crate::http::primatives::{AlphaP, DigitP, HexP, hex_chars_to_nibble, is_alpha, is_digit};
use crate::parser::{
    CharParser, InsensitiveTermParser, MatchParser, ParseResult, Parser, TermParser,
};

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

fn is_alphanum(u: u8) -> bool {
    is_alpha(u) || is_digit(u)
}

pub(crate) struct AlphanumP;
impl Parser for AlphanumP {
    type Out = u8;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        MatchParser::new(is_alphanum).parse(input)
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

#[derive(Debug)]
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

#[derive(Debug)]
pub(crate) struct Opaque;
pub(crate) struct OpaquePartP;
impl Parser for OpaquePartP {
    type Out = Opaque;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UricNoSlashP.and(UricP.span()).map(|_| Opaque).parse(input)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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
            .and(
                AlphanumP
                    .or(CharParser::new(b'-'))
                    .map(Either::unify)
                    .bounded_span(1, usize::MAX)
                    .select(|u| u.last().is_some_and(|u| is_alphanum(*u))),
            )
            .map(|(a, b)| [a].into_iter().chain(b).collect())
            .or(AlphaP.map(|a| vec![a]))
            .map(Either::unify)
            .parse(input)
    }
}

pub(crate) struct DomainLabelP;
impl Parser for DomainLabelP {
    type Out = Vec<u8>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AlphanumP
            .and(
                AlphanumP
                    .or(CharParser::new(b'-'))
                    .map(Either::unify)
                    .bounded_span(1, usize::MAX)
                    .select(|u| u.last().is_some_and(|u| is_alphanum(*u))),
            )
            .map(|(a, b)| [a].into_iter().chain(b).collect())
            .or(AlphanumP.map(|a| vec![a]))
            .map(Either::unify)
            .parse(input)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub(crate) struct Server {
    user_info: Option<Vec<u8>>,
    location: HostPort,
}
pub(crate) struct ServerP;
impl Parser for ServerP {
    type Out = Option<Server>;
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        UserInfoP
            .inspect()
            .and(CharParser::new(b'@').inspect())
            .inspect()
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

pub(crate) type Authority = Either<Option<Server>, Vec<u8>>;
pub(crate) struct AuthorityP;
impl Parser for AuthorityP {
    type Out = Authority;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        ServerP.or(RegNameP).parse(input)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub(crate) struct RelPath(Vec<Segment>);
pub(crate) struct RelPathP;
impl Parser for RelPathP {
    type Out = RelPath;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        RelSegmentP
            .and(RelPathP)
            .map(|(a, b)| RelPath([a].into_iter().chain(b.0).collect()))
            .parse(input)
    }
}

#[derive(Debug)]
pub(crate) struct NetPath {
    authority: Authority,
    path: Option<AbsPath>,
}
pub(crate) struct NetPathP;
impl Parser for NetPathP {
    type Out = NetPath;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        TermParser::new(b"//")
            .then(|_| {
                AuthorityP
                    .and(AbsPathP.optional())
                    .map(|(authority, path)| NetPath { authority, path })
            })
            .parse(input)
    }
}

#[derive(Debug)]
pub(crate) enum UriPath {
    NetPath(NetPath),
    RelPath(RelPath),
    AbsPath(AbsPath),
}

#[derive(Debug)]
pub(crate) struct RelativeUri {
    path: UriPath,
    query: Option<Vec<u8>>,
}

pub(crate) struct HierPartP;
impl Parser for HierPartP {
    type Out = RelativeUri;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        NetPathP
            .map(UriPath::NetPath)
            .or(AbsPathP.map(UriPath::AbsPath))
            .map(Either::unify)
            .and(CharParser::new(b'?').then(|_| QueryP).optional())
            .map(|(path, query)| RelativeUri { path, query })
            .parse(input)
    }
}

pub(crate) struct RelativeUriP;
impl Parser for RelativeUriP {
    type Out = RelativeUri;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        NetPathP
            .map(UriPath::NetPath)
            .or(AbsPathP.map(UriPath::AbsPath))
            .map(Either::unify)
            .or(RelPathP.map(UriPath::RelPath))
            .map(Either::unify)
            .and(CharParser::new(b'?').then(|_| QueryP).optional())
            .map(|(path, query)| RelativeUri { path, query })
            .parse(input)
    }
}

#[derive(Debug)]
pub(crate) struct AbsoluteUri {
    scheme: Scheme,
    location: Either<RelativeUri, Opaque>,
}

pub(crate) struct AbsoluteUriP;
impl Parser for AbsoluteUriP {
    type Out = AbsoluteUri;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        SchemeP
            .and(CharParser::new(b':'))
            .map(|(a, _)| a)
            .and(HierPartP.or(OpaquePartP))
            .map(|(scheme, location)| AbsoluteUri { scheme, location })
            .parse(input)
    }
}

#[derive(Debug)]
pub(crate) struct UriReference {
    uri: Either<AbsoluteUri, RelativeUri>,
    fragment: Option<Vec<u8>>,
}
pub(crate) struct UriReferenceP;
impl Parser for UriReferenceP {
    type Out = UriReference;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AbsoluteUriP
            .or(RelativeUriP)
            .and(CharParser::new(b'#').then(|_| FragmentP).optional())
            .map(|(uri, fragment)| UriReference { uri, fragment })
            .parse(input)
    }
}
