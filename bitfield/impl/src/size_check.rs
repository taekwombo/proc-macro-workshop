use quote::quote;
use syn::TypePath;
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
