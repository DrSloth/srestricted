use alloc::{collections, vec::Vec, string::String};
use core::ops::DerefMut;

use crate::{LinearSizedCollection, ViewMut};

impl<T> LinearSizedCollection<T> for alloc::vec::Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    fn push(&mut self, val: T) {
        self.push(val)
    }

    fn shrink_to(&mut self, len: usize) {
        self.truncate(len)
    }

    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
}

unsafe impl<'a, T: 'a> ViewMut<'a> for Vec<T> {
    type MutableView = &'a mut [T];
    fn view_mut(&'a mut self) -> Self::MutableView {
        self.deref_mut()
    }
}

impl<T> LinearSizedCollection<T> for collections::VecDeque<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn pop(&mut self) -> Option<T> {
        self.pop_back()
    }

    fn push(&mut self, val: T) {
        self.push_back(val)
    }

    fn shrink_to(&mut self, len: usize) {
        self.truncate(len)
    }

    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
}

unsafe impl<'a, T: 'a> ViewMut<'a> for collections::VecDeque<T> {
    type MutableView = &'a mut [T];
    fn view_mut(&'a mut self) -> Self::MutableView {
        self.make_contiguous()
    }
}

impl<T> LinearSizedCollection<T> for collections::LinkedList<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn pop(&mut self) -> Option<T> {
        self.pop_back()
    }

    fn push(&mut self, val: T) {
        self.push_back(val)
    }

    fn reserve(&mut self, _additional: usize) {}
}

impl LinearSizedCollection<char> for String {
    fn len(&self) -> usize {
        self.len()
    }

    fn pop(&mut self) -> Option<char> {
        self.pop()
    }

    fn push(&mut self, val: char) {
        self.push(val)
    }

    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
}

#[cfg(test)]
mod test {
    mod linear_alloc_collection_test {
        crate::test::complete_test!(Vec::new(), vec_test);
        crate::test::complete_test!(alloc::collections::VecDeque::new(), vecdeque_test);
        crate::test::complete_test!(alloc::collections::LinkedList::new(), linkedlist_test);
    }
}
