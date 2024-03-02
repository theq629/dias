use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub struct ConfigStringHandler;

impl super::generic::ConfigStringHandler for ConfigStringHandler {
    fn to_string<T>(value: &T) -> std::io::Result<String>
    where
        T: Serialize + ?Sized,
    {
        toml::to_string(value).map_err(std::io::Error::other)
    }

    fn from_str<T>(string: &str) -> std::io::Result<T>
    where
        T: DeserializeOwned,
    {
        toml::from_str(string).map_err(std::io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::super::generic::tests as generic_tests;
    use super::super::write_config;
    use super::*;
    use std::io::Cursor;

    #[test]
    fn format() {
        let config = generic_tests::TestConfig {
            foo: 12345,
            bar: "hello world".to_string(),
            baz: generic_tests::TestEnum::Two,
        };
        let mut buf = Cursor::new(Vec::new());
        write_config(&config, &mut buf).unwrap();
        let buf = buf.into_inner();
        let text = std::str::from_utf8(&buf).unwrap();
        assert_eq!(
            text.trim(),
            "foo = 12345\nbar = \"hello world\"\nbaz = \"Two\"",
        );
    }

    #[test]
    fn basic() {
        generic_tests::basic::<ConfigStringHandler>();
    }

    #[cfg(feature = "storage")]
    #[test]
    fn basic_storage() {
        generic_tests::basic_storage::<ConfigStringHandler>();
    }
}
