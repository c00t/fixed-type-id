use quote::{format_ident, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self, Brace, Paren},
    Expr, Generics, Ident, Index, Result, Token, Type, Visibility,
};

mod attributes;
mod visit;
pub use attributes::{Direct, FieldOptions, FilteredAttributes, ItemOptions, VariantOptions};
pub use visit::*;

#[derive(Debug)]
pub struct Item {
    pub attrs: FilteredAttributes<ItemOptions>,
    pub vis: Visibility,
    pub kind: ItemKind,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            vis: input.parse()?,
            kind: input.parse()?,
        })
    }
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.attrs.to_tokens(tokens);
        self.vis.to_tokens(tokens);
        self.kind.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub enum ItemKind {
    Enum(Enum),
    Struct(Struct),
}

impl Parse for ItemKind {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![enum]) {
            return Ok(ItemKind::Enum(input.parse()?));
        }

        if input.peek(Token![struct]) {
            return Ok(ItemKind::Struct(input.parse()?));
        }

        Err(input.error("unsupported item, revision only supporst structs and enums."))
    }
}

impl ToTokens for ItemKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ItemKind::Enum(e) => e.to_tokens(tokens),
            ItemKind::Struct(s) => s.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
pub struct Enum {
    pub enum_: Token![enum],
    pub name: Ident,
    pub generics: Generics,
    pub braces: Brace,
    pub variants: Punctuated<Variant, Token![,]>,
}

impl Parse for Enum {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Enum {
            enum_: input.parse()?,
            name: input.parse()?,
            generics: input.parse()?,
            braces: braced!(content in input),
            variants: content.parse_terminated(Variant::parse, Token![,])?,
        })
    }
}

impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.enum_.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.braces.surround(tokens, |tokens| {
            self.variants.to_tokens(tokens);
        });
    }
}

#[derive(Debug)]
pub struct Variant {
    pub attrs: FilteredAttributes<VariantOptions>,
    pub ident: Ident,
    pub fields: Fields,
    pub discriminant: Option<(Token![=], Expr)>,
}

impl Variant {
    /// Returns the name of the fields struct of this variant
    pub fn fields_name(&self, enum_name: &str) -> Ident {
        self.attrs
            .options
            .fields_name
            .as_ref()
            .map(|x| Ident::new(&x.value(), x.span()))
            .unwrap_or_else(|| {
                Ident::new(
                    &format!("{}{}Fields", enum_name, self.ident),
                    self.ident.span(),
                )
            })
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.parse()?;
        let ident = input.parse()?;
        let fields = if input.peek(token::Paren) {
            let content;
            let paren = parenthesized!(content in input);
            let mut fields = content.parse_terminated(Field::parse_unnamed, Token![,])?;
            fields.iter_mut().enumerate().for_each(|(idx, f)| {
                f.name = FieldName::Index(Index::from(idx));
            });
            Fields::Unnamed { paren, fields }
        } else if input.peek(token::Brace) {
            let content;
            let brace = braced!(content in input);
            let fields = content.parse_terminated(Field::parse_named, Token![,])?;
            Fields::Named { brace, fields }
        } else {
            Fields::Unit
        };

        let discriminant = if input.peek(Token![:]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };

        Ok(Self {
            attrs,
            ident,
            fields,
            discriminant,
        })
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.attrs.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.fields.to_tokens(tokens);
        if let Some((eq, expr)) = &self.discriminant {
            eq.to_tokens(tokens);
            expr.to_tokens(tokens);
        }
    }
}

#[derive(Debug)]
pub struct Struct {
    pub struct_: Token![struct],
    pub name: Ident,
    pub generics: Generics,
    pub fields: Fields,
}

impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_ = input.parse()?;
        let name = input.parse()?;
        let generics = input.parse()?;
        let fields = if input.peek(token::Paren) {
            let content;
            let paren = parenthesized!(content in input);
            let mut fields = content.parse_terminated(Field::parse_unnamed, Token![,])?;
            fields.iter_mut().enumerate().for_each(|(idx, f)| {
                f.name = FieldName::Index(Index::from(idx));
            });
            input.parse::<Token![;]>()?;
            Fields::Unnamed { paren, fields }
        } else if input.peek(token::Brace) {
            let content;
            let brace = braced!(content in input);
            let fields = content.parse_terminated(Field::parse_named, Token![,])?;
            Fields::Named { brace, fields }
        } else {
            input.parse::<Token![;]>()?;
            Fields::Unit
        };

        Ok(Self {
            struct_,
            name,
            generics,
            fields,
        })
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.struct_.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.fields.to_tokens(tokens);

        // Add semicolon for unit structs or tuple structs
        if matches!(self.fields, Fields::Unit | Fields::Unnamed { .. }) {
            Token![;](self.struct_.span).to_tokens(tokens);
        }
    }
}

#[derive(Debug)]
pub enum Fields {
    Named {
        brace: Brace,
        fields: Punctuated<Field, Token![,]>,
    },
    Unnamed {
        paren: Paren,
        fields: Punctuated<Field, Token![,]>,
    },
    Unit,
}

impl ToTokens for Fields {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Fields::Named { brace, fields } => {
                brace.surround(tokens, |tokens| {
                    fields.to_tokens(tokens);
                });
            }
            Fields::Unnamed { paren, fields } => {
                paren.surround(tokens, |tokens| {
                    fields.to_tokens(tokens);
                });
            }
            Fields::Unit => {}
        }
    }
}

#[derive(Debug)]
pub enum FieldName {
    Ident(Ident),
    Index(Index),
}

impl FieldName {
    pub fn to_binding(&self) -> Ident {
        match self {
            FieldName::Ident(x) => x.clone(),
            FieldName::Index(x) => {
                format_ident!("field_{}", x.index)
            }
        }
    }
}

impl ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldName::Ident(x) => x.to_tokens(tokens),
            FieldName::Index(x) => x.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub attrs: FilteredAttributes<FieldOptions>,
    pub vis: Visibility,
    pub name: FieldName,
    pub colon_token: Option<Token![:]>,
    pub ty: Type,
}

impl Field {
    pub fn parse_unnamed(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.parse()?;
        let vis = input.parse()?;
        // This is later fixed
        let name = FieldName::Index(Index::from(0));
        let ty = input.parse()?;

        Ok(Self {
            attrs,
            vis,
            name,
            colon_token: None,
            ty,
        })
    }

    pub fn parse_named(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.parse()?;
        let vis = input.parse()?;
        let name = FieldName::Ident(input.parse()?);
        let colon_token = Some(input.parse()?);
        let ty = input.parse()?;

        Ok(Self {
            attrs,
            vis,
            name,
            colon_token,
            ty,
        })
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.attrs.to_tokens(tokens);
        self.vis.to_tokens(tokens);

        // For named fields, output the name and colon
        if let Some(colon) = &self.colon_token {
            self.name.to_tokens(tokens);
            colon.to_tokens(tokens);
        }

        self.ty.to_tokens(tokens);
    }
}
