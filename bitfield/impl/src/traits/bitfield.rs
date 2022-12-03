use quote::quote;
use proc_macro2::{Ident, TokenStream};
use syn::TypePath;

pub fn get_bitfield_definition() -> TokenStream {
    quote! {
        pub trait Bitfield<const L: usize> {
            const BIT_INDICES: [[usize; 2]; L];
        }
    }
}

pub fn impl_bitfield(name: &Ident, fields: &[(&Ident, &TypePath)]) -> TokenStream {
    let field_len = fields.len();

    let mut fields_i = fields.iter();
    let (_, first_ty) = fields_i.next().unwrap();

    let mut indices = vec![quote! {
        [0, <#first_ty as bitfield::Specifier>::BITS as usize]
    }];

    let mut bit_offset = quote! {
        (<#first_ty as bitfield::Specifier>::BITS as usize)
    };

    for (_, ty) in fields_i {
        indices.push(quote! {
            , [(#bit_offset), <#ty as bitfield::Specifier>::BITS as usize]
        });
        bit_offset = quote! {
            #bit_offset + (<#ty as bitfield::Specifier>::BITS as usize)
        }
    }

    let bit_indices = TokenStream::from_iter(indices);

    quote! {
        impl bitfield::Bitfield<#field_len> for #name {
            const BIT_INDICES: [[usize; 2]; #field_len] = [#bit_indices];
        }
    }
}
