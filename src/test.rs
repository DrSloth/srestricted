//! Module for generating automated tests
//!
//! The easiest way is to use the [`complete_test`] macro for the type you implement [`crate::LinearSizedCollection`] for

/// Test the coherence of a `LinearSizedCollection`.
///
/// `$create` has to be an expression which creates the `LinearSizedCollection`.
/// `$name` has to be the name of the test module.
#[macro_export]
macro_rules! linear_collection_test {
    ($create:expr, $name:ident) => {
        #[cfg(test)]
        mod $name {
            use $crate::LinearSizedCollection;
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

pub use linear_collection_test;

/// Tests for [`SizeRestricted`] collection with the underlying collection being the tested type.
#[macro_export]
macro_rules! size_restricted_collection {
    ($create:expr, $name:ident) => {
        #[cfg(test)]
        mod $name {
            use $crate::{LinearSizedCollection, NonEmpty, SizeRangeError, SizeRestricted};
            fn is_linear_sized_collection<T, C: LinearSizedCollection<T>>(_collection: &C) {}

            #[test]
            fn empty() {
                let collection = $create;
                is_linear_sized_collection::<i32, _>(&collection);
                let _collection = SizeRestricted::<i32, _, 0, 1>::new(collection).unwrap();
            }

            #[test]
            fn always_empty() {
                let collection = $create;
                is_linear_sized_collection::<i32, _>(&collection);
                let _collection = SizeRestricted::<i32, _, 0, 0>::new(collection).unwrap();
            }

            #[test]
            fn always_empty_err_nonempty() {
                let mut collection = $create;
                LinearSizedCollection::push(&mut collection, 1);
                let too_large = SizeRestricted::<i32, _, 0, 0>::new(collection).unwrap_err();
                assert_eq!(too_large.0, SizeRangeError::TooLarge)
            }

            #[test]
            fn too_small_empty() {
                let collection = $create;
                is_linear_sized_collection::<i32, _>(&collection);
                let too_small = SizeRestricted::<i32, _, 1, 5>::new(collection).unwrap_err();
                assert_eq!(too_small.0, SizeRangeError::TooSmall);
            }

            #[test]
            fn too_small_nonempty() {
                let collection = $create;
                is_linear_sized_collection::<i32, _>(&collection);
                let too_small = SizeRestricted::<i32, _, 100, 5000>::new(collection).unwrap_err();
                assert_eq!(too_small.0, SizeRangeError::TooSmall);
            }

            #[test]
            fn nonempty_err_empty() {
                let collection = $create;
                is_linear_sized_collection::<i32, _>(&collection);
                let _collection = NonEmpty::new(collection).unwrap_err();
            }
        }
    };
}

pub use size_restricted_collection;

/// Creates a complete test suite for the type created with $create by creating using all other test macros.
///
/// $name should be the name of the test module and $create has to be an expression which creates an instance of the type to be tested.
#[macro_export]
macro_rules! complete_test {
    ($create:expr, $name:ident) => {
        #[cfg(test)]
        mod $name {
            #[cfg(test)]
            mod linear_collection_test {
                $crate::test::linear_collection_test!($create, $name);
            }

            #[cfg(test)]
            mod size_restricted_collection {
                $crate::test::size_restricted_collection!($create, $name);
            }
        }
    };
}

pub use complete_test;
