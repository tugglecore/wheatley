pub mod bbhash {
    // This code was copied then modified from rust-boomphf to meet the
    // needs of this project.
    // Below is a link to the project and the license.

    // https://github.com/10XGenomics/rust-boomphf/tree/master

    //The MIT License (MIT)

    // Copyright (c) 2014-2017 10x Genomics, Inc.

    // Permission is hereby granted, free of charge, to any person obtaining a copy
    // of this software and associated documentation files (the "Software"), to deal
    // in the Software without restriction, including without limitation the rights
    // to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    // copies of the Software, and to permit persons to whom the Software is
    // furnished to do so, subject to the following conditions:
    //
    // The above copyright notice and this permission notice shall be included in
    // all copies or substantial portions of the Software.
    //
    // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    // THE SOFTWARE.

    use crate::mphf::bitvector::BitVector;

    use std::fmt::Debug;
    use std::hash::Hash;
    use std::hash::Hasher;

    fn fold(v: u64) -> u32 {
        ((v & 0xFFFFFFFF) as u32) ^ ((v >> 32) as u32)
    }

    fn hash_with_seed<T: Hash + ?Sized>(iter: u64, v: &T) -> u64 {
        let mut state = wyhash::WyHash::with_seed(1 << (iter + iter));
        v.hash(&mut state);
        state.finish()
    }

    fn hash_with_seed32<T: Hash + ?Sized>(iter: u64, v: &T) -> u32 {
        fold(hash_with_seed(iter, v))
    }

    fn fastmod(hash: u32, n: u32) -> u64 {
        ((hash as u64) * (n as u64)) >> 32
    }

    fn hashmod<T: Hash + ?Sized>(iter: u64, v: &T, n: u64) -> u64 {
        // when n < 2^32, use the fast alternative to modulo described here:
        // https://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
        if n < (1 << 32) {
            let h = hash_with_seed32(iter, v);
            fastmod(h, n as u32) as u64
        } else {
            let h = hash_with_seed(iter, v);
            h % (n as u64)
        }
    }

    /// A minimal perfect hash function over a set of objects of type `T`.
    #[derive(Clone, Debug)]
    pub struct Mphf<'a> {
        // bitvecs: Box<[(BitVector<'a>, Box<[u64]>)]>,
        // phantom: PhantomData<T>,
        bit_vectors_with_ranks: &'a [(BitVector<'a>, &'a [u64])],
    }

    impl<'a> Mphf<'a> {
        pub const fn new(bit_vectors_with_ranks: &'a [(BitVector<'a>, &'a [u64])]) -> Mphf {
            // Mphf { bitvecs: Box::new([]), phantom: PhantomData }
            Mphf {
                bit_vectors_with_ranks,
            }
        }

        fn get_rank(&self, hash: u64, i: usize) -> u64 {
            let idx = hash as usize;
            let (bv, ranks) = self
                .bit_vectors_with_ranks
                .get(i)
                .expect("that level doesn't exist");

            // Last pre-computed rank
            let mut rank = ranks[idx / 512];

            // Add rank of intervening words
            for j in (idx / 64) & !7..idx / 64 {
                rank += bv.get_word(j).count_ones() as u64;
            }

            // Add rank of final word up to hash
            let final_word = bv.get_word(idx / 64);
            if idx % 64 > 0 {
                rank += (final_word << (64 - (idx % 64))).count_ones() as u64;
            }
            rank
        }

        /// Compute the hash value of `item`. This method should only be used
        /// with items known to be in construction set. Use `try_hash` if you cannot
        /// guarantee that `item` was in the construction set. If `item` was not present
        /// in the construction set this function may panic.
        pub fn hash<T: Hash + Debug>(&self, item: &T) -> u64 {
            for i in 0..self.bit_vectors_with_ranks.len() {
                let (bv, _) = &self.bit_vectors_with_ranks[i];
                let hash = hashmod(i as u64, item, bv.capacity() as u64);

                if bv.contains(hash) {
                    return self.get_rank(hash, i);
                }
            }

            unreachable!("must find a hash value");
        }
    }

    // struct Context<'a> {
    //     size: u64,
    //     seed: u64,
    //     a: BitVector<'a>,
    //     collide: BitVector<'a>,
    // }
    //
    // impl<'a> Context<'a> {
    //     fn new(size: u64, seed: u64) -> Self {
    //         Self {
    //             size: size as u64,
    //             seed,
    //             a: BitVector::new(size),
    //             collide: BitVector::new(size),
    //         }
    //     }
    //
    // }

    // #[cfg(test)]
    // mod tests {
    //
    //     use super::*;
    //     use std::collections::HashSet;
    //     use std::iter::FromIterator;
    //
    //     /// Check that a Minimal perfect hash function (MPHF) is generated for the set xs
    //     fn check_mphf<T>(xs: HashSet<T>) -> bool
    //     where
    //         T: Sync + Hash + PartialEq + Eq + Debug + Send,
    //     {
    //         let xsv: Vec<T> = xs.into_iter().collect();
    //
    //         // test single-shot data input
    //         check_mphf_serial(&xsv) && check_mphf_parallel(&xsv)
    //     }
    //
    //     /// Check that a Minimal perfect hash function (MPHF) is generated for the set xs
    //     fn check_mphf_serial<T>(xsv: &[T]) -> bool
    //     where
    //         T: Hash + PartialEq + Eq + Debug,
    //     {
    //         // Generate the MPHF
    //         let phf = Mphf::new(1.7, xsv);
    //
    //         // Hash all the elements of xs
    //         let mut hashes: Vec<u64> = xsv.iter().map(|v| phf.hash(v)).collect();
    //
    //         hashes.sort_unstable();
    //
    //         // Hashes must equal 0 .. n
    //         let gt: Vec<u64> = (0..xsv.len() as u64).collect();
    //         hashes == gt
    //     }
    //
    //
    //     fn check_chunked_mphf<T>(values: Vec<Vec<T>>, total: u64) -> bool
    //     where
    //         T: Sync + Hash + PartialEq + Eq + Debug + Send,
    //     {
    //         let phf = Mphf::from_chunked_iterator(1.7, &values, total);
    //
    //         // Hash all the elements of xs
    //         let mut hashes: Vec<u64> = values
    //             .iter()
    //             .flat_map(|x| x.iter().map(|v| phf.hash(&v)))
    //             .collect();
    //
    //         hashes.sort_unstable();
    //
    //         // Hashes must equal 0 .. n
    //         let gt: Vec<u64> = (0..total as u64).collect();
    //         hashes == gt
    //     }
    //
    //     quickcheck! {
    //         fn check_int_slices(v: HashSet<u64>, lens: Vec<usize>) -> bool {
    //
    //             let mut lens = lens;
    //
    //             let items: Vec<u64> = v.iter().cloned().collect();
    //             if lens.is_empty() || lens.iter().all(|x| *x == 0) {
    //                 lens.clear();
    //                 lens.push(items.len())
    //             }
    //
    //             let mut slices: Vec<Vec<u64>> = Vec::new();
    //
    //             let mut total = 0_usize;
    //             for slc_len in lens {
    //                 let end = std::cmp::min(items.len(), total.saturating_add(slc_len));
    //                 let slc = Vec::from(&items[total..end]);
    //                 slices.push(slc);
    //                 total = end;
    //
    //                 if total == items.len() {
    //                     break;
    //                 }
    //             }
    //
    //             check_chunked_mphf(slices.clone(), total as u64) && check_chunked_mphf_parallel(slices, total as u64)
    //         }
    //     }
    //
    //     quickcheck! {
    //         fn check_string(v: HashSet<Vec<String>>) -> bool {
    //             check_mphf(v)
    //         }
    //     }
    //
    //     quickcheck! {
    //         fn check_u32(v: HashSet<u32>) -> bool {
    //             check_mphf(v)
    //         }
    //     }
    //
    //     quickcheck! {
    //         fn check_isize(v: HashSet<isize>) -> bool {
    //             check_mphf(v)
    //         }
    //     }
    //
    //     quickcheck! {
    //         fn check_u64(v: HashSet<u64>) -> bool {
    //             check_mphf(v)
    //         }
    //     }
    //
    //     quickcheck! {
    //         fn check_vec_u8(v: HashSet<Vec<u8>>) -> bool {
    //             check_mphf(v)
    //         }
    //     }
    //
    //     #[test]
    //     fn from_ints_serial() {
    //         let items = (0..1000000).map(|x| x * 2);
    //         assert!(check_mphf(HashSet::from_iter(items)));
    //     }
    // }
}

pub mod bitvector {
    // The following code was copied and modified from rust-boomphf
    // which was also copied from another source. I will include
    // the link and license from rust-boomphf along with the
    // rust-boomphf attribution to the original source code.

    // https://github.com/10XGenomics/rust-boomphf/tree/master

    // The MIT License (MIT)

    // Copyright (c) 2014-2017 10x Genomics, Inc.

    // Permission is hereby granted, free of charge, to any person obtaining a copy
    // of this software and associated documentation files (the "Software"), to deal
    // in the Software without restriction, including without limitation the rights
    // to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    // copies of the Software, and to permit persons to whom the Software is
    // furnished to do so, subject to the following conditions:
    //
    // The above copyright notice and this permission notice shall be included in
    // all copies or substantial portions of the Software.
    //
    // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    // THE SOFTWARE.

    // Copyright (c) 2018 10x Genomics, Inc. All rights reserved.
    //
    // Note this code was copied from https://github.com/zhaihj/bitvector (MIT licensed),
    // and modified to add rank/select operations, and to use atomic primitives to allow
    // multi-threaded access. The original copyright license text is here:
    //
    // The MIT License (MIT)
    //
    // Copyright (c) 2016 Hongjie Zhai

    // ### BitVector Module
    //
    // BitVector uses one bit to represent a bool state.
    // BitVector is useful for the programs that need fast set operation (intersection, union,
    // difference), because that all these operations can be done with simple bitand, bitor, bitxor.
    //
    // ### Implementation Details
    //
    // BitVector is realized with a `Vec<u64>`. Each bit of an u64 represent if a elements exists.
    // BitVector always increases from the end to begin, it meats that if you add element `0` to an
    // empty bitvector, then the `Vec<u64>` will change from `0x00` to `0x01`.
    //
    // Of course, if the real length of set can not be divided by 64,
    // it will have a `capacity() % 64` bit memory waste.
    //

    type Word = u64;

    /// Bitvector
    #[derive(Debug)]
    pub struct BitVector<'a> {
        bits: u64,

        vector: &'a [u64],
    }

    impl<'a> BitVector<'a> {
        /// Build a new empty bitvector
        // pub fn new(bits: u64) -> Self {
        //     let n = u64s(bits);
        //     let mut v: Vec<Word> = Vec::with_capacity(n as usize);
        //     for _ in 0..n {
        //         v.push(Word::default());
        //     }
        //
        //     BitVector {
        //         bits,
        //         vector: v.into_boxed_slice(),
        //     }
        // }

        pub const fn from_embedded_state(embedded_state: (u64, &'a [u64])) -> Self {
            let (bits, vector) = embedded_state;
            BitVector { bits, vector }
        }

        /// new bitvector contains all elements
        ///
        /// If `bits % 64 > 0`, the last u64 is guaranteed not to
        /// have any extra 1 bits.
        #[allow(dead_code)]
        // pub fn ones(bits: u64) -> Self {
        //     let (word, offset) = word_offset(bits);
        //     let mut bvec: Vec<Word> = Vec::with_capacity((word + 1) as usize);
        //     for _ in 0..word {
        //         bvec.push(u64::max_value().into());
        //     }
        //
        //     let last_val = u64::max_value() >> (64 - offset);
        //     bvec.push(last_val.into());
        //     BitVector {
        //         bits,
        //         vector: bvec.into_boxed_slice(),
        //     }
        // }

        /// return if this set is empty
        ///
        /// if set does not contain any elements, return true;
        /// else return false.
        ///
        /// This method is averagely faster than `self.len() > 0`.
        #[allow(dead_code)]
        pub fn is_empty(&self) -> bool {
            return self.vector.iter().all(|x| *x == 0);
        }

        /// the number of elements in set
        pub fn len(&self) -> u64 {
            self.vector.iter().fold(0u64, |x0, x| {
                return x0 + x.count_ones() as u64;
            })
        }

        /*
        /// Clear all elements from a bitvector
        pub fn clear(&mut self) {
            for p in &mut self.vector {
                *p = 0;
            }
        }
        */

        /// If `bit` belongs to set, return `true`, else return `false`.
        ///
        /// Insert, remove and contains do not do bound check.
        pub fn contains(&self, bit: u64) -> bool {
            let (word, mask) = word_mask(bit);
            (self.get_word(word) & mask) != 0
        }

        /// compare if the following is true:
        ///
        /// self \cap {0, 1, ... , bit - 1} == other \cap {0, 1, ... ,bit - 1}
        pub fn eq_left(&self, other: &BitVector, bit: u64) -> bool {
            if bit == 0 {
                return true;
            }
            let (word, offset) = word_offset(bit - 1);
            // We can also use slice comparison, which only take 1 line.
            // However, it has been reported that the `Eq` implementation of slice
            // is extremly slow.
            //
            // self.vector.as_slice()[0 .. word] == other.vector.as_slice[0 .. word]
            //
            self.vector
                .iter()
                .zip(other.vector.iter())
                .take(word as usize)
                .all(|(s1, s2)| {
                    return s1 == s2;
                })
                && (self.get_word(word as usize) << (63 - offset))
                    == (other.get_word(word as usize) << (63 - offset))
        }

        /// insert a new element to set
        ///
        /// If value is inserted, return true,
        /// if value already exists in set, return false.
        ///
        /// Insert, remove and contains do not do bound check.

        /// insert a new element synchronously.
        /// requires &mut self, but doesn't use
        /// atomic instructions so may be faster
        /// than `insert()`.
        ///
        /// If value is inserted, return true,
        /// if value already exists in set, return false.
        ///
        /// Insert, remove and contains do not do bound check.
        // #[inline]

        /// remove an element from set
        ///
        /// If value is removed, return true,
        /// if value doesn't exist in set, return false.
        ///
        /// Insert, remove and contains do not do bound check.

        /// import elements from another bitvector
        ///
        /// If any new value is inserted, return true,
        /// else return false.

        /// the max number of elements can be inserted into set
        pub fn capacity(&self) -> u64 {
            self.bits
        }

        #[inline]
        pub fn get_word(&self, word: usize) -> u64 {
            return self.vector[word] as u64;
        }

        pub fn num_words(&self) -> usize {
            self.vector.len()
        }

        /// Return a iterator of the set element in the bitvector,
        pub fn iter(&self) -> BitVectorIter<'_> {
            BitVectorIter {
                iter: self.vector.iter(),
                current: 0,
                idx: 0,
                size: self.bits,
            }
        }
    }

    /// Iterator for BitVector
    pub struct BitVectorIter<'a> {
        iter: ::std::slice::Iter<'a, Word>,
        current: u64,
        idx: u64,
        size: u64,
    }

    impl<'a> Iterator for BitVectorIter<'a> {
        type Item = u64;
        fn next(&mut self) -> Option<u64> {
            if self.idx >= self.size {
                return None;
            }
            while self.current == 0 {
                self.current = if let Some(_i) = self.iter.next() {
                    let i = *_i;
                    if i == 0 {
                        self.idx += 64;
                        continue;
                    } else {
                        self.idx = u64s(self.idx) * 64;
                        i
                    }
                } else {
                    return None;
                }
            }
            let offset = self.current.trailing_zeros() as u64;
            self.current >>= offset;
            self.current >>= 1; // shift otherwise overflows for 0b1000_0000_…_0000
            self.idx += offset + 1;
            Some(self.idx - 1)
        }
    }

    fn u64s(elements: u64) -> u64 {
        (elements + 63) / 64
    }

    fn word_offset(index: u64) -> (u64, u64) {
        (index / 64, index % 64)
    }

    fn word_mask(index: u64) -> (usize, u64) {
        let word = (index / 64) as usize;
        let mask = 1 << (index % 64);
        (word, mask)
    }

    // #[cfg(test)]
    // mod tests {
    //     use super::*;
    //     #[test]
    //     fn union_two_vecs() {
    //         #[allow(unused_mut)]
    //         let mut vec1 = BitVector::new(65);
    //         #[allow(unused_mut)]
    //         let mut vec2 = BitVector::new(65);
    //         assert!(vec1.insert(3));
    //         assert!(!vec1.insert(3));
    //         assert!(vec2.insert(5));
    //         assert!(vec2.insert(64));
    //         assert!(vec1.insert_all(&vec2));
    //         assert!(!vec1.insert_all(&vec2));
    //         assert!(vec1.contains(3));
    //         assert!(!vec1.contains(4));
    //         assert!(vec1.contains(5));
    //         assert!(!vec1.contains(63));
    //         assert!(vec1.contains(64));
    //     }
    //
    //     #[test]
    //     fn bitvec_iter_works() {
    //         #[allow(unused_mut)]
    //         let mut bitvec = BitVector::new(100);
    //         bitvec.insert(1);
    //         bitvec.insert(10);
    //         bitvec.insert(19);
    //         bitvec.insert(62);
    //         bitvec.insert(63);
    //         bitvec.insert(64);
    //         bitvec.insert(65);
    //         bitvec.insert(66);
    //         bitvec.insert(99);
    //         assert_eq!(
    //             bitvec.iter().collect::<Vec<_>>(),
    //             [1, 10, 19, 62, 63, 64, 65, 66, 99]
    //         );
    //     }
    //
    //     #[test]
    //     fn bitvec_iter_works_2() {
    //         #[allow(unused_mut)]
    //         let mut bitvec = BitVector::new(319);
    //         bitvec.insert(0);
    //         bitvec.insert(127);
    //         bitvec.insert(191);
    //         bitvec.insert(255);
    //         bitvec.insert(319);
    //         assert_eq!(bitvec.iter().collect::<Vec<_>>(), [0, 127, 191, 255, 319]);
    //     }
    //
    //     #[test]
    //     fn eq_left() {
    //         #[allow(unused_mut)]
    //         let mut bitvec = BitVector::new(50);
    //         for i in &[0, 1, 3, 5, 11, 12, 19, 23] {
    //             bitvec.insert(*i);
    //         }
    //         #[allow(unused_mut)]
    //         let mut bitvec2 = BitVector::new(50);
    //         for i in &[0, 1, 3, 5, 7, 11, 13, 17, 19, 23] {
    //             bitvec2.insert(*i);
    //         }
    //
    //         assert!(bitvec.eq_left(&bitvec2, 1));
    //         assert!(bitvec.eq_left(&bitvec2, 2));
    //         assert!(bitvec.eq_left(&bitvec2, 3));
    //         assert!(bitvec.eq_left(&bitvec2, 4));
    //         assert!(bitvec.eq_left(&bitvec2, 5));
    //         assert!(bitvec.eq_left(&bitvec2, 6));
    //         assert!(bitvec.eq_left(&bitvec2, 7));
    //         assert!(!bitvec.eq_left(&bitvec2, 8));
    //         assert!(!bitvec.eq_left(&bitvec2, 9));
    //         assert!(!bitvec.eq_left(&bitvec2, 50));
    //     }
    //
    //     #[test]
    //     fn eq() {
    //         #[allow(unused_mut)]
    //         let mut bitvec = BitVector::new(50);
    //         for i in &[0, 1, 3, 5, 11, 12, 19, 23] {
    //             bitvec.insert(*i);
    //         }
    //         #[allow(unused_mut)]
    //         let mut bitvec2 = BitVector::new(50);
    //         for i in &[0, 1, 3, 5, 7, 11, 13, 17, 19, 23] {
    //             bitvec2.insert(*i);
    //         }
    //         #[allow(unused_mut)]
    //         let mut bitvec3 = BitVector::new(50);
    //         for i in &[0, 1, 3, 5, 11, 12, 19, 23] {
    //             bitvec3.insert(*i);
    //         }
    //
    //         assert!(bitvec != bitvec2);
    //         assert!(bitvec == bitvec3);
    //         assert!(bitvec2 != bitvec3);
    //     }
    //
    //     #[test]
    //     fn remove() {
    //         #[allow(unused_mut)]
    //         let mut bitvec = BitVector::new(50);
    //         for i in &[0, 1, 3, 5, 11, 12, 19, 23] {
    //             bitvec.insert(*i);
    //         }
    //         assert!(bitvec.contains(3));
    //         bitvec.remove(3);
    //         assert!(!bitvec.contains(3));
    //         assert_eq!(
    //             bitvec.iter().collect::<Vec<_>>(),
    //             vec![0, 1, 5, 11, 12, 19, 23]
    //         );
    //     }
    //
    //     #[test]
    //     fn is_empty() {
    //         assert!(!BitVector::ones(60).is_empty());
    //         assert!(!BitVector::ones(65).is_empty());
    //         #[allow(unused_mut)]
    //         let mut bvec = BitVector::new(60);
    //
    //         assert!(bvec.is_empty());
    //
    //         bvec.insert(5);
    //         assert!(!bvec.is_empty());
    //         bvec.remove(5);
    //         assert!(bvec.is_empty());
    //         #[allow(unused_mut)]
    //         let mut bvec = BitVector::ones(65);
    //         for i in 0..65 {
    //             bvec.remove(i);
    //         }
    //         assert!(bvec.is_empty());
    //     }
    //
    //     #[test]
    //     fn test_ones() {
    //         let bvec = BitVector::ones(60);
    //         for i in 0..60 {
    //             assert!(bvec.contains(i));
    //         }
    //         assert_eq!(bvec.iter().collect::<Vec<_>>(), (0..60).collect::<Vec<_>>());
    //     }
    //
    //     #[test]
    //     fn len() {
    //         assert_eq!(BitVector::ones(60).len(), 60);
    //         assert_eq!(BitVector::ones(65).len(), 65);
    //         assert_eq!(BitVector::new(65).len(), 0);
    //         #[allow(unused_mut)]
    //         let mut bvec = BitVector::new(60);
    //         bvec.insert(5);
    //         assert_eq!(bvec.len(), 1);
    //         bvec.insert(6);
    //         assert_eq!(bvec.len(), 2);
    //         bvec.remove(5);
    //         assert_eq!(bvec.len(), 1);
    //     }
    // }
    //
    // #[cfg(all(feature = "unstable", test))]
    // mod bench {
    //     extern crate test;
    //     use self::test::Bencher;
    //     use super::*;
    //     use std::collections::{BTreeSet, HashSet};
    //     #[bench]
    //     fn bench_bitset_operator(b: &mut Bencher) {
    //         b.iter(|| {
    //             #[allow(unused_mut)]
    //             let mut vec1 = BitVector::new(65);
    //             #[allow(unused_mut)]
    //             let mut vec2 = BitVector::new(65);
    //             for i in vec![0, 1, 2, 10, 15, 18, 25, 31, 40, 42, 60, 64] {
    //                 vec1.insert(i);
    //             }
    //             for i in vec![3, 5, 7, 12, 13, 15, 21, 25, 30, 29, 42, 50, 61, 62, 63, 64] {
    //                 vec2.insert(i);
    //             }
    //             vec1.intersection(&vec2);
    //             vec1.union(&vec2);
    //             vec1.difference(&vec2);
    //         });
    //     }
    //
    //     #[bench]
    //     fn bench_bitset_operator_inplace(b: &mut Bencher) {
    //         b.iter(|| {
    //             #[allow(unused_mut)]
    //             let mut vec1 = BitVector::new(65);
    //             #[allow(unused_mut)]
    //             let mut vec2 = BitVector::new(65);
    //             for i in vec![0, 1, 2, 10, 15, 18, 25, 31, 40, 42, 60, 64] {
    //                 vec1.insert(i);
    //             }
    //             for i in vec![3, 5, 7, 12, 13, 15, 21, 25, 30, 29, 42, 50, 61, 62, 63, 64] {
    //                 vec2.insert(i);
    //             }
    //             vec1.intersection_inplace(&vec2);
    //             vec1.union_inplace(&vec2);
    //             vec1.difference_inplace(&vec2);
    //         });
    //     }
    //
    //     #[bench]
    //     fn bench_hashset_operator(b: &mut Bencher) {
    //         b.iter(|| {
    //             #[allow(unused_mut)]
    //             let mut vec1 = HashSet::with_capacity(65);
    //             #[allow(unused_mut)]
    //             let mut vec2 = HashSet::with_capacity(65);
    //             for i in vec![0, 1, 2, 10, 15, 18, 25, 31, 40, 42, 60, 64] {
    //                 vec1.insert(i);
    //             }
    //             for i in vec![3, 5, 7, 12, 13, 15, 21, 25, 30, 29, 42, 50, 61, 62, 63, 64] {
    //                 vec2.insert(i);
    //             }
    //
    //             vec1.intersection(&vec2).cloned().collect::<HashSet<_>>();
    //             vec1.union(&vec2).cloned().collect::<HashSet<_>>();
    //             vec1.difference(&vec2).cloned().collect::<HashSet<_>>();
    //         });
    //     }
    //
    //     #[bench]
    //     fn bench_btreeset_operator(b: &mut Bencher) {
    //         b.iter(|| {
    //             #[allow(unused_mut)]
    //             let mut vec1 = BTreeSet::new();
    //             #[allow(unused_mut)]
    //             let mut vec2 = BTreeSet::new();
    //             for i in vec![0, 1, 2, 10, 15, 18, 25, 31, 40, 42, 60, 64] {
    //                 vec1.insert(i);
    //             }
    //             for i in vec![3, 5, 7, 12, 13, 15, 21, 25, 30, 29, 42, 50, 61, 62, 63, 64] {
    //                 vec2.insert(i);
    //             }
    //
    //             vec1.intersection(&vec2).cloned().collect::<HashSet<_>>();
    //             vec1.union(&vec2).cloned().collect::<HashSet<_>>();
    //             vec1.difference(&vec2).cloned().collect::<HashSet<_>>();
    //         });
    //     }
    // }
}
