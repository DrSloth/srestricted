extern crate alloc;

pub mod test;

mod collections;

pub use collections::*;

use core::marker::PhantomData;
use std::ops::Deref;

pub type NonEmpty<T, C> = SizeRestricted<T, C, 1, { usize::MAX }>;
pub type ExactSized<T, C, const SIZE: usize> = SizeRestricted<T, C, SIZE, SIZE>;

/// A trait for linear collections which have a determinable size at any given point in time.
///
/// LinearSizedCollection reflects the general implementation of collections like [Vec](alloc::vec::Vec)
/// by defining operations like [pop](LinearSizedCollection::pop) or [push](LinearSizedCollection::push).
///
/// This trait is required by [SizeRestricted].
///
/// Every implementor should be tested with the [test::complete_test] macro.
pub trait LinearSizedCollection<T> {
    /// Get the len of the collection. This has to represent the number of elements inside this collection.
    fn len(&self) -> usize;
    /// Push element `val` to the end of the collection. Using [pop](LinearSizedCollection::pop) after [push](LinearSizedCollection::push)
    /// should return val. <br/> This behavior is important for coherent behavior of the trait and is tested inside the test module.
    fn push(&mut self, val: T);
    /// Pop one element from the end of the collection. If the collection is empty [None](core::option::Option::None) should be returned.
    fn pop(&mut self) -> Option<T>;
    /// Shrink this collection to len. By default this behavior is implemented using consecutive calls to [pop](LinearSizedCollection::pop)
    fn shrink_to(&mut self, len: usize) {
        for _ in len..self.len() {
            self.pop();
        }
    }
    /// Extend this collection to the size given by len. All newly pushed elements should be filled with val
    /// Works by calling [push](LinearSizedCollection::push) and cloning val.
    fn extend_to(&mut self, len: usize, val: T)
    where
        T: Clone,
        Self: Sized,
    {
        self.extend_to_with(len, || val.clone())
    }
    /// Extend this collection to the size given by len. The collection should be  filled by consecutively generating values with the
    /// function f. By default this function pushes the generated values using [push](LinearSizedCollection::push)
    /// This function is lazy no value is generated (f isn't called) unless the value is needed. The values also are generated
    /// for every new element, you can thus generate values with a mutable closure. It is not recommended to use if the computation always
    /// generates the same result AND is expensive
    fn extend_to_with<F: FnMut() -> T>(&mut self, len: usize, mut f: F)
    where
        Self: Sized,
    {
        self.reserve(len.saturating_sub(self.len()));
        for _ in self.len()..len {
            self.push(f());
        }
    }
    /// Try to reserve more space for at least additional more elements. Every push after calling this should be O(1).
    /// If this collection is not a reserving/array based collection (like [LinkedList](alloc::collections::LinkedList))
    /// this function should silently return.
    ///
    /// # Panics
    ///
    /// This function should panic if the inner implementation panics (a function 'try_reserve' will be added as
    /// soon as the alloc try reserve api) is stabilised.
    fn reserve(&mut self, additional: usize);
}

/// Used to receive a mutable view into a linear collection
///
/// This trait is marked unsafe as a wrong implementation can break invariants for [SizeRestricted] if the size of the
/// containing linear collection can be altered by the view.
pub unsafe trait ViewMut<'a> {
    /// This has to be a mutable view into the Self which can NOT mutate the length of the collection it is viewing into.
    /// If this type can mutate the length it can cause undefined behavior inside [SizeRestricted].
    type MutableView: 'a;
    /// Create an instane of [MutableView](crate::ViewMut::MutableView). Calling this function MAY mutate self but
    /// MUST NOT mutate the length of self.
    fn view_mut(&'a mut self) -> Self::MutableView;
}

/// An error representing a [LinearSizedCollection]s len being out of the bound of a [SizeRestricted]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum SizeRangeError {
    /// The length was larger then [SizeRestricted]::MAX
    TooLarge,
    /// The length was smaller then [SizeRestricted]::MIN
    TooSmall,
}

/// A wrapper around a [LinearSizedCollection] to restricts its size. The [length](LinearSizedCollection::len) is ensured
/// to be between MIN and MAX including both MIN and MAX. 
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct SizeRestricted<T, C: LinearSizedCollection<T>, const MIN: usize, const MAX: usize> {
    /// The inner collection whichs size is restricted
    collection: C,
    _phantom: PhantomData<T>,
}

impl<T, C: LinearSizedCollection<T>, const MIN: usize, const MAX: usize>
    SizeRestricted<T, C, MIN, MAX>
{
    /// Create a [SizeRestricted] while ensuring that the given collection has a correct size.
    /// If an error occurs the collection will be returned and a [SizeRangeError] describing the error.
    pub fn new(collection: C) -> Result<Self, (SizeRangeError, C)> {
        match Self::check_fit(&collection) {
            Ok(_) => Ok(unsafe { Self::create(collection) }),
            Err(e) => Err((e, collection))
        }
    }

    /// Creates a new instance of Self while making collection fit into the restriction using [Self::make_fit](SizeRestricted::make_fit)
    pub fn new_fit(mut collection: C) -> Self where T: Default {
        Self::make_fit(&mut collection);
        unsafe { Self::create(collection) }
    }

    /// Returns wether the given collections size is correct. [Ok] will be returned if it fits, if it is too small
    /// [Err]\([SizeRangeError::TooSmall]) and if the collection is too large [Err]\([SizeRangeError::TooSmall]) will be returned.
    pub fn check_fit(collection: &C) -> Result<(), SizeRangeError> {
        let len = collection.len();
        if len > MAX {
            Err(SizeRangeError::TooLarge)
        } else if len < MIN {
            Err(SizeRangeError::TooSmall)
        } else {
            Ok(())
        }
    }

    /// Makes the given collection fit into the size restriction
    pub fn make_fit(collection: &mut C) where T: Default {
        match Self::check_fit(collection) {
            Ok(()) => {},
            Err(SizeRangeError::TooLarge) => collection.shrink_to(MIN),
            Err(SizeRangeError::TooSmall) => collection.extend_to_with(MAX, Default::default),
        }
    }

    /// Creates this SizeRestricted collection from the collection parameter without checking its len.
    ///
    /// # Panics
    ///
    /// This function panics if MIN > MAX
    pub unsafe fn create(collection: C) -> Self {
        assert!(
            MIN <= MAX,
            "The Minimum size of a SizeRestricted collection has to be lower than its maximum"
        );
        Self {
            collection,
            _phantom: Default::default(),
        }
    }

    pub fn inner(&self) -> &C {
        &self.collection
    }

    pub unsafe fn inner_mut(&mut self) -> &mut C {
        &mut self.collection
    }

    pub fn mutate(&mut self, fill: impl FnMut() -> T, mut mutator: impl FnMut(&mut C)) {
        mutator(&mut self.collection);

        // Store the len because the calculation is potentially expensive
        let len = self.collection.len();
        if len > MAX {
            self.collection.shrink_to(MAX)
        } else if len < MIN {
            self.collection.extend_to_with(MIN, fill)
        }
    }

    pub fn push(&mut self, val: T) -> Result<(), (SizeRangeError, T)> {
        if self.collection.len() == MAX {
            Err((SizeRangeError::TooLarge, val))
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

    pub fn view(&self) -> &<C as Deref>::Target
    where
        C: Deref,
    {
        &self.collection
    }

    pub fn view_mut<'a>(&'a mut self) -> <C as ViewMut>::MutableView
    where
        C: ViewMut<'a>,
    {
        self.collection.view_mut()
    }
}

impl<T, C, const MIN: usize, const MAX: usize> Default for SizeRestricted<T, C, MIN, MAX>
where
    C: LinearSizedCollection<T> + Default,
    T: Default,
{
    fn default() -> Self {
        let mut collection = C::default();
        collection.extend_to_with(MIN, Default::default);
        unsafe { Self::create(collection) }
    }
}
