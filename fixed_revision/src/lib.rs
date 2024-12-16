mod remote_impl;

use fixed_type_id::{type_version, FixedTypeId, FixedVersion};

pub trait RevisionedSerializer: Sized {}

pub trait RevisionedDeserializer<'de>: Sized {
    type Error;
    fn deserialize_versiontag<T: Revisioned>(self) -> Result<Option<FixedVersion>, Self::Error>;
}

pub trait Revisioned: FixedTypeId {
    // /// Serialize with a Serializer
    // fn serialize_revisioned<S: RevisionedSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
    // /// Deserialize with a Deserializer
    // fn deserialize_revisioned<D, E>(deserializer: D) -> Result<Self, E>
    // where
    //     D: for<'de> RevisionedDeserializer<'de, Error = E>,
    //     Self: Sized;
    /// It's a type that won't be versioned, which means it's ty_version will be (0,0,0)
    #[inline(always)]
    fn no_version() -> bool {
        if type_version::<Self>() == FixedVersion::new(0, 0, 0) {
            true
        } else {
            false
        }
    }
}
