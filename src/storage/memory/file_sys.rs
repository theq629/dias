use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct Shared<T> {
    value: Arc<RwLock<T>>,
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
        }
    }

    pub fn read(&self) -> std::io::Result<RwLockReadGuard<'_, T>> {
        self.value
            .read()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    pub fn write(&self) -> std::io::Result<RwLockWriteGuard<'_, T>> {
        self.value
            .write()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

pub struct MemoryFile {
    pub contents: Vec<u8>,
}

impl MemoryFile {
    fn new() -> Self {
        Self {
            contents: Vec::new(),
        }
    }
}

pub struct FileSystem {
    contents: HashMap<String, Shared<MemoryFile>>,
}

impl FileSystem {
    pub fn new() -> Self {
        Self {
            contents: HashMap::new(),
        }
    }

    pub fn exists(&self, path: &String) -> bool {
        self.contents.contains_key(path)
    }

    pub fn get(&self, path: &String) -> std::io::Result<Shared<MemoryFile>> {
        self.contents
            .get(path)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, ""))
            .cloned()
    }

    pub fn get_or_create(&mut self, path: &String) -> std::io::Result<Shared<MemoryFile>> {
        Ok(self
            .contents
            .entry(path.clone())
            .or_insert_with(|| Shared::new(MemoryFile::new()))
            .clone())
    }

    pub fn remove(&mut self, path: &String) -> std::io::Result<()> {
        self.contents.remove(path);
        Ok(())
    }
}
