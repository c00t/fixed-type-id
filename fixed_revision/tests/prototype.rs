//! Copy expand from macros
//!
use fixed_type_id::{FixedId, FixedTypeId, FixedVersion};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename(deserialize = "TestEnum"))]
#[allow(non_camel_case_types)]
pub enum TestEnum_1 {
    Zero,
    One(u32),
    Four(i32),
    Five(u64),
}

#[derive(Deserialize)]
#[serde(rename(deserialize = "TestEnum"))]
#[allow(non_camel_case_types)]
pub enum TestEnum_2 {
    Zero,
    Two(u64),
    Three { a: i64, b: f32 },
    Four,
    Five(u64),
}

#[allow(non_camel_case_types)]
pub enum TestEnum_3 {
    Zero,
    Two(u64),
    Three { a: i64, c: f64, d: String },
    Four(usize),
    Five(i64),
}

type TestEnum = TestEnum_3;
impl self::FixedTypeId for TestEnum {
    const TYPE_NAME: &'static str = "fixed_revision_macros::tests::TestEnum";
    const TYPE_ID: self::FixedId = self::FixedId(11385998469915967018u64 as u64);
    const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(3u64, 0u64, 0u64);
    #[inline]
    fn ty_name(&self) -> &'static str {
        Self::TYPE_NAME
    }
    #[inline]
    fn ty_id(&self) -> self::FixedId {
        Self::TYPE_ID
    }
    #[inline]
    fn ty_version(&self) -> self::FixedVersion {
        Self::TYPE_VERSION
    }
}
