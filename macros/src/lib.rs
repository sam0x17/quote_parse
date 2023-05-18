use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::Result;

#[proc_macro]
pub fn quote_parse(tokens: TokenStream) -> TokenStream {
    match quote_parse_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn quote_parse_internal(tokens: impl Into<TokenStream2>) -> Result<TokenStream2> {
    let tokens = tokens.into();
    for token in tokens {
        match token {
            TokenTree::Group(group) => println!("{:?}..{:?}", group.delimiter(), group.delimiter()),
            TokenTree::Ident(ident) => println!("{}", ident.to_string()),
            TokenTree::Punct(punct) => println!("{}", punct.as_char()),
            TokenTree::Literal(lit) => println!("{}", lit.to_string()),
        }
    }
    Ok(quote!())
}

#[test]
fn test_quote_parse_internal() {
    quote_parse_internal(quote! {
        struct Something {
            field1: u32,
            field2: u32,
        }
    })
    .unwrap();
}
