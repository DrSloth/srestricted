//! A create for creating and managing size restricted collections.
//!
//! This crate can realise size restricted collections whichs len is clamped in between a `MIN` and `MAX` value.
//! Those values are set with const generics. There are simple cases like [`NonEmpty`] to realise never empty collections in safe code.
//! The main type of the crate is the [`SizeRestricted`] struct which handles the size restriction of the collection.
//!
//! If you want to use your own collection type with [`SizeRestricted`] you just have to implement the [`LinearSizedCollection`] trait.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod test;

mod collections;

pub use collections::*;

use core::{marker::PhantomData, ops::Deref};

/// A never empty linear sized collection
pub type NonEmpty<T, C> = SizeRestricted<T, C, 1, { usize::MAX }>;
/// A collection which has an exact amount of elements which can't change
pub type ExactSized<T, C, const SIZE: usize> = SizeRestricted<T, C, SIZE, SIZE>;

/// A trait for linear collections which have a determinable size at any given point in time.
///
/// [`LinearSizedCollection`] reflects the general implementation of collections like [`Vec`](alloc::vec::Vec)
/// by defining operations like [`pop`](LinearSizedCollection::pop) or [`push`](LinearSizedCollection::push).
///
/// This trait is required by [`SizeRestricted`].
///
/// Every implementor should be tested with the [`test::complete_test`] macro.
pub trait LinearSizedCollection<T> {
    /// Get the len of the collection. This has to represent the number of elements inside this collection.
    fn len(&self) -> usize;
    /// Push element `val` to the end of the collection. Using [`pop`](LinearSizedCollection::pop) after [`push`](LinearSizedCollection::push)
    /// should return val. <br/> This behavior is important for coherent behavior of the trait and is tested inside the test module.
    fn push(&mut self, val: T);
    /// Pop one element from the end of the collection. If the collection is empty [`None`](core::option::Option::None) should be returned.
    fn pop(&mut self) -> Option<T>;
    /// Shrink this collection to len. By default this behavior is implemented using consecutive calls to [`pop`](LinearSizedCollection::pop)
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
        self.extend_to_with(len, || val.clone());
    }
    /// Extends the linear collection to `len` by consecutively calling `fill` for every filled value. The values should be appended to the end.
    ///
    /// Fill is called for every added element, this means you can create generator like closures.
    /// You should use [`extend_to`](LinearSizedCollection::extend_to) if the value is always the same and expensive to compute but not to clone.
    fn extend_to_with<F: FnMut() -> T>(&mut self, len: usize, mut fill: F)
    where
        Self: Sized,
    {
        self.reserve(len.saturating_sub(self.len()));
        for _ in self.len()..len {
            self.push(fill());
        }
    }
    /// Try to reserve more space for at least additional more elements. Every push after calling this should be O(1).
    /// If this collection is not a reserving/array based collection (like [`LinkedList`](alloc::collections::LinkedList))
    /// this function should silently return.
    ///
    /// # Panics
    ///
    /// This function should panic if the inner implementation panics (a function `try_reserve` will be added as
    /// soon as the alloc try reserve api) is stabilised.
    fn reserve(&mut self, additional: usize);

    /// Check wether this [`LinearSizedCollection`] is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Used to receive a mutable view into a linear collection
///
/// This trait is marked unsafe as a wrong implementation can break invariants for [`SizeRestricted`] if the size of the
/// containing linear collection can be altered by the view.
///
/// # Safety
///
/// Implementors of this trait must guarantee that [`MutableView`] can not mutate the length of the
/// [`LinearSizedCollection`], certain functions may rely on the corectness of the length thus this
/// invariant must be upheld.
pub unsafe trait ViewMut<'a> {
    /// This has to be a mutable view into the Self which can NOT mutate the length of the collection it is viewing into.
    /// If this type can mutate the length it can cause undefined behavior inside [`SizeRestricted`].
    type MutableView: 'a;
    /// Create an instane of [`MutableView`](crate::ViewMut::MutableView). Calling this function MAY mutate self but
    /// MUST NOT mutate the length of self.
    fn view_mut(&'a mut self) -> Self::MutableView;
}

/// An error representing a [`LinearSizedCollection`]s len being out of the bound of a [`SizeRestricted`]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum SizeRangeError {
    /// The length was larger then [SizeRestricted]::MAX
    TooLarge,
    /// The length was smaller then [SizeRestricted]::MIN
    TooSmall,
}

impl core::fmt::Display for SizeRangeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::TooLarge => write!(f, "Too Large"),
            Self::TooSmall => write!(f, "Too Small"),
        }
    }
}

/// A wrapper around a [`LinearSizedCollection`] to restricts its size. The [`length`](LinearSizedCollection::len) is ensured
/// to be between MIN and MAX including both MIN and MAX.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct SizeRestricted<T, C: LinearSizedCollection<T>, const MIN: usize, const MAX: usize> {
    /// The inner collection whichs size is restricted
    collection: C,
    /// Boo
    _phantom: PhantomData<T>,
}

impl<T, C: LinearSizedCollection<T>, const MIN: usize, const MAX: usize>
    SizeRestricted<T, C, MIN, MAX>
{
    /// The min length
    pub const MIN: usize = MIN;
    /// The max length
    pub const MAX: usize = MAX;
    /// A validity check for the range
    const VALID: bool = {
        assert!(
            MIN <= MAX,
            "The MIN size of a SizeRestricted must be smaller or equal its MAX size"
        );
        true
    };

    /// Create a [`SizeRestricted`] while ensuring that the given collection has a correct size.
    /// If an error occurs the collection will be returned and a [`SizeRangeError`] describing the error.
    ///
    /// # Errors
    ///
    /// Returns an error if the collection doesn't fit in the size restriction (see [`check_fit`])
    pub fn new(collection: C) -> Result<Self, (SizeRangeError, C)> {
        if !Self::VALID {
            unreachable!("Self should always be valid or panic during compilation")
        }

        match Self::check_fit(&collection) {
            Ok(_) => Ok(Self::create(collection)),
            Err(e) => Err((e, collection)),
        }
    }

    /// Creates a new instance of Self while making collection fit into the restriction using [`Self::make_fit`](SizeRestricted::make_fit)
    pub fn new_fit(mut collection: C) -> Self
    where
        T: Default,
    {
        Self::make_fit(&mut collection);
        Self::create(collection)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Returns wether the given collections size is correct. [`Ok`] will be returned if it fits, if it is too small
    /// [`SizeRangeError::TooSmall`] and if the collection is too large [`SizeRangeError::TooLarge`] will be returned.
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

    /// Makes the given collection fit into the size restriction. Uses [`Default::default`] for extending with [`SizeRestricted::make_fit_with`]
    pub fn make_fit(collection: &mut C)
    where
        T: Default,
    {
        Self::make_fit_with(collection, Default::default);
    }

    /// Makes the given collection fit into the size restriction. Uses `fill` to extend the collection with [`LinearSizedCollection::extend_to_with`]
    /// if the collection is to small
    pub fn make_fit_with<F: FnMut() -> T>(collection: &mut C, fill: F) {
        match Self::check_fit(collection) {
            Ok(()) => {}
            Err(SizeRangeError::TooLarge) => collection.shrink_to(MIN),
            Err(SizeRangeError::TooSmall) => collection.extend_to_with(MAX, fill),
        }
    }

    /// Creates this `SizeRestricted` collection from the collection parameter.
    ///
    /// # Panics
    ///
    /// This function panics if the collection does not fit the size restriction.
    pub fn create(collection: C) -> Self {
        assert!(Self::VALID);
        Self::check_fit(&collection).unwrap_or_else(|e| {
            panic!(
                "The collection does not fit in {} {} (size is {}): {}",
                MIN,
                MAX,
                collection.len(),
                e,
            );
        });
        Self {
            collection,
            _phantom: PhantomData::default(),
        }
    }

    /// Get a immutable reference to the inner collection
    pub fn inner(&self) -> &C {
        &self.collection
    }

    /// Mutate the inner collection directly with the `mutator` function.
    ///
    /// The size range may be violated inside the mutator function and the collection is made fitting after `mutator` got executed.
    /// The resizing of the collection is done with [`make_fit_with`](SizeRestricted::make_fit_with) using `fill` as filling function.
    pub fn mutate(&mut self, fill: impl FnMut() -> T, mut mutator: impl FnMut(&mut C)) {
        mutator(&mut self.collection);

        Self::make_fit_with(&mut self.collection, fill);
    }

    /// Push an element to the collections. Returns [Ok] if pushing the element doesn't violate the size restriction,
    /// returns ([`SizeRangeError::TooLarge`], val) on error
    ///
    /// # Errors
    ///
    /// This function returns [`SizeRangeError::TooLarge`] if the size would exceed [`Self::MAX`]
    /// after the push.
    pub fn push(&mut self, val: T) -> Result<(), (SizeRangeError, T)> {
        if self.collection.len() == MAX {
            Err((SizeRangeError::TooLarge, val))
        } else {
            self.collection.push(val);
            Ok(())
        }
    }

    /// Pops an element if the size restriction doesn't get violated by the pop.
    pub fn pop(&mut self) -> Option<T> {
        if self.collection.len() == MIN {
            None
        } else {
            self.collection.pop()
        }
    }

    /// Unwraps the inner collection and lifts the size restriction
    pub fn into_inner(self) -> C {
        self.collection
    }

    /// Get an immutable view into the collection
    pub fn view(&self) -> &<C as Deref>::Target
    where
        C: Deref,
    {
        &self.collection
    }

    /// Get a mutable view into the collection.
    ///
    /// This is implemented with the [`ViewMut`] trait refer to it for more information on safety
    pub fn view_mut<'a>(&'a mut self) -> <C as ViewMut>::MutableView
    where
        C: ViewMut<'a>,
    {
        self.collection.view_mut()
    }
}

/// Creates a `SizeRestricted` collection with a size of `MIN`
impl<T, C, const MIN: usize, const MAX: usize> Default for SizeRestricted<T, C, MIN, MAX>
where
    C: LinearSizedCollection<T> + Default,
    T: Default,
{
    fn default() -> Self {
        let mut collection = C::default();
        collection.extend_to_with(MIN, Default::default);
        Self::create(collection)
    }
}

impl<T, C, const MIN: usize, const MAX: usize> IntoIterator for SizeRestricted<T, C, MIN, MAX>
where
    C: LinearSizedCollection<T> + IntoIterator<Item = T>,
{
    type IntoIter = C::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

#[cfg(feature = "impl_serde")]
impl<T, C: LinearSizedCollection<T> + serde::Serialize, const MIN: usize, const MAX: usize>
    serde::Serialize for SizeRestricted<T, C, MIN, MAX>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.collection.serialize(serializer)
    }
}

#[cfg(feature = "impl_serde")]
impl<
        'de,
        T,
        C: LinearSizedCollection<T> + serde::Deserialize<'de>,
        const MIN: usize,
        const MAX: usize,
    > serde::Deserialize<'de> for SizeRestricted<T, C, MIN, MAX>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let collection = C::deserialize(deserializer)?;
        Self::new(collection).map_err(|(e, _)| serde::de::Error::custom(e))
    }
}
