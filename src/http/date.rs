use crate::{
    http::U8P,
    parser::{AnyParser, CharParser, ParseResult, Parser, TermParser},
};

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

impl Time {
    pub(crate) fn valid(&self) -> bool {
        (0..=23).contains(&self.hour)
            && (0..=59).contains(&self.minute)
            && (0..=59).contains(&self.second)
    }
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

fn d(term: &[u8], d: WeekDay) -> impl Parser<Out = WeekDay> {
    TermParser::new(term).map(move |_| d)
}

pub(crate) struct WeekDayP;
impl Parser for WeekDayP {
    type Out = WeekDay;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AnyParser::new([
            d(b"Monday", WeekDay::Monday),
            d(b"Tuesday", WeekDay::Tuesday),
            d(b"Wednesday", WeekDay::Wednesday),
            d(b"Thursday", WeekDay::Thursday),
            d(b"Friday", WeekDay::Friday),
            d(b"Saturday", WeekDay::Saturday),
            d(b"Sunday", WeekDay::Sunday),
        ])
        .parse(input)
    }
}

pub(crate) struct WkDayP;
impl Parser for WkDayP {
    type Out = WeekDay;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        AnyParser::new([
            d(b"Mon", WeekDay::Monday),
            d(b"Tue", WeekDay::Tuesday),
            d(b"Wed", WeekDay::Wednesday),
            d(b"Thu", WeekDay::Thursday),
            d(b"Fri", WeekDay::Friday),
            d(b"Sat", WeekDay::Saturday),
            d(b"Sun", WeekDay::Sunday),
        ])
        .parse(input)
    }
}

pub(crate) struct TimeP;
impl Parser for TimeP {
    type Out = Time;

    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Self::Out> {
        let digit_2 = || U8P::Bounded(2, 2);
        let sep = || CharParser::new(b':');
        digit_2()
            .and(sep())
            .and(digit_2())
            .and(sep())
            .and(digit_2())
            .map(|((((a, _), b), _), c)| Time {
                hour: a,
                minute: b,
                second: c,
            })
            .parse(input)
            .then(|subject, rest| {
                if subject.valid() {
                    ParseResult::Found { subject, rest }
                } else {
                    ParseResult::Missed { rest: input }
                }
            })
    }
}
