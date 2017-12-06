//! A referenceable linked list
//!
//! Particular feature of importance is extracting any node from middle of list
//! in constant time

use std::cell::Cell;
use std::rc::Rc;
use std::iter::{Iterator, IntoIterator};
use std::ops::Deref;
use std::fmt;

pub struct LinkedList<T> {
    front: Option<Atom<T>>,
    back:  Option<Atom<T>>,
}

pub struct Atom<T>(Rc<AtomInner<T>>);
struct AtomInner<T> {
    prev: Cell<Option<Atom<T>>>,
    next: Cell<Option<Atom<T>>>,
    value: T
}

impl<T> LinkedList<T> {
    /// Create an empty `LinkedList`
    pub fn new() -> Self {
        LinkedList {
            front: None,
            back: None,
        }
    }

    /// Return a reference to the first element of the `LinkedList`
    pub fn front(&self) -> Option<&Atom<T>> {
        self.front.as_ref()
    }

    /// Return a reference to the last element of the `LinkedList`
    pub fn back(&self) -> Option<&Atom<T>> {
        self.back.as_ref()
    }

    /// Add an element to be the beginning of the `LinkedList`
    pub fn push_front(&mut self, value: T) {
        let atom = Atom::new(value);

        if let Some(next) = self.front.take() {
            next.0.prev.set(Some(atom.clone()));
            atom.0.next.set(Some(next));
            self.front = Some(atom);
        } else {
            self.front = Some(atom.clone());
            self.back  = Some(atom);
        }
    }

    /// Add an element to be the end of the `LinkedList`
    pub fn push_back(&mut self, value: T) {
        let atom = Atom::new(value);

        if let Some(prev) = self.back.take() {
            prev.0.next.set(Some(atom.clone()));
            atom.0.prev.set(Some(prev));
            self.back = Some(atom);
        } else {
            self.front = Some(atom.clone());
            self.back  = Some(atom);
        }
    }

    /// Remove an element from the beginning of the `LinkedList`
    pub fn pop_front(&mut self) -> Atom<T> {
        if let Some(front) = self.front.take() {
            if Rc::ptr_eq(&front.0, &self.back.as_ref().unwrap().0) { // If there's a front, there's a back
                self.back = None;
            } else {
                if let Some(next) = front.0.next.replace(None) {
                    next.0.prev.set(None);
                    self.front = Some(next)
                } else {
                    self.front = None;
                }
            }
            front
        } else {
            panic!("Empty LinkedList")
        }
    }

    /// Remove an element from the list
    ///
    /// Cannot be a method on `Atom` because we must update the front/back pointers
    /// of the `LinkedList`
    ///
    /// # Panic
    /// Will panic (on debug) if `atom` does not belong to this list
    pub fn extract(&mut self, atom: Atom<T>) {
        /// Ensure this atom exists in this LinkedList
        debug_assert!(self.into_iter().filter(|a| Rc::ptr_eq(&a.0, &atom.0)).count() == 1);

        let prev = atom.0.prev.replace(None);
        let next = atom.0.next.replace(None);
        if let Some(ref prev) = prev {
            prev.0.next.set(next.as_ref().map(|a| a.clone()));
        }
        if let Some(ref next) = next {
            next.0.prev.set(prev.as_ref().map(|a| a.clone()));
        }
        if self.front.is_some() && Rc::ptr_eq(&atom.0, &self.front.as_ref().unwrap().0) {
            self.front = next;
        }
        if self.back.is_some() && Rc::ptr_eq(&atom.0, &self.back.as_ref().unwrap().0) {
            self.back = prev;
        }
    }
}


// TODO figure out Iterator<Item = &T> instead of this
pub struct AtomIter<'a, T: 'a> {
    item: Option<Atom<T>>,

    // keep a lifetime
    list: &'a LinkedList<T>,
}

impl<'a, T: 'a> Iterator for AtomIter<'a, T> {
    type Item = Atom<T>;
    fn next(&mut self) -> Option<Self::Item> {
        fn clone_cell<T: Clone+Default>(cell: &Cell<T>) -> T {
            let x = cell.take();
            let y = x.clone();
            cell.set(x);
            y
        }

        match self.item.take() {
            Some(a) => {
                self.item = clone_cell(&a.0.next);
                Some(a)
            }
            None => None,
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = Atom<T>;
    type IntoIter = AtomIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        AtomIter {
            item: self.front().cloned(),
            list: self,
        }
    }
}

impl<T> Atom<T> {
    fn new(value: T) -> Self {
        Atom(Rc::new(AtomInner {
            prev: None.into(),
            next: None.into(),
            value: value,
        }))
    }

    /// Will unwrap if there is only one pointer to this Atom
    pub fn try_unwrap(self) -> Option<T> {
        Rc::try_unwrap(self.0).ok().map(|a| a.value)
    }
}
impl<T: Copy> Atom<T> {
    pub fn get(&self) -> T {
        self.0.value
    }
}

impl<T> Clone for Atom<T> {
    fn clone(&self) -> Self {
        Atom(self.0.clone())
    }
}

impl<T> Deref for Atom<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

impl<T: fmt::Debug> fmt::Debug for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.into_iter().collect::<Vec<_>>())
    }
}

impl<T: fmt::Debug> fmt::Debug for Atom<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.0.value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test::Bencher;
    use order::test::N;

    #[test]
    fn test_ll() {
        let mut ll = LinkedList::new();
        ll.push_back(1);
        ll.push_back(2);
        ll.push_front(-1);
        ll.push_front(-2);
        assert_eq!(ll.into_iter().map(|a| *a).collect::<Vec<_>>(), [-2, -1, 1, 2]);
    }

    #[bench]
    fn bench_ll_push_front(b: &mut Bencher) {
        b.iter(|| {
            let mut ll = LinkedList::new();
            for i in 0..N { ll.push_front(i)  }
        })
    }

    #[bench]
    fn bench_ll_push_back(b: &mut Bencher) {
        b.iter(|| {
            let mut ll = LinkedList::new();
            for i in 0..N { ll.push_back(i)  }
        })
    }

    #[bench]
    fn bench_ll_iter(b: &mut Bencher) {
        let mut ll = LinkedList::new();
        for i in 0..N { ll.push_back(i) }
        b.iter(|| ll.into_iter().map(|a| *a).sum::<usize>())
    }
}
