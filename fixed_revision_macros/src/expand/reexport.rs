use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{punctuated::Pair, spanned::Spanned, token, Generics, Ident, Path, PathSegment};

use crate::ast::{self, Fields, Visit};

use super::common::{rkyv_compare_trait_fn, rkyv_derive_trait_fn};

/// Visitor which reexports the item, recreating it with only the given fields.
pub struct Reexport<'a> {
    pub current: usize,
    pub revision: usize,
    pub fixed_id_prefix: Option<Path>,
    pub with_revision_suffix: bool,
    pub stream: &'a mut TokenStream,
    pub enum_stream: Option<&'a mut TokenStream>,
    pub specific_derives: &'a mut HashSet<&'static str>,
    pub generics: &'a mut HashMap<usize, Generics>,
    pub serde_support: bool,
    pub rkyv_support: bool,
}
impl<'a, 'ast> Visit<'ast> for Reexport<'a> {
    fn visit_item(&mut self, i: &'ast ast::Item) -> syn::Result<()> {
        self.stream.append_all(quote! {
            #[allow(non_camel_case_types)]
        });
        // Note(cupofc0t): process specific derive macros `Debug, PartialEq, Copy, Clone`, it's useful when deal with rkyv.
        for attr in i.attrs.other.iter() {
            // check derive attrs
            if attr.path().is_ident("derive") {
                match &attr.meta {
                    syn::Meta::List(meta_list) => {
                        let _ = meta_list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("Copy") {
                                self.specific_derives.insert("Copy");
                            }
                            if meta.path.is_ident("Clone") {
                                self.specific_derives.insert("Clone");
                            }
                            if meta.path.is_ident("PartialEq") {
                                self.specific_derives.insert("PartialEq");
                            }
                            if meta.path.is_ident("Debug") {
                                self.specific_derives.insert("Debug");
                            }
                            if meta.path.is_ident("PartialOrd") {
                                self.specific_derives.insert("PartialOrd");
                            }
                            Ok(())
                        });
                    }
                    _ => {}
                }
            }
            attr.to_tokens(self.stream);
        }
        if self.serde_support {
            self.stream.append_all(quote! {
                #[derive(::serde::Serialize, ::serde::Deserialize)]
            });
        }
        if self.rkyv_support {
            // when rkyv, add specific derives to archived types.
            let compare_vec: Vec<_> = self
                .specific_derives
                .iter()
                .filter_map(|&s| {
                    if rkyv_compare_trait_fn(s) {
                        Some(Ident::new(s, i.span()))
                    } else {
                        None
                    }
                })
                .collect();
            let filterd_out_vec: Vec<_> = self
                .specific_derives
                .iter()
                .filter_map(|&s| {
                    if rkyv_derive_trait_fn(s) {
                        Some(Ident::new(s, i.span()))
                    } else {
                        None
                    }
                })
                .collect();
            self.stream.append_all(quote! {
                #[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
                #[rkyv(compare(#(#compare_vec,)*), derive(#(#filterd_out_vec,)*))]
            });
        }
        i.vis.to_tokens(self.stream);
        ast::visit_item(self, i)
    }

    fn visit_item_kind(&mut self, i: &'ast ast::ItemKind) -> syn::Result<()> {
        ast::visit_item_kind(self, i)
    }

    fn visit_enum(&mut self, i: &'ast ast::Enum) -> syn::Result<()> {
        i.enum_.to_tokens(self.stream);
        let name = if self.with_revision_suffix {
            syn::Ident::new(&format!("{}_{}", i.name, self.current), i.name.span())
        } else {
            i.name.clone()
        };
        name.to_tokens(self.stream);
        self.generics.insert(self.current, i.generics.clone());
        // i.name.to_tokens(self.stream);
        i.generics.to_tokens(self.stream);
        i.braces.surround(self.stream, |stream| {
            let mut this = Reexport {
                revision: self.revision,
                current: self.current,
                with_revision_suffix: false,
                fixed_id_prefix: None,
                stream,
                enum_stream: None,
                specific_derives: self.specific_derives,
                generics: self.generics,
                serde_support: self.serde_support,
                rkyv_support: self.rkyv_support,
            };
            for pairs in i.variants.pairs() {
                match pairs {
                    Pair::Punctuated(v, p) => {
                        if v.attrs.options.exists_at(self.current) {
                            this.visit_variant(v).unwrap();
                            p.to_tokens(this.stream);
                        }
                    }
                    Pair::End(v) => {
                        if v.attrs.options.exists_at(self.current) {
                            this.visit_variant(v).unwrap();
                        }
                    }
                }
            }
        });
        if self.with_revision_suffix {
            let current = self.current as u16;
            let enum_ident = syn::Ident::new(&format!("{}", i.name), i.name.span());
            let enum_var_ident = syn::Ident::new(&format!("V{}", current), i.name.span());
            // let alias_enum_name = format!("{}", i.name);
            let fixed_id_name = match &self.fixed_id_prefix {
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
            self.stream.append_all(quote! {
                self::fixed_type_id! {
                    #[version((#current,0,0))]
                    #[equal_to(#enum_ident)]
                    #[omit_version_hash]
                    #fixed_id_name
                }

                impl From<#name> for #enum_ident {
                    fn from(value: #name) -> Self {
                        #enum_ident::#enum_var_ident(value)
                    }
                }

                impl From<#enum_ident> for #name {
                    fn from(value: #enum_ident) -> Self {
                        match value {
                            #enum_ident::#enum_var_ident(v) => v,
                            _ => panic!("Invalid enum variant"),
                        }
                    }
                }
            });
            self.enum_stream.as_mut().map(|stream| {
                let enum_var_name = syn::Ident::new(&format!("V{}", current), i.name.span());
                let name = name.clone();
                stream.append_all(quote! {
                    #enum_var_name(#name) = #current as _,
                });
            });
        }
        Ok(())
    }

    fn visit_struct(&mut self, i: &'ast ast::Struct) -> syn::Result<()> {
        i.struct_.to_tokens(self.stream);
        let name = if self.with_revision_suffix {
            syn::Ident::new(&format!("{}_{}", i.name, self.current), i.name.span())
        } else {
            i.name.clone()
        };
        name.to_tokens(self.stream);
        // i.generics.to_tokens(self.stream);
        ast::visit_struct(self, i)?;
        if matches!(i.fields, Fields::Unnamed { .. } | Fields::Unit) {
            token::Semi(Span::call_site()).to_tokens(self.stream);
        }
        self.generics.insert(self.current, i.generics.clone());
        if self.with_revision_suffix {
            let current = self.current as u16;
            let enum_ident = syn::Ident::new(&format!("{}", i.name), i.name.span());
            let enum_var_ident = syn::Ident::new(&format!("V{}", current), i.name.span());
            // let alias_enum_name = format!("{}", i.name);
            let fixed_id_name = match &self.fixed_id_prefix {
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
            self.stream.append_all(quote! {
                self::fixed_type_id! {
                    #[version((#current,0,0))]
                    #[equal_to(#enum_ident)]
                    #[omit_version_hash]
                    #fixed_id_name
                }

                impl From<#name> for #enum_ident {
                    fn from(value: #name) -> Self {
                        #enum_ident::#enum_var_ident(value)
                    }
                }

                impl From<#enum_ident> for #name {
                    fn from(value: #enum_ident) -> Self {
                        match value {
                            #enum_ident::#enum_var_ident(v) => v,
                            _ => panic!("Invalid enum variant"),
                        }
                    }
                }
            });
            self.enum_stream.as_mut().map(|stream| {
                let enum_var_name = syn::Ident::new(&format!("V{}", current), i.name.span());
                let name = name.clone();
                stream.append_all(quote! {
                    #enum_var_name(#name) = #current as _,
                });
            });
        }
        Ok(())
    }

    fn visit_variant(&mut self, i: &'ast ast::Variant) -> syn::Result<()> {
        if !i.attrs.options.exists_at(self.current) {
            return Ok(());
        }

        i.attrs.other.iter().for_each(|x| x.to_tokens(self.stream));
        i.ident.to_tokens(self.stream);
        ast::visit_variant(self, i)?;

        if let Some((eq, expr)) = i.discriminant.as_ref() {
            eq.to_tokens(self.stream);
            expr.to_tokens(self.stream);
        }

        Ok(())
    }

    fn visit_fields(&mut self, i: &'ast ast::Fields) -> syn::Result<()> {
        match i {
            ast::Fields::Named { brace, fields } => {
                brace.surround(self.stream, |stream| {
                    let mut this = Reexport {
                        revision: self.revision,
                        current: self.current,
                        with_revision_suffix: false,
                        fixed_id_prefix: None,
                        stream,
                        enum_stream: None,
                        specific_derives: self.specific_derives,
                        generics: self.generics,
                        serde_support: self.serde_support,
                        rkyv_support: self.rkyv_support,
                    };
                    for pair in fields.pairs() {
                        match pair {
                            Pair::Punctuated(f, c) => {
                                if f.attrs.options.exists_at(self.current) {
                                    this.visit_field(f).unwrap();
                                    c.to_tokens(this.stream)
                                }
                            }
                            Pair::End(f) => {
                                if f.attrs.options.exists_at(self.current) {
                                    this.visit_field(f).unwrap();
                                }
                            }
                        }
                    }
                });
                Ok(())
            }
            ast::Fields::Unnamed { paren, fields } => {
                paren.surround(self.stream, |stream| {
                    let mut this = Reexport {
                        revision: self.revision,
                        current: self.current,
                        with_revision_suffix: false,
                        fixed_id_prefix: None,
                        stream,
                        enum_stream: None,
                        specific_derives: self.specific_derives,
                        generics: self.generics,
                        serde_support: self.serde_support,
                        rkyv_support: self.rkyv_support,
                    };
                    for pair in fields.pairs() {
                        match pair {
                            Pair::Punctuated(f, c) => {
                                if f.attrs.options.exists_at(self.current) {
                                    this.visit_field(f).unwrap();
                                    c.to_tokens(this.stream)
                                }
                            }
                            Pair::End(f) => {
                                if f.attrs.options.exists_at(self.current) {
                                    this.visit_field(f).unwrap();
                                }
                            }
                        }
                    }
                });
                Ok(())
            }
            ast::Fields::Unit => Ok(()),
        }
    }

    fn visit_field(&mut self, i: &'ast ast::Field) -> syn::Result<()> {
        i.attrs.other.iter().for_each(|x| x.to_tokens(self.stream));
        i.vis.to_tokens(self.stream);
        match i.name {
            ast::FieldName::Ident(ref x) => x.to_tokens(self.stream),
            ast::FieldName::Index(_) => {}
        }
        i.colon_token.map(|x| x.to_tokens(self.stream));
        i.ty.to_tokens(self.stream);
        Ok(())
    }
}
