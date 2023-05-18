pub use quote_parse_macros::*;

quote_parse!(MyThing,
    impl #impl_trait_ident for #impl_target_ident {
        pub fn a_cool_thing() -> #{TypePath as type1};
        pub fn another_cool_thing() -> #{TypePath as type2};
    }

    #{Visibility as viz} struct #struct_ident {
        #field1: #{Type as field1_type},
        #field2: #{Type as field2_type},
        fixed: usize,
    }
);
