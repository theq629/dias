use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};

#[derive(Debug)]
pub enum OuterDirectoryError {
    NotAvailable,
}

impl fmt::Display for OuterDirectoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for OuterDirectoryError {}

pub trait File {
    type ReadText: Read;
    type ReadBinary: Read;
    fn exists(&self) -> std::io::Result<bool>;
    fn read_text(&self) -> std::io::Result<Self::ReadText>;
    fn read_binary(&self) -> std::io::Result<Self::ReadBinary>;
}

pub trait WritableFile: File {
    type WriteText: Write;
    type WriteBinary: Write;

    fn remove(&mut self) -> std::io::Result<()>;
    fn write_text(&mut self) -> std::io::Result<Self::WriteText>;
    fn write_binary(&mut self) -> std::io::Result<Self::WriteBinary>;
}

pub trait Dir {
    type File: File;
    fn file(&self, name: Cow<'static, str>) -> Self::File;
}

pub trait WritableDir: Dir {
    type WritableFile: WritableFile;
    fn writable_file(&mut self, name: Cow<'static, str>) -> Self::WritableFile;
}

pub trait ParentDir: Dir {
    type LeafDir: Dir;
    fn subdir(&self, name: Cow<'static, str>) -> Self;
    fn into_leaf(self) -> Self::LeafDir;
}

pub trait WritableParentDir: WritableDir + ParentDir {
    type WritableLeafDir: WritableDir;
    fn writable_subdir(&mut self, name: Cow<'static, str>) -> Self;
    fn into_writable_leaf(self) -> Self::WritableLeafDir;
}

pub trait Storage {
    type Dir: ParentDir;
    type WritableDir: WritableParentDir;
    fn data(&self) -> Result<Self::Dir, OuterDirectoryError>;
    fn config(&self) -> Result<Self::Dir, OuterDirectoryError>;
    fn cache(&self) -> Result<Self::Dir, OuterDirectoryError>;
    fn writable_data(&mut self) -> Result<Self::WritableDir, OuterDirectoryError>;
    fn writable_config(&mut self) -> Result<Self::WritableDir, OuterDirectoryError>;
    fn writable_cache(&mut self) -> Result<Self::WritableDir, OuterDirectoryError>;
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use std::io::{Read, Write};

    pub fn text_file(mut storage: impl Storage) {
        let text = "hello world";

        let mut file = storage
            .writable_data()
            .unwrap()
            .writable_file("test".into());
        file.write_text()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
        assert!(file.exists().unwrap());
        let mut got = String::new();
        file.read_text().unwrap().read_to_string(&mut got).unwrap();
        assert_eq!(got, text);

        let file = storage.data().unwrap().file("test".into());
        assert!(file.exists().unwrap());
        let mut got = String::new();
        file.read_text().unwrap().read_to_string(&mut got).unwrap();
        assert_eq!(got, text);

        let mut file = storage
            .writable_data()
            .unwrap()
            .writable_file("test".into());
        assert!(file.exists().unwrap());
        file.remove().unwrap();
        assert!(!file.exists().unwrap());
        let file = storage.data().unwrap().file("test".into());
        assert!(!file.exists().unwrap());
    }

    pub fn binary_file(mut storage: impl Storage) {
        let text = "hello world";

        let mut file = storage
            .writable_data()
            .unwrap()
            .writable_file("test".into());
        file.write_text()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
        assert!(file.exists().unwrap());
        let mut got = String::new();
        file.read_text().unwrap().read_to_string(&mut got).unwrap();
        assert_eq!(got, text);

        let file = storage.data().unwrap().file("test".into());
        assert!(file.exists().unwrap());
        let mut got = String::new();
        file.read_text().unwrap().read_to_string(&mut got).unwrap();
        assert_eq!(got, text);

        let mut file = storage
            .writable_data()
            .unwrap()
            .writable_file("test".into());
        assert!(file.exists().unwrap());
        file.remove().unwrap();
        assert!(!file.exists().unwrap());
        let file = storage.data().unwrap().file("test".into());
        assert!(!file.exists().unwrap());
    }

    pub fn file_uniqueness(mut storage: impl Storage) {
        let to_check = vec![
            (storage.writable_data().unwrap(), "data", "one", "a"),
            (storage.writable_data().unwrap(), "data", "one", "b"),
            (storage.writable_data().unwrap(), "data", "two", "a"),
            (storage.writable_data().unwrap(), "data", "two", "b"),
            (storage.writable_config().unwrap(), "config", "one", "a"),
            (storage.writable_config().unwrap(), "config", "one", "b"),
            (storage.writable_config().unwrap(), "config", "two", "a"),
            (storage.writable_config().unwrap(), "config", "two", "b"),
            (storage.writable_cache().unwrap(), "cache", "one", "a"),
            (storage.writable_cache().unwrap(), "cache", "one", "b"),
            (storage.writable_cache().unwrap(), "cache", "two", "a"),
            (storage.writable_cache().unwrap(), "cache", "two", "b"),
        ];

        for (outer_dir, outer_dir_name, inner_dir_name, file_name) in to_check.iter() {
            let value = format!(
                "contents {} {} {}",
                outer_dir_name, inner_dir_name, file_name
            );
            outer_dir
                .subdir((*inner_dir_name).into())
                .writable_file((*file_name).into())
                .write_text()
                .unwrap()
                .write_all(value.as_bytes())
                .unwrap();
        }

        for (outer_dir, outer_dir_name, inner_dir_name, file_name) in to_check.iter() {
            let value = format!(
                "contents {} {} {}",
                outer_dir_name, inner_dir_name, file_name
            );
            let mut got = String::new();
            outer_dir
                .subdir((*inner_dir_name).into())
                .file((*file_name).into())
                .read_text()
                .unwrap()
                .read_to_string(&mut got)
                .unwrap();
            assert_eq!(got, value);
        }
    }
}
