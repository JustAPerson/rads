//! Iacono's Working Set Structure
//!
//! The working set property is as follows:
//! if `t` elements have been accessed since `x` was last accessed,
//! it will take `O(log t)` time to access `x` again. Thus, if the structure is large
//! but a small subset are frequently accessed, they will be fast to access.
//!
//! Splay trees are a well known structure with this property, but splay trees have
//! amortized time bounds. [Iacono](https://dl.acm.org/citation.cfm?id=365522) described
//! a simple structure with ideal worst case time bounds.
//!
//! This structure functions by maintaining a sequence of exponentially growing trees.
//! When an item is accessed, it is moved from its current tree, to the first and smallest
//! tree so that it may easily be found again.

use order::linked_list::{LinkedList, Atom};
use std::collections::BTreeMap;
use std::cmp::{Ord, Ordering};

pub struct Iacono<K: Ord, V> {
    buckets: Vec<Bucket<Repr<K>, Box<(K, V)>>>,
    len: usize,
}

impl<K: Ord, V> Iacono<K, V> {
    pub fn new() -> Self {
        Iacono {
            buckets: Vec::new(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn find_bucket(&mut self, key: Repr<K>) -> Option<usize> {
        let mut i = 0;
        for bucket in &mut self.buckets {
            if bucket.tree.get(&key).is_some() {
                return Some(i)
            }
            i += 1
        }
        return None
    }

    fn bucket_push(&mut self, index: usize, key: Repr<K>, value: Box<(K, V)>) {
        if index == self.buckets.len() {
            self.buckets.push(Bucket::new())
        }
        self.buckets[index].push(key, value)
    }

    fn bucket_pop(&mut self, index: usize) -> (Repr<K>, Box<(K, V)>) {
        self.buckets[index].pop()
    }

    fn shift_single(&mut self, index: usize) {
        let (repr, pair) = self.bucket_pop(index);
        self.bucket_push(index + 1, repr, pair);
    }

    fn shift_multi(&mut self, max: usize) {
        for index in 0..max {
            if self.buckets[index].tree.len() <= (1 << index) {
                break
            }
            self.shift_single(index)
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let pair = Box::new((key, value));
        let key = Repr(&pair.0);

        if self.find_bucket(key).is_none() {
            let max = self.buckets.len();
            self.shift_multi(max);
            self.bucket_push(0, key, pair);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let key = Repr(key);

        if let Some(index) = self.find_bucket(key) {
            let pair = self.buckets[index].remove(&key);
            self.len -= 1;
            Some(pair.1)
        } else {
            None
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let repr = Repr(key);
        if let Some(index) = self.find_bucket(repr) {
            self.shift_multi(index);
            let pair = self.buckets[index].remove(&repr);

            let repr = Repr(&pair.0);
            self.buckets[0].push(repr, pair);
            self.buckets[0].tree.get(&repr).map(|&(_, ref v)| &v.1)
        } else {
            None
        }
    }
}

struct Bucket<K: Ord + Copy, V> {
    tree: BTreeMap<K, (Atom<K>, V)>,
    list: LinkedList<K>,
}

impl<K: Ord+Copy, V> Bucket<K, V> {
    fn new() -> Self {
        Bucket {
            tree: BTreeMap::new(),
            list: LinkedList::new(),
        }
    }

    fn push(&mut self, key: K, value: V) {
        self.list.push_back(key);

        let r = self.list.back().unwrap().clone();
        self.tree.insert(key, (r, value));
    }

    fn pop(&mut self) -> (K, V) {
        let key = self.list.pop_front().get();
        let val = self.tree.remove(&key).unwrap();
        (val.0.get(), val.1)
    }

    fn remove(&mut self, key: &K) -> V {
        let (atom, val) = self.tree.remove(key).unwrap();
        self.list.extract(atom);
        val
    }
}

/// A wrapper around *ptrs that is useful
struct Repr<T>(*const T);
impl<T: Ord> Ord for Repr<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { (*self.0).cmp(&*other.0) }
    }
}
impl<T: Ord> PartialOrd for Repr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unsafe { Some((*self.0).cmp(&*other.0)) }
    }
}
impl<T: Ord> PartialEq for Repr<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.0).eq(&*other.0) }
    }
}
impl<T: Ord> Eq for Repr<T> {}
impl<T> Copy for Repr<T> {}
impl<T> Clone for Repr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iacono_basic() {
        let mut t: Iacono<&str, i32> = Iacono::new();
        t.insert("a", 3);
        t.insert("b", 2);
        t.insert("c", 1);
        assert_eq!(t.get(&"a"), Some(&3i32));
        assert_eq!(t.get(&"b"), Some(&2i32));
        assert_eq!(t.get(&"c"), Some(&1i32));
    }

    #[test]
    fn test_iacono_growth() {
        let mut t: Iacono<usize, ()> = Iacono::new();
        for i in 0..512 {
            t.insert(i, ());
        }

        let buckets = t.buckets.iter().map(|b| b.tree.len()).collect::<Vec<_>>();
        assert_eq!(buckets, [2, 2, 4, 8, 16, 32, 64, 128, 256]);
    }
}
