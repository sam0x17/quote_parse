use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Nothing, Parse, ParseBuffer, ParseStream},
    parse2, parse_quote,
    token::{Brace, Bracket, Paren},
    Error, Expr, Ident, Result, Token, Type, Visibility,
};

#[proc_macro]
pub fn quote_parse(tokens: TokenStream) -> TokenStream {
    match quote_parse_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

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
            stream: input.to_token_stream()?,
        })
    }
}

fn quote_parse_internal(tokens: impl Into<TokenStream2>) -> Result<TokenStream2> {
    let args = parse2::<QuoteParseArgs>(tokens.into())?;
    let struct_contents = walk_token_stream(&parse_quote!(input), args.stream)?;
    let struct_name = args.ident;
    let output = quote! {
        struct #struct_name {
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

trait ToTokenStream {
    fn to_token_stream(&self) -> Result<TokenStream2>;
}

impl ToTokenStream for ParseBuffer<'_> {
    fn to_token_stream(&self) -> Result<TokenStream2> {
        let mut contents = TokenStream2::new();
        while !self.is_empty() {
            contents.extend(self.parse::<TokenTree>()?.to_token_stream());
        }
        Ok(contents)
    }
}

struct Walker(TokenStream2);

impl Parse for Walker {
    fn parse(input: ParseStream) -> Result<Self> {
        let parser_ident = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
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
                    } else if input.peek(Paren) {
                        // #(#{var})*
                        let content;
                        parenthesized!(content in input);
                        while !content.is_empty() {
                            if content.peek(Token![#]) {
                                // the interpolation variable
                                content.parse::<Token![#]>()?;
                                // TODO: we need to break up into methods to do this properly
                            }
                            let _token = content.parse::<TokenTree>()?;
                        }
                    } else if input.peek(Token![?]) {
                        // #? ...
                        input.parse::<Token![?]>()?;
                        if input.peek(Ident) {
                            // #?ident
                            let ident = input.parse::<Ident>()?;
                            println!("Option<ident> var: {} ", ident.to_string());
                            continue;
                        } else if input.peek(Brace) {
                            // #?{..}
                            // everything inside is interpreted as a parsing field
                            let content;
                            braced!(content in input);
                            let ident = content.parse::<Ident>()?;
                            content.parse::<Token![:]>()?;
                            let typ = content.parse::<Type>()?;
                            println!(
                                "optional typed var: {}: {} ",
                                ident.to_string(),
                                typ.to_token_stream().to_string(),
                            );
                            if content.peek(Token![,]) {
                                // , ..
                                content.parse::<Token![,]>()?;
                                if content.peek(Token![if]) {
                                    // if: ..
                                    content.parse::<Token![if]>()?;
                                    content.parse::<Token![:]>()?;
                                    let expr = content.parse::<Expr>()?;
                                    content.parse::<Nothing>()?;
                                }
                            }
                        } else if input.peek(Bracket) {
                            // #?[..]
                            // everything inside is interpreted as a recursive entrypoint
                            // peeked off the first item. All items are created as Option
                            // fields, however all are marked as required except those
                            // explicitly marked as ? fields
                            let content;
                            bracketed!(content in input);
                            let content = content.to_token_stream()?;
                            let content = walk_token_stream(&parse_quote!(input), content)?;
                        } else if input.peek(Token![if]) {
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
                                    // TODO: filter _parser into proper parser variable in body
                                    let body = content.to_token_stream()?;
                                    let body = walk_token_stream(&parse_quote!(input), body)?;
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
                            return Err(Error::new(
                                input.span(),
                                "Expected `ident`, `{`, or `if`.",
                            ));
                        }
                        continue;
                    }
                }
            }
            match token {
                TokenTree::Group(group) => {
                    // TODO: process parens/brackets/etc
                    //print!("{}\n", group.delimiter().to_char(true));
                    output.extend(walk_token_stream(&parse_quote!(input), group.stream()));
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

fn walk_token_stream(parser_ident: &Ident, tokens: TokenStream2) -> Result<TokenStream2> {
    match parse2::<Walker>(quote!(#parser_ident, #tokens)) {
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
