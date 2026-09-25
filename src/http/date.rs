use crate::parser::{AnyParser, ParseResult, Parser, TermParser};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WeekDay {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Time {
    hour: u8,
    minute: u8,
    second: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Date {
    year: u16,
    month: Month,
    week_day: WeekDay,
    day: u8,
}

pub(crate) struct MonthP;
impl Parser for MonthP {
    type Out = Month;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        fn m(term: &[u8], month: Month) -> impl Parser<Out = Month> {
            TermParser::new(term).map(move |_| month)
        }

        AnyParser::new([
            m(b"Jan", Month::January),
            m(b"Feb", Month::February),
            m(b"Mar", Month::March),
            m(b"Apr", Month::April),
            m(b"May", Month::May),
            m(b"Jun", Month::June),
            m(b"Jul", Month::July),
            m(b"Aug", Month::August),
            m(b"Sep", Month::September),
            m(b"Oct", Month::October),
            m(b"Nov", Month::November),
            m(b"Dec", Month::December),
        ])
        .parse(input)
    }
}
