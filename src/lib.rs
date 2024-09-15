mod mphf;
use std::hash::Hash;

pub use mphf::bitvector::BitVector;
pub use wheatley_macro::*;

pub struct File<'a> {
    pub path: &'a [u8],
    pub contents: &'a [u8],
}

impl<'a> File<'a> {
    pub const fn new(path: &'a [u8], contents: &'a [u8]) -> Self {
        File { path, contents }
    }
}

pub enum Entry<'a> {
    File(File<'a>),
}

pub struct Wheatley<'a> {
    entries: &'a [Entry<'a>],
    mphf: mphf::bbhash::Mphf<'a>,
}

impl<'a> Wheatley<'a> {
    pub const fn new(entries: &'a [Entry], mphf_state: &'a [(BitVector<'a>, &'a [u64])]) -> Self {
        let mphf = mphf::bbhash::Mphf::new(mphf_state);

        Self { mphf, entries }
    }

    pub fn get<T: Hash + core::fmt::Debug>(&self, key: T) -> &File {
        let entry_position = self.mphf.hash(&key) as usize;
        let entry = &self.entries[entry_position];

        match entry {
            Entry::File(file) => file,
        }
    }
}
