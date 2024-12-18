use fixed_type_id_macros::fixed_type_id;

use crate::usize_to_str;
use crate::{
    fstr_to_str, implement_wrapper_fixed_type_id, ConstTypeName, FixedId, FixedTypeId, FixedVersion,
};

// implement the trait for primitive types in prelude
fixed_type_id! {
  u8;
  u16;
  u32;
  u64;
  u128;
  usize;
  i8;
  i16;
  i32;
  i64;
  i128;
  isize;
  f32;
  f64;
  bool;
  char;
  alloc::string::String;
  str;
}

// implement FixedTypeId for basic reference types
impl<T: FixedTypeId + ?Sized> FixedTypeId for &T {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<T: FixedTypeId + ?Sized> ConstTypeName for &'_ T {
    const RAW_SLICE: &'static [&'static str] = &["&", T::TYPE_NAME];
}

impl<T: FixedTypeId + ?Sized> FixedTypeId for &mut T {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<T: FixedTypeId + ?Sized> ConstTypeName for &'_ mut T {
    const RAW_SLICE: &'static [&'static str] = &["&mut", T::TYPE_NAME];
}

// Unit type
impl FixedTypeId for () {
    const TYPE_NAME: &'static str = "()";
}

// // infallible
// impl FixedTypeId for ! {
//     const TYPE_NAME: &'static str = "!";
// }

// dyn Any
impl FixedTypeId for dyn Any {
    const TYPE_NAME: &'static str = "dyn core::any::Any";
}

impl<T: FixedTypeId + ?Sized> FixedTypeId for Box<T> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<T: FixedTypeId + ?Sized> ConstTypeName for Box<T> {
    const RAW_SLICE: &[&str] = &["alloc::boxed::Box<", T::TYPE_NAME, ">"];
}

use core::marker::PhantomData;
use core::num::NonZero;
use core::ops::Range;
use core::ops::RangeFrom;
use core::ops::RangeTo;
use core::ops::RangeToInclusive;
use core::result::Result;
use std::any::Any;
use std::collections::VecDeque;
use std::collections::{BTreeMap, HashMap};

// impl types with 1 or more generic parameters
implement_wrapper_fixed_type_id! {
  PhantomData<T> => "core::marker::PhantomData";
  Vec<T> => "alloc::vec::Vec";
  VecDeque<T> => "alloc::collections::VecDeque";
  // HashMap<K,V> => "std::collections::HashMap";
  // Box<T: ?Sized> => "alloc::boxed::Box";
  BTreeMap<K,V> => "alloc::collections::BTreeMap";
  Option<T> => "core::option::Option";
  Result<T,E> => "core::result::Result";
  Range<T> => "core::ops::Range";
  RangeFrom<T> => "core::ops::RangeFrom";
  RangeTo<T> => "core::ops::RangeTo";
  RangeToInclusive<T> => "core::ops::RangeToInclusive";
  NonZero<T: ZeroablePrimitive> => "core::num::nonzero::NonZero";
}

impl<K: FixedTypeId, V: FixedTypeId, S> FixedTypeId for HashMap<K, V, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, V: FixedTypeId, S> ConstTypeName for HashMap<K, V, S> {
    const RAW_SLICE: &[&str] = &[
        "std::collections::HashMap<",
        K::TYPE_NAME,
        ",",
        V::TYPE_NAME,
        ">",
    ];
}

use core::convert::Infallible;
use core::ops::RangeFull;
use core::time::Duration;
use std::num::ZeroablePrimitive;

// impl types with 0 generic parameters
fixed_type_id! {
    core::time::Duration;
    core::ops::RangeFull;
    core::convert::Infallible;
}

/// Internal macro to implement FixedTypeId for tuples.
macro_rules! implement_tuple_fixed_type_id {
    () => {
        // implement_tuple_fixed_type_id!(@internal 1, T1);
        implement_tuple_fixed_type_id!(@internal 2, T1, T2);
        implement_tuple_fixed_type_id!(@internal 3, T1, T2, T3);
        implement_tuple_fixed_type_id!(@internal 4, T1, T2, T3, T4);
        implement_tuple_fixed_type_id!(@internal 5, T1, T2, T3, T4, T5);
        implement_tuple_fixed_type_id!(@internal 6, T1, T2, T3, T4, T5, T6);
        implement_tuple_fixed_type_id!(@internal 7, T1, T2, T3, T4, T5, T6, T7);
        implement_tuple_fixed_type_id!(@internal 8, T1, T2, T3, T4, T5, T6, T7, T8);
        implement_tuple_fixed_type_id!(@internal 9, T1, T2, T3, T4, T5, T6, T7, T8, T9);
        implement_tuple_fixed_type_id!(@internal 10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
        implement_tuple_fixed_type_id!(@internal 11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
        implement_tuple_fixed_type_id!(@internal 12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
        implement_tuple_fixed_type_id!(@internal 13, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
        implement_tuple_fixed_type_id!(@internal 14, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
        implement_tuple_fixed_type_id!(@internal 15, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
        implement_tuple_fixed_type_id!(@internal 16, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
    };

    (@internal $n:tt, $first:ident $(, $rest:ident)*) => {
        impl<$first $(, $rest)*> FixedTypeId for ($first, $($rest,)*)
        where
            $first: FixedTypeId,
            $($rest: FixedTypeId,)*
            Self: ConstTypeName,
        {
            const TYPE_NAME: &'static str = fstr_to_str(&<Self as ConstTypeName>::TYPE_NAME_FSTR);
        }

        impl<$first: FixedTypeId $(, $rest: FixedTypeId)*> ConstTypeName for ($first, $($rest,)*) {
            const RAW_SLICE: &[&str] = &[
                "(",
                $first::TYPE_NAME,
                $(
                    ",",
                    $rest::TYPE_NAME,
                )*
                ")"
            ];
        }
    };
}

implement_tuple_fixed_type_id!();

// for (T,)
impl<T: FixedTypeId> FixedTypeId for (T,) {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<T: FixedTypeId> ConstTypeName for (T,) {
    const RAW_SLICE: &[&str] = &["(", T::TYPE_NAME, ",)"];
}

/// Only valid for N <= 32
impl<T: FixedTypeId, const N: usize> FixedTypeId for [T; N] {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

/// Only valid for N <= 32
impl<T: FixedTypeId, const N: usize> ConstTypeName for [T; N]
where
    [T; N]:,
{
    const RAW_SLICE: &[&str] = &["[", T::TYPE_NAME, ";", usize_to_str(N), "]"];
}

impl<T: FixedTypeId> FixedTypeId for &[T] {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<T: FixedTypeId> ConstTypeName for &'_ [T] {
    const RAW_SLICE: &'static [&'static str] = &["&[", T::TYPE_NAME, "]"];
}

impl<T: FixedTypeId> FixedTypeId for &mut [T] {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<T: FixedTypeId> ConstTypeName for &'_ mut [T] {
    const RAW_SLICE: &'static [&'static str] = &["&mut [", T::TYPE_NAME, "]"];
}

// function pointer
impl<T: FixedTypeId, R: FixedTypeId> FixedTypeId for fn(T) -> R {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<T: FixedTypeId, R: FixedTypeId> ConstTypeName for fn(T) -> R {
    const RAW_SLICE: &'static [&'static str] = &["fn(", T::TYPE_NAME, ") -> ", R::TYPE_NAME, ")"];
}

impl<R: FixedTypeId> FixedTypeId for fn() -> R {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<R: FixedTypeId> ConstTypeName for fn() -> R {
    const RAW_SLICE: &'static [&'static str] = &["fn() -> ", R::TYPE_NAME];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuple_type() {
        assert_eq!(
            <(String, u32) as FixedTypeId>::TYPE_NAME,
            "(alloc::string::String,u32)"
        );
    }

    #[test]
    fn more_types() {
        use std::collections::{BTreeMap, HashMap};

        // Basic stdlib type tests
        assert_eq!(
            <Vec<(String, u32)> as FixedTypeId>::TYPE_NAME,
            "alloc::vec::Vec<(alloc::string::String,u32)>"
        );
        assert_eq!(
            <PhantomData<i32> as FixedTypeId>::TYPE_NAME,
            "core::marker::PhantomData<i32>"
        );
        assert_eq!(
            <((), String) as FixedTypeId>::TYPE_NAME,
            "((),alloc::string::String)"
        );
        assert_eq!(
            <std::time::Duration as FixedTypeId>::TYPE_NAME,
            "core::time::Duration"
        );
        assert_eq!(
            <Vec<(String, u32)> as FixedTypeId>::TYPE_VERSION,
            FixedVersion::new(0, 0, 0)
        );
        assert_eq!(
            <PhantomData<i32> as FixedTypeId>::TYPE_VERSION,
            FixedVersion::new(0, 0, 0)
        );
        assert_eq!(
            <((), String) as FixedTypeId>::TYPE_VERSION,
            FixedVersion::new(0, 0, 0)
        );
        assert_eq!(
            <std::time::Duration as FixedTypeId>::TYPE_VERSION,
            FixedVersion::new(0, 0, 0)
        );
        assert_ne!(
            <Vec<u8> as FixedTypeId>::TYPE_ID,
            <Vec<u16> as FixedTypeId>::TYPE_ID
        );

        // Array type tests
        assert_eq!(<[u8; 10] as FixedTypeId>::TYPE_NAME, "[u8;10]");
        assert_eq!(<[(u8, u32); 20] as FixedTypeId>::TYPE_NAME, "[(u8,u32);20]");
        assert_ne!(
            <[u8; 10] as FixedTypeId>::TYPE_ID,
            <[(u8, u32); 20] as FixedTypeId>::TYPE_ID
        );

        // Reference type tests
        assert_eq!(<&str as FixedTypeId>::TYPE_NAME, "&str");
        assert_eq!(<&[u8] as FixedTypeId>::TYPE_NAME, "&[u8]");

        // Complex nested type tests
        assert_eq!(
            <HashMap<String, Vec<u32>> as FixedTypeId>::TYPE_NAME,
            "std::collections::HashMap<alloc::string::String,alloc::vec::Vec<u32>>"
        );
        assert_eq!(
            <Option<Box<Vec<String>>> as FixedTypeId>::TYPE_NAME,
            "core::option::Option<alloc::boxed::Box<alloc::vec::Vec<alloc::string::String>>>"
        );
        assert_eq!(
            <(Vec<u8>, HashMap<String, u32>) as FixedTypeId>::TYPE_NAME,
            "(alloc::vec::Vec<u8>,std::collections::HashMap<alloc::string::String,u32>)"
        );
        assert_ne!(
            <HashMap<String, u32> as FixedTypeId>::TYPE_ID,
            <BTreeMap<String, u32> as FixedTypeId>::TYPE_ID
        );
        assert_ne!(
            <Option<String> as FixedTypeId>::TYPE_ID,
            <Option<&str> as FixedTypeId>::TYPE_ID
        );
        assert_eq!(
            <HashMap<String, u32> as FixedTypeId>::TYPE_VERSION,
            FixedVersion::new(0, 0, 0)
        );
        assert_eq!(
            <Option<Box<Vec<String>>> as FixedTypeId>::TYPE_VERSION,
            FixedVersion::new(0, 0, 0)
        );
    }
}
