use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;

    unimplemented!()
}

#[proc_macro]
pub fn bitfield_types(_input: TokenStream) -> TokenStream {
    use quote::quote;
    use proc_macro2::{Ident, Span};

    let types = (1_u8..=64)
        .map(|v| (v, Ident::new(&format!("B{}", v), Span::call_site())))
        .map(|(v, i)| quote! {
            pub struct #i;

            impl Specifier for #i {
                const BITS: u8 = #v;
            }
        })
        .collect::<Vec<_>>();

    quote! {
        pub trait Specifier {
            const BITS: u8;
        }

        #(#types)*
    }
    .into()
}
