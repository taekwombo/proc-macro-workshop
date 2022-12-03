use quote::quote;
use syn::{TypePath, Expr};
use proc_macro2::{Ident, TokenStream};

fn ty_as_specifier_bits(ty: &TypePath) -> TokenStream {
    quote! {
        (<#ty as bitfield::Specifier>::BITS as usize)
    }
}

pub fn bitfield_size(fields: &[(&Ident, &TypePath)]) -> TokenStream {
    let len = fields.len();

    TokenStream::from_iter(
        fields
            .iter()
            .enumerate()
            .flat_map(|(index, (_, ty))| {
                let mut res = vec![ty_as_specifier_bits(ty)];

                if index + 1 < len {
                    res.push(quote! { + });
                }

                res
            })
    )
}

fn is_pow_of_2(v: usize) -> bool {
    v != 0 && (v & (v - 1) == 0)
}

pub fn enum_variants_pow_of_2(variants: &[(&Ident, Option<&Expr>)]) -> bool {
    is_pow_of_2(variants.len())
}

pub fn enum_bit_size(variants: &[(&Ident, Option<&Expr>)]) -> Option<u8> {
    let count = variants.len();
    let log2 = count.ilog2();

    let bits_needed = if (1 << log2) < count { log2 + 1 } else { log2 };

    match bits_needed {
        v @ 0..=64 => Some(v as u8),
        _ => None,
    }
}
