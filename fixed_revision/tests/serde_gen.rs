use std::num::Wrapping;

use fixed_revision::{
    get_tag_serde, FixedTypeIdTag, FixedTypeIdTagged, TypeIdMismatchError, VersionTooNewError,
};
use fixed_revision_macros::revisioned;
use fixed_type_id::{type_id, type_name, type_version};

use fixed_type_id::prelude::*;

#[revisioned(
    revision = 3,
    fixed_id_prefix = "fixed_revision_macros::tests",
    serde_support
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
    serde_support
)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TestUnit;

#[revisioned(
    revision = 2,
    fixed_id_prefix = "fixed_revision_macros::tests",
    serde_support
)]
#[derive(Debug, PartialEq, Clone)]
pub struct TestTuple2(
    #[revision(end = 2)] pub Vec<i64>,
    #[revision(start = 2)] pub Vec<f64>,
);

impl TestTuple2 {}

// Used to serialize the struct at revision 3
#[revisioned(
    revision = 3,
    fixed_id_prefix = "fixed_revision_macros::tests",
    serde_support
)]
#[derive(Debug, PartialEq, Clone)]
pub struct Tester3 {
    #[revision(start = 3)] // used to force the version to 3
    usize_1: usize,
    isize_1: isize,
    u16_1: u16,
    i8_1: i8,
    i32_1: i32,
    f32_1: f32,
    f64_1: f64,
    char_1: char,
    bool_1: bool,
    string_1: String,
    enum_1: TestEnum,
    option_1: Option<u8>,
    vec_1: Vec<char>,
    unit_1: TestUnit,
    tuple_1: TestTuple2_1,
    #[allow(clippy::box_collection)] // we want to explicitly test Box
    box_1: Box<String>,
    wrapping_1: Wrapping<u32>,
}

#[revisioned(
    revision = 4,
    fixed_id_prefix = "fixed_revision_macros::tests",
    serde_support
)]
#[derive(Debug, PartialEq, Clone)]
pub struct Tester4 {
    usize_1: usize,
    #[revision(start = 2, end = 4)]
    isize_1: isize,
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
    wrapping_1: Wrapping<u32>,
}

// #[revisioned(
//     revision = 1
// )]
// pub struct GenericTest<T> {
//     some: T,
//     u32_1: u32,
// }

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
        TestEnum::max_type_version(),
        Into::<FixedVersion>::into((3, 0, 0))
    );
    assert_eq!(type_id::<TestEnum>(), TestEnum::TYPE_ID);
    assert_eq!(
        type_name::<TestEnum>(),
        "fixed_revision_macros::tests::TestEnum"
    );

    assert_eq!(
        TestUnit::max_type_version(),
        Into::<FixedVersion>::into((1, 0, 0))
    );
    assert_eq!(
        TestTuple2::max_type_version(),
        Into::<FixedVersion>::into((2, 0, 0))
    );
    assert_eq!(
        Tester4::max_type_version(),
        Into::<FixedVersion>::into((4, 0, 0))
    );

    let test_enum = TestEnum::V3(TestEnum_3::Zero);
    // let test_enum_str =
    //     ron::ser::to_string_pretty(&test_enum, ron::ser::PrettyConfig::default()).unwrap();
    let test_enum_str = test_enum
        .serialize_serde(|value| {
            ron::ser::to_string_pretty(&value, ron::ser::PrettyConfig::default())
        })
        .unwrap();
    eprintln!("{test_enum_str}");
    let test_enum_deser =
        TestEnum::deserialize_serde(|| ron::de::Deserializer::from_str(&test_enum_str).unwrap())
            .unwrap();
    assert_eq!(test_enum_deser, test_enum);
    // test raw string, note that type_id is the same across all versions
    let test_enum_str_edited = r#"
(
    type_id: (9386386583157998584),
    data: (
        version: V4,
        content: Six,
    ),
)
"#;
    let test_enum_edited_deser = TestEnum::deserialize_serde(|| {
        ron::de::Deserializer::from_str(&test_enum_str_edited).unwrap()
    });
    assert!(test_enum_edited_deser.is_err());
    assert_eq!(
        format!("{:?}", test_enum_edited_deser),
        "Err(Message(\"version too new, current_max:3, de_ver:4\"))"
    );

    let test_struct_v3_old = Tester4::V3(Tester4_3 {
        usize_1: 57918374,
        isize_1: 1234,
        u16_1: 1223,
        i8_1: 14,
        i32_1: -234234,
        f32_1: 1.0,
        f64_1: 2.0,
        char_1: 'x',
        bool_1: true,
        string_1: String::from("A test"),
        enum_1: test_enum.clone(),
        option_1: None,
        vec_1: vec!['a', 'b', 'c'],
        unit_1: TestUnit::V1(TestUnit_1),
        tuple_1: TestTuple2_1(vec![234324, 1234234]),
        box_1: Box::new(String::from("A test")),
        wrapping_1: Wrapping(1234),
    });
    let test_struct_v3_string_old = test_struct_v3_old
        .serialize_serde(|value| {
            ron::ser::to_string_pretty(&value, ron::ser::PrettyConfig::default())
        })
        .unwrap();
    // ron::ser::to_string_pretty(&test_struct_v3_old, ron::ser::PrettyConfig::default()).unwrap();
    let test_struct_v3_new = Tester4::V3(Tester4_3 {
        usize_1: 57918374,
        isize_1: 1234,
        u16_1: 1223,
        i8_1: 14,
        i32_1: -234234,
        f32_1: 1.0,
        f64_1: 2.0,
        char_1: 'x',
        bool_1: true,
        string_1: String::from("A test"),
        enum_1: test_enum.clone(),
        option_1: None,
        vec_1: vec!['a', 'b', 'c'],
        unit_1: TestUnit::V1(TestUnit_1),
        tuple_1: TestTuple2_1(vec![234324, 1234234]),
        box_1: Box::new(String::from("A test")),
        wrapping_1: Wrapping(1234),
    });
    let test_struct_v3_string = test_struct_v3_new
        .serialize_serde(|value| {
            ron::ser::to_string_pretty(&value, ron::ser::PrettyConfig::default())
        })
        .unwrap();
    // ron::ser::to_string_pretty(&test_struct_v3_new, ron::ser::PrettyConfig::default()).unwrap();
    assert_eq!(test_struct_v3_string, test_struct_v3_string_old);
    let test_struct_v3 = ron::from_str::<Tester4>(&test_struct_v3_string_old).unwrap();
    // let meta = ron::from_str::<FixedTypeIdTag>(&test_struct_v3_string_old).unwrap();
    let (id, version) =
        get_tag_serde(|| ron::de::Deserializer::from_str(&test_struct_v3_string_old).unwrap())
            .unwrap();
    eprintln!("{:?}", (id, version));
    assert_eq!(test_struct_v3, test_struct_v3_new);
    eprintln!("{}", test_struct_v3_string);
    assert_eq!(version.major, Tester4_3::TYPE_VERSION.major);
}
