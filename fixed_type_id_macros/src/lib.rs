mod fixed_type_id;

/// Macro to generate a unique id for trait object type or a general type.
///
/// This macro generates unique type IDs that can be used for type identification and casting.
/// It supports both trait objects and concrete types, with optional version or file-based uniqueness.
///
/// For types with generic parameters, you should use **specific** type parameters when you're using this macro
/// to define [`FixedTypeId`] for them.
///
/// # Examples
///
/// Basic usage with trait objects:
/// ```
/// # mod some {
/// use fixed_type_id::fixed_type_id;
/// use fixed_type_id::{FixedId, FixedTypeId, FixedVersion};
///
/// pub trait MyTrait {}
///
/// fixed_type_id! {
///     dyn MyTrait
/// }
/// # }
/// ```
///
/// With version-based uniqueness (commonly used pattern):
/// ```
///
/// mod my_crate {
///     mod api {
///         use fixed_type_id::fixed_type_id;
///         use fixed_type_id::{FixedId, FixedTypeId, FixedVersion};
///         pub trait MyTrait {}
///         fixed_type_id! {
///             #[FixedTypeIdVersion((0,1,0))]
///             dyn my_crate::api::MyTrait
///         }
///     }
/// }
/// ```
///
/// With file-based uniqueness and filename:
/// ```
/// # mod some {
/// use fixed_type_id::fixed_type_id;
/// use fixed_type_id::{FixedId, FixedTypeId, FixedVersion};
///
/// pub trait MyTrait {}
///
/// fixed_type_id! {
///     #[FixedTypeIdFile("ids.toml")]
///     dyn MyTrait
/// }
/// # }
/// ```
///
/// Real-world example from some APIs:
/// ```ignore,no_compile
/// use fixed_type_id::fixed_type_id;
/// use fixed_type_id::{FixedId, FixedTypeId, FixedVersion};
///
/// fixed_type_id! {
///     #[FixedTypeIdVersion((1,0,0))]
///     dyn bubble_core::api::my_api::MyApi
/// }
/// ```
#[proc_macro]
pub fn fixed_type_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fixed_type_id::fixed_type_id_impl(input)
}

/// Macro to generate a unique id for a trait object type without version.
///
/// For types which version is managed externally, or want to use semantic versioning.
#[proc_macro]
pub fn fixed_type_id_without_version_hash(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    fixed_type_id::fixed_type_id_impl_without_version_hash_in_type(input)
}

/// Macro to generate a random unique id.
#[proc_macro]
pub fn random_fixed_type_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fixed_type_id::random_fixed_type_id_impl(input)
}
