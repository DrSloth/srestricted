#[cfg(all(not(feature="std"), feature="alloc"))]
extern crate alloc;

mod collections;

pub use collections::*;

use core::marker::PhantomData;
use std::{ops::{Deref}};

pub type NonEmpty<T, C> = LenRanged<T, C, 1, { usize::MAX }>; 

pub trait LinearSizedCollection<T> {
    fn len(&self) -> usize;
    fn push(&mut self, val: T);
    fn pop(&mut self) -> Option<T>;
    fn shrink_to(&mut self, len: usize) {
        for _ in 0..self.len().saturating_sub(len) {
            self.pop();
        }
    }
    fn extend_to(&mut self, len: usize, val: T) where T: Clone {
        for _ in 0..self.len().saturating_add(len) {
            self.push(val.clone());
        }
    }
    fn extend_to_with<F: Fn() -> T>(&mut self, len: usize, f: F) where Self: Sized {
        for _ in 0..self.len().saturating_add(len) {
            self.push(f());
        }
    }
    fn reserve(&mut self, additional: usize);
}

/// Used to receive a mutable view into a linear collection
///
/// This trait is marked unsafe as a wrong implementation can break invariants for [LenRanged] if the size of the
/// containing linear collection can be altered by the view.
pub unsafe trait ViewMut<'a> {
    type MutableView: 'a;
    fn view_mut(&'a mut self) -> Self::MutableView;
} 

pub enum LenRangeError {
    TooLarge,
    TooSmall,
}

pub struct LenRanged<T, C: LinearSizedCollection<T>, const MIN: usize, const MAX: usize>{ 
    collection: C, 
    _phantom: PhantomData<T>,
}

impl<T, C: LinearSizedCollection<T>, const MIN: usize, const MAX: usize> LenRanged<T, C, MIN, MAX> {
    pub fn new(collection: C) -> Result<Self, (LenRangeError, C)> {
        let len = collection.len();
        if len > MAX {
            Err((LenRangeError::TooLarge, collection))
        } else if len < MIN {
            Err((LenRangeError::TooSmall, collection))
        } else {
            Ok(Self {
                collection,
                _phantom: Default::default()
            })
        }
    }

    pub fn inner(&self) -> &C {
        &self.collection
    }

    pub unsafe fn inner_mut(&mut self) -> &mut C {
        &mut self.collection
    }

    pub fn mutate(&mut self, fill: impl Fn() -> T, mutator: impl Fn(&mut C)) {
        mutator(&mut self.collection);

        // Store the len because the calculation is potentially expensive
        let len = self.collection.len();
        if len > MAX {
            self.collection.shrink_to(MAX)
        } else if len < MIN {
            self.collection.extend_to_with(MIN, fill)
        } 
    }

    pub fn push(&mut self, val: T) -> Result<(), (LenRangeError, T)> {
        if self.collection.len() == MAX {
            Err((LenRangeError::TooLarge, val))
        } else {
            self.collection.push(val);
            Ok(())
        }
    } 

    pub fn pop(&mut self) -> Option<T> {
        if self.collection.len() == MIN {
            None
        } else {
            self.collection.pop()
        }
    }

    pub fn into_inner(self) -> C {
        self.collection
    }

    pub fn view(&self) -> &<C as Deref>::Target where C: Deref {
        &self.collection
    }

    pub fn view_mut<'a>(&'a mut self) -> <C as ViewMut>::MutableView where C: ViewMut<'a> {
        self.collection.view_mut()
    }
}


