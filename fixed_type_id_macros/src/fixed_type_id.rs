use core::panic;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use rand::prelude::*;
use rapidhash::rapidhash;
use std::fs::File;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, Expr, GenericArgument, Lit, PathArguments, Result, Token,
};

/// Copy from [`rapidhash`]
#[inline(always)]
const fn rapid_mum(a: u64, b: u64) -> (u64, u64) {
    let r = a as u128 * b as u128;
    (r as u64, (r >> 64) as u64)
}

/// Copy from [`rapidhash`]
#[inline(always)]
const fn rapid_mix(a: u64, b: u64) -> u64 {
    let (a, b) = rapid_mum(a, b);
    a ^ b
}

fn version_to_hash(version: &(u64, u64, u64)) -> u64 {
    // make a version tuple to a [u8; 24]
    let mut bytes = [0u8; 24];
    bytes[0..8].copy_from_slice(&version.0.to_le_bytes());
    bytes[8..16].copy_from_slice(&version.1.to_le_bytes());
    bytes[16..24].copy_from_slice(&version.2.to_le_bytes());
    rapidhash(&bytes)
}

#[derive(Clone)]
enum Type {
    Path(syn::Path),
    Tuple(Vec<(RefType, Type)>),
}

// Add helper function to parse types recursively
fn parse_type(input: ParseStream) -> Result<(RefType, Type)> {
    let mut ref_type = RefType::None;

    // Check for reference
    if input.peek(Token![&]) {
        let _ = input.parse::<Token![&]>()?;
        let mutability = input.parse::<Token![mut]>().ok();
        ref_type = if mutability.is_some() {
            RefType::Mutable
        } else {
            RefType::Shared
        };
    }

    // Check for tuple
    if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);
        let mut elements = Vec::new();

        loop {
            let (elem_ref, elem_type) = parse_type(&content)?;
            elements.push((elem_ref, elem_type));

            if content.is_empty() {
                break;
            }
            content.parse::<Token![,]>()?;
        }

        Ok((ref_type, Type::Tuple(elements)))
    } else {
        // Parse as path
        let path = input.parse()?;
        Ok((ref_type, Type::Path(path)))
    }
}

// Update path_to_string to handle tuples
fn type_to_string(ref_type: RefType, is_dyn: bool, ty: &Type) -> String {
    let ref_str = match ref_type {
        RefType::None => "",
        RefType::Shared => "&",
        RefType::Mutable => "&mut",
    };

    let type_str = match ty {
        Type::Path(path) => {
            let is_dyn_str = if is_dyn { "dyn" } else { "" };
            let path_str = format! {"{}", quote!(#path)}.replace(" ", "");
            format!("{} {}", is_dyn_str, path_str)
        }
        Type::Tuple(elements) => {
            let elem_strs: Vec<_> = elements
                .iter()
                .map(|(ref_type, ty)| type_to_string(*ref_type, false, ty))
                .collect();
            format!("({})", elem_strs.join(","))
        }
    };

    format!("{}{}", ref_str, type_str).trim().to_string()
}

fn extract_generics(path: &syn::Path) -> Vec<GenericArgument> {
    path.segments
        .last()
        .and_then(|seg| match &seg.arguments {
            PathArguments::AngleBracketed(angle_bracketed) => {
                Some(angle_bracketed.args.clone().into_iter().collect())
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// Custom structure to represent `dyn TraitName` and multiple attributes
struct GeneralTypesInput {
    file: Option<String>, // The value of FixedTypeIdFile, e.g. "types.toml"
    version: (u64, u64, u64),
    type_id_equal_to: Option<String>,
    types: Vec<Type>,
    /// current generics is always empty, because that is parsed to [`Self::paths`]
    _generics: Vec<Vec<GenericArgument>>,
    is_dyn: Vec<bool>,
    /// ref type at outer level
    ref_type: Vec<RefType>,
}

#[derive(Debug, Clone, Copy)]
enum RefType {
    None,
    Shared,
    Mutable,
}

impl Parse for GeneralTypesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut file = None;
        // Parse the outer attributes (e.g., #[FixedTypeIdFile], #[FixedTypeIdStart], etc.)
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let mut version = (0, 0, 0);
        let mut type_id_equal_to = None;
        for attr in attrs {
            if attr.path().is_ident("FixedTypeIdFile") {
                if let Expr::Lit(expr_lit) = attr.parse_args()? {
                    if let Lit::Str(lit_str) = expr_lit.lit {
                        file = Some(lit_str.value());
                    }
                }
            } else if attr.path().is_ident("FixedTypeIdVersion") {
                if let Expr::Tuple(expr_tuple) = attr.parse_args()? {
                    let mut version_parts = vec![];
                    for elem in expr_tuple.elems {
                        if let Expr::Lit(expr_lit) = elem {
                            if let Lit::Int(lit_int) = expr_lit.lit {
                                version_parts.push(lit_int.base10_parse::<u64>().unwrap());
                            }
                        } else if let Expr::Group(expr_group) = elem {
                            // used inside declare macro
                            let inner = expr_group.expr;
                            if let Expr::Lit(expr_lit) = *inner {
                                if let Lit::Int(lit_int) = expr_lit.lit {
                                    version_parts.push(lit_int.base10_parse::<u64>().unwrap());
                                }
                            }
                        }
                    }
                    assert_eq!(version_parts.len(), 3);
                    version = (version_parts[0], version_parts[1], version_parts[2]);
                }
            } else if attr.path().is_ident("FixedTypeIdEqualTo") {
                // all types scope in this macro will have the same TypeIdEqualTo type specified by this attribute
                if let Expr::Lit(expr_lit) = attr.parse_args()? {
                    if let Lit::Str(lit_str) = expr_lit.lit {
                        type_id_equal_to = Some(lit_str.value());
                    }
                }
            }
        }
        // Parse the `dyn TraitName` part with following format:
        //
        // dyn TraitName
        // dyn TraitName<T,U,K<T>>
        // StructName
        // StructName<T,U,K<T>>

        let mut types = Vec::new();
        let mut generics = Vec::new();
        let mut is_dyn = Vec::new();
        let mut ref_type = Vec::new();
        // Parse multiple definitions
        while !input.is_empty() {
            // Check if it's a dyn trait
            let is_dyn_current = input.parse::<Token![dyn]>().is_ok();
            is_dyn.push(is_dyn_current);

            // Parse type which could be reference, tuple or path
            let (current_ref_type, type_value) = parse_type(input)?;
            ref_type.push(current_ref_type);

            // Optionally parse "::" before generic arguments, when parsed by `syn``
            if input.peek(Token![::]) {
                let _ = input.parse::<Token![::]>();
            }
            // Get generics if it's a path type
            let current_generics = match &type_value {
                Type::Path(path) => extract_generics(path),
                _ => Vec::new(),
            };
            types.push(type_value);
            generics.push(current_generics);

            // If there's a comma, continue to the next definition
            if input.parse::<Token![;]>().is_ok() {
                continue;
            } else {
                break;
            }
        }

        Ok(GeneralTypesInput {
            file,
            version,
            type_id_equal_to,
            types,
            _generics: generics,
            is_dyn,
            ref_type,
        })
    }
}

fn store_id_in_file(file_name: &str, type_names: &[String], idvec: &[u64]) -> std::io::Result<()> {
    use std::io::Write;
    let path = std::path::Path::new(file_name);
    let mut ids = std::collections::HashMap::new();

    // Read existing IDs from the file
    if path.exists() {
        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);
        for line in std::io::BufRead::lines(reader) {
            let line = line?;
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                ids.insert(
                    parts[0].trim().to_string(),
                    parts[1].trim().parse::<u64>().unwrap(),
                );
            }
        }
        for (type_name, id) in type_names.iter().zip(idvec.iter()) {
            // Update or add the new ID
            let entry = ids.entry(type_name.to_string()).or_insert(*id);
            *entry = *id;
        }

        // Write all IDs back to the file
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        for (name, id) in ids {
            writeln!(file, "{} = {}", name, id)?;
        }
        Ok(())
    } else {
        panic!("File {} doesn't exist!", file_name);
    }
}

fn path_to_prefix_path(path: &syn::Path) -> (syn::Path, syn::Path) {
    // the last segments is the name
    // other segments are the prefix
    let mut prefix = path.clone();
    let mut name = path.clone();
    prefix.segments.pop();
    name.segments = name
        .segments
        .last()
        .map(|seg| {
            let mut new_path = syn::Path::from(seg.ident.clone());
            new_path.segments[0].arguments = seg.arguments.clone();
            new_path.segments
        })
        .unwrap_or_default();
    (prefix, name)
}

fn type_to_token_stream(ref_type: RefType, is_dyn: bool, ty: &Type) -> proc_macro2::TokenStream {
    let ref_type = match ref_type {
        RefType::None => quote! {},
        RefType::Shared => quote! { & },
        RefType::Mutable => quote! { &mut },
    };

    let type_tokens = match ty {
        Type::Path(path) => {
            let (_, path) = path_to_prefix_path(path);
            let is_dyn = if is_dyn {
                quote! { dyn }
            } else {
                quote! {}
            };
            quote! { #is_dyn #path }
        }
        Type::Tuple(elements) => {
            let elem_tokens = elements
                .iter()
                .map(|(ref_type, ty)| type_to_token_stream(*ref_type, false, ty));
            quote! { (#(#elem_tokens),*) }
        }
    };

    quote! { #ref_type #type_tokens }
}

pub fn fixed_type_id_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as GeneralTypesInput);
    let types_file_name = ast.file;
    let id_type = syn::parse_str::<syn::Type>("u64").unwrap();

    let mut implementations = Vec::new();

    let mut names = vec![];
    let mut hashes = vec![];

    for (index, path) in ast.types.iter().enumerate() {
        let mut path_str = type_to_string(ast.ref_type[index], ast.is_dyn[index], path);
        let type_token_stream = type_to_token_stream(ast.ref_type[index], ast.is_dyn[index], path);
        // Hash the name and version to a u64
        names.push(path_str.clone());
        // path string hash
        let path_hash = rapidhash(path_str.as_bytes());
        // version hash
        let version_hash = version_to_hash(&ast.version);
        let mut hash = rapid_mix(path_hash, version_hash);
        if cfg!(feature = "erase_name") {
            let path_str_hash = rapidhash(path_str.as_bytes());
            path_str = format!("0x{:x}", path_str_hash);
        }
        if ast.type_id_equal_to.is_some() {
            // store 0u64
            hash = 0;
        }
        hashes.push(hash);

        let major = ast.version.0;
        let minor = ast.version.1;
        let patch = ast.version.2;

        let implementation = if let Some(type_id_equal_to) = &ast.type_id_equal_to {
            // create a ident
            let type_id_equal_to_ident = Ident::new(type_id_equal_to, Span::call_site());
            quote! {
              impl self::FixedTypeId for #type_token_stream {
                  const TYPE_NAME: &'static str = #path_str;
                  const TYPE_ID: self::FixedId = <#type_id_equal_to_ident as self::FixedTypeId>::TYPE_ID;
                  const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(#major, #minor, #patch);

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
            }
        } else {
            quote! {
                impl self::FixedTypeId for #type_token_stream {
                    const TYPE_NAME: &'static str = #path_str;
                    const TYPE_ID: self::FixedId = self::FixedId(#hash as #id_type);
                    const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(#major, #minor, #patch);

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
            }
        };

        implementations.push(implementation);
    }
    if let Some(file_name) = types_file_name {
        if let Err(e) = store_id_in_file(&file_name, &names, &hashes) {
            panic!("Failed to store ID in file: {}", e);
        }
    }
    // let id = gen_id(&types_file_name, &ast.trait_name.to_string(), gen_start);

    TokenStream::from(quote! {
        #(#implementations)*
    })
}

pub fn fixed_type_id_impl_without_version_hash_in_type(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as GeneralTypesInput);

    let types_file_name = ast.file;
    let id_type = syn::parse_str::<syn::Type>("u64").unwrap();

    let mut implementations = Vec::new();

    let mut names = vec![];
    let mut hashes = vec![];

    for (index, path) in ast.types.iter().enumerate() {
        let mut path_str = type_to_string(ast.ref_type[index], ast.is_dyn[index], path);
        let type_token_stream = type_to_token_stream(ast.ref_type[index], ast.is_dyn[index], path);
        // Hash the name and version to a u64
        names.push(path_str.clone());
        let path_str_hash = rapidhash(path_str.as_bytes());
        let mut hash = path_str_hash;
        if cfg!(feature = "erase_name") {
            path_str = format!("0x{:x}", path_str_hash);
        }
        if ast.type_id_equal_to.is_some() {
            // store 0u64
            hash = 0;
        }
        hashes.push(hash);

        let major = ast.version.0;
        let minor = ast.version.1;
        let patch = ast.version.2;
        let implementation = if let Some(type_id_equal_to) = &ast.type_id_equal_to {
            // create a ident
            let type_id_equal_to_ident = Ident::new(type_id_equal_to, Span::call_site());
            quote! {
              impl self::FixedTypeId for #type_token_stream {
                  const TYPE_NAME: &'static str = #path_str;
                  const TYPE_ID: self::FixedId = <#type_id_equal_to_ident as self::FixedTypeId>::TYPE_ID;
                  const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(#major, #minor, #patch);

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
            }
        } else {
            quote! {
                impl self::FixedTypeId for #type_token_stream {
                    const TYPE_NAME: &'static str = #path_str;
                    const TYPE_ID: self::FixedId = self::FixedId(#hash as #id_type);
                    const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(#major, #minor, #patch);

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
            }
        };

        implementations.push(implementation);
    }

    if let Some(file_name) = types_file_name {
        if let Err(e) = store_id_in_file(&file_name, &names, &hashes) {
            panic!("Failed to store ID in file: {}", e);
        }
    }
    // let id = gen_id(&types_file_name, &ast.trait_name.to_string(), gen_start);

    TokenStream::from(quote! {
        #(#implementations)*
    })
}

pub fn random_fixed_type_id_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as GeneralTypesInput);

    let types_file_name = ast.file;
    let id_type = syn::parse_str::<syn::Type>("u64").unwrap();

    let mut implementations = Vec::new();

    let mut names = vec![];
    let mut hashes = vec![];

    for (index, path) in ast.types.iter().enumerate() {
        let mut path_str = type_to_string(ast.ref_type[index], ast.is_dyn[index], path);
        let type_token_stream = type_to_token_stream(ast.ref_type[index], ast.is_dyn[index], path);
        // Hash the name and version to a u64
        names.push(path_str.clone());

        if cfg!(feature = "erase_name") {
            let path_str_hash = rapidhash(path_str.as_bytes());
            path_str = format!("0x{:x}", path_str_hash);
        }
        let hash: u64 = random();
        hashes.push(hash);

        let major = ast.version.0;
        let minor = ast.version.1;
        let patch = ast.version.2;
        let implementation = quote! {
            impl self::FixedTypeId for #type_token_stream {
                const TYPE_NAME: &'static str = #path_str;
                const TYPE_ID: self::FixedId = self::FixedId(#hash as #id_type);
                const TYPE_VERSION: self::FixedVersion = self::FixedVersion::new(#major, #minor, #patch);

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
        };

        implementations.push(implementation);
    }

    if let Some(file_name) = types_file_name {
        if let Err(e) = store_id_in_file(&file_name, &names, &hashes) {
            panic!("Failed to store ID in file: {}", e);
        }
    }
    // let id = gen_id(&types_file_name, &ast.trait_name.to_string(), gen_start);

    TokenStream::from(quote! {
        #(#implementations)*
    })
}
