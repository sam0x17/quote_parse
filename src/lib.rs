pub use quote_parse_macros::*;

quote_parse!(MyThing,
    impl #impl_trait_ident for #impl_target_ident {
        pub fn a_cool_thing() -> #{type1: TypePath};
        pub fn another_cool_thing() -> #{type2: TypePath};
    }

    #{Visibility: viz} struct #struct_ident {
        #field1: #{field1_type: Type},
        fixed: usize,
        // each thing in #!{..} is optional but the presence of the first token requires all
        // subsequent tokens except those nested inside an #?{..}. All tokens inside are
        // generated as Option fields
        // #?{..} is optional
        #!{#field2: #{field2_type: Type} #?{field2_comma: Token![,]}} // note this comma is optional, but field3 will only be parsed if this particular comma is present
        #? if field2_comma.is_some() {
            // conditional branches have the same behavior as #?{..} in terms of tokens being auto-optioned
            #field3: #{field3_type: Type}
        } else {
            // if field2 is missing, we parse this `fixed2` as the 4th field
            fixed2: #{fixed2_type: TypePath}
        }
    }

    // the magic variable `$parser` can be used to dispatch methods to the underlying `ParseStream`
    #? if $parser.peek(Ident) {
        #{goodbye: Option<keywords::goodbye>}
    }
);
