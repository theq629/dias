use super::ParsingError;
use crate::cmd_line::shared::ArgId;
use core::str::FromStr;
use std::any::Any;
use std::error::Error;
use web_sys::UrlSearchParams;

trait ArgHandler {
    fn get(&self, url_params: &UrlSearchParams) -> Result<Option<Box<dyn Any>>, ParsingError>;
}

struct FlagArgHandler {
    short: &'static [char],
    long: &'static [&'static str],
}

impl ArgHandler for FlagArgHandler {
    fn get(&self, url_params: &UrlSearchParams) -> Result<Option<Box<dyn Any>>, ParsingError> {
        let value = self
            .short
            .iter()
            .any(|n| url_params.has(n.to_string().as_str()))
            || self.long.iter().any(|n| url_params.has(n));
        if value {
            Ok(Some(Box::new(true)))
        } else {
            Ok(None)
        }
    }
}

struct OptionArgHandler<F> {
    short: &'static [char],
    long: &'static [&'static str],
    parse: F,
}

impl<T, E, F> OptionArgHandler<F>
where
    T: 'static,
    E: 'static + Into<Box<dyn Error>>,
    F: 'static + Fn(&str) -> Result<T, E>,
{
    fn get_name(
        &self,
        name: &str,
        url_params: &UrlSearchParams,
    ) -> Result<Option<Box<dyn Any>>, ParsingError> {
        // Note that get() seems to return an empty string if there is no value for the argument.
        let value = url_params
            .get(name)
            .map(|v| (self.parse)(v.as_ref()))
            .transpose()
            .map_err(|e| ParsingError::ValueParsingFailed {
                arg_name: name.to_string(),
                error: e.into(),
            })?
            .map(|v| {
                let boxed: Box<dyn Any> = Box::new(v);
                boxed
            });
        Ok(value)
    }
}

impl<T, E, F> ArgHandler for OptionArgHandler<F>
where
    T: 'static,
    E: 'static + Into<Box<dyn Error>>,
    F: 'static + Fn(&str) -> Result<T, E>,
{
    fn get(&self, url_params: &UrlSearchParams) -> Result<Option<Box<dyn Any>>, ParsingError> {
        for name in self.long.iter() {
            if let Some(value) = self.get_name(name, url_params)? {
                return Ok(Some(value));
            }
        }
        for name in self.short.iter() {
            if let Some(value) = self.get_name(name.to_string().as_ref(), url_params)? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
}

pub struct Parser {
    args: Vec<Box<dyn ArgHandler>>,
}

impl Parser {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    #[cfg(test)]
    fn parse_string(&self, args: &str) -> Result<Parsed, ParsingError> {
        self.parse_url_params(
            &UrlSearchParams::new_with_str(args).map_err(|_| ParsingError::ParsingFailed)?,
        )
    }

    fn parse_url_params(&self, url_params: &UrlSearchParams) -> Result<Parsed, ParsingError> {
        let values = self
            .args
            .iter()
            .map(|handler| handler.get(url_params))
            .collect::<Result<Vec<_>, _>>()?;
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
        let params = web_sys::window()
            .ok_or(ParsingError::ParsingFailed)?
            .location()
            .search()
            .and_then(|s| UrlSearchParams::new_with_str(&s))
            .map_err(|_| ParsingError::ParsingFailed)?;
        self.parse_url_params(&params)
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
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    impl generic_tests::ParseTest for Parser {
        fn new() -> Self {
            Parser::new()
        }

        fn parse_test_args<S: ToString>(
            &self,
            args: &[(S, Option<S>)],
        ) -> Result<Self::Parsed, ParsingError> {
            let flat_args: Vec<_> = args
                .iter()
                .map(|arg| match arg {
                    (arg, None) => arg.to_string(),
                    (arg, Some(value)) => format!("{}={}", arg.to_string(), value.to_string()),
                })
                .collect();
            self.parse_string(&flat_args.join("&"))
        }
    }

    #[wasm_bindgen_test]
    fn flags() {
        generic_tests::flags::<Parser>();
    }

    #[wasm_bindgen_test]
    fn options() {
        generic_tests::options::<Parser>();
    }
}
