use std::io::{Cursor, Read, Write};
use web_sys::Storage as WebStorage;

pub struct TextStorageReader {
    cursor: Cursor<String>,
}

impl TextStorageReader {
    pub fn new(web_storage: &WebStorage, key: &str) -> std::io::Result<Self> {
        let value = web_storage.get_item(key).unwrap_or(None);
        if let Some(value) = value {
            Ok(Self {
                cursor: Cursor::new(value),
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cannot find stored value",
            ))
        }
    }
}

impl Read for TextStorageReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}

pub struct TextStorageWriter {
    web_storage: WebStorage,
    key: String,
    buf: Vec<u8>,
}

impl TextStorageWriter {
    pub fn new(web_storage: &WebStorage, key: &str) -> std::io::Result<Self> {
        Ok(Self {
            web_storage: web_storage.clone(),
            key: key.to_string(),
            buf: Vec::new(),
        })
    }
}

impl Write for TextStorageWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let value = std::str::from_utf8(self.buf.as_slice())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "cannot convert utf8"))?
            .to_string();
        self.web_storage
            .set_item(&self.key, &value)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "cannot store value"))
    }
}

impl Drop for TextStorageWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
