use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse2,
    token::Brace,
    Ident, Result, Token, Type, Visibility,
};

#[proc_macro]
pub fn quote_parse(tokens: TokenStream) -> TokenStream {
    match quote_parse_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/*
quote_parse!(MyThing,
    struct #ident {
        #field1: #{type1 as TypePath},
        #field2: #{type2 as TypePath}
    }
);
*/

struct QuoteParseArgs {
    viz: Visibility,
    ident: Ident,
    _comma: Token![,],
    stream: TokenStream2,
}

impl Parse for QuoteParseArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(QuoteParseArgs {
            viz: input.parse()?,
            ident: input.parse()?,
            _comma: input.parse()?,
            stream: {
                let mut stream: TokenStream2 = TokenStream2::new();
                while let Ok(token) = input.parse::<TokenTree>() {
                    stream.extend(TokenStream2::from(token));
                }
                stream
            },
        })
    }
}

fn quote_parse_internal(tokens: impl Into<TokenStream2>) -> Result<TokenStream2> {
    let args = parse2::<QuoteParseArgs>(tokens.into())?;
    let struct_contents = walk_token_stream(args.stream)?;
    Ok(quote! {
        struct ParsedThing {
            #struct_contents
        }
    })
}

trait ToChar {
    fn to_char(&self, open: bool) -> char;
}

impl ToChar for Delimiter {
    fn to_char(&self, open: bool) -> char {
        match (self, open) {
            (Delimiter::Parenthesis, true) => '(',
            (Delimiter::Parenthesis, false) => ')',
            (Delimiter::Brace, true) => '{',
            (Delimiter::Brace, false) => '}',
            (Delimiter::Bracket, true) => '[',
            (Delimiter::Bracket, false) => ']',
            (Delimiter::None, _) => ' ',
        }
    }
}

struct Walker(TokenStream2);

impl Parse for Walker {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut output: TokenStream2 = TokenStream2::new();
        while !input.is_empty() {
            let token = input.parse::<TokenTree>()?;
            if let TokenTree::Punct(t) = &token {
                if t.as_char() == '#' {
                    if input.peek(Ident) {
                        // #ident
                        let ident = input.parse::<Ident>()?;
                        print!("ident var: {}", ident.to_string());
                        continue;
                    } else if input.peek(Brace) {
                        // #{ident as Type}
                        let content;
                        braced!(content in input);
                        let ident = content.parse::<Ident>()?;
                        content.parse::<Token![as]>()?;
                        let typ = content.parse::<Type>()?;
                        print!(
                            "typed var: {} as {}",
                            typ.to_token_stream().to_string(),
                            ident.to_string(),
                        );
                        continue;
                    }
                }
            }
            match token {
                TokenTree::Group(group) => {
                    // TODO: process parens/brackets/etc
                    print!("{}\n", group.delimiter().to_char(true));
                    output.extend(walk_token_stream(group.stream()));
                    print!("{}\n", group.delimiter().to_char(false));
                }
                TokenTree::Ident(ident) => print!("{} ", ident.to_string()),
                TokenTree::Punct(punct) => match punct.as_char() {
                    ';' => println!(";"),
                    ',' => println!(","),
                    _ => print!("{}", punct.as_char()),
                },
                TokenTree::Literal(lit) => print!("'{}'", lit.to_string()),
            }
        }
        Ok(Walker(output))
    }
}

fn walk_token_stream(tokens: TokenStream2) -> Result<TokenStream2> {
    match parse2::<Walker>(tokens) {
        Ok(walker) => Ok(walker.0),
        Err(err) => Err(err),
    }
}

#[test]
fn test_quote_parse_internal() {
    quote_parse_internal(quote! {
        pub MyThing,
        struct Something {
            field1: u32,
            field2: u32,
        }
    })
    .unwrap();
}
