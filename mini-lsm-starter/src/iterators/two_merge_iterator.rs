#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use anyhow::Result;

use super::StorageIterator;

/// Merges two iterators of different types into one. If the two iterators have the same key, only
/// produce the key once and prefer the entry from A.
pub struct TwoMergeIterator<A: StorageIterator, B: StorageIterator> {
    a: A,
    b: B,
    // Add fields as need
    choose_a: bool,
}

impl<
        A: 'static + StorageIterator,
        B: 'static + for<'a> StorageIterator<KeyType<'a> = A::KeyType<'a>>,
    > TwoMergeIterator<A, B>
{
    fn choose_a(a: &A, b: &B) -> bool {
        if !a.is_valid() {
            return false;
        }
        if !b.is_valid() {
            return true;
        }
        a.key() < b.key()
    }

    fn skip_b(&mut self) -> Result<()> {
        if self.a.is_valid() {
            if self.b.is_valid() && self.b.key() == self.a.key() {
                self.b.next()?;
            }
        }
        Ok(())
    }

    pub fn create(a: A, b: B) -> Result<Self> {
        let mut iter = Self {
            choose_a: false,
            a,
            b,
        };
        iter.skip_b()?;
        iter.choose_a = Self::choose_a(&iter.a, &iter.b);
        Ok(iter)
    }
}

impl<
        A: 'static + StorageIterator,
        B: 'static + for<'a> StorageIterator<KeyType<'a> = A::KeyType<'a>>,
    > StorageIterator for TwoMergeIterator<A, B>
{
    type KeyType<'a> = A::KeyType<'a>;

    fn key(&self) -> Self::KeyType<'_> {
        if self.choose_a {
            debug_assert!(self.a.is_valid());
            self.a.key()
        } else {
            debug_assert!(self.a.is_valid());
            self.b.key()
        }
    }

    fn value(&self) -> &[u8] {
        if self.choose_a {
            self.a.value()
        } else {
            self.b.value()
        }
    }

    fn is_valid(&self) -> bool {
        if self.choose_a {
            self.a.is_valid()
        } else {
            self.b.is_valid()
        }
    }

    fn next(&mut self) -> Result<()> {
        if self.choose_a {
            self.a.next()?;
        } else {
            self.b.next()?;
        }
        self.skip_b()?;
        self.choose_a = Self::choose_a(&self.a, &self.b);
        Ok(())
    }

    fn num_active_iterators(&self) -> usize {
        self.a.num_active_iterators() + self.b.num_active_iterators()
    }
}
