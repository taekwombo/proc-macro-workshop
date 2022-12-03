use quote::{quote, quote_spanned};
use proc_macro2::{Ident, TokenStream};
use syn::{Expr, parse::Error};

pub fn get_checks_module_definition() -> TokenStream {
    quote! {
        pub mod checks {
            pub trait BitfieldSizeMod8<const SIZE: usize> {}
            pub trait TotalSizeIsMultipleOfEightBits: BitfieldSizeMod8<0> {}
            pub trait EnumDiscriminantInRange<const IN_RANGE: bool> {}
            pub trait DiscriminantInRange: EnumDiscriminantInRange<true> {}
        }
    }
}

pub fn impl_bitfield_checks(name: &Ident, size_mod8_const: &Ident) -> TokenStream { 
    quote_spanned! {
        name.span() =>
        impl bitfield::checks::BitfieldSizeMod8<#size_mod8_const> for #name {}
        impl bitfield::checks::TotalSizeIsMultipleOfEightBits for #name {}
    }
}

pub fn impl_bitfield_specifier_checks(name: &Ident, variants: &[(&Ident, Option<&Expr>)]) -> TokenStream {
    let bit_size = match crate::size::enum_bit_size(variants) {
        Some(v) => v,
        None => return Error::new_spanned(name, "Enum variant discriminant values should fit in 64 bits.").to_compile_error(),
    };
    let max = crate::idents::specifier::max_value(name);
    let max_value: isize = 1 << bit_size;
    let mut impls = vec![quote! {
        const #max: isize = #max_value;
    }];

    let mut prev_discr_value = quote! { 0isize };

    for (idx, (variant, discriminant)) in variants.iter().enumerate() {
        let check_struct = crate::idents::specifier::discriminant_check_struct(name, variant);
        let discr = crate::idents::specifier::discriminant_value(name, variant);
        let discr_ok = crate::idents::specifier::discriminant_in_range(name, variant);

        let discr_value = discriminant
            .map(|v| quote! { #v })
            .unwrap_or_else(|| if idx == 0 {
                prev_discr_value.clone()
            } else {
                quote! { #prev_discr_value + 1 }
            });

        impls.push(quote_spanned! {
            variant.span() => 
            struct #check_struct;
            const #discr: isize = #discr_value;
            const #discr_ok: bool = 0 <= #discr && #discr < #max;
            impl bitfield::checks::EnumDiscriminantInRange<#discr_ok> for #check_struct {}
            impl bitfield::checks::DiscriminantInRange for #check_struct {}
        });

        prev_discr_value = discr_value;
    }

    TokenStream::from_iter(impls)
}

