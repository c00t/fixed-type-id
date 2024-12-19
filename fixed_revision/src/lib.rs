use core::fmt;

use fixed_type_id::{type_id, FixedId, FixedTypeId, FixedVersion};

/// A struct that wraps a type id and a data.
///
/// It's used by [`fixed_revision_macros::revisioned`] to wrap the enum that contains all revisions of a type.
/// So the `T` is often an enum.
///
/// 1. For the [`serde`] serialization is used to convert between the original enum and
/// [`FixedTypeIdTagged`] by using `serde(from = "FixedTypeIdTagged<T>", into = "FixedTypeIdTagged<T>")`.
/// 2. For the [`rkyv`] serialization, currently it isn't implemented.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", rkyv(attr(allow(missing_docs))))]
#[derive(Debug, Clone, Copy)]
pub struct FixedTypeIdTagged<T: FixedTypeId> {
    /// The [`FixedId`] type id of the type.
    pub type_id: FixedId,
    /// The data of the type.
    ///
    /// For [`rkyv`], this field is annotated with `#[rkyv(with = rkyv::with::AsBox)]`, because without it,
    /// [`FixedTypeIdTagged`] has different layout in memory than the [`FixedTypeIdTag`].
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsBox))]
    pub data: T,
}

impl<T: FixedTypeId> From<T> for FixedTypeIdTagged<T> {
    fn from(data: T) -> Self {
        FixedTypeIdTagged {
            type_id: type_id::<T>(),
            data,
        }
    }
}

/// Used as the type deserialization target for data which deserialized by [`FixedTypeIdTagged`].
///
/// When deserializing the data, first deserialize the data into a [`FixedTypeIdTag`], get the [`FixedId`]
/// and [`FixedVersion`] metadata, then deserialize the data into the actual target type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", rkyv(attr(allow(missing_docs))))]
#[derive(Debug, Clone, Copy)]
pub struct FixedTypeIdTag {
    /// The [`FixedId`] typeid of the deserialized type.
    pub type_id: FixedId,
    /// The version of the deserialized data.
    ///
    /// for rkyv, the `with = Box<GeneralVersion>` must be specified, because the `data` will have different layout in [`FixedTypeIdTagged`].
    #[cfg_attr(feature = "rkyv", rkyv(with = Box<GeneralVersion>))]
    data: FixedVersionTag,
}

impl FixedTypeIdTag {
    /// Get the [`FixedId`] type id and [`FixedVersion`] version of the underlying data.
    pub fn get_identifier(&self) -> (FixedId, FixedVersion) {
        (self.type_id, self.data.version.into())
    }
}

impl ArchivedFixedTypeIdTag {
    pub fn get_identifier(&self) -> (FixedId, FixedVersion) {
        ((&self.type_id).into(), self.data.get().into())
    }
}

/// A struct only contains a [`FixedVersion`] tag.
///
/// It's field names conform to the [`fixed_revision_macros::revisioned`] implementation,
/// so it can be used to deserialized the [`FixedTypeIdTagged::data`] field which serialized by [`FixedTypeIdTagged`],
/// but **without the actual versioned** data.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", rkyv(attr(allow(missing_docs))))]
#[derive(Debug, Clone, Copy)]
struct FixedVersionTag {
    /// The version of the type.
    pub version: GeneralVersion,
}

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", rkyv(attr(allow(missing_docs))))]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug, Copy, Clone)))]
#[cfg_attr(not(any(feature = "rkyv", feature = "serde")), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
enum GeneralVersion {
    // upto V128
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
    V10 = 10,
    V11 = 11,
    V12 = 12,
    V13 = 13,
    V14 = 14,
    V15 = 15,
    V16 = 16,
    V17 = 17,
    V18 = 18,
    V19 = 19,
    V20 = 20,
    V21 = 21,
    V22 = 22,
    V23 = 23,
    V24 = 24,
    V25 = 25,
    V26 = 26,
    V27 = 27,
    V28 = 28,
    V29 = 29,
    V30 = 30,
    V31 = 31,
    V32 = 32,
    V33 = 33,
    V34 = 34,
    V35 = 35,
    V36 = 36,
    V37 = 37,
    V38 = 38,
    V39 = 39,
    V40 = 40,
    V41 = 41,
    V42 = 42,
    V43 = 43,
    V44 = 44,
    V45 = 45,
    V46 = 46,
    V47 = 47,
    V48 = 48,
    V49 = 49,
    V50 = 50,
    V51 = 51,
    V52 = 52,
    V53 = 53,
    V54 = 54,
    V55 = 55,
    V56 = 56,
    V57 = 57,
    V58 = 58,
    V59 = 59,
    V60 = 60,
    V61 = 61,
    V62 = 62,
    V63 = 63,
    V64 = 64,
    V65 = 65,
    V66 = 66,
    V67 = 67,
    V68 = 68,
    V69 = 69,
    V70 = 70,
    V71 = 71,
    V72 = 72,
    V73 = 73,
    V74 = 74,
    V75 = 75,
    V76 = 76,
    V77 = 77,
    V78 = 78,
    V79 = 79,
    V80 = 80,
    V81 = 81,
    V82 = 82,
    V83 = 83,
    V84 = 84,
    V85 = 85,
    V86 = 86,
    V87 = 87,
    V88 = 88,
    V89 = 89,
    V90 = 90,
    V91 = 91,
    V92 = 92,
    V93 = 93,
    V94 = 94,
    V95 = 95,
    V96 = 96,
    V97 = 97,
    V98 = 98,
    V99 = 99,
    V100 = 100,
    V101 = 101,
    V102 = 102,
    V103 = 103,
    V104 = 104,
    V105 = 105,
    V106 = 106,
    V107 = 107,
    V108 = 108,
    V109 = 109,
    V110 = 110,
    V111 = 111,
    V112 = 112,
    V113 = 113,
    V114 = 114,
    V115 = 115,
    V116 = 116,
    V117 = 117,
    V118 = 118,
    V119 = 119,
    V120 = 120,
    V121 = 121,
    V122 = 122,
    V123 = 123,
    V124 = 124,
    V125 = 125,
    V126 = 126,
    V127 = 127,
    V128 = 128,
}

impl From<GeneralVersion> for FixedVersion {
    #[inline(always)]
    fn from(value: GeneralVersion) -> Self {
        FixedVersion::new(value as u64, 0, 0)
    }
}

#[cfg(feature = "rkyv")]
impl From<&ArchivedGeneralVersion> for FixedVersion {
    #[inline(always)]
    fn from(value: &ArchivedGeneralVersion) -> Self {
        FixedVersion::new(*value as u64, 0, 0)
    }
}

#[cfg(feature = "rkyv")]
impl ::rkyv::with::ArchiveWith<FixedVersionTag> for Box<GeneralVersion> {
    type Archived = rkyv::Archived<Box<GeneralVersion>>;

    type Resolver = rkyv::boxed::BoxResolver;

    fn resolve_with(
        field: &FixedVersionTag,
        resolver: Self::Resolver,
        out: rkyv::Place<Self::Archived>,
    ) {
        rkyv::boxed::ArchivedBox::resolve_from_ref(&field.version, resolver, out)
    }
}

#[derive(Debug)]
pub struct TypeIdMismatchError {
    pub deser_id: FixedId,
    pub expect_id: FixedId,
}

impl fmt::Display for TypeIdMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type id mismatch, deser_id:{}, expect_id:{}",
            self.deser_id, self.expect_id
        )
    }
}

impl core::error::Error for TypeIdMismatchError {}

#[derive(Debug)]
pub struct VersionTooNewError {
    pub deser_ver: u64,
    pub current_max_ver: u64,
}

impl fmt::Display for VersionTooNewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "version too new, current_max:{}, deser_ver:{}",
            self.current_max_ver, self.deser_ver
        )
    }
}

impl core::error::Error for VersionTooNewError {}
