use super::ParsingError;
use crate::cmd_line::shared::ArgId;
use core::str::FromStr;
use std::error::Error;

/// Parser which will always have no values for arguments.
#[derive(Default)]
pub struct DummyParser {
    num_args: usize,
}

impl DummyParser {
    pub fn new() -> Self {
        Self { num_args: 0 }
    }
}

impl super::Parser for DummyParser {
    type ArgId<T> = ArgId<T>;
    type Parsed = DummyParsed;

    fn add_flag(&mut self, _: &'static [char], _: &'static [&'static str]) -> Self::ArgId<bool> {
        let id = self.num_args;
        self.num_args += 1;
        ArgId::new(id)
    }

    fn add_option<T, E>(
        &mut self,
        short: &'static [char],
        long: &'static [&'static str],
    ) -> Self::ArgId<T>
    where
        T: FromStr<Err = E> + 'static,
        E: 'static + Into<Box<dyn Error>>,
    {
        self.add_option_with(short, long, FromStr::from_str)
    }

    fn add_option_with<T: 'static, E, F>(
        &mut self,
        _: &'static [char],
        _: &'static [&'static str],
        _: F,
    ) -> Self::ArgId<T>
    where
        F: 'static + Fn(&str) -> Result<T, E>,
        E: 'static + Into<Box<dyn Error>>,
    {
        let id = self.num_args;
        self.num_args += 1;
        ArgId::new(id)
    }

    fn parse(&self) -> Result<Self::Parsed, ParsingError> {
        Ok(DummyParsed {})
    }
}

/// Parsed arguments which will always have no values.
#[derive(Default)]
pub struct DummyParsed {}

impl DummyParsed {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Parsed for DummyParsed {
    type Parser = DummyParser;

    fn get<T: 'static>(&self, _: &ArgId<T>) -> Option<&T> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Parsed as _, Parser as _};
    use super::*;

    #[test]
    fn flags() {
        let mut parser = DummyParser::new();
        let foo = parser.add_flag(&['f'], &["foo"]);
        let bar = parser.add_flag(&['b'], &["bar"]);

        let args = parser.parse().unwrap();
        assert_eq!(args.get(&foo), None);
        assert_eq!(args.get(&bar), None);
    }

    #[test]
    fn options() {
        let mut parser = DummyParser::new();
        let foo = parser.add_option_with::<_, _, _>(&['f'], &["foo"], str::parse::<i32>);
        let bar = parser.add_option::<String, _>(&['b'], &["bar"]);

        let args = parser.parse().unwrap();
        assert_eq!(args.get(&foo), None);
        assert_eq!(args.get(&bar), None);
    }
}
