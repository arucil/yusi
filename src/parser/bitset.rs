use std::fmt::{self, Debug, Formatter};

type BitBlock = u64;

const BLOCK_NBITS: usize = std::mem::size_of::<BitBlock>() * 8;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BitSet {
  slice: Box<[BitBlock]>,
}

impl BitSet {
  pub fn new(num_bits: usize) -> Self {
    let len = (num_bits + BLOCK_NBITS - 1) / BLOCK_NBITS;
    Self {
      slice: vec![0; len].into_boxed_slice(),
    }
  }

  pub fn from_bit(num_bits: usize, bit: usize) -> Self {
    let mut s = Self::new(num_bits);
    s.insert(bit);
    s
  }

  pub fn clear(&mut self) {
    for x in self.slice.iter_mut() {
      *x = 0;
    }
  }

  pub fn insert(&mut self, bit: usize) {
    self.slice[bit / BLOCK_NBITS] |=
      1 << (bit as u64 % BLOCK_NBITS as u64);
  }

  /// Returns whether the set has changed.
  pub fn union_with(&mut self, other: &BitSet) -> bool {
    let mut changed = false;
    for i in 0..self.slice.len() {
      let old = self.slice[i];
      self.slice[i] |= other.slice[i];
      changed |= old != self.slice[i];
    }
    changed
  }

  pub fn iter(&self) -> Iter {
    Iter {
      slice: &*self.slice,
      bit: 0,
      index: 0,
    }
  }

  pub fn get(&self, bit: usize) -> bool {
    self.slice[bit / BLOCK_NBITS] & (bit as u64 % BLOCK_NBITS as u64) != 0
  }
}

pub struct Iter<'a> {
  slice: &'a [BitBlock],
  bit: usize,
  index: usize,
}

impl<'a> Iterator for Iter<'a> {
  type Item = usize;

  fn next(&mut self) -> Option<usize> {
    while self.index < self.slice.len() {
      if self.bit < BLOCK_NBITS {
        let bit = (self.slice[self.index] & !((1 << self.bit) - 1))
          .trailing_zeros() as usize;
        if bit < BLOCK_NBITS {
          self.bit = bit + 1;
          return Some(self.index * BLOCK_NBITS + bit);
        }
      }

      self.index += 1;
      self.bit = 0;
    }
    None
  }
}

impl Debug for BitSet {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_set().entries(self.iter()).finish()
  }
}

#[cfg(test)]
mod tests {
  use super::BitSet;
  use pretty_assertions::assert_eq;

  #[test]
  fn insert() {
    let mut set = BitSet::new(15);

    set.insert(7);
    set.insert(3);
    set.insert(7);
    set.insert(14);

    let vec = set.iter().collect::<Vec<_>>();

    assert_eq!(vec, vec![3, 7, 14]);
  }
}