//! Support for boxing storage types. Awkward but useful for cases where it is easier to store a
//! box than use generic types.

use crate::storage::generic::{Dir, File, WritableDir, WritableFile};
use std::borrow::Cow;
use std::io::{Read, Write};

pub type BoxedFile = Box<dyn File<ReadText = Box<dyn Read>, ReadBinary = Box<dyn Read>>>;
pub type BoxedWritableFile = Box<
    dyn WritableFile<
        ReadText = Box<dyn Read>,
        ReadBinary = Box<dyn Read>,
        WriteText = Box<dyn Write>,
        WriteBinary = Box<dyn Write>,
    >,
>;
pub type BoxedDir = Box<dyn Dir<File = BoxedFile>>;
pub type BoxedWritableDir =
    Box<dyn WritableDir<File = BoxedFile, WritableFile = BoxedWritableFile>>;

impl<F> File for Box<F>
where
    F: File + ?Sized,
{
    type ReadText = F::ReadText;
    type ReadBinary = F::ReadBinary;

    fn exists(&self) -> std::io::Result<bool> {
        (**self).exists()
    }

    fn read_text(&self) -> std::io::Result<Self::ReadText> {
        (**self).read_text()
    }

    fn read_binary(&self) -> std::io::Result<Self::ReadBinary> {
        (**self).read_binary()
    }
}

impl<F> WritableFile for Box<F>
where
    F: WritableFile + ?Sized,
{
    type WriteText = F::WriteText;
    type WriteBinary = F::WriteBinary;

    fn remove(&mut self) -> std::io::Result<()> {
        (**self).remove()
    }

    fn write_text(&mut self) -> std::io::Result<Self::WriteText> {
        (**self).write_text()
    }

    fn write_binary(&mut self) -> std::io::Result<Self::WriteBinary> {
        (**self).write_binary()
    }
}

pub struct BoxableFile<F> {
    source: F,
}

impl<F> From<F> for BoxableFile<F> {
    fn from(source: F) -> Self {
        Self { source }
    }
}

impl<Rt, Rb, F> File for BoxableFile<F>
where
    Rt: 'static + Read,
    Rb: 'static + Read,
    F: File<ReadText = Rt, ReadBinary = Rb>,
{
    type ReadText = Box<dyn Read>;
    type ReadBinary = Box<dyn Read>;

    fn exists(&self) -> std::io::Result<bool> {
        self.source.exists()
    }

    fn read_text(&self) -> std::io::Result<Self::ReadText> {
        self.source
            .read_text()
            .map(|r| Box::new(r) as Box<dyn Read>)
    }

    fn read_binary(&self) -> std::io::Result<Self::ReadBinary> {
        self.source
            .read_binary()
            .map(|r| Box::new(r) as Box<dyn Read>)
    }
}

impl<Rt, Rb, Wt, Wb, F> WritableFile for BoxableFile<F>
where
    Rt: 'static + Read,
    Rb: 'static + Read,
    Wt: 'static + Write,
    Wb: 'static + Write,
    F: WritableFile<ReadText = Rt, ReadBinary = Rb, WriteText = Wt, WriteBinary = Wb>,
{
    type WriteText = Box<dyn Write>;
    type WriteBinary = Box<dyn Write>;

    fn remove(&mut self) -> std::io::Result<()> {
        self.source.remove()
    }

    fn write_text(&mut self) -> std::io::Result<Self::WriteText> {
        self.source
            .write_text()
            .map(|w| Box::new(w) as Box<dyn Write>)
    }

    fn write_binary(&mut self) -> std::io::Result<Self::WriteText> {
        self.source
            .write_binary()
            .map(|w| Box::new(w) as Box<dyn Write>)
    }
}

impl<D> Dir for Box<D>
where
    D: Dir,
{
    type File = D::File;

    fn file(&self, name: Cow<'static, str>) -> Self::File {
        (**self).file(name)
    }
}

impl<D> WritableDir for Box<D>
where
    D: WritableDir,
{
    type WritableFile = D::WritableFile;

    fn writable_file(&mut self, name: Cow<'static, str>) -> Self::WritableFile {
        (**self).writable_file(name)
    }
}

pub struct BoxableDir<D> {
    source: D,
}

impl<D> From<D> for BoxableDir<D> {
    fn from(source: D) -> Self {
        Self { source }
    }
}

impl<D> Dir for BoxableDir<D>
where
    D: 'static + Dir,
{
    type File = BoxedFile;

    fn file(&self, name: Cow<'static, str>) -> Self::File {
        Box::new(BoxableFile::from(self.source.file(name)))
    }
}

impl<D> WritableDir for BoxableDir<D>
where
    D: 'static + WritableDir,
{
    type WritableFile = BoxedWritableFile;

    fn writable_file(&mut self, name: Cow<'static, str>) -> Self::WritableFile {
        Box::new(BoxableFile::from(self.source.writable_file(name)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::storage::*;

    #[test]
    fn boxability() {
        // It is useful to be able to store files and directories, which for practical purposes
        // requires boxing. So test that boxing is possible and not difficult.
        fn _test<S: 'static + Storage>(mut storage: S) {
            let d0: BoxedDir = Box::new(BoxableDir::from(storage.data().unwrap()));
            let mut d1: BoxedWritableDir =
                Box::new(BoxableDir::from(storage.writable_data().unwrap()));
            let _: BoxedFile = Box::new(BoxableFile::from(
                storage.data().unwrap().file("test".into()),
            ));
            let _: BoxedWritableFile = Box::new(BoxableFile::from(
                storage
                    .writable_data()
                    .unwrap()
                    .writable_file("test".into()),
            ));
            let _: BoxedFile = d0.file("test".into());
            let _: BoxedWritableFile = d1.writable_file("test".into());
        }
    }
}
