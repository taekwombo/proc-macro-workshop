use quote::quote;
use proc_macro2::{Ident, TokenStream};

pub fn get_checks_module_definition() -> TokenStream {
    quote! {
        pub mod checks {
            pub trait BitfieldSizeMod8<const SIZE: usize> {}
            pub trait TotalSizeIsMultipleOfEightBits: BitfieldSizeMod8<0> {}
        }
    }
}

pub fn impl_checks_traits(target: &Ident, size_mod8_const: &Ident) -> TokenStream { 
    quote! {
        impl bitfield::checks::BitfieldSizeMod8<#size_mod8_const> for #target {}
        impl bitfield::checks::TotalSizeIsMultipleOfEightBits for #target {}
    }
}

