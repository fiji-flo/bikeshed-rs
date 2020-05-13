use chrono::format::strftime::StrftimeItems;
use chrono::format::{DelayedFormat, ParseError};
use chrono::{Datelike, NaiveDate};
use std::fmt;

pub type ParseResult = Result<Date, ParseError>;

#[derive(Debug, Clone)]
pub struct Date {
    pub date: NaiveDate,
}

impl Default for Date {
    fn default() -> Self {
        Date::now()
    }
}

impl fmt::Display for Date {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.date.fmt(fmt)
    }
}

impl Date {
    pub fn new(date: NaiveDate) -> Self {
        Date { date }
    }

    pub fn now() -> Self {
        Date::new(chrono::offset::Utc::now().naive_utc().date())
    }

    pub fn parse_from_str(date_str: &str, fmt: &str) -> ParseResult {
        match NaiveDate::parse_from_str(date_str, fmt) {
            Ok(date) => Ok(Date::new(date)),
            Err(err) => Err(err),
        }
    }

    #[inline]
    pub fn format<'a>(&self, fmt: &'a str) -> DelayedFormat<StrftimeItems<'a>> {
        self.date.format(fmt)
    }

    #[inline]
    pub fn day(&self) -> u32 {
        self.date.day()
    }
}
