use super::super::OuterDirectoryError;
use super::binary_values::{BinaryStorageReader, BinaryStorageWriter};
use super::text_values::{TextStorageReader, TextStorageWriter};
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use web_sys::Storage as WebStorage;

static SEP: char = '/';

#[derive(Debug)]
pub enum WebStorageAvailabilityError {
    NoWindow,
    NoLocalStorage,
}

impl fmt::Display for WebStorageAvailabilityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WebStorageAvailabilityError {}

fn exists(web_storage: &WebStorage, path: &str) -> std::io::Result<bool> {
    Ok(web_storage
        .get_item(path)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "cannot get value"))?
        .is_some())
}

fn remove(web_storage: &WebStorage, path: &str) -> std::io::Result<()> {
    web_storage
        .remove_item(path)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "cannot remove value"))
}

pub struct ReadOnly;
pub struct ReadWrite;

pub struct File<R> {
    _phantom: PhantomData<R>,
    web_storage: WebStorage,
    path: String,
}

impl<R> File<R> {
    fn new(parent_path: String, name: Cow<'static, str>, web_storage: WebStorage) -> Self {
        let mut path = parent_path;
        path.push_str(name.as_ref());
        Self {
            _phantom: PhantomData,
            path,
            web_storage,
        }
    }
}

impl<R> super::super::File for File<R> {
    type ReadText = TextStorageReader;
    type ReadBinary = BinaryStorageReader;

    fn exists(&self) -> std::io::Result<bool> {
        exists(&self.web_storage, &self.path)
    }

    fn read_text(&self) -> std::io::Result<Self::ReadText> {
        TextStorageReader::new(&self.web_storage, &self.path)
    }

    fn read_binary(&self) -> std::io::Result<Self::ReadBinary> {
        BinaryStorageReader::new(&self.web_storage, &self.path)
    }
}

impl super::super::WritableFile for File<ReadWrite> {
    type WriteText = TextStorageWriter;
    type WriteBinary = BinaryStorageWriter;

    fn remove(&mut self) -> std::io::Result<()> {
        remove(&self.web_storage, &self.path)
    }

    fn write_text(&mut self) -> std::io::Result<Self::WriteText> {
        TextStorageWriter::new(&self.web_storage, &self.path)
    }

    fn write_binary(&mut self) -> std::io::Result<Self::WriteBinary> {
        BinaryStorageWriter::new(&self.web_storage, &self.path)
    }
}

pub struct Dir<R> {
    _phantom: PhantomData<R>,
    web_storage: WebStorage,
    path: String,
}

impl<R> Dir<R> {
    fn new(parent_path: String, name: Cow<'static, str>, web_storage: WebStorage) -> Self {
        let mut path = parent_path;
        path.push_str(name.as_ref());
        path.push(SEP);
        Dir {
            _phantom: PhantomData,
            path,
            web_storage,
        }
    }
}

impl<R> super::super::Dir for Dir<R> {
    type File = File<R>;

    fn file(&self, name: Cow<'static, str>) -> Self::File {
        File::new(self.path.clone(), name, self.web_storage.clone())
    }
}

impl<R> super::super::ParentDir for Dir<R> {
    type LeafDir = Dir<R>;

    fn subdir(&self, name: Cow<'static, str>) -> Self {
        Self::new(self.path.clone(), name, self.web_storage.clone())
    }

    fn into_leaf(self) -> Self::LeafDir {
        self
    }
}

impl super::super::WritableDir for Dir<ReadWrite> {
    type WritableFile = File<ReadWrite>;

    fn writable_file(&mut self, name: Cow<'static, str>) -> Self::WritableFile {
        File::new(self.path.clone(), name, self.web_storage.clone())
    }
}

impl super::super::WritableParentDir for Dir<ReadWrite> {
    type WritableLeafDir = Dir<ReadWrite>;

    fn writable_subdir(&mut self, name: Cow<'static, str>) -> Self {
        Self::new(self.path.clone(), name, self.web_storage.clone())
    }

    fn into_writable_leaf(self) -> Self::WritableLeafDir {
        self
    }
}

pub struct Storage {
    web_storage: WebStorage,
}

impl Storage {
    pub fn new() -> Result<Self, WebStorageAvailabilityError> {
        Ok(Self {
            web_storage: web_sys::window()
                .ok_or(WebStorageAvailabilityError::NoWindow)?
                .local_storage()
                .map_err(|_| WebStorageAvailabilityError::NoLocalStorage)?
                .ok_or(WebStorageAvailabilityError::NoLocalStorage)?,
        })
    }
}

impl super::super::Storage for Storage {
    type Dir = Dir<ReadOnly>;
    type WritableDir = Dir<ReadWrite>;

    fn data(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Ok(Dir::new(
            "".to_string(),
            "data".into(),
            self.web_storage.clone(),
        ))
    }

    fn config(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Ok(Dir::new(
            "".to_string(),
            "config".into(),
            self.web_storage.clone(),
        ))
    }

    fn cache(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Ok(Dir::new(
            "".to_string(),
            "cache".into(),
            self.web_storage.clone(),
        ))
    }

    fn writable_data(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Ok(Dir::new(
            "".to_string(),
            "data".into(),
            self.web_storage.clone(),
        ))
    }

    fn writable_config(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Ok(Dir::new(
            "".to_string(),
            "config".into(),
            self.web_storage.clone(),
        ))
    }

    fn writable_cache(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Ok(Dir::new(
            "".to_string(),
            "cache".into(),
            self.web_storage.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::generic::tests as generic_tests;
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn make_storage() -> Storage {
        Storage::new().unwrap()
    }

    #[wasm_bindgen_test]
    fn text_file() {
        generic_tests::text_file(make_storage());
    }

    #[wasm_bindgen_test]
    fn binary_file() {
        generic_tests::binary_file(make_storage());
    }

    #[wasm_bindgen_test]
    fn file_uniqueness() {
        generic_tests::file_uniqueness(make_storage());
    }
}
