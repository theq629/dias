use core::str::FromStr;
use std::error::Error;

pub trait Parser {
    type ArgId<T>;
    type Parsed: Parsed<Parser = Self>;

    /// Add a boolean flag. The argument needs no value on the command line, but will be treated as
    /// a [bool] value when parsed.
    fn add_flag(
        &mut self,
        short: &'static [char],
        long: &'static [&'static str],
    ) -> Self::ArgId<bool>;

    /// Add an option argument with a value parsed via [FromStr].
    fn add_option<T: 'static, E>(
        &mut self,
        short: &'static [char],
        long: &'static [&'static str],
    ) -> Self::ArgId<T>
    where
        T: FromStr<Err = E>,
        E: 'static + Into<Box<dyn Error>>;

    /// Add an option argument with a value parsed via a given function.
    fn add_option_with<T: 'static, E, F>(
        &mut self,
        short: &'static [char],
        long: &'static [&'static str],
        parse: F,
    ) -> Self::ArgId<T>
    where
        F: 'static + Fn(&str) -> Result<T, E>,
        E: 'static + Into<Box<dyn Error>>;

    /// Parse the arguments the program was run with.
    fn parse(&self) -> Result<Self::Parsed, ParsingError>;
}

pub trait Parsed {
    type Parser: Parser;

    /// Get the value of an option argument if it was present, or [None] otherwise.
    fn get<T: 'static>(&self, arg: &<Self::Parser as Parser>::ArgId<T>) -> Option<&T>;
}

/// An error during parsing.
///
/// Not all parser implementations will be able to catch all errors, and some errors may not be
/// applicable to some parsers at all.
#[derive(Debug)]
pub enum ParsingError {
    /// An overall failure to parse the arguments input.
    ParsingFailed,

    /// Parsing of the value for an option failed. Provides an underlying error, which is generally
    /// from the caller-provided value parsing function, but could also be from internal
    /// conversions (eg string conversions).
    ValueParsingFailed {
        arg_name: String,
        error: Box<dyn Error>,
    },

    /// Parsing found an unknown option argument.
    UnknownOption { arg_name: String },

    /// Parsing did not find a value for a non-flag option argument.
    MissingValue { arg_name: String },

    /// Parsing found a value not corresponding to any option.
    UnknownValue,
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParsingFailed => write!(f, "could not parse arguments"),
            Self::ValueParsingFailed { arg_name, error } => write!(
                f,
                "could not parse value for option argument {}: {}",
                arg_name, error
            ),
            Self::UnknownOption { arg_name } => {
                write!(f, "found unknown option argument {}", arg_name)
            }
            Self::MissingValue { arg_name } => {
                write!(f, "no value for option argument {}", arg_name)
            }
            Self::UnknownValue => write!(
                f,
                "found unknown value which is not an option argument or expected value for one"
            ),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub trait ParseTest: Parser {
        fn new() -> Self;
        fn parse_test_args<S: ToString>(
            &self,
            args: &[(S, Option<S>)],
        ) -> Result<Self::Parsed, ParsingError>;
    }

    pub fn flags<P: ParseTest>() {
        let mut parser = P::new();
        let foo = parser.add_flag(&['f'], &["foo"]);
        let bar = parser.add_flag(&['b'], &["bar"]);

        let args = parser.parse_test_args::<&str>(&[]).unwrap();
        assert_eq!(args.get(&foo), None);
        assert_eq!(args.get(&bar), None);

        let args = parser.parse_test_args(&[("foo", None)]).unwrap();
        assert_eq!(args.get(&foo), Some(true).as_ref());
        assert_eq!(args.get(&bar), None);
        let args = parser.parse_test_args(&[("bar", None)]).unwrap();
        assert_eq!(args.get(&foo), None);
        assert_eq!(args.get(&bar), Some(true).as_ref());
        let args = parser
            .parse_test_args(&[("foo", None), ("bar", None)])
            .unwrap();
        assert_eq!(args.get(&foo), Some(true).as_ref());
        assert_eq!(args.get(&bar), Some(true).as_ref());

        let args = parser.parse_test_args(&[("f", None)]).unwrap();
        assert_eq!(args.get(&foo), Some(true).as_ref());
        assert_eq!(args.get(&bar), None);
        let args = parser.parse_test_args(&[("b", None)]).unwrap();
        assert_eq!(args.get(&foo), None);
        assert_eq!(args.get(&bar), Some(true).as_ref());
        let args = parser.parse_test_args(&[("f", None), ("b", None)]).unwrap();
        assert_eq!(args.get(&foo), Some(true).as_ref());
        assert_eq!(args.get(&bar), Some(true).as_ref());
    }

    pub fn flags_unknown<P: ParseTest>() {
        let mut parser = P::new();
        parser.add_flag(&['f'], &["foo"]);
        parser.add_flag(&['b'], &["bar"]);

        assert!(match parser.parse_test_args(&[("baz", None)]) {
            Err(ParsingError::UnknownOption { arg_name }) if arg_name == "baz".to_string() => true,
            _ => false,
        });
        assert!(match parser.parse_test_args(&[("x", None)]) {
            Err(ParsingError::UnknownOption { arg_name }) if arg_name == "x".to_string() => true,
            _ => false,
        });
    }

    pub fn options<P: ParseTest>() {
        let mut parser = P::new();
        let foo = parser.add_option_with::<_, _, _>(&['f'], &["foo"], |v| str::parse::<i32>(v));
        let bar = parser.add_option::<String, _>(&['b'], &["bar"]);

        let args = parser.parse_test_args::<&str>(&[]).unwrap();
        assert_eq!(args.get(&foo), None);
        assert_eq!(args.get(&bar), None);

        let args = parser.parse_test_args(&[("foo", Some("123"))]).unwrap();
        assert_eq!(args.get(&foo), Some(123).as_ref());
        assert_eq!(args.get(&bar), None);
        let args = parser.parse_test_args(&[("bar", Some("abc"))]).unwrap();
        assert_eq!(args.get(&foo), None);
        assert_eq!(args.get(&bar), Some("abc".to_string()).as_ref());
        let args = parser
            .parse_test_args(&[("foo", Some("123")), ("bar", Some("abc"))])
            .unwrap();
        assert_eq!(args.get(&foo), Some(123).as_ref());
        assert_eq!(args.get(&bar), Some("abc".to_string()).as_ref());

        let args = parser.parse_test_args(&[("f", Some("123"))]).unwrap();
        assert_eq!(args.get(&foo), Some(123).as_ref());
        assert_eq!(args.get(&bar), None);
        let args = parser.parse_test_args(&[("b", Some("abc"))]).unwrap();
        assert_eq!(args.get(&foo), None);
        assert_eq!(args.get(&bar), Some("abc".to_string()).as_ref());
        let args = parser
            .parse_test_args(&[("f", Some("123")), ("b", Some("abc"))])
            .unwrap();
        assert_eq!(args.get(&foo), Some(123).as_ref());
        assert_eq!(args.get(&bar), Some("abc".to_string()).as_ref());

        assert!(
            match parser.parse_test_args(&[("foo", Some("abc")), ("123", None)]) {
                Err(ParsingError::ValueParsingFailed { arg_name, .. })
                    if arg_name == "foo".to_string() =>
                    true,
                _ => false,
            }
        );
    }

    pub fn options_unknown<P: ParseTest>() {
        let mut parser = P::new();
        parser.add_option_with::<_, _, _>(&['f'], &["foo"], |v| str::parse::<i32>(v));
        parser.add_option::<String, _>(&['b'], &["bar"]);

        assert!(match parser.parse_test_args(&[("baz", Some("123"))]) {
            Err(ParsingError::UnknownOption { arg_name }) if arg_name == "baz".to_string() => true,
            _ => false,
        });
        assert!(match parser.parse_test_args(&[("x", Some("123"))]) {
            Err(ParsingError::UnknownOption { arg_name }) if arg_name == "x".to_string() => true,
            _ => false,
        });
    }

    pub fn options_missing_value<P: ParseTest>() {
        let mut parser = P::new();
        parser.add_option_with::<_, _, _>(&['f'], &["foo"], |v| str::parse::<i32>(v));

        assert!(match parser.parse_test_args(&[("foo", None)]) {
            Err(ParsingError::MissingValue { arg_name }) if arg_name == "foo".to_string() => true,
            _ => false,
        });
    }
}
