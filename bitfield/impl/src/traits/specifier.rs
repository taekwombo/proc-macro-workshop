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

fn is_pow_of_2(v: usize) -> bool {
    v != 0 && (v & (v - 1) == 0)
}

pub fn impl_specifier_for_enum(target: &Ident, variants: &[(&Ident, Option<&Expr>)]) -> TokenStream {
    let len = variants.len();

    if !is_pow_of_2(len) {
        return Error::new(
            Span::call_site(),
            "BitfieldSpecifier expected a number of variants which is a power of 2"
        ).to_compile_error();
    }

    let l2 = len.ilog2();
    let bit_size = if 1 << l2 < len { l2 + 1 } else { l2 };
    let bit_size = match bit_size {
        v @ 0..=64 => v as u8,
        _ => return Error::new_spanned(target, "Enum variant discriminant values should fit in 64 bits.").to_compile_error(),
    };

    quote! {
        impl bitfield::Specifier for #target {
            type InOutType = #target;

            const BITS: u8 = #bit_size;
        }
    }
}
