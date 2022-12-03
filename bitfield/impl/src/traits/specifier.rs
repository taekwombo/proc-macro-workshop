use quote::quote;
use proc_macro2::{Ident, Span, TokenStream};
use syn::{Expr, parse::Error};

pub fn get_specifier_definition() -> TokenStream {
    quote! {
        pub trait Specifier {
            type InOutType;

            const BITS: u8;
        }
    }
}

pub fn get_types_implementing_specifier() -> TokenStream {
    let bitfield_types = TokenStream::from_iter((1_u8..=64)
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
        }));

    quote! {
        #bitfield_types

        impl Specifier for bool {
            type InOutType = bool;

            const BITS: u8 = 1;
        }
    }
}

pub fn impl_specifier_for_enum(name: &Ident, variants: &[(&Ident, Option<&Expr>)]) -> TokenStream {
    if !crate::size::enum_variants_pow_of_2(variants) {
        return Error::new(
            Span::call_site(),
            "BitfieldSpecifier expected a number of variants which is a power of 2"
        ).to_compile_error();
    }

    let bit_size = match crate::size::enum_bit_size(variants) {
        Some(v) => v,
        None => return Error::new_spanned(name, "Enum variant discriminant values should fit in 64 bits.").to_compile_error(),
    };

    quote! {
        impl bitfield::Specifier for #name {
            type InOutType = #name;

            const BITS: u8 = #bit_size;
        }
    }
}
