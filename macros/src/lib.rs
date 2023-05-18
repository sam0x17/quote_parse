use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Literal, Punct, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{
    parse::{Nothing, Parse, ParseStream},
    parse2, Ident, Result, Token, Visibility,
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
    struct $ident {
        $field1: ${type1 as TypePath},
        $field2: ${type2 as TypePath}
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

enum TokenW {
    Group(Delimiter, Group),
    Ident(Ident),
    Punct(char, Punct),
    Lit(String, Literal),
}

impl From<&TokenTree> for TokenW {
    fn from(value: &TokenTree) -> Self {
        match value {
            TokenTree::Group(group) => TokenW::Group(group.delimiter(), group.clone()),
            TokenTree::Ident(ident) => TokenW::Ident(ident.clone()),
            TokenTree::Punct(punct) => TokenW::Punct(punct.as_char(), punct.clone()),
            TokenTree::Literal(lit) => TokenW::Lit(lit.to_string(), lit.clone()),
        }
    }
}

impl TokenW {
    fn from_opt(tt: Option<&TokenTree>) -> Option<TokenW> {
        match tt {
            Some(tt) => Some(TokenW::from(tt)),
            None => None,
        }
    }
}

fn walk_token_stream(tokens: TokenStream2) -> Result<TokenStream2> {
    let mut output: TokenStream2 = TokenStream2::new();
    let mut tokens = tokens.into_iter().collect::<Vec<TokenTree>>();
    tokens.reverse();
    let mut i = 0;
    while {
        i += 1;
        !tokens.is_empty()
    } {
        let token = tokens.pop();
        let (peek1, peek2) = (
            TokenW::from_opt(tokens.get(tokens.len() - 1 - 1)),
            TokenW::from_opt(tokens.get(tokens.len() - 1 - 2)),
        );
        match (peek1, peek2) {
            (Some(TokenW::Punct('$', _)), Some(TokenW::Ident(_))) => {
                // $ident
            }
            (Some(TokenW::Punct('$', _)), Some(TokenW::Group(Delimiter::Brace, _))) => {
                // ${ident}
            }
            (_, _) => (),
        }
    }
    for (i, token) in tokens.iter().enumerate() {
        let (peek1, peek2, peek3, peek4) = (
            tokens.get(i + 1),
            tokens.get(i + 2),
            tokens.get(i + 3),
            tokens.get(i + 4),
        );
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
    Ok(output)
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
