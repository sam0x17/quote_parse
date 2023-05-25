use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    token::Brace,
    Expr, Ident, Result, Stmt, Token, Type, Visibility,
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
    let output = quote! {
        struct ParsedThing {
            #struct_contents
        }
    };
    println!("output:\n{}", output);
    Ok(output)
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
                // commands
                if t.as_char() == '#' {
                    if input.peek(Ident) {
                        // #ident
                        let ident = input.parse::<Ident>()?;
                        println!("ident var: {} ", ident.to_string());
                        continue;
                    } else if input.peek(Brace) {
                        // #{ident as Type}
                        let content;
                        braced!(content in input);
                        let ident = content.parse::<Ident>()?;
                        content.parse::<Token![:]>()?;
                        let typ = content.parse::<Type>()?;
                        println!(
                            "typed var: {}: {} ",
                            ident.to_string(),
                            typ.to_token_stream().to_string(),
                        );
                        continue;
                    } else if input.peek(Token![?]) {
                        // #? [conditional]
                        input.parse::<Token![?]>()?;
                        if input.peek(Token![if]) {
                            // if chain
                            loop {
                                if input.peek(Token![if]) {
                                    input.parse::<Token![if]>()?;
                                    // output.extend(quote!(if));
                                    // TODO: filter _parser into proper parser variable in expr
                                    let expr = input.parse::<Expr>()?.to_token_stream();
                                    println!("if {} {{", expr.to_token_stream().to_string());
                                    // output.extend(expr);
                                    let content;
                                    braced!(content in input);
                                    let mut body = TokenStream2::new();
                                    // TODO: filter _parser into proper parser variable in body
                                    while !content.is_empty() {
                                        body.extend(
                                            content.parse::<TokenTree>()?.to_token_stream(),
                                        );
                                    }
                                    let body = walk_token_stream(body)?;
                                    println!("}}");
                                    // output.extend(quote!({#body}));
                                }
                                if input.peek(Token![else]) {
                                    input.parse::<Token![else]>()?;
                                    println!("else");
                                } else {
                                    break;
                                }
                            }
                        } else {
                            // match expression
                        }
                        continue;
                    }
                }
            }
            match token {
                TokenTree::Group(group) => {
                    // TODO: process parens/brackets/etc
                    //print!("{}\n", group.delimiter().to_char(true));
                    output.extend(walk_token_stream(group.stream()));
                    //print!("{}\n", group.delimiter().to_char(false));
                }
                TokenTree::Ident(_ident) => (), //print!("{} ", ident.to_string()),
                TokenTree::Punct(punct) => match punct.as_char() {
                    ';' => (), //println!(";"),
                    ',' => (), //println!(","),
                    _ => (),   //print!("{}", punct.as_char()),
                },
                TokenTree::Literal(_lit) => (), //print!("'{}'", lit.to_string()),
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
