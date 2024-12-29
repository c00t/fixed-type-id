//! rkyv has some limitations,
//! [`usize`] can't be put inside types with automatically `rkyv(comapre(PartialEq))`.
//! [`String`] can't be put inside types with automatically `rkyv(derive(Clone))` for generated archived type.
//! So currently `Clone` and `Copy` are not derived for archived type, you should implement them yourself.

use fixed_revision::{
    access_tag_rkyv, FixedTypeIdTag, FixedTypeIdTagged, TypeIdMismatchError, VersionTooNewError,
};
use fixed_revision_macros::revisioned;
use fixed_type_id::{type_id, type_name, type_version};

use fixed_type_id::prelude::*;

use rkyv::{from_bytes, rancor::Error, to_bytes, Archive, Deserialize, Serialize};

#[revisioned(
    revision = 3,
    fixed_id_prefix = "fixed_revision_macros::tests",
    rkyv_support
)]
#[derive(Debug, PartialEq, Clone)]
pub enum TestEnum {
    Zero,
    #[revision(end = 2)]
    One(u32),
    #[revision(start = 2)]
    Two(u64),
    #[revision(start = 2)]
    Three {
        a: i64,
        #[revision(end = 3)]
        b: f32,
        #[revision(start = 3)]
        c: f64,
        #[revision(start = 3)]
        d: String,
    },
    #[revision(start = 1, end = 2)]
    Four(i32),
    #[revision(start = 2, end = 3)]
    Four,
    #[revision(start = 3)]
    Four(u32),
    Five(#[revision(end = 3)] u64, #[revision(start = 3)] i64),
}

#[revisioned(
    revision = 1,
    fixed_id_prefix = "fixed_revision_macros::tests",
    rkyv_support
)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TestUnit;

#[revisioned(
    revision = 2,
    fixed_id_prefix = "fixed_revision_macros::tests",
    rkyv_support
)]
#[derive(Debug, PartialEq, Clone)]
pub struct TestTuple2(
    #[revision(end = 2)] pub Vec<i64>,
    #[revision(start = 2)] pub Vec<f64>,
);

impl TestTuple2 {}

#[revisioned(
    revision = 4,
    fixed_id_prefix = "fixed_revision_macros::tests",
    rkyv_support
)]
#[derive(Debug, PartialEq, Clone)]
pub struct Tester4 {
    usize_1: u32,
    #[revision(start = 2, end = 4)]
    isize_1: i32,
    u16_1: u16,
    #[revision(end = 3)]
    u64_1: u64,
    i8_1: i8,
    #[revision(start = 2, end = 3)]
    i16_1: i16,
    i32_1: i32,
    #[revision(start = 2, end = 3)]
    i64_1: i64,
    f32_1: f32,
    f64_1: f64,
    char_1: char,
    #[revision(start = 2)]
    bool_1: bool,
    string_1: String,
    enum_1: TestEnum,
    option_1: Option<u8>,
    #[revision(start = 3, end = 4)]
    vec_1: Vec<char>,
    #[revision(start = 3)]
    unit_1: TestUnit,
    #[revision(start = 3)]
    tuple_1: TestTuple2_1,
    #[allow(clippy::box_collection)] // we want to explicitly test Box
    box_1: Box<String>,
    #[revision(start = 3)]
    wrapping_1: u32,
}

#[test]
fn basic_gen() {
    assert_eq!(type_id::<TestEnum_1>(), TestEnum::TYPE_ID);
    assert_eq!(
        type_version::<TestEnum_1>(),
        Into::<FixedVersion>::into((1, 0, 0))
    );
    assert_eq!(
        type_name::<TestEnum_1>(),
        "fixed_revision_macros::tests::TestEnum_1"
    );

    assert_eq!(type_id::<TestEnum_2>(), TestEnum::TYPE_ID);
    assert_eq!(
        type_version::<TestEnum_2>(),
        Into::<FixedVersion>::into((2, 0, 0))
    );
    assert_eq!(
        type_name::<TestEnum_2>(),
        "fixed_revision_macros::tests::TestEnum_2"
    );

    assert_eq!(
        type_version::<TestEnum_3>(),
        Into::<FixedVersion>::into((3, 0, 0))
    );
    assert_eq!(type_id::<TestEnum_3>(), TestEnum::TYPE_ID);
    assert_eq!(
        type_name::<TestEnum_3>(),
        "fixed_revision_macros::tests::TestEnum_3"
    );

    assert_eq!(
        type_version::<TestEnum>(),
        Into::<FixedVersion>::into((3, 0, 0))
    );
    assert_eq!(type_id::<TestEnum>(), TestEnum::TYPE_ID);
    assert_eq!(
        type_name::<TestEnum>(),
        "fixed_revision_macros::tests::TestEnum"
    );

    assert_eq!(
        type_version::<TestUnit>(),
        Into::<FixedVersion>::into((1, 0, 0))
    );
    assert_eq!(
        type_version::<TestTuple2>(),
        Into::<FixedVersion>::into((2, 0, 0))
    );
    assert_eq!(
        type_version::<Tester4>(),
        Into::<FixedVersion>::into((4, 0, 0))
    );

    let test_enum = TestEnum::V3(TestEnum_3::Zero);

    let test_enum_rkyv_aligned_vec = test_enum.serialize_rkyv::<Error>().unwrap();
    let test_enum_archived = TestEnum::access_rkyv(&test_enum_rkyv_aligned_vec).unwrap();
    assert_eq!(*test_enum_archived, test_enum);
    let (id, version) = access_tag_rkyv(&test_enum_rkyv_aligned_vec).unwrap();
    assert_eq!(id, TestEnum::TYPE_ID);
    assert_eq!(version, TestEnum_3::TYPE_VERSION);
    let test_enum_deser = TestEnum::deserialize_rkyv(&test_enum_rkyv_aligned_vec).unwrap();
    assert_eq!(test_enum_deser, test_enum);
}
