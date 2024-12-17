mod remote_impl;

use fixed_type_id::{type_version, FixedTypeId, FixedVersion};

/// [`Revisioned`] trait indicates that the type enable schema evolution.
///
/// Typically, this trait is implemented automatically by attribute macro, and the type must implement [`FixedTypeId`] trait.
pub trait Revisioned: FixedTypeId {
    /// Check if the type is a no-version type.
    ///
    /// A no-version type is a type that won't be versioned, which means it's ty_version will be `(0,0,0)`.
    /// If you enable `specialization` feature of the [`fixed_type_id`], then [`FixedTypeId`] is implemented for all types,
    /// and you can use this method to check if the type is a no-version type.
    ///
    #[inline(always)]
    fn no_version() -> bool {
        type_version::<Self>() == FixedVersion::new(0, 0, 0)
    }
}
