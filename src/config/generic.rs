use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub trait ConfigStringHandler {
    fn to_string<T>(value: &T) -> std::io::Result<String>
    where
        T: Serialize + ?Sized;

    fn from_str<T>(string: &str) -> std::io::Result<T>
    where
        T: DeserializeOwned;
}

#[cfg(test)]
pub(super) mod tests {
    use super::super::{read_config, write_config};
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub enum TestEnum {
        One,
        Two,
        Three,
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct TestConfig {
        pub foo: i32,
        pub bar: String,
        pub baz: TestEnum,
    }

    pub fn basic<H: ConfigStringHandler>() {
        let config = TestConfig {
            foo: 12345,
            bar: "hello world".to_string(),
            baz: TestEnum::Two,
        };
        let mut buf = Cursor::new(Vec::new());
        write_config(&config, &mut buf).unwrap();
        buf.set_position(0);
        let in_config: TestConfig = read_config(&mut buf).unwrap();
        assert_eq!(in_config, config);
    }

    #[cfg(feature = "storage")]
    pub fn basic_storage<H: ConfigStringHandler>() {
        use crate::config::{read_config_file, write_config_file};
        use crate::storage::{MemoryStorage, Storage, WritableDir};
        let config = TestConfig {
            foo: 12345,
            bar: "hello world".to_string(),
            baz: TestEnum::Two,
        };
        let mut file = MemoryStorage::new()
            .writable_config()
            .unwrap()
            .writable_file("test".into());
        write_config_file(&config, &mut file).unwrap();
        let in_config: TestConfig = read_config_file(&file).unwrap();
        assert_eq!(in_config, config);
    }
}
