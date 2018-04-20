use std::path::PathBuf;
use std::io::*;
use std::fs::*;
use std::sync::Arc;
use rusttype::SharedBytes;

pub trait Loadable<'a> {
    type Reader: BufRead+Seek;
    fn uid(&self) -> String;
    fn open(&self) -> Self::Reader;
    fn bytes(&self) -> SharedBytes<'a>;
}

//-----------------------------------------------------------//
// PathBuf

impl Loadable<'static> for PathBuf {
    type Reader = BufReader<File>;
    fn uid(&self) -> String {
        self.to_string_lossy().to_string()
    }
    fn open(&self) -> Self::Reader {
        BufReader::new(File::open(self).unwrap())
    }
    fn bytes(&self) -> SharedBytes<'static> {
        let mut data = Vec::new();
        File::open(self)
            .unwrap()
            .read_to_end(&mut data)
            .expect("unable to read file");
        Arc::<[u8]>::from(data).into()
    }
}

//-----------------------------------------------------------//
// LoadFromStaticMemory

#[derive(Clone)]
pub struct LoadFromStaticMemory<'a> {
    pub id: &'a str,
    pub memory: &'a [u8],
}

macro_rules! load_from_static_memory {
    ($path:expr) => (LoadFromStaticMemory{
        id: $path,
        memory: include_bytes!($path)
    })
}

impl<'a> Loadable<'a> for LoadFromStaticMemory<'a> {
    type Reader = BufReader<Cursor<&'a[u8]>>;
    fn uid(&self) -> String {
        self.id.to_string()
    }
    fn open(&self) -> Self::Reader {
        BufReader::new(Cursor::new(self.memory))
    }
    fn bytes(&self) -> SharedBytes<'a> {
        self.memory.into()
    }
}