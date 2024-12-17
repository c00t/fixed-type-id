mod common;
mod de;
mod reexport;
mod ser;
mod validate_version;

use core::panic;
use std::u16;

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
    //   just convert it to distinct semver version, `(0,x++,0)` or `(x++,0,0)` to make it compatible with fixed_type_id.
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
            stream.extend(quote! {
                #[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
            });
        }

        stream
    };

    let attr_def_stream = {
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

        if rkyv_support {
            stream.extend(quote! {
                #[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
            });
        }

        stream
    };
    let enum_def_name = Ident::new(&format!("{}Def", name), name.span());
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
    let enum_alias = quote! {
        #attr_stream
        #[repr(u16)]
        #vis enum #name {
            #enum_stream
        }

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

    Ok(quote! {
        #(#reexport_revisions)*

        #enum_alias

        self::fixed_type_id_without_version_hash! {
            #[FixedTypeIdVersion((#revision,0,0))]
            #fixed_id_name
        }

        self::fixed_type_id_without_version_hash! {
            // it's always the current revision
            #[FixedTypeIdVersion((#revision,0,0))]
            #[FixedTypeIdEqualTo(#name)]
            #fixed_id_def_name
        }
    })
}
