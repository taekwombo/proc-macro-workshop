#![feature(int_log)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, parse::Error, DeriveInput};

mod fields;
mod idents;
mod size;
mod methods;
mod traits;

#[proc_macro_attribute]
pub fn bits(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return Error::new(Span::call_site(), "#[bitfield] macro does not accept any arguments.")
            .to_compile_error()
            .into();
    }

    let input = parse_macro_input!(input as DeriveInput);

    let (fields, fields_name_type) = match fields::get_struct_fields(&input) {
        Ok(r) => r,
        Err(e) => return e.into(),
    };

    let bitfield_size = size::bitfield_size(&fields_name_type);

    let DeriveInput { vis, ident: name, .. } = &input;

    let size_const = idents::size_const(name);
    let size_mod8_const = idents::size_const_mod(name);
    let checks_impl = traits::impl_bitfield_checks(name, &size_mod8_const);
    let bitfield_impl = traits::impl_bitfield(name, &fields_name_type);
    let getters = methods::getters(&fields_name_type);
    let setters = methods::setters(&fields_name_type);
    let bit_attr_checks = traits::impl_bits_checks(name, fields);

    quote! {
        const #size_const: usize = #bitfield_size;
        const #size_mod8_const: usize = #size_const % 8;

        #[repr(C)]
        #[derive(Debug)]
        #vis struct #name {
            data: [u8; #size_const / 8],
        }

        #bitfield_impl

        impl #name {
            fn new() -> Self {
                Self {
                    data: [0; #size_const / 8],
                }
            }

            #getters
            #setters
        }

        #checks_impl
        #bit_attr_checks
    }.into()
}

#[proc_macro]
pub fn bitfield_types(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        return Error::new(Span::call_site(), "bitfield_types! macro does not accept any input.")
            .to_compile_error()
            .into();
    }

    let specifier = traits::get_specifier_definition();
    let specifier_impl_types = traits::get_types_implementing_specifier();
    let checks_mod = traits::get_checks_module_definition();
    let bitfield = traits::get_bitfield_definition();

    quote! {
        #bitfield
        #checks_mod
        #specifier
        #specifier_impl_types
    }
    .into()
}

#[proc_macro_derive(BitfieldSpecifier, attributes(bits))]
pub fn bitfield_specifier(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let variants = match fields::get_enum_variants(&derive_input) {
        Ok(v) => v,
        Err(e) => return e.into(),
    };
    let specifier_impl = traits::impl_specifier_for_enum(&derive_input.ident, &variants);
    let checks_impl = traits::impl_bitfield_specifier_checks(&derive_input.ident, &variants);

    quote!{
        #specifier_impl
        #checks_impl
    }.into()
}
