//! A cyclically linked list

use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::ops::Deref;
use std::clone::Clone;


/// # Warning
/// Currently leaks memory. This is possible to fix but not a priority.
pub struct CyclicList<T>(Rc<Inner<T>>);

struct Inner<T> {
    item: T,
    prev: RefCell<CyclicList<T>>,
    next: RefCell<CyclicList<T>>,
}

impl<T> CyclicList<T> {
    pub fn new(item: T) -> Self {
        use std::mem;
        use std::ptr;

        unsafe {
            let list = CyclicList(Rc::new(Inner {
                item,
                prev: mem::uninitialized(),
                next: mem::uninitialized(),
            }));

            let prev = list.clone();
            let next = list.clone();
            ptr::write(&mut *list.0.prev.borrow_mut(), prev);
            ptr::write(&mut *list.0.prev.borrow_mut(), next);

            list
        }
    }

    pub fn prev(&self) -> Ref<CyclicList<T>> {
        self.0.prev.borrow()
    }

    pub fn next(&self) -> Ref<CyclicList<T>> {
        self.0.next.borrow()
    }

    pub fn is_single(&self) -> bool {
        Rc::ptr_eq(&self.0, &self.next().0)
    }

    fn put_behind(&self, other: Self) {
        *other.0.next.borrow_mut() = self.clone();
        *self.0.prev.borrow_mut() = other
    }
    pub fn push_front(&self, other: Self) {
        debug_assert!(other.is_single());
        let prev = self.prev().clone();

        other.put_behind(prev);
        self.put_behind(other);
    }

    pub fn push_back(&self, other: Self) {
        debug_assert!(other.is_single());
        let next = self.next().clone();

        other.put_behind(self.clone());
        next.put_behind(other);
    }

    pub fn extend_back(&self, first: Self) {
        let next = self.next().clone();
        let last = first.prev().clone();

        first.put_behind(self.clone());
        next.put_behind(last);
    }

    pub fn extend_front(&self, first: Self) {
        let prev = self.prev().clone();
        let last = first.prev().clone();

        first.put_behind(prev);
        self.put_behind(last);
    }

    pub fn remove(&self) {
        if self.is_single() { return }
        self.next().put_behind(self.prev().clone());
    }

    pub fn ptr_eq(this: &Self, that: &Self) -> bool {
        Rc::ptr_eq(&this.0, &that.0)
    }
}

impl<T> Deref for CyclicList<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0.item
    }
}

impl<T> Clone for CyclicList<T> {
    fn clone(&self) -> Self {
        CyclicList(self.0.clone())
    }
}
