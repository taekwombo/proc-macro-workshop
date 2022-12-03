use quote::quote;
use proc_macro2::{Ident, Span, TokenStream};

pub fn get_specifier_definition() -> TokenStream {
    quote! {
        pub trait Specifier {
            type InOutType;

            const BITS: u8;
        }
    }
}

pub fn get_types_implementing_specifier() -> TokenStream {
    let types = (1_u8..=64)
        .map(|bit_size| {
            let bit_type_ident = Ident::new(&format!("B{}", bit_size), Span::call_site());
            let upper_bit_size = match bit_size {
                1..=8 => 8,
                9..=16 => 16,
                17..=32 => 32,
                33..=64 => 64,
                _ => unreachable!(),
            };
            let matching_type_ident = Ident::new(&format!("u{}", upper_bit_size), Span::call_site());

            quote! {
                pub struct #bit_type_ident;

                impl Specifier for #bit_type_ident {
                    type InOutType = #matching_type_ident;

                    const BITS: u8 = #bit_size;
                }
            }
        });

    TokenStream::from_iter(types)
}
