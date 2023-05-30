pub use quote_parse_macros::*;

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
