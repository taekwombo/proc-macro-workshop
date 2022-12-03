#![feature(once_cell)]

use proc_macro::TokenStream;

mod fields;
mod idents;
mod size_check;
mod methods;
mod traits;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, AttributeArgs, DeriveInput};
    use quote::quote;

    let _args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as DeriveInput);

    let struct_fields = match fields::get_struct_fields(&input) {
        Ok(r) => r,
        Err(e) => return e.into(),
    };

    let bitfield_size = size_check::bitfield_size(&struct_fields);

    let DeriveInput { vis, ident: target, .. } = &input;

    let size_const = idents::size_const(target);
    let size_mod8_const = idents::size_const_mod(target);
    let checks_impl = traits::impl_checks_traits(target, &size_mod8_const);
    let bitfield_impl = traits::impl_bitfield(target, &struct_fields);
    let getters = methods::getters(&struct_fields);
    let setters = methods::setters(&struct_fields);

    quote! {
        const #size_const: usize = #bitfield_size;
        const #size_mod8_const: usize = #size_const % 8;

        #[repr(C)]
        #[derive(Debug)]
        #vis struct #target {
            data: [u8; #size_const / 8],
        }

        #checks_impl

        #bitfield_impl

        impl #target {
            fn new() -> Self {
                Self {
                    data: [0; #size_const / 8],
                }
            }

            #getters
            #setters
        }
    }.into()
}

#[proc_macro]
pub fn bitfield_types(_input: TokenStream) -> TokenStream {
    use quote::quote;

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
