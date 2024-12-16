#![feature(prelude_import)]
#![allow(incomplete_features)]
#![feature(specialization)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use fixed_type_id::{fixed_type_id, type_version};
use fixed_type_id::{FixedId, FixedTypeId, FixedVersion, FixedVersioned};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[serde(into = "FixedVersioned<TestEnum>")]
pub enum TestEnum {
    Zero,
    One(u32),
    Two(u64),
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for TestEnum {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            _serde::Serialize::serialize(
                &_serde::__private::Into::<
                    FixedVersioned<TestEnum>,
                >::into(_serde::__private::Clone::clone(self)),
                __serializer,
            )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for TestEnum {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            TestEnum::Zero => ::core::fmt::Formatter::write_str(f, "Zero"),
            TestEnum::One(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "One", &__self_0)
            }
            TestEnum::Two(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Two", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for TestEnum {
    #[inline]
    fn clone(&self) -> TestEnum {
        match self {
            TestEnum::Zero => TestEnum::Zero,
            TestEnum::One(__self_0) => {
                TestEnum::One(::core::clone::Clone::clone(__self_0))
            }
            TestEnum::Two(__self_0) => {
                TestEnum::Two(::core::clone::Clone::clone(__self_0))
            }
        }
    }
}
impl Into<FixedVersioned<TestEnum>> for TestEnum {
    fn into(self) -> FixedVersioned<TestEnum> {
        FixedVersioned {
            version: type_version::<TestEnum>(),
            data: self,
        }
    }
}
impl self::FixedTypeId for TestEnum {
    const TYPE_NAME: &'static str = "TestEnum";
    const TYPE_ID: self::FixedId = self::FixedId(4078985092022392563u64 as u64);
    const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(0u64, 1u64, 0u64);
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
#[serde(from = "FixedVersioned<TestStruct>", into = "FixedVersioned<TestStruct>")]
pub struct TestStruct {
    pub a: u32,
    pub b: u64,
    pub c: u8,
    pub d: Inner,
    pub e: TestEnum,
    pub f: UnitStruct,
    pub g: f32,
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for TestStruct {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            _serde::Serialize::serialize(
                &_serde::__private::Into::<
                    FixedVersioned<TestStruct>,
                >::into(_serde::__private::Clone::clone(self)),
                __serializer,
            )
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for TestStruct {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            _serde::__private::Result::map(
                <FixedVersioned<
                    TestStruct,
                > as _serde::Deserialize>::deserialize(__deserializer),
                _serde::__private::From::from,
            )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for TestStruct {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &["a", "b", "c", "d", "e", "f", "g"];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.a,
            &self.b,
            &self.c,
            &self.d,
            &self.e,
            &self.f,
            &&self.g,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "TestStruct",
            names,
            values,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for TestStruct {
    #[inline]
    fn clone(&self) -> TestStruct {
        TestStruct {
            a: ::core::clone::Clone::clone(&self.a),
            b: ::core::clone::Clone::clone(&self.b),
            c: ::core::clone::Clone::clone(&self.c),
            d: ::core::clone::Clone::clone(&self.d),
            e: ::core::clone::Clone::clone(&self.e),
            f: ::core::clone::Clone::clone(&self.f),
            g: ::core::clone::Clone::clone(&self.g),
        }
    }
}
impl self::FixedTypeId for TestStruct {
    const TYPE_NAME: &'static str = "TestStruct";
    const TYPE_ID: self::FixedId = self::FixedId(14113608197088431024u64 as u64);
    const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(0u64, 1u64, 0u64);
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
impl From<FixedVersioned<TestStruct>> for TestStruct {
    fn from(val: FixedVersioned<TestStruct>) -> Self {
        val.data
    }
}
impl Into<FixedVersioned<TestStruct>> for TestStruct {
    fn into(self) -> FixedVersioned<TestStruct> {
        FixedVersioned {
            version: type_version::<TestStruct>(),
            data: self,
        }
    }
}
#[serde(from = "FixedVersioned<Inner>", into = "FixedVersioned<Inner>")]
pub struct Inner {
    pub a: u32,
    pub b: u64,
    pub c: String,
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Inner {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            _serde::Serialize::serialize(
                &_serde::__private::Into::<
                    FixedVersioned<Inner>,
                >::into(_serde::__private::Clone::clone(self)),
                __serializer,
            )
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Inner {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            _serde::__private::Result::map(
                <FixedVersioned<
                    Inner,
                > as _serde::Deserialize>::deserialize(__deserializer),
                _serde::__private::From::from,
            )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for Inner {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Inner",
            "a",
            &self.a,
            "b",
            &self.b,
            "c",
            &&self.c,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Inner {
    #[inline]
    fn clone(&self) -> Inner {
        Inner {
            a: ::core::clone::Clone::clone(&self.a),
            b: ::core::clone::Clone::clone(&self.b),
            c: ::core::clone::Clone::clone(&self.c),
        }
    }
}
impl From<FixedVersioned<Inner>> for Inner {
    fn from(val: FixedVersioned<Inner>) -> Self {
        val.data
    }
}
impl Into<FixedVersioned<Inner>> for Inner {
    fn into(self) -> FixedVersioned<Inner> {
        FixedVersioned {
            version: type_version::<Inner>(),
            data: self,
        }
    }
}
impl self::FixedTypeId for Inner {
    const TYPE_NAME: &'static str = "Inner";
    const TYPE_ID: self::FixedId = self::FixedId(6653622757496992577u64 as u64);
    const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(0u64, 1u64, 0u64);
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
#[serde(from = "FixedVersioned<UnitStruct>", into = "FixedVersioned<UnitStruct>")]
pub struct UnitStruct;
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for UnitStruct {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            _serde::Serialize::serialize(
                &_serde::__private::Into::<
                    FixedVersioned<UnitStruct>,
                >::into(_serde::__private::Clone::clone(self)),
                __serializer,
            )
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for UnitStruct {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            _serde::__private::Result::map(
                <FixedVersioned<
                    UnitStruct,
                > as _serde::Deserialize>::deserialize(__deserializer),
                _serde::__private::From::from,
            )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for UnitStruct {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "UnitStruct")
    }
}
#[automatically_derived]
impl ::core::clone::Clone for UnitStruct {
    #[inline]
    fn clone(&self) -> UnitStruct {
        UnitStruct
    }
}
impl From<FixedVersioned<UnitStruct>> for UnitStruct {
    fn from(val: FixedVersioned<UnitStruct>) -> Self {
        val.data
    }
}
impl Into<FixedVersioned<UnitStruct>> for UnitStruct {
    fn into(self) -> FixedVersioned<UnitStruct> {
        FixedVersioned {
            version: type_version::<UnitStruct>(),
            data: self,
        }
    }
}
impl self::FixedTypeId for UnitStruct {
    const TYPE_NAME: &'static str = "UnitStruct";
    const TYPE_ID: self::FixedId = self::FixedId(7891383077726075075u64 as u64);
    const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(0u64, 1u64, 0u64);
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
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "smoke"]
#[doc(hidden)]
pub const smoke: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("smoke"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "fixed_revision\\tests\\smoke.rs",
        start_line: 127usize,
        start_col: 4usize,
        end_line: 127usize,
        end_col: 9usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(#[coverage(off)] || test::assert_test_result(smoke())),
};
fn smoke() {
    let test_enum = TestEnum::One(1);
    let ver: FixedVersioned<TestEnum> = test_enum.clone().into();
    let test_enum2: TestEnum = test_enum.clone();
    {
        ::std::io::_print(
            format_args!("{0}\n", serde_json::to_string(& test_enum2).unwrap()),
        );
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", ver));
    };
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
    match (&type_version::<UnitStruct>(), &FixedVersion::new(0, 1, 0)) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let ron = ron::to_string(&test_struct).unwrap();
    {
        ::std::io::_eprint(format_args!("{0}\n", ron));
    };
    let deserialized: TestStruct = ron::de::from_str(&ron).unwrap();
    {
        ::std::io::_print(format_args!("{0:?}\n", deserialized));
    };
    {
        #[cold]
        #[track_caller]
        #[inline(never)]
        const fn panic_cold_explicit() -> ! {
            ::core::panicking::panic_explicit()
        }
        panic_cold_explicit();
    }
}
#[rustc_main]
#[coverage(off)]
#[doc(hidden)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&smoke])
}
