use quote::ToTokens;
use syn::parse::Parse;

pub use quote_parse_macros::*;

#[doc(hidden)]
pub mod __private {
    pub use proc_macro2::TokenStream as TokenStream2;
    pub use quote;
    pub use syn;
}

pub struct ParseVec<T>(Vec<T>);

impl<T: Parse> Parse for ParseVec<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items: Vec<T> = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(ParseVec(items))
    }
}

impl<T> From<ParseVec<T>> for Vec<T> {
    fn from(value: ParseVec<T>) -> Self {
        value.0
    }
}

impl<T> From<Vec<T>> for ParseVec<T> {
    fn from(value: Vec<T>) -> Self {
        ParseVec(value)
    }
}

impl<T: Parse + ToTokens> ToTokens for ParseVec<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for item in &self.0 {
            tokens.extend(item.to_token_stream());
        }
    }
}

#[test]
fn test_parse_vec() {
    use quote::quote;
    use syn::Stmt;
    let items: Vec<Stmt> = syn::parse2::<ParseVec<Stmt>>(quote!(hello; hello; hello;))
        .unwrap()
        .into();
    assert_eq!(items.len(), 3);
    assert_eq!(
        items.first().unwrap().to_token_stream().to_string(),
        "hello ;"
    );
    let items: ParseVec<Stmt> = items.into();
    assert_eq!(
        items.to_token_stream().to_string(),
        "hello ; hello ; hello ;"
    );
}

quote_parse!(FunctionDef,
    #{vis: Visibility} fn #ident(#args) #?[-> #{return_type: TypePath}] {
        #{stmts: Vec<Stmt>}
    }
);
// new style:

quote_parse!(ForwardTokensArgs,
    #some_ident,
    #?some_other_ident,
    /// docs about `path`
    #{path: Path},
    /// docs about `target`
    #{target: Path}#?{_comma2: Token![,]}
    #?{mm_path: Path, if: _comma2.is_some()}
    #?{_comma3: Token![,], if: mm_path.is_some()}
    #?{extra: LitStr, if: _comma3.is_some()}
    #?{something: Path, if: input.peek(Path)}
);

// automatically exclude fields beginning with _ from final struct

/*
#[derive(Parse)]
pub struct ForwardTokensArgs {
    /// The path of the item whose tokens are being forwarded
    pub source: Path,
    _comma1: Comma,
    /// The path of the macro that will receive the forwarded tokens
    pub target: Path,
    _comma2: Option<Comma>,
    #[parse_if(_comma2.is_some())]
    pub mm_path: Option<Path>,
    _comma3: Option<Comma>,
    #[parse_if(_comma3.is_some())]
    /// Optional extra data that can be passed as a [`struct@LitStr`]. This is how
    /// [`import_tokens_attr_internal`] passes the item the attribute macro is attached to, but
    /// this can be repurposed for other things potentially as [`str`] could encode anything.
    pub extra: Option<LitStr>,
}

^ implement this one
 */
