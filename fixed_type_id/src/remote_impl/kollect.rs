use kollect::{LinearMap, LinearSet, OrderedMap, OrderedSet, UnorderedMap, UnorderedSet};

use crate::{fstr_to_str, implement_wrapper_fixed_type_id, ConstTypeName, FixedTypeId};

implement_wrapper_fixed_type_id!(
    LinearMap<K,V> => "kollect::LinearMap";
    LinearSet<K> => "kollect::LinearSet";
);

impl<K: FixedTypeId, V: FixedTypeId, S> FixedTypeId for OrderedMap<K, V, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, S> FixedTypeId for OrderedSet<K, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, V: FixedTypeId, S> ConstTypeName for OrderedMap<K, V, S> {
    const RAW_SLICE: &[&str] = &["OrderedMap<", K::TYPE_NAME, ",", V::TYPE_NAME, ">"];
}

impl<K: FixedTypeId, S> ConstTypeName for OrderedSet<K, S> {
    const RAW_SLICE: &[&str] = &["OrderedSet<", K::TYPE_NAME, ">"];
}

impl<K: FixedTypeId, V: FixedTypeId, S> FixedTypeId for UnorderedMap<K, V, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, S> FixedTypeId for UnorderedSet<K, S> {
    const TYPE_NAME: &'static str = fstr_to_str(&Self::TYPE_NAME_FSTR);
}

impl<K: FixedTypeId, V: FixedTypeId, S> ConstTypeName for UnorderedMap<K, V, S> {
    const RAW_SLICE: &[&str] = &["UnorderedMap<", K::TYPE_NAME, ",", V::TYPE_NAME, ">"];
}

impl<K: FixedTypeId, S> ConstTypeName for UnorderedSet<K, S> {
    const RAW_SLICE: &[&str] = &["UnorderedSet<", K::TYPE_NAME, ">"];
}
