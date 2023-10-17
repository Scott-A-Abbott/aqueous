use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Message)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let ident_string = ident.to_string();

    let tokens = quote! {
        impl Message for #ident {
            const TYPE_NAME: &'static str = #ident_string;
        }
    };

    TokenStream::from(tokens)
}
