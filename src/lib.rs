mod mphf;
use std::hash::Hash;

pub use mphf::bitvector::BitVector;
pub use wheatley_macro::*;

pub struct File<'a> {
    pub path: &'a [u8],
    pub contents: &'a [u8],
}

impl std::fmt::Debug for File<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = String::from_utf8_lossy(self.path);
        let content = String::from_utf8(self.contents.to_vec())
            .map(|content| content.chars().take(30).collect::<String>() + "...")
            .unwrap_or_else(|_| String::from("---snipped non-utf8 content---"));

        f.debug_struct("File")
            .field("path", &path)
            .field("content", &content)
            .finish()
    }
}

impl<'a> File<'a> {
    pub const fn new(path: &'a [u8], contents: &'a [u8]) -> Self {
        File { path, contents }
    }
}

#[derive(Debug)]
pub enum Entry<'a> {
    File(File<'a>),
}

pub struct Wheatley<'a> {
    entries: &'a [Entry<'a>],
    mphf: mphf::bbhash::Mphf<'a>,
}

impl std::fmt::Debug for Wheatley<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("entries", &self.entries)
            .finish()
    }
}

impl<'a> Wheatley<'a> {
    pub const fn new(entries: &'a [Entry], mphf_state: &'a [(BitVector<'a>, &'a [u64])]) -> Self {
        let mphf = mphf::bbhash::Mphf::new(mphf_state);

        Self { mphf, entries }
    }

    pub fn get<T: Hash + core::fmt::Debug>(&self, key: T) -> Option<&File> {
        println!("we should be here");
        println!("{:#?}", self.entries);
        println!("Right after printing");
        let entry = self
            .mphf
            .hash(&key)
            .inspect(|s| println!("Got milk: {s:#?}"))
            .map(|position| &self.entries[position as usize])?;

        println!("why not here?");

        println!("What we got");

        match entry {
            Entry::File(file) => Some(file),
        }
    }
}
