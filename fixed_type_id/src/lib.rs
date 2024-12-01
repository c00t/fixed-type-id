#![deny(missing_docs)]
#![allow(internal_features)]
#![allow(incomplete_features)]
#![feature(str_from_raw_parts)]
#![feature(generic_const_exprs)]
#![feature(nonzero_internals)]
#![doc = include_str!("../README.md")]

mod remote_impl;

use core::fmt;
use std::hash::Hash;

pub use fixed_type_id_macros::{
    fixed_type_id, fixed_type_id_without_version_hash, random_fixed_type_id,
};
use semver::Version;

/// The length of the type name, can be configured by feature flags `len128`, `len64` and `len256`, the default is `len128`.
#[cfg(feature = "len128")]
pub const CONST_TYPENAME_LEN: usize = 128;

/// The length of the type name, can be configured by feature flags `len128`, `len64` and `len256`, the default is `len128`.
#[cfg(feature = "len64")]
pub const CONST_TYPENAME_LEN: usize = 64;

/// The length of the type name, can be configured by feature flags `len128`, `len64` and `len256`, the default is `len128`.
#[cfg(feature = "len256")]
pub const CONST_TYPENAME_LEN: usize = 256;

/// A strong type for type id.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FixedId(pub u64);

/// Just write internal [`u64`] with [`std::hash::Hasher::write_u64`].
impl Hash for FixedId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

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
                // rapidhash::rapidhash(&u64s_to_bytes(&[name_hash, version_hash]));
                //
                // or use rapid_mix
                rapid_mix(name_hash, version_hash)
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

/// for n <= 32, returns a static string
/// for n > 32, returns "N"
/// for special usize, eg 64, 128, 256, 512, 768, 1024, 2048, 4096, 8192, 16384, 32768, 65536, returns a static string
pub const fn usize_to_str(n: usize) -> &'static str {
    match n {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "10",
        11 => "11",
        12 => "12",
        13 => "13",
        14 => "14",
        15 => "15",
        16 => "16",
        17 => "17",
        18 => "18",
        19 => "19",
        20 => "20",
        21 => "21",
        22 => "22",
        23 => "23",
        24 => "24",
        25 => "25",
        26 => "26",
        27 => "27",
        28 => "28",
        29 => "29",
        30 => "30",
        31 => "31",
        32 => "32",
        64 => "64",
        128 => "128",
        256 => "256",
        512 => "512",
        768 => "768",
        1024 => "1024",
        2048 => "2048",
        4096 => "4096",
        8192 => "8192",
        16384 => "16384",
        32768 => "32768",
        65536 => "65536",
        _ => "N",
    }
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

/// A semver for a type, but without pre release, build meta etc.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FixedVersion {
    /// The major version number.
    pub major: u64,
    /// The minor version number.
    pub minor: u64,
    /// The patch version number.
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

/// Get the hash from a type name and version, use the same procedure as [`FixedId::from_type_name`], but better performance.
///
/// It can't be used in const context.
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
    const TYPE_NAME_FSTR: fixedstr_ext::fstr<CONST_TYPENAME_LEN> = slice_to_fstr(Self::RAW_SLICE);
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

/// Implements [`FixedTypeId`] and [`ConstTypeName`] for wrapper types with 1 or moregeneric parameters which implement [`FixedTypeId`].
///
/// It's useful for types like `Vec<T>`, `HashMap<K,V>`, `Option<T>`, etc.
///
/// When you want to implement [`FixedTypeId`] for types with 0 generic parameters, use [`fixed_type_id!`] or [`fixed_type_id_without_version_hash!`] instead,
/// or implement it manually.
#[macro_export]
macro_rules! implement_wrapper_fixed_type_id {
    // New arm for handling generic with bounds
    (@impl_generics $wrapper:ident, ($first:ident: $bound:path $(, $rest:ident)*), $prefix:expr) => {
        impl<$first $(, $rest)*> FixedTypeId for $wrapper<$first $(, $rest)*>
        where
            $first: FixedTypeId + $bound,
            $($rest: FixedTypeId,)*
            Self: ConstTypeName,
        {
            const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
        }

        impl<$first $(, $rest)*> ConstTypeName for $wrapper<$first $(, $rest)*>
        where
            $first: FixedTypeId + $bound,
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

    // Original arm for simple generics (keep for backward compatibility)
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

    // Entry point - handle both bounded and unbounded generics
    ($($wrapper:ident<$first:ident $(: $bound:path)? $(, $rest:ident)*> => $prefix:expr);* $(;)?) => {
        $(
            implement_wrapper_fixed_type_id!(@impl_generics $wrapper, ($first $(: $bound)? $(, $rest)*), $prefix);
        )*
    };
}

/// Helper function to convert a fixed string [`fixedstr_ext::fstr`] to a string.
pub const fn fstr_to_str<const N: usize>(fstr: &'static fixedstr_ext::fstr<N>) -> &'static str {
    unsafe { core::str::from_raw_parts(fstr.to_ptr(), fstr.len()) }
}

/// Helper function to convert a slice of string to a fixed string [`fixedstr_ext::fstr`].
pub const fn slice_to_fstr<const N: usize>(slice: &[&str]) -> fixedstr_ext::fstr<N> {
    fixedstr_ext::fstr::<N>::const_create_from_str_slice(slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_id_typeid_equal_to() {
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
    fn unique_id_generic_ne() {
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
    }

    #[test]
    fn macro_manual_diff() {
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
            use super::FixedTypeId;
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
}
