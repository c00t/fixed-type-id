use kollect::{LinearMap, LinearSet, OrderedMap, OrderedSet, UnorderedMap, UnorderedSet};

use crate::prelude::*;

fixed_type_id!(
    kollect::LinearMap<K:FixedTypeId,V:FixedTypeId>;
    kollect::LinearSet<K:FixedTypeId>;
);

impl<K: FixedTypeId, V: FixedTypeId, S> FixedTypeId for OrderedMap<K, V, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, S> FixedTypeId for OrderedSet<K, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, V: FixedTypeId, S> ConstTypeName for OrderedMap<K, V, S> {
    const RAW_SLICE: &[&str] = &["kollect::OrderedMap<", K::TYPE_NAME, ",", V::TYPE_NAME, ">"];
}

impl<K: FixedTypeId, S> ConstTypeName for OrderedSet<K, S> {
    const RAW_SLICE: &[&str] = &["kollect::OrderedSet<", K::TYPE_NAME, ">"];
}

impl<K: FixedTypeId, V: FixedTypeId, S> FixedTypeId for UnorderedMap<K, V, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, S> FixedTypeId for UnorderedSet<K, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, V: FixedTypeId, S> ConstTypeName for UnorderedMap<K, V, S> {
    const RAW_SLICE: &[&str] = &[
        "kollect::UnorderedMap<",
        K::TYPE_NAME,
        ",",
        V::TYPE_NAME,
        ">",
    ];
}

impl<K: FixedTypeId, S> ConstTypeName for UnorderedSet<K, S> {
    const RAW_SLICE: &[&str] = &["kollect::UnorderedSet<", K::TYPE_NAME, ">"];
}
