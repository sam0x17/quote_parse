pub use quote_parse_macros::*;

quote_parse!(MyThing,
    impl #impl_trait_ident for #impl_target_ident {
        pub fn a_cool_thing() -> #{type1: TypePath};
        pub fn another_cool_thing() -> #{type2: TypePath};
    }

    #{Visibility: viz} struct #struct_ident {
        #field1: #{field1_type: Type},
        #field2: #{field2_type: Type},
        fixed: usize,
    }
);
