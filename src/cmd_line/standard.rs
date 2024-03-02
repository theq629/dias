use super::ParsingError;
use crate::cmd_line::shared::ArgId;
use core::str::FromStr;
use std::any::Any;
use std::error::Error;
use std::ffi::OsString;

trait ArgHandler {
    fn match_arg<'a>(&'a self, lexopt_arg: &lexopt::Arg) -> Option<String>;
    fn get_value(
        &self,
        arg_name: &String,
        lexopt_parser: &mut lexopt::Parser,
    ) -> Result<Box<dyn Any>, ParsingError>;
}

struct FlagArgHandler {
    short: &'static [char],
    long: &'static [&'static str],
}

impl ArgHandler for FlagArgHandler {
    fn match_arg<'a>(&'a self, lexopt_arg: &lexopt::Arg) -> Option<String> {
        match lexopt_arg {
            lexopt::Arg::Short(name) => self.short.contains(&name).then(|| name.to_string()),
            lexopt::Arg::Long(name) => self.long.contains(&name).then(|| name.to_string()),
            _ => None,
        }
    }

    fn get_value(&self, _: &String, _: &mut lexopt::Parser) -> Result<Box<dyn Any>, ParsingError> {
        Ok(Box::new(true))
    }
}

struct OptionArgHandler<F> {
    short: &'static [char],
    long: &'static [&'static str],
    parse: F,
}

impl<T, E, F> ArgHandler for OptionArgHandler<F>
where
    T: 'static,
    E: 'static + Into<Box<dyn Error>>,
    F: 'static + Fn(&str) -> Result<T, E>,
{
    fn match_arg<'a>(&'a self, lexopt_arg: &lexopt::Arg) -> Option<String> {
        match lexopt_arg {
            lexopt::Arg::Short(name) => self.short.contains(&name).then(|| name.to_string()),
            lexopt::Arg::Long(name) => self.long.contains(&name).then(|| name.to_string()),
            _ => None,
        }
    }

    fn get_value(
        &self,
        arg_name: &String,
        lexopt_parser: &mut lexopt::Parser,
    ) -> Result<Box<dyn Any>, ParsingError> {
        let value = lexopt_parser
            .value()
            .map_err(|_| ParsingError::MissingValue {
                arg_name: arg_name.to_string(),
            })?;
        match value.to_str() {
            Some(value) => (self.parse)(value).map_err(|e| e.into()),
            None => Err(Box::new(lexopt::Error::NonUnicodeValue(value)) as Box<dyn Error>),
        }
        .map_err(|e| ParsingError::ValueParsingFailed {
            arg_name: arg_name.to_string(),
            error: e,
        })
        .map(|v| Box::new(v) as Box<dyn Any>)
    }
}

pub struct Parser {
    args: Vec<Box<dyn ArgHandler>>,
}

impl Parser {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn parse_args<I>(&self, args: I) -> Result<Parsed, ParsingError>
    where
        I: IntoIterator,
        I::Item: Into<OsString>,
    {
        Ok(self.parse_lexopt(lexopt::Parser::from_iter(args))?)
    }

    fn parse_next(
        &self,
        lexopt_parser: &mut lexopt::Parser,
    ) -> Result<Option<(usize, Box<dyn Any>)>, ParsingError> {
        if let Some(lexopt_arg) = lexopt_parser
            .next()
            .map_err(|_| ParsingError::ParsingFailed)?
        {
            for (id, arg) in self.args.iter().enumerate() {
                if let Some(arg_name) = arg.match_arg(&lexopt_arg) {
                    return Ok(Some((id, arg.get_value(&arg_name, lexopt_parser)?)));
                }
            }
            match lexopt_arg {
                lexopt::Arg::Short(name) => Err(ParsingError::UnknownOption {
                    arg_name: name.to_string(),
                }),
                lexopt::Arg::Long(name) => Err(ParsingError::UnknownOption {
                    arg_name: name.to_string(),
                }),
                lexopt::Arg::Value(_) => Err(ParsingError::UnknownValue),
            }
        } else {
            Ok(None)
        }
    }

    fn parse_lexopt(&self, mut lexopt_parser: lexopt::Parser) -> Result<Parsed, ParsingError> {
        let mut values: Vec<Option<Box<dyn Any>>> = self.args.iter().map(|_| None).collect();
        loop {
            if let Some((id, value)) = self.parse_next(&mut lexopt_parser)? {
                values[id] = Some(value);
            } else {
                break;
            }
        }
        Ok(Parsed { values })
    }
}

impl super::Parser for Parser {
    type ArgId<T> = ArgId<T>;
    type Parsed = Parsed;

    fn add_flag(
        &mut self,
        short: &'static [char],
        long: &'static [&'static str],
    ) -> Self::ArgId<bool> {
        let id = self.args.len();
        self.args.push(Box::new(FlagArgHandler { short, long }));
        ArgId::new(id)
    }

    fn add_option<T: 'static, E>(
        &mut self,
        short: &'static [char],
        long: &'static [&'static str],
    ) -> Self::ArgId<T>
    where
        T: FromStr<Err = E>,
        E: 'static + Into<Box<dyn Error>>,
    {
        self.add_option_with(short, long, FromStr::from_str)
    }

    fn add_option_with<T: 'static, E, F>(
        &mut self,
        short: &'static [char],
        long: &'static [&'static str],
        parse: F,
    ) -> Self::ArgId<T>
    where
        F: 'static + Fn(&str) -> Result<T, E>,
        E: 'static + Into<Box<dyn Error>>,
    {
        let id = self.args.len();
        self.args
            .push(Box::new(OptionArgHandler { short, long, parse }));
        ArgId::new(id)
    }

    fn parse(&self) -> Result<Self::Parsed, ParsingError> {
        self.parse_args(std::env::args_os())
    }
}

pub struct Parsed {
    values: Vec<Option<Box<dyn Any>>>,
}

impl super::Parsed for Parsed {
    type Parser = Parser;

    fn get<T: 'static>(&self, arg: &ArgId<T>) -> Option<&T> {
        self.values[arg.id]
            .as_ref()
            .map(|v| v.downcast_ref().expect("wrong type"))
    }
}

#[cfg(test)]
mod tests {
    use super::super::generic::tests as generic_tests;
    use super::super::Parser as _;
    use super::*;

    fn mark_arg(arg: String) -> String {
        if arg.len() == 1 {
            format!("-{}", arg)
        } else if arg.len() > 1 {
            format!("--{}", arg)
        } else {
            arg
        }
    }

    impl generic_tests::ParseTest for Parser {
        fn new() -> Self {
            Parser::new()
        }

        fn parse_test_args<S: ToString>(
            &self,
            args: &[(S, Option<S>)],
        ) -> Result<Self::Parsed, ParsingError> {
            let flat_args = args.iter().flat_map(|arg| match arg {
                (arg, None) => vec![mark_arg(arg.to_string())],
                (arg, Some(value)) => vec![mark_arg(arg.to_string()), value.to_string()],
            });
            self.parse_args(["".to_string()].into_iter().chain(flat_args))
        }
    }

    #[test]
    fn flags() {
        generic_tests::flags::<Parser>();
    }

    #[test]
    fn flags_unknown() {
        generic_tests::flags_unknown::<Parser>();
    }

    #[test]
    fn options() {
        generic_tests::options::<Parser>();
    }

    #[test]
    fn options_unknown() {
        generic_tests::options_unknown::<Parser>();
    }

    #[test]
    fn options_missing_value() {
        generic_tests::options_missing_value::<Parser>();
    }

    #[test]
    fn extra_value() {
        let mut parser = Parser::new();
        parser.add_option::<i32, _>(&['f'], &["foo"]);
        parser.add_flag(&['b'], &["bar"]);

        assert!(match parser.parse_args(&["", "--foo", "123", "abc"]) {
            Err(ParsingError::UnknownValue) => true,
            _ => false,
        });
        assert!(match parser.parse_args(&["", "--bar", "abc"]) {
            Err(ParsingError::UnknownValue) => true,
            _ => false,
        });
    }
}
