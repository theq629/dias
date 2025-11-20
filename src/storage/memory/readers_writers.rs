use crate::storage::memory::file_sys::{MemoryFile, Shared};
use std::io::{Cursor, Read, Write};

pub struct StorageReader {
    cursor: Cursor<Vec<u8>>,
}

impl StorageReader {
    pub fn new(stored: Shared<MemoryFile>) -> std::io::Result<Self> {
        let read = stored
            .read()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(Self {
            cursor: Cursor::new(read.contents.clone()),
        })
    }
}

impl Read for StorageReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}

pub struct StorageWriter {
    stored: Shared<MemoryFile>,
    buf: Vec<u8>,
}

impl StorageWriter {
    pub fn new(stored: Shared<MemoryFile>) -> std::io::Result<Self> {
        Ok(Self {
            stored,
            buf: Vec::new(),
        })
    }
}

impl Write for StorageWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let contents = &mut self
            .stored
            .write()
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .contents;
        contents.clear();
        contents.extend(&self.buf);
        self.buf.clear();
        Ok(())
    }
}

impl Drop for StorageWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
