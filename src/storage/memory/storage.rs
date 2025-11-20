use super::super::OuterDirectoryError;
use super::readers_writers::{StorageReader, StorageWriter};
use crate::storage::memory::file_sys::{FileSystem, Shared};
use std::borrow::Cow;
use std::marker::PhantomData;

static SEP: char = '/';

pub struct ReadOnly;
pub struct ReadWrite;

pub struct File<R> {
    _phantom: PhantomData<R>,
    fs: Shared<FileSystem>,
    path: String,
}

impl<R> File<R> {
    fn new(parent_path: String, name: Cow<'static, str>, fs: Shared<FileSystem>) -> Self {
        let mut path = parent_path;
        path.push_str(name.as_ref());
        Self {
            _phantom: PhantomData,
            fs,
            path,
        }
    }
}

impl<R> super::super::File for File<R> {
    type ReadText = StorageReader;
    type ReadBinary = StorageReader;

    fn exists(&self) -> std::io::Result<bool> {
        Ok(self
            .fs
            .read()
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .exists(&self.path))
    }

    fn read_text(&self) -> std::io::Result<Self::ReadText> {
        StorageReader::new(self.fs.read()?.get(&self.path)?)
    }

    fn read_binary(&self) -> std::io::Result<Self::ReadBinary> {
        StorageReader::new(self.fs.read()?.get(&self.path)?)
    }
}

impl super::super::WritableFile for File<ReadWrite> {
    type WriteText = StorageWriter;
    type WriteBinary = StorageWriter;

    fn remove(&mut self) -> std::io::Result<()> {
        self.fs.write()?.remove(&self.path)
    }

    fn write_text(&mut self) -> std::io::Result<Self::WriteText> {
        StorageWriter::new(self.fs.write()?.get_or_create(&self.path)?)
    }

    fn write_binary(&mut self) -> std::io::Result<Self::WriteBinary> {
        StorageWriter::new(self.fs.write()?.get_or_create(&self.path)?)
    }
}

pub struct Dir<R> {
    _phantom: PhantomData<R>,
    fs: Shared<FileSystem>,
    path: String,
}

impl<R> Dir<R> {
    fn new(parent_path: String, name: Cow<'static, str>, fs: Shared<FileSystem>) -> Self {
        let mut path = parent_path;
        path.push_str(name.as_ref());
        path.push(SEP);
        Dir {
            _phantom: PhantomData,
            fs,
            path,
        }
    }
}

impl<R> super::super::Dir for Dir<R> {
    type File = File<R>;

    fn file(&self, name: Cow<'static, str>) -> Self::File {
        File::new(self.path.clone(), name, self.fs.clone())
    }
}

impl<R> super::super::ParentDir for Dir<R> {
    type LeafDir = Dir<R>;

    fn subdir(&self, name: Cow<'static, str>) -> Self {
        Self::new(self.path.clone(), name, self.fs.clone())
    }

    fn into_leaf(self) -> Self::LeafDir {
        self
    }
}

impl super::super::WritableDir for Dir<ReadWrite> {
    type WritableFile = File<ReadWrite>;

    fn writable_file(&mut self, name: Cow<'static, str>) -> Self::WritableFile {
        File::new(self.path.clone(), name, self.fs.clone())
    }
}

impl super::super::WritableParentDir for Dir<ReadWrite> {
    type WritableLeafDir = Dir<ReadWrite>;

    fn writable_subdir(&mut self, name: Cow<'static, str>) -> Self {
        Self::new(self.path.clone(), name, self.fs.clone())
    }

    fn into_writable_leaf(self) -> Self::WritableLeafDir {
        self
    }
}

/// Basic implementation of storage in memory, mostly for writing tests against storage.
pub struct MemoryStorage {
    fs: Shared<FileSystem>,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            fs: Shared::new(FileSystem::new()),
        }
    }
}

impl super::super::Storage for MemoryStorage {
    type Dir = Dir<ReadOnly>;
    type WritableDir = Dir<ReadWrite>;

    fn data(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Ok(Dir::new("".to_string(), "data".into(), self.fs.clone()))
    }

    fn config(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Ok(Dir::new("".to_string(), "config".into(), self.fs.clone()))
    }

    fn cache(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Ok(Dir::new("".to_string(), "cache".into(), self.fs.clone()))
    }

    fn writable_data(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Ok(Dir::new("".to_string(), "data".into(), self.fs.clone()))
    }

    fn writable_config(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Ok(Dir::new("".to_string(), "config".into(), self.fs.clone()))
    }

    fn writable_cache(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Ok(Dir::new("".to_string(), "cache".into(), self.fs.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::generic::tests as generic_tests;
    use super::*;

    fn make_storage() -> MemoryStorage {
        MemoryStorage::new()
    }

    #[test]
    fn text_file() {
        generic_tests::text_file(make_storage());
    }

    #[test]
    fn binary_file() {
        generic_tests::binary_file(make_storage());
    }

    #[test]
    fn file_uniqueness() {
        generic_tests::file_uniqueness(make_storage());
    }
}
