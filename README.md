## Fixed Type Id

> Nightly toolchain required.

Make your types have a fixed type id and stable type name between different builds.

The crate provides a trait and a procedural macro. By implementing [`FixedTypeId`],
other crates can use [`FixedTypeId::ty_id()`], [`FixedTypeId::ty_name()`] and [`FixedTypeId::ty_version()`]
to get the type id, name and version about this type

It use [rapidhash](https://github.com/hoxxep/rapidhash) to hash the type name you provided, with/without version hashed into the id.
Then provide the hash as a fixed id for your type. So you can construct exact the same id from the same type name and version.

The purpose of this crate is to provide a fixed type id for simple types, which you may want to persist their metadata, like `u8`, `i16`, `f32`, `str`, `String`, `bool`, `(u8,i16,f32)`, `[u8; 3]`, `[i16; 3]`, `&[u8]`, etc. Or types frequently used in your structs, like `HashMap<K, V>`, `Vec<T>`, `Box<T>` etc.

It also support trait objects, which is used by [trait_cast_rs](https://github.com/c00t/trait_cast_rs) to
cast between different traits.

Note that the type name implemented by default for standard library types may be different from [`core::any::type_name`], you shouldn't compare it with [`FixedTypeId::ty_name()`].

Because of the orphan rule, if you want to implement [`FixedTypeId`] for types in other crates, you can submit a PR to add them.

### Usage

The example usage:

```rust
# #![cfg_attr(feature = "specialization", feature(specialization))]
use fixed_type_id::{FixedTypeId, FixedId, fixed_type_id, name_version_to_hash};
use std::hash::Hasher;

mod m {
    // the macro use `self::xxx` to import required items, I avoid to use `$crate` because it avoid to reexport items from this crate.
    use fixed_type_id::{FixedTypeId, FixedId, fixed_type_id, FixedVersion};
    pub trait Q {}
    pub trait W {}
    pub trait E<T> {}
    fixed_type_id!{
        // default to (0,0,0)
        #[FixedTypeIdVersion((0,1,0))]
        // #[FixedTypeIdFile("types.toml")]
        // no default, but when store into file, version will be dropped, so only use it for debug.
        dyn m::Q; // type name is "dyn m::Q", it only store the type name you provided, without modification.
        dyn W; // type name is "dyn W", though `W` is under `m` module, it still store "dyn W"
        dyn E<u8>; // type name is "dyn E<u8>"
        A; // type name is "A"
        B<u8>; // type name is "B<u8>"
    }
    pub struct A;
    pub struct B<T> {
    pub t: T
    }
    impl Q for A {}
}
use m::*;
assert_eq!(<dyn Q>::TYPE_ID.0, name_version_to_hash("dyn m::Q", &(0,1,0).into()));
assert_eq!(<dyn Q>::TYPE_NAME, "dyn m::Q");
assert_eq!(<A as FixedTypeId>::TYPE_VERSION, (0,1,0).into());
assert_eq!(<A as FixedTypeId>::TYPE_NAME, "A");
```

If you want to define for your generic struct, you can use [`implement_wrapper_fixed_type_id`]:

```rust
# #![cfg_attr(feature = "specialization", feature(specialization))]
use fixed_type_id::{FixedTypeId, FixedId, ConstTypeName, fstr_to_str, FixedVersion, implement_wrapper_fixed_type_id};
use std::ops::Add;

pub struct MyStruct<T: Add> {
    pub x: T,
}

implement_wrapper_fixed_type_id!{
    // You can define a typename with different module path, but it's not recommended.
    // Generally, you should define it in the same module as the struct for better readability.
    MyStruct<T:Add> => "SomeSpecialModule::MyStruct"
}

assert_eq!(<MyStruct<u8> as FixedTypeId>::TYPE_NAME, "SomeSpecialModule::MyStruct<u8>");
```

Also, you can define this trait yoursellf:

```rust
# #![cfg_attr(feature = "specialization", feature(specialization))]
use fixed_type_id::{FixedTypeId, FixedId, FixedVersion};
use rapidhash::rapidhash;

struct MyType;

impl FixedTypeId for MyType {
    const TYPE_NAME: &'static str = "MyType";
    // make this type id hash without version
    const TYPE_ID: FixedId = FixedId::from_type_name(Self::TYPE_NAME, None);
    const TYPE_VERSION: FixedVersion = FixedVersion::new(0, 0, 0);
}

assert_eq!(<MyType as FixedTypeId>::TYPE_NAME, "MyType");
assert_eq!(<MyType as FixedTypeId>::TYPE_ID.0, rapidhash::rapidhash("MyType".as_bytes()));
assert_eq!(<MyType as FixedTypeId>::TYPE_VERSION, (0,0,0).into());
```

There are standalone functions to get the type_name, type_id and type_version, like [`std::any::type_name`], [`std::any::type_id`]:

```rust
# #![cfg_attr(feature = "specialization", feature(specialization))]
use fixed_type_id::{type_name, type_id, type_version};
use fixed_type_id::{FixedTypeId, FixedId, FixedVersion};

struct MyType;

impl FixedTypeId for MyType {
    const TYPE_NAME: &'static str = "MyType";
    // make this type id hash without version
    const TYPE_ID: FixedId = FixedId::from_type_name(Self::TYPE_NAME, None);
    const TYPE_VERSION: FixedVersion = FixedVersion::new(0, 0, 0);
}

assert_eq!(type_name::<MyType>(), "MyType");
assert_eq!(type_id::<MyType>(), FixedId::from_type_name("MyType", None));
assert_eq!(type_version::<MyType>(), (0,0,0).into());
```

### Notes

#### Specialization

You can enable specialization by feature flag `specialization`, default is disabled. When enabled, it will implement [`FixedTypeId`] for all types, with dummy type info, not only the types you defined. Make it more like [`std::any::type_name`], [`std::any::type_id`].

Currently, the dummy type info is:

```plaintext
type_name: "NOT_IMPLEMENTED"
type_id: FixedId::from_type_name("NOT_IMPLEMENTED", Some(FixedVersion::new(0,0,0)))
type_version: FixedVersion::new(0,0,0)
```

When you are working with extern crates's generic functions, these dummy type info may be useful.

#### Version

For standard libraries types, the version is always `(0,0,0)`, in the future, it may be changed to rustc version you are using.

Currently, this crate implement [`FixedTypeId`] for these types:

- `()`, `Infallible`
- `T` for all primitive types, like `u8`, `i16`, `f32`, `str`, `String`, `bool` etc.
- `&T`, `&mut T` for all primitive types
- `Box<T>`, `Vec<T>`, `HashMap<K, V>`, `PhantomData<T>`, `NonZero<T>`, `fn(T) -> R`, `fn() -> R` for all generic types that implement [`FixedTypeId`]
- `(T,)`, `(T,U)`, `(T,U,V)`... `(T1,..., T16)` for all generic types that implement [`FixedTypeId`]
- `[T; N]` for all `T` that implement [`FixedTypeId`], but there is a limit for `N`, only `N <= 32` or some special numbers(`64`, `128`, `256`,..., `768`, `1024`, `2048`,...,`65535`) are supported, other numbers will just leave it as `N`. If you know how to generate `&str` for `const N: usize` in const context, you can submit a PR to add it.

#### Type Name Length

When you want to implement [`FixedTypeId`] for your types with generic parameters, you need to provide a dynamic generated `&str` as type name either 1. in const context or 2. store a `&[&str]` in const and then concat them at runtime.

If we choose to generate it in const context, because the only way i know to dynamically generate a `&str` in const context is to **fill a fixed length array `[u8;N]`**, and this array will be persisted into the binary, so the length of the type name is limited by **the binary size**. Currently, the length can be configured by feature flags `len64`, `len128` and `len256`, the default is `len128`, it means the max length of the type name is 128 bytes.

If we choose to store a `&[&str]` in const and then concat them at runtime, the return type of [`FixedTypeId::ty_name()`] will be `String`, it's different from the return type of [`core::any::type_name`], which is `&'static str`. It makes it difficult to just replace [`core::any::type_name`] with [`type_name`] or [`FixedTypeId::ty_name()`].

So currently we choose to generate it in const context.

#### Differences between `fixed_type_id`, `fixed_type_id_without_version_hash` and `random_fixed_type_id`

- `fixed_type_id`: Generate a unique id for the type, with a [`FixedId`] that [`rapidhash::rapidhash`] the name you provided,
  the version is also hashed into the [`FixedId`]. Version defaults to `(0,0,0)`, use `#[FixedTypeIdVersion((0,1,0))]` to change it.
  Use it when you want that different versions of your type have different ids.
- `fixed_type_id_without_version_hash`: Generate a unique id for the type, with a [`FixedId`] that [`rapidhash::rapidhash`] the name you provided,
  without version hashed into the [`FixedId`]. Use it when you want that different versions of your type have the same id.
- `random_fixed_type_id`: Generate a random id for the type, with a [`FixedId`] that random generated for each build.

All these macros can be used with:

- `#[FixedTypeIdVersion((x,y,z))]`: Set the version to `(x,y,z)`.
- `#[FixedTypeIdFile("filename.toml")]`: Store the type id into a file, so you can use it for debug, make sure the file already exists.
- `#[FixedTypeIdEqualTo("other_type")]`: Make the type id [`FixedId`] equal to `other_type`, so the two types have the same id, but different type names, and versions.

#### Erase Type Name

It can be configured by feature flag `erase_name`, default is disabled.

Currently, it only works for implementation of [`FixedTypeId`] by `fixed_type_id` and `fixed_type_id_without_version_hash`.
