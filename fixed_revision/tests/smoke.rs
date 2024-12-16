#![allow(incomplete_features)]
#![feature(specialization)]

use fixed_type_id::{fixed_type_id, type_version};
use fixed_type_id::{FixedId, FixedTypeId, FixedVersion, FixedVersioned};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(
    into = "FixedVersioned<TestEnumDef,TestEnum>",
    from = "FixedVersioned<TestEnumDef,TestEnum>"
)] // need a intermediate type to avoid infinite recursion, the serde attr should be copy to the def type.
pub enum TestEnum {
    Zero,
    One(u32),
    Two(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TestEnumDef {
    Zero,
    One(u32),
    Two(u64),
}

impl From<TestEnum> for TestEnumDef {
    #[inline(always)]
    fn from(val: TestEnum) -> Self {
        match val {
            TestEnum::Zero => TestEnumDef::Zero,
            TestEnum::One(v) => TestEnumDef::One(v),
            TestEnum::Two(v) => TestEnumDef::Two(v),
        }
    }
}

impl From<TestEnumDef> for TestEnum {
    #[inline(always)]
    fn from(val: TestEnumDef) -> Self {
        match val {
            TestEnumDef::Zero => TestEnum::Zero,
            TestEnumDef::One(v) => TestEnum::One(v),
            TestEnumDef::Two(v) => TestEnum::Two(v),
        }
    }
}

impl From<FixedVersioned<TestEnumDef, TestEnum>> for TestEnum {
    #[inline(always)]
    fn from(val: FixedVersioned<TestEnumDef, TestEnum>) -> Self {
        val.data.into()
    }
}

fixed_type_id! {
    #[FixedTypeIdVersion((0,1,0))]
    TestEnum;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(
    from = "FixedVersioned<TestStructDef,TestStruct>",
    into = "FixedVersioned<TestStructDef,TestStruct>"
)]
pub struct TestStruct {
    pub a: u32,
    pub b: u64,
    pub c: u8,
    pub d: Inner,
    pub e: TestEnum,
    pub f: UnitStruct,
    pub g: f32,
    // for u128, need to use other helper functions
    // #[serde(serialize_with = "ss_test")]
    // #[serde(deserialize_with = "sd_test")]
    // pub h: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestStructDef {
    pub a: u32,
    pub b: u64,
    pub c: u8,
    pub d: Inner,
    pub e: TestEnum,
    pub f: UnitStruct,
    pub g: f32,
}

fixed_type_id! {
    #[FixedTypeIdVersion((0,1,0))]
    TestStruct;
}

impl From<TestStructDef> for TestStruct {
    #[inline(always)]
    fn from(val: TestStructDef) -> Self {
        TestStruct {
            a: val.a,
            b: val.b,
            c: val.c,
            d: val.d,
            e: val.e,
            f: val.f,
            g: val.g,
        }
    }
}

impl From<TestStruct> for TestStructDef {
    #[inline(always)]
    fn from(val: TestStruct) -> Self {
        TestStructDef {
            a: val.a,
            b: val.b,
            c: val.c,
            d: val.d,
            e: val.e,
            f: val.f,
            g: val.g,
        }
    }
}

impl From<FixedVersioned<TestStructDef, TestStruct>> for TestStruct {
    #[inline(always)]
    fn from(val: FixedVersioned<TestStructDef, TestStruct>) -> Self {
        val.data.into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(
    from = "FixedVersioned<InnerDef,Inner>",
    into = "FixedVersioned<InnerDef,Inner>"
)]
pub struct Inner {
    pub a: u32,
    pub b: u64,
    pub c: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InnerDef {
    pub a: u32,
    pub b: u64,
    pub c: String,
}

impl From<InnerDef> for Inner {
    #[inline(always)]
    fn from(val: InnerDef) -> Self {
        Inner {
            a: val.a,
            b: val.b,
            c: val.c,
        }
    }
}

impl From<Inner> for InnerDef {
    #[inline(always)]
    fn from(val: Inner) -> Self {
        InnerDef {
            a: val.a,
            b: val.b,
            c: val.c,
        }
    }
}

impl From<FixedVersioned<InnerDef, Inner>> for Inner {
    #[inline(always)]
    fn from(val: FixedVersioned<InnerDef, Inner>) -> Self {
        val.data.into()
    }
}

fixed_type_id! {
    #[FixedTypeIdVersion((0,1,0))]
    Inner;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(
    from = "FixedVersioned<UnitStructDef,UnitStruct>",
    into = "FixedVersioned<UnitStructDef,UnitStruct>"
)]
pub struct UnitStruct;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnitStructDef;

impl From<UnitStructDef> for UnitStruct {
    #[inline(always)]
    fn from(_val: UnitStructDef) -> Self {
        UnitStruct
    }
}

impl From<UnitStruct> for UnitStructDef {
    #[inline(always)]
    fn from(_val: UnitStruct) -> Self {
        UnitStructDef
    }
}

impl From<FixedVersioned<UnitStructDef, UnitStruct>> for UnitStruct {
    #[inline(always)]
    fn from(val: FixedVersioned<UnitStructDef, UnitStruct>) -> Self {
        val.data.into()
    }
}

fixed_type_id! {
    #[FixedTypeIdVersion((0,1,0))]
    UnitStruct;
}

#[test]
fn smoke() {
    let test_enum = TestEnum::One(1);
    let inner = Inner {
        a: 4,
        b: 5,
        c: "6".to_string(),
    };
    let test_struct = TestStruct {
        a: 1,
        b: 2,
        c: 3,
        d: inner,
        e: test_enum,
        f: UnitStruct,
        g: 1.0,
    };
    assert_eq!(type_version::<UnitStruct>(), FixedVersion::new(0, 1, 0));

    let mut buf = Vec::new();
    let ser = &mut serde_json::Serializer::pretty(&mut buf);
    // let ser = SerdeRevisionSerializer::new(ser);
    test_struct.serialize(ser).unwrap();
    eprintln!("{}", std::str::from_utf8(&buf).unwrap());
    let deserialized: TestStruct = serde_json::from_slice(&buf).unwrap();
    println!("{:?}", deserialized);

    // // hjson
    // let hjson = serde_hjson::to_string(&test_struct).unwrap();
    // eprintln!("{}", hjson);
    // let deserialized: TestStruct = serde_hjson::from_str(&hjson).unwrap();
    // // println!("{:?}", deserialized);

    // // toml
    // let toml = toml::to_string(&test_struct).unwrap();
    // eprintln!("{}", toml);
    // let deserialized: TestStruct = toml::from_str(&toml).unwrap();
    // // println!("{:?}", deserialized);

    // ron
    let ron = ron::ser::to_string_pretty(&test_struct, ron::ser::PrettyConfig::new()).unwrap();
    eprintln!("{}", ron);
    let deserialized: TestStruct = ron::de::from_str(&ron).unwrap();
    println!("{:?}", deserialized);

    panic!()
}
