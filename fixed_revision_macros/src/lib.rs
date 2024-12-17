use proc_macro::TokenStream;
mod ast;
mod expand;

#[proc_macro_attribute]
pub fn revisioned(attrs: TokenStream, input: TokenStream) -> TokenStream {
    match expand::revision(attrs.into(), input.into()) {
        Ok(x) => x.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
