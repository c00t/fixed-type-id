mod common;
mod de;
mod reexport;
mod ser;
mod validate_version;

use core::panic;
use std::{
    collections::{HashMap, HashSet},
    u16,
};

use common::{rkyv_compare_trait_fn, rkyv_derive_trait_fn};
use de::{DeserializeVisitor, EnumStructsVisitor};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use reexport::Reexport;
use ser::SerializeVisitor;
use syn::{spanned::Spanned, Ident};
use validate_version::ValidateRevision;

use crate::ast::{self, Direct, ItemOptions, Visit};

pub fn revision(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let attrs: Direct<ItemOptions> = syn::parse2(attr)?;
    let ast: ast::Item = syn::parse2(input)?;

    // Note(cupofc0t): I don't want to modify the revision type to (u64,u64,u64) or semver here,
    //   just convert it to distinct semver version, `(0,x++,0)` or `(x++,0,0)`(current) to make it compatible with fixed_type_id.
    let revision = match (ast.attrs.options.revision, attrs.0.revision) {
		(Some(x), None) | (None, Some(x)) => {
			x
		}
		(None,None) => {
			return Err(syn::Error::new(Span::call_site(),"Current revision not specified, please specify the current revision with `#[revisioned(revision = ..)]` "))
		}
		(Some(_),Some(_)) => {
			return Err(syn::Error::new(Span::call_site(),"Current revision specified twice"))
		}
	};

    let fixed_id_prefix = match (&ast.attrs.options.fixed_id_prefix, &attrs.0.fixed_id_prefix) {
        (Some(x), None) | (None, Some(x)) => Some(x.clone()),
        // you're allowed to not specify a fixed_id_prefix
        (None, None) => None,
        // you're not allowed to specify a fixed_id_prefix twice
        (Some(_), Some(_)) => {
            return Err(syn::Error::new(
                Span::call_site(),
                "fixed_id_prefix specified twice",
            ))
        }
    };

    let serde_support = match (&ast.attrs.options.serde_support, &attrs.0.serde_support) {
        (true, false) | (false, true) | (true, true) => true,
        (false, false) => false,
    };

    let rkyv_support = match (&ast.attrs.options.rkyv_support, &attrs.0.rkyv_support) {
        (true, false) | (false, true) | (true, true) => true,
        (false, false) => false,
    };

    let skip_fixedid_gen = match (&ast.attrs.options.rkyv_support, &attrs.0.rkyv_support) {
        (true, false) | (false, true) | (true, true) => true,
        (false, false) => false,
    };

    // Note(cupofc0t): Maybe increase the max revision to u32 or u64 in the future, but i don't think it's necessary for serializing usage.
    if revision > u16::MAX as usize {
        return Err(syn::Error::new(
            Span::call_site(),
            format_args!("Revision exceeded maximum supported value of {}", u16::MAX),
        ));
    }
    if revision == 0 {
        return Err(syn::Error::new(
            Span::call_site(),
            "Revision versions start at 1",
        ));
    }

    // Make sure that all used revisions are less or equal to the current revision.
    ValidateRevision(revision).visit_item(&ast)?;

    let mut enum_stream = TokenStream::new();

    let mut specific_derives = HashSet::new();
    let mut generics = HashMap::new();
    // deserialize implementation
    let reexport_revisions = (1..=revision)
        .map(|x| {
            let mut reexport = TokenStream::new();
            Reexport {
                revision,
                current: x,
                with_revision_suffix: true,
                fixed_id_prefix: fixed_id_prefix.clone(),
                stream: &mut reexport,
                enum_stream: Some(&mut enum_stream),
                specific_derives: &mut specific_derives,
                generics: &mut generics,
                serde_support,
                rkyv_support,
            }
            .visit_item(&ast)
            .unwrap();
            reexport
        })
        .collect::<Vec<_>>();

    let name = match ast.kind {
        ast::ItemKind::Enum(x) => x.name,
        ast::ItemKind::Struct(x) => x.name,
    };
    let vis = ast.vis;

    let revision = revision as u16;

    let attr_stream = {
        let mut stream = TokenStream::new();
        for attr in ast.attrs.other.iter() {
            attr.to_tokens(&mut stream);
        }

        if serde_support {
            let tagged_type_name = format!("FixedTypeIdTagged<{}Def>", name);
            stream.extend(quote! {
                #[derive(::serde::Serialize, ::serde::Deserialize)]
                #[serde(from = #tagged_type_name, into = #tagged_type_name)]
            });
        }

        if rkyv_support {
            // when rkyv, add specific derives to archived types.
            let compare_vec: Vec<_> = specific_derives
                .iter()
                .filter_map(|&s| {
                    if rkyv_compare_trait_fn(s) {
                        Some(Ident::new(s, name.span()))
                    } else {
                        None
                    }
                })
                .collect();
            let filterd_out_vec: Vec<_> = specific_derives
                .iter()
                .filter_map(|&s| {
                    if rkyv_derive_trait_fn(s) {
                        Some(Ident::new(s, name.span()))
                    } else {
                        None
                    }
                })
                .collect();
            stream.extend(quote! {
                #[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
                #[rkyv(compare(#(#compare_vec,)*), derive(#(#filterd_out_vec,)*))]
            });
        }

        stream
    };

    let attr_def_stream = {
        // if serde, we need def defs
        // but rkyv don't need to derive in it
        let mut stream = TokenStream::new();
        for attr in ast.attrs.other.iter() {
            attr.to_tokens(&mut stream);
        }

        if serde_support {
            stream.extend(quote! {
                #[derive(::serde::Serialize, ::serde::Deserialize)]
                #[serde(tag = "version", content = "content")]
            });
        }

        if rkyv_support {}

        stream
    };

    let enum_def_name_str = format!("{}Def", name);
    let enum_def_name = Ident::new(&enum_def_name_str, name.span());
    // let enum_name_str = format!("{}", name);
    let enum_def_from_stream = {
        let mut stream = TokenStream::new();
        (1..=revision).for_each(|x| {
            let variant_ident = Ident::new(&format!("V{}", x), name.span());
            let branch_stream = quote! {
                #enum_def_name::#variant_ident(v) => #name::#variant_ident(v),
            };
            stream.extend(quote! {
                #branch_stream
            });
        });
        stream
    };
    let enum_def_to_stream = {
        let mut stream = TokenStream::new();
        (1..=revision).for_each(|x| {
            let variant_ident = Ident::new(&format!("V{}", x), name.span());
            let branch_stream = quote! {
                #name::#variant_ident(v) => #enum_def_name::#variant_ident(v),
            };
            stream.extend(quote! {
                #branch_stream
            });
        });
        stream
    };

    let enum_alias = match (serde_support, rkyv_support) {
        (true, _) => quote! {
            #attr_stream
            #[repr(u16)]
            #vis enum #name {
                #enum_stream
            }

            /// Only used by serde internals.
            #attr_def_stream
            #[repr(u16)]
            enum #enum_def_name {
                #enum_stream
            }

            impl From<#name> for #enum_def_name {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    match val {
                        #enum_def_to_stream
                    }
                }
            }

            impl From<#enum_def_name> for #name {
                #[inline(always)]
                fn from(val: #enum_def_name) -> Self {
                    match val {
                        #enum_def_from_stream
                    }
                }
            }

            impl From<FixedTypeIdTagged<#enum_def_name>> for #name {
                #[inline(always)]
                fn from(val: FixedTypeIdTagged<#enum_def_name>) -> Self {
                    val.data.into()
                }
            }

            impl From<#name> for FixedTypeIdTagged<#enum_def_name> {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    FixedTypeIdTagged {
                        data: val.into(),
                        type_id: type_id::<#enum_def_name>(),
                    }
                }
            }
        },
        (false, _) => quote! {
            #attr_stream
            #[repr(u16)]
            #vis enum #name {
                #enum_stream
            }
        },
        _ => {
            todo!()
        }
    };

    let enum_alias_impl = {
        let revision = revision as u64;
        quote! {
            impl #name {
                #[inline(always)]
                pub fn max_type_version() -> self::FixedVersion {
                    self::FixedVersion::new(#revision,0,0)
                }
            }
        }
    };

    let enum_alias_serde_impl = if serde_support {
        quote! {
            impl #name {
                /// Automatically deserialize corresponding version tagged struct with given deserializer
                pub fn deserialize_serde<'de, F, T, E>(get_deserializer: F) -> ::core::result::Result<Self, E>
                where
                    F: Fn() -> T,
                    for<'a> &'a mut T: ::serde::de::Deserializer<'de, Error = E>,
                    E: ::serde::de::Error,
                {
                    let mut de = get_deserializer();
                    let tag: FixedTypeIdTag = ::serde::Deserialize::deserialize(&mut de)?;
                    let (id, ver) = tag.get_identifier();
                    let de_ver = ver.major;
                    let expect_id = self::type_id::<Self>();
                    if id != expect_id {
                        return Err(::serde::de::Error::custom(format!("type id mismatch, de:{}, expect:{}", id, expect_id)));
                    }
                    let current_max_ver = Self::max_type_version().major;
                    if current_max_ver < de_ver {
                        return Err(::serde::de::Error::custom(format!("version too new, current_max:{}, de_ver:{}", current_max_ver, de_ver)));
                    }
                    let mut de = get_deserializer();
                    ::serde::Deserialize::deserialize(&mut de)
                }

                /// Serialize this struct into a version tagged struct.
                ///
                /// For serde, you can use this method or specific methods provides by the serde lib you choose.
                /// This method is just provided for API consistency with rkyv or other binary serialize framework.
                pub fn serialize_serde<F, T, E>(&self, serialize_fn: F) -> ::core::result::Result<T, E>
                where
                    F: Fn(&Self) -> ::core::result::Result<T,E>,
                    E: ::std::error::Error,
                {
                    serialize_fn(self)
                }
            }
        }
    } else {
        quote! {}
    };

    let enum_alias_rkyv_impl = if rkyv_support {
        quote! {
            impl #name {
                pub fn access_rkyv(data: &[u8]) -> ::core::result::Result<&::rkyv::Archived<Self>, ::rkyv::rancor::Error> {
                    let tag = ::rkyv::access::<::rkyv::Archived<self::FixedTypeIdTag>, _>(data)?;
                    let (deser_id, ver) = tag.get_identifier();
                    let deser_ver = ver.major;
                    let expect_id = self::type_id::<Self>();
                    if deser_id != expect_id {
                        ::rkyv::rancor::fail!(self::TypeIdMismatchError {
                            deser_id,
                            expect_id
                        });
                    }
                    let current_max_ver = Self::max_type_version().major;
                    if current_max_ver < deser_ver {
                        ::rkyv::rancor::fail!(self::VersionTooNewError {
                            current_max_ver,
                            deser_ver
                        });
                    }
                    let archived_tagged = ::rkyv::access::<::rkyv::Archived<self::FixedTypeIdTagged<Self>>, _>(data)?;
                    Ok(archived_tagged.data.get())
                }

                pub fn deserialize_rkyv(data: &[u8]) -> ::core::result::Result<Self, ::rkyv::rancor::Error> {
                    let tag = ::rkyv::access::<::rkyv::Archived<self::FixedTypeIdTag>, _>(data)?;
                    let (deser_id, ver) = tag.get_identifier();
                    let deser_ver = ver.major;
                    let expect_id = self::type_id::<Self>();
                    if deser_id != expect_id {
                        ::rkyv::rancor::fail!(self::TypeIdMismatchError {
                            deser_id,
                            expect_id
                        });
                    }
                    let current_max_ver = Self::max_type_version().major;
                    if current_max_ver < deser_ver {
                        ::rkyv::rancor::fail!(self::VersionTooNewError {
                            current_max_ver,
                            deser_ver
                        });
                    }
                    let archived_tagged = ::rkyv::access::<::rkyv::Archived<self::FixedTypeIdTagged<Self>>, _>(data)?;
                    let archived_enum = archived_tagged.data.get();
                    ::rkyv::deserialize(archived_enum)
                }

                pub fn serialize_rkyv<E>(&self) -> ::core::result::Result<::rkyv::util::AlignedVec,E>
                where
                    E: ::rkyv::rancor::Source
                {
                    ::rkyv::to_bytes::<E>(
                        &::core::convert::Into::<self::FixedTypeIdTagged<Self>>::into(self.clone()),
                    )
                }

                pub fn serialize_into_rkyv<E>(self) -> ::core::result::Result<::rkyv::util::AlignedVec,E>
                where
                    E: ::rkyv::rancor::Source
                {
                    ::rkyv::to_bytes::<E>(
                        &::core::convert::Into::<self::FixedTypeIdTagged<Self>>::into(self),
                    )
                }
            }
        }
    } else {
        quote! {}
    };

    // Build the full path by combining prefix path (if any) with the type name
    let fixed_id_name = match &fixed_id_prefix {
        None => syn::Path::from(name.clone()),
        Some(prefix) => {
            let mut segments = prefix.segments.clone();
            segments.push(syn::PathSegment {
                ident: name.clone(),
                arguments: syn::PathArguments::None,
            });
            syn::Path {
                leading_colon: prefix.leading_colon,
                segments,
            }
        }
    };

    let fixed_id_def_name = match &fixed_id_prefix {
        None => syn::Path::from(enum_def_name),
        Some(prefix) => {
            let mut segments = prefix.segments.clone();
            segments.push(syn::PathSegment {
                ident: enum_def_name,
                arguments: syn::PathArguments::None,
            });
            syn::Path {
                leading_colon: prefix.leading_colon,
                segments,
            }
        }
    };

    Ok(match (serde_support, rkyv_support) {
        (true, _) => quote! {
            #(#reexport_revisions)*

            #enum_alias

            #enum_alias_impl

            #enum_alias_serde_impl

            #enum_alias_rkyv_impl

            self::fixed_type_id! {
                #[version((0,0,0))]
                #[omit_version_hash]
                #fixed_id_name
            }

            self::fixed_type_id! {
                // it's always the current revision
                #[version((0,0,0))]
                #[equal_to(#name)]
                #[omit_version_hash]
                #fixed_id_def_name
            }
        },
        (false, _) => quote! {
            #(#reexport_revisions)*

            #enum_alias

            #enum_alias_impl

            #enum_alias_serde_impl

            #enum_alias_rkyv_impl

            self::fixed_type_id! {
                #[version((0,0,0))]
                #[omit_version_hash]
                #fixed_id_name
            }
        },
        _ => unimplemented!(),
    })
}
