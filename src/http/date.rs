use crate::either::Either;
use crate::parser::{ParseResult, Parser, TermParser};

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

