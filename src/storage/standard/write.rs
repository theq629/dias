use atomic_write_file::AtomicWriteFile;
use std::io::{IoSlice, Write};

pub struct FileWrite {
    source: Option<AtomicWriteFile>,
}

impl FileWrite {
    pub(crate) fn new(source: AtomicWriteFile) -> Self {
        Self {
            source: Some(source),
        }
    }

    fn source(&self) -> &AtomicWriteFile {
        self.source
            .as_ref()
            .expect("should have underlying file until dropped")
    }
}

impl Write for FileWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.source().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.source().flush()
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        self.source().write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.source().write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.source().write_fmt(fmt)
    }
}

impl Drop for FileWrite {
    fn drop(&mut self) {
        // AtomicWriteFile doesn't seem to commit on drop as expected (at least in test code), so
        // we call it explicitly in this wrapper.
        if let Some(source) = self.source.take() {
            let _ = source.commit();
        }
    }
}
