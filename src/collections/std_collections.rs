use crate::{LinearSizedCollection, ViewMut};

use std::{collections::VecDeque, ops::DerefMut};
#[cfg(feature="std")]
use std::collections as collections;
#[cfg(all(not(feature="std"), feature="alloc"))]
use alloc::collections as collections;

#[cfg(all(not(feature="std"), feature="alloc"))]
type Vec<T> = alloc::vec::Vec<T>;

impl<T> LinearSizedCollection<T> for Vec<T> {
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

unsafe impl<'a, T: 'a> ViewMut<'a> for VecDeque<T> {
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

pub type NonEmptyString = crate::NonEmpty<char, String>;

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
