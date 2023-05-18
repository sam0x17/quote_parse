pub use quote_parse_macros::*;

quote_parse!(MyThing,
    impl SomeTrait for #Something {
        pub fn a_cool_thing();
        pub fn another_cool_thing() -> #{usize as type_name};
    }
);
