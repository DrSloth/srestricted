use alloc::collections;
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

#[cfg(test)]
mod test {
    mod linear_collection_test {
        /// Test the coherence of LinearSizedCollection
        macro_rules! linear_collection_test {
            ($create:expr, $name:ident) => {
                mod $name {
                    use crate::LinearSizedCollection;
                    #[test]
                    fn pop_after_push() {
                        let mut collection = $create;
                        LinearSizedCollection::push(&mut collection, 10);
                        LinearSizedCollection::push(&mut collection, 20);

                        assert_eq!(LinearSizedCollection::pop(&mut collection), Some(20));
                        assert_eq!(LinearSizedCollection::pop(&mut collection), Some(10));
                    }

                    #[test]
                    fn len_after_extend() {
                        let mut collection = $create;
                        LinearSizedCollection::extend_to(&mut collection, 10, 0);

                        assert_eq!(LinearSizedCollection::len(&mut collection), 10)
                    }

                    #[test]
                    fn extend_shrink() {
                        let mut collection = $create;
                        LinearSizedCollection::extend_to(&mut collection, 10, 0);
                        LinearSizedCollection::shrink_to(&mut collection, 4);

                        assert_eq!(LinearSizedCollection::len(&mut collection), 4)
                    }

                    #[test]
                    fn multiple_resizes() {
                        let mut collection = $create;
                        LinearSizedCollection::extend_to(&mut collection, 10, 0);
                        LinearSizedCollection::shrink_to(&mut collection, 4);
                        assert_eq!(LinearSizedCollection::len(&mut collection), 4);

                        LinearSizedCollection::extend_to(&mut collection, 15, 0);
                        assert_eq!(LinearSizedCollection::len(&mut collection), 15);

                        LinearSizedCollection::push(&mut collection, 42);

                        assert_eq!(LinearSizedCollection::len(&mut collection), 16);
                        assert_eq!(LinearSizedCollection::pop(&mut collection), Some(42));
                        assert_eq!(LinearSizedCollection::len(&mut collection), 15);

                        assert_eq!(LinearSizedCollection::pop(&mut collection), Some(0));
                        assert_eq!(LinearSizedCollection::len(&mut collection), 14);

                        LinearSizedCollection::extend_to(&mut collection, 100, 0);
                        assert_eq!(LinearSizedCollection::len(&mut collection), 100);

                        LinearSizedCollection::shrink_to(&mut collection, 2);
                        assert_eq!(LinearSizedCollection::len(&mut collection), 2);
                    }
                }
            };
        }
        linear_collection_test!(Vec::new(), vec_test);
        linear_collection_test!(alloc::collections::VecDeque::new(), vecdeque_test);
        linear_collection_test!(alloc::collections::LinkedList::new(), linkedlist_test);
    }
}
