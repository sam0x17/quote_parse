use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Nothing, Parse, ParseBuffer, ParseStream},
    parse2, parse_quote,
    token::{Brace, Bracket, Paren, Star},
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

struct ParseState {
    parser_ident: Ident,
}

impl Parse for ParseState {
    fn parse(input: ParseStream) -> Result<Self> {
        let parser_ident = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        Ok(ParseState { parser_ident })
    }
}

struct IdentVar {
    _pound: Token![#],
    ident: Ident,
}

/// Parses `#ident` declarations
fn parse_ident_var(
    input: ParseStream,
    _output: &mut TokenStream2,
    _state: &mut ParseState,
) -> Result<IdentVar> {
    let _pound = input.parse::<Token![#]>()?;
    let ident = input.parse::<Ident>()?;
    println!("ident var: {} ", ident.to_string());
    Ok(IdentVar { _pound, ident })
}

struct TypedVar {
    _pound: Token![#],
    ident: Ident,
    typ: Type,
}

enum Var {
    IdentVar(IdentVar),
    TypedVar(TypedVar),
}

impl Var {
    fn ident(&self) -> &Ident {
        match self {
            Var::IdentVar(ivar) => &ivar.ident,
            Var::TypedVar(tvar) => &tvar.ident,
        }
    }

    fn typ(&self) -> Type {
        match self {
            Var::IdentVar(_) => parse_quote!(syn::Ident),
            Var::TypedVar(tvar) => tvar.typ.clone(),
        }
    }
}

fn parse_var(input: ParseStream, output: &mut TokenStream2, state: &mut ParseState) -> Result<Var> {
    if input.peek(Brace) {
        return Ok(Var::TypedVar(parse_typed_var(input, output, state)?));
    }
    Ok(Var::IdentVar(parse_ident_var(input, output, state)?))
}

/// Parses `#{ident as Type}` declarations
fn parse_typed_var(
    input: ParseStream,
    _output: &mut TokenStream2,
    _state: &mut ParseState,
) -> Result<TypedVar> {
    let _pound = input.parse::<Token![#]>()?;
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
    Ok(TypedVar { _pound, ident, typ })
}

struct Repetition {
    _pound: Token![#],
    _paren: Paren,
    inner_prefix: TokenStream2,
    var: Option<Var>,
    inner_suffix: TokenStream2,
    separator: TokenStream2,
    _star: Star,
}

/// Parses `#(#{var: Type})*`, `#(#{var: Type}),*`, etc. statements
fn parse_repetition(
    input: ParseStream,
    _output: &mut TokenStream2,
    _state: &mut ParseState,
) -> Result<Repetition> {
    let _pound = input.parse::<Token![#]>()?;
    let content;
    let _paren = parenthesized!(content in input);
    let mut var: Option<Var> = None;
    let mut separator: TokenStream2 = TokenStream2::new();
    let mut inner_prefix: TokenStream2 = TokenStream2::new();
    let mut inner_suffix: TokenStream2 = TokenStream2::new();
    while !content.is_empty() {
        if content.peek(Token![#]) {
            // interpolation variable
            if var.is_some() {
                return Err(Error::new(
                    content.span(),
                    "Only one interpolation variable is allowed per repetition.",
                ));
            }
            var = Some(parse_var(input, _output, _state)?);
            continue;
        }
        let token = content.parse::<TokenTree>()?;
        if var.is_none() {
            // prefix
            inner_prefix.extend(token.to_token_stream());
        } else {
            // suffix
            inner_suffix.extend(token.to_token_stream());
        }
    }
    // parse separator
    while !input.is_empty() && !input.peek(Token![*]) {
        separator.extend(input.parse::<TokenTree>()?.to_token_stream());
    }
    let _star = input.parse::<Star>()?;
    Ok(Repetition {
        _pound,
        _paren,
        inner_prefix,
        var,
        inner_suffix,
        separator,
        _star,
    })
}

impl Parse for Walker {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut state = input.parse::<ParseState>()?;
        let mut output: TokenStream2 = TokenStream2::new();
        while !input.is_empty() {
            if input.peek(Token![#]) {
                // commands
                if input.peek2(Ident) {
                    // #ident
                    parse_ident_var(input, &mut output, &mut state)?;
                    continue;
                } else if input.peek2(Brace) {
                    // #{ident as Type}
                    parse_typed_var(input, &mut output, &mut state)?;
                    continue;
                } else if input.peek2(Paren) {
                    // #(#{var})*
                    parse_repetition(input, &mut output, &mut state)?;
                    continue;
                } else if input.peek2(Token![?]) {
                    // #? ...
                    let _pound = input.parse::<Token![#]>()?;
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
                        return Err(Error::new(input.span(), "Expected `ident`, `{`, or `if`."));
                    }
                    continue;
                }
            }
            let token = input.parse::<TokenTree>()?;
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
