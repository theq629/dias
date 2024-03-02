use super::super::OuterDirectoryError;
use super::write::FileWrite;
use atomic_write_file::AtomicWriteFile;
use directories::ProjectDirs;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::fs::{create_dir_all, remove_file};
use std::marker::PhantomData;
use std::path::{Component, PathBuf};

#[derive(Debug)]
pub enum StandardStorageAvailabilityError {
    UnknownHomeDirectory,
}

impl fmt::Display for StandardStorageAvailabilityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for StandardStorageAvailabilityError {}

pub struct ReadOnly;
pub struct ReadWrite;

pub struct File<R> {
    _phantom: PhantomData<R>,
    path: PathBuf,
}

impl<R> File<R> {
    fn new(path: PathBuf) -> Self {
        Self {
            _phantom: PhantomData,
            path,
        }
    }
}

impl<R> super::super::File for File<R> {
    type ReadText = std::fs::File;
    type ReadBinary = std::fs::File;

    fn exists(&self) -> std::io::Result<bool> {
        Ok(self.path.exists())
    }

    fn read_text(&self) -> std::io::Result<Self::ReadText> {
        std::fs::File::open(&self.path)
    }

    fn read_binary(&self) -> std::io::Result<Self::ReadBinary> {
        self.read_text()
    }
}

impl super::super::WritableFile for File<ReadWrite> {
    type WriteText = FileWrite;
    type WriteBinary = FileWrite;

    fn remove(&mut self) -> std::io::Result<()> {
        remove_file(&self.path)
    }

    fn write_text(&mut self) -> std::io::Result<Self::WriteText> {
        if let Some(dir_path) = self.path.parent() {
            create_dir_all(&dir_path)?;
        }
        Ok(FileWrite::new(AtomicWriteFile::open(&self.path)?))
    }

    fn write_binary(&mut self) -> std::io::Result<Self::WriteText> {
        self.write_text()
    }
}

pub struct Dir<R> {
    _phantom: PhantomData<R>,
    path: PathBuf,
}

impl<R> Dir<R> {
    fn new(path: PathBuf) -> Self {
        Self {
            _phantom: PhantomData,
            path,
        }
    }

    fn new_outer(
        path: PathBuf,
        path_prefix: Option<&PathBuf>,
    ) -> Result<Self, OuterDirectoryError> {
        let mut new = Self::new(path);
        if let Some(path_prefix) = path_prefix {
            // This is so we can re-root paths for testing.
            let mut components: Vec<_> = path_prefix.components().collect();
            components.extend(new.path.components().filter(|c| c != &Component::RootDir));
            new.path = components.iter().collect();
        }
        create_dir_all(&new.path).map_err(|_| OuterDirectoryError::NotAvailable)?;
        if new.path.is_dir() {
            Ok(new)
        } else {
            Err(OuterDirectoryError::NotAvailable)
        }
    }
}

impl<R> super::super::Dir for Dir<R> {
    type File = File<R>;

    fn file(&self, name: Cow<'static, str>) -> Self::File {
        File::new(self.path.join(name.to_string()))
    }
}

impl super::super::WritableDir for Dir<ReadWrite> {
    type WritableFile = File<ReadWrite>;

    fn writable_file(&mut self, name: Cow<'static, str>) -> Self::WritableFile {
        File::new(self.path.join(name.to_string()))
    }
}

impl<R> super::super::ParentDir for Dir<R> {
    type LeafDir = Dir<R>;

    fn subdir(&self, name: Cow<'static, str>) -> Self {
        Self::new(self.path.join(name.to_string()))
    }

    fn into_leaf(self) -> Self::LeafDir {
        self
    }
}

impl super::super::WritableParentDir for Dir<ReadWrite> {
    type WritableLeafDir = Dir<ReadWrite>;

    fn writable_subdir(&mut self, name: Cow<'static, str>) -> Self {
        Self {
            _phantom: PhantomData,
            path: self.path.join(name.to_string()),
        }
    }

    fn into_writable_leaf(self) -> Self::WritableLeafDir {
        self
    }
}

pub struct Storage {
    path_prefix: Option<PathBuf>,
    project_dirs: ProjectDirs,
}

impl Storage {
    pub fn new(
        qualifier: &str,
        organization: &str,
        application: &str,
    ) -> Result<Self, StandardStorageAvailabilityError> {
        ProjectDirs::from(qualifier, organization, application)
            .map(|pd| Self {
                path_prefix: None,
                project_dirs: pd,
            })
            .ok_or(StandardStorageAvailabilityError::UnknownHomeDirectory)
    }
}

impl super::super::Storage for Storage {
    type Dir = Dir<ReadOnly>;
    type WritableDir = Dir<ReadWrite>;

    fn data(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Dir::new_outer(
            self.project_dirs.data_dir().into(),
            self.path_prefix.as_ref(),
        )
    }

    fn config(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Dir::new_outer(
            self.project_dirs.config_dir().into(),
            self.path_prefix.as_ref(),
        )
    }

    fn cache(&self) -> Result<Self::Dir, OuterDirectoryError> {
        Dir::new_outer(
            self.project_dirs.cache_dir().into(),
            self.path_prefix.as_ref(),
        )
    }

    fn writable_data(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Dir::new_outer(
            self.project_dirs.data_dir().into(),
            self.path_prefix.as_ref(),
        )
    }

    fn writable_config(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Dir::new_outer(
            self.project_dirs.config_dir().into(),
            self.path_prefix.as_ref(),
        )
    }

    fn writable_cache(&mut self) -> Result<Self::WritableDir, OuterDirectoryError> {
        Dir::new_outer(
            self.project_dirs.cache_dir().into(),
            self.path_prefix.as_ref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::generic::tests as generic_tests;
    use super::super::super::Storage as _;
    use super::*;
    use tempfile::tempdir;

    fn make_storage() -> Storage {
        let temp = tempdir().unwrap();
        let mut storage = Storage::new("a", "b", "c").unwrap();
        storage.path_prefix = Some(temp.path().into());
        storage
    }

    #[test]
    fn storage_uniqueness() {
        let storage0 = Storage::new("a", "b", "c").unwrap();
        let storage1 = Storage::new("d", "e", "f").unwrap();
        assert_ne!(storage0.data().unwrap().path, storage1.data().unwrap().path);
        assert_ne!(
            storage0.config().unwrap().path,
            storage1.config().unwrap().path
        );
        assert_ne!(
            storage0.cache().unwrap().path,
            storage1.cache().unwrap().path
        );
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
