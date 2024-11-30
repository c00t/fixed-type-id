#![feature(str_from_raw_parts)]
#![feature(generic_const_exprs)]

//! A unique id generator for rust types.
//!
//! The crate provides a trait and a procedural macro. By implementing [`FixedTypeId`],
//! other crates can use [`FixedTypeId::ty_id()`], [`FixedTypeId::ty_name()`] and [`FixedTypeId::ty_version()`]
//! to get the type id, name and version about this type
//!
//! ## Notes
//!
//! ### Version
//!
//! For standard types, the version is always `(0,0,0)`, in the future, it may be changed to rustc version you are using.
//!
//! ### Differences between `fixed_type_id`, `fixed_type_id_without_version_hash` and `random_fixed_type_id`
//!
//! - `fixed_type_id`: Generate a unique id for the type, with a [`FixedId`] that [`rapidhash::rapidhash`] the name you provided,
//!    the version is also hashed into the [`FixedId`]. Version defaults to `(0,0,0)`, use `#[FixedTypeIdVersion((0,1,0))]` to change it.
//!    Use it when you want that different versions of your type have different ids.
//! - `fixed_type_id_without_version_hash`: Generate a unique id for the type, with a [`FixedId`] that [`rapidhash::rapidhash`] the name you provided,
//!    without version hashed into the [`FixedId`]. Use it when you want that different versions of your type have the same id.
//! - `random_fixed_type_id`: Generate a random id for the type, with a [`FixedId`] that random generated for each build.
//!
//! All these macros can be used with:
//!
//! - `#[FixedTypeIdVersion((x,y,z))]`: Set the version to `(x,y,z)`.
//! - `#[FixedTypeIdFile("filename.toml")]`: Store the type id into a file, so you can use it for debug, make sure the file already exists.
//! - `#[FixedTypeIdEqualTo("other_type")]`: Make the type id [`FixedId`] equal to `other_type`, so the two types have the same id, but different type names, and versions.
//!
//! ## Usage
//!
//! The example usage:
//!
//! ```rust
//! use fixed_type_id::{FixedTypeId, FixedId, fixed_type_id, name_version_to_hash};
//! use std::hash::Hasher;
//!
//! mod m {
//!     use fixed_type_id::{FixedTypeId, FixedId, fixed_type_id, FixedVersion};
//!     pub trait Q {}
//!     pub trait W {}
//!     pub trait E<T> {}
//!     fixed_type_id!{
//!        #[FixedTypeIdVersion((0,1,0))]
//!        // default to (0,0,0)
//!        // #[FixedTypeIdFile("types.toml")]
//!        // no default, but when store into file, version will be dropped, so only use it for debug.
//!        dyn m::Q; // type name is "dyn m::Q", it only store the type name you provided, without modification.
//!        dyn W; // type name is "dyn W", though `W` is under `m` module, it still store "dyn W"
//!        dyn E<u8>; // type name is "dyn E<u8>"
//!        A; // type name is "A"
//!        B<u8>; // type name is "B<u8>"
//!     }
//!     pub struct A;
//!     pub struct B<T> {
//!        pub t: T
//!     }
//!     impl Q for A {}
//! }
//! use m::*;
//! assert_eq!(<dyn Q>::TYPE_ID.0, name_version_to_hash("dyn m::Q", &(0,1,0).into()));
//! assert_eq!(<dyn Q>::TYPE_NAME, "dyn m::Q");
//! assert_eq!(<A as FixedTypeId>::TYPE_VERSION, (0,1,0).into());
//! assert_eq!(<A as FixedTypeId>::TYPE_NAME, "A");
//! ```
//!
//! Also, you can define this trait yoursellf:
//!
//! ```rust
//! use fixed_type_id::{FixedTypeId, FixedId, FixedVersion};
//! use rapidhash::rapidhash;
//!
//! struct MyType;
//!
//! impl FixedTypeId for MyType {
//!     const TYPE_NAME: &'static str = "MyType";
//!     const TYPE_ID: FixedId = FixedId::from_type_name(Self::TYPE_NAME, None);
//!     const TYPE_VERSION: FixedVersion = FixedVersion::new(0, 0, 0);
//! }
//!
//! assert_eq!(<MyType as FixedTypeId>::TYPE_NAME, "MyType");
//! assert_eq!(<MyType as FixedTypeId>::TYPE_ID.0, rapidhash::rapidhash("MyType".as_bytes()));
//! assert_eq!(<MyType as FixedTypeId>::TYPE_VERSION, (0,0,0).into());
//! ```
//!

use core::fmt;

pub use fixed_type_id_macros::{
    fixed_type_id, fixed_type_id_without_version_hash, random_fixed_type_id,
};
use semver::Version;

#[cfg(feature = "len128")]
pub const CONST_TYPENAME_LEN: usize = 128;

#[cfg(feature = "len64")]
pub const CONST_TYPENAME_LEN: usize = 64;

#[cfg(feature = "len256")]
pub const CONST_TYPENAME_LEN: usize = 256;

/// A strong type for type id.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FixedId(pub u64);

impl FixedId {
    /// Get UniqueId of a type
    pub const fn from<Target: 'static + ?Sized + FixedTypeId>() -> Self {
        Target::TYPE_ID
    }

    /// Get the inner u64 value.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Get UniqueId from a type name.
    ///
    /// It can be used inside const context.
    pub const fn from_type_name(type_name: &'static str, version: Option<FixedVersion>) -> Self {
        let hash = match version {
            None => rapidhash::rapidhash(type_name.as_bytes()),
            Some(version) => {
                // first hash the typename, get a base hash
                let name_hash = rapidhash::rapidhash(type_name.as_bytes());
                // then hash the version
                let version_hash = rapidhash::rapidhash(&version.const_to_bytes());
                // then combine name_hash and version_hash as a new `&[u8]`
                //
                // let combined_hash = rapidhash::rapidhash(&u64s_to_bytes(&[name_hash, version_hash]));
                //
                // or use rapid_mix
                let combined_hash = rapid_mix(name_hash, version_hash);
                // combine them
                combined_hash
            }
        };
        FixedId(hash)
    }
}

const fn u64s_to_bytes<const N: usize>(slice: &[u64; N]) -> [u8; N * 8] {
    let mut bytes = [0u8; N * 8];

    let mut slice_remaining: &[u64] = slice;
    let mut i = 0;
    let mut slice_index = 0;
    while let [current, tail @ ..] = slice_remaining {
        let mut current_bytes: &[u8] = &current.to_le_bytes();
        while let [current, tail @ ..] = current_bytes {
            bytes[i] = *current;
            i += 1;
            current_bytes = tail;
        }
        slice_index += 1;
        debug_assert!(i == 8 * slice_index);
        slice_remaining = tail;
    }

    bytes
}

/// Copy from [`rapidhash`]
#[inline(always)]
const fn rapid_mum(a: u64, b: u64) -> (u64, u64) {
    let r = a as u128 * b as u128;
    (r as u64, (r >> 64) as u64)
}

/// Copy from [`rapidhash`]
#[inline(always)]
const fn rapid_mix(a: u64, b: u64) -> u64 {
    let (a, b) = rapid_mum(a, b);
    a ^ b
}

impl fmt::Display for FixedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A trait for providing a type id number.
pub trait FixedTypeId {
    /// The type name defined by the user, more unique and stable name than the [`core::any::type_name`]
    ///
    /// When enabled feature `erase_name`, the type name will be a hex string of the hash of the original type name if it's a primitive type without generic.
    /// Otherwise, it will be the original type name.
    ///
    /// You should implement this trait as specific as possible. Because that the generic implement will make your binary size larger.
    const TYPE_NAME: &'static str;
    /// A unique id for a type.
    ///
    /// It's default use [`FixedId::from_type_name`] with [`Self::TYPE_VERSION`] as additional parameter.
    /// When you want to define an id without version, you can use [`FixedId::from_type_name`] without additional version parameter.
    const TYPE_ID: FixedId = FixedId::from_type_name(Self::TYPE_NAME, Some(Self::TYPE_VERSION));
    /// A semver for a type, with out pre release, build meta etc.
    ///
    /// Used to check version compatibility. If versions are not compatible, it can be cast to an semver.
    const TYPE_VERSION: FixedVersion = FixedVersion::new(0, 0, 0);

    /// Returns the type name.
    fn ty_name(&self) -> &'static str {
        Self::TYPE_NAME
    }

    /// Returns the type id number.
    fn ty_id(&self) -> FixedId {
        Self::TYPE_ID
    }

    /// Returns the version for a type
    fn ty_version(&self) -> FixedVersion {
        Self::TYPE_VERSION
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FixedVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl FixedVersion {
    /// Create a new `FixedVersion`
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        FixedVersion {
            major,
            minor,
            patch,
        }
    }

    /// Get the bytes of the version, can be used in const context.
    ///
    /// It's slower than [`as_bytes`], but can be used in const context.
    pub const fn const_to_bytes(&self) -> [u8; 24] {
        u64s_to_bytes(&[self.major, self.minor, self.patch])
    }

    /// Get the bytes presentation of the version, as a `[u8; 24]`
    pub fn to_bytes(&self) -> [u8; 24] {
        let mut bytes = [0u8; 24];
        bytes[0..8].copy_from_slice(&self.major.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.minor.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.patch.to_le_bytes());
        bytes
    }

    /// If a [`FixedVersion`] compatible with another [`FixedVersion`]
    pub fn is_compatible(&self, expected_version: &FixedVersion) -> bool {
        let compatible_cmp = semver::Comparator {
            op: semver::Op::Caret,
            major: expected_version.major,
            minor: Some(expected_version.minor),
            patch: Some(expected_version.patch),
            pre: semver::Prerelease::EMPTY,
        };
        compatible_cmp.matches(&Version::new(self.major, self.minor, self.patch))
    }

    /// If a [`FixedVersion`] matches a [`semver::Comparator`]?
    pub fn matches(&self, comparator: &semver::Comparator) -> bool {
        comparator.matches(&Version::new(self.major, self.minor, self.patch))
    }
}

impl From<(u64, u64, u64)> for FixedVersion {
    fn from(value: (u64, u64, u64)) -> Self {
        FixedVersion::new(value.0, value.1, value.2)
    }
}

impl From<Version> for FixedVersion {
    fn from(value: Version) -> Self {
        FixedVersion::new(value.major, value.minor, value.patch)
    }
}

pub fn name_version_to_hash(name: &str, version: &FixedVersion) -> u64 {
    let name_hash = rapidhash::rapidhash(name.as_bytes());
    // let version_hash = rapidhash::rapidhash(&version.as_bytes());
    let mut bytes = [0u8; 24];
    bytes[0..8].copy_from_slice(&version.major.to_le_bytes());
    bytes[8..16].copy_from_slice(&version.minor.to_le_bytes());
    bytes[16..24].copy_from_slice(&version.patch.to_le_bytes());
    rapid_mix(name_hash, rapidhash::rapidhash(&bytes))
}

/// A trait for providing a const fixed string for the type name, used to avoid heap when need to format the type name.
///
/// Useful for types with generic parameters. the size of the type name is limited by `CONST_TYPENAME_LEN`, which can be
/// configured by feature flags `len128`, `len64` and `len256`, the default is `len128`.
///
/// But note that implementing this trait for a lot of types will make your binary size larger,
/// and slow down your compile time.
///
/// ## Example
///
/// ```rust
/// use fixed_type_id::{ConstTypeName, FixedTypeId, fstr_to_str};
/// pub struct A<T> {
///     pub t: T,
/// }
///
/// impl<T: FixedTypeId> FixedTypeId for A<T> {
///     const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
/// }
///
/// impl<T: FixedTypeId> ConstTypeName for A<T> {
///     const RAW_SLICE: &[&str] = &["A", "<", T::TYPE_NAME, ">"];
/// }
///
/// assert_eq!(<A<u8> as FixedTypeId>::TYPE_NAME, "A<u8>");
/// ```
pub trait ConstTypeName {
    /// A raw slice for the type name, used to create a fixed `fstr`.
    ///
    /// It's the only const you should defined for your struct.
    const RAW_SLICE: &[&str];
    /// A fixed string for the type name, used to avoid heap when need to format the type name.
    const TYPE_NAME_FSTR: fixedstr::fstr<CONST_TYPENAME_LEN> = slice_to_fstr(Self::RAW_SLICE);
}

/// A helper function to get the type name of a type.
pub fn type_name<T: ?Sized + FixedTypeId>() -> &'static str {
    T::TYPE_NAME
}

/// A helper function to get the type id of a type.
pub fn type_id<T: ?Sized + FixedTypeId>() -> FixedId {
    T::TYPE_ID
}

/// A helper function to get the version of a type.
pub fn type_version<T: ?Sized + FixedTypeId>() -> FixedVersion {
    T::TYPE_VERSION
}

// implement the trait for primitive types in prelude
fixed_type_id_without_version_hash! {
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
  String;
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

/// Implements UniqueTypeId and ConstTypeName for wrapper types that delegate to their inner type(s).
///
#[macro_export]
macro_rules! implement_wrapper_fixed_type_id {
  (@impl_generics $wrapper:ident, ($first:ident $(, $rest:ident)*), $prefix:expr) => {
        impl<$first $(, $rest)*> FixedTypeId for $wrapper<$first $(, $rest)*>
        where
            $first: FixedTypeId,
            $($rest: FixedTypeId,)*
            Self: ConstTypeName,
        {
            const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
        }

        impl<$first: FixedTypeId $(, $rest: FixedTypeId)*> ConstTypeName for $wrapper<$first $(, $rest)*>
        where
            $first: FixedTypeId,
            $($rest: FixedTypeId,)*
        {
            const RAW_SLICE: &[&str] = &[
                $prefix,
                "<",
                $first::TYPE_NAME,
                $(
                    ",",
                    $rest::TYPE_NAME,
                )*
                ">"
            ];
        }
    };

    ($($wrapper:ident<$first:ident $(, $rest:ident)*> => $prefix:expr);* $(;)?) => {
      $(
        implement_wrapper_fixed_type_id!(@impl_generics $wrapper, ($first $(, $rest)*), $prefix);
      )*
  };
}

pub const fn fstr_to_str<const N: usize>(fstr: &'static fixedstr::fstr<N>) -> &'static str {
    unsafe { core::str::from_raw_parts(fstr.to_ptr(), fstr.len()) }
}

pub const fn slice_to_fstr<const N: usize>(slice: &[&str]) -> fixedstr::fstr<N> {
    fixedstr::fstr::<N>::const_create_from_str_slices(slice)
}

use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

implement_wrapper_fixed_type_id! {
  PhantomData<T> => "core::marker::PhantomData";
  Vec<T> => "alloc::vec::Vec";
  HashMap<K,V> => "std::collections::HashMap";
  Box<T> => "alloc::boxed::Box";
  BTreeMap<K,V> => "alloc::collections::BTreeMap";
  Option<T> => "core::option::Option";
}

/// Internal macro to implement FixedTypeId for tuples.
macro_rules! implement_tuple_fixed_type_id {
    () => {
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
    };

    (@internal $n:tt, $first:ident $(, $rest:ident)*) => {
        impl<$first $(, $rest)*> FixedTypeId for ($first $(, $rest)*)
        where
            $first: FixedTypeId,
            $($rest: FixedTypeId,)*
            Self: ConstTypeName,
        {
            const TYPE_NAME: &'static str = fstr_to_str(&<Self as ConstTypeName>::TYPE_NAME_FSTR);
        }

        impl<$first: FixedTypeId $(, $rest: FixedTypeId)*> ConstTypeName for ($first $(, $rest)*) {
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

/// Internal macro to implement [`FixedTypeId`] for fixed size arrays.
macro_rules! implement_array_fixed_type_id {
  () => {
      implement_array_fixed_type_id!(@internal 0);
      implement_array_fixed_type_id!(@internal 1);
      implement_array_fixed_type_id!(@internal 2);
      implement_array_fixed_type_id!(@internal 3);
      implement_array_fixed_type_id!(@internal 4);
      implement_array_fixed_type_id!(@internal 5);
      implement_array_fixed_type_id!(@internal 6);
      implement_array_fixed_type_id!(@internal 7);
      implement_array_fixed_type_id!(@internal 8);
      implement_array_fixed_type_id!(@internal 9);
      implement_array_fixed_type_id!(@internal 10);
      implement_array_fixed_type_id!(@internal 11);
      implement_array_fixed_type_id!(@internal 12);
      implement_array_fixed_type_id!(@internal 13);
      implement_array_fixed_type_id!(@internal 14);
      implement_array_fixed_type_id!(@internal 15);
      implement_array_fixed_type_id!(@internal 16);
      implement_array_fixed_type_id!(@internal 17);
      implement_array_fixed_type_id!(@internal 18);
      implement_array_fixed_type_id!(@internal 19);
      implement_array_fixed_type_id!(@internal 20);
      implement_array_fixed_type_id!(@internal 21);
      implement_array_fixed_type_id!(@internal 22);
      implement_array_fixed_type_id!(@internal 23);
      implement_array_fixed_type_id!(@internal 24);
      implement_array_fixed_type_id!(@internal 25);
      implement_array_fixed_type_id!(@internal 26);
      implement_array_fixed_type_id!(@internal 27);
      implement_array_fixed_type_id!(@internal 28);
      implement_array_fixed_type_id!(@internal 29);
      implement_array_fixed_type_id!(@internal 30);
      implement_array_fixed_type_id!(@internal 31);
      implement_array_fixed_type_id!(@internal 32);
  };

  (@internal $n:tt) => {
      impl<T: FixedTypeId> FixedTypeId for [T; $n]
      where
          Self: ConstTypeName,
      {
          const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
      }

      impl<T: FixedTypeId> ConstTypeName for [T; $n] {
          const RAW_SLICE: &[&str] = &[
              "[",
              T::TYPE_NAME,
              ";",
              stringify!($n),
              "]"
          ];
      }
  };
}

implement_array_fixed_type_id!();

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

#[cfg(test)]
mod tests {
    use std::hash::Hasher;
    use std::marker::PhantomData;

    use rapidhash::{rapidhash, RapidInlineHasher};

    use super::{fixed_type_id, fixed_type_id_without_version_hash};

    use super::*;

    #[test]
    fn test_unique_id_typeid_equal_to() {
        pub struct A1;
        pub struct A2;
        fixed_type_id_without_version_hash! {
          #[FixedTypeIdVersion((0,1,0))]
          A1;
        }
        fixed_type_id_without_version_hash! {
          #[FixedTypeIdVersion((0,2,0))]
          #[FixedTypeIdEqualTo("A1")]
          A2;
        }

        assert_eq!(<A1 as FixedTypeId>::TYPE_NAME, "A1");
        assert_eq!(<A2 as FixedTypeId>::TYPE_NAME, "A2");
        assert_eq!(<A1 as FixedTypeId>::TYPE_ID, <A2 as FixedTypeId>::TYPE_ID);
        assert_eq!(<A1 as FixedTypeId>::TYPE_VERSION, (0, 1, 0).into());
        assert_eq!(<A2 as FixedTypeId>::TYPE_VERSION, (0, 2, 0).into());
    }

    #[test]
    fn test_macro_manual_diff() {
        // with versin hash, default implementation
        mod a {
            use super::fixed_type_id;
            use super::{FixedId, FixedTypeId, FixedVersion};

            pub struct A;
            fixed_type_id! {
                A;
            }
        }
        mod b {
            use super::{FixedId, FixedTypeId, FixedVersion};
            pub struct A;
            impl FixedTypeId for A {
                const TYPE_NAME: &'static str = "A";
            }
        }
        assert_eq!(<b::A as FixedTypeId>::TYPE_ID.0, {
            name_version_to_hash("A", &(0, 0, 0).into())
        });
        assert_eq!(
            <b::A as FixedTypeId>::TYPE_ID.0,
            <a::A as FixedTypeId>::TYPE_ID.0
        );
        assert_eq!(<a::A as FixedTypeId>::TYPE_VERSION, (0, 0, 0).into());
        assert_eq!(
            <b::A as FixedTypeId>::TYPE_VERSION,
            <a::A as FixedTypeId>::TYPE_VERSION
        );
        assert_eq!(<a::A as FixedTypeId>::TYPE_NAME, "A");
        assert_eq!(
            <b::A as FixedTypeId>::TYPE_NAME,
            <a::A as FixedTypeId>::TYPE_NAME
        );
    }

    #[test]
    fn test_unique_id_generic_ne() {
        pub struct A<T> {
            pub _t: T,
        }
        fixed_type_id! {
          A<u8>;
          A<u16>;
        }
        assert_eq!(<A<u8> as FixedTypeId>::TYPE_NAME, "A<u8>");
        assert_eq!(<A<u16> as FixedTypeId>::TYPE_NAME, "A<u16>");
        assert_ne!(
            <A<u8> as FixedTypeId>::TYPE_ID,
            <A<u16> as FixedTypeId>::TYPE_ID
        );
        assert_eq!(
            <A<u8> as FixedTypeId>::TYPE_VERSION,
            <A<u16> as FixedTypeId>::TYPE_VERSION
        );
        assert_eq!(<A<u8> as FixedTypeId>::TYPE_VERSION, (0, 0, 0).into());
        assert_eq!(<A<u16> as FixedTypeId>::TYPE_VERSION, (0, 0, 0).into());
        let x = "xf";
        fn f<T: FixedTypeId>(x: T) {
            assert_eq!(x.ty_id(), <T as FixedTypeId>::TYPE_ID);
        }
        f(x);
    }

    #[test]
    fn test_tuple_type() {
        assert_eq!(<(String, u32) as FixedTypeId>::TYPE_NAME, "(String,u32)");
    }

    #[test]
    fn test_more_types() {
        // Test basic type name formatting
        assert_eq!(
            <Vec<(String, u32)> as FixedTypeId>::TYPE_NAME,
            "alloc::vec::Vec<(String,u32)>"
        );
        assert_eq!(
            <PhantomData<i32> as FixedTypeId>::TYPE_NAME,
            "core::marker::PhantomData<i32>"
        );
        assert_eq!(<((), String) as FixedTypeId>::TYPE_NAME, "((),String)");

        // Test array type formatting
        assert_eq!(<[u8; 10] as FixedTypeId>::TYPE_NAME, "[u8;10]");
        assert_eq!(<[(u8, u32); 20] as FixedTypeId>::TYPE_NAME, "[(u8,u32);20]");

        // Test complex nested types
        assert_eq!(
            <HashMap<String, Vec<u32>> as FixedTypeId>::TYPE_NAME,
            "std::collections::HashMap<String,alloc::vec::Vec<u32>>"
        );
        assert_eq!(
            <Option<Box<Vec<String>>> as FixedTypeId>::TYPE_NAME,
            "core::option::Option<alloc::boxed::Box<alloc::vec::Vec<String>>>"
        );
        assert_eq!(
            <(Vec<u8>, HashMap<String, u32>) as FixedTypeId>::TYPE_NAME,
            "(alloc::vec::Vec<u8>,std::collections::HashMap<String,u32>)"
        );

        // Test reference types
        assert_eq!(<&str as FixedTypeId>::TYPE_NAME, "&str");
        assert_eq!(<&[u8] as FixedTypeId>::TYPE_NAME, "&[u8]");

        // Test type ID comparisons
        assert_ne!(
            <[u8; 10] as FixedTypeId>::TYPE_ID,
            <[(u8, u32); 20] as FixedTypeId>::TYPE_ID
        );
        assert_ne!(
            <Vec<u8> as FixedTypeId>::TYPE_ID,
            <Vec<u16> as FixedTypeId>::TYPE_ID
        );
        assert_ne!(
            <Option<String> as FixedTypeId>::TYPE_ID,
            <Option<&str> as FixedTypeId>::TYPE_ID
        );
        assert_ne!(
            <HashMap<String, u32> as FixedTypeId>::TYPE_ID,
            <BTreeMap<String, u32> as FixedTypeId>::TYPE_ID
        );

        // Test that same types have same IDs
        assert_eq!(
            <Vec<u8> as FixedTypeId>::TYPE_ID,
            <Vec<u8> as FixedTypeId>::TYPE_ID
        );
        assert_eq!(
            <Option<String> as FixedTypeId>::TYPE_ID,
            <Option<String> as FixedTypeId>::TYPE_ID
        );

        // Test version consistency
        assert_eq!(<Vec<u8> as FixedTypeId>::TYPE_VERSION, (0, 0, 0).into());
        assert_eq!(
            <HashMap<String, u32> as FixedTypeId>::TYPE_VERSION,
            (0, 0, 0).into()
        );
        assert_eq!(
            <Option<Box<Vec<String>>> as FixedTypeId>::TYPE_VERSION,
            (0, 0, 0).into()
        );
    }
}
